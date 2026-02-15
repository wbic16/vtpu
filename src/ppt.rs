//! Phext Page Table (PPT)
//!
//! Maps 11-dimensional phext coordinates to flat physical addresses.
//! Exploits dimensional locality: coordinates differing in only low dimensions
//! map to adjacent cache lines, enabling hardware prefetchers to work on
//! semantic traversals.
//!
//! Design:
//! - Z-order (Morton) curve interleaving for inner 3 dimensions → L1 locality
//! - Hierarchical grouping for outer dimensions → L2/L3/DDR5 locality
//! - Translation cache (PTC) for hot coordinates → 1-cycle hit
//! - Wildcard support for SASSOC partial-coordinate queries

use crate::phext_coord::PhextCoord;
use std::collections::HashMap;

/// Z-order lookup table for 3D → 1D interleaving (7-bit per dimension)
/// Pre-computed for O(1) coordinate translation
struct ZOrderLUT {
    tables: [[u32; 128]; 3], // One table per inner dimension
}

impl ZOrderLUT {
    fn new() -> Self {
        let mut tables = [[0u32; 128]; 3];
        for v in 0..128u32 {
            for bit in 0..7 {
                tables[0][v as usize] |= ((v >> bit) & 1) << (bit * 3);
                tables[1][v as usize] |= ((v >> bit) & 1) << (bit * 3 + 1);
                tables[2][v as usize] |= ((v >> bit) & 1) << (bit * 3 + 2);
            }
        }
        ZOrderLUT { tables }
    }

    /// Interleave 3 dimension values into a Z-order index
    #[inline(always)]
    fn encode(&self, d0: u16, d1: u16, d2: u16) -> u32 {
        self.tables[0][(d0 & 0x7F) as usize]
            | self.tables[1][(d1 & 0x7F) as usize]
            | self.tables[2][(d2 & 0x7F) as usize]
    }
}

/// Translation cache entry
#[derive(Clone, Copy)]
struct PTCEntry {
    coord_lo: u64, // Lower 64 bits of PhextCoord (dims 0-5)
    coord_hi: u64, // Upper 64 bits (dims 6-10 + flags)
    address: usize, // Translated physical address
    valid: bool,
}

impl Default for PTCEntry {
    fn default() -> Self {
        PTCEntry { coord_lo: 0, coord_hi: 0, address: 0, valid: false }
    }
}

/// Phext Translation Cache — like a TLB but for 11D coordinates
/// 4-way set associative, 256 sets = 1024 entries
struct PTC {
    sets: Vec<[PTCEntry; 4]>,
    num_sets: usize,
    // Telemetry
    hits: u64,
    misses: u64,
}

impl PTC {
    fn new(num_sets: usize) -> Self {
        PTC {
            sets: vec![[PTCEntry::default(); 4]; num_sets],
            num_sets,
            hits: 0,
            misses: 0,
        }
    }

    /// Hash coordinate to set index
    #[inline(always)]
    fn set_index(&self, coord: &PhextCoord) -> usize {
        // Mix all dimensions for even distribution
        let dims = coord.dims();
        let mut hash = 0u64;
        for (i, &d) in dims.iter().enumerate() {
            hash ^= (d as u64).wrapping_mul(0x517cc1b727220a95_u64.wrapping_add(i as u64 * 7));
        }
        (hash as usize) % self.num_sets
    }

    /// Lookup coordinate in cache. Returns Some(address) on hit.
    #[inline]
    fn lookup(&mut self, coord: &PhextCoord) -> Option<usize> {
        let set_idx = self.set_index(coord);
        let (lo, hi) = coord.as_raw();
        
        for entry in &self.sets[set_idx] {
            if entry.valid && entry.coord_lo == lo && entry.coord_hi == hi {
                self.hits += 1;
                return Some(entry.address);
            }
        }
        self.misses += 1;
        None
    }

    /// Insert a translation into the cache (LRU-approximate: replace first invalid or slot 0)
    fn insert(&mut self, coord: &PhextCoord, address: usize) {
        let set_idx = self.set_index(coord);
        let (lo, hi) = coord.as_raw();

        // Find first invalid slot
        for entry in &mut self.sets[set_idx] {
            if !entry.valid {
                *entry = PTCEntry { coord_lo: lo, coord_hi: hi, address, valid: true };
                return;
            }
        }
        // All valid — evict slot 0 (pseudo-LRU)
        self.sets[set_idx][0] = PTCEntry { coord_lo: lo, coord_hi: hi, address, valid: true };
    }

    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { return 0.0; }
        self.hits as f64 / total as f64
    }

    fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }
}

