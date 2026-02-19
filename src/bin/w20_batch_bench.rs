//! W20: Stream Batching Benchmark
//! Compares run() vs run_batched() on uniform vs mixed workloads.
//!
//! Key prediction: uniform streams (same mode throughout) should batch
//! into a single run and execute faster than mode-switching streams.
//! Mixed streams should show similar performance between the two paths.

use vtpu_runtime::{
    exec::{run, run_batched},
    memory::Memory,
    sentron::Sentron,
    siw::SIW,
    pipes::{DenseOp, SparseOp, CoordOp},
    phext_coord::PhextCoord,
};
use std::time::Instant;

const N: usize = 20_000;
const WARMUP: usize = 3;
const TRIALS: usize = 7;

fn dadd_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| {
        let rd = (i % 14 + 1) as u8;
        let rs1 = (i % 15) as u8;
        let rs2 = ((i + 1) % 15) as u8;
        SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect()
}

fn dfma_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| {
        let rd = (i % 12 + 1) as u8;
        let rs1 = rd;
        let rs2 = (rd + 1) % 15 + 1;
        let rs3 = (rd + 2) % 15 + 1;
        SIW::new(DenseOp::DFMA { rd, rs1, rs2, rs3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect()
}

/// Alternates D/S/C every SIW — worst case for batching (run = 1)
fn alternating_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| match i % 3 {
        0 => SIW::new(
            DenseOp::DADD { rd: 1, rs1: 0, rs2: 2 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        1 => SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP, PhextCoord::zero()),
        _ => SIW::new(
            DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CBAR { barrier_id: 0, count: 1 }, PhextCoord::zero()),
    }).collect()
}

/// Long uniform runs: 1000 DADD then 1000 SGATHER then 1000 CBAR repeating
fn block_stream(n: usize) -> Vec<SIW> {
    let block = 1000;
    (0..n).map(|i| match (i / block) % 3 {
        0 => SIW::new(
            DenseOp::DADD { rd: ((i % 14) as u8) + 1, rs1: (i % 15) as u8, rs2: ((i+1)%15) as u8 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        1 => SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP, PhextCoord::zero()),
        _ => SIW::new(
            DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CBAR { barrier_id: 0, count: 1 }, PhextCoord::zero()),
    }).collect()
}

fn bench_both(name: &str, stream: &[SIW]) {
    let mut best_run = u128::MAX;
    let mut best_batched = u128::MAX;
    let mut ops_run = 0u64;
    let mut ops_batched = 0u64;

    for trial in 0..WARMUP + TRIALS {
        // run()
        let mut s1 = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let mut m1 = Memory::new();
        s1.spawn(stream.to_vec());
        let t0 = Instant::now();
        let st1 = run(&mut s1, &mut m1);
        let ns1 = t0.elapsed().as_nanos();

        // run_batched()
        let mut s2 = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let mut m2 = Memory::new();
        s2.spawn(stream.to_vec());
        let t0 = Instant::now();
        let st2 = run_batched(&mut s2, &mut m2);
        let ns2 = t0.elapsed().as_nanos();

        if trial >= WARMUP {
            if ns1 < best_run    { best_run = ns1;    ops_run = st1.ops_retired; }
            if ns2 < best_batched { best_batched = ns2; ops_batched = st2.ops_retired; }
        }
    }

    let ns_per_run     = best_run     as f64 / stream.len() as f64;
    let ns_per_batched = best_batched as f64 / stream.len() as f64;
    let speedup = best_run as f64 / best_batched as f64;
    let parity = if ops_run == ops_batched {
        "✅".to_string()
    } else {
        format!("❌ ({ops_run} vs {ops_batched})")
    };

    println!("{:<24}  run: {:5.2}ns/SIW  batched: {:5.2}ns/SIW  speedup: {:.2}×  parity:{}",
        name, ns_per_run, ns_per_batched, speedup, parity);
}

fn main() {
    println!("R23 W20: Stream Batching Benchmark");
    println!("run() vs run_batched() — N={N} SIWs, best of {TRIALS} trials\n");
    println!("{:<24}  {:<20}  {:<22}  {:<12}", "Workload", "run()", "run_batched()", "speedup");
    println!("{}", "-".repeat(80));

    bench_both("DADD uniform",        &dadd_stream(N));
    bench_both("DFMA uniform",        &dfma_stream(N));
    bench_both("Alternating D/S/C",   &alternating_stream(N));
    bench_both("Block (1k runs)",     &block_stream(N));

    println!("\nBatch advantage appears on uniform streams (1 run = whole program).");
    println!("Alternating streams: run len=1, overhead cancels any gain.");
    println!("Block streams: run len=1000, LLVM vectorizes within each block.");
}
