// vTPU Analysis Module - R23 Wave 3
//
// Performance analysis tools for vTPU SIW streams:
// - Port conflict detection (Zen 4 execution ports)
// - Cache thrashing analysis (L1/L2/L3 working sets)
// - Memory access pattern classification

pub mod port_conflicts;
pub mod cache_thrash;
pub mod memory_patterns;

pub use port_conflicts::PortConflictAnalyzer;
pub use cache_thrash::CacheThrashDetector;
pub use memory_patterns::MemoryPatternAnalyzer;
