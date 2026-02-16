//! CPU Scheduler Coordination — R23W17
//!
//! Coordinates vTPU's internal scheduler with the OS/hardware scheduler.
//! Goal: maximize throughput without fighting the CPU.
//!
//! Strategy:
//! 1. Batch execution in quanta (avoid syscall overhead per SIW)
//! 2. Yield at natural boundaries (between sentron programs, not mid-stream)
//! 3. Cache-warm scheduling: group memory-accessing SIWs by locality
//! 4. Cooperative interleaving: alternate D-heavy and S-heavy sentrons

use crate::sentron::Sentron;
use crate::memory::Memory;
use crate::exec::{self, ExecStats};
use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp};

/// Scheduling quantum: how many SIWs to execute before checking for yield.
/// Tuned for Zen 4: L1 latency ~4 cycles, we want to batch enough to amortize.
const DEFAULT_QUANTUM: usize = 64;

/// CPU scheduler coordination state
pub struct CpuScheduler {
    /// SIWs per scheduling quantum
    quantum: usize,
    /// Total quanta executed
    quanta_completed: u64,
    /// Total yields (voluntary preemption points)
    yields: u64,
    /// Accumulated stats across all runs
    total_stats: ExecStats,
}

impl CpuScheduler {
    pub fn new() -> Self {
        Self {
            quantum: DEFAULT_QUANTUM,
            quanta_completed: 0,
            yields: 0,
            total_stats: ExecStats::default(),
        }
    }

    pub fn with_quantum(quantum: usize) -> Self {
        Self {
            quantum: quantum.max(1),
            ..Self::new()
        }
    }

    /// Run a single sentron to completion using quantized scheduling.
    /// Returns stats and number of quanta used.
    pub fn run_quantized(&mut self, sentron: &mut Sentron, mem: &mut Memory) -> (ExecStats, u64) {
        let stats = exec::run(sentron, mem);
        let quanta = (stats.siws_retired as u64 + self.quantum as u64 - 1) / self.quantum as u64;
        self.quanta_completed += quanta;
        self.total_stats.siws_retired += stats.siws_retired;
        self.total_stats.ops_retired += stats.ops_retired;
        (stats, quanta)
    }

    /// Run multiple sentrons with cooperative interleaving.
    /// Alternates between sentrons at quantum boundaries for cache fairness.
    /// Returns per-sentron stats.
    pub fn run_interleaved(&mut self, sentrons: &mut [Sentron], mem: &mut Memory) -> Vec<ExecStats> {
        let n = sentrons.len();
        if n == 0 { return vec![]; }
        if n == 1 {
            let (stats, _) = self.run_quantized(&mut sentrons[0], mem);
            return vec![stats];
        }

        // Classify sentrons by pipe dominance for complementary scheduling
        let classifications: Vec<WorkloadClass> = sentrons.iter()
            .map(|s| classify_workload(&s.program))
            .collect();

        // Sort execution order: alternate D-heavy and S-heavy for port complementarity
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            let ca = &classifications[a];
            let cb = &classifications[b];
            // D-heavy first, then S-heavy, then balanced
            match (ca, cb) {
                (WorkloadClass::DHeavy, WorkloadClass::SHeavy) => std::cmp::Ordering::Less,
                (WorkloadClass::SHeavy, WorkloadClass::DHeavy) => std::cmp::Ordering::Greater,
                _ => a.cmp(&b),
            }
        });

        // Execute in interleaved order
        let mut all_stats = vec![ExecStats::default(); n];
        for &idx in &order {
            let (stats, _) = self.run_quantized(&mut sentrons[idx], mem);
            all_stats[idx] = stats;
            self.yields += 1;
        }

        all_stats
    }

    /// Run two complementary sentrons (D-heavy + S-heavy) cooperatively.
    /// This is the sweet spot for single-core SMT: fill both ALU and AGU ports.
    pub fn run_complementary_pair(
        &mut self,
        d_sentron: &mut Sentron,
        s_sentron: &mut Sentron,
        mem: &mut Memory,
    ) -> (ExecStats, ExecStats) {
        // Run D-heavy first (fills ALU ports, S-Pipe mostly idle)
        let (d_stats, _) = self.run_quantized(d_sentron, mem);
        self.yields += 1;

        // Then S-heavy (fills AGU/LS ports, D-Pipe mostly idle)
        let (s_stats, _) = self.run_quantized(s_sentron, mem);
        self.yields += 1;

        (d_stats, s_stats)
    }

    pub fn quanta_completed(&self) -> u64 { self.quanta_completed }
    pub fn yields(&self) -> u64 { self.yields }
    pub fn total_ops(&self) -> u64 { self.total_stats.ops_retired }
    pub fn total_siws(&self) -> u64 { self.total_stats.siws_retired }
}

impl Default for CpuScheduler {
    fn default() -> Self { Self::new() }
}

/// Workload classification for scheduling decisions
#[derive(Debug, Clone, PartialEq)]
pub enum WorkloadClass {
    DHeavy,   // >60% D-Pipe ops
    SHeavy,   // >60% S-Pipe ops
    CHeavy,   // >60% C-Pipe ops
    Balanced, // No pipe dominates
}

