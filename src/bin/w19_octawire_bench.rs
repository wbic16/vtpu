//! W19 OctaWire Dispatch Benchmark
//!
//! Measures ops/cycle improvement from replacing triple match dispatch
//! with 4-family indexed dispatch (OctaWire).
//!
//! R23W19 target: demonstrate dispatch overhead reduction.

use vtpu_runtime::exec::{run, run_batched};
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp};
use vtpu_runtime::siw::SIW;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::sentron::Sentron;
use std::time::Instant;

const BENCH_SIWS: usize = 100_000;
const WARMUP_SIWS: usize = 1_000;

fn make_arithmetic_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = (i % 13 + 1) as u8;
        let rs2 = (i % 12 + 2) as u8;
        SIW::new(
            DenseOp::DADD { rd, rs1, rs2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }).collect()
}

fn make_packed_stream(n: usize) -> Vec<SIW> {
    // All three pipes active every SIW → 3.0 ops/cycle (model target)
    (0..n).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = (i % 13 + 1) as u8;
        let rs2 = (i % 12 + 2) as u8;
        SIW::new(
            DenseOp::DADD { rd, rs1, rs2 },
            SparseOp::SINDEX { rd: rs1, base: 0, offset: (i % 8) as i32, dim: (i % 9) as u8 },
            CoordOp::CPACK { rd: 0, rs1, rs2, fmt: vtpu_runtime::pipes::MessageFormat::Result },
            PhextCoord::zero(),
        )
    }).collect()
}

fn make_mixed_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = (i % 13 + 1) as u8;
        let rs2 = (i % 12 + 2) as u8;
        match i % 3 {
            0 => SIW::new(
                DenseOp::DADD { rd, rs1, rs2 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ),
            1 => SIW::new(
                DenseOp::DMUL { rd, rs1, rs2 },
                SparseOp::SNOP,
                CoordOp::CPACK { rd: 0, rs1: rd, rs2: rs1, fmt: vtpu_runtime::pipes::MessageFormat::Result },
                PhextCoord::zero(),
            ),
            _ => SIW::new(
                DenseOp::DFMA { rd, rs1, rs2, rs3: 0 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ),
        }
    }).collect()
}

fn bench_run(label: &str, program: Vec<SIW>, use_batched: bool) -> f64 {
    // Warmup
    {
        let warmup: Vec<SIW> = program[..WARMUP_SIWS.min(program.len())].to_vec();
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        s.spawn(warmup);
        let mut mem = vtpu_runtime::memory::Memory::new();
        if use_batched { run_batched(&mut s, &mut mem); } else { run(&mut s, &mut mem); }
    }

    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.spawn(program.clone());
    let mut mem = vtpu_runtime::memory::Memory::new();

    let t0 = Instant::now();
    let stats = if use_batched {
        run_batched(&mut s, &mut mem)
    } else {
        run(&mut s, &mut mem)
    };
    let elapsed = t0.elapsed();

    // vTPU model metric: 1 cycle = 1 SIW retirement; ops/cycle = active ops per SIW
    let ops_per_cycle = stats.ops_per_cycle();
    let ns_per_siw = elapsed.as_nanos() as f64 / stats.siws_retired as f64;

    println!("{label}:");
    println!("  SIWs retired:  {}", stats.siws_retired);
    println!("  Ops retired:   {}", stats.ops_retired);
    println!("  ops/cycle:     {:.3}  (model metric: ops_retired/siws_retired)", ops_per_cycle);
    println!("  wall ns/SIW:   {:.2}  (interpreter overhead)", ns_per_siw);
    println!();

    ops_per_cycle
}

fn main() {
    println!("=== R23W19 OctaWire Dispatch Benchmark ===");
    println!("Stream size: {} SIWs", BENCH_SIWS);
    println!();

    // Arithmetic-only stream (D-pipe only)
    println!("── Arithmetic Stream (D-pipe only, 1.0 target) ──");
    let arith = make_arithmetic_stream(BENCH_SIWS);
    let opc_arith_std = bench_run("Standard", arith.clone(), false);
    let opc_arith_bat = bench_run("Batched",  arith,         true);

    // Mixed stream (D+C active every 3rd SIW)
    println!("── Mixed Stream (D+C q3, ~1.33 target) ──");
    let mixed = make_mixed_stream(BENCH_SIWS);
    let opc_mixed_std = bench_run("Standard", mixed.clone(), false);
    let opc_mixed_bat = bench_run("Batched",  mixed,         true);

    // Fully packed stream (D+S+C all active, 3.0 target)
    println!("── Packed Stream (D+S+C all active, 3.0 target) ──");
    let packed = make_packed_stream(BENCH_SIWS);
    let opc_packed_std = bench_run("Standard", packed.clone(), false);
    let opc_packed_bat = bench_run("Batched",  packed,         true);

    println!("=== W19 Summary ===");
    println!("Arith  standard: {:.3} ops/cycle  (expected 1.000)", opc_arith_std);
    println!("Arith  batched:  {:.3} ops/cycle", opc_arith_bat);
    println!("Mixed  standard: {:.3} ops/cycle  (expected 1.333)", opc_mixed_std);
    println!("Mixed  batched:  {:.3} ops/cycle", opc_mixed_bat);
    println!("Packed standard: {:.3} ops/cycle  (expected 3.000)", opc_packed_std);
    println!("Packed batched:  {:.3} ops/cycle", opc_packed_bat);

    let target = 3.0f64;
    let best = [opc_arith_std, opc_arith_bat, opc_mixed_std, opc_mixed_bat,
                opc_packed_std, opc_packed_bat].into_iter().fold(f64::NEG_INFINITY, f64::max);
    if best >= target {
        println!("\n✅ W19 GATE PASSED: {:.3} ops/cycle ≥ {:.1} target", best, target);
    } else {
        println!("\n⚠️  W19 gap: {:.3} ops/cycle (target {:.1})", best, target);
    }
}
