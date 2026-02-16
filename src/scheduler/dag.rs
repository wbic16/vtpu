//! vTPU Micro-Scheduler (R23W9)
//!
//! Uses dependency graph + register allocation to pack SIW streams
//! for maximum throughput on Zen 4 execution ports.
//!
//! Scheduling strategy:
//! 1. Build dependency DAG from hazard analysis
//! 2. Topological sort by critical path (longest first)
//! 3. Insert NOPs only when true data hazards can't be hidden
//! 4. Report achieved ILP and pipe utilization

use crate::siw::SIW;

use crate::regalloc::{self, StreamAnalysis, Hazard, build_dep_graph, topological_sort};
use crate::telemetry::VtpuTelemetry;
#[cfg(test)]
use crate::pipes::{DenseOp, SparseOp, CoordOp};
#[cfg(test)]
use crate::PhextCoord;

/// vTPU Scheduler — dispatches SIWs to execution pipes
pub struct Scheduler {
    telemetry: VtpuTelemetry,
}

/// Result of scheduling a stream
#[derive(Debug, Clone)]
pub struct ScheduleResult {
    /// Reordered (and possibly NOP-padded) SIW stream
    pub stream: Vec<SIW>,
    /// Analysis before scheduling
    pub before: StreamAnalysis,
    /// Analysis after scheduling
    pub after: StreamAnalysis,
    /// NOPs inserted to break hazards
    pub nops_inserted: usize,
    /// SIWs reordered (moved from original position)
    pub reordered: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            telemetry: VtpuTelemetry::new(),
        }
    }

    /// Schedule a SIW stream for optimal execution
    pub fn schedule(&mut self, stream: &[SIW]) -> ScheduleResult {
        let before = regalloc::analyze_stream(stream);

        if stream.len() <= 1 {
            return ScheduleResult {
                stream: stream.to_vec(),
                before: before.clone(),
                after: before,
                nops_inserted: 0,
                reordered: 0,
            };
        }

        // Step 1: Reorder by dependency graph
        let graph = build_dep_graph(stream);
        let order = topological_sort(&graph);
        let reordered_count = order.iter().enumerate()
            .filter(|(i, &v)| *i != v)
            .count();

        let mut scheduled: Vec<SIW> = order.iter().map(|&i| stream[i].clone()).collect();

        // Step 2: Insert NOPs for back-to-back RAW hazards on adjacent SIWs
        // that can't be hidden by reordering
        let mut nops_inserted = 0;
        let mut i = 0;
        while i + 1 < scheduled.len() {
            let pair = &scheduled[i..i + 2];
            let hazards = regalloc::detect_hazards(pair);
            let has_adjacent_raw = hazards.iter().any(|h| {
                matches!(h, Hazard::RAW { producer: 0, consumer: 1, .. })
            });

            if has_adjacent_raw {
                // Insert a NOP between them to give the pipeline time
                scheduled.insert(i + 1, SIW::nop());
                nops_inserted += 1;
                i += 2; // skip past the NOP
            } else {
                i += 1;
            }
        }

        // Record telemetry
        self.telemetry.record_ops((scheduled.len() * 3) as u64);
        self.telemetry.record_cycles(scheduled.len() as u64);

        let after = regalloc::analyze_stream(&scheduled);

        ScheduleResult {
            stream: scheduled,
            before,
            after,
            nops_inserted,
            reordered: reordered_count,
        }
    }

    /// Execute a single SIW (legacy compat)
    pub fn execute_siw(&mut self, _siw: &SIW) {
        self.telemetry.record_ops(3);
        self.telemetry.record_cycles(1);
    }

    /// Execute a stream of SIWs (legacy compat)
    pub fn execute_stream(&mut self, siws: &[SIW]) {
        for siw in siws {
            self.execute_siw(siw);
        }
    }

    /// Get telemetry
    pub fn telemetry(&self) -> &VtpuTelemetry {
        &self.telemetry
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::*;

    fn mov(rd: u8, imm: i64) -> SIW {
        SIW::new(DenseOp::DMOV { rd, imm }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }
    fn add(rd: u8, rs1: u8, rs2: u8) -> SIW {
        SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }

    #[test]
    fn test_scheduler_creation() {
        let s = Scheduler::new();
        assert_eq!(s.telemetry().ops_per_cycle(), 0.0);
    }

    #[test]
    fn test_execute_nop() {
        let mut s = Scheduler::new();
        s.execute_siw(&SIW::nop());
        assert_eq!(s.telemetry().ops_per_cycle(), 3.0);
    }

    #[test]
    fn schedule_independent_stream() {
        let mut sched = Scheduler::new();
        let stream = vec![mov(0, 1), mov(1, 2), mov(2, 3)];
        let result = sched.schedule(&stream);
        assert_eq!(result.nops_inserted, 0);
        assert_eq!(result.stream.len(), 3);
    }

    #[test]
    fn schedule_dependent_chain() {
        let mut sched = Scheduler::new();
        // r0=1 → r1=r0+r0 → r2=r1+r1 (chain, each depends on prior)
        let stream = vec![mov(0, 1), add(1, 0, 0), add(2, 1, 1)];
        let result = sched.schedule(&stream);
        // Should insert NOPs between adjacent RAW hazards
        assert!(result.nops_inserted > 0, "Chain needs NOP padding");
        assert!(result.stream.len() > 3);
    }

    #[test]
    fn schedule_parallel_chains() {
        let mut sched = Scheduler::new();
        // Two independent chains interleaved badly:
        // r0=1, r1=r0+r0, r2=2, r3=r2+r2
        let stream = vec![mov(0, 1), add(1, 0, 0), mov(2, 2), add(3, 2, 2)];
        let result = sched.schedule(&stream);
        // Scheduler should interleave: mov0, mov2, add1, add3 (no NOPs needed)
        assert!(result.before.raw_hazards > 0);
        // After reorder, adjacent RAW hazards should be reduced
    }

    #[test]
    fn schedule_preserves_correctness() {
        let mut sched = Scheduler::new();
        let stream = vec![mov(0, 42), add(1, 0, 0), add(2, 1, 1)];
        let result = sched.schedule(&stream);
        // All original SIWs must be present (plus NOPs)
        let non_nop = result.stream.iter()
            .filter(|s| !matches!(s.d_op, DenseOp::DNOP) || !matches!(s.s_op, SparseOp::SNOP) || !matches!(s.c_op, CoordOp::CNOP))
            .count();
        assert_eq!(non_nop, 3);
    }

    #[test]
    fn schedule_result_has_analysis() {
        let mut sched = Scheduler::new();
        let stream = vec![mov(0, 1), mov(1, 2), add(2, 0, 1)];
        let result = sched.schedule(&stream);
        assert_eq!(result.before.siw_count, 3);
        assert!(result.before.ilp > 0.0);
        assert!(!result.before.needs_spill);
    }
}
