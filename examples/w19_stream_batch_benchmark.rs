//! W19 Stream Batching Benchmark — R23 W19 LLVM Integration
//!
//! Measures throughput of exec_stream_batched vs exec_siw_octawire vs legacy.
//!
//! Key hypothesis: same-mode SIW runs let LLVM hoist the branch check outside
//! the inner loop → better instruction scheduling + potential auto-vectorization.
//!
//! target-cpu=native enables:
//!   - AVX2+BMI2 on AWS (Zen 1 EPYC 7571)
//!   - AVX-512 on ranch (Zen 4 R9 8945HS) — 2× more SIMD width

use vtpu_runtime::{
    exec::{self, exec_siw_octawire, exec_stream_batched},
    memory::Memory,
    phext_coord::PhextCoord,
    pipes::{CoordOp, DenseOp, MessageFormat, SparseOp},
    sentron::{Sentron, SentronState},
    siw::SIW,
};
use std::time::Instant;

const STREAM_LEN: usize = 100_000;
const WARMUP: usize = 10_000;
const RUNS: usize = 5;

// ── Stream Generators ────────────────────────────────────────────────────────

/// All-same mode: D-only arithmetic (pure mode run, max batching benefit)
fn make_uniform_d_stream() -> Vec<SIW> {
    (0..STREAM_LEN).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = ((i + 1) % 14) as u8;
        let rs2 = ((i + 2) % 14) as u8;
        SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect()
}

/// All-same mode: full 3-wide (D+S+C all active, same family throughout)
fn make_uniform_full_stream() -> Vec<SIW> {
    (0..STREAM_LEN).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = ((i + 1) % 14) as u8;
        let rs2 = ((i + 2) % 14) as u8;
        SIW::new(
            DenseOp::DADD { rd, rs1, rs2 },
            SparseOp::SINDEX { rd: rd % 8, base: rs1 % 8, offset: 1, dim: (i % 9) as u8 },
            CoordOp::CPACK { rd: rd % 4, rs1, rs2, fmt: MessageFormat::Result },
            PhextCoord::zero(),
        )
    }).collect()
}

/// Alternating modes: maximally bad for batching (2-SIW runs)
fn make_alternating_stream() -> Vec<SIW> {
    (0..STREAM_LEN).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = ((i + 1) % 14) as u8;
        let rs2 = ((i + 2) % 14) as u8;
        if i % 2 == 0 {
            SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
        } else {
            SIW::new(DenseOp::DNOP,
                SparseOp::SINDEX { rd: rd % 8, base: rs1 % 8, offset: 1, dim: 0 },
                CoordOp::CNOP, PhextCoord::zero())
        }
    }).collect()
}

/// Real-world pattern: 80% D-only, 15% D+S, 5% D+S+C
fn make_realistic_stream() -> Vec<SIW> {
    (0..STREAM_LEN).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = ((i + 1) % 14) as u8;
        let rs2 = ((i + 2) % 14) as u8;
        match i % 20 {
            0..=15 => SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            16..=18 => SIW::new(
                DenseOp::DADD { rd, rs1, rs2 },
                SparseOp::SINDEX { rd: rd%8, base: rs1%8, offset: 1, dim: 0 },
                CoordOp::CNOP, PhextCoord::zero()),
            _ => SIW::new(
                DenseOp::DADD { rd, rs1, rs2 },
                SparseOp::SINDEX { rd: rd%8, base: rs1%8, offset: 1, dim: 0 },
                CoordOp::CPACK { rd: rd%4, rs1, rs2, fmt: MessageFormat::Result },
                PhextCoord::zero()),
        }
    }).collect()
}

// ── Benchmark Harness ────────────────────────────────────────────────────────

fn make_sentron() -> Sentron {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    for i in 0..16 { s.regs.general[i] = (i as i64 + 1) * 7; }
    s
}

fn bench_batched(label: &str, stream: &[SIW]) {
    let mut mem = Memory::new();
    // warmup
    for _ in 0..WARMUP {
        let mut s = make_sentron();
        exec_stream_batched(&mut s, &stream[..1], &mut mem);
    }

    let mut best_ns = f64::MAX;
    for _ in 0..RUNS {
        let mut s = make_sentron();
        let t = Instant::now();
        exec_stream_batched(&mut s, stream, &mut mem);
        let elapsed = t.elapsed().as_secs_f64() * 1e9;
        let ns_per = elapsed / stream.len() as f64;
        if ns_per < best_ns { best_ns = ns_per; }
    }
    println!("[Batched  ] {:<38} | {:>8.1} ns/SIW | {:>12.0} SIW/s",
        label, best_ns, 1e9 / best_ns);
}

fn bench_octawire(label: &str, stream: &[SIW]) {
    let mut mem = Memory::new();
    for _ in 0..WARMUP {
        let mut s = make_sentron();
        for siw in &stream[..1] { exec_siw_octawire(&mut s, siw, &mut mem); }
    }

    let mut best_ns = f64::MAX;
    for _ in 0..RUNS {
        let mut s = make_sentron();
        let t = Instant::now();
        for siw in stream { exec_siw_octawire(&mut s, siw, &mut mem); }
        let elapsed = t.elapsed().as_secs_f64() * 1e9;
        let ns_per = elapsed / stream.len() as f64;
        if ns_per < best_ns { best_ns = ns_per; }
    }
    println!("[OctaWire ] {:<38} | {:>8.1} ns/SIW | {:>12.0} SIW/s",
        label, best_ns, 1e9 / best_ns);
}

fn main() {
    println!("=== W19 Stream Batching Benchmark (R23) ===");
    println!("Stream length: {} SIWs | {} warmup | {} runs (best-of)", STREAM_LEN, WARMUP, RUNS);
    println!("CPU: target-cpu=native (AVX2+BMI2 on AWS Zen1; AVX-512 on ranch Zen4)");
    println!();

    let streams = [
        ("Uniform D-only (100% same-mode)", make_uniform_d_stream()),
        ("Uniform D+S+C (100% same-mode)", make_uniform_full_stream()),
        ("Alternating D/S (2-SIW runs)", make_alternating_stream()),
        ("Realistic 80/15/5 mix", make_realistic_stream()),
    ];

    for (label, stream) in &streams {
        // Count unique modes
        let mut modes = std::collections::HashSet::new();
        let mut run_lens = Vec::new();
        let mut run = 1usize;
        for i in 1..stream.len() {
            if stream[i].d_fam == stream[i-1].d_fam
                && stream[i].s_fam == stream[i-1].s_fam
                && stream[i].c_fam == stream[i-1].c_fam {
                run += 1;
            } else {
                run_lens.push(run);
                run = 1;
            }
            modes.insert((stream[i].d_fam, stream[i].s_fam, stream[i].c_fam));
        }
        run_lens.push(run);
        let avg_run = run_lens.iter().sum::<usize>() as f64 / run_lens.len() as f64;
        println!("--- {} ---", label);
        println!("    Unique modes: {} | Avg run length: {:.1} SIWs", modes.len(), avg_run);
        bench_batched(label, stream);
        bench_octawire(label, stream);
        println!();
    }

    println!("=== Run Distribution Analysis ===");
    println!("Batching benefit = proportional to avg run length × mode count savings");
    println!("Best case: uniform stream → 1 mode check per 100K SIWs");
    println!("Worst case: alternating → 1 mode check per 2 SIWs (no gain)");
}