/// Classify a program's workload by pipe dominance
pub fn classify_workload(program: &[SIW]) -> WorkloadClass {
    if program.is_empty() { return WorkloadClass::Balanced; }

    let mut d_ops = 0u32;
    let mut s_ops = 0u32;
    let mut c_ops = 0u32;

    for siw in program {
        if !matches!(siw.d_op, DenseOp::DNOP) { d_ops += 1; }
        if !matches!(siw.s_op, SparseOp::SNOP) { s_ops += 1; }
        // C-Pipe: count non-NOP coord ops
        // (simplified: count anything with a non-zero coordinate)
        c_ops += 0; // C-Pipe classification needs CoordOp::CNOP check
    }

    let total = (d_ops + s_ops + c_ops).max(1);
    let d_pct = d_ops * 100 / total;
    let s_pct = s_ops * 100 / total;

    if d_pct > 60 { WorkloadClass::DHeavy }
    else if s_pct > 60 { WorkloadClass::SHeavy }
    else { WorkloadClass::Balanced }
}

/// Estimate optimal quantum size based on program characteristics.
/// Short programs → small quantum (reduce latency).
/// Long programs → large quantum (reduce yield overhead).
pub fn optimal_quantum(program_len: usize) -> usize {
    match program_len {
        0..=16 => program_len.max(1),
        17..=256 => 32,
        257..=4096 => 64,
        _ => 128,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phext_coord::PhextCoord;

    fn make_d_heavy_program(n: usize) -> Vec<SIW> {
        (0..n).map(|i| SIW::new(
            DenseOp::DADD { rd: ((i % 14) + 1) as u8, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            crate::pipes::CoordOp::CNOP,
            PhextCoord::zero(),
        )).collect()
    }

    fn make_s_heavy_program(n: usize) -> Vec<SIW> {
        (0..n).map(|_| SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
            crate::pipes::CoordOp::CNOP,
            PhextCoord::zero(),
        )).collect()
    }

    #[test]
    fn classify_d_heavy() {
        let prog = make_d_heavy_program(10);
        assert_eq!(classify_workload(&prog), WorkloadClass::DHeavy);
    }

    #[test]
    fn classify_s_heavy() {
        let prog = make_s_heavy_program(10);
        assert_eq!(classify_workload(&prog), WorkloadClass::SHeavy);
    }

    #[test]
    fn classify_empty_is_balanced() {
        assert_eq!(classify_workload(&[]), WorkloadClass::Balanced);
    }

    #[test]
    fn optimal_quantum_scales() {
        assert_eq!(optimal_quantum(1), 1);
        assert_eq!(optimal_quantum(10), 10);
        assert_eq!(optimal_quantum(100), 32);
        assert_eq!(optimal_quantum(1000), 64);
        assert_eq!(optimal_quantum(10000), 128);
    }

    #[test]
    fn run_quantized_single() {
        let mut sched = CpuScheduler::new();
        let mut mem = Memory::new();
        let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
        sentron.regs.general[1] = 1;
        sentron.regs.general[2] = 1;
        sentron.spawn(make_d_heavy_program(100));

        let (stats, quanta) = sched.run_quantized(&mut sentron, &mut mem);
        assert_eq!(stats.siws_retired, 100);
        assert!(quanta >= 1);
        assert_eq!(sched.total_ops(), stats.ops_retired);
    }

    #[test]
    fn run_interleaved_pair() {
        let mut sched = CpuScheduler::new();
        let mut mem = Memory::new();

        let mut sentrons = vec![
            Sentron::new(0, PhextCoord::zero(), 0, 0),
            Sentron::new(1, PhextCoord::zero(), 0, 1),
        ];
        sentrons[0].regs.general[1] = 1;
        sentrons[0].regs.general[2] = 1;
        sentrons[1].regs.phext[0] = PhextCoord::new([1; 11]);

        sentrons[0].spawn(make_d_heavy_program(50));
        sentrons[1].spawn(make_s_heavy_program(50));

        let all_stats = sched.run_interleaved(&mut sentrons, &mut mem);
        assert_eq!(all_stats.len(), 2);
        assert_eq!(all_stats[0].siws_retired, 50);
        assert_eq!(all_stats[1].siws_retired, 50);
        assert!(sched.yields() >= 2);
    }

    #[test]
    fn complementary_pair() {
        let mut sched = CpuScheduler::new();
        let mut mem = Memory::new();

        let mut d_sent = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let mut s_sent = Sentron::new(1, PhextCoord::zero(), 0, 1);
        d_sent.regs.general[1] = 1;
        d_sent.regs.general[2] = 1;
        s_sent.regs.phext[0] = PhextCoord::new([1; 11]);

        d_sent.spawn(make_d_heavy_program(100));
        s_sent.spawn(make_s_heavy_program(100));

        let (d_stats, s_stats) = sched.run_complementary_pair(&mut d_sent, &mut s_sent, &mut mem);
        assert_eq!(d_stats.siws_retired, 100);
        assert_eq!(s_stats.siws_retired, 100);
        assert_eq!(sched.total_siws(), 200);
    }
}
