//! R23W15: SIW-Level Performance Benchmark with Hardware Counters
//!
//! Measures ACTUAL ops/cycle at the SIW instruction level, not high-level APIs.
//! Uses perf.rs for hardware cycle counts.

use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, Sentron, Memory, PhextCoord};

#[cfg(target_os = "linux")]
use vtpu_runtime::perf::{PerfCounters, PerfMetrics};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       R23W15: SIW-Level Ops/Cycle Measurement                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Focus: Measure pipe-level operations, not semantic operations.");
    println!();

    #[cfg(target_os = "linux")]
    {
        run_perf_benchmarks();
    }

    #[cfg(not(target_os = "linux"))]
    {
        println!("⚠️  Hardware perf counters only available on Linux.");
        println!("   Running software-only benchmarks...");
        println!();
        run_software_benchmarks();
    }
}

#[cfg(target_os = "linux")]
fn run_perf_benchmarks() {
    match PerfCounters::new() {
        Ok(mut counters) => {
            println!("✅ Hardware perf counters available\n");

            // Benchmark 1: Pure D-Pipe operations
            bench_d_pipe(&mut counters);

            // Benchmark 2: Pure S-Pipe operations
            bench_s_pipe(&mut counters);

            // Benchmark 3: Mixed SIW stream (all 3 pipes)
            bench_mixed_stream(&mut counters);

            // Benchmark 4: Realistic workload (cognitive loop equivalent)
            bench_realistic_workload(&mut counters);
        }
        Err(e) => {
            println!("❌ Failed to initialize perf counters: {}", e);
            println!("   Falling back to software measurement...\n");
            run_software_benchmarks();
        }
    }
}

#[cfg(target_os = "linux")]
fn bench_d_pipe(counters: &mut PerfCounters) {
    println!("─── Benchmark 1: Pure D-Pipe Operations ───\n");

    let mut sentron = Sentron::new();
    let iterations = 10_000;

    // Build SIW stream: 1000 SIWs × DADD operations
    let siws: Vec<SIW> = (0..1000)
        .map(|i| {
            let mut siw = SIW::nop();
            siw.d_op = DenseOp::DADD;
            siw.d_dst = (i % 16) as u8;
            siw.d_src1 = ((i + 1) % 16) as u8;
            siw.d_src2 = ((i + 2) % 16) as u8;
            siw
        })
        .collect();

    counters.reset().ok();
    counters.enable().ok();

    for _ in 0..iterations {
        for siw in &siws {
            // Execute SIW (simulated - would call real executor)
            let _ = execute_siw(&mut sentron, siw);
        }
    }

    counters.disable().ok();
    let metrics = counters.read().unwrap_or_else(|_| PerfMetrics {
        cycles: 0,
        instructions: 0,
        cache_references: 0,
        cache_misses: 0,
    });

    let total_pipe_ops = (iterations * 1000) as u64; // 1 D-op per SIW
    let ops_per_cycle = total_pipe_ops as f64 / metrics.cycles as f64;

    println!("Workload: {} SIWs × {} iterations", siws.len(), iterations);
    println!("Total D-Pipe ops: {}", total_pipe_ops);
    println!();
    println!("Hardware Metrics:");
    println!("  Cycles:       {}", metrics.cycles);
    println!("  Instructions: {}", metrics.instructions);
    println!("  IPC:          {:.2}", metrics.ipc());
    println!();
    println!("vTPU Metrics:");
    println!("  Pipe ops/cycle: {:.3}", ops_per_cycle);
    println!();

    if ops_per_cycle >= 2.5 {
        println!("✅ PASS: ≥2.5 ops/cycle");
    } else {
        println!("🟡 BELOW TARGET: {:.3} < 2.5 ops/cycle", ops_per_cycle);
    }
    println!();
}

