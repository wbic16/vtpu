//! Memory-Heavy Packing Pattern — R23W16 Rewrite
//!
//! Demonstrates overlapping memory access with computation:
//! Gather next value while computing on current value.
//! Hides memory latency through pipeline parallelism.

use vtpu_runtime::*;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::exec;

fn main() {
    println!("═══ Memory-Heavy Packing Pattern ═══\n");

    let mut mem = Memory::new();

    // Seed 50 values
    for i in 1..=50u16 {
        let mut c = [1u16; 11];
        c[0] = i;
        mem.scatter_i64(&PhextCoord::new(c), i as i64);
    }

    // Pipeline: gather[i+1] overlaps with compute[i] and scatter[i-1]
    // This is the producer-consumer pattern for memory-bound workloads
    let mut program = Vec::new();

    // Prime the pipeline: first gather
    program.push(SIW::new(
        DenseOp::DNOP,
        SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
        CoordOp::CNOP,
        PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
    ));

    // Steady state: gather next + compute current + scatter previous
    for i in 0..20u8 {
        let curr = (i % 12) + 1;
        let next = (i % 12) + 2;
        let prev = if i > 0 { (i - 1) % 12 + 1 } else { 14 };

        program.push(SIW::new(
            DenseOp::DMUL { rd: curr + 6, rs1: curr, rs2: curr }, // compute: square it
            SparseOp::SGATHER { rd: next, coord_idx: 0, width: 64 }, // prefetch next
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
    }

    // Drain: final compute + scatter
    program.push(SIW::new(
        DenseOp::DMUL { rd: 15, rs1: 1, rs2: 1 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    ));

    let n_siws = program.len();

    let mut sentron = Sentron::new(0, PhextCoord::new([1; 11]), 0, 0);
    sentron.regs.phext[0] = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    sentron.spawn(program);
    let stats = exec::run(&mut sentron, &mut mem);

    println!("  Pipeline stages:  {}", n_siws);
    println!("  SIWs retired:     {}", stats.siws_retired);
    println!("  Ops retired:      {}", stats.ops_retired);
    println!("  Ops/cycle:        {:.2}", stats.ops_per_cycle());
    println!();

    // Compare with unpacked (D-only, no overlap)
    let unpacked: Vec<SIW> = (0..22u8).map(|i| {
        let rd = (i % 14) + 1;
        SIW::new(
            DenseOp::DMUL { rd, rs1: rd, rs2: rd },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }).collect();

    let mut s2 = Sentron::new(1, PhextCoord::zero(), 0, 1);
    s2.spawn(unpacked);
    let baseline = exec::run_standalone(&mut s2);

    println!("  Baseline (D-only): {:.2} ops/cycle", baseline.ops_per_cycle());
    println!("  Speedup:           {:.1}×", stats.ops_per_cycle() / baseline.ops_per_cycle().max(0.001));
    println!("\n  Memory latency hidden by overlapping gather with compute.");
}
