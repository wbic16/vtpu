// -----------------------------------------------
// prefetch.rs — Dimensional Prefetcher
// -----------------------------------------------
// Predicts next phext coordinates based on access patterns.
// Three strategies: Sequential, Stride, Topology.
// Zero external dependencies.
//
// R23W19 — Chrys 🦋

use crate::phext_coord::PhextCoord;

/// Prefetch strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrefetchStrategy {
    /// Predict scroll+1 (and section+1 at scroll boundary)
    Sequential,
    /// Detect repeated deltas and extrapolate
    Stride,
    /// Use NeuronWiring neighbor coordinates
    Topology,
}

/// Tracks access patterns and generates prefetch predictions
pub struct DimensionalPrefetcher {
    /// Recent access history (ring buffer)
    history: Vec<PhextCoord>,
    /// Max history entries
    capacity: usize,
    /// Write position in ring buffer
    write_pos: usize,
    /// Number of entries filled
    count: usize,
    /// Active strategy
    strategy: PrefetchStrategy,
    /// Prefetch hits (predicted coord was later accessed)
    hits: u64,
    /// Total accesses
    total: u64,
    /// Outstanding predictions (coords we think will be accessed next)
    predictions: Vec<PhextCoord>,
}

impl DimensionalPrefetcher {
    /// Create a new prefetcher with given history capacity
    pub fn new(capacity: usize, strategy: PrefetchStrategy) -> Self {
        Self {
            history: vec![PhextCoord::zero(); capacity.max(4)],
            capacity: capacity.max(4),
            write_pos: 0,
            count: 0,
            strategy,
            hits: 0,
            total: 0,
            predictions: Vec::with_capacity(4),
        }
    }

    /// Record an access and generate predictions
    pub fn access(&mut self, coord: &PhextCoord) -> Vec<PhextCoord> {
        self.total += 1;

        // Check if this access was predicted
        if self.predictions.iter().any(|p| p.fast_hash() == coord.fast_hash()) {
            self.hits += 1;
        }

        // Store in history
        self.history[self.write_pos] = coord.clone();
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }

        // Generate predictions based on strategy
        self.predictions = match self.strategy {
            PrefetchStrategy::Sequential => self.predict_sequential(coord),
            PrefetchStrategy::Stride => self.predict_stride(coord),
            PrefetchStrategy::Topology => self.predict_topology(coord),
        };

