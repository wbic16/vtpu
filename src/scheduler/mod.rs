//! OS Scheduler Integration — R23W17
//!
//! Coordinates with the operating system scheduler for optimal thread placement.
//! Provides CPU topology discovery, thread affinity, and NUMA awareness.

use std::collections::HashMap;

/// CPU topology information
#[derive(Debug, Clone)]
pub struct CpuTopology {
    /// Number of physical cores
    pub physical_cores: usize,
    
    /// Total number of hardware threads (including SMT)
    pub hardware_threads: usize,
    
    /// SMT sibling pairs: (physical_thread, smt_sibling)
    /// For single-threaded cores, sibling = None
    pub siblings: Vec<(usize, Option<usize>)>,
    
    /// NUMA nodes (empty if single-node system)
    pub numa_nodes: Vec<NumaNode>,
    
    /// Cache hierarchy
    pub cache: CacheInfo,
}

/// NUMA node information
#[derive(Debug, Clone)]
pub struct NumaNode {
    pub id: usize,
    pub cpus: Vec<usize>,
    pub memory_gb: f64,
}

/// Cache hierarchy
#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub l1d_kb: usize,  // L1 data cache per core
    pub l1i_kb: usize,  // L1 instruction cache per core
    pub l2_kb: usize,   // L2 cache per core
    pub l3_kb: usize,   // L3 cache (shared)
}

impl CpuTopology {
    /// Discover CPU topology on the current system
    pub fn discover() -> Self {
        #[cfg(target_os = "linux")]
        return Self::discover_linux();
        
        #[cfg(target_os = "macos")]
        return Self::discover_macos();
        
        #[cfg(target_os = "windows")]
        return Self::discover_windows();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return Self::discover_fallback();
    }
    
    /// Linux-specific topology discovery via /proc and /sys
    #[cfg(target_os = "linux")]
    fn discover_linux() -> Self {
        use std::fs;
        use std::path::Path;
        
        // Get hardware thread count
        let hardware_threads = num_cpus::get();
        
        // Discover SMT siblings via /sys/devices/system/cpu/cpu*/topology/thread_siblings_list
        let mut siblings = Vec::new();
        let mut sibling_map: HashMap<usize, usize> = HashMap::new();
        
        for cpu in 0..hardware_threads {
            let siblings_path = format!("/sys/devices/system/cpu/cpu{}/topology/thread_siblings_list", cpu);
            
            if let Ok(content) = fs::read_to_string(&siblings_path) {
                let sibling_ids: Vec<usize> = content
                    .trim()
                    .split(',')
                    .filter_map(|s| s.parse().ok())
                    .collect();
                
                // If there are 2 siblings, this is SMT
                if sibling_ids.len() == 2 && sibling_ids.contains(&cpu) {
                    let other = sibling_ids.iter().find(|&&id| id != cpu).copied();
                    sibling_map.insert(cpu, other.unwrap_or(cpu));
                }
            }
        }
        
        // Build siblings list
        for cpu in 0..hardware_threads {
            if let Some(&sibling) = sibling_map.get(&cpu) {
                if sibling != cpu {
                    siblings.push((cpu, Some(sibling)));
                } else {
                    siblings.push((cpu, None));
                }
            } else {
                siblings.push((cpu, None));
            }
        }
        
        // Count physical cores (CPUs without duplicates via siblings)
        let physical_cores = siblings.iter().filter(|(cpu, sib)| {
            sib.map_or(true, |s| *cpu < s)
        }).count();
        
        // Detect NUMA nodes
        let numa_nodes = Self::discover_numa_linux();
        
        // Detect cache info
        let cache = Self::discover_cache_linux();
        
        CpuTopology {
            physical_cores,
            hardware_threads,
            siblings,
            numa_nodes,
            cache,
        }
    }
    
