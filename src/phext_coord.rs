//! Phext Coordinate Representation
//!
//! 11-dimensional coordinate packed into 128 bits.
//! Each dimension: 11 bits (addresses 2048 positions).
//! Total address space: 2048^11 ≈ 10^36 positions.
//!
//! Layout:
//! [D0:11b|D1:11b|D2:11b|D3:11b|D4:11b|D5:11b|D6:11b|D7:11b|D8:11b|D9:11b|D10:11b|Flags:7b]
//!  = 121 bits + 7 flag bits = 128 bits total

use std::fmt;

/// Phext Coordinate - 11D address in phext space
///
/// Fits exactly in one 128-bit register (SSE, half of AVX-256, quarter of AVX-512).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(16))]
pub struct PhextCoord {
    /// Lower 64 bits: dimensions 0-5 (6 × 11 bits = 66 bits used)
    lo: u64,
    
    /// Upper 64 bits: dimensions 6-10 + flags (5 × 11 bits + 7 flags = 62 bits used)
    hi: u64,
}

impl PhextCoord {
    /// Maximum value per dimension (11 bits = 2048 positions)
    pub const MAX_DIM: u16 = 2047;
    
    /// Bit mask for one dimension (11 bits)
    const DIM_MASK: u16 = 0x7FF;
    
    /// Create a new phext coordinate from 11 dimension values
    pub fn new(dims: [u16; 11]) -> Self {
        // Validate all dimensions fit in 11 bits
        assert!(dims.iter().all(|&d| d <= Self::MAX_DIM), 
                "Dimension values must fit in 11 bits (0-2047)");
        
        // Pack dimensions into 128 bits
        let lo = (dims[0] as u64) 
               | ((dims[1] as u64) << 11)
               | ((dims[2] as u64) << 22)
               | ((dims[3] as u64) << 33)
               | ((dims[4] as u64) << 44)
               | ((dims[5] as u64) << 55);
        
        let hi = (dims[6] as u64)
               | ((dims[7] as u64) << 11)
               | ((dims[8] as u64) << 22)
               | ((dims[9] as u64) << 33)
               | ((dims[10] as u64) << 44);
        
        Self { lo, hi }
    }
    
    /// Create zero coordinate (origin)
    pub fn zero() -> Self {
        Self { lo: 0, hi: 0 }
    }
    
    /// Create coordinate without validation (for testing invalid coordinates)
    ///
    /// # Safety
    /// This bypasses the validation in `new()`. Use only for testing validation logic.
    #[cfg(test)]
    pub unsafe fn new_unchecked(dims: [u16; 11]) -> Self {
        let lo = (dims[0] as u64) 
               | ((dims[1] as u64) << 11)
               | ((dims[2] as u64) << 22)
               | ((dims[3] as u64) << 33)
               | ((dims[4] as u64) << 44)
               | ((dims[5] as u64) << 55);
        
        let hi = (dims[6] as u64)
               | ((dims[7] as u64) << 11)
               | ((dims[8] as u64) << 22)
               | ((dims[9] as u64) << 33)
               | ((dims[10] as u64) << 44);
        
        Self { lo, hi }
    }
    
    /// Get dimension value (0-10)
    pub fn get_dim(&self, dim: u8) -> u16 {
        assert!(dim < 11, "Dimension index must be 0-10");
        
        if dim < 6 {
            ((self.lo >> (dim * 11)) & 0x7FF) as u16
        } else {
            ((self.hi >> ((dim - 6) * 11)) & 0x7FF) as u16
        }
    }
    
    /// Set dimension value (0-10)
    pub fn set_dim(&mut self, dim: u8, value: u16) {
        assert!(dim < 11, "Dimension index must be 0-10");
        assert!(value <= Self::MAX_DIM, "Dimension value must fit in 11 bits");
        
        if dim < 6 {
            let shift = dim * 11;
            let mask = !(0x7FFu64 << shift);
            self.lo = (self.lo & mask) | ((value as u64) << shift);
        } else {
            let shift = (dim - 6) * 11;
            let mask = !(0x7FFu64 << shift);
            self.hi = (self.hi & mask) | ((value as u64) << shift);
        }
    }
    
