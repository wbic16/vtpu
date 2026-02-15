// Z-Order (Morton) Curve - R23 Wave 4
//
// Maps PhextCoord to linear index via bit-interleaving.
// Preserves some locality but not as well as Hilbert curve.

use crate::phext_coord::PhextCoord;

pub struct ZOrderCurve;

impl ZOrderCurve {
    /// Encode PhextCoord to Z-order index (u64)
    ///
    /// Takes lower 5 bits of each dimension (11 dims × 5 bits = 55 bits total).
    /// Interleaves bits: d0[0], d1[0], d2[0], ..., d10[0], d0[1], d1[1], ...
    pub fn encode(coord: &PhextCoord) -> u64 {
        let dims = coord.dims();
        let mut result = 0u64;
        
        // Interleave 5 bits from each of 11 dimensions
        for bit in 0..5 {
            for dim in 0..11 {
                if dims[dim] & (1 << bit) != 0 {
                    let output_bit = bit * 11 + dim;
                    result |= 1u64 << output_bit;
                }
            }
        }
        
        result
    }
    
    /// Decode Z-order index back to PhextCoord
    ///
    /// Reverses the bit-interleaving to reconstruct coordinate.
    /// Only recovers the lower 5 bits of each dimension.
    pub fn decode(index: u64) -> PhextCoord {
        let mut dims = [0u16; 11];
        
        // De-interleave bits
        for bit in 0..5 {
            for dim in 0..11 {
                let input_bit = bit * 11 + dim;
                if index & (1u64 << input_bit) != 0 {
                    dims[dim] |= 1u16 << bit;
                }
            }
        }
        
        PhextCoord::new(dims)
    }
    
    /// Encode using full 11 bits per dimension into u128
    ///
    /// For cases where we need full precision (121 bits total).
    pub fn encode_u128(coord: &PhextCoord) -> u128 {
        let dims = coord.dims();
        let mut result = 0u128;
        
        // Interleave all 11 bits from each of 11 dimensions
        for bit in 0..11 {
            for dim in 0..11 {
                if dims[dim] & (1 << bit) != 0 {
                    let output_bit = bit * 11 + dim;
                    result |= 1u128 << output_bit;
                }
            }
        }
        
        result
    }
    
    /// Decode from u128 Z-order index
    pub fn decode_u128(index: u128) -> PhextCoord {
        let mut dims = [0u16; 11];
        
        // De-interleave bits
        for bit in 0..11 {
            for dim in 0..11 {
                let input_bit = bit * 11 + dim;
                if index & (1u128 << input_bit) != 0 {
                    dims[dim] |= 1u16 << bit;
                }
            }
        }
        
        PhextCoord::new(dims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zorder_encode_decode_u64() {
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let index = ZOrderCurve::encode(&coord);
        let decoded = ZOrderCurve::decode(index);
        
        // Lower 5 bits should match (values 0-31)
        for dim in 0..11 {
            assert_eq!(
                coord.get_dim(dim) & 0x1F,
                decoded.get_dim(dim),
                "Dimension {} mismatch",
                dim
            );
        }
    }

    #[test]
    fn test_zorder_encode_decode_u128() {
        let coord = PhextCoord::new([
            100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100
        ]);
        let index = ZOrderCurve::encode_u128(&coord);
        let decoded = ZOrderCurve::decode_u128(index);
        
        // All 11 bits should match exactly
        for dim in 0..11 {
            assert_eq!(
                coord.get_dim(dim),
                decoded.get_dim(dim),
                "Dimension {} mismatch",
                dim
            );
        }
    }

    #[test]
    fn test_zorder_sequential_has_locality() {
        // Sequential indices should produce nearby coordinates
        let coord1 = ZOrderCurve::decode(100);
        let coord2 = ZOrderCurve::decode(101);
        
        // Manhattan distance should be small (typically 1)
        let dist = coord1.manhattan_distance(&coord2);
        assert!(dist <= 2, "Sequential Z-order indices should be nearby: dist={}", dist);
    }

    #[test]
    fn test_zorder_origin() {
        let origin = PhextCoord::zero();
        let index = ZOrderCurve::encode(&origin);
        assert_eq!(index, 0);
        
        let decoded = ZOrderCurve::decode(0);
        assert_eq!(decoded.dims(), origin.dims());
    }
}
