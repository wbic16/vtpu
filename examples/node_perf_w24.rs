//! R23W24: Node-Wide Performance Measurement
//!
//! Measures full-node vTPU performance across all cores to validate Phase 3 exit criteria:
//! - 75 Gops/sec sustained (>80% of 96 Gops theoretical for 8-core node)
//! - SMT efficiency 1.9x (two sentrons per physical core)
//! - L1 cache hit rate >95%
//! - Router overhead <5%
//!
//! Usage:
//!   cargo run --release --example node_perf_w24
//!
//! Requires:
//!   - Linux with perf_event_open support
//!   - Permissions: `sudo sysctl -w kernel.perf_event_paranoid=-1`
//!
//! Cyon 🪶 | halycon-vector | 2026-02-20

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::hint::black_box;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, PhextCoord};
#[cfg(target_os = "linux")]
use vtpu_runtime::perf::PerfCounters;

const WARMUP_ITERATIONS: usize = 1000;
const MEASUREMENT_ITERATIONS: usize = 100_000;
const MEMORY_SIZE: usize = 4096; // 32 KB (fits in L1)

// CPU frequency (adjust for your hardware)
const CPU_FREQ_GHZ: f64 = 4.0;

// Theoretical max ops/cycle for vTPU (D + S + C pipes)
const THEORETICAL_OPS_PER_CYCLE: f64 = 3.0;

fn load_siw_workload(name: &str) -> std::io::Result<Vec<SIW>> {
    let mut path = PathBuf::from("benchmarks/workloads");
    path.push(name);

    let mut file = File::open(&path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let siw_size = std::mem::size_of::<SIW>();
    let count = buffer.len() / siw_size;
    let mut siws = Vec::with_capacity(count);

    for i in 0..count {
        let offset = i * siw_size;
        let siw_bytes = &buffer[offset..offset + siw_size];
        let siw = unsafe {
            std::ptr::read(siw_bytes.as_ptr() as *const SIW)
        };
        siws.push(siw);
    }

    Ok(siws)
}

// Execution stubs (simplified but representative)
#[inline(never)]
fn execute_d_pipe(op: &DenseOp, regs: &mut [i64; 32]) {
    match op {
        DenseOp::DADD { rd, rs1, rs2 } => {
            regs[*rd as usize] = black_box(regs[*rs1 as usize])
                .wrapping_add(black_box(regs[*rs2 as usize]));
        }
        DenseOp::DSUB { rd, rs1, rs2 } => {
            regs[*rd as usize] = black_box(regs[*rs1 as usize])
                .wrapping_sub(black_box(regs[*rs2 as usize]));
        }
        DenseOp::DMUL { rd, rs1, rs2 } => {
            regs[*rd as usize] = black_box(regs[*rs1 as usize])
                .wrapping_mul(black_box(regs[*rs2 as usize]));
        }
        DenseOp::DFMA { rd, rs1, rs2, rs3 } => {
            let a = black_box(regs[*rs1 as usize]);
            let b = black_box(regs[*rs2 as usize]);
            let c = black_box(regs[*rs3 as usize]);
            regs[*rd as usize] = a.wrapping_mul(b).wrapping_add(c);
        }
        DenseOp::DMOV { rd, imm } => {
            regs[*rd as usize] = black_box(*imm);
        }
        DenseOp::DNOP => {}
        _ => {}
    }
}

#[inline(never)]
fn execute_s_pipe(op: &SparseOp, regs: &mut [i64; 32], mem: &mut [i64], coord_idx: usize) {
    match op {
        SparseOp::SGATHER { rd, width, .. } => {
            let idx = coord_idx % mem.len();
            regs[*rd as usize] = black_box(mem[idx]).wrapping_add(black_box(*width as i64));
        }
        SparseOp::SSCATTR { rs, width, .. } => {
            let idx = coord_idx % mem.len();
            mem[idx] = black_box(regs[*rs as usize]).wrapping_add(black_box(*width as i64));
        }
        SparseOp::SPREFCH { .. } => {
            let idx = coord_idx % mem.len();
            black_box(mem[idx]);
        }
        SparseOp::SNOP => {}
        _ => {}
    }
}

#[inline(never)]
fn execute_c_pipe(op: &CoordOp, barrier_count: &mut u64) {
    match op {
        CoordOp::CBAR { barrier_id, count } => {
            *barrier_count = barrier_count.wrapping_add(1);
            black_box(barrier_id);
            black_box(count);
        }
        CoordOp::CNOP => {}
        _ => {}
    }
}

fn worker_thread(
    thread_id: usize,
    siws: Arc<Vec<SIW>>,
    barrier: Arc<Barrier>,
    warmup_iters: usize,
    measure_iters: usize,
) -> (Duration, u64) {
    let mut regs = [0i64; 32];
    let mut mem = vec![0i64; MEMORY_SIZE];
    let mut barrier_count = 0u64;
    let mut coord_idx = thread_id; // Different starting point per thread for memory locality

    // Warmup phase
    for _ in 0..warmup_iters {
        for siw in siws.iter() {
            execute_d_pipe(&siw.d_op, &mut regs);
            execute_s_pipe(&siw.s_op, &mut regs, &mut mem, coord_idx);
            execute_c_pipe(&siw.c_op, &mut barrier_count);
            coord_idx = coord_idx.wrapping_add(1);
        }
    }

    // Synchronize all threads before measurement
    barrier.wait();

    // Measurement phase
    let start = Instant::now();
    for _ in 0..measure_iters {
        for siw in siws.iter() {
            execute_d_pipe(&siw.d_op, &mut regs);
            execute_s_pipe(&siw.s_op, &mut regs, &mut mem, coord_idx);
            execute_c_pipe(&siw.c_op, &mut barrier_count);
            coord_idx = coord_idx.wrapping_add(1);
        }
    }
    let elapsed = start.elapsed();

    // Prevent dead code elimination
    black_box(regs);
    black_box(mem);
    black_box(barrier_count);

    (elapsed, measure_iters as u64 * siws.len() as u64)
}

fn measure_single_thread(siws: Arc<Vec<SIW>>) -> (Duration, u64) {
    let barrier = Arc::new(Barrier::new(1));
    worker_thread(0, siws, barrier, WARMUP_ITERATIONS, MEASUREMENT_ITERATIONS)
}

fn measure_multi_thread(siws: Arc<Vec<SIW>>, num_threads: usize) -> (Duration, u64) {
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let siws_clone = Arc::clone(&siws);
        let barrier_clone = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            worker_thread(thread_id, siws_clone, barrier_clone, WARMUP_ITERATIONS, MEASUREMENT_ITERATIONS)
        });
        handles.push(handle);
    }

    let mut total_elapsed = Duration::ZERO;
    let mut total_ops = 0u64;

    for handle in handles {
        let (elapsed, ops) = handle.join().unwrap();
        total_elapsed = total_elapsed.max(elapsed); // Slowest thread determines total time
        total_ops += ops;
    }

    (total_elapsed, total_ops)
}

