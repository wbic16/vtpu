//! R23W21 Benchmark: All-pipe const dispatch tables + wall ns/SIW measurement
//!
//! W20 landed D_TABLE + C_TABLE.
//! W21 adds S_TABLE (address/route via _wrap fns, load/store direct).
//! All three pipes now use const function pointer arrays — no match trees.
//!
//! Targets:
//!   Packed D+S+C  → 3.000 ops/cycle  ✓
//!   S-only        → 1.000 ops/cycle  ✓
//!   Wall ns/SIW   → baseline recorded

use vtpu_runtime::exec::{run, run_batched};
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp, MessageFormat};
use vtpu_runtime::siw::SIW;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::memory::Memory;
use std::time::Instant;

const BENCH_N: usize = 100_000;
const WARMUP_N: usize = 2_000;

fn bench(label: &str, siws: Vec<SIW>, batched: bool) -> (f64, f64) {
    // warmup
    {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        s.spawn(siws[..WARMUP_N.min(siws.len())].to_vec());
        let mut m = Memory::new();
        if batched { run_batched(&mut s, &mut m); } else { run(&mut s, &mut m); }
    }

    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.spawn(siws.clone());
    let mut m = Memory::new();

    let t0 = Instant::now();
    let stats = if batched { run_batched(&mut s, &mut m) } else { run(&mut s, &mut m) };
    let elapsed = t0.elapsed();

    let opc    = stats.ops_per_cycle();
    let ns_siw = elapsed.as_nanos() as f64 / stats.siws_retired.max(1) as f64;
    println!("  {:<30}  {:>7.3} ops/cyc   {:>8.2} ns/SIW   [{} siws]",
        label, opc, ns_siw, stats.siws_retired);
    (opc, ns_siw)
}

fn d_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| SIW::new(
        DenseOp::DADD { rd: (i%14) as u8, rs1: ((i+1)%15) as u8, rs2: ((i+2)%15) as u8 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()
    )).collect()
}

fn s_stream(n: usize) -> Vec<SIW> {
    // S-pipe only: SINDEX (fam 2 — address, goes through exec_s_address_wrap)
    (0..n).map(|i| SIW::new(
        DenseOp::DNOP,
        SparseOp::SINDEX { rd: (i%8) as u8, base: 0, offset: ((i%31) as i32) - 15, dim: (i%11) as u8 },
        CoordOp::CNOP, PhextCoord::zero()
    )).collect()
}

fn s_load_stream(n: usize) -> Vec<SIW> {
    // S-pipe fam 0 (load) — goes through exec_s_load directly
    (0..n).map(|i| SIW::new(
        DenseOp::DNOP,
        SparseOp::SGATHER { rd: (i%14) as u8, coord_idx: (i%8) as u8, width: 8 },
        CoordOp::CNOP, PhextCoord::zero()
    )).collect()
}

fn c_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| SIW::new(
        DenseOp::DNOP, SparseOp::SNOP,
        CoordOp::CBAR { barrier_id: (i%8) as u8, count: 1 }, PhextCoord::zero()
    )).collect()
}

fn packed_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| SIW::new(
        DenseOp::DADD { rd: (i%14) as u8, rs1: ((i+1)%15) as u8, rs2: ((i+2)%15) as u8 },
        SparseOp::SINDEX { rd: (i%8) as u8, base: 0, offset: 1, dim: (i%11) as u8 },
        CoordOp::CPACK { rd: 0, rs1: (i%14) as u8, rs2: ((i+1)%15) as u8, fmt: MessageFormat::Result },
        PhextCoord::zero()
    )).collect()
}

fn main() {
    println!("=== R23W21 All-Pipe Const Dispatch Benchmark ===");
    println!("  D_TABLE[4] + S_TABLE[4] + C_TABLE[4] — no match trees");
    println!("  {} SIWs  {} warmup", BENCH_N, WARMUP_N);
    println!();

    println!("[ run() — single SIW path ]");
    bench("D-only (DADD, fam 0)",          d_stream(BENCH_N),       false);
    bench("S-only SINDEX (fam 2 addr)",    s_stream(BENCH_N),       false);
    bench("S-only SGATHER (fam 0 load)",   s_load_stream(BENCH_N),  false);
    bench("C-only CBAR (fam 2)",           c_stream(BENCH_N),       false);
    let (opc_packed, _) =
    bench("Packed D+S+C",                  packed_stream(BENCH_N),  false);
    println!();

    println!("[ run_batched() — same-mode grouping path ]");
    bench("D-only batched",                d_stream(BENCH_N),       true);
    bench("S-only batched (SINDEX)",       s_stream(BENCH_N),       true);
    bench("Packed batched",                packed_stream(BENCH_N),  true);
    println!();

    println!("Gate: Packed ops/cycle = {:.3}  (≥ 3.000 required)", opc_packed);
    if opc_packed >= 3.0 - 1e-6 {
        println!("✅ R23W21 GATE PASSED");
    } else {
        println!("❌ gate not yet met");
        std::process::exit(1);
    }
}
