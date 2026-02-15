//! Port Validation Benchmark
//!
//! Tests whether SIW instructions can retire 3-wide on Zen 4 by exercising
//! different execution port combinations.
//!
//! Run with: cargo run --release --example port_validation
//! Perf: perf stat -e cycles,instructions,uops_retired.all \
//!       cargo run --release --example port_validation --test 3wide

use std::time::Instant;
use std::hint::black_box;
use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, PhextCoord};

const ITERATIONS: usize = 10_000;

/// Stub D-Pipe execution (maps to ALU0/1)
#[inline(never)]
fn execute_d_pipe(op: &DenseOp, regs: &mut [i64; 32]) {
    match op {
        DenseOp::DADD { rd, rs1, rs2 } => {
            regs[*rd as usize] = black_box(regs[*rs1 as usize]) + black_box(regs[*rs2 as usize]);
        }
        DenseOp::DSUB { rd, rs1, rs2 } => {
            regs[*rd as usize] = black_box(regs[*rs1 as usize]) - black_box(regs[*rs2 as usize]);
        }
        DenseOp::DMUL { rd, rs1, rs2 } => {
            regs[*rd as usize] = black_box(regs[*rs1 as usize]) * black_box(regs[*rs2 as usize]);
        }
        DenseOp::DFMA { rd, rs1, rs2, rs3 } => {
            let a = black_box(regs[*rs1 as usize]);
            let b = black_box(regs[*rs2 as usize]);
            let c = black_box(regs[*rs3 as usize]);
            regs[*rd as usize] = a * b + c;
        }
        DenseOp::DMOV { rd, imm } => {
            regs[*rd as usize] = black_box(*imm);
        }
        DenseOp::DNOP => {}
        _ => {} // Other ops not needed for PoC
    }
}

/// Stub S-Pipe execution (maps to AGU4/5)
#[inline(never)]
fn execute_s_pipe(op: &SparseOp, regs: &mut [i64; 32], mem: &mut Vec<i64>, coord_idx: usize) {
    match op {
        SparseOp::SGATHER { rd, width, .. } => {
            // Simulate memory load/gather (touches AGU)
            let idx = coord_idx % mem.len();
            regs[*rd as usize] = black_box(mem[idx]) + black_box(*width as i64);
        }
        SparseOp::SSCATTR { rs, width, .. } => {
            // Simulate memory store/scatter (touches AGU)
            let idx = coord_idx % mem.len();
            mem[idx] = black_box(regs[*rs as usize]) + black_box(*width as i64);
        }
        SparseOp::SPREFCH { .. } => {
            // Prefetch hint - simulate cache touch
            let idx = coord_idx % mem.len();
            black_box(mem[idx]);
        }
        SparseOp::SNOP => {}
        _ => {} // Other sparse ops not needed for PoC
    }
}

/// Stub C-Pipe execution (maps to ALU2/3)
#[inline(never)]
fn execute_c_pipe(op: &CoordOp, barrier_count: &mut u64) {
    match op {
        CoordOp::CBAR { barrier_id, count } => {
            // Simulate barrier (atomic increment on ALU2/3)
            *barrier_count = black_box(*barrier_count + 1);
            black_box(barrier_id);
            black_box(count);
        }
        CoordOp::CNOP => {}
        _ => {} // Other coord ops not needed for PoC
    }
}

/// Execute a single SIW (all three pipes)
#[inline(never)]
fn execute_siw(siw: &SIW, regs: &mut [i64; 32], mem: &mut Vec<i64>, 
               coord_idx: usize, barrier_count: &mut u64) {
    // Extract ops (using public accessor methods - adjust if internal)
    // For now, we'll recreate the ops based on what we know about the struct
    
    // Execute each pipe (using the actual ops from the SIW)
    execute_d_pipe(&siw.d_op, regs);
    execute_s_pipe(&siw.s_op, regs, mem, coord_idx);
    execute_c_pipe(&siw.c_op, barrier_count);
    
    black_box(siw); // Prevent optimization
}