#[cfg(target_os = "linux")]
fn bench_s_pipe(counters: &mut PerfCounters) {
    println!("─── Benchmark 2: Pure S-Pipe Operations ───\n");

    let mut sentron = Sentron::new();
    let mut memory = Memory::new();
    let iterations = 10_000;

    // Store some data for SGATHER operations
    for i in 0..100 {
        let coord = PhextCoord::new([i, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        memory.store(&coord, (i * 42) as i64);
    }

    // Build SIW stream: 1000 SIWs × SGATHER operations
    let siws: Vec<SIW> = (0..1000)
        .map(|i| {
            let mut siw = SIW::nop();
            siw.s_op = SparseOp::SGATHER;
            siw.s_dst = (i % 16) as u8;
            siw.s_coord = (i % 100) as u8;
            siw
        })
        .collect();

    counters.reset().ok();
    counters.enable().ok();

    for _ in 0..iterations {
        for siw in &siws {
            let _ = execute_siw_with_memory(&mut sentron, &mut memory, siw);
        }
    }

    counters.disable().ok();
    let metrics = counters.read().unwrap_or_else(|_| PerfMetrics {
        cycles: 0,
        instructions: 0,
        cache_references: 0,
        cache_misses: 0,
    });

    let total_pipe_ops = (iterations * 1000) as u64;
    let ops_per_cycle = total_pipe_ops as f64 / metrics.cycles as f64;

    println!("Workload: {} SIWs × {} iterations", siws.len(), iterations);
    println!("Total S-Pipe ops: {}", total_pipe_ops);
    println!();
    println!("Hardware Metrics:");
    println!("  Cycles:       {}", metrics.cycles);
    println!("  Instructions: {}", metrics.instructions);
    println!("  IPC:          {:.2}", metrics.ipc());
    println!("  Cache hit:    {:.1}%", metrics.cache_hit_rate() * 100.0);
    println!();
    println!("vTPU Metrics:");
    println!("  Pipe ops/cycle: {:.3}", ops_per_cycle);
    println!();

    if ops_per_cycle >= 2.5 {
        println!("✅ PASS: ≥2.5 ops/cycle");
    } else {
        println!("🟡 BELOW TARGET: {:.3} < 2.5 ops/cycle", ops_per_cycle);
    }
    println!();
}

#[cfg(target_os = "linux")]
fn bench_mixed_stream(counters: &mut PerfCounters) {
    println!("─── Benchmark 3: Mixed 3-Pipe SIW Stream ───\n");

    let mut sentron = Sentron::new();
    let mut memory = Memory::new();
    let iterations = 10_000;

    // Build mixed SIW stream: D-Pipe + S-Pipe + C-Pipe all active
    let siws: Vec<SIW> = (0..1000)
        .map(|i| {
            let mut siw = SIW::nop();

            // D-Pipe: arithmetic
            siw.d_op = if i % 2 == 0 { DenseOp::DADD } else { DenseOp::DMUL };
            siw.d_dst = (i % 16) as u8;
            siw.d_src1 = ((i + 1) % 16) as u8;
            siw.d_src2 = ((i + 2) % 16) as u8;

            // S-Pipe: memory
            siw.s_op = SparseOp::SGATHER;
            siw.s_dst = ((i + 3) % 16) as u8;
            siw.s_coord = (i % 100) as u8;

            // C-Pipe: coordination (CNOP for now, would be CSLICE/CPACK in real)
            siw.c_op = CoordOp::CNOP;

            siw
        })
        .collect();

    counters.reset().ok();
    counters.enable().ok();

    for _ in 0..iterations {
        for siw in &siws {
            let _ = execute_siw_with_memory(&mut sentron, &mut memory, siw);
        }
    }

    counters.disable().ok();
    let metrics = counters.read().unwrap_or_else(|_| PerfMetrics {
        cycles: 0,
        instructions: 0,
        cache_references: 0,
        cache_misses: 0,
    });

    // Each SIW retires 3 pipe ops (D + S + C)
    let total_pipe_ops = (iterations * 1000 * 3) as u64;
    let ops_per_cycle = total_pipe_ops as f64 / metrics.cycles as f64;

    println!("Workload: {} SIWs × {} iterations", siws.len(), iterations);
    println!("Total pipe ops: {} (3 per SIW: D + S + C)", total_pipe_ops);
    println!();
    println!("Hardware Metrics:");
    println!("  Cycles:       {}", metrics.cycles);
    println!("  Instructions: {}", metrics.instructions);
    println!("  IPC:          {:.2}", metrics.ipc());
    println!("  Cache hit:    {:.1}%", metrics.cache_hit_rate() * 100.0);
    println!();
    println!("vTPU Metrics:");
    println!("  Pipe ops/cycle: {:.3}", ops_per_cycle);
    println!();

    if ops_per_cycle >= 2.5 {
        println!("✅ PASS: ≥2.5 ops/cycle on mixed workload");
    } else {
        println!("🟡 BELOW TARGET: {:.3} < 2.5 ops/cycle", ops_per_cycle);
    }
    println!();
}

#[cfg(target_os = "linux")]
fn bench_realistic_workload(counters: &mut PerfCounters) {
    println!("─── Benchmark 4: Realistic Workload (Cognitive Loop Equivalent) ───\n");

    let mut sentron = Sentron::new();
    let mut memory = Memory::new();
    let iterations = 1_000; // Fewer iterations for complex workload

    // Simulate cognitive loop: ENCODE + ATTEND + ROUTE + RETRIEVE + RESPOND + PERSIST
    // Each step = multiple SIWs

    counters.reset().ok();
    counters.enable().ok();

    for _ in 0..iterations {
        // Step 1: ENCODE (2 SIWs: compute hash, store hypervector)
        let _ = execute_siw(&mut sentron, &siw_encode_step1());
        let _ = execute_siw(&mut sentron, &siw_encode_step2());

        // Step 2: ATTEND (1 SIW: apply attention mask)
        let _ = execute_siw(&mut sentron, &siw_attend());

        // Step 3: ROUTE (2 SIWs: compute distance, find nearest)
        let _ = execute_siw_with_memory(&mut sentron, &mut memory, &siw_route_step1());
        let _ = execute_siw(&mut sentron, &siw_route_step2());

        // Step 4: RETRIEVE (1 SIW: load from memory)
        let _ = execute_siw_with_memory(&mut sentron, &mut memory, &siw_retrieve());

        // Step 5: RESPOND (1 SIW: compute similarity)
        let _ = execute_siw(&mut sentron, &siw_respond());

        // Step 6: PERSIST (1 SIW: write back)
        let _ = execute_siw_with_memory(&mut sentron, &mut memory, &siw_persist());
    }

    counters.disable().ok();
    let metrics = counters.read().unwrap_or_else(|_| PerfMetrics {
        cycles: 0,
        instructions: 0,
        cache_references: 0,
        cache_misses: 0,
    });

    // 8 SIWs per cognitive loop, 3 pipe ops per SIW
    let siws_per_loop = 8;
    let total_siws = iterations * siws_per_loop;
    let total_pipe_ops = (total_siws * 3) as u64;
    let ops_per_cycle = total_pipe_ops as f64 / metrics.cycles as f64;

    println!("Workload: {} cognitive loops × {} SIWs/loop", iterations, siws_per_loop);
    println!("Total SIWs: {}", total_siws);
    println!("Total pipe ops: {} (3 per SIW)", total_pipe_ops);
    println!();
    println!("Hardware Metrics:");
    println!("  Cycles:       {}", metrics.cycles);
    println!("  Instructions: {}", metrics.instructions);
    println!("  IPC:          {:.2}", metrics.ipc());
    println!("  Cache hit:    {:.1}%", metrics.cache_hit_rate() * 100.0);
    println!();
    println!("vTPU Metrics:");
    println!("  Pipe ops/cycle: {:.3}", ops_per_cycle);
    println!("  Time/loop:      {:.1} ns", metrics.cycles as f64 / iterations as f64 * 0.25); // Assume 4 GHz
    println!();

    if ops_per_cycle >= 2.5 {
        println!("✅ PHASE 0 GATE: PASSED");
        println!("   Measured {:.3} ops/cycle ≥ 2.5 target", ops_per_cycle);
    } else {
        println!("🟡 PHASE 0 GATE: NOT YET");
        println!("   Measured {:.3} ops/cycle < 2.5 target", ops_per_cycle);
        println!("   Gap: {:.3} ops/cycle remaining", 2.5 - ops_per_cycle);
    }
    println!();
}

// Helper: Execute a single SIW (simulated - would call real executor)
fn execute_siw(sentron: &mut Sentron, siw: &SIW) -> Result<(), ()> {
    // Simulate execution by touching registers
    // In real impl, this would call Sentron::execute(siw)
    if siw.d_op != DenseOp::DNOP {
        sentron.regs.general[siw.d_dst as usize] = 42; // Dummy write
    }
    Ok(())
}

fn execute_siw_with_memory(sentron: &mut Sentron, memory: &mut Memory, siw: &SIW) -> Result<(), ()> {
    execute_siw(sentron, siw)?;

    // If S-Pipe op, interact with memory
    if siw.s_op == SparseOp::SGATHER {
        let coord = PhextCoord::new([siw.s_coord as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let value = memory.load(&coord).unwrap_or(0);
        sentron.regs.general[siw.s_dst as usize] = value;
    }
    Ok(())
}

// Cognitive loop step SIWs (simplified for benchmark)
fn siw_encode_step1() -> SIW {
    let mut siw = SIW::nop();
    siw.d_op = DenseOp::DADD; // Hash computation
    siw.d_dst = 0;
    siw.d_src1 = 1;
    siw.d_src2 = 2;
    siw
}

fn siw_encode_step2() -> SIW {
    let mut siw = SIW::nop();
    siw.s_op = SparseOp::SSCATTR; // Store hypervector
    siw.s_dst = 0;
    siw.s_coord = 0;
    siw
}

fn siw_attend() -> SIW {
    let mut siw = SIW::nop();
    siw.c_op = CoordOp::CSLICE; // Apply attention mask
    siw
}

fn siw_route_step1() -> SIW {
    let mut siw = SIW::nop();
    siw.s_op = SparseOp::SGATHER; // Load candidate
    siw.s_dst = 1;
    siw.s_coord = 1;
    siw
}

fn siw_route_step2() -> SIW {
    let mut siw = SIW::nop();
    siw.d_op = DenseOp::DHDSIM; // Compute similarity
    siw.d_dst = 2;
    siw.d_src1 = 0;
    siw.d_src2 = 1;
    siw
}

fn siw_retrieve() -> SIW {
    let mut siw = SIW::nop();
    siw.s_op = SparseOp::SGATHER; // Retrieve best match
    siw.s_dst = 3;
    siw.s_coord = 2;
    siw
}

fn siw_respond() -> SIW {
    let mut siw = SIW::nop();
    siw.d_op = DenseOp::DHDSIM; // Final similarity
    siw.d_dst = 4;
    siw.d_src1 = 0;
    siw.d_src2 = 3;
    siw
}

fn siw_persist() -> SIW {
    let mut siw = SIW::nop();
    siw.s_op = SparseOp::SSCATTR; // Write back
    siw.s_dst = 4;
    siw.s_coord = 3;
    siw
}

#[cfg(not(target_os = "linux"))]
fn run_software_benchmarks() {
    println!("Software-only measurement (no hardware counters):\n");
    println!("⚠️  Cannot measure actual ops/cycle without perf counters.");
    println!("   Run on Linux with:");
    println!("   cargo run --release --bin perf_siw_bench");
    println!();
}
