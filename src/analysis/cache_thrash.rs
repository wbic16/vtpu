// Cache Thrashing Detector - R23 Wave 3
//
// Analyzes coordinate access patterns to detect cache thrashing.
// Tracks working set size and estimates cache miss rates for L1/L2/L3.

use crate::phext_coord::PhextCoord;
use crate::siw::SIW;
use std::collections::HashSet;

/// Cache tier sizes (AMD R9 8945HS)
const L1_SIZE_BYTES: usize = 32 * 1024;      // 32 KB per core
const L2_SIZE_BYTES: usize = 512 * 1024;     // 512 KB per core
const L3_SIZE_BYTES: usize = 16 * 1024 * 1024; // 16 MB shared

const COORD_SIZE_BYTES: usize = 16; // PhextCoord = 16 bytes

/// Estimated cache hit rates for each tier
#[derive(Debug)]
pub struct CacheHitRates {
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub l3_hit_rate: f64,
}

/// Access pattern classification
#[derive(Debug, PartialEq, Eq)]
pub enum AccessPattern {
    LinearSweep,    // Sequential access exceeding cache
    PingPong,       // Alternating between distant coordinates
    RandomSpray,    // Uniformly random (worst case)
    Tiled,          // Blocked access (cache-friendly)
}

/// Cache thrashing analysis report
#[derive(Debug)]
pub struct CacheThrashReport {
    pub working_set_coords: usize,
    pub working_set_bytes: usize,
    pub hit_rates: CacheHitRates,
    pub access_pattern: AccessPattern,
    pub recommendation: String,
}

pub struct CacheThrashDetector;

impl CacheThrashDetector {
    pub fn new() -> Self {
        Self
    }

    /// Analyze SIW stream for cache thrashing
    pub fn analyze(&self, siws: &[SIW]) -> CacheThrashReport {
        // Extract all unique coordinates accessed
        let unique_coords = self.extract_unique_coordinates(siws);
        let working_set_coords = unique_coords.len();
        let working_set_bytes = working_set_coords * COORD_SIZE_BYTES;

        // Estimate cache hit rates based on working set size
        let hit_rates = self.estimate_hit_rates(working_set_bytes);

        // Classify access pattern
        let access_pattern = self.classify_pattern(siws, working_set_coords);

        // Generate recommendation
        let recommendation = self.generate_recommendation(
            working_set_bytes,
            &hit_rates,
            &access_pattern,
        );

        CacheThrashReport {
            working_set_coords,
            working_set_bytes,
            hit_rates,
            access_pattern,
            recommendation,
        }
    }

    /// Extract all unique coordinates from SIW stream
    fn extract_unique_coordinates(&self, siws: &[SIW]) -> HashSet<u64> {
        let mut coords = HashSet::new();

        for siw in siws {
            // Hash the coordinate to a u64 for uniqueness tracking
            let coord_hash = self.hash_coordinate(&siw.phext_addr);
            coords.insert(coord_hash);
        }

        coords
    }

    /// Simple hash function for PhextCoord
    fn hash_coordinate(&self, coord: &PhextCoord) -> u64 {
        // Combine all dimensions into a single hash
        coord.dims().iter().enumerate().fold(0u64, |acc, (i, &dim)| {
            acc.wrapping_mul(2654435761).wrapping_add((dim as u64) << (i * 5))
        })
    }

    /// Estimate cache hit rates based on working set size
    fn estimate_hit_rates(&self, working_set_bytes: usize) -> CacheHitRates {
        // Simple model: if working set fits in cache, high hit rate
        // If exceeds cache significantly, low hit rate
        
        let l1_hit_rate = if working_set_bytes <= L1_SIZE_BYTES {
            0.99 // Fits entirely in L1
        } else if working_set_bytes <= 2 * L1_SIZE_BYTES {
            0.70 // Partial thrashing
        } else if working_set_bytes <= 4 * L1_SIZE_BYTES {
            0.30 // Heavy thrashing
        } else {
            0.08 // Completely exceeds L1
        };

        let l2_hit_rate = if working_set_bytes <= L2_SIZE_BYTES {
            0.95
        } else if working_set_bytes <= 2 * L2_SIZE_BYTES {
            0.60
        } else if working_set_bytes <= 4 * L2_SIZE_BYTES {
            0.25
        } else {
            0.12
        };

        let l3_hit_rate = if working_set_bytes <= L3_SIZE_BYTES {
            0.92
        } else if working_set_bytes <= 2 * L3_SIZE_BYTES {
            0.55
        } else {
            0.18
        };

        CacheHitRates {
            l1_hit_rate,
            l2_hit_rate,
            l3_hit_rate,
        }
    }

