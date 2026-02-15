// Memory Access Pattern Analyzer - R23 Wave 3
//
// Classifies memory access patterns and measures locality metrics.
// Helps predict prefetch effectiveness and memory bandwidth utilization.

use crate::phext_coord::PhextCoord;
use crate::siw::SIW;

/// Classified access pattern types
#[derive(Debug, PartialEq, Eq)]
pub enum AccessPatternType {
    Stride(u32),      // Regular stride (value = stride length)
    PointerChase,     // Dependent loads (worst for prefetch)
    GatherScatter,    // Irregular indexed access
    Tiled,            // Blocked/tiled access (cache-friendly)
    Sequential,       // Stride-1 (best case)
}

/// Locality metrics for phext coordinates
#[derive(Debug)]
pub struct LocalityMetrics {
    pub avg_manhattan_distance: f64,
    pub same_scroll_pct: f64,    // % within same 3D scroll
    pub same_section_pct: f64,   // % within same 4D section
    pub same_chapter_pct: f64,   // % within same 5D chapter
}

/// Memory access pattern analysis report
#[derive(Debug)]
pub struct MemoryPatternReport {
    pub pattern_type: AccessPatternType,
    pub confidence: f64,             // 0.0 - 1.0
    pub locality: LocalityMetrics,
    pub prefetch_score: f64,         // 0.0 - 10.0
    pub bandwidth_utilization: f64,  // 0.0 - 1.0
}

pub struct MemoryPatternAnalyzer;

