//! Runtime Scheduler — Coordinate Sentrons Across CPU Topology
//!
//! Bridges instruction-level scheduling (scheduler.rs) with OS-level
//! thread scheduling (topology.rs + affinity.rs).
//!
//! Philosophy:
//! - Instruction scheduler: Reorder SIWs for maximum ILP
//! - Runtime scheduler: Assign sentrons to physical cores
//! - OS scheduler: Execute threads on SMT siblings
//!
//! Together: End-to-end coordination from source to silicon.

use crate::scheduler::CpuTopology;
use crate::sentron::Sentron;
use crate::smt::SmtPair;
use crate::phext_coord::PhextCoord;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Runtime scheduler configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Use SMT (pair complementary workloads on same core)
    pub use_smt: bool,
    
    /// Number of cores to use (0 = all available)
    pub core_count: usize,
    
    /// NUMA-aware allocation (pin memory to same node as CPU)
    pub numa_aware: bool,
    
    /// Thread priority (0 = normal, higher = more CPU time)
    pub thread_priority: i32,
    
    /// Scheduler policy (normal, batch, realtime)
    pub scheduler_policy: SchedulerPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerPolicy {
    /// SCHED_OTHER (CFS - Completely Fair Scheduler)
    Normal,
    
    /// SCHED_BATCH (throughput-oriented, less preemption)
    Batch,
    
    /// SCHED_FIFO (realtime, requires root)
    RealtimeFifo,
    
    /// SCHED_RR (realtime round-robin, requires root)
    RealtimeRR,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            use_smt: true,
            core_count: 0, // use all
            numa_aware: true,
            thread_priority: 0,
            scheduler_policy: SchedulerPolicy::Batch, // throughput-oriented
        }
    }
}

/// Sentron assignment to physical core
#[derive(Debug, Clone)]
pub struct Assignment {
    pub sentron_id: u16,
    pub core_id: usize,
    pub cpu_id: usize,      // Logical CPU (SMT thread)
    pub numa_node: usize,
}

/// Runtime scheduler state
pub struct RuntimeScheduler {
    topology: CpuTopology,
    config: RuntimeConfig,
    assignments: Arc<Mutex<HashMap<u16, Assignment>>>,
    next_sentron_id: Arc<Mutex<u16>>,
}

impl RuntimeScheduler {
    /// Create runtime scheduler with detected topology
    pub fn new(config: RuntimeConfig) -> std::io::Result<Self> {
        let topology = CpuTopology::detect()?;
        
        Ok(Self {
            topology,
            config,
            assignments: Arc::new(Mutex::new(HashMap::new())),
            next_sentron_id: Arc::new(Mutex::new(0)),
        })
    }
    
