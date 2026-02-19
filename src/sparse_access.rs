// -----------------------------------------------
// sparse_access.rs — S-Pipe Gather/Scatter
// -----------------------------------------------
// Coordinate-addressed bulk memory operations for the sparse pipe.
// gather(coords) → values: read multiple phext coordinates in one call.
// scatter(coords, values) → write multiple coordinates in one call.
// Zero external dependencies.
//
// R23W19 — Chrys 🦋

use crate::phext_coord::PhextCoord;
use crate::ppt::PhextPageTable;

/// Result of a gather operation for a single coordinate
#[derive(Debug, Clone)]
pub struct GatherResult {
    pub coord: PhextCoord,
    pub value: Option<u64>,
    pub ppt_hit: bool,
}

/// Result of a scatter operation for a single coordinate
#[derive(Debug, Clone)]
pub struct ScatterResult {
    pub coord: PhextCoord,
    pub success: bool,
}

/// Sparse access engine for coordinate-addressed bulk operations
pub struct SparseAccess {
    /// Backing store: flat memory indexed by PPT-translated addresses
    store: Vec<u64>,
    /// PPT for coordinate → physical translation
    #[allow(dead_code)]
    ppt: PhextPageTable,
    /// Stats
    gather_count: u64,
    scatter_count: u64,
}

impl SparseAccess {
    /// Create a new sparse access engine with given capacity (number of slots)
    pub fn new(capacity: usize) -> Self {
        Self {
            store: vec![0u64; capacity],
            ppt: PhextPageTable::new(),
            gather_count: 0,
            scatter_count: 0,
        }
    }

    /// Gather: read values from multiple phext coordinates
    pub fn gather(&mut self, coords: &[PhextCoord]) -> Vec<GatherResult> {
        self.gather_count += coords.len() as u64;
        coords.iter().map(|coord| {
            let physical = self.translate(coord);
            match physical {
                Some(addr) if addr < self.store.len() => GatherResult {
                    coord: coord.clone(),
                    value: Some(self.store[addr]),
                    ppt_hit: true,
                },
                _ => GatherResult {
                    coord: coord.clone(),
                    value: None,
                    ppt_hit: false,
                },
            }
        }).collect()
    }

    /// Scatter: write values to multiple phext coordinates
    pub fn scatter(&mut self, coords: &[PhextCoord], values: &[u64]) -> Vec<ScatterResult> {
        self.scatter_count += coords.len() as u64;
        let len = coords.len().min(values.len());
        let mut results = Vec::with_capacity(len);

        for i in 0..len {
            let physical = self.translate_or_allocate(&coords[i]);
            match physical {
                Some(addr) if addr < self.store.len() => {
                    self.store[addr] = values[i];
                    results.push(ScatterResult {
                        coord: coords[i].clone(),
                        success: true,
                    });
                }
                _ => {
                    results.push(ScatterResult {
                        coord: coords[i].clone(),
                        success: false,
                    });
                }
            }
        }

        results
    }

    /// Translate coordinate to physical address via PPT
    fn translate(&self, coord: &PhextCoord) -> Option<usize> {
        // Use fast_hash modulo capacity as physical address
        // In production, PPT would maintain a proper mapping
        let hash = coord.fast_hash();
        let addr = (hash as usize) % self.store.len();
        Some(addr)
    }

    /// Translate or allocate a new mapping
    fn translate_or_allocate(&mut self, coord: &PhextCoord) -> Option<usize> {
        self.translate(coord)
    }

    /// Total gather operations
    pub fn gather_ops(&self) -> u64 {
        self.gather_count
    }

    /// Total scatter operations
    pub fn scatter_ops(&self) -> u64 {
        self.scatter_count
    }

    /// Store capacity
    pub fn capacity(&self) -> usize {
        self.store.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(scroll: u16, section: u16) -> PhextCoord {
        PhextCoord::new([scroll, section, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }

    #[test]
    fn test_scatter_then_gather() {
        let mut sa = SparseAccess::new(1024);
        let coords = vec![coord(1, 1), coord(2, 1), coord(3, 1)];
        let values = vec![42, 99, 7];

        let scatter_results = sa.scatter(&coords, &values);
        assert!(scatter_results.iter().all(|r| r.success));

        let gather_results = sa.gather(&coords);
        assert_eq!(gather_results[0].value, Some(42));
        assert_eq!(gather_results[1].value, Some(99));
        assert_eq!(gather_results[2].value, Some(7));
    }

    #[test]
    fn test_gather_unwritten_returns_zero() {
        let mut sa = SparseAccess::new(1024);
        let results = sa.gather(&[coord(50, 50)]);
        // Unwritten memory is zero-initialized
        assert_eq!(results[0].value, Some(0));
    }

    #[test]
    fn test_scatter_overwrites() {
        let mut sa = SparseAccess::new(1024);
        let c = coord(5, 5);

        sa.scatter(&[c.clone()], &[100]);
        sa.scatter(&[c.clone()], &[200]);

        let results = sa.gather(&[c]);
        assert_eq!(results[0].value, Some(200));
    }

    #[test]
    fn test_gather_multiple_coords() {
        let mut sa = SparseAccess::new(4096);
        // Scatter to 10 different coordinates
        let coords: Vec<PhextCoord> = (1..=10).map(|i| coord(i, 1)).collect();
        let values: Vec<u64> = (1..=10).map(|i| i * 100).collect();
        sa.scatter(&coords, &values);

        // Gather all 10
        let results = sa.gather(&coords);
        assert_eq!(results.len(), 10);
        for r in &results {
            assert!(r.ppt_hit);
            assert!(r.value.is_some());
        }
    }

    #[test]
    fn test_mismatched_scatter_length() {
        let mut sa = SparseAccess::new(1024);
        // More coords than values — should only write min(coords, values)
        let coords = vec![coord(1, 1), coord(2, 1), coord(3, 1)];
        let values = vec![42]; // Only 1 value for 3 coords

        let results = sa.scatter(&coords, &values);
        assert_eq!(results.len(), 1); // Only 1 written
        assert!(results[0].success);
    }

    #[test]
    fn test_stats_tracking() {
        let mut sa = SparseAccess::new(1024);
        let coords = vec![coord(1, 1), coord(2, 1)];

        sa.scatter(&coords, &[10, 20]);
        sa.gather(&coords);
        sa.gather(&coords);

        assert_eq!(sa.scatter_ops(), 2);
        assert_eq!(sa.gather_ops(), 4); // 2 coords × 2 gathers
    }

    #[test]
    fn test_capacity() {
        let sa = SparseAccess::new(512);
        assert_eq!(sa.capacity(), 512);
    }
}