impl MemoryPatternAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze SIW stream for memory access patterns
    pub fn analyze(&self, siws: &[SIW]) -> MemoryPatternReport {
        if siws.len() < 2 {
            return self.default_report();
        }

        // Extract coordinates
        let coords: Vec<&PhextCoord> = siws.iter().map(|siw| &siw.phext_addr).collect();

        // Classify pattern type
        let (pattern_type, confidence) = self.classify_pattern(&coords);

        // Measure locality
        let locality = self.measure_locality(&coords);

        // Calculate prefetch score
        let prefetch_score = self.calculate_prefetch_score(&pattern_type, &locality);

        // Estimate bandwidth utilization
        let bandwidth_utilization = self.estimate_bandwidth(&pattern_type, &locality);

        MemoryPatternReport {
            pattern_type,
            confidence,
            locality,
            prefetch_score,
            bandwidth_utilization,
        }
    }

    /// Classify the access pattern type
    fn classify_pattern(&self, coords: &[&PhextCoord]) -> (AccessPatternType, f64) {
        // Calculate distances between consecutive accesses
        let mut distances: Vec<u32> = Vec::with_capacity(coords.len() - 1);
        for i in 1..coords.len() {
            distances.push(coords[i].manhattan_distance(coords[i - 1]));
        }

        // Check for sequential (stride-1)
        let sequential_count = distances.iter().filter(|&&d| d == 1).count();
        if sequential_count as f64 / distances.len() as f64 > 0.90 {
            return (AccessPatternType::Sequential, 0.95);
        }

        // Check for regular stride
        if let Some((stride, confidence)) = self.detect_stride(&distances) {
            if confidence > 0.75 {
                return (AccessPatternType::Stride(stride), confidence);
            }
        }

        // Check for tiled access (small consistent distances)
        let avg_dist = distances.iter().sum::<u32>() as f64 / distances.len() as f64;
        let max_dist = *distances.iter().max().unwrap_or(&0);
        
        if avg_dist < 10.0 && max_dist < 50 {
            return (AccessPatternType::Tiled, 0.80);
        }

        // Check for pointer chase (highly variable distances, large gaps)
        let variance = self.calculate_variance(&distances);
        if variance > 1000.0 && max_dist > 500 {
            return (AccessPatternType::PointerChase, 0.70);
        }

        // Default: gather/scatter
        (AccessPatternType::GatherScatter, 0.60)
    }

    /// Detect regular stride pattern
    fn detect_stride(&self, distances: &[u32]) -> Option<(u32, f64)> {
        if distances.is_empty() {
            return None;
        }

        // Find most common distance
        let mut stride_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for &dist in distances {
            *stride_counts.entry(dist).or_insert(0) += 1;
        }

        let (most_common_stride, count) = stride_counts
            .iter()
            .max_by_key(|(_, &count)| count)?;

        let confidence = *count as f64 / distances.len() as f64;

        if confidence > 0.60 {
            Some((*most_common_stride, confidence))
        } else {
            None
        }
    }

    /// Calculate variance of distances
    fn calculate_variance(&self, distances: &[u32]) -> f64 {
        if distances.is_empty() {
            return 0.0;
        }

        let mean = distances.iter().sum::<u32>() as f64 / distances.len() as f64;
        let variance = distances
            .iter()
            .map(|&d| {
                let diff = d as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / distances.len() as f64;

        variance
    }

    /// Measure locality metrics
    fn measure_locality(&self, coords: &[&PhextCoord]) -> LocalityMetrics {
        if coords.len() < 2 {
            return LocalityMetrics {
                avg_manhattan_distance: 0.0,
                same_scroll_pct: 100.0,
                same_section_pct: 100.0,
                same_chapter_pct: 100.0,
            };
        }

        let mut total_distance = 0u64;
        let mut same_scroll = 0;
        let mut same_section = 0;
        let mut same_chapter = 0;

        for i in 1..coords.len() {
            total_distance += coords[i].manhattan_distance(coords[i - 1]) as u64;

            // Check dimensional locality
            if Self::same_scroll(coords[i], coords[i - 1]) {
                same_scroll += 1;
            }
            if Self::same_section(coords[i], coords[i - 1]) {
                same_section += 1;
            }
            if Self::same_chapter(coords[i], coords[i - 1]) {
                same_chapter += 1;
            }
        }

        let n = coords.len() - 1;

        LocalityMetrics {
            avg_manhattan_distance: total_distance as f64 / n as f64,
            same_scroll_pct: 100.0 * same_scroll as f64 / n as f64,
            same_section_pct: 100.0 * same_section as f64 / n as f64,
            same_chapter_pct: 100.0 * same_chapter as f64 / n as f64,
        }
    }

    /// Check if coordinates are in same scroll (dimensions 3+ match)
    fn same_scroll(a: &PhextCoord, b: &PhextCoord) -> bool {
        a.dims()[2..].iter().zip(&b.dims()[2..]).all(|(x, y)| x == y)
    }

    /// Check if coordinates are in same section (dimensions 4+ match)
    fn same_section(a: &PhextCoord, b: &PhextCoord) -> bool {
        a.dims()[3..].iter().zip(&b.dims()[3..]).all(|(x, y)| x == y)
    }

    /// Check if coordinates are in same chapter (dimensions 5+ match)
    fn same_chapter(a: &PhextCoord, b: &PhextCoord) -> bool {
        a.dims()[4..].iter().zip(&b.dims()[4..]).all(|(x, y)| x == y)
    }

    /// Calculate prefetch effectiveness score (0.0 - 10.0)
    fn calculate_prefetch_score(&self, pattern: &AccessPatternType, locality: &LocalityMetrics) -> f64 {
        let base_score = match pattern {
            AccessPatternType::Sequential => 10.0,
            AccessPatternType::Stride(stride) if *stride <= 8 => 9.0,
            AccessPatternType::Stride(stride) if *stride <= 64 => 7.0,
            AccessPatternType::Stride(_) => 5.0,
            AccessPatternType::Tiled => 8.0,
            AccessPatternType::GatherScatter => 3.0,
            AccessPatternType::PointerChase => 1.0,
        };

        // Adjust based on locality
        let locality_bonus = if locality.same_scroll_pct > 90.0 {
            0.5
        } else if locality.same_section_pct > 90.0 {
            0.3
        } else {
            0.0
        };

        f64::min(base_score + locality_bonus, 10.0)
    }

    /// Estimate memory bandwidth utilization (0.0 - 1.0)
    fn estimate_bandwidth(&self, pattern: &AccessPatternType, locality: &LocalityMetrics) -> f64 {
        // Sequential access achieves highest bandwidth
        let pattern_efficiency = match pattern {
            AccessPatternType::Sequential => 0.95,
            AccessPatternType::Stride(stride) if *stride <= 8 => 0.85,
            AccessPatternType::Stride(stride) if *stride <= 64 => 0.65,
            AccessPatternType::Stride(_) => 0.45,
            AccessPatternType::Tiled => 0.75,
            AccessPatternType::GatherScatter => 0.35,
            AccessPatternType::PointerChase => 0.15,
        };

        // Adjust for locality (better locality = better bandwidth)
        let locality_factor = locality.same_section_pct / 100.0;

        pattern_efficiency * (0.7 + 0.3 * locality_factor)
    }

    fn default_report(&self) -> MemoryPatternReport {
        MemoryPatternReport {
            pattern_type: AccessPatternType::Sequential,
            confidence: 0.0,
            locality: LocalityMetrics {
                avg_manhattan_distance: 0.0,
                same_scroll_pct: 0.0,
                same_section_pct: 0.0,
                same_chapter_pct: 0.0,
            },
            prefetch_score: 0.0,
            bandwidth_utilization: 0.0,
        }
    }
}

impl MemoryPatternReport {
    /// Print human-readable report
    pub fn print(&self) {
        println!("Memory Access Pattern Analysis");
        println!("==============================");
        println!("Pattern type: {:?} ({:.0}% confidence)", 
            self.pattern_type, self.confidence * 100.0);
        println!();
        println!("Locality metrics:");
        println!("  Average Manhattan distance: {:.1}", self.locality.avg_manhattan_distance);
        println!("  Same scroll: {:.1}%", self.locality.same_scroll_pct);
        println!("  Same section: {:.1}%", self.locality.same_section_pct);
        println!("  Same chapter: {:.1}%", self.locality.same_chapter_pct);
        println!();
        println!("Performance prediction:");
        println!("  Prefetch score: {:.1}/10.0 ({})", 
            self.prefetch_score,
            Self::score_rating(self.prefetch_score)
        );
        println!("  Bandwidth utilization: {:.1}% ({})", 
            self.bandwidth_utilization * 100.0,
            Self::utilization_rating(self.bandwidth_utilization)
        );
    }

    fn score_rating(score: f64) -> &'static str {
        match score {
            s if s >= 9.0 => "excellent",
            s if s >= 7.0 => "good",
            s if s >= 5.0 => "fair",
            s if s >= 3.0 => "poor",
            _ => "very poor",
        }
    }

    fn utilization_rating(util: f64) -> &'static str {
        match util {
            u if u >= 0.80 => "excellent",
            u if u >= 0.60 => "good",
            u if u >= 0.40 => "fair",
            u if u >= 0.20 => "poor",
            _ => "very poor",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::siw::SIW;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};

    #[test]
    fn test_sequential_pattern() {
        let analyzer = MemoryPatternAnalyzer::new();
        
        // Sequential coordinates (stride-1)
        let siws: Vec<SIW> = (0..100)
            .map(|i| {
                SIW::new(
                    DenseOp::DNOP,
                    SparseOp::SNOP,
                    CoordOp::CNOP,
                    PhextCoord::new([i as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                )
            })
            .collect();

        let report = analyzer.analyze(&siws);
        
        assert_eq!(report.pattern_type, AccessPatternType::Sequential);
        assert!(report.confidence > 0.90);
        assert!(report.prefetch_score > 9.0);
    }

    #[test]
    fn test_stride_pattern() {
        let analyzer = MemoryPatternAnalyzer::new();
        
        // Stride-8 pattern
        let siws: Vec<SIW> = (0..100)
            .map(|i| {
                SIW::new(
                    DenseOp::DNOP,
                    SparseOp::SNOP,
                    CoordOp::CNOP,
                    PhextCoord::new([(i * 8) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                )
            })
            .collect();

        let report = analyzer.analyze(&siws);
        
        assert!(matches!(report.pattern_type, AccessPatternType::Stride(_)));
        assert!(report.prefetch_score > 7.0);
    }
}
