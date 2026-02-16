//! R23W16 - SMT Baseline Measurement
//!
//! Establishes single-thread reference performance before SMT optimization.
//! Measures D-heavy, S-heavy, and mixed workloads.
//!
//! Usage: cargo run --release --bin w16_smt_baseline

use vtpu_runtime::*;
use std::time::Instant;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║         R23W16: SMT Baseline (Single-Thread Reference)       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Measuring single-thread performance before SMT optimization.");
    println!();

    let d_heavy = bench_d_heavy();
    let s_heavy = bench_s_heavy();
    let mixed = bench_mixed();

    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    Summary (Baseline)                         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("D-Pipe Heavy:  {:.2}M ops/sec", d_heavy / 1e6);
    println!("S-Pipe Heavy:  {:.2}M ops/sec", s_heavy / 1e6);
    println!("Mixed (50/50): {:.2}M ops/sec", mixed / 1e6);
    println!();
    println!("Average:       {:.2}M ops/sec", (d_heavy + s_heavy + mixed) / 3.0 / 1e6);
    println!();
    println!("Next: W16 SMT dual-thread (target 1.8× speedup)");
}

fn bench_d_heavy() -> f64 {
    println!("─── D-Pipe Heavy (80% Compute, 20% Memory) ───");
    
    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    sentron.regs.general[0] = 7;
    sentron.regs.general[1] = 6;

    let mut program = Vec::new();

    // 80% D-Pipe ops
    for _ in 0..1000 {
        program.push(SIW::new(
            DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        program.push(SIW::new(
            DenseOp::DADD { rd: 3, rs1: 2, rs2: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        program.push(SIW::new(
            DenseOp::DSUB { rd: 4, rs1: 3, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        program.push(SIW::new(
            DenseOp::DFMA { rd: 5, rs1: 2, rs2: 3, rs3: 4 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // 20% S-Pipe (load data occasionally)
        if program.len() % 5 == 0 {
            let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            sentron.regs.phext[0] = coord;
            
            program.push(SIW::new(
                DenseOp::DNOP,
                SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
        }
    }

    sentron.spawn(program);

    let start = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = start.elapsed();

    let ops_per_sec = stats.ops_retired as f64 / elapsed.as_secs_f64();

    println!("  SIWs:        {}", stats.siws_retired);
    println!("  Ops:         {}", stats.ops_retired);
    println!("  Time:        {:.2} µs", elapsed.as_micros());
    println!("  Throughput:  {:.2}M ops/sec", ops_per_sec / 1e6);
    println!("  D-Pipe util: {:.1}%", stats.d_utilization() * 100.0);
    println!("  S-Pipe util: {:.1}%", stats.s_utilization() * 100.0);
    println!();

    ops_per_sec
}

fn bench_s_heavy() -> f64 {
    println!("─── S-Pipe Heavy (80% Memory, 20% Compute) ───");
    
    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Pre-populate memory
    for i in 1..=1000 {
        let coord = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&coord, i as i64);
    }

    let mut program = Vec::new();

    // 80% S-Pipe ops
    for i in 1..=1000 {
        let coord = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        sentron.regs.phext[0] = coord;

        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 0, rs: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // 20% D-Pipe (compute addresses)
        if i % 5 == 0 {
            program.push(SIW::new(
                DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
        }
    }

    sentron.spawn(program);

    let start = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = start.elapsed();

    let ops_per_sec = stats.ops_retired as f64 / elapsed.as_secs_f64();

    println!("  SIWs:        {}", stats.siws_retired);
    println!("  Ops:         {}", stats.ops_retired);
    println!("  Time:        {:.2} µs", elapsed.as_micros());
    println!("  Throughput:  {:.2}M ops/sec", ops_per_sec / 1e6);
    println!("  D-Pipe util: {:.1}%", stats.d_utilization() * 100.0);
    println!("  S-Pipe util: {:.1}%", stats.s_utilization() * 100.0);
    println!();

    ops_per_sec
}

fn bench_mixed() -> f64 {
    println!("─── Mixed (50% Compute, 50% Memory) ───");
    
    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Pre-populate memory
    for i in 1..=100 {
        let coord = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&coord, i as i64 * 10);
    }

    let mut program = Vec::new();

    // 50/50 D-Pipe and S-Pipe
    for i in 1..=100 {
        let src = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dst = PhextCoord::new([i + 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        sentron.regs.phext[0] = src;
        sentron.regs.phext[1] = dst;

        // Load
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Compute
        program.push(SIW::new(
            DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        program.push(SIW::new(
            DenseOp::DMUL { rd: 2, rs1: 1, rs2: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Store
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 1, rs: 2, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
    }

    sentron.spawn(program);

    let start = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = start.elapsed();

    let ops_per_sec = stats.ops_retired as f64 / elapsed.as_secs_f64();

    println!("  SIWs:        {}", stats.siws_retired);
    println!("  Ops:         {}", stats.ops_retired);
    println!("  Time:        {:.2} µs", elapsed.as_micros());
    println!("  Throughput:  {:.2}M ops/sec", ops_per_sec / 1e6);
    println!("  D-Pipe util: {:.1}%", stats.d_utilization() * 100.0);
    println!("  S-Pipe util: {:.1}%", stats.s_utilization() * 100.0);
    println!();

    ops_per_sec
}