        self.predictions.clone()
    }

    /// Sequential: predict scroll+1, and section+1 if at scroll boundary
    fn predict_sequential(&self, coord: &PhextCoord) -> Vec<PhextCoord> {
        let mut preds = Vec::with_capacity(2);
        let dims = coord.dims();

        // Scroll dimension is index 0 (innermost)
        // Predict next scroll
        if dims[0] < PhextCoord::MAX_DIM {
            let mut next = dims;
            next[0] += 1;
            preds.push(PhextCoord::new(next));
        }

        // If scroll is high, also predict section rollover
        if dims[0] >= 90 && dims[1] < PhextCoord::MAX_DIM {
            let mut next = dims;
            next[0] = 1; // Reset scroll
            next[1] += 1; // Increment section
            preds.push(PhextCoord::new(next));
        }

        preds
    }

    /// Stride: detect repeated deltas across dimensions and extrapolate
    fn predict_stride(&self, coord: &PhextCoord) -> Vec<PhextCoord> {
        if self.count < 3 {
            return self.predict_sequential(coord);
        }

        // Get last 3 accesses (including current)
        let prev1_idx = if self.write_pos == 0 { self.capacity - 1 } else { self.write_pos - 1 };
        let prev2_idx = if prev1_idx == 0 { self.capacity - 1 } else { prev1_idx - 1 };

        // write_pos already advanced past current, so prev1 IS current
        // prev2 is one before current, prev3 is two before
        let current = coord.dims();
        let prev1 = self.history[prev2_idx].dims();
        let prev2 = self.history[if prev2_idx == 0 { self.capacity - 1 } else { prev2_idx - 1 }].dims();

        // Compute deltas
        let mut delta1 = [0i32; 11];
        let mut delta2 = [0i32; 11];
        for i in 0..11 {
            delta1[i] = current[i] as i32 - prev1[i] as i32;
            delta2[i] = prev1[i] as i32 - prev2[i] as i32;
        }

        // If deltas match, we have a stride — extrapolate
        if delta1 == delta2 {
            let mut next = [0u16; 11];
            let mut valid = true;
            for i in 0..11 {
                let val = current[i] as i32 + delta1[i];
                if val < 0 || val > PhextCoord::MAX_DIM as i32 {
                    valid = false;
                    break;
                }
                next[i] = val as u16;
            }
            if valid {
                return vec![PhextCoord::new(next)];
            }
        }

        // No stride detected, fall back to sequential
        self.predict_sequential(coord)
    }

    /// Topology: predict neighbor coordinates (4 upstream + 4 downstream from wiring)
    /// For now, predicts ±1 on the 4 lowest dimensions (scroll, section, chapter, book)
    fn predict_topology(&self, coord: &PhextCoord) -> Vec<PhextCoord> {
        let dims = coord.dims();
        let mut preds = Vec::with_capacity(4);

        // Predict +1 on the 4 lowest dimensions (the Ba Gua directions)
        for dim_idx in 0..4u8 {
            let val = dims[dim_idx as usize];
            if val < PhextCoord::MAX_DIM {
                let mut next = dims;
                next[dim_idx as usize] = val + 1;
                preds.push(PhextCoord::new(next));
            }
        }

        preds
    }

    /// Prefetch hit rate (0.0 - 1.0)
    pub fn hit_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.hits as f64 / self.total as f64
    }

    /// Total accesses recorded
    pub fn total_accesses(&self) -> u64 {
        self.total
    }

    /// Total prefetch hits
    pub fn total_hits(&self) -> u64 {
        self.hits
    }

    /// Current strategy
    pub fn strategy(&self) -> PrefetchStrategy {
        self.strategy
    }

    /// Switch strategy
    pub fn set_strategy(&mut self, strategy: PrefetchStrategy) {
        self.strategy = strategy;
    }

    /// Reset statistics (keep history)
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.total = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_prefetch() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Sequential);
        let coord = PhextCoord::new([5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let preds = pf.access(&coord);
        assert!(!preds.is_empty());
        // Should predict scroll 6
        assert_eq!(preds[0].get_dim(0), 6);
    }

    #[test]
    fn test_sequential_section_rollover() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Sequential);
        // Scroll at 95 — should predict both scroll+1 AND section rollover
        let coord = PhextCoord::new([95, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let preds = pf.access(&coord);
        assert!(preds.len() >= 2);
        assert_eq!(preds[0].get_dim(0), 96); // scroll+1
        assert_eq!(preds[1].get_dim(0), 1);  // scroll reset
        assert_eq!(preds[1].get_dim(1), 4);  // section+1
    }

    #[test]
    fn test_stride_detection() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Stride);
        // Access pattern: scroll 10, 15, 20 (stride of 5)
        pf.access(&PhextCoord::new([10, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        pf.access(&PhextCoord::new([15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        let preds = pf.access(&PhextCoord::new([20, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // Should predict scroll 25
        assert!(!preds.is_empty());
        assert_eq!(preds[0].get_dim(0), 25);
    }

    #[test]
    fn test_stride_hit_tracking() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Stride);
        // Build up stride
        pf.access(&PhextCoord::new([10, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        pf.access(&PhextCoord::new([15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        pf.access(&PhextCoord::new([20, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // Now access the predicted coord (25)
        pf.access(&PhextCoord::new([25, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        assert!(pf.total_hits() >= 1);
        assert!(pf.hit_rate() > 0.0);
    }

    #[test]
    fn test_topology_prefetch() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Topology);
        let coord = PhextCoord::new([5, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1]);
        let preds = pf.access(&coord);
        // Should predict +1 on 4 lowest dims
        assert_eq!(preds.len(), 4);
        assert_eq!(preds[0].get_dim(0), 6);  // scroll+1
        assert_eq!(preds[1].get_dim(1), 4);  // section+1
        assert_eq!(preds[2].get_dim(2), 3);  // chapter+1
        assert_eq!(preds[3].get_dim(3), 2);  // book+1
    }

    #[test]
    fn test_strategy_switch() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Sequential);
        assert_eq!(pf.strategy(), PrefetchStrategy::Sequential);
        pf.set_strategy(PrefetchStrategy::Topology);
        assert_eq!(pf.strategy(), PrefetchStrategy::Topology);
    }

    #[test]
    fn test_empty_hit_rate() {
        let pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Sequential);
        assert_eq!(pf.hit_rate(), 0.0);
        assert_eq!(pf.total_accesses(), 0);
    }

    #[test]
    fn test_max_dim_boundary() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Sequential);
        // At max scroll — should not predict beyond MAX_DIM
        let coord = PhextCoord::new([PhextCoord::MAX_DIM, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let preds = pf.access(&coord);
        // No scroll+1 possible, but section rollover should work
        for p in &preds {
            assert!(p.get_dim(0) <= PhextCoord::MAX_DIM);
        }
    }

    #[test]
    fn test_stride_multidimensional() {
        let mut pf = DimensionalPrefetcher::new(8, PrefetchStrategy::Stride);
        // Stride in both scroll AND section: (1,1) → (3,2) → (5,3)
        pf.access(&PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        pf.access(&PhextCoord::new([3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        let preds = pf.access(&PhextCoord::new([5, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // Should predict (7, 4, ...)
        assert!(!preds.is_empty());
        assert_eq!(preds[0].get_dim(0), 7);
        assert_eq!(preds[0].get_dim(1), 4);
    }
}
