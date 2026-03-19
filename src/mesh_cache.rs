//! Mesh-Aware Global Cache Coordination
//!
//! Views cache across all nodes as a single logical resource.
//! Each node publishes its cache heat map to SQ. Other nodes read
//! this metadata to route tasks organically — if node A has dimension 3
//! warm in L1, route coordinate-3-heavy tasks to node A.
//!
//! Architecture:
//!   - Each node publishes CacheHeatMap to SQ at a known coordinate
//!   - Task router reads all heat maps, scores nodes per task
//!   - Prefetch hints propagate: if node A sees a miss, it notifies
//!     the mesh — other nodes prefetch proactively
//!
//! SQ coordinates:
//!   cache-heat/<node_idx>/1.1.1/1.1.1  — per-dimension hit rates
//!   cache-miss-alert/1.1.1/1.1.1/1.1.1 — hot miss coordinates (broadcast)
//!   task-route/<task_hash>/1.1.1/1.1.1  — routing decision + rationale

use crate::cache_sim::CacheSimulator;
use crate::phext_coord::PhextCoord;
use crate::prefetch::{DimensionalPrefetcher, PrefetchStrategy};
use std::collections::HashMap;

/// Per-dimension cache heat: hit rates across 11 phext dimensions
#[derive(Debug, Clone, Default)]
pub struct DimHeat {
    pub dim: usize,
    pub l1_rate: f64,    // fraction of accesses hitting L1
    pub l2_rate: f64,    // fraction hitting L2
    pub access_count: u64,
}

/// Node cache heat map — what this node has warm in each dimension
#[derive(Debug, Clone)]
pub struct CacheHeatMap {
    pub node_idx: u16,
    pub node_name: String,
    pub dims: [DimHeat; 11],
    pub overall_l1_rate: f64,
    pub overall_l2_rate: f64,
    pub hottest_dim: usize,     // dimension with best L1 hit rate
    pub coldest_dim: usize,     // dimension with most misses (prefetch target)
    pub total_accesses: u64,
    pub timestamp_secs: u64,
}

impl CacheHeatMap {
    /// Build from a running CacheSimulator
    pub fn from_sim(node_idx: u16, node_name: &str, sim: &CacheSimulator) -> Self {
        let mut dims = std::array::from_fn(|i| DimHeat {
            dim: i,
            l1_rate: 0.0,
            l2_rate: 0.0,
            access_count: 0,
        });

        let mut hottest_dim = 0;
        let mut coldest_dim = 0;
        let mut best_l1 = 0.0f64;
        let mut worst_l1 = 1.0f64;

        for i in 0..11 {
            let ds = sim.dim_stats(i);
            let total = ds.accesses.max(1) as f64;
            dims[i].dim = i;
            dims[i].l1_rate = ds.l1_hits as f64 / total;
            dims[i].l2_rate = ds.l2_hits as f64 / total;
            dims[i].access_count = ds.accesses;

            if dims[i].l1_rate > best_l1 && ds.accesses > 0 {
                best_l1 = dims[i].l1_rate;
                hottest_dim = i;
            }
            if dims[i].l1_rate < worst_l1 && ds.accesses > 0 {
                worst_l1 = dims[i].l1_rate;
                coldest_dim = i;
            }
        }

        CacheHeatMap {
            node_idx,
            node_name: node_name.to_string(),
            dims,
            overall_l1_rate: sim.l1_hit_rate(),
            overall_l2_rate: sim.total_l2() as f64 / sim.total_accesses().max(1) as f64,
            hottest_dim,
            coldest_dim,
            total_accesses: sim.total_accesses(),
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Serialize to SQ-publishable string
    pub fn to_sq_value(&self) -> String {
        let dim_summary: Vec<String> = self.dims.iter()
            .filter(|d| d.access_count > 0)
            .map(|d| format!("d{}:{:.2}/{:.2}", d.dim, d.l1_rate, d.l2_rate))
            .collect();
        format!(
            "node:{} idx:{} l1:{:.3} l2:{:.3} hot_dim:{} cold_dim:{} accesses:{} dims:[{}] ts:{}",
            self.node_name, self.node_idx,
            self.overall_l1_rate, self.overall_l2_rate,
            self.hottest_dim, self.coldest_dim,
            self.total_accesses,
            dim_summary.join(","),
            self.timestamp_secs,
        )
    }

    /// Score this node for executing a task touching given dimensions.
    /// Higher = better (warmer cache for this task's working set).
    pub fn score_for_task(&self, task_dims: &[usize]) -> f64 {
        if task_dims.is_empty() { return self.overall_l1_rate; }
        let score: f64 = task_dims.iter()
            .map(|&d| {
                if d < 11 && self.dims[d].access_count > 0 {
                    // Weight: L1 = 1.0, L2 = 0.5, cold = 0.0
                    self.dims[d].l1_rate + self.dims[d].l2_rate * 0.5
                } else {
                    0.0 // cold — no advantage
                }
            })
            .sum::<f64>() / task_dims.len() as f64;
        score
    }
}

/// Task routing decision — which node is best for a given task
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub task_hash: u64,
    pub best_node: u16,
    pub best_node_name: String,
    pub score: f64,
    pub reason: String,
    pub alternatives: Vec<(u16, f64)>,
}

/// Global cache router — reads heat maps from SQ and routes tasks
pub struct MeshCacheRouter {
    heat_maps: HashMap<u16, CacheHeatMap>,
    local_sim: CacheSimulator,
    local_prefetcher: DimensionalPrefetcher,
    pub node_idx: u16,
    pub node_name: String,
    // SQ connection details
    sq_host: String,
    sq_port: u16,
}

impl MeshCacheRouter {
    /// Create a new router. Call `sync_from_mesh()` to populate heat maps.
    pub fn new(node_idx: u16, node_name: &str, sq_host: &str, sq_port: u16) -> Self {
        Self {
            heat_maps: HashMap::new(),
            local_sim: CacheSimulator::default_sizes(),
            local_prefetcher: DimensionalPrefetcher::new(256, PrefetchStrategy::Topology),
            node_idx,
            node_name: node_name.to_string(),
            sq_host: sq_host.to_string(),
            sq_port,
        }
    }