/// Memory tier classification based on which dimensions differ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryTier {
    /// Dimensions 0-2 only → L1 (32 KiB scratchpad)
    L1Scratchpad,
    /// Dimensions 0-4 → L2 (1 MiB local)
    L2Local,
    /// Dimensions 0-7 → L3 (32 MiB shared)
    L3Shared,
    /// Dimensions 0-9 → DDR5 (96 GiB node-local)
    NodeLocal,
    /// All dimensions → Remote (cluster-wide)
    Remote,
}

/// Phext Page Table
///
/// The core address translation engine for the vTPU.
/// Maps 11D phext coordinates to flat physical addresses using:
/// 1. Z-order curve for inner 3 dimensions (cache-line adjacency)
/// 2. Hierarchical page grouping for outer dimensions
/// 3. Translation cache for hot coordinates
pub struct PhextPageTable {
    z_lut: ZOrderLUT,
    ptc: PTC,
    /// Allocated regions: maps outer-dimension group key to base address
    regions: HashMap<u64, usize>,
    /// Total allocated bytes
    allocated: usize,
    /// Slab size for inner-dimension pages (Z-order mapped)
    inner_page_size: usize,
    /// Track which dimensions are "hot" for prefetch optimization
    hot_dims: u16,
}

/// PPT statistics for KPI tracking
#[derive(Debug, Clone)]
pub struct PPTStats {
    pub ptc_hits: u64,
    pub ptc_misses: u64,
    pub ptc_hit_rate: f64,
    pub regions_allocated: usize,
    pub bytes_allocated: usize,
    pub hot_dims: u16,
}

impl PhextPageTable {
    /// Create a new PPT with default configuration
    /// - 256 PTC sets × 4-way = 1024 entries
    /// - 2^21 = 2M entries per inner page (fits L1→L3 hierarchy)
    pub fn new() -> Self {
        Self::with_config(256, 1 << 21)
    }

    /// Create with custom PTC sets and inner page size
    pub fn with_config(ptc_sets: usize, inner_page_size: usize) -> Self {
        PhextPageTable {
            z_lut: ZOrderLUT::new(),
            ptc: PTC::new(ptc_sets),
            regions: HashMap::new(),
            allocated: 0,
            inner_page_size,
            hot_dims: 0,
        }
    }

    /// Translate a phext coordinate to a physical address offset
    ///
    /// This is the hot path — called by every S-Pipe operation.
    /// PTC hit: ~1 cycle. PTC miss: ~12 cycles (hash + region lookup + Z-encode).
    #[inline]
    pub fn translate(&mut self, coord: &PhextCoord) -> usize {
        // Fast path: PTC hit
        if let Some(addr) = self.ptc.lookup(coord) {
            return addr;
        }

        // Slow path: compute translation
        let addr = self.compute_address(coord);
        self.ptc.insert(coord, addr);
        addr
    }

    /// Compute physical address from coordinate (slow path)
    fn compute_address(&mut self, coord: &PhextCoord) -> usize {
        let dims = coord.dims();

        // Outer dimensions (3-10) form the region key
        let region_key = self.outer_key(&dims);

        // Get or allocate region base
        let base = *self.regions.entry(region_key).or_insert_with(|| {
            let addr = self.allocated;
            self.allocated += self.inner_page_size * std::mem::size_of::<f64>();
            addr
        });

        // Inner dimensions (0-2) mapped via Z-order for cache locality
        let z_offset = self.z_lut.encode(dims[0], dims[1], dims[2]) as usize;

        base + z_offset * std::mem::size_of::<f64>()
    }

    /// Compute outer-dimension region key from dims 3-10
    /// Adjacent outer coordinates get nearby keys for L2/L3 locality
    #[inline]
    fn outer_key(&self, dims: &[u16; 11]) -> u64 {
        // Pack dims 3-10 into 64 bits (8 dims × 8 bits each = 64 bits)
        // We truncate to 8 bits per dim (256 positions) for the region key.
        // Full precision is in the PTC entry.
        let mut key = 0u64;
        for i in 3..11 {
            key |= ((dims[i] as u64) & 0xFF) << ((i - 3) * 8);
        }
        key
    }

