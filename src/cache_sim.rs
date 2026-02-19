// -----------------------------------------------
// cache_sim.rs — Cache Hierarchy Simulator
// -----------------------------------------------
// Simulates L1/L2/L3 cache behavior for phext coordinate accesses.
// Tracks hit rates per cache level and per dimension.
// Zero external dependencies.
//
// R23W20 — Chrys 🦋

use crate::phext_coord::PhextCoord;

/// Cache level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheLevel {
    L1,
    L2,
    L3,
    Miss,
}

/// Per-dimension hit statistics
#[derive(Debug, Clone)]
pub struct DimStats {
    pub accesses: u64,
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub l3_hits: u64,
    pub misses: u64,
}

impl DimStats {
    fn new() -> Self {
        Self { accesses: 0, l1_hits: 0, l2_hits: 0, l3_hits: 0, misses: 0 }
    }

    pub fn hit_rate(&self) -> f64 {
        if self.accesses == 0 { return 0.0; }
        (self.l1_hits + self.l2_hits + self.l3_hits) as f64 / self.accesses as f64
    }

    pub fn l1_rate(&self) -> f64 {
        if self.accesses == 0 { return 0.0; }
        self.l1_hits as f64 / self.accesses as f64
    }
}

/// Ring buffer cache line (stores coord hashes)
struct CacheRing {
    entries: Vec<u64>,
    capacity: usize,
    write_pos: usize,
    count: usize,
}

impl CacheRing {
    fn new(capacity: usize) -> Self {
        Self {
            entries: vec![0; capacity.max(1)],
            capacity: capacity.max(1),
            write_pos: 0,
            count: 0,
        }
    }

    fn contains(&self, hash: u64) -> bool {
        let limit = self.count.min(self.capacity);
        for i in 0..limit {
            if self.entries[i] == hash {
                return true;
            }
        }
        false
    }

    fn insert(&mut self, hash: u64) {
        self.entries[self.write_pos] = hash;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }
}

/// Simulated 3-level cache hierarchy for phext coordinate accesses
pub struct CacheSimulator {
    l1: CacheRing,
    l2: CacheRing,
    l3: CacheRing,
    /// Per-dimension statistics (11 dimensions)
    dim_stats: [DimStats; 11],
    /// Aggregate stats
    total_accesses: u64,
    total_l1: u64,
    total_l2: u64,
    total_l3: u64,
    total_miss: u64,
}

impl CacheSimulator {
    /// Create cache simulator with given sizes per level
    pub fn new(l1_size: usize, l2_size: usize, l3_size: usize) -> Self {
        Self {
            l1: CacheRing::new(l1_size),
            l2: CacheRing::new(l2_size),
            l3: CacheRing::new(l3_size),
            dim_stats: core::array::from_fn(|_| DimStats::new()),
            total_accesses: 0,
            total_l1: 0,
            total_l2: 0,
            total_l3: 0,
            total_miss: 0,
        }
    }

    /// Default cache sizes: L1=64, L2=512, L3=4096
    pub fn default_sizes() -> Self {
        Self::new(64, 512, 4096)
    }

    /// Access a coordinate and return which cache level served it
    pub fn access(&mut self, coord: &PhextCoord) -> CacheLevel {
        let hash = coord.fast_hash();
        self.total_accesses += 1;

        // Determine which dimension changed (for per-dim stats)
        let changing_dim = self.detect_changing_dimension(coord);

        let level = if self.l1.contains(hash) {
            self.total_l1 += 1;
            CacheLevel::L1
        } else if self.l2.contains(hash) {
            self.total_l2 += 1;
            // Promote to L1
            self.l1.insert(hash);
            CacheLevel::L2
        } else if self.l3.contains(hash) {
            self.total_l3 += 1;
            // Promote to L1 and L2
            self.l1.insert(hash);
            self.l2.insert(hash);
            CacheLevel::L3
        } else {
            self.total_miss += 1;
            // Insert into all levels
            self.l1.insert(hash);
            self.l2.insert(hash);
            self.l3.insert(hash);
            CacheLevel::Miss
        };

        // Update per-dimension stats
        if let Some(dim) = changing_dim {
            let ds = &mut self.dim_stats[dim];
            ds.accesses += 1;
            match level {
                CacheLevel::L1 => ds.l1_hits += 1,
                CacheLevel::L2 => ds.l2_hits += 1,
                CacheLevel::L3 => ds.l3_hits += 1,
                CacheLevel::Miss => ds.misses += 1,
            }
        }

        level
    }

    /// Pre-warm: insert a coordinate into L1 (used by prefetcher)
    pub fn prewarm(&mut self, coord: &PhextCoord) {
        let hash = coord.fast_hash();
        self.l1.insert(hash);
        self.l2.insert(hash);
    }

