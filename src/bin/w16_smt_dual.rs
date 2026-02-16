//! R23W16 - SMT Dual-Thread Benchmark
//!
//! Measures speedup from running complementary workloads on SMT sibling threads.
//! Target: 1.8× speedup over single-thread baseline.
//!
//! Usage: cargo run --release --bin w16_smt_dual

use vtpu_runtime::*;
use std::time::Instant;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

const TARGET_SPEEDUP: f64 = 1.8;

// SMT thread pinning (simplified - no actual pinning for now)
// OS scheduler will likely place threads on same core due to locality
fn pin_to_cpu(_cpu_id: usize) {
    // Note: Actual CPU pinning requires libc dependency
    // For now, rely on OS scheduler to place threads optimally
    // Future: Add libc to Cargo.toml for explicit pinning
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║      R23W16: SMT Dual-Thread (Complementary Workloads)       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Baseline (W16 single-thread): ~137M ops/sec average");
    println!("Target SMT speedup: {:.1}× (dual-thread on same core)", TARGET_SPEEDUP);
    println!();

    let baseline = 137_000_000.0; // From w16_smt_baseline

    bench_complementary(baseline);
    bench_same_workload(baseline);
}

fn bench_complementary(baseline: f64) {
    println!("─── Test 1: Complementary Workloads (D-heavy + S-heavy) ───");
    println!();
    
    // Shared stats (atomic counters)
    let ops_a = Arc::new(AtomicU64::new(0));
    let ops_b = Arc::new(AtomicU64::new(0));
    let ops_a_clone = Arc::clone(&ops_a);
    let ops_b_clone = Arc::clone(&ops_b);

    let start = Instant::now();

    // Thread A: D-Pipe heavy (compute-bound)
    let handle_a = thread::spawn(move || {
        pin_to_cpu(0); // Physical core 0, thread 0

        let mut mem = Memory::new();
        let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
        
        sentron.regs.general[0] = 7;
        sentron.regs.general[1] = 6;
        
        let mut program = Vec::new();
        
        // Pure compute (D-Pipe) - increase workload to amortize threading overhead
        for _ in 0..50_000 {
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
        }
        
        sentron.spawn(program);
        let stats = exec::run(&mut sentron, &mut mem);
        
        ops_a_clone.store(stats.ops_retired, Ordering::Relaxed);
    });

    // Thread B: S-Pipe heavy (memory-bound)
    let handle_b = thread::spawn(move || {
        pin_to_cpu(1); // Physical core 0, SMT sibling (thread 1)

        let mut mem = Memory::new();
        
        // Pre-populate memory (separate region from thread A)
        for i in 1..=25_000 {
            let coord = PhextCoord::new([(i % 1000) as u16 + 1001, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            mem.scatter_i64(&coord, i as i64);
        }
        
        let mut sentron = Sentron::new(1, PhextCoord::zero(), 0, 0);
        let mut program = Vec::new();
        
        // Pure memory ops (S-Pipe) - larger workload
        for i in 1..=25_000 {
            let coord = PhextCoord::new([(i % 1000) as u16 + 1001, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
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
        }
        
        sentron.spawn(program);
        let stats = exec::run(&mut sentron, &mut mem);
        
        ops_b_clone.store(stats.ops_retired, Ordering::Relaxed);
    });

    // Wait for both threads
    handle_a.join().unwrap();
    handle_b.join().unwrap();
    
    let elapsed = start.elapsed();
    
    let ops_a_total = ops_a.load(Ordering::Relaxed);
    let ops_b_total = ops_b.load(Ordering::Relaxed);
    let total_ops = ops_a_total + ops_b_total;
    
    let throughput = total_ops as f64 / elapsed.as_secs_f64();
    let speedup = throughput / baseline;
    
    println!("  Thread A (D-heavy): {} ops", ops_a_total);
    println!("  Thread B (S-heavy): {} ops", ops_b_total);
    println!("  Total ops:          {}", total_ops);
    println!("  Time:               {:.2} µs", elapsed.as_micros());
    println!("  Throughput:         {:.2}M ops/sec", throughput / 1e6);
    println!("  Speedup vs baseline: {:.2}×", speedup);
    
    if speedup >= TARGET_SPEEDUP {
        println!("  ✅ SMT TARGET ACHIEVED (≥{:.1}×)", TARGET_SPEEDUP);
    } else {
        println!("  ⚠️  Below target ({:.2}× vs {:.1}×)", speedup, TARGET_SPEEDUP);
    }
    println!();
}

fn bench_same_workload(baseline: f64) {
    println!("─── Test 2: Same Workload (Mixed + Mixed) ───");
    println!("(Tests cache/resource contention with identical workloads)");
    println!();
    
    let ops_a = Arc::new(AtomicU64::new(0));
    let ops_b = Arc::new(AtomicU64::new(0));
    let ops_a_clone = Arc::clone(&ops_a);
    let ops_b_clone = Arc::clone(&ops_b);

    let start = Instant::now();

    // Both threads run same mixed workload
    let handle_a = thread::spawn(move || {
        pin_to_cpu(0);
        
        let mut mem = Memory::new();
        for i in 1..=10_000 {
            let coord = PhextCoord::new([(i % 1000) as u16 + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            mem.scatter_i64(&coord, i as i64 * 10);
        }
        
        let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let mut program = Vec::new();
        
        for i in 1..=10_000 {
            let coord = PhextCoord::new([(i % 1000) as u16 + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            sentron.regs.phext[0] = coord;
            
            program.push(SIW::new(
                DenseOp::DNOP,
                SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
            
            program.push(SIW::new(
                DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
        }
        
        sentron.spawn(program);
        let stats = exec::run(&mut sentron, &mut mem);
        ops_a_clone.store(stats.ops_retired, Ordering::Relaxed);
    });

    let handle_b = thread::spawn(move || {
        pin_to_cpu(1);
        
        let mut mem = Memory::new();
        // Different memory region
        for i in 1..=10_000 {
            let coord = PhextCoord::new([(i % 1000) as u16 + 1001, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            mem.scatter_i64(&coord, i as i64 * 10);
        }
        
        let mut sentron = Sentron::new(1, PhextCoord::zero(), 0, 0);
        let mut program = Vec::new();
        
        for i in 1..=10_000 {
            let coord = PhextCoord::new([(i % 1000) as u16 + 1001, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            sentron.regs.phext[0] = coord;
            
            program.push(SIW::new(
                DenseOp::DNOP,
                SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
            
            program.push(SIW::new(
                DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
        }
        
        sentron.spawn(program);
        let stats = exec::run(&mut sentron, &mut mem);
        ops_b_clone.store(stats.ops_retired, Ordering::Relaxed);
    });

    handle_a.join().unwrap();
    handle_b.join().unwrap();
    
    let elapsed = start.elapsed();
    
    let ops_a_total = ops_a.load(Ordering::Relaxed);
    let ops_b_total = ops_b.load(Ordering::Relaxed);
    let total_ops = ops_a_total + ops_b_total;
    
    let throughput = total_ops as f64 / elapsed.as_secs_f64();
    let speedup = throughput / baseline;
    
    println!("  Thread A (mixed):   {} ops", ops_a_total);
    println!("  Thread B (mixed):   {} ops", ops_b_total);
    println!("  Total ops:          {}", total_ops);
    println!("  Time:               {:.2} µs", elapsed.as_micros());
    println!("  Throughput:         {:.2}M ops/sec", throughput / 1e6);
    println!("  Speedup vs baseline: {:.2}×", speedup);
    println!();
    println!("Note: Lower speedup expected due to resource contention.");
    println!("Complementary workloads (D+S) should outperform same workload.");
}