    /// Classify which memory tier a coordinate targets
    /// Used for prefetch hint generation
    pub fn classify_tier(&self, coord: &PhextCoord, reference: &PhextCoord) -> MemoryTier {
        let a = coord.dims();
        let b = reference.dims();

        // Find highest dimension that differs
        let mut highest_diff = 0;
        for i in (0..11).rev() {
            if a[i] != b[i] {
                highest_diff = i + 1;
                break;
            }
        }

        match highest_diff {
            0..=3 => MemoryTier::L1Scratchpad,
            4..=5 => MemoryTier::L2Local,
            6..=8 => MemoryTier::L3Shared,
            9..=10 => MemoryTier::NodeLocal,
            _ => MemoryTier::Remote,
        }
    }

    /// Record a dimension access for hot-dim tracking
    #[inline]
    pub fn touch_dim(&mut self, dim: u8) {
        if dim < 11 {
            self.hot_dims |= 1 << dim;
        }
    }

    /// Get the currently hot dimensions (bitmask)
    pub fn hot_dims(&self) -> u16 {
        self.hot_dims
    }

    /// Prefetch hint: generate addresses along a dimension for the hardware prefetcher
    /// Returns a sequence of addresses that should be prefetched
    pub fn prefetch_along_dim(&mut self, base_coord: &PhextCoord, dim: u8, count: u16) -> Vec<usize> {
        self.touch_dim(dim);
        let mut addrs = Vec::with_capacity(count as usize);
        let mut coord = *base_coord;
        for i in 0..count {
            let current = coord.get_dim(dim);
            coord.set_dim(dim, (current + 1).min(PhextCoord::MAX_DIM));
            addrs.push(self.translate(&coord));
        }
        let _ = count; // suppress unused
        addrs
    }

    /// Get PPT statistics for KPI reporting
    pub fn stats(&self) -> PPTStats {
        PPTStats {
            ptc_hits: self.ptc.hits,
            ptc_misses: self.ptc.misses,
            ptc_hit_rate: self.ptc.hit_rate(),
            regions_allocated: self.regions.len(),
            bytes_allocated: self.allocated,
            hot_dims: self.hot_dims,
        }
    }