    /// Record a local cache access and update prefetcher
    pub fn access(&mut self, coord: &PhextCoord) -> (crate::cache_sim::CacheLevel, Vec<PhextCoord>) {
        let level = self.local_sim.access(coord);
        let predictions = self.local_prefetcher.access(coord);
        // Prewarm predicted coordinates in simulator
        for pred in &predictions {
            self.local_sim.prewarm(pred);
        }
        (level, predictions)
    }

    /// Publish local heat map to all mesh SQ nodes.
    /// Call this periodically (e.g., every 10K accesses or 30s).
    pub fn publish_heat_map(&self) -> String {
        let heat_map = CacheHeatMap::from_sim(
            self.node_idx,
            &self.node_name,
            &self.local_sim,
        );
        heat_map.to_sq_value()
    }

    /// Ingest a heat map from another node (parsed from SQ)
    pub fn ingest_heat_map(&mut self, map: CacheHeatMap) {
        self.heat_maps.insert(map.node_idx, map);
    }

    /// Route a task to the best node for its working set.
    /// task_dims: which phext dimensions the task accesses heavily.
    pub fn route_task(&self, task_hash: u64, task_dims: &[usize]) -> RoutingDecision {
        if self.heat_maps.is_empty() {
            return RoutingDecision {
                task_hash,
                best_node: self.node_idx,
                best_node_name: self.node_name.clone(),
                score: 0.0,
                reason: "no mesh data — routing locally".into(),
                alternatives: vec![],
            };
        }

        // Score all known nodes
        let mut scores: Vec<(u16, &str, f64)> = self.heat_maps.values()
            .map(|m| (m.node_idx, m.node_name.as_str(), m.score_for_task(task_dims)))
            .collect();

        // Include self (using local sim)
        let local_heat = CacheHeatMap::from_sim(self.node_idx, &self.node_name, &self.local_sim);
        scores.push((self.node_idx, self.node_name.as_str(), local_heat.score_for_task(task_dims)));

        scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let (best_node, best_name, best_score) = scores[0];
        let alternatives: Vec<(u16, f64)> = scores[1..].iter()
            .map(|(idx, _, score)| (*idx, *score))
            .collect();

        let dims_str = task_dims.iter().map(|d| format!("d{}", d)).collect::<Vec<_>>().join(",");
        let reason = format!(
            "dims [{}]: node {} has {:.1}% L1 advantage vs avg {:.1}%",
            dims_str,
            best_name,
            best_score * 100.0,
            scores.iter().map(|(_, _, s)| s).sum::<f64>() / scores.len() as f64 * 100.0,
        );

        RoutingDecision {
            task_hash,
            best_node,
            best_node_name: best_name.to_string(),
            score: best_score,
            reason,
            alternatives,
        }
    }

    /// Generate prefetch hints to broadcast to the mesh.
    /// When this node sees a miss pattern, tell siblings to warm their caches.
    pub fn generate_mesh_prefetch_hints(&self) -> Vec<(PhextCoord, String)> {
        // Find the coldest dimension — siblings should prefetch in that space
        let cold_dim = (0..11)
            .filter(|&i| self.local_sim.dim_stats(i).accesses > 0)
            .min_by(|&a, &b| {
                self.local_sim.dim_stats(a).l1_rate()
                    .partial_cmp(&self.local_sim.dim_stats(b).l1_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(dim) = cold_dim {
            let ds = self.local_sim.dim_stats(dim);
            let hint_str = format!(
                "PREFETCH dim:{} l1_rate:{:.3} misses:{} — warm this dimension",
                dim, ds.l1_rate(), ds.misses
            );
            // Generate a representative coordinate in this dimension
            let mut coord = [1u16; 11];
            coord[dim] = 2; // hint: access coord+1 in the cold dimension
            vec![(PhextCoord::new(coord), hint_str)]
        } else {
            vec![]
        }
    }

    /// Stats summary for display
    pub fn stats_summary(&self) -> String {
        format!(
            "{} cache: L1={:.1}% L2={:.1}% | mesh nodes={} | hot_dim={}",
            self.node_name,
            self.local_sim.l1_hit_rate() * 100.0,
            self.local_sim.total_l2() as f64 / self.local_sim.total_accesses().max(1) as f64 * 100.0,
            self.heat_maps.len(),
            CacheHeatMap::from_sim(self.node_idx, &self.node_name, &self.local_sim).hottest_dim,
        )
    }
}
