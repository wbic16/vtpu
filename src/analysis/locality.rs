// Dimensional Locality Analyzer - R23 Wave 4
//
// Analyzes phext coordinate access patterns to measure dimensional locality.
// Tests the hypothesis: "coordinate locality = memory locality"

use crate::phext_coord::PhextCoord;
use crate::siw::SIW;

/// Locality level classification
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LocalityLevel {
    SameScroll,    // dims[2..] match (within 3D boundary)
    SameSection,   // dims[3..] match (within 4D boundary)
    SameChapter,   // dims[4..] match (within 5D boundary)
    SameVolume,    // dims[6..] match (within 7D boundary)
    CrossVolume,   // dims[6..] differ (crosses 7D boundary)
}

/// Locality breakdown statistics
#[derive(Debug)]
pub struct LocalityStats {
    pub same_scroll: usize,    // 2D locality
    pub same_section: usize,   // 3D locality
    pub same_chapter: usize,   // 4D locality
    pub same_volume: usize,    // 6D locality
    pub cross_volume: usize,   // 7D+ (low locality)
    pub total_transitions: usize,
}

/// Locality analysis report
#[derive(Debug)]
pub struct LocalityReport {
    pub stats: LocalityStats,
    pub predicted_cache_performance: CachePrediction,
}

/// Predicted cache performance based on locality
#[derive(Debug)]
pub struct CachePrediction {
    pub l1_hit_rate: f64,  // Based on 2D locality
    pub l2_hit_rate: f64,  // Based on 3D locality
    pub l3_hit_rate: f64,  // Based on 4D locality
}

pub struct LocalityAnalyzer;

impl LocalityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze locality of coordinate transitions in SIW stream
    pub fn analyze(&self, siws: &[SIW]) -> LocalityReport {
        if siws.len() < 2 {
            return self.empty_report();
        }

        let mut stats = LocalityStats {
            same_scroll: 0,
            same_section: 0,
            same_chapter: 0,
            same_volume: 0,
            cross_volume: 0,
            total_transitions: siws.len() - 1,
        };

        for i in 1..siws.len() {
            let level = self.classify_locality(&siws[i - 1].phext_addr, &siws[i].phext_addr);
            
            match level {
                LocalityLevel::SameScroll => stats.same_scroll += 1,
                LocalityLevel::SameSection => stats.same_section += 1,
                LocalityLevel::SameChapter => stats.same_chapter += 1,
                LocalityLevel::SameVolume => stats.same_volume += 1,
                LocalityLevel::CrossVolume => stats.cross_volume += 1,
            }
        }

        let predicted_cache_performance = self.predict_cache_performance(&stats);

        LocalityReport {
            stats,
            predicted_cache_performance,
        }
    }

    /// Classify locality level between two coordinates
    fn classify_locality(&self, a: &PhextCoord, b: &PhextCoord) -> LocalityLevel {
        let dims_a = a.dims();
        let dims_b = b.dims();

        // Check from highest dimension down
        // If higher dims match, we have higher locality
        
        // Same scroll: dims[2..] all match (3D and above constant)
        if Self::dims_match(&dims_a, &dims_b, 2) {
            return LocalityLevel::SameScroll;
        }

        // Same section: dims[3..] all match (4D and above constant)
        if Self::dims_match(&dims_a, &dims_b, 3) {
            return LocalityLevel::SameSection;
        }

        // Same chapter: dims[4..] all match (5D and above constant)
        if Self::dims_match(&dims_a, &dims_b, 4) {
            return LocalityLevel::SameChapter;
        }

        // Same volume: dims[6..] all match (7D and above constant)
        if Self::dims_match(&dims_a, &dims_b, 6) {
            return LocalityLevel::SameVolume;
        }

        // Cross-volume: dims[6..] differ
        LocalityLevel::CrossVolume
    }

    /// Check if dimensions from `start` onwards all match
    fn dims_match(a: &[u16; 11], b: &[u16; 11], start: usize) -> bool {
        a[start..].iter().zip(&b[start..]).all(|(x, y)| x == y)
    }

    /// Predict cache performance based on locality statistics
    fn predict_cache_performance(&self, stats: &LocalityStats) -> CachePrediction {
        if stats.total_transitions == 0 {
            return CachePrediction {
                l1_hit_rate: 0.0,
                l2_hit_rate: 0.0,
                l3_hit_rate: 0.0,
            };
        }

        let total = stats.total_transitions as f64;

        // L1 hit rate correlates with same-scroll (2D) locality
        // High 2D locality → high L1 hit rate
        let scroll_pct = stats.same_scroll as f64 / total;
        let l1_hit_rate = 0.50 + (scroll_pct * 0.45);  // 50-95% range

        // L2 hit rate correlates with same-section (3D) locality
        let section_pct = (stats.same_scroll + stats.same_section) as f64 / total;
        let l2_hit_rate = 0.40 + (section_pct * 0.50);  // 40-90% range

        // L3 hit rate correlates with same-chapter (4D) locality
        let chapter_pct = (stats.same_scroll + stats.same_section + stats.same_chapter) as f64 / total;
        let l3_hit_rate = 0.30 + (chapter_pct * 0.60);  // 30-90% range

        CachePrediction {
            l1_hit_rate,
            l2_hit_rate,
            l3_hit_rate,
        }
    }

    fn empty_report(&self) -> LocalityReport {
        LocalityReport {
            stats: LocalityStats {
                same_scroll: 0,
                same_section: 0,
                same_chapter: 0,
                same_volume: 0,
                cross_volume: 0,
                total_transitions: 0,
            },
            predicted_cache_performance: CachePrediction {
                l1_hit_rate: 0.0,
                l2_hit_rate: 0.0,
                l3_hit_rate: 0.0,
            },
        }
    }
}

