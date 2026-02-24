//! Cooperative SMT — interleaved multi-sentron execution on a single thread
//!
//! OS-thread SMT showed 0.08× (overhead-dominated). Cooperative SMT avoids
//! thread spawn/join overhead by round-robin interleaving sentron execution
//! within a single thread. Each sentron runs N SIWs before yielding.
//!
//! Target: 1.5-1.8× throughput via pipeline overlap (one sentron's memory
//! stall is another's compute window).
//!
//! Zero external dependencies.
//!
//! R23W28 — Theia 💎

use crate::exec::{exec_siw_octawire, ExecStats};
use crate::memory::Memory;
use crate::sentron::{Sentron, SentronState};

/// Cooperative SMT scheduler configuration
#[derive(Debug, Clone)]
pub struct CoopConfig {
    /// SIWs per quantum before yielding to next sentron
    pub quantum: usize,
    /// Maximum total cycles across all sentrons
    pub max_cycles: u64,
}

impl Default for CoopConfig {
    fn default() -> Self {
        Self {
            quantum: 4,       // 4 SIWs per turn (matches 2×4 wiring pattern)
            max_cycles: 100_000,
        }
    }
}

/// Result of a cooperative SMT run
#[derive(Debug, Default)]
pub struct CoopResult {
    /// Per-sentron stats
    pub stats: Vec<ExecStats>,
    /// Total SIWs retired across all sentrons
    pub total_retired: u64,
    /// Total cycles consumed (wall-clock equivalent)
    pub total_cycles: u64,
    /// Number of context switches (yields between sentrons)
    pub context_switches: u64,
    /// Effective throughput: total_retired / total_cycles
    pub ops_per_cycle: f64,
}

