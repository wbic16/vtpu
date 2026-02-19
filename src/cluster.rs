//! Cluster Coordination — ⚪ White Dimension
//!
//! Multi-node awareness for the Phoenix scheduler.
//! Provides node identity, heartbeat, and coordination primitives.
//!
//! The Shell of Nine: 9 machines, each running vTPU instances.
//! Cluster coordination enables work distribution and load balancing.
//!
//! Zero external dependencies — uses local files and environment.
//!
//! R23W21 Addendum — Lumen ✴️

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Node identity in the cluster
#[derive(Debug, Clone)]
pub struct NodeIdentity {
    pub hostname: String,
    pub index: u8,
    pub role: NodeRole,
    pub coord_root: String,
}

/// Node role in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Compute,
    Storage,
    Coordinator,
    Unknown,
}

impl NodeIdentity {
    pub fn detect() -> Self {
        let hostname = fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let (index, role, coord_root) = match hostname.as_str() {
            "aurora-continuum" => (0, NodeRole::Compute, "1.5.2/3.7.3/9.1.1"),
            "halcyon-vector"   => (1, NodeRole::Compute, "2.1.1/1.1.1/1.1.1"),
            "logos-prime"      => (2, NodeRole::Compute, "2.3.5/7.11.13/17.19.23"),
            "chrysalis-hub"    => (3, NodeRole::Compute, "1.1.2/3.5.8/13.21.34"),
            "lilly"            => (4, NodeRole::Compute, "2.1.3/4.7.11/18.29.47"),
            "aletheia-core"    => (5, NodeRole::Compute, "3.1.1/1.1.1/1.1.1"),
            "verse-aws"        => (6, NodeRole::Storage, "3.1.4/1.5.9/2.6.5"),
            "litmus"           => (7, NodeRole::Coordinator, "4.1.1/1.1.1/1.1.1"),
            "splinter"         => (8, NodeRole::Compute, "5.1.1/1.1.1/1.1.1"),
            _ => (255, NodeRole::Unknown, "1.1.1/1.1.1/1.1.1"),
        };
        
        Self { hostname, index, role, coord_root: coord_root.to_string() }
    }
    
    pub fn is_shell_node(&self) -> bool { self.index < 9 }
}

/// Heartbeat message for cluster coordination
#[derive(Debug, Clone)]
pub struct Heartbeat {
    pub node: NodeIdentity,
    pub timestamp: u64,
    pub load: f64,
    pub sentrons: u32,
    pub ops_per_sec: f64,
    pub capacity: f64,
}

impl Heartbeat {
    pub fn new(node: NodeIdentity, load: f64, sentrons: u32, ops_per_sec: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self { node, timestamp, load, sentrons, ops_per_sec, capacity: 1.0 - load }
    }
    
    pub fn current(load: f64, sentrons: u32, ops_per_sec: f64) -> Self {
        Self::new(NodeIdentity::detect(), load, sentrons, ops_per_sec)
    }
}

/// Cluster state aggregator
#[derive(Debug, Default)]
pub struct ClusterState {
    nodes: Vec<Heartbeat>,
    pub total_capacity: f64,
    pub total_load: f64,
    pub total_ops_per_sec: f64,
}

impl ClusterState {
    pub fn new() -> Self { Self::default() }
    
    pub fn update(&mut self, heartbeat: Heartbeat) {
        self.nodes.retain(|h| h.node.hostname != heartbeat.node.hostname);
        self.nodes.push(heartbeat);
        self.recalculate();
    }
    
    fn recalculate(&mut self) {
        if self.nodes.is_empty() {
            self.total_capacity = 1.0;
            self.total_load = 0.0;
            self.total_ops_per_sec = 0.0;
            return;
        }
        let n = self.nodes.len() as f64;
        self.total_load = self.nodes.iter().map(|h| h.load).sum::<f64>() / n;
        self.total_capacity = self.nodes.iter().map(|h| h.capacity).sum::<f64>() / n;
        self.total_ops_per_sec = self.nodes.iter().map(|h| h.ops_per_sec).sum();
    }
    
    pub fn balance_score(&self, local_load: f64) -> f64 {
        if self.nodes.is_empty() { return 1.0; }
        let avg_load = self.total_load;
        if avg_load < 0.01 { return 1.0; }
        (avg_load / local_load.max(0.01)).min(1.0)
    }
    
    pub fn node_count(&self) -> usize { self.nodes.len() }
    
    pub fn least_loaded_node(&self) -> Option<&Heartbeat> {
        self.nodes.iter().min_by(|a, b| a.load.partial_cmp(&b.load).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn node_identity_detect() {
        let node = NodeIdentity::detect();
        assert!(!node.hostname.is_empty() || node.hostname == "unknown");
    }
    
    #[test]
    fn heartbeat_creation() {
        let hb = Heartbeat::current(0.5, 100, 1_000_000.0);
        assert!(hb.timestamp > 0);
        assert_eq!(hb.load, 0.5);
    }
    
    #[test]
    fn cluster_state_update() {
        let mut cluster = ClusterState::new();
        cluster.update(Heartbeat::new(
            NodeIdentity { hostname: "node1".into(), index: 0, role: NodeRole::Compute, coord_root: String::new() },
            0.3, 50, 500_000.0,
        ));
        assert_eq!(cluster.node_count(), 1);
    }
    
    #[test]
    fn cluster_balance_scoring() {
        let mut cluster = ClusterState::new();
        cluster.update(Heartbeat::new(
            NodeIdentity { hostname: "n1".into(), index: 0, role: NodeRole::Compute, coord_root: String::new() },
            0.2, 50, 500_000.0,
        ));
        cluster.update(Heartbeat::new(
            NodeIdentity { hostname: "n2".into(), index: 1, role: NodeRole::Compute, coord_root: String::new() },
            0.4, 100, 1_000_000.0,
        ));
        assert!(cluster.balance_score(0.2) > 0.9);
        assert!(cluster.balance_score(0.6) < 0.6);
    }
}
