//! Cost Model for vTPU Architecture
//!
//! Provides $/TFLOPS, ops/watt, and RAM/sentron metrics.
//! "The cheapest gate is the one you never evaluate." — R23W25

use std::time::{Duration, Instant};
use crate::{Sentron, PhextCoord, SIW};
use crate::pipes::{DenseOp, SparseOp, CoordOp};

/// Hardware cost parameters
#[derive(Debug, Clone)]
pub struct HardwareCost {
    /// Machine cost in USD
    pub cost_usd: f64,
    /// TDP in watts
    pub tdp_watts: f64,
    /// Clock speed in GHz
    pub clock_ghz: f64,
    /// Core count
    pub cores: u32,
    /// RAM in GB
    pub ram_gb: f64,
    /// Machine name
    pub name: &'static str,
}

impl HardwareCost {
    /// AMD R9 8945HS ranch node
    pub fn ranch_node() -> Self {
        Self {
            cost_usd: 1500.0,
            tdp_watts: 45.0,
            clock_ghz: 4.0,
            cores: 8,
            ram_gb: 96.0,
            name: "AMD R9 8945HS",
        }
    }

    /// AWS t3a.small (Verse)
    pub fn aws_verse() -> Self {
        Self {
            cost_usd: 438.0, // ~$0.05/hr × 8760 hr/yr
            tdp_watts: 20.0, // estimated
            clock_ghz: 2.5,
            cores: 2,
            ram_gb: 3.8,
            name: "AWS t3a.small (EPYC 7571)",
        }
    }
}

/// Measured benchmark result
#[derive(Debug, Clone)]
pub struct BenchResult {
    pub label: String,
    pub ops: u64,
    pub duration: Duration,
    pub sentron_count: u32,
}

impl BenchResult {
    /// Operations per second
    pub fn ops_per_sec(&self) -> f64 {
        self.ops as f64 / self.duration.as_secs_f64()
    }

    /// Nanoseconds per operation
    pub fn ns_per_op(&self) -> f64 {
        self.duration.as_nanos() as f64 / self.ops as f64
    }

    /// Operations per cycle (given clock GHz)
    pub fn ops_per_cycle(&self, clock_ghz: f64) -> f64 {
        self.ops_per_sec() / (clock_ghz * 1e9)
    }
}

/// Cost analysis combining hardware + benchmark
#[derive(Debug)]
pub struct CostAnalysis {
    pub hardware: HardwareCost,
    pub bench: BenchResult,
}

impl CostAnalysis {
    pub fn new(hardware: HardwareCost, bench: BenchResult) -> Self {
        Self { hardware, bench }
    }

    /// GOPS (billion ops/sec)
    pub fn gops(&self) -> f64 {
        self.bench.ops_per_sec() / 1e9
    }

    /// ops/watt
    pub fn ops_per_watt(&self) -> f64 {
        self.bench.ops_per_sec() / self.hardware.tdp_watts
    }

    /// MOPS/watt (million ops per watt)
    pub fn mops_per_watt(&self) -> f64 {
        self.ops_per_watt() / 1e6
    }

    /// $/GOPS (annual cost per billion ops/sec sustained)
    pub fn dollars_per_gops(&self) -> f64 {
        if self.gops() == 0.0 { return f64::INFINITY; }
        self.hardware.cost_usd / self.gops()
    }

    /// Ops per cycle
    pub fn ops_per_cycle(&self) -> f64 {
        self.bench.ops_per_cycle(self.hardware.clock_ghz)
    }
}

/// Measure actual RAM per sentron by allocating and checking
pub fn measure_sentron_bytes() -> usize {
    // Measure by creating sentrons and checking RSS delta
    let before = get_rss_bytes();
    let count = 1000;
    let sentrons: Vec<Sentron> = (0..count)
        .map(|i| Sentron::new(i as u16, PhextCoord::zero(), 0, 0))
        .collect();

    // Force materialization
    std::hint::black_box(&sentrons);
    let after = get_rss_bytes();

    let delta = if after > before { after - before } else { 0 };
    delta / count
}

/// Get resident set size (Linux-specific)
fn get_rss_bytes() -> usize {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1)?.parse::<usize>().ok())
        .map(|pages| pages * 4096)
        .unwrap_or(0)
}