    /// Reset PTC statistics (for benchmark isolation)
    pub fn reset_stats(&mut self) {
        self.ptc.reset_stats();
        self.hot_dims = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_deterministic() {
        let mut ppt = PhextPageTable::new();
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let addr1 = ppt.translate(&coord);
        let addr2 = ppt.translate(&coord);
        assert_eq!(addr1, addr2, "Same coordinate must produce same address");
    }

    #[test]
    fn test_different_coords_different_addrs() {
        let mut ppt = PhextPageTable::new();
        let a = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let b = PhextCoord::new([4, 5, 6, 4, 5, 6, 7, 8, 9, 10, 11]);
        let addr_a = ppt.translate(&a);
        let addr_b = ppt.translate(&b);
        assert_ne!(addr_a, addr_b, "Different inner coords must produce different addresses");
    }

    #[test]
    fn test_inner_locality() {
        // Coordinates differing only in dim 0 should be close in physical space
        let mut ppt = PhextPageTable::new();
        let a = PhextCoord::new([10, 20, 30, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = PhextCoord::new([11, 20, 30, 1, 1, 1, 1, 1, 1, 1, 1]);
        let addr_a = ppt.translate(&a);
        let addr_b = ppt.translate(&b);
        // Z-order: adjacent in dim 0 should be within a small stride
        let distance = if addr_a > addr_b { addr_a - addr_b } else { addr_b - addr_a };
        assert!(distance < 64, "Adjacent dim-0 coords should be within 64 bytes (1 cache line), got {}", distance);
    }

    #[test]
    fn test_ptc_hit_rate() {
        let mut ppt = PhextPageTable::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        // First access: miss
        ppt.translate(&coord);
        assert_eq!(ppt.stats().ptc_hits, 0);
        assert_eq!(ppt.stats().ptc_misses, 1);

        // Second access: hit
        ppt.translate(&coord);
        assert_eq!(ppt.stats().ptc_hits, 1);
        assert_eq!(ppt.stats().ptc_misses, 1);
        assert!((ppt.stats().ptc_hit_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_ptc_high_hit_rate_structured() {
        let mut ppt = PhextPageTable::new();

        // Simulate structured workload: sweep dim 0 repeatedly (same outer dims)
        // This should achieve very high PTC hit rate
        let n_sweeps = 10;
        let sweep_size = 64;

        for _ in 0..n_sweeps {
            for d0 in 0..sweep_size {
                let coord = PhextCoord::new([d0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
                ppt.translate(&coord);
            }
        }

        let stats = ppt.stats();
        let expected_total = n_sweeps * sweep_size;
        assert_eq!(stats.ptc_hits + stats.ptc_misses, expected_total as u64);
        // First sweep: all misses (64). Subsequent sweeps: all hits (9 × 64 = 576).
        // Expected hit rate: 576/640 = 90%. PTC has 1024 entries, 64 coords fit easily.
        assert!(stats.ptc_hit_rate > 0.85,
            "Structured workload should achieve >85% PTC hit rate, got {:.1}%",
            stats.ptc_hit_rate * 100.0);
    }

    #[test]
    fn test_memory_tier_classification() {
        let ppt = PhextPageTable::new();
        let base = PhextCoord::new([10, 20, 30, 4, 5, 6, 7, 8, 9, 10, 11]);

        // Same coord
        assert_eq!(ppt.classify_tier(&base, &base), MemoryTier::L1Scratchpad);

        // Diff in dim 0 only
        let near = PhextCoord::new([11, 20, 30, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(ppt.classify_tier(&near, &base), MemoryTier::L1Scratchpad);

        // Diff in dim 4
        let mid = PhextCoord::new([10, 20, 30, 4, 6, 6, 7, 8, 9, 10, 11]);
        assert_eq!(ppt.classify_tier(&mid, &base), MemoryTier::L2Local);

        // Diff in dim 7
        let far = PhextCoord::new([10, 20, 30, 4, 5, 6, 7, 99, 9, 10, 11]);
        assert_eq!(ppt.classify_tier(&far, &base), MemoryTier::L3Shared);

        // Diff in dim 10
        let remote = PhextCoord::new([10, 20, 30, 4, 5, 6, 7, 8, 9, 10, 12]);
        assert_eq!(ppt.classify_tier(&remote, &base), MemoryTier::Remote);
    }

    #[test]
    fn test_z_order_locality() {
        let lut = ZOrderLUT::new();
        // Adjacent in dim 0: Z-indices should differ by 1 (stride 1 in Z-order)
        let z_a = lut.encode(10, 20, 30);
        let z_b = lut.encode(11, 20, 30);
        let diff = if z_a > z_b { z_a - z_b } else { z_b - z_a };
        assert!(diff <= 8, "Z-order adjacent-dim-0 should be close, got diff={}", diff);
    }

    #[test]
    fn test_prefetch_along_dim() {
        let mut ppt = PhextPageTable::new();
        let base = PhextCoord::new([10, 20, 30, 1, 1, 1, 1, 1, 1, 1, 1]);
        let addrs = ppt.prefetch_along_dim(&base, 0, 8);
        assert_eq!(addrs.len(), 8);
        // All addresses should be in the same region (outer dims unchanged)
        // and monotonically related via Z-order
        assert!(ppt.hot_dims() & 1 == 1, "Dim 0 should be marked hot");
    }

    #[test]
    fn test_hot_dims_tracking() {
        let mut ppt = PhextPageTable::new();
        assert_eq!(ppt.hot_dims(), 0);
        ppt.touch_dim(0);
        ppt.touch_dim(3);
        ppt.touch_dim(7);
        assert_eq!(ppt.hot_dims(), (1 << 0) | (1 << 3) | (1 << 7));
    }

    #[test]
    fn test_region_allocation() {
        let mut ppt = PhextPageTable::new();
        // Two coordinates with same outer dims should share a region
        let a = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let b = PhextCoord::new([4, 5, 6, 4, 5, 6, 7, 8, 9, 10, 11]);
        ppt.translate(&a);
        ppt.translate(&b);
        assert_eq!(ppt.stats().regions_allocated, 1, "Same outer dims = same region");

        // Different outer dims = different region
        let c = PhextCoord::new([1, 2, 3, 99, 5, 6, 7, 8, 9, 10, 11]);
        ppt.translate(&c);
        assert_eq!(ppt.stats().regions_allocated, 2, "Different outer dims = new region");
    }
}