    #[cfg(target_os = "linux")]
    fn discover_numa_linux() -> Vec<NumaNode> {
        use std::fs;
        
        let mut nodes = Vec::new();
        
        // Check /sys/devices/system/node/node*/cpulist
        for entry in fs::read_dir("/sys/devices/system/node").ok().into_iter().flatten() {
            let entry = entry.ok();
            if entry.is_none() {
                continue;
            }
            
            let entry = entry.unwrap();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if !name_str.starts_with("node") {
                continue;
            }
            
            // Extract node ID
            let node_id: usize = name_str.trim_start_matches("node").parse().unwrap_or(0);
            
            // Read CPU list
            let cpulist_path = entry.path().join("cpulist");
            let cpus = if let Ok(content) = fs::read_to_string(&cpulist_path) {
                Self::parse_cpu_list(&content)
            } else {
                Vec::new()
            };
            
            // Try to read memory info (in KB)
            let meminfo_path = entry.path().join("meminfo");
            let memory_gb = if let Ok(content) = fs::read_to_string(&meminfo_path) {
                // Parse "Node N MemTotal: XXXXX kB"
                content
                    .lines()
                    .find(|l| l.contains("MemTotal"))
                    .and_then(|l| l.split_whitespace().nth(3))
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|kb| kb / 1024.0 / 1024.0)  // KB to GB
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            
            nodes.push(NumaNode {
                id: node_id,
                cpus,
                memory_gb,
            });
        }
        
        nodes
    }
    
    #[cfg(target_os = "linux")]
    fn parse_cpu_list(list: &str) -> Vec<usize> {
        let mut cpus = Vec::new();
        
        for part in list.trim().split(',') {
            if part.contains('-') {
                // Range: "0-3"
                let range: Vec<&str> = part.split('-').collect();
                if range.len() == 2 {
                    if let (Ok(start), Ok(end)) = (range[0].parse::<usize>(), range[1].parse::<usize>()) {
                        cpus.extend(start..=end);
                    }
                }
            } else {
                // Single CPU
                if let Ok(cpu) = part.parse::<usize>() {
                    cpus.push(cpu);
                }
            }
        }
        
        cpus
    }
    
    #[cfg(target_os = "linux")]
    fn discover_cache_linux() -> CacheInfo {
        use std::fs;
        
        // Try to read from /sys/devices/system/cpu/cpu0/cache/index*/size
        let mut l1d_kb = 32;  // Default for Zen 4
        let mut l1i_kb = 32;
        let mut l2_kb = 1024;
        let mut l3_kb = 32768;
        
        for level in 0..4 {
            let size_path = format!("/sys/devices/system/cpu/cpu0/cache/index{}/size", level);
            let type_path = format!("/sys/devices/system/cpu/cpu0/cache/index{}/type", level);
            
            if let (Ok(size_str), Ok(type_str)) = (fs::read_to_string(&size_path), fs::read_to_string(&type_path)) {
                let size_kb = size_str.trim().trim_end_matches('K').parse::<usize>().unwrap_or(0);
                let cache_type = type_str.trim();
                
                if cache_type == "Data" && size_kb > 0 && size_kb < 64 {
                    l1d_kb = size_kb;
                } else if cache_type == "Instruction" && size_kb > 0 && size_kb < 64 {
                    l1i_kb = size_kb;
                } else if cache_type == "Unified" && size_kb > 64 && size_kb < 2048 {
                    l2_kb = size_kb;
                } else if cache_type == "Unified" && size_kb >= 2048 {
                    l3_kb = size_kb;
                }
            }
        }
        
        CacheInfo { l1d_kb, l1i_kb, l2_kb, l3_kb }
    }
    
    /// macOS-specific topology (no SMT detection, best-effort)
    #[cfg(target_os = "macos")]
    fn discover_macos() -> Self {
        let hardware_threads = num_cpus::get();
        let physical_cores = num_cpus::get_physical();
        
        // Best guess: if HT > PC, assume 2-way SMT
        let siblings = if hardware_threads > physical_cores {
            (0..hardware_threads)
                .map(|cpu| {
                    let sibling = if cpu < physical_cores {
                        Some(cpu + physical_cores)
                    } else {
                        Some(cpu - physical_cores)
                    };
                    (cpu, sibling)
                })
                .collect()
        } else {
            (0..hardware_threads).map(|cpu| (cpu, None)).collect()
        };
        
        CpuTopology {
            physical_cores,
            hardware_threads,
            siblings,
            numa_nodes: Vec::new(),  // macOS is typically single-node
            cache: CacheInfo {
                l1d_kb: 64,   // Typical for Apple Silicon
                l1i_kb: 128,
                l2_kb: 4096,
                l3_kb: 0,     // Varies by model
            },
        }
    }
    
