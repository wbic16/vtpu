//! vTPU Memory Backend — PPT-backed flat memory for S-Pipe operations
//!
//! Routes gather/scatter through the Phext Page Table for address translation,
//! then reads/writes a flat Vec<u8> backing store. Phase 0: single-node only.

use crate::ppt::PhextPageTable;
use crate::phext_coord::PhextCoord;

/// PPT-backed memory for S-Pipe gather/scatter
pub struct Memory {
    /// Address translation
    pub ppt: PhextPageTable,
    /// Flat backing store (grows on demand)
    store: Vec<u8>,
}

/// Memory operation statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub reads: u64,
    pub writes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ppt: PhextPageTable::new(),
            store: vec![0u8; 16 * 1024 * 1024], // 16 MiB initial
        }
    }

    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            ppt: PhextPageTable::new(),
            store: vec![0u8; bytes],
        }
    }

    /// Read up to 8 bytes from a phext coordinate, return as i64
    pub fn gather_i64(&mut self, coord: &PhextCoord) -> i64 {
        let addr = self.ppt.translate(coord);
        self.ensure_capacity(addr + 8);
        let bytes: [u8; 8] = self.store[addr..addr + 8].try_into().unwrap();
        i64::from_le_bytes(bytes)
    }

    /// Write i64 to a phext coordinate
    pub fn scatter_i64(&mut self, coord: &PhextCoord, value: i64) {
        let addr = self.ppt.translate(coord);
        self.ensure_capacity(addr + 8);
        self.store[addr..addr + 8].copy_from_slice(&value.to_le_bytes());
    }

    /// Read arbitrary width (up to backing store size)
    pub fn gather(&mut self, coord: &PhextCoord, width: usize) -> &[u8] {
        let addr = self.ppt.translate(coord);
        self.ensure_capacity(addr + width);
        &self.store[addr..addr + width]
    }

    /// Write arbitrary width
    pub fn scatter(&mut self, coord: &PhextCoord, data: &[u8]) {
        let addr = self.ppt.translate(coord);
        self.ensure_capacity(addr + data.len());
        self.store[addr..addr + data.len()].copy_from_slice(data);
    }

    /// Grow backing store if needed (double until sufficient)
    fn ensure_capacity(&mut self, needed: usize) {
        if needed > self.store.len() {
            let mut new_size = self.store.len();
            while new_size < needed {
                new_size *= 2;
            }
            self.store.resize(new_size, 0);
        }
    }

    /// Current backing store size
    pub fn capacity(&self) -> usize {
        self.store.len()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_i64() {
        let mut mem = Memory::new();
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        mem.scatter_i64(&coord, 42);
        assert_eq!(mem.gather_i64(&coord), 42);
    }

    #[test]
    fn different_coords_independent() {
        let mut mem = Memory::new();
        let a = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = PhextCoord::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&a, 100);
        mem.scatter_i64(&b, 200);
        assert_eq!(mem.gather_i64(&a), 100);
        assert_eq!(mem.gather_i64(&b), 200);
    }

    #[test]
    fn ptc_hit_on_repeated_access() {
        let mut mem = Memory::new();
        let coord = PhextCoord::new([5, 5, 5, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&coord, 999);
        let _ = mem.gather_i64(&coord); // second access = PTC hit
        let stats = mem.ppt.stats();
        assert!(stats.ptc_hits >= 1);
    }

    #[test]
    fn auto_grow() {
        let mut mem = Memory::with_capacity(64);
        // Write to a coordinate that maps far out
        let far = PhextCoord::new([2000, 2000, 2000, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&far, 77);
        assert_eq!(mem.gather_i64(&far), 77);
        assert!(mem.capacity() > 64); // grew
    }

    #[test]
    fn gather_width() {
        let mut mem = Memory::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter(&coord, &[0xDE, 0xAD, 0xBE, 0xEF]);
        let data = mem.gather(&coord, 4);
        assert_eq!(data, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }
}

#[cfg(test)]
mod w20_tests {
    use super::*;
    use crate::phext_coord::PhextCoord;

    fn coord(x: u16) -> PhextCoord {
        PhextCoord::new([x, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }

    #[test]
    fn memory_gather_uninit_returns_zero() {
        let mut m = Memory::new();
        let v = m.gather_i64(&coord(42));
        assert_eq!(v, 0, "unwritten address returns 0");
    }

    #[test]
    fn memory_scatter_gather_roundtrip() {
        let mut m = Memory::new();
        let c = coord(7);
        m.scatter_i64(&c, 12345);
        assert_eq!(m.gather_i64(&c), 12345);
    }

    #[test]
    fn memory_overwrite_updates_value() {
        let mut m = Memory::new();
        let c = coord(3);
        m.scatter_i64(&c, 100);
        m.scatter_i64(&c, 200);
        assert_eq!(m.gather_i64(&c), 200);
    }

    #[test]
    fn memory_different_coords_independent() {
        let mut m = Memory::new();
        m.scatter_i64(&coord(1), 111);
        m.scatter_i64(&coord(2), 222);
        assert_eq!(m.gather_i64(&coord(1)), 111);
        assert_eq!(m.gather_i64(&coord(2)), 222);
    }

    #[test]
    fn memory_capacity_nonzero() {
        let m = Memory::new();
        assert!(m.capacity() > 0, "default memory has capacity");
    }

    #[test]
    fn memory_with_capacity_larger() {
        let m = Memory::with_capacity(1024 * 1024);
        assert!(m.capacity() >= 1024 * 1024);
    }
}
