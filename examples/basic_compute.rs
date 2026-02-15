//! Basic Compute Kernel Example
//!
//! Demonstrates a simple phext-addressed computation:
//! Load two values → Add them → Store result

use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, PhextCoord, StreamBuilder, Scheduler};

fn main() {
    println!("=== vTPU Basic Compute Kernel ===\n");
    println!("Building kernel: z = x + y\n");

    let mut builder = StreamBuilder::new();

    // Load x from coordinate 1.1.1/1.1.1/1.1.1
    builder.push(SIW::new(
        DenseOp::DNOP,
        SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
        CoordOp::CNOP,
        PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
    ));

    // Load y from coordinate 1.1.1/1.1.1/1.1.2
    builder.push(SIW::new(
        DenseOp::DNOP,
        SparseOp::SGATHER { rd: 2, coord_idx: 1, width: 64 },
        CoordOp::CNOP,
        PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]),
    ));

    // Compute: r3 = r1 + r2
    builder.push(SIW::new(
        DenseOp::DADD { rd: 3, rs1: 1, rs2: 2 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    ));

    // Store result to coordinate 1.1.1/1.1.1/1.1.3
    builder.push(SIW::new(
        DenseOp::DNOP,
        SparseOp::SSCATTR { coord_idx: 2, rs: 3, width: 64 },
        CoordOp::CNOP,
        PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3]),
    ));

    let stream = builder.build();
    println!("Generated SIW Stream ({} instructions)\n", stream.len());

    // Execute
    let mut scheduler = Scheduler::new();
    scheduler.execute_stream(&stream);
    let telemetry = scheduler.telemetry();

    println!("Execution complete:");
    println!("  Total ops:       {}", telemetry.total_ops());
    println!("  Total cycles:    {}", telemetry.total_cycles());
    println!("  Ops/cycle:       {:.2}", telemetry.ops_per_cycle());
    println!("  Target:          3.0 ops/cycle\n");

    if telemetry.ops_per_cycle() >= 2.5 {
        println!("✅ Phase 0 gate passed ({:.1} ops/cycle)", telemetry.ops_per_cycle());
    } else {
        println!("⏳ Below Phase 0 gate — need more pipe packing");
    }
}