impl LocalityReport {
    /// Print human-readable report
    pub fn print(&self) {
        println!("Dimensional Locality Analysis");
        println!("=============================");
        
        if self.stats.total_transitions == 0 {
            println!("No coordinate transitions to analyze");
            return;
        }

        let total = self.stats.total_transitions as f64;
        
        println!("Coordinate Transitions: {}", self.stats.total_transitions);
        println!();
        println!("Locality Breakdown:");
        println!("  Same scroll (2D):   {:6} ({:5.1}%)", 
            self.stats.same_scroll,
            100.0 * self.stats.same_scroll as f64 / total);
        println!("  Same section (3D):  {:6} ({:5.1}%)", 
            self.stats.same_section,
            100.0 * self.stats.same_section as f64 / total);
        println!("  Same chapter (4D):  {:6} ({:5.1}%)", 
            self.stats.same_chapter,
            100.0 * self.stats.same_chapter as f64 / total);
        println!("  Same volume (6D):   {:6} ({:5.1}%)", 
            self.stats.same_volume,
            100.0 * self.stats.same_volume as f64 / total);
        println!("  Cross-volume (7D+): {:6} ({:5.1}%)", 
            self.stats.cross_volume,
            100.0 * self.stats.cross_volume as f64 / total);
        println!();
        println!("Predicted Cache Performance:");
        println!("  L1 hit rate: {:.1}%", self.predicted_cache_performance.l1_hit_rate * 100.0);
        println!("  L2 hit rate: {:.1}%", self.predicted_cache_performance.l2_hit_rate * 100.0);
        println!("  L3 hit rate: {:.1}%", self.predicted_cache_performance.l3_hit_rate * 100.0);
    }

    /// Get percentage at each locality level
    pub fn percentages(&self) -> [f64; 5] {
        if self.stats.total_transitions == 0 {
            return [0.0; 5];
        }

        let total = self.stats.total_transitions as f64;
        [
            100.0 * self.stats.same_scroll as f64 / total,
            100.0 * self.stats.same_section as f64 / total,
            100.0 * self.stats.same_chapter as f64 / total,
            100.0 * self.stats.same_volume as f64 / total,
            100.0 * self.stats.cross_volume as f64 / total,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};

    #[test]
    fn test_same_scroll_locality() {
        let analyzer = LocalityAnalyzer::new();
        
        let coord1 = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let coord2 = PhextCoord::new([2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1]);  // dims[2..] match
        
        let level = analyzer.classify_locality(&coord1, &coord2);
        assert_eq!(level, LocalityLevel::SameScroll);
    }

    #[test]
    fn test_same_section_locality() {
        let analyzer = LocalityAnalyzer::new();
        
        let coord1 = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let coord2 = PhextCoord::new([2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1]);  // dims[3..] match
        
        let level = analyzer.classify_locality(&coord1, &coord2);
        assert_eq!(level, LocalityLevel::SameSection);
    }

    #[test]
    fn test_cross_volume_locality() {
        let analyzer = LocalityAnalyzer::new();
        
        let coord1 = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let coord2 = PhextCoord::new([2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1]);  // dims[6..] differ
        
        let level = analyzer.classify_locality(&coord1, &coord2);
        assert_eq!(level, LocalityLevel::CrossVolume);
    }

    #[test]
    fn test_locality_report_high_locality() {
        let analyzer = LocalityAnalyzer::new();
        
        // Generate SIWs with high locality (all same scroll)
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
        
        // All transitions should be same-scroll
        assert_eq!(report.stats.same_scroll, 99);
        assert_eq!(report.stats.total_transitions, 99);
        
        // Should predict high L1 hit rate
        assert!(report.predicted_cache_performance.l1_hit_rate > 0.90);
    }
}
