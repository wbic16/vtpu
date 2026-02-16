//! Scheduler Redux — Real-Time Feedback Loop
//!
//! "Redux" = State flows in one direction, feedback drives adaptation
//!
//! Flow:
//! 1. Execute sentrons on assigned cores
//! 2. Collect performance metrics (cache misses, stalls, throughput)
//! 3. Feed metrics back to scheduler
//! 4. Scheduler adapts assignments (migrate, rebalance, reprioritize)
//! 5. Repeat
//!
//! Philosophy:
//! - Observe, don't predict
//! - Adapt to reality, not theory
//! - Feedback is the teacher

use crate::runtime_scheduler::{RuntimeScheduler, Assignment};
use crate::exec::ExecStats;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for a sentron
#[derive(Debug, Clone)]
pub struct SentronMetrics {
    pub sentron_id: u16,
    pub core_id: usize,
    pub cpu_id: usize,
    
    /// Operations retired
    pub ops_retired: u64,
    
    /// Cycles elapsed
    pub cycles: u64,
    
    /// Ops per cycle
    pub ops_per_cycle: f64,
    
    /// Cache stats (estimated)
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    
    /// Stall cycles
    pub stall_cycles: u64,
    
    /// Last updated
    pub timestamp: Instant,
}

impl SentronMetrics {
    pub fn from_exec_stats(sentron_id: u16, assignment: &Assignment, stats: &ExecStats) -> Self {
        let ops_per_cycle = if stats.cycles > 0 {
            stats.ops_retired as f64 / stats.cycles as f64
        } else {
            0.0
        };
        
        // Estimate cache stats from PPT hits/misses
        let l1_hits = stats.ppt_stats.ptc_hits;
        let l1_misses = stats.ppt_stats.ptc_misses;
        
        // L2 is harder to estimate without hardware counters
        // For now, assume 90% of L1 misses hit L2
        let l2_hits = (l1_misses as f64 * 0.9) as u64;
        let l2_misses = l1_misses - l2_hits;
        
        Self {
            sentron_id,
            core_id: assignment.core_id,
            cpu_id: assignment.cpu_id,
            ops_retired: stats.ops_retired,
            cycles: stats.cycles,
            ops_per_cycle,
            l1_hits,
            l1_misses,
            l2_hits,
            l2_misses,
            stall_cycles: stats.stalls,
            timestamp: Instant::now(),
        }
    }
    
    /// Cache hit rate (L1 + L2)
    pub fn cache_hit_rate(&self) -> f64 {
        let total_accesses = self.l1_hits + self.l1_misses;
        if total_accesses == 0 {
            return 1.0;
        }
        
        let total_hits = self.l1_hits + self.l2_hits;
        total_hits as f64 / total_accesses as f64
    }
    
    /// Stall rate (fraction of cycles spent stalled)
    pub fn stall_rate(&self) -> f64 {
        if self.cycles == 0 {
            return 0.0;
        }
        self.stall_cycles as f64 / self.cycles as f64
    }
}

/// Aggregate metrics for a physical core
#[derive(Debug, Clone)]
pub struct CoreMetrics {
    pub core_id: usize,
    pub sentron_count: usize,
    
    /// Total ops across all sentrons on this core
    pub total_ops: u64,
    
    /// Total cycles (max across sentrons if SMT)
    pub total_cycles: u64,
    
    /// Aggregate ops/cycle
    pub ops_per_cycle: f64,
    
    /// Average cache hit rate
    pub avg_cache_hit_rate: f64,
    
    /// Average stall rate
    pub avg_stall_rate: f64,
    
    /// Load (sentrons per core)
    pub load: f64,
}