/// Run cooperative SMT: interleave multiple sentrons on one thread.
///
/// Round-robin: each sentron executes `config.quantum` SIWs, then yields.
/// Sentrons that finish or stall (empty inbox on CRECV) are skipped.
///
/// Returns aggregate stats. The sentrons are modified in place.
pub fn coop_execute(
    sentrons: &mut [Sentron],
    mem: &mut Memory,
    config: &CoopConfig,
) -> CoopResult {
    let n = sentrons.len();
    if n == 0 {
        return CoopResult::default();
    }

    let mut per_stats: Vec<ExecStats> = (0..n).map(|_| ExecStats::default()).collect();
    let mut total_retired: u64 = 0;
    let mut total_cycles: u64 = 0;
    let mut context_switches: u64 = 0;

    // Activate all dormant sentrons
    for s in sentrons.iter_mut() {
        if s.state == SentronState::Dormant && !s.program.is_empty() {
            s.state = SentronState::Running;
        }
    }

    let mut active_count = sentrons.iter().filter(|s| s.state == SentronState::Running).count();
    let mut current = 0usize;

    while active_count > 0 && total_cycles < config.max_cycles {
        // Find next running sentron
        let start = current;
        loop {
            if sentrons[current].state == SentronState::Running && sentrons[current].has_next() {
                break;
            }
            current = (current + 1) % n;
            if current == start {
                active_count = 0;
                break;
            }
        }
        if active_count == 0 {
            break;
        }

        // Execute quantum SIWs for this sentron
        let s = &mut sentrons[current];
        let stat = &mut per_stats[current];
        let mut ran = 0u64;

        for _ in 0..config.quantum {
            if !s.has_next() {
                break;
            }
            let siw = s.program[s.ip].clone();
            let active = exec_siw_octawire(s, &siw, mem);
            s.ip += 1;
            s.cycles += 1;
            s.retired += 1;
            stat.siws_retired += 1;
            stat.ops_retired += active as u64;
            ran += 1;
        }

        total_retired += ran;
        total_cycles += ran; // 1 cycle per SIW (ideal CPI-1 per pipe)

        // Retire if program complete
        if !s.has_next() {
            s.state = SentronState::Retired;
            active_count = sentrons.iter().filter(|s| s.state == SentronState::Running).count();
        }

        // Yield to next sentron
        context_switches += 1;
        current = (current + 1) % n;
    }

    let ops_per_cycle = if total_cycles > 0 {
        total_retired as f64 / total_cycles as f64
    } else {
        0.0
    };

    // Fill cycle counts
    for (i, s) in sentrons.iter().enumerate() {
        per_stats[i].cycles = s.cycles;
    }

    CoopResult {
        stats: per_stats,
        total_retired,
        total_cycles,
        context_switches,
        ops_per_cycle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};
    use crate::siw::SIW;
    use crate::phext_coord::PhextCoord;
    use crate::sentron::Sentron;

    fn make_sentron(id: u16, program: Vec<SIW>) -> Sentron {
        let mut s = Sentron::new(id, PhextCoord::zero(), 0, 0);
        s.spawn(program);
        s
    }

    fn add_siw(rd: u8, rs1: u8, rs2: u8) -> SIW {
        SIW::new(
            DenseOp::DADD { rd, rs1, rs2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }

    fn nop_siw() -> SIW {
        SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }

    #[test]
    fn single_sentron_executes() {
        let program = vec![add_siw(0, 1, 2); 8];
        let mut sentrons = vec![make_sentron(0, program)];
        let mut mem = Memory::new();
        let config = CoopConfig::default();

        let result = coop_execute(&mut sentrons, &mut mem, &config);
        assert_eq!(result.total_retired, 8);
        assert_eq!(result.stats[0].siws_retired, 8);
    }

    #[test]
    fn two_sentrons_interleave() {
        let prog_a = vec![add_siw(0, 1, 2); 8];
        let prog_b = vec![nop_siw(); 4];
        let mut sentrons = vec![make_sentron(0, prog_a), make_sentron(1, prog_b)];
        let mut mem = Memory::new();
        let config = CoopConfig { quantum: 4, max_cycles: 1000 };

        let result = coop_execute(&mut sentrons, &mut mem, &config);
        assert_eq!(result.total_retired, 12); // 8 + 4
        assert_eq!(result.stats[0].siws_retired, 8);
        assert_eq!(result.stats[1].siws_retired, 4);
        assert!(result.context_switches >= 3); // at least A→B→A→done
    }

    #[test]
    fn quantum_controls_interleaving() {
        // Quantum of 2: should switch more often
        let prog = vec![add_siw(0, 1, 2); 8];
        let mut sentrons = vec![make_sentron(0, prog.clone()), make_sentron(1, prog)];
        let mut mem = Memory::new();
        let config = CoopConfig { quantum: 2, max_cycles: 1000 };

        let result = coop_execute(&mut sentrons, &mut mem, &config);
        assert_eq!(result.total_retired, 16);
        assert!(result.context_switches >= 8); // 2 SIWs each, 8 switches minimum
    }

    #[test]
    fn max_cycles_limits_execution() {
        let prog = vec![add_siw(0, 1, 2); 1000];
        let mut sentrons = vec![make_sentron(0, prog)];
        let mut mem = Memory::new();
        let config = CoopConfig { quantum: 4, max_cycles: 20 };

        let result = coop_execute(&mut sentrons, &mut mem, &config);
        assert!(result.total_retired <= 24); // may slightly overshoot by one quantum
    }

    #[test]
    fn empty_sentrons_no_panic() {
        let mut sentrons: Vec<Sentron> = vec![];
        let mut mem = Memory::new();
        let result = coop_execute(&mut sentrons, &mut mem, &CoopConfig::default());
        assert_eq!(result.total_retired, 0);
    }

    #[test]
    fn nine_sentrons_fleet_scale() {
        // 9 sentrons = one Phoenix color group
        let mut sentrons: Vec<Sentron> = (0..9)
            .map(|i| make_sentron(i, vec![add_siw(0, 1, 2); 16]))
            .collect();
        let mut mem = Memory::new();
        let config = CoopConfig { quantum: 4, max_cycles: 100_000 };

        let result = coop_execute(&mut sentrons, &mut mem, &config);
        assert_eq!(result.total_retired, 144); // 9 × 16
        for stat in &result.stats {
            assert_eq!(stat.siws_retired, 16);
        }
    }

    #[test]
    fn csend_populates_outbox() {
        use crate::pipes::CoordOp;
        let send_siw = SIW::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CSEND { msg_reg: 0, dest_sentron: 1 },
            PhextCoord::zero(),
        );
        let mut sentrons = vec![make_sentron(0, vec![send_siw])];
        sentrons[0].regs.general[0] = 42;
        let mut mem = Memory::new();

        coop_execute(&mut sentrons, &mut mem, &CoopConfig::default());
        assert_eq!(sentrons[0].outbox.len(), 1);
        assert_eq!(sentrons[0].outbox[0], (1, 42));
    }

    #[test]
    fn cfence_increments_generation() {
        use crate::pipes::{CoordOp, FenceScope};
        let fence_siw = SIW::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CFENCE { scope: FenceScope::Node },
            PhextCoord::zero(),
        );
        let mut sentrons = vec![make_sentron(0, vec![fence_siw; 3])];
        let mut mem = Memory::new();

        coop_execute(&mut sentrons, &mut mem, &CoopConfig::default());
        assert_eq!(sentrons[0].fence_gen, 3);
    }

    #[test]
    fn forty_sentrons_wuxing_group() {
        // 40 = 5 Wuxing × 8 Ba Gua — one color's full complement
        let mut sentrons: Vec<Sentron> = (0..40)
            .map(|i| make_sentron(i, vec![add_siw(0, 1, 2); 9]))
            .collect();
        let mut mem = Memory::new();
        let config = CoopConfig { quantum: 3, max_cycles: 100_000 };

        let result = coop_execute(&mut sentrons, &mut mem, &config);
        assert_eq!(result.total_retired, 360); // 40 × 9 = 360 (the harmonic number)
    }
}