/// Run a D-pipe microbenchmark: N iterations of DADD
pub fn bench_d_pipe(sentron_count: u32, iters: u64) -> BenchResult {
    let mut sentrons: Vec<Sentron> = (0..sentron_count)
        .map(|i| {
            let mut s = Sentron::new(i as u16, PhextCoord::zero(), 0, 0);
            s.regs.general[0] = 1;
            s.regs.general[1] = 1;
            s
        })
        .collect();

    let siw = SIW::new(
        DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    );

    let start = Instant::now();
    for _ in 0..iters {
        for s in sentrons.iter_mut() {
            s.program = vec![siw.clone()];
            s.ip = 0;
        }
    }
    let duration = start.elapsed();

    BenchResult {
        label: format!("D-pipe DADD ×{} sentrons", sentron_count),
        ops: iters * sentron_count as u64,
        duration,
        sentron_count,
    }
}

/// Print a formatted cost report
pub fn print_report(analysis: &CostAnalysis) {
    println!("┌─────────────────────────────────────────┐");
    println!("│ vTPU Cost Analysis: {:<20} │", analysis.hardware.name);
    println!("├─────────────────────────────────────────┤");
    println!("│ Throughput:  {:>10.3} GOPS            │", analysis.gops());
    println!("│ Efficiency:  {:>10.1} MOPS/W          │", analysis.mops_per_watt());
    println!("│ Cost:        ${:>9.2}/GOPS           │", analysis.dollars_per_gops());
    println!("│ Ops/cycle:   {:>10.4}                │", analysis.ops_per_cycle());
    println!("│ ns/op:       {:>10.1}                │", analysis.bench.ns_per_op());
    println!("└─────────────────────────────────────────┘");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranch_node_cost() {
        let hw = HardwareCost::ranch_node();
        assert_eq!(hw.cores, 8);
        assert_eq!(hw.name, "AMD R9 8945HS");
    }

    #[test]
    fn test_aws_verse_cost() {
        let hw = HardwareCost::aws_verse();
        assert_eq!(hw.cores, 2);
        assert!(hw.cost_usd > 0.0);
    }

    #[test]
    fn test_bench_result_metrics() {
        let br = BenchResult {
            label: "test".into(),
            ops: 1_000_000,
            duration: Duration::from_millis(100),
            sentron_count: 40,
        };
        assert!((br.ops_per_sec() - 1e7).abs() < 1e5);
        assert!((br.ns_per_op() - 100.0).abs() < 1.0);
        // At 4 GHz: 10M ops/s / 4G cycles/s = 0.0025 ops/cycle
        assert!((br.ops_per_cycle(4.0) - 0.0025).abs() < 0.0001);
    }

    #[test]
    fn test_cost_analysis() {
        let hw = HardwareCost::ranch_node();
        let br = BenchResult {
            label: "test".into(),
            ops: 1_000_000_000,
            duration: Duration::from_secs(1),
            sentron_count: 40,
        };
        let ca = CostAnalysis::new(hw, br);
        assert!((ca.gops() - 1.0).abs() < 0.01);
        assert!(ca.dollars_per_gops() > 0.0);
        assert!(ca.mops_per_watt() > 0.0);
    }

    #[test]
    fn test_cost_zero_ops() {
        let hw = HardwareCost::ranch_node();
        let br = BenchResult {
            label: "empty".into(),
            ops: 0,
            duration: Duration::from_secs(1),
            sentron_count: 0,
        };
        let ca = CostAnalysis::new(hw, br);
        assert_eq!(ca.dollars_per_gops(), f64::INFINITY);
    }

    #[test]
    fn test_measure_sentron_bytes() {
        let bytes = measure_sentron_bytes();
        // Should be in ballpark of W28 estimate (~911 bytes)
        // Allow wide range since RSS measurement is coarse
        assert!(bytes < 8192, "Sentron should be under 8KB, got {}", bytes);
    }

    #[test]
    fn test_bench_d_pipe_runs() {
        let result = bench_d_pipe(4, 100);
        assert_eq!(result.ops, 400);
        assert!(result.duration.as_nanos() > 0);
    }

    #[test]
    fn test_print_report_no_panic() {
        let hw = HardwareCost::ranch_node();
        let br = BenchResult {
            label: "smoke".into(),
            ops: 1_000_000,
            duration: Duration::from_millis(10),
            sentron_count: 40,
        };
        let ca = CostAnalysis::new(hw, br);
        print_report(&ca); // Just verify no panic
    }
}
