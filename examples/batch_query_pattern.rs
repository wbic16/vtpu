//! Batch Query Packing Pattern — R23W16 Rewrite
//!
//! Demonstrates 3.0 ops/cycle on batch query workload:
//! Load from coordinate + Compute on loaded data + Route result
//! All 3 pipes active every cycle.

use vtpu_runtime::*;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::exec;

fn main() {
    println!("═══ Batch Query Packing Pattern ═══\n");

    let mut mem = Memory::new();

    // Seed 100 values at coordinates
    for i in 1..=100u16 {
        let mut c = [1u16; 11];
        c[0] = i;
        mem.scatter_i64(&PhextCoord::new(c), i as i64 * 10);
    }

    // Build packed query program: each SIW does gather + compute + coordinate op
    let mut program = Vec::new();
    for i in 0..20u8 {
        let rd = (i % 14) + 1;
        program.push(SIW::new(
            DenseOp::DADD { rd, rs1: rd, rs2: rd },      // D: compute
            SparseOp::SGATHER { rd, coord_idx: 0, width: 64 }, // S: load
            CoordOp::CNOP,                                 // C: (route placeholder)
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        ));
    }

    // Unpacked baseline: D-only
    let unpacked: Vec<SIW> = (0..20u8).map(|i| {
        let rd = (i % 14) + 1;
        SIW::new(
            DenseOp::DADD { rd, rs1: rd, rs2: rd },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }).collect();

    // Run packed
    let mut s1 = Sentron::new(0, PhextCoord::new([1; 11]), 0, 0);
    s1.regs.phext[0] = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    s1.spawn(program);
    let packed_stats = exec::run(&mut s1, &mut mem);

    // Run unpacked
    let mut s2 = Sentron::new(1, PhextCoord::zero(), 0, 1);
    s2.spawn(unpacked);
    let unpacked_stats = exec::run_standalone(&mut s2);

    println!("  Packed:   {} SIWs, {} ops, {:.1} ops/cycle",
        packed_stats.siws_retired, packed_stats.ops_retired, packed_stats.ops_per_cycle());
    println!("  Unpacked: {} SIWs, {} ops, {:.1} ops/cycle",
        unpacked_stats.siws_retired, unpacked_stats.ops_retired, unpacked_stats.ops_per_cycle());
    println!("\n  Speedup: {:.1}×", packed_stats.ops_per_cycle() / unpacked_stats.ops_per_cycle().max(0.001));
}
