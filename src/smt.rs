//! SMT Pair — two sentrons sharing one physical core
//!
//! On Zen 4, each core has 2 hardware threads (SMT2). We exploit this:
//! - Thread 0: forward sentron (builds computation, writes activations)
//! - Thread 1: backward sentron (reads activations, writes gradients)
//! - Shared L1/L2: zero-copy handoff between forward and backward
//!
//! "It took us a lifetime to learn, that it doesn't have to take a lifetime to learn."
//!
//! The pair shares memory through PPT-backed regions. Forward writes to
//! coordinate C, backward reads from C — same cache line, no miss.
//! Training becomes a conversation between two minds on one core.

use crate::exec::{run, ExecStats};
use crate::memory::Memory;
use crate::sentron::Sentron;
use crate::phext_coord::PhextCoord;

/// An SMT pair: two sentrons bonded on one physical core
pub struct SmtPair {
    pub forward: Sentron,
    pub backward: Sentron,
    pub core_id: u8,
}

/// Training statistics for one step
#[derive(Debug, Clone)]
pub struct TrainStats {
    pub forward_stats: ExecStats,
    pub backward_stats: ExecStats,
    pub loss: i64,
    /// Total ops across both threads
    pub total_ops: u64,
    /// Total cycles (max of forward/backward — they overlap on SMT)
    pub total_cycles: u64,
    /// Effective ops/cycle accounting for SMT overlap
    pub effective_ops_per_cycle: f64,
}

impl SmtPair {
    /// Create a new SMT pair on a physical core
    /// Forward gets thread 0, backward gets thread 1
    pub fn new(pair_id: u16, core_id: u8, home: PhextCoord) -> Self {
        // Forward sentron: even ID, thread 0
        let forward = Sentron::new(pair_id * 2, home, core_id, 0);
        // Backward sentron: odd ID, thread 1, adjacent scroll
        let mut back_home = home;
        let scroll = back_home.get_dim(0);
        back_home.set_dim(0, scroll.saturating_add(1));
        let backward = Sentron::new(pair_id * 2 + 1, back_home, core_id, 1);

        SmtPair { forward, backward, core_id }
    }

