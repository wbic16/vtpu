//! CPU Topology Detection
//!
//! Detects physical cores, SMT siblings, and NUMA nodes by parsing /sys.
//! Zero external dependencies — just filesystem reads.
//!
//! Usage:
//! ```
//! let topology = CpuTopology::detect()?;
//! println!("Physical cores: {}", topology.cores.len());
//! for (cpu0, cpu1) in topology.smt_pairs() {
//!     println!("SMT pair: CPU {} + CPU {}", cpu0, cpu1);
//! }
//! ```

use std::fs;
use std::io;
use std::collections::{HashMap, HashSet};

/// CPU topology: physical cores, SMT siblings, NUMA nodes
#[derive(Debug, Clone)]
pub struct CpuTopology {
    /// Physical cores (each has 1+ logical CPUs)
    pub cores: Vec<PhysicalCore>,
    /// NUMA nodes (memory locality)
    pub numa_nodes: Vec<NumaNode>,
    /// Logical CPU count
    pub logical_cpus: usize,
}

#[derive(Debug, Clone)]
pub struct PhysicalCore {
    /// Physical core ID (unique across system)
    pub core_id: usize,
    /// Logical CPU IDs (SMT siblings)
    pub logical_cpus: Vec<usize>,
    /// NUMA node this core belongs to
    pub numa_node: usize,
}

#[derive(Debug, Clone)]
pub struct NumaNode {
    /// NUMA node ID
    pub node_id: usize,
    /// CPUs in this node
    pub cpus: Vec<usize>,
}

impl CpuTopology {
    /// Detect topology from /sys/devices/system/cpu/
    pub fn detect() -> io::Result<Self> {
        let cpu_dir = "/sys/devices/system/cpu";
        
        // Find all online CPUs
        let mut online_cpus = Vec::new();
        
        for entry in fs::read_dir(cpu_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if name_str.starts_with("cpu") && name_str[3..].chars().all(|c| c.is_ascii_digit()) {
                if let Ok(cpu_id) = name_str[3..].parse::<usize>() {
                    // Check if online
                    let online_path = format!("{}/cpu{}/online", cpu_dir, cpu_id);
                    
                    // CPU0 may not have /online file (always online)
                    let is_online = if cpu_id == 0 {
                        true
                    } else {
                        match fs::read_to_string(&online_path) {
                            Ok(content) => content.trim() == "1",
                            Err(_) => true, // If file doesn't exist, assume online
                        }
                    };
                    
                    if is_online {
                        online_cpus.push(cpu_id);
                    }
                }
            }
        }
        
        online_cpus.sort_unstable();
        
        if online_cpus.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No online CPUs detected"
            ));
        }
        
        // Group CPUs by physical core
        let mut core_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut numa_map: HashMap<usize, Vec<usize>> = HashMap::new();
        
        for &cpu in &online_cpus {
            // Read core_id
            let core_id_path = format!("{}/cpu{}/topology/core_id", cpu_dir, cpu);
            let core_id = match fs::read_to_string(&core_id_path) {
                Ok(content) => content.trim().parse::<usize>().unwrap_or(cpu), // Fallback to cpu ID
                Err(_) => cpu, // No topology info, treat as separate core
            };
            
            core_map.entry(core_id).or_insert_with(Vec::new).push(cpu);
            
            // Read NUMA node (physical_package_id)
            let numa_path = format!("{}/cpu{}/topology/physical_package_id", cpu_dir, cpu);
            let numa_node = match fs::read_to_string(&numa_path) {
                Ok(content) => content.trim().parse::<usize>().unwrap_or(0),
                Err(_) => 0, // Default to node 0
            };
            
            numa_map.entry(numa_node).or_insert_with(Vec::new).push(cpu);
        }
        
        // Build PhysicalCore structs
        let mut cores: Vec<PhysicalCore> = core_map
            .into_iter()
            .map(|(core_id, mut logical_cpus)| {
                logical_cpus.sort_unstable();
                
                // Determine NUMA node (majority vote)
                let numa_node = logical_cpus
                    .iter()
                    .filter_map(|&cpu| {
                        let path = format!("{}/cpu{}/topology/physical_package_id", cpu_dir, cpu);
                        fs::read_to_string(&path)
                            .ok()
                            .and_then(|s| s.trim().parse::<usize>().ok())
                    })
                    .next()
                    .unwrap_or(0);
                
                PhysicalCore {
                    core_id,
                    logical_cpus,
                    numa_node,
                }
            })
            .collect();
        
        cores.sort_by_key(|c| c.core_id);
        
        // Build NumaNode structs
        let mut numa_nodes: Vec<NumaNode> = numa_map
            .into_iter()
            .map(|(node_id, mut cpus)| {
                cpus.sort_unstable();
                NumaNode { node_id, cpus }
            })
            .collect();
        
        numa_nodes.sort_by_key(|n| n.node_id);
        
        Ok(CpuTopology {
            cores,
            numa_nodes,
            logical_cpus: online_cpus.len(),
        })
    }
    
    /// Get SMT sibling pairs (for complementary workloads)
    /// Returns (cpu0, cpu1) where cpu0 and cpu1 share a physical core
    pub fn smt_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        
        for core in &self.cores {
            if core.logical_cpus.len() >= 2 {
                // Pair first two SMT siblings
                pairs.push((core.logical_cpus[0], core.logical_cpus[1]));
            }
        }
        
        pairs
    }
    
    /// Get all SMT siblings for a given CPU
    pub fn smt_siblings(&self, cpu: usize) -> Vec<usize> {
        for core in &self.cores {
            if core.logical_cpus.contains(&cpu) {
                return core.logical_cpus.clone();
            }
        }
        vec![cpu] // No siblings found, return self
    }
    
    /// Check if system has SMT enabled (any core with >1 logical CPU)
    pub fn has_smt(&self) -> bool {
        self.cores.iter().any(|c| c.logical_cpus.len() > 1)
    }
    
    /// Get physical core count
    pub fn physical_cores(&self) -> usize {
        self.cores.len()
    }
    
    /// Get SMT width (threads per core)
    /// Returns max across all cores (usually 2 for SMT2, 1 if SMT disabled)
    pub fn smt_width(&self) -> usize {
        self.cores
            .iter()
            .map(|c| c.logical_cpus.len())
            .max()
            .unwrap_or(1)
    }
}

