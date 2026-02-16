//! Associative Engine — R23W15 "Focus on the Gap"
//!
//! Wires HDC associative memory into the executor.
//! This is the 1/9th — the intelligence layer that was hollow.
//!
//! SASSOC: Store a pattern at a coordinate (bind coord→hypervector, insert into memory)
//! SROUTE: Query nearest match (encode query, find closest in memory)
//! SNEIGHBR: Find k-nearest neighbors within threshold

use crate::hdc::{HyperVector, AssociativeMemory, HDC_DEFAULT_WIDTH};
use crate::phext_coord::PhextCoord;

/// Associative state attached to a sentron.
/// This is the "mind" — the accumulated knowledge a sentron has built.
#[derive(Debug, Clone)]
pub struct AssocState {
    pub memory: AssociativeMemory,
    width: usize,
}

impl AssocState {
    pub fn new() -> Self {
        Self {
            memory: AssociativeMemory::new(),
            width: HDC_DEFAULT_WIDTH,
        }
    }

    /// SASSOC: Store a phext coordinate in associative memory.
    /// Returns the index of the stored entry.
    pub fn store(&mut self, coord: &PhextCoord) -> usize {
        let dims = coord.dims();
        self.memory.store(dims, self.width);
        self.memory.len() - 1
    }

    /// SROUTE: Query for the nearest coordinate in memory.
    /// Returns (similarity_score_as_i64, matched_coord_hash) or (0, 0) if empty.
    pub fn route(&self, coord: &PhextCoord) -> (i64, i64) {
        if self.memory.is_empty() {
            return (0, 0);
        }
        let query = HyperVector::from_coord(&coord.dims(), self.width);
        match self.memory.query_nearest(&query) {
            Some((found_dims, similarity)) => {
                // Encode similarity as fixed-point i64 (multiply by 1000)
                let sim_i64 = (similarity * 1000.0) as i64;
                // Hash the found coordinate for identification
                let hash = coord_hash(&found_dims);
                (sim_i64, hash)
            }
            None => (0, 0),
        }
    }

    /// SNEIGHBR: Find entries above a similarity threshold.
    /// Returns count of matches found.
    pub fn neighbors(&self, coord: &PhextCoord, threshold: f64) -> i64 {
        if self.memory.is_empty() {
            return 0;
        }
        let query = HyperVector::from_coord(&coord.dims(), self.width);
        let results = self.memory.query_above(&query, threshold);
        results.len() as i64
    }

    /// How many entries stored
    pub fn len(&self) -> usize {
        self.memory.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }
}

impl Default for AssocState {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple deterministic hash for a coordinate (for register-width identification)
fn coord_hash(dims: &[u16; 11]) -> i64 {
    let mut h: u64 = 0xcafe_babe_dead_beef;
    for &d in dims {
        h = h.wrapping_mul(31).wrapping_add(d as u64);
    }
    h as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_route_exact() {
        let mut state = AssocState::new();
        let coord = PhextCoord::new([2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1]);
        state.store(&coord);

        let (sim, _hash) = state.route(&coord);
        // Exact match should have high similarity (>900 on 1000 scale)
        assert!(sim > 900, "Exact match similarity should be >900, got {}", sim);
    }

    #[test]
    fn store_and_route_nearest() {
        let mut state = AssocState::new();
        // Store 5 coordinates
        for i in 1..=5u16 {
            let mut c = [1u16; 11];
            c[0] = i;
            state.store(&PhextCoord::new(c));
        }

        // Query coord [3,1,1,...] — should find itself
        let query = PhextCoord::new([3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let (sim, _) = state.route(&query);
        assert!(sim > 900, "Near-exact match should be high, got {}", sim);
    }

    #[test]
    fn neighbors_threshold() {
        let mut state = AssocState::new();
        for i in 1..=10u16 {
            let mut c = [1u16; 11];
            c[0] = i;
            state.store(&PhextCoord::new(c));
        }

        let query = PhextCoord::new([5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        // With threshold 0.5, should find multiple neighbors
        let count = state.neighbors(&query, 0.5);
        assert!(count > 0, "Should find at least 1 neighbor, got {}", count);
    }

    #[test]
    fn empty_returns_zero() {
        let state = AssocState::new();
        let coord = PhextCoord::new([1; 11]);
        let (sim, hash) = state.route(&coord);
        assert_eq!(sim, 0);
        assert_eq!(hash, 0);
        assert_eq!(state.neighbors(&coord, 0.5), 0);
    }

    #[test]
    fn store_many() {
        let mut state = AssocState::new();
        for i in 0..100u16 {
            let mut c = [1u16; 11];
            c[0] = i % 10 + 1;
            c[1] = i / 10 + 1;
            state.store(&PhextCoord::new(c));
        }
        assert_eq!(state.len(), 100);
    }

    #[test]
    fn coord_hash_deterministic() {
        let c = [2u16, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1];
        assert_eq!(coord_hash(&c), coord_hash(&c));
    }

    #[test]
    fn coord_hash_different() {
        let a = [1u16; 11];
        let b = [2u16; 11];
        assert_ne!(coord_hash(&a), coord_hash(&b));
    }
}