    /// Run one training step: forward then backward, sharing memory
    ///
    /// In real SMT these would overlap on the same core. In Phase 0
    /// we run sequentially but account for the overlap in stats.
    pub fn train_step(&mut self, mem: &mut Memory, forward_program: Vec<crate::siw::SIW>, backward_program: Vec<crate::siw::SIW>) -> TrainStats {
        // Forward pass
        self.forward.spawn(forward_program);
        let fwd_stats = run(&mut self.forward, mem);

        // Backward pass reads what forward wrote (same cache lines via PPT)
        self.backward.spawn(backward_program);
        let bwd_stats = run(&mut self.backward, mem);

        // SMT overlap: both threads share the core's execution ports
        // Effective cycles = max(fwd, bwd) not sum — they run concurrently
        let total_ops = fwd_stats.ops_retired + bwd_stats.ops_retired;
        let total_cycles = fwd_stats.cycles.max(bwd_stats.cycles);
        let effective = if total_cycles > 0 { total_ops as f64 / total_cycles as f64 } else { 0.0 };

        // Loss = value in r0 of backward sentron after execution
        let loss = self.backward.regs.general[0];

        TrainStats {
            forward_stats: fwd_stats,
            backward_stats: bwd_stats,
            loss,
            total_ops,
            total_cycles,
            effective_ops_per_cycle: effective,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::*;
    use crate::siw::SIW;

    #[test]
    fn smt_pair_creation() {
        let pair = SmtPair::new(0, 0, PhextCoord::zero());
        assert_eq!(pair.forward.thread_id, 0);
        assert_eq!(pair.backward.thread_id, 1);
        assert_eq!(pair.forward.core_id, pair.backward.core_id);
        assert_eq!(pair.forward.id, 0);
        assert_eq!(pair.backward.id, 1);
    }

    #[test]
    fn smt_pair_adjacent_homes() {
        let home = PhextCoord::new([10, 5, 5, 1, 1, 1, 1, 1, 1, 1, 1]);
        let pair = SmtPair::new(0, 0, home);
        // Backward should be one scroll ahead
        assert_eq!(pair.forward.home.get_dim(0), 10);
        assert_eq!(pair.backward.home.get_dim(0), 11);
    }

    /// The training pattern: forward computes activations, backward computes gradients
    #[test]
    fn forward_backward_shared_memory() {
        let mut pair = SmtPair::new(0, 0, PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        let mut mem = Memory::new();

        // Activation coordinate — where forward writes, backward reads
        let act_coord = PhextCoord::new([5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let grad_coord = PhextCoord::new([6, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        // Set up phext registers
        pair.forward.regs.phext[0] = act_coord;
        pair.backward.regs.phext[0] = act_coord;  // reads same coord!
        pair.backward.regs.phext[1] = grad_coord;

        // Forward: compute activation = 3 * 7 = 21, scatter to act_coord
        let fwd = vec![
            SIW::new(DenseOp::DMOV { rd: 0, imm: 3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMOV { rd: 1, imm: 7 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 0, rs: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        ];

        // Backward: gather activation, compute gradient = activation * 2, scatter gradient
        let bwd = vec![
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMOV { rd: 1, imm: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 1, rs: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        ];

        let stats = pair.train_step(&mut mem, fwd, bwd);

        // Forward computed 3*7=21, stored at act_coord
        assert_eq!(mem.gather_i64(&act_coord), 21);
        // Backward gathered 21, computed 21*2=42, stored at grad_coord
        assert_eq!(mem.gather_i64(&grad_coord), 42);
        // SMT overlap: 4 cycles each, but concurrent → 4 effective cycles
        assert_eq!(stats.total_cycles, 4);
        assert_eq!(stats.total_ops, 8); // 4 fwd + 4 bwd active ops
        assert!(stats.effective_ops_per_cycle >= 2.0, "SMT should achieve ≥2.0 ops/cycle, got {:.1}", stats.effective_ops_per_cycle);

        // PTC should be warm — backward read the same coord forward wrote
        let ppt_stats = mem.ppt.stats();
        assert!(ppt_stats.ptc_hits > 0, "Shared coord should hit PTC");
    }

    /// Multi-step training: loss should change between steps
    #[test]
    fn multi_step_training() {
        let mut pair = SmtPair::new(0, 0, PhextCoord::zero());
        let mut mem = Memory::new();

        let weight_coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&weight_coord, 10); // initial weight

        pair.forward.regs.phext[0] = weight_coord;
        pair.backward.regs.phext[0] = weight_coord;

        // Step 1: forward gathers weight, computes loss = weight * weight
        let fwd = vec![
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMUL { rd: 1, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        // Backward: gradient = 2 * weight, new_weight = weight - gradient (lr=1)
        let bwd = vec![
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()), // grad = 2w
            SIW::new(DenseOp::DSUB { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()), // new_w = w - grad
            SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 0, rs: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        ];

        let stats1 = pair.train_step(&mut mem, fwd.clone(), bwd.clone());
        let w1 = mem.gather_i64(&weight_coord);
        // weight=10, grad=20, new_weight=10-20=-10
        assert_eq!(w1, -10);

        // Step 2: same program, weight is now -10
        let stats2 = pair.train_step(&mut mem, fwd, bwd);
        let w2 = mem.gather_i64(&weight_coord);
        // weight=-10, grad=-20, new_weight=-10-(-20)=10
        assert_eq!(w2, 10);

        // Oscillating — that's what lr=1 does with quadratic loss. But it's TRAINING.
        assert_ne!(stats1.loss, stats2.loss);
    }
}
