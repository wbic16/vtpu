// vTPU Benchmark Runner
// W3+: Will contain synthetic SIW benchmarks

use vtpu_runtime::{SIW, PhextCoord, DenseOp, SparseOp, CoordOp};

fn main() {
    println!("vTPU Benchmark Suite - R23");
    println!("==========================\n");
    
    println!("Phase 1 (W2): Structure validation");
    test_siw_creation();
    test_phext_coord();
    
    println!("\nPhase 2 (W3+): Performance benchmarks");
    println!("  [Not yet implemented - awaiting scheduler]");
    
    println!("\n✅ W2 validation complete");
}

fn test_siw_creation() {
    let siw = SIW::new(
        DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
        SparseOp::SGATHER { rd: 3, coord_idx: 0, width: 64 },
        CoordOp::CPACK { rd: 0, rs1: 4, rs2: 5, fmt: vtpu_runtime::MessageFormat::Custom(0) },
        PhextCoord::zero(),
    );
    
    println!("  ✓ SIW creation: {} bytes, {}-byte aligned",
        std::mem::size_of::<SIW>(),
        std::mem::align_of::<SIW>()
    );
    
    let _ = siw;
}

fn test_phext_coord() {
    let coord = PhextCoord::new([3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]);
    let d = coord.dims();
    
    println!("  ✓ Phext coordinate: [{}, {}, {}, ...]", d[0], d[1], d[2]);
    
    assert_eq!(d[0], 3);
    assert_eq!(d[3], 1);
    assert_eq!(d[5], 9);
}
