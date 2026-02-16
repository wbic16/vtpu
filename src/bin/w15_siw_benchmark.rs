//! R23W15 - SIW-Level Ops/Cycle Benchmark
//!
//! Measures actual pipe-level operations per cycle, not high-level API calls.
//! Target: ≥2.5 ops/cycle sustained
//!
//! Usage: cargo run --release --bin w15_siw_benchmark

use vtpu_runtime::*;
use std::time::Instant;

const TARGET_OPS_PER_CYCLE: f64 = 2.5;
const ESTIMATED_CPU_GHZ: f64 = 4.0; // Zen 4 typical boost clock

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║     R23W15: SIW-Level Ops/Cycle Benchmark (Gap Closure)      ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Target: ≥{:.1} ops/cycle sustained", TARGET_OPS_PER_CYCLE);
    println!("CPU: Zen 4 @ ~{:.1} GHz", ESTIMATED_CPU_GHZ);
    println!();

    // Benchmark 1: Balanced D/S/C workload
    bench_balanced_workload();
    
    // Benchmark 2: D-Pipe heavy (compute)
    bench_d_heavy();
    
    // Benchmark 3: S-Pipe heavy (memory)
    bench_s_heavy();
    
    // Benchmark 4: Real inference workload
    bench_real_inference();
}

