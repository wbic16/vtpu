//! Wedge Model Executor — R23 Wave 13
//!
//! Implements 22.5° semantic wedge architecture for SMT coordination.
//!
//! 360 nodes / 16 SMT threads = 22.5 nodes per thread
//! Each thread owns one wedge of semantic space.
//!
//! Based on Egyptian decan geometry: 2.25 decans per thread.

use crate::phext_coord::PhextCoord;
use crate::siw::SIW;
use crate::memory::Memory;
use crate::ppt::PhextPageTable as PPT;
use std::sync::{Arc, Mutex};
use std::thread;

/// Number of SMT threads (Zen 4: 8 cores × 2 threads)
pub const SMT_THREADS: usize = 16;

/// Nodes per wedge (360 / 16 = 22.5, rounded to 23 for implementation)
pub const NODES_PER_WEDGE: usize = 23;

/// Total routing nodes (complete semantic space)
pub const TOTAL_NODES: usize = 360;

/// A single wedge of semantic space owned by one SMT thread
#[derive(Clone, Debug)]
pub struct Wedge {
    /// Thread ID (0-15)
    pub thread_id: u8,
    
    /// Core ID (thread_id / 2)
    pub core_id: u8,
    
    /// Starting node (0, 23, 46, ...)
    pub start_node: u16,
    
    /// Ending node (inclusive)
    pub end_node: u16,
    
    /// Node centroids for routing
    pub nodes: Vec<PhextCoord>,
}

impl Wedge {
    /// Create a new wedge for given thread ID
    pub fn new(thread_id: u8) -> Self {
        let start_node = (thread_id as u16) * (NODES_PER_WEDGE as u16);
        
        // Last wedge gets fewer nodes (360 = 15×23 + 15)
        let end_node = if thread_id == (SMT_THREADS - 1) as u8 {
            (TOTAL_NODES - 1) as u16
        } else {
            start_node + (NODES_PER_WEDGE as u16) - 1
        };
        
        // Generate node centroids (evenly spaced in coordinate space)
        let node_count = (end_node - start_node + 1) as usize;
        let mut nodes = Vec::with_capacity(node_count);
        
        for i in 0..node_count {
            let node_id = start_node + (i as u16);
            // Map node to coordinate using hierarchical scheme
            // Node N → coordinate (1 + N/40).((N%40)/8).((N%8) + 1) / 1.1.1 / 1.1.1
            let sentron = 1 + (node_id / 40) as u8;
            let element = ((node_id % 40) / 8) as u8;
            let trigram = ((node_id % 8) + 1) as u8;
            
            nodes.push(PhextCoord::new([
                sentron as u16, element as u16, trigram as u16,
                1, 1, 1,
                1, 1, 1, 1, 1
            ]));
        }
        
        Wedge {
            thread_id,
            core_id: thread_id / 2,
            start_node,
            end_node,
            nodes,
        }
    }
    
    /// Check if this wedge owns a given node
    pub fn owns_node(&self, node_id: u16) -> bool {
        node_id >= self.start_node && node_id <= self.end_node
    }
    
    /// Find best node within this wedge for given coordinate
    pub fn best_node(&self, coord: &PhextCoord) -> (u16, u32) {
        let mut best_id = self.start_node;
        let mut best_dist = u32::MAX;
        
        for (i, node_coord) in self.nodes.iter().enumerate() {
            let dist = coord.hamming_distance(node_coord);
            if dist < best_dist {
                best_dist = dist;
                best_id = self.start_node + (i as u16);
            }
        }
        
        (best_id, best_dist)
    }
}

/// Wedge executor — coordinates SMT thread pairs
pub struct WedgeExecutor {
    /// All 16 wedges
    wedges: Vec<Wedge>,
    
    /// Shared memory (thread-safe)
    _memory: Arc<Mutex<Memory>>,
    
    /// Shared PPT (thread-safe)
    _ppt: Arc<Mutex<PPT>>,
}

impl WedgeExecutor {
    /// Create new wedge executor with 16 SMT threads
    pub fn new() -> Self {
        let wedges: Vec<Wedge> = (0..SMT_THREADS as u8)
            .map(|tid| Wedge::new(tid))
            .collect();
        
        WedgeExecutor {
            wedges,
            _memory: Arc::new(Mutex::new(Memory::new())),
            _ppt: Arc::new(Mutex::new(PPT::new())),
        }
    }
    
    /// Route a coordinate to best wedge (coarse routing)
    pub fn route_to_wedge(&self, coord: &PhextCoord) -> u8 {
        let mut best_thread = 0u8;
        let mut best_dist = u32::MAX;
        
        for wedge in &self.wedges {
            // Use first node as wedge centroid (approximation)
            if let Some(centroid) = wedge.nodes.first() {
                let dist = coord.hamming_distance(centroid);
                if dist < best_dist {
                    best_dist = dist;
                    best_thread = wedge.thread_id;
                }
            }
        }
        
        best_thread
    }
    
