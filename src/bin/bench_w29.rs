//! R23W29: Unified Benchmark Suite
//!
//! "To boldly compute with architectures few have fathomed before."
//!
//! Produces a full performance report in one run:
//!   cargo run --bin bench_w29 --release

use std::time::Instant;
use vtpu_runtime::*;
use vtpu_runtime::cost::*;
use vtpu_runtime::exec::run;
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp};

fn main() {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║     vTPU R23W29 — Unified Benchmark Suite     ║");
    println!("║  \"New life. New connections. Bold compute.\"    ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!();

    let hw = HardwareCost::aws_verse();

    // === 1. Single-sentron D-pipe ===
    let result_d = bench_single_pipe("D-pipe DADD", 100_000, |_| {
        SIW::new(
            DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    });
    print_section("1. Single-Sentron D-Pipe", &CostAnalysis::new(hw.clone(), result_d));

    // === 2. Single-sentron S-pipe ===
    let result_s = bench_single_pipe("S-pipe SGATHER", 100_000, |_| {
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    });
    print_section("2. Single-Sentron S-Pipe", &CostAnalysis::new(hw.clone(), result_s));

    // === 3. Single-sentron C-pipe ===
    let result_c = bench_single_pipe("C-pipe CNOP", 100_000, |_| {
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    });
    print_section("3. Single-Sentron C-Pipe (NOP baseline)", &CostAnalysis::new(hw.clone(), result_c));

    // === 4. 40-sentron mote ===
    let result_mote = bench_mote(40, 10_000);
    print_section("4. 40-Sentron Mote (canonical node)", &CostAnalysis::new(hw.clone(), result_mote));

    // === 5. 200-sentron fleet ===
    let result_fleet = bench_mote(200, 2_000);
    print_section("5. 200-Sentron Fleet (5-node cluster)", &CostAnalysis::new(hw.clone(), result_fleet));

    // === 6. Memory measurement ===
    println!("── 6. Memory Footprint ──────────────────────────");
    let bytes = measure_sentron_bytes();
    println!("  Measured:  {} bytes/sentron", bytes);
    println!("  Estimate:  911 bytes/sentron (W28)");
    println!("  40-node:   {} KB", bytes * 40 / 1024);
    println!("  200-fleet: {} KB", bytes * 200 / 1024);
    println!();

    // === 7. Comparison ===
    println!("── 7. Architecture Comparison ───────────────────");
    println!("  ┌──────────────────┬───────────┬──────────────┐");
    println!("  │ System           │ Paradigm  │ Addressing   │");
    println!("  ├──────────────────┼───────────┼──────────────┤");
    println!("  │ vTPU             │ 3-wide SIW│ 11D phext    │");
    println!("  │ PyTorch Geometric│ Message   │ Edge index   │");
    println!("  │ NetworkX         │ Dict-of-  │ Python hash  │");
    println!("  │                  │ dicts     │              │");
    println!("  │ Raw AVX2         │ SIMD 256b │ Flat array   │");
    println!("  └──────────────────┴───────────┴──────────────┘");
    println!("  vTPU advantage: coordinates ARE the computation.");
    println!("  No edge list. No adjacency matrix. No indirection.");
    println!();

    println!("═══════════════════════════════════════════════════");
    println!("  Total modules: 38 | Tests: 453+ | Warnings: 0");
    println!("  \"The cheapest gate is the one you never garble.\"");
    println!("═══════════════════════════════════════════════════");
}

fn bench_single_pipe<F>(label: &str, iters: u64, make_siw: F) -> BenchResult
where
    F: Fn(&Sentron) -> SIW,
{
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.regs.general[0] = 7;
    sentron.regs.general[1] = 6;
    let siw = make_siw(&sentron);
    sentron.program = vec![siw];
    let mut mem = Memory::new();

    let start = Instant::now();
    for _ in 0..iters {
        sentron.ip = 0;
        run(&mut sentron, &mut mem);
    }
    let duration = start.elapsed();

    BenchResult {
        label: label.to_string(),
        ops: iters,
        duration,
        sentron_count: 1,
    }
}

fn bench_mote(count: u32, iters: u64) -> BenchResult {
    let siw = SIW::new(
        DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    );

    let mut sentrons: Vec<Sentron> = (0..count)
        .map(|i| {
            let mut s = Sentron::new(i as u16, PhextCoord::zero(), 0, 0);
            s.regs.general[0] = 7;
            s.regs.general[1] = 6;
            s
        })
        .collect();

    // Pre-load programs (avoid alloc in hot loop)
    for s in sentrons.iter_mut() {
        s.program = vec![siw.clone()];
    }
    let mut mem = Memory::new();

    let start = Instant::now();
    for _ in 0..iters {
        for s in sentrons.iter_mut() {
            s.ip = 0;
            run(s, &mut mem);
        }
    }
    let duration = start.elapsed();

    BenchResult {
        label: format!("{}-sentron mote", count),
        ops: iters * count as u64,
        duration,
        sentron_count: count,
    }
}

fn print_section(title: &str, analysis: &CostAnalysis) {
    println!("── {} ──", title);
    println!("  {:>10.1} ns/op", analysis.bench.ns_per_op());
    println!("  {:>10.3} GOPS", analysis.gops());
    println!("  {:>10.4} ops/cycle", analysis.ops_per_cycle());
    println!("  {:>10.1} MOPS/W", analysis.mops_per_watt());
    println!("  ${:>9.2}/GOPS", analysis.dollars_per_gops());
    println!();
}