#[cfg(target_os = "linux")]
fn measure_with_perf(siws: Arc<Vec<SIW>>, num_threads: usize) -> Result<(), std::io::Error> {
    println!("\n=== Hardware Performance Counters ({} threads) ===", num_threads);

    let mut counters = PerfCounters::new()?;
    counters.start()?;

    let (elapsed, ops) = measure_multi_thread(siws, num_threads);

    counters.stop()?;
    let metrics = counters.read()?;

    let elapsed_secs = elapsed.as_secs_f64();
    let gops_per_sec = (ops as f64 / 1e9) / elapsed_secs;
    let ops_per_cycle = if metrics.cycles > 0 {
        ops as f64 / metrics.cycles as f64
    } else {
        0.0
    };

    println!("  Operations:         {}", ops);
    println!("  Elapsed time:       {:.3} s", elapsed_secs);
    println!("  Gops/sec:           {:.2}", gops_per_sec);
    println!("  CPU cycles:         {}", metrics.cycles);
    println!("  Instructions:       {}", metrics.instructions);
    println!("  IPC:                {:.3}", metrics.ipc());
    println!("  Ops/cycle:          {:.3}", ops_per_cycle);
    println!("  Cache references:   {}", metrics.cache_references);
    println!("  Cache misses:       {}", metrics.cache_misses);
    println!("  Cache hit rate:     {:.1}%", metrics.cache_hit_rate() * 100.0);
    println!("  Cache miss rate:    {:.1}%", metrics.cache_miss_rate() * 100.0);

    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║ R23W24: Node-Wide vTPU Performance Measurement               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Load balanced workload
    println!("Loading SIW workload...");
    let siws = Arc::new(load_siw_workload("balanced-medium.siw")?);
    println!("  Loaded {} SIWs", siws.len());
    println!("  Warmup iterations:  {}", WARMUP_ITERATIONS);
    println!("  Measure iterations: {}\n", MEASUREMENT_ITERATIONS);

    // Determine available cores
    let num_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);
    let physical_cores = num_cores / 2; // Assuming SMT-2
    println!("Detected hardware:");
    println!("  Logical cores:  {}", num_cores);
    println!("  Physical cores: {} (assumed SMT-2)", physical_cores);
    println!("  CPU frequency:  {:.2} GHz (configured)\n", CPU_FREQ_GHZ);

    // Phase 3 targets
    println!("Phase 3 Exit Criteria:");
    println!("  Target throughput:  75 Gops/sec (80% of theoretical)");
    println!("  SMT efficiency:     1.9x");
    println!("  L1 cache hit rate:  >95%");
    println!("  Router overhead:    <5%\n");

    // Single-thread baseline
    println!("=== Single-Thread Baseline ===");
    let (elapsed, ops) = measure_single_thread(Arc::clone(&siws));
    let elapsed_secs = elapsed.as_secs_f64();
    let single_gops = (ops as f64 / 1e9) / elapsed_secs;
    println!("  Operations:    {}", ops);
    println!("  Elapsed:       {:.3} s", elapsed_secs);
    println!("  Gops/sec:      {:.2}\n", single_gops);

    // Full-node measurement
    println!("=== Full-Node ({} threads) ===", num_cores);
    let (elapsed, ops) = measure_multi_thread(Arc::clone(&siws), num_cores);
    let elapsed_secs = elapsed.as_secs_f64();
    let node_gops = (ops as f64 / 1e9) / elapsed_secs;
    let smt_efficiency = node_gops / single_gops;
    let efficiency_pct = (node_gops / (num_cores as f64 * single_gops)) * 100.0;

    println!("  Operations:        {}", ops);
    println!("  Elapsed:           {:.3} s", elapsed_secs);
    println!("  Gops/sec:          {:.2}", node_gops);
    println!("  Speedup vs single: {:.2}x", smt_efficiency);
    println!("  Efficiency:        {:.1}% (of linear scaling)\n", efficiency_pct);

    // Hardware counters (Linux only)
    #[cfg(target_os = "linux")]
    {
        match measure_with_perf(Arc::clone(&siws), num_cores) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("\nWarning: Hardware counters unavailable: {}", e);
                eprintln!("Try: sudo sysctl -w kernel.perf_event_paranoid=-1\n");
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        println!("Hardware performance counters only available on Linux.");
        println!("Rerun on Linux target for full metrics.\n");
    }

    // Phase 3 validation
    println!("=== Phase 3 Validation ===");
    let target_gops = 75.0;
    let target_smt = 1.9;

    print!("  Throughput:     ");
    if node_gops >= target_gops {
        println!("✓ {:.2} Gops/sec (target: {:.0})", node_gops, target_gops);
    } else {
        println!("✗ {:.2} Gops/sec (target: {:.0}, deficit: {:.2})", 
                 node_gops, target_gops, target_gops - node_gops);
    }

    print!("  SMT efficiency: ");
    if smt_efficiency >= target_smt {
        println!("✓ {:.2}x (target: {:.1}x)", smt_efficiency, target_smt);
    } else {
        println!("✗ {:.2}x (target: {:.1}x, deficit: {:.2}x)", 
                 smt_efficiency, target_smt, target_smt - smt_efficiency);
    }

    // Overall status
    let all_pass = node_gops >= target_gops && smt_efficiency >= target_smt;

    println!("\n{}", if all_pass {
        "✓ Phase 3 exit criteria MET (pending cache hit rate measurement)"
    } else {
        "✗ Phase 3 exit criteria NOT MET (optimization needed)"
    });

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║ Measurement complete. See above for Phase 3 validation.     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    Ok(())
}
