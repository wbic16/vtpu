//! W21: 8-lane SIMD row execution benchmark.
//! Compares run_row_8() (all 8 columns of a WuXing row) vs 8× scalar run().

use vtpu_runtime::{
    exec::run,
    memory::Memory,
    phext_coord::PhextCoord,
    pipes::{DenseOp, SparseOp, CoordOp},
    sentron::Sentron,
    simd::run_row_8,
    siw::SIW,
};
use std::time::Instant;

const N: usize = 20_000;
const WARMUP: usize = 3;
const TRIALS: usize = 7;

fn dadd_program(n: usize) -> Vec<SIW> {
    (0..n).map(|i| {
        let rd = (i % 14 + 1) as u8;
        let rs1 = (i % 15) as u8;
        let rs2 = ((i + 1) % 15) as u8;
        SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect()
}

fn dfma_program(n: usize) -> Vec<SIW> {
    (0..n).map(|i| {
        let rd = (i % 12 + 1) as u8;
        SIW::new(DenseOp::DFMA { rd, rs1: rd, rs2: (rd+1)%15+1, rs3: (rd+2)%15+1 },
                 SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect()
}

fn make_row(seed: i64) -> [Sentron; 8] {
    std::array::from_fn(|i| {
        let mut s = Sentron::new(i as u16, PhextCoord::zero(), 0, i as u8);
        for r in 0..16 { s.regs.general[r] = seed * (i as i64 + 1) * (r as i64 + 1); }
        s
    })
}

fn bench_simd(name: &str, program: &[SIW]) {
    let mut best_row = u128::MAX;
    let mut best_scalar = u128::MAX;

    for trial in 0..WARMUP + TRIALS {
        // 8-lane row
        let mut row = make_row(trial as i64 + 1);
        let t0 = Instant::now();
        run_row_8(&mut row, program);
        let row_ns = t0.elapsed().as_nanos();

        // 8× scalar (sequential)
        let mut scalar_total: u128 = 0;
        for lane in 0..8u16 {
            let mut s = Sentron::new(lane, PhextCoord::zero(), 0, lane as u8);
            for r in 0..16 { s.regs.general[r] = (trial as i64 + 1) * (lane as i64 + 1) * (r as i64 + 1); }
            let mut mem = Memory::new();
            s.spawn(program.to_vec());
            let t1 = Instant::now();
            run(&mut s, &mut mem);
            scalar_total += t1.elapsed().as_nanos();
        }

        if trial >= WARMUP {
            if row_ns    < best_row    { best_row    = row_ns; }
            if scalar_total < best_scalar { best_scalar = scalar_total; }
        }
    }

    // Per-lane-SIW throughput
    let total_lane_siws = (N * 8) as f64;
    let row_ns_per    = best_row    as f64 / total_lane_siws;
    let scalar_ns_per = best_scalar as f64 / total_lane_siws;
    let speedup = best_scalar as f64 / best_row as f64;

    // ops/cycle at 5 GHz
    let ops_per_cycle_row    = 1.0 / (row_ns_per * 5.0);
    let ops_per_cycle_scalar = 1.0 / (scalar_ns_per * 5.0);

    println!("{:<18}  row8: {:5.2}ns/lane-SIW ({:.3}ops/cyc)  scalar: {:5.2}ns ({:.3}ops/cyc)  speedup: {:.2}×",
        name, row_ns_per, ops_per_cycle_row, scalar_ns_per, ops_per_cycle_scalar, speedup);
}

fn main() {
    // Also measure phext-load gate improvement
    println!("R23 W21: 8-Lane SIMD Row Execution");
    println!("run_row_8() (8 sentrons in parallel) vs 8× scalar run()");
    println!("N={N} SIWs per lane, best of {TRIALS} trials (after {WARMUP} warmup)\n");

    println!("{:<18}  {:<44}  {:<28}  {}", "Workload", "row8 (8 lanes parallel)", "scalar (8 sequential)", "speedup");
    println!("{}", "-".repeat(110));

    bench_simd("DADD uniform",  &dadd_program(N));
    bench_simd("DFMA uniform",  &dfma_program(N));

    println!("\nTarget: ≥4× speedup on uniform D-pipe streams (LLVM auto-vectorization).");
    println!("Zen 4 AVX-256: 4 i64 ops per cycle per lane = 8 lanes × 4 ops = 32 ops/cycle theoretical.");
    println!("\nAlso: phext-load gate — only load phext[0] when S/C pipes active.");

    // Measure single-sentron improvement from phext gate
    println!("\n--- Single-sentron phext-gate effect (DADD, no S/C pipes) ---");
    let prog = dadd_program(N);
    let mut best = u128::MAX;
    for trial in 0..WARMUP + TRIALS {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let mut mem = Memory::new();
        s.spawn(prog.clone());
        let t0 = Instant::now();
        run(&mut s, &mut mem);
        let ns = t0.elapsed().as_nanos();
        if trial >= WARMUP && ns < best { best = ns; }
    }
    println!("  Single sentron DADD: {:.2}ns/SIW  ({:.3}ops/cyc @ 5GHz)",
        best as f64 / N as f64, N as f64 / (best as f64 * 5.0));
}
