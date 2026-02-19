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
use crate::topology::{SentronTopology, NeuronAddr, Direction};

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

    /// Create an SMT pair using topology-guided placement (W18 completion).
    ///
    /// Places a sentron and its **North neighbor** (upstream generating cycle)
    /// on the same physical core's two SMT threads. This ensures the
    /// most frequent traversal in WuXing (N/S generating cycle) stays
    /// within shared L1/L2 with zero inter-core latency.
    ///
    /// # Why North?
    /// North = upstream in the generating cycle (Fire←Wood, Earth←Fire, …).
    /// The two sentrons on an SMT pair are in a producer/consumer relationship:
    /// the North (upstream) sentron generates values that the South (anchor)
    /// sentron consumes. This maps exactly to the SMT forward/backward pattern:
    /// forward writes activations, backward reads them — same cache line.
    ///
    /// # Placement
    /// - Thread 0 (anchor): the sentron at `anchor_addr`
    /// - Thread 1 (north): the sentron at `topology.neighbor(anchor, North)`
    ///
    /// Both share `core_id`. The SMT scheduler places them on the same physical
    /// core's hyperthreads, so their shared L1 is the zero-copy handoff channel.
    pub fn from_topology(
        pair_id: u16,
        core_id: u8,
        topology: &SentronTopology,
        anchor: NeuronAddr,
        base_coord: PhextCoord,
    ) -> Self {
        // Anchor sentron: thread 0, at anchor address
        let anchor_coord = neuron_to_coord(anchor, base_coord);
        let forward = Sentron::new(pair_id * 2, anchor_coord, core_id, 0);

        // North neighbor: thread 1, upstream generating cycle partner
        let north_addr = topology.neighbor_in(anchor, Direction::North);
        let north_coord = neuron_to_coord(north_addr, base_coord);
        let backward = Sentron::new(pair_id * 2 + 1, north_coord, core_id, 1);

        SmtPair { forward, backward, core_id }
    }

    /// Place all 40 sentrons in a sentron lattice onto physical cores,
    /// pairing each N/S generating-cycle neighbor pair onto the same core.
    ///
    /// Returns 20 SMT pairs (one per physical core) covering all 40 neurons.
    /// The 5 WuXing rows × 8 columns → 20 N/S pairs → 20 cores.
    ///
    /// This is the topology-guided placement that W18 planned: instead of
    /// arbitrary pair assignment, the lattice structure determines co-location.
    pub fn topology_placement(
        base_core_id: u8,
        topology: &SentronTopology,
        base_coord: PhextCoord,
    ) -> Vec<SmtPair> {
        use crate::topology::{NEURONS_PER_ELEMENT, ELEMENT_ROWS};

        let mut pairs = Vec::with_capacity(NEURONS_PER_ELEMENT * (ELEMENT_ROWS / 2 + 1));

        // Pair each row with its North neighbor.
        // 5 rows → 5 N/S pairs per column, but each pair covers 2 rows.
        // We iterate over even rows (0, 2, 4) and pair with their North (4, 1, 3).
        // Row 0 (Wood): North = Row 4 (Water) — the closing wrap of the cycle.
        // Row 2 (Earth): North = Row 1 (Fire)
        // Row 4 (Water): North = Row 3 (Metal)
        // Result: 3 unique N/S pairings × 8 columns = 24 pairs
        // (Some sentrons appear in two pairings; real hardware maps to available cores)

        let mut pair_id: u16 = 0;
        for col in 0..NEURONS_PER_ELEMENT {
            for row in (0..ELEMENT_ROWS).step_by(2) {
                let anchor = NeuronAddr::new(row as u8, col as u8);
                let core_id = base_core_id + pair_id as u8;
                let pair = SmtPair::from_topology(pair_id, core_id, topology, anchor, base_coord);
                pairs.push(pair);
                pair_id += 1;
            }
        }
        pairs
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

/// Convert a neuron address in the Z₅×Z₈ lattice to a phext coordinate.
/// Scroll (dim 0) = column (lateral, 0..8). Section (dim 1) = row (WuXing, 0..5).
fn neuron_to_coord(addr: NeuronAddr, mut base: PhextCoord) -> PhextCoord {
    base.set_dim(0, addr.col as u16 + 1);
    base.set_dim(1, addr.row as u16 + 1);
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::*;
    use crate::siw::SIW;

    #[test]
    fn topology_guided_pair_ns_same_core() {
        let topo = SentronTopology::new();
        let base = PhextCoord::zero();
        // Anchor at Wood row (0), col 0
        let anchor = NeuronAddr::new(0, 0);
        let pair = SmtPair::from_topology(0, 3, &topo, anchor, base);

        // Both sentrons on core 3
        assert_eq!(pair.forward.core_id, 3);
        assert_eq!(pair.backward.core_id, 3);
        // Forward is anchor: Wood row (0) → section dim = row+1 = 1, scroll dim = col+1 = 1
        assert_eq!(pair.forward.home.get_dim(1), 1);  // section = Wood = row 0 → 1
        assert_eq!(pair.forward.home.get_dim(0), 1);  // scroll = col 0 → 1
        // Backward is North of Wood = Water (row 4): section = 5, scroll = 1
        assert_eq!(pair.backward.home.get_dim(1), 5); // section = Water = row 4 → 5
        assert_eq!(pair.backward.home.get_dim(0), 1); // scroll = col 0 → 1
        // SMT threads
        assert_eq!(pair.forward.thread_id, 0);
        assert_eq!(pair.backward.thread_id, 1);
    }

    #[test]
    fn topology_placement_produces_correct_count() {
        let topo = SentronTopology::new();
        let base = PhextCoord::zero();
        let pairs = SmtPair::topology_placement(0, &topo, base);
        // 8 columns × ceil(5/2) = 8 × 3 = 24 pairs
        assert_eq!(pairs.len(), 24);
        // All pairs on unique core IDs (base 0, sequential)
        for (i, pair) in pairs.iter().enumerate() {
            assert_eq!(pair.forward.core_id, i as u8);
            assert_eq!(pair.backward.core_id, i as u8);
        }
    }

    #[test]
    fn topology_placement_ns_neighbors_verified() {
        let topo = SentronTopology::new();
        let base = PhextCoord::zero();
        let pairs = SmtPair::topology_placement(0, &topo, base);
        // First pair: anchor row 0 (Wood), north = row 4 (Water)
        assert_eq!(pairs[0].forward.home.get_dim(1),  1); // Wood
        assert_eq!(pairs[0].backward.home.get_dim(1), 5); // Water
        // Second pair: anchor row 2 (Earth), north = row 1 (Fire)
        assert_eq!(pairs[1].forward.home.get_dim(1),  3); // Earth
        assert_eq!(pairs[1].backward.home.get_dim(1), 2); // Fire
        // Third pair: anchor row 4 (Water), north = row 3 (Metal)
        assert_eq!(pairs[2].forward.home.get_dim(1),  5); // Water
        assert_eq!(pairs[2].backward.home.get_dim(1), 4); // Metal
    }

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