    /// Windows-specific topology
    #[cfg(target_os = "windows")]
    fn discover_windows() -> Self {
        let hardware_threads = num_cpus::get();
        let physical_cores = num_cpus::get_physical();
        
        // TODO: Use GetLogicalProcessorInformation for real topology
        // For now, assume 2-way SMT like macOS
        let siblings = if hardware_threads > physical_cores {
            (0..hardware_threads)
                .map(|cpu| {
                    let sibling = if cpu < physical_cores {
                        Some(cpu + physical_cores)
                    } else {
                        Some(cpu - physical_cores)
                    };
                    (cpu, sibling)
                })
                .collect()
        } else {
            (0..hardware_threads).map(|cpu| (cpu, None)).collect()
        };
        
        CpuTopology {
            physical_cores,
            hardware_threads,
            siblings,
            numa_nodes: Vec::new(),
            cache: CacheInfo {
                l1d_kb: 32,
                l1i_kb: 32,
                l2_kb: 512,
                l3_kb: 8192,
            },
        }
    }
    
    /// Fallback for unsupported platforms
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn discover_fallback() -> Self {
        let hardware_threads = num_cpus::get();
        
        CpuTopology {
            physical_cores: hardware_threads,
            hardware_threads,
            siblings: (0..hardware_threads).map(|cpu| (cpu, None)).collect(),
            numa_nodes: Vec::new(),
            cache: CacheInfo {
                l1d_kb: 32,
                l1i_kb: 32,
                l2_kb: 256,
                l3_kb: 2048,
            },
        }
    }
    
    /// Get the SMT sibling of a given CPU
    pub fn smt_sibling(&self, cpu: usize) -> Option<usize> {
        self.siblings.get(cpu).and_then(|(_, sib)| *sib)
    }
    
    /// Check if this system has SMT enabled
    pub fn has_smt(&self) -> bool {
        self.hardware_threads > self.physical_cores
    }
}

/// Pin current thread to specific CPU (Linux only, best-effort on others)
#[cfg(target_os = "linux")]
pub fn pin_to_cpu(cpu_id: usize) -> Result<(), String> {
    unsafe {
        let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_ZERO(&mut cpu_set);
        libc::CPU_SET(cpu_id, &mut cpu_set);
        
        if libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set) != 0 {
            return Err(format!("Failed to set CPU affinity to {}", cpu_id));
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn pin_to_cpu(_cpu_id: usize) -> Result<(), String> {
    Err("CPU pinning not supported on this platform".to_string())
}

/// Set thread name (for profiling/debugging)
pub fn set_thread_name(name: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let cname = std::ffi::CString::new(name).map_err(|e| format!("Invalid thread name: {}", e))?;
        unsafe {
            if libc::pthread_setname_np(libc::pthread_self(), cname.as_ptr()) != 0 {
                return Err("Failed to set thread name".to_string());
            }
        }
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    {
        let cname = std::ffi::CString::new(name).map_err(|e| format!("Invalid thread name: {}", e))?;
        unsafe {
            if libc::pthread_setname_np(cname.as_ptr()) != 0 {
                return Err("Failed to set thread name".to_string());
            }
        }
        Ok(())
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = name;
        Err("Thread naming not supported on this platform".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn topology_discovery_works() {
        let topo = CpuTopology::discover();
        
        assert!(topo.hardware_threads > 0);
        assert!(topo.physical_cores > 0);
        assert!(topo.physical_cores <= topo.hardware_threads);
        assert_eq!(topo.siblings.len(), topo.hardware_threads);
    }
    
    #[test]
    fn smt_sibling_lookup() {
        let topo = CpuTopology::discover();
        
        if topo.has_smt() {
            // If SMT is enabled, check first CPU has a sibling
            let sibling = topo.smt_sibling(0);
            assert!(sibling.is_some());
            
            // Sibling should be different from self
            assert_ne!(sibling.unwrap(), 0);
        }
    }
}