    /// Classify the access pattern
    fn classify_pattern(&self, siws: &[SIW], _working_set_size: usize) -> AccessPattern {
        if siws.len() < 10 {
            return AccessPattern::Tiled; // Too small to classify
        }

        // Sample first 100 SIWs to detect pattern
        let sample_size = siws.len().min(100);
        let mut distances: Vec<u32> = Vec::with_capacity(sample_size - 1);

        for i in 1..sample_size {
            let dist = siws[i].phext_addr.manhattan_distance(&siws[i - 1].phext_addr);
            distances.push(dist);
        }

        let avg_distance = distances.iter().sum::<u32>() as f64 / distances.len() as f64;
        let max_distance = *distances.iter().max().unwrap_or(&0);

        // Classification heuristics
        if avg_distance < 5.0 && max_distance < 20 {
            AccessPattern::Tiled // Small, consistent distances
        } else if max_distance > 1000 && distances.len() > 10 {
            let ping_pong_count = distances
                .windows(2)
                .filter(|w| (w[0] as i32 - w[1] as i32).abs() > 500)
                .count();
            if ping_pong_count > distances.len() / 3 {
                AccessPattern::PingPong
            } else {
                AccessPattern::RandomSpray
            }
        } else if avg_distance > 100.0 {
            AccessPattern::RandomSpray
        } else {
            AccessPattern::LinearSweep
        }
    }

    /// Generate recommendation based on analysis
    fn generate_recommendation(
        &self,
        working_set_bytes: usize,
        _hit_rates: &CacheHitRates,
        pattern: &AccessPattern,
    ) -> String {
        let working_set_kb = working_set_bytes / 1024;

        if working_set_bytes <= L1_SIZE_BYTES {
            return format!(
                "✅ Working set ({} KB) fits in L1 cache. Excellent locality.",
                working_set_kb
            );
        }

        if working_set_bytes <= L2_SIZE_BYTES {
            return format!(
                "⚠️  Working set ({} KB) exceeds L1 but fits in L2. Consider tiling to 4 KB blocks.",
                working_set_kb
            );
        }

        if working_set_bytes <= L3_SIZE_BYTES {
            match pattern {
                AccessPattern::LinearSweep => {
                    format!(
                        "⚠️  Linear sweep of {} KB. Tile into 64 KB blocks for L2 locality.",
                        working_set_kb
                    )
                }
                AccessPattern::PingPong => {
                    format!(
                        "❌ Ping-pong pattern detected ({} KB working set). Severe thrashing. Reorder accesses.",
                        working_set_kb
                    )
                }
                AccessPattern::RandomSpray => {
                    format!(
                        "❌ Random access over {} KB. Consider space-filling curve (Hilbert/Z-order).",
                        working_set_kb
                    )
                }
                AccessPattern::Tiled => {
                    format!(
                        "✅ Tiled access detected. Optimize tile size for L2 (512 KB).",
                    )
                }
            }
        } else {
            format!(
                "❌ Working set ({} KB) exceeds L3 (16 MB). RAM-bound. Reduce working set or use streaming.",
                working_set_kb
            )
        }
    }
}

impl CacheThrashReport {
    /// Print human-readable report
    pub fn print(&self) {
        println!("Cache Thrashing Analysis");
        println!("========================");
        println!("Working set: {} coordinates ({} KB)", 
            self.working_set_coords,
            self.working_set_bytes / 1024
        );
        println!("Access pattern: {:?}", self.access_pattern);
        println!();
        println!("Estimated cache hit rates:");
        println!("  L1 (32 KB): {:.1}%", self.hit_rates.l1_hit_rate * 100.0);
        println!("  L2 (512 KB): {:.1}%", self.hit_rates.l2_hit_rate * 100.0);
        println!("  L3 (16 MB): {:.1}%", self.hit_rates.l3_hit_rate * 100.0);
        println!();
        println!("Recommendation: {}", self.recommendation);
    }

    /// Return L1 miss rate as percentage
    pub fn l1_miss_rate(&self) -> f64 {
        (1.0 - self.hit_rates.l1_hit_rate) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::siw::SIW;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};

    #[test]
    fn test_small_working_set() {
        let detector = CacheThrashDetector::new();
        
        // 100 SIWs with same coordinate (minimal working set)
        let siws: Vec<SIW> = (0..100)
            .map(|_| {
                SIW::new(
                    DenseOp::DNOP,
                    SparseOp::SNOP,
                    CoordOp::CNOP,
                    PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                )
            })
            .collect();

        let report = detector.analyze(&siws);
        
        assert_eq!(report.working_set_coords, 1);
        assert!(report.hit_rates.l1_hit_rate > 0.95);
    }

    #[test]
    fn test_large_working_set() {
        let detector = CacheThrashDetector::new();
        
        // 4000 SIWs with unique coordinates (exceeds L1 cache)
        // Use two dimensions to create 4000 unique coords: (i % 100, i / 100)
        let siws: Vec<SIW> = (0..4000)
            .map(|i| {
                SIW::new(
                    DenseOp::DNOP,
                    SparseOp::SNOP,
                    CoordOp::CNOP,
                    PhextCoord::new([
                        (i % 100 + 1) as u16,
                        (i / 100 + 1) as u16,
                        1, 1, 1, 1, 1, 1, 1, 1, 1
                    ]),
                )
            })
            .collect();

        let report = detector.analyze(&siws);
        
        assert_eq!(report.working_set_coords, 4000);
        assert!(report.working_set_bytes > L1_SIZE_BYTES);
        // Working set is 4000 * 16 = 64KB, which is 2x L1 (32KB)
        // Our estimate should show partial thrashing
        assert!(report.hit_rates.l1_hit_rate < 0.80);
    }
}
