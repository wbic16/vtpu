//! W23 — 8-Sentron Cooperative Messaging Model
//!
//! Runs 8 virtual sentrons on **one OS thread**. This is not an 8-core or SMT
//! benchmark. The coordinated workload uses coop_fleet_execute, which drains
//! CSEND outboxes after each quantum; coop_execute alone does not route them.
//! Wall-clock timing is measured, while 5 GHz cycle equivalents are modeled.
//!
//! CBAR is currently a non-blocking marker; it does not synchronize pairs.
//! No L1/L2 affinity, physical hardware IPC, or cache-crosstalk claims are
//! established by this model.

use vtpu_runtime::coop_smt::{coop_execute, CoopConfig};
use vtpu_runtime::coop_fleet::{coop_fleet_execute, CoopFleetResult};
use vtpu_runtime::fleet::Fleet;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp, PrefetchHint, MessageFormat};
use vtpu_runtime::siw::SIW;

use std::time::Instant;

const N_SENTRONS: usize = 8;
const PROGRAM_LEN: usize = 256;
const QUANTUM: usize = 8;        // 8 SIWs per turn — 2×4 wiring pattern
const ITERATIONS: u32 = 50000;
const MAX_CYCLES: u64 = 10_000_000;
const GHZ: f64 = 5.0e9;

/// Build a synthetic 3-pipe program with C-pipe messages and barrier markers.
/// These are virtual sentron IDs, not measured SMT siblings.
fn make_coordinated_program(partner_id: u16, len: usize) -> Vec<SIW> {
    (0..len).map(|i| {
        let rd = (i % 12 + 1) as u8;
        let rs1 = (i % 6 + 1) as u8;
        let rs2 = (i % 5 + 2) as u8;
        let rs3 = (i % 4 + 3) as u8;

        // D-pipe: synthetic FMA workload
        let d_op = DenseOp::DFMA { rd, rs1, rs2, rs3 };

        // S-pipe: prefetch next coordinate (temporal exploitation)
        let s_op = if i % 4 == 0 {
            SparseOp::SPREFCH {
                coord_idx: (i % 8) as u8,
                hint: PrefetchHint::L1,
            }
        } else {
            SparseOp::SNOP
        };

        // C-pipe: alternate CSEND/CRECV with partner (virtual communication).
        let c_op = if i % 8 < 4 {
            CoordOp::CSEND {
                msg_reg: rs1,
                dest_sentron: partner_id as u8,
            }
        } else if i % 8 == 4 {
            CoordOp::CRECV {
                rd: rd + 1,
                src_sentron: partner_id as u8,
            }
        } else if i % 8 == 7 {
            // CBAR is a marker only; this executor does not block for partners.
            CoordOp::CBAR {
                barrier_id: (i / 8 % 256) as u8,
                count: 2, // just the SMT pair synchronizes (not all 8)
            }
        } else {
            CoordOp::CPACK {
                rd: 0, rs1, rs2,
                fmt: MessageFormat::Result,
            }
        };

        SIW::new(d_op, s_op, c_op, PhextCoord::zero())
    }).collect()
}

/// Build a packed-only program (no coordination) for baseline comparison
fn make_packed_baseline(len: usize) -> Vec<SIW> {
    (0..len).map(|i| {
        let rd = (i % 12 + 1) as u8;
        let rs1 = (i % 6 + 1) as u8;
        let rs2 = (i % 5 + 2) as u8;
        let rs3 = (i % 4 + 3) as u8;
        SIW::new(
            DenseOp::DFMA { rd, rs1, rs2, rs3 },
            SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L2 },
            CoordOp::CPACK { rd: 0, rs1, rs2, fmt: MessageFormat::Result },
            PhextCoord::zero(),
        )
    }).collect()
}

/// Explicitly select the messaging-capable cooperative execution mode.
fn execute_coordinated(fleet: &mut Fleet, mem: &mut Memory, config: &CoopConfig) -> CoopFleetResult {
    coop_fleet_execute(fleet, mem, config.quantum, config.max_cycles)
}

