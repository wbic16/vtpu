// vTPU Benchmark Runner
// W3+: Will contain synthetic SIW benchmarks

use vtpu::{SIW, PhextCoord, DenseOp, SparseOp, CoordOp};

fn main() {
    println!("vTPU Benchmark Suite - R23");
    println!("==========================\n");
    
    // W2: Basic structure validation
    println!("Phase 1 (W2): Structure validation");
    test_siw_creation();
    test_phext_coord();
    
    println!("\nPhase 2 (W3+): Performance benchmarks");
    println!("  [Not yet implemented - awaiting scheduler]");
    
    println!("\n✅ W2 validation complete");
}

fn test_siw_creation() {
    let siw = SIW::new(
        DenseOp::DADD(0, 1, 2),
        SparseOp::SGATHER(3, 64),
        CoordOp::CPACK(0, 4, 5, 0),
        PhextCoord::zero(),
    );
    
    println!("  ✓ SIW creation: {} bytes, {}-byte aligned",
        std::mem::size_of::<SIW>(),
        std::mem::align_of::<SIW>()
    );
    
    assert_eq!(std::mem::size_of::<SIW>(), 64);
    assert_eq!(std::mem::align_of::<SIW>(), 64);
}

fn test_phext_coord() {
    let coord = PhextCoord::from_string("3.1.4 / 1.5.9 / 2.6.5").unwrap();
    
    println!("  ✓ Phext coordinate: {} (parsed from string)",
        coord.to_string()
    );
    
    assert_eq!(coord.to_string(), "3.1.4 / 1.5.9 / 2.6.5");
}