/// Generate test stream for single-pipe testing
fn gen_single_pipe_stream(pipe: &str, count: usize) -> Vec<SIW> {
    match pipe {
        "d" => {
            (0..count).map(|_| {
                SIW::new(
                    DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
                    SparseOp::SNOP,
                    CoordOp::CNOP,
                    PhextCoord::zero(),
                )
            }).collect()
        }
        "s" => {
            (0..count).map(|i| {
                SIW::new(
                    DenseOp::DNOP,
                    SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
                    CoordOp::CNOP,
                    PhextCoord::new([(i % 2048) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                )
            }).collect()
        }
        "c" => {
            (0..count).map(|_| {
                SIW::new(
                    DenseOp::DNOP,
                    SparseOp::SNOP,
                    CoordOp::CBAR { barrier_id: 1, count: 1 },
                    PhextCoord::zero(),
                )
            }).collect()
        }
        _ => panic!("Unknown pipe: {}", pipe),
    }
}

/// Generate test stream for dual-pipe testing
fn gen_dual_pipe_stream(pipes: &str, count: usize) -> Vec<SIW> {
    match pipes {
        "ds" => {
            (0..count).map(|i| {
                SIW::new(
                    DenseOp::DADD { rd: 2, rs1: 2, rs2: 1 },
                    SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
                    CoordOp::CNOP,
                    PhextCoord::new([(i % 2048) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                )
            }).collect()
        }
        "dc" => {
            (0..count).map(|_| {
                SIW::new(
                    DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
                    SparseOp::SNOP,
                    CoordOp::CBAR { barrier_id: 1, count: 1 },
                    PhextCoord::zero(),
                )
            }).collect()
        }
        "sc" => {
            (0..count).map(|i| {
                SIW::new(
                    DenseOp::DNOP,
                    SparseOp::SSCATTR { rs: 1, coord_idx: 0, width: 64 },
                    CoordOp::CBAR { barrier_id: 1, count: 1 },
                    PhextCoord::new([(i % 2048) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                )
            }).collect()
        }
        _ => panic!("Unknown pipe combo: {}", pipes),
    }
}

/// Generate 3-wide test stream
fn gen_3wide_stream(count: usize) -> Vec<SIW> {
    (0..count).map(|i| {
        SIW::new(
            DenseOp::DADD { rd: 2, rs1: 2, rs2: 1 },
            SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
            CoordOp::CBAR { barrier_id: 1, count: 1 },
            PhextCoord::new([(i % 2048) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        )
    }).collect()
}

/// Run benchmark on a stream
fn benchmark_stream(name: &str, stream: &[SIW], active_pipes: usize) {
    let mut regs = [0i64; 32];
    regs[2] = 10; // Initialize some registers
    regs[3] = 5;
    
    let mut mem = vec![0i64; 1024]; // 8KB working set (fits in L1)
    let mut barrier_count = 0u64;
    
    // Warm up
    for (idx, siw) in stream.iter().enumerate() {
        execute_siw(siw, &mut regs, &mut mem, idx, &mut barrier_count);
    }
    
    // Timed run
    let start = Instant::now();
    for (idx, siw) in stream.iter().enumerate() {
        execute_siw(siw, &mut regs, &mut mem, idx, &mut barrier_count);
    }
    let elapsed = start.elapsed();
    
    // Prevent optimization
    black_box(regs);
    black_box(mem);
    black_box(barrier_count);
    
    // Calculate metrics
    let total_ops = stream.len() * active_pipes;
    let elapsed_ns = elapsed.as_nanos();
    let ops_per_ns = total_ops as f64 / elapsed_ns as f64;
    
    // Assume 4 GHz CPU → 0.25 ns/cycle
    let cycle_time_ns = 0.25;
    let ops_per_cycle = ops_per_ns * cycle_time_ns;
    
    println!("=== {} ===", name);
    println!("  SIWs executed: {}", stream.len());
    println!("  Active pipes: {}", active_pipes);
    println!("  Total ops: {}", total_ops);
    println!("  Elapsed: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("  Ops/cycle: {:.2}", ops_per_cycle);
    println!();
}

fn main() {
    println!("vTPU Port Validation Benchmark");
    println!("AMD Zen 4 3-Wide Retirement Test\n");
    
    // Phase 1: Single-pipe baselines
    println!("Phase 1: Single-Pipe Baselines");
    benchmark_stream("D-Pipe only", &gen_single_pipe_stream("d", ITERATIONS), 1);
    benchmark_stream("S-Pipe only", &gen_single_pipe_stream("s", ITERATIONS), 1);
    benchmark_stream("C-Pipe only", &gen_single_pipe_stream("c", ITERATIONS), 1);
    
    // Phase 2: Dual-pipe independence
    println!("Phase 2: Dual-Pipe Independence");
    benchmark_stream("D+S Pipes", &gen_dual_pipe_stream("ds", ITERATIONS), 2);
    benchmark_stream("D+C Pipes", &gen_dual_pipe_stream("dc", ITERATIONS), 2);
    benchmark_stream("S+C Pipes", &gen_dual_pipe_stream("sc", ITERATIONS), 2);
    
    // Phase 3: Full 3-wide
    println!("Phase 3: Full 3-Wide Retirement");
    benchmark_stream("D+S+C (3-wide)", &gen_3wide_stream(ITERATIONS), 3);
    
    println!("✅ Port validation complete");
    println!("Run with `perf stat` for detailed microarchitecture counters");
}