fn bench_balanced_workload() {
    println!("─── Benchmark 1: Balanced D/S/C Workload ───");
    println!("Workload: 50% D-Pipe, 40% S-Pipe, 10% C-Pipe");
    println!();

    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Pre-populate memory with test data
    for i in 1..=100 {
        let coord = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&coord, i as i64 * 10);
    }

    // Build balanced program:
    // - Load from memory (S-Pipe)
    // - Compute (D-Pipe)
    // - Store result (S-Pipe)
    // - Repeat
    let mut program = Vec::new();
    
    for i in 0..100 {
        let src1 = PhextCoord::new([i + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let src2 = PhextCoord::new([i + 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dst = PhextCoord::new([i + 200, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        sentron.regs.phext[0] = src1;
        sentron.regs.phext[1] = src2;
        sentron.regs.phext[2] = dst;

        // Load r0 from memory
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Load r1 from memory
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 1, coord_idx: 1, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Compute: r2 = r0 + r1
        program.push(SIW::new(
            DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Compute: r3 = r2 * r0
        program.push(SIW::new(
            DenseOp::DMUL { rd: 3, rs1: 2, rs2: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Store r3 to memory
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 2, rs: 3, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
    }

    sentron.spawn(program);

    // Execute and measure
    let start = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = start.elapsed();

    // Calculate ops/cycle (using executor's cycle count, not wall-clock estimate)
    // Note: Executor assumes 1 SIW = 1 cycle (ideal 3-wide retirement)
    let ops_per_cycle = stats.ops_per_cycle();

    // Report
    println!("  SIWs retired:    {}", stats.siws_retired);
    println!("  D-Pipe ops:      {} ({:.1}% util)", stats.d_ops, stats.d_utilization() * 100.0);
    println!("  S-Pipe ops:      {} ({:.1}% util)", stats.s_ops, stats.s_utilization() * 100.0);
    println!("  C-Pipe ops:      {} ({:.1}% util)", stats.c_ops, stats.c_utilization() * 100.0);
    println!("  Total ops:       {}", stats.ops_retired);
    println!("  Time:            {:.2} µs", elapsed.as_micros());
    println!("  Cycles (sim):    {}", stats.cycles);
    println!("  Ops/cycle:       {:.3}", ops_per_cycle);
    
    if ops_per_cycle >= TARGET_OPS_PER_CYCLE {
        println!("  Status:          ✅ PASS (≥{:.1})", TARGET_OPS_PER_CYCLE);
    } else {
        println!("  Status:          ⚠️  GAP ({:.3} short)", TARGET_OPS_PER_CYCLE - ops_per_cycle);
    }
    println!();
}

fn bench_d_heavy() {
    println!("─── Benchmark 2: D-Pipe Heavy (Compute-Bound) ───");
    println!("Workload: 80% D-Pipe, 20% S/C-Pipe");
    println!();

    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Initialize registers
    sentron.regs.general[0] = 7;
    sentron.regs.general[1] = 6;

    // Build compute-heavy program
    let mut program = Vec::new();
    
    for _ in 0..1000 {
        // Multiply
        program.push(SIW::new(
            DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Add
        program.push(SIW::new(
            DenseOp::DADD { rd: 3, rs1: 2, rs2: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Subtract
        program.push(SIW::new(
            DenseOp::DSUB { rd: 4, rs1: 3, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // FMA
        program.push(SIW::new(
            DenseOp::DFMA { rd: 5, rs1: 2, rs2: 3, rs3: 4 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
    }

    sentron.spawn(program);

    // Execute and measure
    let start = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = start.elapsed();

    
    
    let ops_per_cycle = stats.ops_per_cycle();

    println!("  SIWs retired:    {}", stats.siws_retired);
    println!("  D-Pipe ops:      {} ({:.1}% util)", stats.d_ops, stats.d_utilization() * 100.0);
    println!("  S-Pipe ops:      {} ({:.1}% util)", stats.s_ops, stats.s_utilization() * 100.0);
    println!("  C-Pipe ops:      {} ({:.1}% util)", stats.c_ops, stats.c_utilization() * 100.0);
    println!("  Total ops:       {}", stats.ops_retired);
    println!("  Time:            {:.2} µs", elapsed.as_micros());
    println!("  Cycles (sim):    {}", stats.cycles);
    println!("  Ops/cycle:       {:.3}", ops_per_cycle);
    
    if ops_per_cycle >= TARGET_OPS_PER_CYCLE {
        println!("  Status:          ✅ PASS (≥{:.1})", TARGET_OPS_PER_CYCLE);
    } else {
        println!("  Status:          ⚠️  GAP ({:.3} short)", TARGET_OPS_PER_CYCLE - ops_per_cycle);
    }
    println!();
}

fn bench_s_heavy() {
    println!("─── Benchmark 3: S-Pipe Heavy (Memory-Bound) ───");
    println!("Workload: 20% D-Pipe, 80% S-Pipe");
    println!();

    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Pre-populate memory
    for i in 1..=1000 {
        let coord = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&coord, i as i64);
    }

    // Build memory-heavy program
    let mut program = Vec::new();
    
    for i in 1..=1000 {
        let coord = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        sentron.regs.phext[0] = coord;

        // Gather
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));

        // Scatter (write back)
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 0, rs: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
    }

    sentron.spawn(program);

    // Execute and measure
    let start = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = start.elapsed();

    
    
    let ops_per_cycle = stats.ops_per_cycle();

    println!("  SIWs retired:    {}", stats.siws_retired);
    println!("  D-Pipe ops:      {} ({:.1}% util)", stats.d_ops, stats.d_utilization() * 100.0);
    println!("  S-Pipe ops:      {} ({:.1}% util)", stats.s_ops, stats.s_utilization() * 100.0);
    println!("  C-Pipe ops:      {} ({:.1}% util)", stats.c_ops, stats.c_utilization() * 100.0);
    println!("  Total ops:       {}", stats.ops_retired);
    println!("  Time:            {:.2} µs", elapsed.as_micros());
    println!("  Cycles (sim):    {}", stats.cycles);
    println!("  Ops/cycle:       {:.3}", ops_per_cycle);
    
    if ops_per_cycle >= TARGET_OPS_PER_CYCLE {
        println!("  Status:          ✅ PASS (≥{:.1})", TARGET_OPS_PER_CYCLE);
    } else {
        println!("  Status:          ⚠️  GAP ({:.3} short)", TARGET_OPS_PER_CYCLE - ops_per_cycle);
    }
    println!();
}

fn bench_real_inference() {
    println!("─── Benchmark 4: Real Inference Workload ───");
    println!("Workload: Pattern matching + retrieval");
    println!();

    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Store patterns in memory
    let patterns = vec![
        ("hello", 42),
        ("world", 99),
        ("ai", 1337),
        ("vTPU", 2026),
    ];

    for (i, (text, value)) in patterns.iter().enumerate() {
        // Hash text to coordinate
        let mut coord_data = [0u16; 11];
        for (j, byte) in text.bytes().enumerate().take(11) {
            coord_data[j] = byte as u16;
        }
        let coord = PhextCoord::new(coord_data);
        mem.scatter_i64(&coord, *value);
    }

    // Build inference program (query patterns)
    let mut program = Vec::new();
    
    for _ in 0..100 {
        for (text, _) in &patterns {
            let mut coord_data = [0u16; 11];
            for (j, byte) in text.bytes().enumerate().take(11) {
                coord_data[j] = byte as u16;
            }
            let coord = PhextCoord::new(coord_data);
            sentron.regs.phext[0] = coord;

            // Gather value
            program.push(SIW::new(
                DenseOp::DNOP,
                SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));

            // Compute similarity (simplified: just add)
            program.push(SIW::new(
                DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
        }
    }

    sentron.spawn(program);

    // Execute and measure
    let start = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = start.elapsed();

    
    
    let ops_per_cycle = stats.ops_per_cycle();

    println!("  SIWs retired:    {}", stats.siws_retired);
    println!("  D-Pipe ops:      {} ({:.1}% util)", stats.d_ops, stats.d_utilization() * 100.0);
    println!("  S-Pipe ops:      {} ({:.1}% util)", stats.s_ops, stats.s_utilization() * 100.0);
    println!("  C-Pipe ops:      {} ({:.1}% util)", stats.c_ops, stats.c_utilization() * 100.0);
    println!("  Total ops:       {}", stats.ops_retired);
    println!("  Time:            {:.2} µs", elapsed.as_micros());
    println!("  Cycles (sim):    {}", stats.cycles);
    println!("  Ops/cycle:       {:.3}", ops_per_cycle);
    println!("  Queries/sec:     {:.2}M", (400.0 / elapsed.as_secs_f64()) / 1e6);
    
    if ops_per_cycle >= TARGET_OPS_PER_CYCLE {
        println!("  Status:          ✅ PASS (≥{:.1})", TARGET_OPS_PER_CYCLE);
    } else {
        println!("  Status:          ⚠️  GAP ({:.3} short)", TARGET_OPS_PER_CYCLE - ops_per_cycle);
    }
    println!();
}