    /// Get all 11 dimension values
    pub fn dims(&self) -> [u16; 11] {
        [
            self.get_dim(0),
            self.get_dim(1),
            self.get_dim(2),
            self.get_dim(3),
            self.get_dim(4),
            self.get_dim(5),
            self.get_dim(6),
            self.get_dim(7),
            self.get_dim(8),
            self.get_dim(9),
            self.get_dim(10),
        ]
    }
    
    /// Get flags (7 bits in upper portion)
    pub fn flags(&self) -> u8 {
        ((self.hi >> 55) & 0x7F) as u8
    }
    
    /// Set flags (7 bits)
    pub fn set_flags(&mut self, flags: u8) {
        assert!(flags < 128, "Flags must fit in 7 bits");
        let mask = !(0x7Fu64 << 55);
        self.hi = (self.hi & mask) | ((flags as u64) << 55);
    }
    
    /// Manhattan distance to another coordinate
    pub fn manhattan_distance(&self, other: &Self) -> u32 {
        let dims_a = self.dims();
        let dims_b = other.dims();
        
        dims_a.iter().zip(dims_b.iter())
            .map(|(a, b)| (*a as i32 - *b as i32).abs() as u32)
            .sum()
    }
    
    /// Check if two coordinates differ only in one dimension
    pub fn adjacent(&self, other: &Self, dim: u8) -> bool {
        let dims_a = self.dims();
        let dims_b = other.dims();
        let dim_usize = dim as usize;
        
        for i in 0..11 {
            if i == dim_usize {
                // Allow difference in the specified dimension
                if (dims_a[i] as i32 - dims_b[i] as i32).abs() > 1 {
                    return false;
                }
            } else {
                // Other dimensions must match exactly
                if dims_a[i] != dims_b[i] {
                    return false;
                }
            }
        }
        
        true
    }
}

impl fmt::Debug for PhextCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dims = self.dims();
        write!(f, "[{}.{}.{}/{}.{}.{}/{}.{}.{}.{}.{}]",
               dims[0], dims[1], dims[2],
               dims[3], dims[4], dims[5],
               dims[6], dims[7], dims[8], dims[9], dims[10])
    }
}

// Display impl moved to display.rs module

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coord_creation() {
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(coord.get_dim(0), 1);
        assert_eq!(coord.get_dim(5), 6);
        assert_eq!(coord.get_dim(10), 11);
    }
    
    #[test]
    fn test_zero_coord() {
        let zero = PhextCoord::zero();
        for i in 0..11 {
            assert_eq!(zero.get_dim(i), 0);
        }
    }
    
    #[test]
    fn test_set_dim() {
        let mut coord = PhextCoord::zero();
        coord.set_dim(3, 42);
        assert_eq!(coord.get_dim(3), 42);
        assert_eq!(coord.get_dim(4), 0);  // Other dims unchanged
    }
    
    #[test]
    fn test_manhattan_distance() {
        let a = PhextCoord::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let b = PhextCoord::new([1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a.manhattan_distance(&b), 6);  // 1 + 2 + 3 = 6
    }
    
    #[test]
    fn test_adjacent() {
        let a = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let b = PhextCoord::new([1, 2, 3, 5, 5, 6, 7, 8, 9, 10, 11]);  // dim 3 differs by 1
        assert!(a.adjacent(&b, 3));
        
        let c = PhextCoord::new([1, 2, 3, 6, 5, 6, 7, 8, 9, 10, 11]);  // dim 3 differs by 2
        assert!(!a.adjacent(&c, 3));
    }
    
    #[test]
    fn test_alignment() {
        use std::mem;
        assert_eq!(mem::align_of::<PhextCoord>(), 16);  // 128-bit aligned
        assert_eq!(mem::size_of::<PhextCoord>(), 16);   // 128 bits = 16 bytes
    }
}
