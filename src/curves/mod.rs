// Space-Filling Curves Module - R23 Wave 4
//
// Implements space-filling curves for cache-friendly phext coordinate traversal:
// - Z-order (Morton) curve: Simple bit-interleaving
// - Hilbert curve: Locality-preserving recursive construction

pub mod zorder;
pub mod hilbert;

pub use zorder::ZOrderCurve;
pub use hilbert::HilbertCurve;
