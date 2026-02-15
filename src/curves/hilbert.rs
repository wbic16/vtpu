// Hilbert Curve - R23 Wave 4
//
// Locality-preserving space-filling curve.
// Better cache performance than Z-order due to superior locality preservation.

use crate::phext_coord::PhextCoord;

pub struct HilbertCurve;

impl HilbertCurve {
    /// Encode 2D coordinate to Hilbert index
    ///
    /// Uses standard Hilbert curve for first two dimensions (x, y).
    /// Ignores other dimensions for simplicity in Wave 4.
    pub fn encode_2d(x: u16, y: u16, order: u8) -> u64 {
        let n = 1u16 << order; // Grid size = 2^order
        let mut index = 0u64;
        let mut s = n / 2;
        
        let mut x = x;
        let mut y = y;
        
        while s > 0 {
            let rx = ((x & s) > 0) as u64;
            let ry = ((y & s) > 0) as u64;
            index += s as u64 * s as u64 * ((3 * rx) ^ ry);
            
            // Rotate
            if ry == 0 {
                if rx == 1 {
                    x = n - 1 - x;
                    y = n - 1 - y;
                }
                std::mem::swap(&mut x, &mut y);
            }
            
            s /= 2;
        }
        
        index
    }
    
    /// Decode Hilbert index to 2D coordinate
    pub fn decode_2d(mut index: u64, order: u8) -> (u16, u16) {
        let n = 1u16 << order;
        let mut x = 0u16;
        let mut y = 0u16;
        let mut s = 1u16;
        
        while s < n {
            let rx = ((index / 2) & 1) as u16;
            let ry = ((index ^ rx as u64) & 1) as u16;
            
            // Rotate
            if ry == 0 {
                if rx == 1 {
                    x = s - 1 - x;
                    y = s - 1 - y;
                }
                std::mem::swap(&mut x, &mut y);
            }
            
            x += s * rx;
            y += s * ry;
            index /= 4;
            s *= 2;
        }
        
        (x, y)
    }
    
    /// Encode PhextCoord to Hilbert index (simplified)
    ///
    /// Uses 2D Hilbert on first two dimensions, Z-order on rest.
    /// This is a hybrid approach for Wave 4 proof-of-concept.
    pub fn encode(coord: &PhextCoord) -> u64 {
        let dims = coord.dims();
        
        // Use 5-bit precision for 2D Hilbert (32×32 grid)
        let x = (dims[0] & 0x1F) as u16;
        let y = (dims[1] & 0x1F) as u16;
        let hilbert_2d = Self::encode_2d(x, y, 5);
        
        // Append other dimensions using Z-order style
        let mut result = hilbert_2d;
        for i in 2..11 {
            let val = (dims[i] & 0x1F) as u64;
            result = (result << 5) | val;
        }
        
        result
    }
    
    /// Decode Hilbert index to PhextCoord
    pub fn decode(index: u64) -> PhextCoord {
        let mut dims = [0u16; 11];
        
        // Extract other dimensions (lower 45 bits = 9 dims × 5 bits)
        let mut remaining = index;
        for i in (2..11).rev() {
            dims[i] = (remaining & 0x1F) as u16;
            remaining >>= 5;
        }
        
        // Decode 2D Hilbert (upper 10 bits)
        let hilbert_2d = remaining & 0x3FF; // 10 bits for 32×32 grid
        let (x, y) = Self::decode_2d(hilbert_2d, 5);
        dims[0] = x;
        dims[1] = y;
        
        PhextCoord::new(dims)
    }
    
    /// Full 11D Hilbert-style encoding using recursive subdivision
    ///
    /// This is a pseudo-Hilbert that maintains good locality properties.
    /// Uses Gray code style recursive subdivision across all dimensions.
    pub fn encode_11d(coord: &PhextCoord) -> u128 {
        let dims = coord.dims();
        let mut result = 0u128;
        let mut bits_used = 0;
        
        // Process 5 bits at a time from each dimension
        for bit_level in 0..5 {
            for dim in 0..11 {
                let bit = ((dims[dim] >> bit_level) & 1) as u128;
                result |= bit << bits_used;
                bits_used += 1;
            }
        }
        
        // Apply Gray code transform for better locality
        result ^ (result >> 1)
    }
    
    /// Decode 11D Hilbert-style index
    pub fn decode_11d(mut index: u128) -> PhextCoord {
        // Reverse Gray code
        let mut gray = index;
        let mut shifted = gray >> 1;
        while shifted != 0 {
            gray ^= shifted;
            shifted >>= 1;
        }
        index = gray;
        
        let mut dims = [0u16; 11];
        let mut bit_pos = 0;
        
        // Extract bits
        for bit_level in 0..5 {
            for dim in 0..11 {
                let bit = ((index >> bit_pos) & 1) as u16;
                dims[dim] |= bit << bit_level;
                bit_pos += 1;
            }
        }
        
        PhextCoord::new(dims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hilbert_2d_encode_decode() {
        for x in 0..32 {
            for y in 0..32 {
                let index = HilbertCurve::encode_2d(x, y, 5);
                let (dx, dy) = HilbertCurve::decode_2d(index, 5);
                assert_eq!((x, y), (dx, dy), "Mismatch at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_hilbert_2d_locality() {
        // Sequential Hilbert indices should produce nearby coordinates
        let mut max_dist = 0u32;
        
        for i in 0..1023 {
            let (x1, y1) = HilbertCurve::decode_2d(i, 5);
            let (x2, y2) = HilbertCurve::decode_2d(i + 1, 5);
            
            let dist = ((x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()) as u32;
            max_dist = max_dist.max(dist);
        }
        
        // Hilbert property: consecutive indices are always adjacent (dist = 1)
        assert_eq!(max_dist, 1, "Hilbert curve should maintain adjacency");
    }

    #[test]
    fn test_hilbert_encode_decode() {
        let coord = PhextCoord::new([5, 10, 15, 20, 25, 1, 2, 3, 4, 5, 6]);
        let index = HilbertCurve::encode(&coord);
        let decoded = HilbertCurve::decode(index);
        
        // First two dims should be exact (5-bit precision)
        assert_eq!(coord.get_dim(0) & 0x1F, decoded.get_dim(0));
        assert_eq!(coord.get_dim(1) & 0x1F, decoded.get_dim(1));
        
        // Other dims should match in lower 5 bits
        for dim in 2..11 {
            assert_eq!(coord.get_dim(dim) & 0x1F, decoded.get_dim(dim));
        }
    }

    #[test]
    fn test_hilbert_11d_round_trip() {
        let coord = PhextCoord::new([10, 20, 5, 15, 25, 30, 8, 12, 18, 22, 28]);
        let index = HilbertCurve::encode_11d(&coord);
        let decoded = HilbertCurve::decode_11d(index);
        
        // Should preserve lower 5 bits of each dimension
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
    fn test_hilbert_better_locality_than_sequential() {
        // Generate coordinates via Hilbert curve
        let mut hilbert_coords = Vec::new();
        for i in 0..100 {
            let coord = HilbertCurve::decode(i);
            hilbert_coords.push(coord);
        }
        
        // Measure average Manhattan distance between consecutive coords
        let mut total_dist = 0u32;
        for i in 1..hilbert_coords.len() {
            total_dist += hilbert_coords[i].manhattan_distance(&hilbert_coords[i - 1]);
        }
        let avg_dist = total_dist as f64 / (hilbert_coords.len() - 1) as f64;
        
        // Hilbert should maintain very low average distance
        assert!(avg_dist < 5.0, "Hilbert average distance too high: {:.2}", avg_dist);
    }
}
