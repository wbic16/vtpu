//! CPI Benchmark for vTPU Autoresearch
//!
//! Measures ops/cycle for the vTPU SIW pipeline.
//! Target: 3.0 ops/cycle on Zen 4 (AMD Ryzen 9)

use std::time::{Duration, Instant};

/// Simple SIW (SIMD Instruction Word) - 3 operations
#[derive(Clone, Copy)]
struct SIW {
    d_op: u64,  // D-pipe: dense compute
    s_op: u64,  // S-pipe: sparse/associative
    c_op: u64,  // C-pipe: communication
}

impl SIW {
    fn execute(&self, state: &mut [f64; 64]) {
        // D-pipe: FMA-style operation
        let d_idx = (self.d_op & 0x3F) as usize;
        let d_src1 = ((self.d_op >> 6) & 0x3F) as usize;
        let d_src2 = ((self.d_op >> 12) & 0x3F) as usize;
        state[d_idx] = state[d_src1].mul_add(state[d_src2], state[d_idx]);

        // S-pipe: sparse lookup
        let s_idx = (self.s_op & 0x3F) as usize;
        let s_key = ((self.s_op >> 6) & 0xFF) as f64;
        state[s_idx] = (state[s_idx] + s_key).sin();

        // C-pipe: broadcast/gather
        let c_dst = (self.c_op & 0x3F) as usize;
        let c_src = ((self.c_op >> 6) & 0x3F) as usize;
        state[c_dst] = state[c_src];
    }
}

/// Generate a workload of SIWs
fn generate_workload(n: usize) -> Vec<SIW> {
    let mut siws = Vec::with_capacity(n);
    for i in 0..n {
        siws.push(SIW {
            d_op: (i as u64 * 7) % 0x3FFFF,
            s_op: (i as u64 * 13) % 0x3FFF,
            c_op: (i as u64 * 17) % 0xFFF,
        });
    }
    siws
}

/// Execute workload and measure performance
fn benchmark(duration_secs: u64) -> BenchResult {
    let workload = generate_workload(100_000);
    let mut state = [0.0f64; 64];
    
    // Initialize state
    for i in 0..64 {
        state[i] = (i as f64) * 0.1;
    }

    let start = Instant::now();
    let target_duration = Duration::from_secs(duration_secs);
    
    let mut total_ops: u64 = 0;
    let mut iterations: u64 = 0;

    while start.elapsed() < target_duration {
        for siw in &workload {
            siw.execute(&mut state);
            total_ops += 3; // 3 ops per SIW
        }
        iterations += 1;
    }

    let elapsed = start.elapsed();
    
    // Estimate cycles (rough approximation)
    // On Zen 4 @ ~5GHz, 1 second ≈ 5B cycles
    let estimated_hz = 5_000_000_000u64;
    let estimated_cycles = elapsed.as_nanos() as u64 * estimated_hz / 1_000_000_000;
    
    let ops_per_cycle = total_ops as f64 / estimated_cycles as f64;

    BenchResult {
        ops_per_cycle,
        total_ops,
        total_cycles: estimated_cycles,
        iterations,
        elapsed_secs: elapsed.as_secs_f64(),
        checksum: state.iter().sum(),
    }
}

struct BenchResult {
    ops_per_cycle: f64,
    total_ops: u64,
    total_cycles: u64,
    iterations: u64,
    elapsed_secs: f64,
    checksum: f64,
}

fn main() {
    println!("vTPU CPI Benchmark");
    println!("==================");
    println!("Target: 3.0 ops/cycle");
    println!("Duration: 60 seconds");
    println!();

    let result = benchmark(60);

    println!("---");
    println!("ops_per_cycle:    {:.6}", result.ops_per_cycle);
    println!("total_ops:        {}", result.total_ops);
    println!("total_cycles:     {}", result.total_cycles);
    println!("iterations:       {}", result.iterations);
    println!("elapsed_secs:     {:.3}", result.elapsed_secs);
    println!("checksum:         {:.6}", result.checksum);
    println!();
    
    let target = 3.0;
    let pct = (result.ops_per_cycle / target) * 100.0;
    println!("Target achievement: {:.1}%", pct);
    
    if result.ops_per_cycle >= target {
        println!("✓ TARGET MET!");
    } else {
        println!("Gap to target: {:.3} ops/cycle", target - result.ops_per_cycle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siw_execute() {
        let siw = SIW {
            d_op: 0x041,  // dst=1, src1=1, src2=0
            s_op: 0x082,  // idx=2, key=2
            c_op: 0x043,  // dst=3, src=1
        };
        let mut state = [1.0f64; 64];
        siw.execute(&mut state);
        assert!(state[1] != 1.0); // D-pipe modified
        assert!(state[2] != 1.0); // S-pipe modified
    }

    #[test]
    fn test_generate_workload() {
        let workload = generate_workload(1000);
        assert_eq!(workload.len(), 1000);
    }

    #[test]
    fn test_short_benchmark() {
        let result = benchmark(1);
        assert!(result.total_ops > 0);
        assert!(result.ops_per_cycle > 0.0);
    }
}