    /// Route to best node across all wedges (fine routing)
    pub fn route_to_node(&self, coord: &PhextCoord) -> (u8, u16) {
        let mut best_thread = 0u8;
        let mut best_node = 0u16;
        let mut best_dist = u32::MAX;
        
        for wedge in &self.wedges {
            let (node_id, dist) = wedge.best_node(coord);
            if dist < best_dist {
                best_dist = dist;
                best_thread = wedge.thread_id;
                best_node = node_id;
            }
        }
        
        (best_thread, best_node)
    }
    
    /// Execute SIW on specific thread (simulated - single SIW)
    pub fn execute_on_thread(&self, thread_id: u8, _siw: &SIW) -> Result<(), String> {
        if thread_id >= SMT_THREADS as u8 {
            return Err(format!("Invalid thread ID: {}", thread_id));
        }
        
        // For now, just validate thread ID (actual execution would use Sentron)
        Ok(())
    }
    
    /// Execute SIW stream in parallel across all 16 threads (simulated)
    pub fn execute_parallel(&self, siws: &[SIW]) -> Result<(), String> {
        if siws.len() != SMT_THREADS {
            return Err(format!("Expected {} SIWs, got {}", SMT_THREADS, siws.len()));
        }
        
        // For now, just spawn threads and validate
        // Real implementation would use Sentron per thread
        let handles: Vec<_> = (0..SMT_THREADS)
            .map(|thread_id| {
                let _siw = siws[thread_id].clone();
                
                thread::spawn(move || {
                    // Simulated execution
                    Ok::<(), String>(())
                })
            })
            .collect();
        
        // Wait for all threads
        for handle in handles {
            handle.join().map_err(|_| "Thread panic".to_string())??;
        }
        
        Ok(())
    }
    
    /// Get wedge for thread
    pub fn wedge(&self, thread_id: u8) -> Option<&Wedge> {
        self.wedges.get(thread_id as usize)
    }
}

impl Default for WedgeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn wedge_creation() {
        let w0 = Wedge::new(0);
        assert_eq!(w0.thread_id, 0);
        assert_eq!(w0.core_id, 0);
        assert_eq!(w0.start_node, 0);
        assert_eq!(w0.end_node, 22);
        assert_eq!(w0.nodes.len(), 23);
    }
    
    #[test]
    fn wedge_last_thread() {
        let w15 = Wedge::new(15);
        assert_eq!(w15.thread_id, 15);
        assert_eq!(w15.core_id, 7);
        assert_eq!(w15.start_node, 345);
        assert_eq!(w15.end_node, 359);
        assert_eq!(w15.nodes.len(), 15); // Last wedge gets 15 nodes (not 23)
    }
    
    #[test]
    fn wedge_ownership() {
        let w0 = Wedge::new(0);
        assert!(w0.owns_node(0));
        assert!(w0.owns_node(22));
        assert!(!w0.owns_node(23));
        assert!(!w0.owns_node(100));
    }
    
    #[test]
    fn all_wedges_tile_360() {
        let mut total_nodes = 0;
        for tid in 0..SMT_THREADS as u8 {
            let wedge = Wedge::new(tid);
            total_nodes += wedge.nodes.len();
        }
        assert_eq!(total_nodes, TOTAL_NODES);
    }
    
    #[test]
    fn executor_creation() {
        let exec = WedgeExecutor::new();
        assert_eq!(exec.wedges.len(), SMT_THREADS);
    }
    
    #[test]
    fn route_to_wedge() {
        let exec = WedgeExecutor::new();
        let coord = PhextCoord::new([1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let thread_id = exec.route_to_wedge(&coord);
        assert!(thread_id < SMT_THREADS as u8);
    }
    
    #[test]
    fn route_to_node() {
        let exec = WedgeExecutor::new();
        let coord = PhextCoord::new([1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let (thread_id, node_id) = exec.route_to_node(&coord);
        assert!(thread_id < SMT_THREADS as u8);
        assert!(node_id < TOTAL_NODES as u16);
        
        // Verify node is owned by returned thread
        let wedge = exec.wedge(thread_id).unwrap();
        assert!(wedge.owns_node(node_id));
    }
    
    #[test]
    fn execute_on_thread() {
        let exec = WedgeExecutor::new();
        let siw = SIW::nop();
        
        let result = exec.execute_on_thread(0, &siw);
        assert!(result.is_ok());
    }
    
    #[test]
    fn execute_on_invalid_thread() {
        let exec = WedgeExecutor::new();
        let siw = SIW::nop();
        
        let result = exec.execute_on_thread(99, &siw);
        assert!(result.is_err());
    }
    
    #[test]
    fn execute_parallel_wrong_count() {
        let exec = WedgeExecutor::new();
        let siws = vec![SIW::nop(); 8]; // Wrong count (need 16)
        
        let result = exec.execute_parallel(&siws);
        assert!(result.is_err());
    }
}