fn run_coordinated_8_sentrons() -> (f64, CoopFleetResult, f64) {
    let mut mem = Memory::new();
    // Eight virtual sentrons, paired: (0,4), (1,5), (2,6), (3,7).
    let mut fleet = Fleet::new(N_SENTRONS);
    let programs: Vec<Vec<SIW>> = (0..N_SENTRONS).map(|i| {
        let partner = if i < 4 { i + 4 } else { i - 4 };
        make_coordinated_program(partner as u16, PROGRAM_LEN)
    }).collect();

    let config = CoopConfig {
        quantum: QUANTUM,
        max_cycles: MAX_CYCLES,
    };

    let t0 = Instant::now();
    let mut agg_result = CoopFleetResult::default();
    for _ in 0..ITERATIONS {
        // Prevent stale messages from crossing iteration boundaries. spawn()
        // resets instruction state but intentionally does not reset inboxes.
        for (i, program) in programs.iter().enumerate() {
            let s = fleet.sentron_mut(i as u16).expect("sentron exists");
            s.inbox.clear();
            s.outbox.clear();
            s.home = PhextCoord::new([i as u16 + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            s.spawn(program.clone());
        }
        let result = execute_coordinated(&mut fleet, &mut mem, &config);
        agg_result.total_retired += result.total_retired;
        agg_result.total_cycles += result.total_cycles;
        agg_result.context_switches += result.context_switches;
        agg_result.messages_delivered += result.messages_delivered;
        agg_result.flushes += result.flushes;
    }
    let wall_secs = t0.elapsed().as_secs_f64();
    let wall_cycles = wall_secs * GHZ;
    let opc = agg_result.total_retired as f64 / wall_cycles;

    (opc, agg_result, wall_secs)
}

fn run_baseline_single() -> (f64, u64, f64) {
    let mut mem = Memory::new();
    let program = make_packed_baseline(PROGRAM_LEN);

    let t0 = Instant::now();
    let mut total_retired = 0u64;
    for _ in 0..ITERATIONS {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        s.spawn(program.clone());
        let result = coop_execute(
            std::slice::from_mut(&mut s), &mut mem,
            &CoopConfig { quantum: PROGRAM_LEN, max_cycles: MAX_CYCLES }
        );
        total_retired += result.total_retired;
    }
    let wall_secs = t0.elapsed().as_secs_f64();
    let wall_cycles = wall_secs * GHZ;
    let opc = total_retired as f64 / wall_cycles;
    (opc, total_retired, wall_secs)
}

fn main() {
    println!("\nvTPU W23 — Single-Thread Cooperative Messaging Model");
    println!("=============================================================");
    println!("Sentrons : {} virtual (4 modeled pairs; 1 OS thread)", N_SENTRONS);
    println!("Program  : {} SIWs (D-pipe FMA + S-pipe prefetch + C-pipe msg)", PROGRAM_LEN);
    println!("Quantum  : {} SIWs/turn (fleet flush after each turn)", QUANTUM);
    println!("Iterations: {}", ITERATIONS);
    println!("Clock    : {} GHz (assumed, not measured CPU cycles)", GHZ / 1e9);
    println!();

    // Baseline: single sentron, packed SIWs, no coordination
    println!("── Baseline (1 sentron, no coordination) ──");
    let (base_opc, base_ops, base_secs) = run_baseline_single();
    println!("  SIWs/assumed cycle : {:.3}", base_opc);
    println!("  SIWs retired       : {}", base_ops);
    println!("  wall time          : {:.3}s", base_secs);
    println!();

    // Coordinated: eight virtual sentrons with explicit message delivery.
    println!("── Coordinated (8 virtual sentrons, C-pipe fleet routing) ──");
    let (coord_opc, coord_result, coord_secs) = run_coordinated_8_sentrons();
    println!("  SIWs/assumed cycle : {:.3}", coord_opc);
    println!("  SIWs retired       : {}", coord_result.total_retired);
    println!("  messages delivered : {}", coord_result.messages_delivered);
    println!("  fleet flushes      : {}", coord_result.flushes);
    println!("  context switches   : {}", coord_result.context_switches);
    println!("  wall time          : {:.3}s", coord_secs);
    println!();

    let base_rate = base_ops as f64 / base_secs;
    let coord_rate = coord_result.total_retired as f64 / coord_secs;

    println!("── Measured wall-clock rates (different workloads) ──");
    println!("  Baseline SIWs/s    : {:.0}", base_rate);
    println!("  Coordinated SIWs/s : {:.0}", coord_rate);
    println!("  Rate ratio         : {:.2}× (NOT a physical-core speedup)", coord_rate / base_rate);
    println!();

    println!("── Model limitations ──");
    println!("  One OS thread; no actual 8-core scheduling or SMT affinity");
    println!("  CSEND/CRECV routed through Fleet after each quantum");
    println!("  CBAR(count=2) is a marker, NOT a synchronization barrier");
    println!("  S-pipe prefetch hints do not prove L1 residency or crosstalk");
    println!();

    println!("Historical 2.5 SIW/nominal-cycle goal is UNVERIFIED here.");
    println!("Physical IPC and hardware scaling require separate RG-4 runs.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinated_w23_path_delivers_messages_between_virtual_sentrons() {
        let send = SIW::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CSEND { msg_reg: 0, dest_sentron: 1 },
            PhextCoord::zero(),
        );
        let recv = SIW::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CRECV { rd: 2, src_sentron: 0 },
            PhextCoord::zero(),
        );
        // Fleet's existing default wiring expects at least four members.
        let mut fleet = Fleet::new(N_SENTRONS);
        fleet.sentron_mut(0).unwrap().regs.general[0] = 42;
        fleet.sentron_mut(0).unwrap().spawn(vec![send]);
        fleet.sentron_mut(1).unwrap().spawn(vec![recv]);

        let mut mem = Memory::with_capacity(8);
        let report = execute_coordinated(
            &mut fleet,
            &mut mem,
            &CoopConfig { quantum: 1, max_cycles: 8 },
        );
        assert_eq!(report.total_retired, 2);
        assert_eq!(report.messages_delivered, 1);
        assert_eq!(fleet.sentron(1).unwrap().regs.general[2], 42);
        assert!(fleet.sentron(0).unwrap().outbox.is_empty());
    }
}