impl std::fmt::Display for CpuTopology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CPU Topology:")?;
        writeln!(f, "  Physical cores: {}", self.physical_cores())?;
        writeln!(f, "  Logical CPUs: {}", self.logical_cpus)?;
        writeln!(f, "  SMT width: {}", self.smt_width())?;
        writeln!(f)?;
        
        for core in &self.cores {
            writeln!(
                f,
                "  Core {}: CPUs {:?} (NUMA node {})",
                core.core_id, core.logical_cpus, core.numa_node
            )?;
        }
        
        writeln!(f)?;
        writeln!(f, "NUMA Nodes:")?;
        for node in &self.numa_nodes {
            writeln!(f, "  Node {}: {} CPUs", node.node_id, node.cpus.len())?;
        }
        
        if self.has_smt() {
            writeln!(f)?;
            writeln!(f, "SMT Pairs:")?;
            for (i, (cpu0, cpu1)) in self.smt_pairs().iter().enumerate() {
                writeln!(f, "  Pair {}: CPU {} + CPU {}", i, cpu0, cpu1)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn detect_topology_basic() {
        // This test will fail if /sys is unavailable (e.g., non-Linux)
        // Run on Linux only
        #[cfg(target_os = "linux")]
        {
            let topology = CpuTopology::detect().expect("Failed to detect topology");
            
            assert!(topology.logical_cpus > 0, "Should detect at least 1 CPU");
            assert!(!topology.cores.is_empty(), "Should have at least 1 core");
            assert!(!topology.numa_nodes.is_empty(), "Should have at least 1 NUMA node");
            
            println!("{}", topology);
        }
    }
    
    #[test]
    fn smt_pairs_valid() {
        #[cfg(target_os = "linux")]
        {
            let topology = CpuTopology::detect().expect("Failed to detect topology");
            
            for (cpu0, cpu1) in topology.smt_pairs() {
                // Siblings should be different
                assert_ne!(cpu0, cpu1, "SMT pair should have distinct CPUs");
                
                // Both should belong to same core
                let siblings0 = topology.smt_siblings(cpu0);
                let siblings1 = topology.smt_siblings(cpu1);
                assert_eq!(siblings0, siblings1, "SMT siblings should belong to same core");
            }
        }
    }
}