    /// Create with explicit topology (for testing)
    pub fn with_topology(topology: CpuTopology, config: RuntimeConfig) -> Self {
        Self {
            topology,
            config,
            assignments: Arc::new(Mutex::new(HashMap::new())),
            next_sentron_id: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Get topology
    pub fn topology(&self) -> &CpuTopology {
        &self.topology
    }
    
    /// Allocate a sentron on a specific core
    pub fn allocate_sentron(&self, core_id: usize, home: PhextCoord) -> Option<Sentron> {
        let core = self.topology.cores.get(core_id)?;
        
        let mut next_id = self.next_sentron_id.lock().unwrap();
        let sentron_id = *next_id;
        *next_id += 1;
        drop(next_id);
        
        // Choose CPU (first logical CPU in core)
        let cpu_id = core.logical_cpus.first().copied()?;
        
        let assignment = Assignment {
            sentron_id,
            core_id,
            cpu_id,
            numa_node: core.numa_node,
        };
        
        self.assignments.lock().unwrap().insert(sentron_id, assignment);
        
        Some(Sentron::new(sentron_id, home, core_id as u8, 0))
    }
    
    /// Allocate SMT pair on a physical core
    pub fn allocate_smt_pair(&self, core_id: usize, home: PhextCoord) -> Option<SmtPair> {
        if !self.config.use_smt {
            return None;
        }
        
        let core = self.topology.cores.get(core_id)?;
        
        if core.logical_cpus.len() < 2 {
            return None; // Core doesn't have SMT
        }
        
        let mut next_id = self.next_sentron_id.lock().unwrap();
        let pair_id = *next_id;
        *next_id += 2; // Reserve two IDs
        drop(next_id);
        
        // Forward on first SMT sibling
        let cpu_fwd = core.logical_cpus[0];
        let assignment_fwd = Assignment {
            sentron_id: pair_id,
            core_id,
            cpu_id: cpu_fwd,
            numa_node: core.numa_node,
        };
        
        // Backward on second SMT sibling
        let cpu_bwd = core.logical_cpus[1];
        let assignment_bwd = Assignment {
            sentron_id: pair_id + 1,
            core_id,
            cpu_id: cpu_bwd,
            numa_node: core.numa_node,
        };
        
        let mut assignments = self.assignments.lock().unwrap();
        assignments.insert(pair_id, assignment_fwd);
        assignments.insert(pair_id + 1, assignment_bwd);
        drop(assignments);
        
        Some(SmtPair::new(pair_id, core_id as u8, home))
    }
    
    /// Allocate sentrons across all available cores
    pub fn allocate_pool(&self, count: usize, home: PhextCoord) -> Vec<Sentron> {
        let mut sentrons = Vec::new();
        let cores_to_use = if self.config.core_count > 0 {
            self.config.core_count.min(self.topology.cores.len())
        } else {
            self.topology.cores.len()
        };
        
        for i in 0..count {
            let core_id = i % cores_to_use;
            if let Some(sentron) = self.allocate_sentron(core_id, home) {
                sentrons.push(sentron);
            }
        }
        
        sentrons
    }
    
    /// Allocate SMT pairs across all available cores
    pub fn allocate_smt_pool(&self, count: usize, home: PhextCoord) -> Vec<SmtPair> {
        let mut pairs = Vec::new();
        let cores_to_use = if self.config.core_count > 0 {
            self.config.core_count.min(self.topology.cores.len())
        } else {
            self.topology.cores.len()
        };
        
        for i in 0..count {
            let core_id = i % cores_to_use;
            if let Some(pair) = self.allocate_smt_pair(core_id, home) {
                pairs.push(pair);
            }
        }
        
        pairs
    }
    
    /// Get CPU assignment for a sentron
    pub fn assignment(&self, sentron_id: u16) -> Option<Assignment> {
        self.assignments.lock().unwrap().get(&sentron_id).cloned()
    }
    
    /// Get recommended CPU for new work (load balancing)
    pub fn recommend_cpu(&self) -> Option<usize> {
        // Simple round-robin for now
        // Future: track load per core, NUMA locality
        let assignments = self.assignments.lock().unwrap();
        let load_per_core: HashMap<usize, usize> = assignments
            .values()
            .fold(HashMap::new(), |mut acc, a| {
                *acc.entry(a.core_id).or_insert(0) += 1;
                acc
            });
        
        // Find least-loaded core
        self.topology
            .cores
            .iter()
            .min_by_key(|c| load_per_core.get(&c.core_id).unwrap_or(&0))
            .and_then(|c| c.logical_cpus.first().copied())
    }
    
    /// Report scheduler stats
    pub fn stats(&self) -> SchedulerStats {
        let assignments = self.assignments.lock().unwrap();
        
        let mut sentrons_per_core: HashMap<usize, usize> = HashMap::new();
        for assignment in assignments.values() {
            *sentrons_per_core.entry(assignment.core_id).or_insert(0) += 1;
        }
        
        let total_sentrons = assignments.len();
        let active_cores = sentrons_per_core.len();
        
        let max_load = sentrons_per_core.values().copied().max().unwrap_or(0);
        let min_load = if sentrons_per_core.is_empty() {
            0
        } else {
            sentrons_per_core.values().copied().min().unwrap_or(0)
        };
        
        let load_balance = if max_load > 0 {
            min_load as f64 / max_load as f64
        } else {
            1.0
        };
        
        SchedulerStats {
            total_sentrons,
            active_cores,
            total_cores: self.topology.cores.len(),
            max_load,
            min_load,
            load_balance,
            smt_enabled: self.topology.has_smt(),
            numa_nodes: self.topology.numa_nodes.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_sentrons: usize,
    pub active_cores: usize,
    pub total_cores: usize,
    pub max_load: usize,      // sentrons on busiest core
    pub min_load: usize,      // sentrons on least-busy core
    pub load_balance: f64,    // 1.0 = perfect, lower = imbalanced
    pub smt_enabled: bool,
    pub numa_nodes: usize,
}

impl std::fmt::Display for SchedulerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Runtime Scheduler Stats:")?;
        writeln!(f, "  Total sentrons: {}", self.total_sentrons)?;
        writeln!(f, "  Active cores: {} / {}", self.active_cores, self.total_cores)?;
        writeln!(f, "  Load: {} - {} sentrons/core", self.min_load, self.max_load)?;
        writeln!(f, "  Load balance: {:.1}% (1.0 = perfect)", self.load_balance * 100.0)?;
        writeln!(f, "  SMT: {}", if self.smt_enabled { "enabled" } else { "disabled" })?;
        writeln!(f, "  NUMA nodes: {}", self.numa_nodes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::CpuTopology;
    
    fn mock_topology_2core_smt() -> CpuTopology {
        CpuTopology {
            cores: vec![
                PhysicalCore {
                    core_id: 0,
                    logical_cpus: vec![0, 2],
                    numa_node: 0,
                },
                PhysicalCore {
                    core_id: 1,
                    logical_cpus: vec![1, 3],
                    numa_node: 0,
                },
            ],
            numa_nodes: vec![NumaNode {
                node_id: 0,
                cpus: vec![0, 1, 2, 3],
            }],
            logical_cpus: 4,
        }
    }
    
    #[test]
    fn allocate_single_sentron() {
        let topology = mock_topology_2core_smt();
        let config = RuntimeConfig::default();
        let sched = RuntimeScheduler::with_topology(topology, config);
        
        let sentron = sched.allocate_sentron(0, PhextCoord::zero()).unwrap();
        assert_eq!(sentron.core_id, 0);
        
        let assignment = sched.assignment(sentron.id).unwrap();
        assert_eq!(assignment.core_id, 0);
        assert_eq!(assignment.cpu_id, 0); // First logical CPU
    }
    
    #[test]
    fn allocate_smt_pair() {
        let topology = mock_topology_2core_smt();
        let config = RuntimeConfig::default();
        let sched = RuntimeScheduler::with_topology(topology, config);
        
        let pair = sched.allocate_smt_pair(0, PhextCoord::zero()).unwrap();
        assert_eq!(pair.core_id, 0);
        
        let fwd_assignment = sched.assignment(pair.forward.id).unwrap();
        let bwd_assignment = sched.assignment(pair.backward.id).unwrap();
        
        assert_eq!(fwd_assignment.core_id, 0);
        assert_eq!(bwd_assignment.core_id, 0);
        assert_ne!(fwd_assignment.cpu_id, bwd_assignment.cpu_id); // Different SMT siblings
    }
    
    #[test]
    fn allocate_pool_round_robin() {
        let topology = mock_topology_2core_smt();
        let config = RuntimeConfig::default();
        let sched = RuntimeScheduler::with_topology(topology, config);
        
        let sentrons = sched.allocate_pool(4, PhextCoord::zero());
        assert_eq!(sentrons.len(), 4);
        
        // Should distribute across 2 cores: 0, 1, 0, 1
        assert_eq!(sentrons[0].core_id, 0);
        assert_eq!(sentrons[1].core_id, 1);
        assert_eq!(sentrons[2].core_id, 0);
        assert_eq!(sentrons[3].core_id, 1);
    }
    
    #[test]
    fn scheduler_stats() {
        let topology = mock_topology_2core_smt();
        let config = RuntimeConfig::default();
        let sched = RuntimeScheduler::with_topology(topology, config);
        
        sched.allocate_pool(4, PhextCoord::zero());
        
        let stats = sched.stats();
        assert_eq!(stats.total_sentrons, 4);
        assert_eq!(stats.active_cores, 2);
        assert_eq!(stats.total_cores, 2);
        assert_eq!(stats.max_load, 2);
        assert_eq!(stats.min_load, 2);
        assert_eq!(stats.load_balance, 1.0); // Perfect balance
    }
    
    #[test]
    fn recommend_cpu_balances_load() {
        let topology = mock_topology_2core_smt();
        let config = RuntimeConfig::default();
        let sched = RuntimeScheduler::with_topology(topology, config);
        
        // Allocate 3 sentrons on core 0
        for _ in 0..3 {
            sched.allocate_sentron(0, PhextCoord::zero());
        }
        
        // Recommend should prefer core 1 (less loaded)
        let cpu = sched.recommend_cpu().unwrap();
        let core_for_cpu = topology.cores.iter()
            .find(|c| c.logical_cpus.contains(&cpu))
            .unwrap();
        assert_eq!(core_for_cpu.core_id, 1);
    }
}