    /// Detect which dimension is varying (crude: look at lowest non-1 dim)
    fn detect_changing_dimension(&self, coord: &PhextCoord) -> Option<usize> {
        let dims = coord.dims();
        // Return the lowest dimension that isn't 1 (heuristic for "most active dim")
        for i in 0..11 {
            if dims[i] != 1 {
                return Some(i);
            }
        }
        Some(0) // All 1s → scroll dimension
    }

    /// Overall L1 hit rate
    pub fn l1_hit_rate(&self) -> f64 {
        if self.total_accesses == 0 { return 0.0; }
        self.total_l1 as f64 / self.total_accesses as f64
    }

    /// Overall combined hit rate (L1+L2+L3)
    pub fn hit_rate(&self) -> f64 {
        if self.total_accesses == 0 { return 0.0; }
        (self.total_l1 + self.total_l2 + self.total_l3) as f64 / self.total_accesses as f64
    }

    /// Per-dimension stats
    pub fn dim_stats(&self, dim: usize) -> &DimStats {
        &self.dim_stats[dim.min(10)]
    }

    /// Total accesses
    pub fn total_accesses(&self) -> u64 { self.total_accesses }
    pub fn total_l1(&self) -> u64 { self.total_l1 }
    pub fn total_l2(&self) -> u64 { self.total_l2 }
    pub fn total_l3(&self) -> u64 { self.total_l3 }
    pub fn total_misses(&self) -> u64 { self.total_miss }

    /// Reset all stats and cache contents
    pub fn reset(&mut self) {
        *self = Self::new(self.l1.capacity, self.l2.capacity, self.l3.capacity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(scroll: u16) -> PhextCoord {
        PhextCoord::new([scroll, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }

    #[test]
    fn test_first_access_is_miss() {
        let mut cache = CacheSimulator::default_sizes();
        assert_eq!(cache.access(&coord(1)), CacheLevel::Miss);
        assert_eq!(cache.total_misses(), 1);
    }

    #[test]
    fn test_repeat_access_is_l1() {
        let mut cache = CacheSimulator::default_sizes();
        cache.access(&coord(5));
        assert_eq!(cache.access(&coord(5)), CacheLevel::L1);
        assert_eq!(cache.total_l1(), 1);
    }

    #[test]
    fn test_sequential_locality() {
        let mut cache = CacheSimulator::default_sizes();
        // Access 1..50, then 1..50 again — all should be L1 on second pass
        for i in 1..=50 {
            cache.access(&coord(i));
        }
        let mut l1_hits = 0;
        for i in 1..=50 {
            if cache.access(&coord(i)) == CacheLevel::L1 {
                l1_hits += 1;
            }
        }
        // L1 is 64 entries, so all 50 should still be in L1
        assert_eq!(l1_hits, 50);
    }

    #[test]
    fn test_l1_eviction_falls_to_l2() {
        // L1 = 4 entries, L2 = 16
        let mut cache = CacheSimulator::new(4, 16, 64);
        // Fill L1 with coords 1-4
        for i in 1..=4 { cache.access(&coord(i)); }
        // Push coord 1 out of L1 by accessing 5-8
        for i in 5..=8 { cache.access(&coord(i)); }
        // Coord 1 should be in L2 (evicted from L1 but still in L2)
        let level = cache.access(&coord(1));
        assert!(level == CacheLevel::L1 || level == CacheLevel::L2,
            "Expected L1 or L2, got {:?}", level);
    }

    #[test]
    fn test_prewarm() {
        let mut cache = CacheSimulator::default_sizes();
        cache.prewarm(&coord(42));
        // Should be L1 hit now
        assert_eq!(cache.access(&coord(42)), CacheLevel::L1);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let mut cache = CacheSimulator::default_sizes();
        cache.access(&coord(1)); // miss
        cache.access(&coord(1)); // L1
        cache.access(&coord(1)); // L1
        assert_eq!(cache.total_accesses(), 3);
        assert!(cache.l1_hit_rate() > 0.6);
    }

    #[test]
    fn test_empty_hit_rate() {
        let cache = CacheSimulator::default_sizes();
        assert_eq!(cache.hit_rate(), 0.0);
        assert_eq!(cache.l1_hit_rate(), 0.0);
    }

    #[test]
    fn test_dim_stats() {
        let mut cache = CacheSimulator::default_sizes();
        let c = PhextCoord::new([5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        cache.access(&c);
        cache.access(&c);
        let ds = cache.dim_stats(0);
        assert_eq!(ds.accesses, 2);
    }
}