/// Scheduler action based on feedback
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerAction {
    /// No action needed
    NoOp,
    
    /// Migrate sentron to different core
    Migrate {
        sentron_id: u16,
        from_core: usize,
        to_core: usize,
        reason: MigrationReason,
    },
    
    /// Change thread priority
    Reprioritize {
        sentron_id: u16,
        new_priority: i32,
    },
    
    /// Pair sentrons on same core (SMT)
    Pair {
        sentron_a: u16,
        sentron_b: u16,
        target_core: usize,
    },
    
    /// Unpair sentrons (move to separate cores)
    Unpair {
        sentron_a: u16,
        sentron_b: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationReason {
    /// Core overloaded (too many sentrons)
    LoadImbalance,
    
    /// High cache miss rate (bad locality)
    CacheThrash,
    
    /// High stall rate (resource contention)
    Contention,
    
    /// NUMA remote access detected
    NumaPenalty,
}

/// Redux scheduler state
pub struct SchedulerRedux {
    runtime: RuntimeScheduler,
    
    /// Performance history per sentron
    metrics_history: HashMap<u16, Vec<SentronMetrics>>,
    
    /// Core-level aggregates
    core_metrics: HashMap<usize, CoreMetrics>,
    
    /// Actions taken (for audit/debugging)
    action_log: Vec<(Instant, SchedulerAction)>,
    
    /// Thresholds for triggering actions
    config: ReduxConfig,
}

#[derive(Debug, Clone)]
pub struct ReduxConfig {
    /// Migrate if load imbalance exceeds this ratio
    pub load_imbalance_threshold: f64,
    
    /// Migrate if cache hit rate below this
    pub cache_hit_threshold: f64,
    
    /// Migrate if stall rate above this
    pub stall_threshold: f64,
    
    /// Min time between migrations (avoid thrashing)
    pub migration_cooldown: Duration,
    
    /// History window (keep last N samples per sentron)
    pub history_window: usize,
}

impl Default for ReduxConfig {
    fn default() -> Self {
        Self {
            load_imbalance_threshold: 0.5,  // 50% imbalance triggers migration
            cache_hit_threshold: 0.7,        // <70% hit rate = cache thrash
            stall_threshold: 0.3,            // >30% stalls = contention
            migration_cooldown: Duration::from_millis(100),
            history_window: 10,
        }
    }
}

impl SchedulerRedux {
    pub fn new(runtime: RuntimeScheduler, config: ReduxConfig) -> Self {
        Self {
            runtime,
            metrics_history: HashMap::new(),
            core_metrics: HashMap::new(),
            action_log: Vec::new(),
            config,
        }
    }
    
    pub fn with_defaults(runtime: RuntimeScheduler) -> Self {
        Self::new(runtime, ReduxConfig::default())
    }
    
    /// Feed performance metrics back to scheduler
    pub fn feed(&mut self, sentron_id: u16, stats: &ExecStats) {
        // Get assignment
        let assignment = match self.runtime.assignment(sentron_id) {
            Some(a) => a,
            None => return, // Sentron not tracked
        };
        
        // Create metrics
        let metrics = SentronMetrics::from_exec_stats(sentron_id, &assignment, stats);
        
        // Store in history
        let history = self.metrics_history.entry(sentron_id).or_insert_with(Vec::new);
        history.push(metrics);
        
        // Trim to window size
        if history.len() > self.config.history_window {
            history.remove(0);
        }
        
        // Update core-level aggregates
        self.update_core_metrics();
    }
    
    /// Update aggregate metrics for all cores
    fn update_core_metrics(&mut self) {
        self.core_metrics.clear();
        
        for (sentron_id, history) in &self.metrics_history {
            if let Some(latest) = history.last() {
                let core_id = latest.core_id;
                
                let entry = self.core_metrics.entry(core_id).or_insert_with(|| CoreMetrics {
                    core_id,
                    sentron_count: 0,
                    total_ops: 0,
                    total_cycles: 0,
                    ops_per_cycle: 0.0,
                    avg_cache_hit_rate: 0.0,
                    avg_stall_rate: 0.0,
                    load: 0.0,
                });
                
                entry.sentron_count += 1;
                entry.total_ops += latest.ops_retired;
                entry.total_cycles = entry.total_cycles.max(latest.cycles); // SMT: max, not sum
            }
        }
        
        // Calculate averages
        for core_metrics in self.core_metrics.values_mut() {
            if core_metrics.total_cycles > 0 {
                core_metrics.ops_per_cycle = core_metrics.total_ops as f64 / core_metrics.total_cycles as f64;
            }
            
            // Average cache hit rate across sentrons on this core
            let mut total_hit_rate = 0.0;
            let mut total_stall_rate = 0.0;
            let mut count = 0;
            
            for (_, history) in &self.metrics_history {
                if let Some(latest) = history.last() {
                    if latest.core_id == core_metrics.core_id {
                        total_hit_rate += latest.cache_hit_rate();
                        total_stall_rate += latest.stall_rate();
                        count += 1;
                    }
                }
            }
            
            if count > 0 {
                core_metrics.avg_cache_hit_rate = total_hit_rate / count as f64;
                core_metrics.avg_stall_rate = total_stall_rate / count as f64;
            }
            
            core_metrics.load = core_metrics.sentron_count as f64;
        }
    }
    
    /// Analyze metrics and decide on actions
    pub fn decide(&mut self) -> Vec<SchedulerAction> {
        let mut actions = Vec::new();
        
        // Check load imbalance
        if let Some(load_action) = self.check_load_imbalance() {
            actions.push(load_action);
        }
        
        // Check cache thrashing
        for (sentron_id, action) in self.check_cache_thrash() {
            actions.push(action);
        }
        
        // Check contention
        for (sentron_id, action) in self.check_contention() {
            actions.push(action);
        }
        
        // Log actions
        let now = Instant::now();
        for action in &actions {
            self.action_log.push((now, action.clone()));
        }
        
        // Trim log (keep last 100 actions)
        if self.action_log.len() > 100 {
            self.action_log.drain(0..self.action_log.len() - 100);
        }
        
        actions
    }
    
    fn check_load_imbalance(&self) -> Option<SchedulerAction> {
        if self.core_metrics.is_empty() {
            return None;
        }
        
        let max_load = self.core_metrics.values()
            .map(|m| m.load)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        let min_load = self.core_metrics.values()
            .map(|m| m.load)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        if max_load == 0.0 {
            return None;
        }
        
        let imbalance = (max_load - min_load) / max_load;
        
        if imbalance > self.config.load_imbalance_threshold {
            // Find busiest and least-busy cores
            let busiest_core = self.core_metrics.iter()
                .max_by(|(_, a), (_, b)| a.load.partial_cmp(&b.load).unwrap())
                .map(|(id, _)| *id)?;
            
            let least_busy_core = self.core_metrics.iter()
                .min_by(|(_, a), (_, b)| a.load.partial_cmp(&b.load).unwrap())
                .map(|(id, _)| *id)?;
            
            // Find a sentron on busiest core to migrate
            let sentron_to_migrate = self.metrics_history.iter()
                .filter(|(_, h)| h.last().map(|m| m.core_id == busiest_core).unwrap_or(false))
                .map(|(id, _)| *id)
                .next()?;
            
            return Some(SchedulerAction::Migrate {
                sentron_id: sentron_to_migrate,
                from_core: busiest_core,
                to_core: least_busy_core,
                reason: MigrationReason::LoadImbalance,
            });
        }
        
        None
    }
    
    fn check_cache_thrash(&self) -> Vec<(u16, SchedulerAction)> {
        let mut actions = Vec::new();
        
        for (&sentron_id, history) in &self.metrics_history {
            if let Some(latest) = history.last() {
                if latest.cache_hit_rate() < self.config.cache_hit_threshold {
                    // Cache thrashing - consider migrating to less-loaded core
                    if let Some(target_core) = self.find_least_loaded_core_excluding(latest.core_id) {
                        actions.push((sentron_id, SchedulerAction::Migrate {
                            sentron_id,
                            from_core: latest.core_id,
                            to_core: target_core,
                            reason: MigrationReason::CacheThrash,
                        }));
                    }
                }
            }
        }
        
        actions
    }
    
    fn check_contention(&self) -> Vec<(u16, SchedulerAction)> {
        let mut actions = Vec::new();
        
        for (&sentron_id, history) in &self.metrics_history {
            if let Some(latest) = history.last() {
                if latest.stall_rate() > self.config.stall_threshold {
                    // High contention - migrate to less busy core
                    if let Some(target_core) = self.find_least_loaded_core_excluding(latest.core_id) {
                        actions.push((sentron_id, SchedulerAction::Migrate {
                            sentron_id,
                            from_core: latest.core_id,
                            to_core: target_core,
                            reason: MigrationReason::Contention,
                        }));
                    }
                }
            }
        }
        
        actions
    }
    
    fn find_least_loaded_core_excluding(&self, exclude_core: usize) -> Option<usize> {
        self.core_metrics.iter()
            .filter(|(&core_id, _)| core_id != exclude_core)
            .min_by(|(_, a), (_, b)| a.load.partial_cmp(&b.load).unwrap())
            .map(|(id, _)| *id)
    }
    
    /// Get metrics for a sentron
    pub fn sentron_metrics(&self, sentron_id: u16) -> Option<&Vec<SentronMetrics>> {
        self.metrics_history.get(&sentron_id)
    }
    
    /// Get metrics for a core
    pub fn core_metrics(&self, core_id: usize) -> Option<&CoreMetrics> {
        self.core_metrics.get(&core_id)
    }
    
    /// Get action log
    pub fn action_log(&self) -> &[(Instant, SchedulerAction)] {
        &self.action_log
    }
    
    /// Get runtime scheduler
    pub fn runtime(&self) -> &RuntimeScheduler {
        &self.runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_scheduler::RuntimeConfig;
    use crate::topology::*;
    use crate::phext_coord::PhextCoord;
    use crate::ppt::PPTStats;
    
    fn mock_topology() -> CpuTopology {
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
    fn feed_metrics() {
        let runtime = RuntimeScheduler::with_topology(mock_topology(), RuntimeConfig::default());
        let mut redux = SchedulerRedux::with_defaults(runtime);
        
        // Allocate a sentron
        let sentron = redux.runtime.allocate_sentron(0, PhextCoord::zero()).unwrap();
        
        // Feed metrics
        let stats = ExecStats {
            ops_retired: 100,
            cycles: 50,
            stalls: 10,
            ppt_stats: PPTStats {
                ptc_hits: 40,
                ptc_misses: 10,
                ..Default::default()
            },
        };
        
        redux.feed(sentron.id, &stats);
        
        // Check history
        let history = redux.sentron_metrics(sentron.id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].ops_retired, 100);
        assert_eq!(history[0].cycles, 50);
        assert_eq!(history[0].ops_per_cycle, 2.0);
    }
    
    #[test]
    fn detect_load_imbalance() {
        let runtime = RuntimeScheduler::with_topology(mock_topology(), RuntimeConfig::default());
        let mut redux = SchedulerRedux::with_defaults(runtime);
        
        // Allocate 3 sentrons on core 0, none on core 1
        for _ in 0..3 {
            let sentron = redux.runtime.allocate_sentron(0, PhextCoord::zero()).unwrap();
            let stats = ExecStats {
                ops_retired: 100,
                cycles: 50,
                stalls: 5,
                ppt_stats: PPTStats { ptc_hits: 45, ptc_misses: 5, ..Default::default() },
            };
            redux.feed(sentron.id, &stats);
        }
        
        // Update aggregates
        redux.update_core_metrics();
        
        // Decide should detect imbalance
        let actions = redux.decide();
        assert!(!actions.is_empty());
        
        // Should recommend migration
        let migrate = actions.iter().find(|a| matches!(a, SchedulerAction::Migrate { .. }));
        assert!(migrate.is_some());
    }
    
    #[test]
    fn detect_cache_thrash() {
        let runtime = RuntimeScheduler::with_topology(mock_topology(), RuntimeConfig::default());
        let mut redux = SchedulerRedux::with_defaults(runtime);
        
        let sentron = redux.runtime.allocate_sentron(0, PhextCoord::zero()).unwrap();
        
        // Feed metrics with low cache hit rate
        let stats = ExecStats {
            ops_retired: 100,
            cycles: 50,
            stalls: 5,
            ppt_stats: PPTStats {
                ptc_hits: 10,   // Only 10% hit rate
                ptc_misses: 90,
                ..Default::default()
            },
        };
        
        redux.feed(sentron.id, &stats);
        redux.update_core_metrics();
        
        let actions = redux.decide();
        
        // Should detect cache thrash
        let has_cache_action = actions.iter().any(|a| {
            matches!(a, SchedulerAction::Migrate { reason: MigrationReason::CacheThrash, .. })
        });
        
        assert!(has_cache_action, "Should detect cache thrashing");
    }
}
