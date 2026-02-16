//! Performance Counter Validation
//!
//! Measures real hardware performance using Linux perf counters and compares
//! against interpreter predictions from W4.
//!
//! Usage:
//!   cargo run --release --example perf_validation
//!
//! Requires:
//!   - Linux with perf_event_open support
//!   - Permissions: may need `sudo sysctl -w kernel.perf_event_paranoid=-1`

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::hint::black_box;
use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, perf::PerfCounters};

#[allow(dead_code)]
const CPU_FREQ_GHZ: f64 = 4.0; // Adjust for your hardware

fn load_siw_file(path: &PathBuf) -> std::io::Result<Vec<SIW>> {
    let mut file = File::open(path)?;
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

// Simplified execution stubs (same as benchmark_runner)
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
        _ => {}
    }
}

#[inline(never)]
fn execute_s_pipe(op: &SparseOp, regs: &mut [i64; 32], mem: &mut Vec<i64>, coord_idx: usize) {
    match op {
        SparseOp::SGATHER { rd, width, .. } => {
            let idx = coord_idx % mem.len();
            regs[*rd as usize] = black_box(mem[idx]) + black_box(*width as i64);
        }
        SparseOp::SSCATTR { rs, width, .. } => {
            let idx = coord_idx % mem.len();
            mem[idx] = black_box(regs[*rs as usize]) + black_box(*width as i64);
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
            *barrier_count = black_box(*barrier_count + 1);
            black_box(barrier_id);
            black_box(count);
        }
        CoordOp::CNOP => {}
        _ => {}
    }
}

#[inline(never)]
fn execute_siw(siw: &SIW, regs: &mut [i64; 32], mem: &mut Vec<i64>, 
               coord_idx: usize, barrier_count: &mut u64) {
    execute_d_pipe(&siw.d_op, regs);
    execute_s_pipe(&siw.s_op, regs, mem, coord_idx);
    execute_c_pipe(&siw.c_op, barrier_count);
    black_box(siw);
}

fn count_active_ops(siws: &[SIW]) -> usize {
    siws.iter().map(|siw| {
        let mut ops = 0;
        if !matches!(siw.d_op, DenseOp::DNOP) { ops += 1; }
        if !matches!(siw.s_op, SparseOp::SNOP) { ops += 1; }
        if !matches!(siw.c_op, CoordOp::CNOP) { ops += 1; }
        ops
    }).sum()
}

fn validate_workload(name: &str, siws: &[SIW], perf: &PerfCounters, interpreter_ops_per_cycle: f64) -> std::io::Result<()> {
    let mut regs = [0i64; 32];
    regs[1] = 42;
    regs[2] = 10;
    regs[3] = 5;
    
    let mut mem = vec![0i64; 1024];
    let mut barrier_count = 0u64;
    
    // Warmup (not measured)
    for (idx, siw) in siws.iter().enumerate() {
        execute_siw(siw, &mut regs, &mut mem, idx, &mut barrier_count);
    }
    
    // Reset counters
    perf.reset()?;
    
    // Measured run
    perf.start()?;
    for (idx, siw) in siws.iter().enumerate() {
        execute_siw(siw, &mut regs, &mut mem, idx, &mut barrier_count);
    }
    perf.stop()?;
    
    // Read metrics
    let metrics = perf.read()?;
    
    // Prevent optimization
    black_box(regs);
    black_box(mem);
    black_box(barrier_count);
    
    // Calculate metrics
    let active_ops = count_active_ops(siws);
    let hardware_ops_per_cycle = active_ops as f64 / metrics.cycles as f64;
    let deviation = ((hardware_ops_per_cycle - interpreter_ops_per_cycle) / interpreter_ops_per_cycle * 100.0).abs();
    
    // Print results
    println!("{:<25} {:>10} SIWs", name, siws.len());
    println!("  Active ops:             {:>10}", active_ops);
    println!("  Cycles:                 {:>10}", metrics.cycles);
    println!("  Instructions:           {:>10}", metrics.instructions);
    println!("  IPC:                    {:>10.2}", metrics.ipc());
    println!("  Ops/cycle (hardware):   {:>10.3}", hardware_ops_per_cycle);
    println!("  Ops/cycle (interpreter):{:>10.3}", interpreter_ops_per_cycle);
    println!("  Deviation:              {:>9.1}%", deviation);
    println!("  Cache references:       {:>10}", metrics.cache_references);
    println!("  Cache misses:           {:>10}", metrics.cache_misses);
    println!("  Cache hit rate:         {:>9.1}%", metrics.cache_hit_rate() * 100.0);
    
    // Validation
    let status = if deviation < 10.0 {
        "✅ PASS (within 10%)"
    } else if deviation < 25.0 {
        "⚠️  WARN (10-25% deviation)"
    } else {
        "❌ FAIL (>25% deviation)"
    };
    println!("  Status:                 {}", status);
    println!();
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("vTPU Performance Counter Validation");
    println!("Comparing interpreter predictions vs hardware reality\n");
    
    // Try to create perf counters
    let perf = match PerfCounters::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Failed to create perf counters: {}", e);
            eprintln!();
            eprintln!("This may require:");
            eprintln!("  sudo sysctl -w kernel.perf_event_paranoid=-1");
            eprintln!();
            eprintln!("Or run with:");
            eprintln!("  sudo ./target/release/examples/perf_validation");
            return Err(e);
        }
    };
    
    println!("✅ Performance counters initialized\n");
    println!("{}", "=".repeat(75));
    println!();
    
    // Workload definitions (from W4 KPI dashboard)
    let workloads = [
        ("Balanced (33/33/33)", "benchmarks/workloads/balanced-medium.siw", 3.00),
        ("D-Heavy (80/10/10)", "benchmarks/workloads/d-heavy-high.siw", 2.50),
        ("S-Heavy (10/80/10)", "benchmarks/workloads/s-heavy-high.siw", 2.50),
    ];
    
    for (name, path, interpreter_prediction) in &workloads {
        match load_siw_file(&PathBuf::from(path)) {
            Ok(siws) => {
                if let Err(e) = validate_workload(name, &siws, &perf, *interpreter_prediction) {
                    eprintln!("Error validating {}: {}", name, e);
                }
            }
            Err(e) => {
                eprintln!("⚠️  Skipping {} (file not found): {}", name, e);
                println!();
            }
        }
    }
    
    println!("{}", "=".repeat(75));
    println!("\n✅ Validation complete");
    println!("\nNext steps:");
    println!("  - If deviation >25%: Tune scheduler, improve locality");
    println!("  - If deviation <10%: Interpreter is accurate ✓");
    println!("  - Compare cache hit rates against 95% target");
    
    Ok(())
}
