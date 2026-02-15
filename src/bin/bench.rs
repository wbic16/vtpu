// vTPU Benchmark Runner
// W2: Structure validation. W3+: Performance benchmarks.

use vtpu_runtime::{SIW, PhextCoord, DenseOp, SparseOp, CoordOp, Scheduler, StreamBuilder};

fn main() {
    println!("vTPU Benchmark Suite - R23");
    println!("==========================\n");

    println!("Phase 1 (W2): Structure validation");
    test_siw_creation();
    test_phext_coord();
    test_scheduler();

    println!("\nPhase 2 (W3+): Performance benchmarks");
    println!("  [Awaiting PPT + real cache measurement]");

    println!("\n✅ W2 validation complete");
}

fn test_siw_creation() {
    let siw = SIW::new(
        DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
        SparseOp::SGATHER { rd: 3, coord_idx: 0, width: 64 },
        CoordOp::CNOP,
        PhextCoord::zero(),
    );

    println!("  ✓ SIW creation: {} bytes, {}-byte aligned",
        std::mem::size_of::<SIW>(),
        std::mem::align_of::<SIW>()
    );

    // SIW should be cache-line aligned (64 bytes)
    assert_eq!(std::mem::align_of::<SIW>(), 64);
}

fn test_phext_coord() {
    let coord = PhextCoord::new([3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]);
    println!("  ✓ Phext coordinate: {}", coord);

    let zero = PhextCoord::zero();
    println!("  ✓ Zero coordinate: {}", zero);
}

fn test_scheduler() {
    let mut builder = StreamBuilder::new();
    builder.push(SIW::new(
        DenseOp::DMOV { rd: 0, imm: 42 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    ));
    builder.push(SIW::new(
        DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    ));

    let stream = builder.build();
    let mut scheduler = Scheduler::new();
    scheduler.execute_stream(&stream);
    let t = scheduler.telemetry();

    println!("  ✓ Scheduler: {} SIWs, {} ops, {:.1} ops/cycle",
        stream.len(), t.total_ops(), t.ops_per_cycle());
}
