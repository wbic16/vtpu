//! W23 — 8-Core Coordination: Cache Crosstalk Exploitation
//!
//! Uses real vTPU execution (coop_execute) with 8 sentrons.
//! Measures ops/cycle under coordinated execution with C-Pipe messaging.
//!
//! Cache exploitation model:
//!   Physical: SMT siblings share L1/L2. Packed SIWs with CSEND/CRECV
//!             land in shared cache — coordination is nearly free.
//!   Temporal: CBAR barriers timed so warmed data is still L1-hot.
//!
//! Gate: ≥2.5 ops/cycle under 8-sentron coordinated execution.

use vtpu_runtime::coop_smt::{coop_execute, CoopConfig, CoopResult};
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

/// Build a program that exploits all three pipes + C-Pipe coordination.
/// Each SIW has D-pipe compute, S-pipe prefetch, and C-pipe messaging.
/// Adjacent sentrons send/recv — exploits SMT sibling L1/L2 sharing.
fn make_coordinated_program(sentron_id: u16, partner_id: u16, len: usize) -> Vec<SIW> {
    (0..len).map(|i| {
        let rd = (i % 12 + 1) as u8;
        let rs1 = (i % 6 + 1) as u8;
        let rs2 = (i % 5 + 2) as u8;
        let rs3 = (i % 4 + 3) as u8;

        // D-pipe: FMA for maximum arithmetic density (3 ops/cycle target)
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

        // C-pipe: alternate CSEND/CRECV with partner (crosstalk exploitation)
        // SMT siblings share L1/L2 — CSEND from core 0 warms cache for core 4
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
            // CBAR every 8th SIW — temporal: data from 4 SIWs ago is L1-hot
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

fn run_coordinated_8core() -> (f64, CoopResult, f64) {
    let mut mem = Memory::new();
    

    // Create 8 sentrons, paired: (0,4), (1,5), (2,6), (3,7)
    // Pairs model SMT siblings sharing the same physical core
    let mut sentrons: Vec<Sentron> = (0..N_SENTRONS).map(|i| {
        let partner = if i < 4 { i + 4 } else { i - 4 };
        let coord = PhextCoord::new([i as u16 + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut s = Sentron::new(i as u16, coord, 0, 0);
        s.spawn(make_coordinated_program(i as u16, partner as u16, PROGRAM_LEN));
        s
    }).collect();

    let config = CoopConfig {
        quantum: QUANTUM,
        max_cycles: MAX_CYCLES,
    };

    let t0 = Instant::now();
    let mut agg_result = CoopResult::default();
    for _ in 0..ITERATIONS {
        // Re-spawn programs each iteration
        for (i, s) in sentrons.iter_mut().enumerate() {
            let partner = if i < 4 { i + 4 } else { i - 4 };
            s.spawn(make_coordinated_program(i as u16, partner as u16, PROGRAM_LEN));
        }
        let result = coop_execute(&mut sentrons, &mut mem, &config);
        agg_result.total_retired += result.total_retired;
        agg_result.total_cycles += result.total_cycles;
        agg_result.context_switches += result.context_switches;
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
    println!("\nvTPU W23 — 8-Core Coordination + Cache Crosstalk Exploitation");
    println!("=============================================================");
    println!("Sentrons : {}  (4 SMT pairs)", N_SENTRONS);
    println!("Program  : {} SIWs (D-pipe FMA + S-pipe prefetch + C-pipe msg)", PROGRAM_LEN);
    println!("Quantum  : {} SIWs/turn (2×4 wiring)", QUANTUM);
    println!("Iterations: {}", ITERATIONS);
    println!("Clock    : {} GHz (assumed)", GHZ / 1e9);
    println!();

    // Baseline: single sentron, packed SIWs, no coordination
    println!("── Baseline (1 sentron, no coordination) ──");
    let (base_opc, base_ops, base_secs) = run_baseline_single();
    println!("  ops/cycle : {:.3}", base_opc);
    println!("  ops total : {}", base_ops);
    println!("  wall time : {:.3}s", base_secs);
    println!();

    // Coordinated: 8 sentrons with C-pipe messaging
    println!("── Coordinated (8 sentrons, C-pipe crosstalk) ──");
    let (coord_opc, coord_result, coord_secs) = run_coordinated_8core();
    println!("  ops/cycle     : {:.3}", coord_opc);
    println!("  ops total     : {}", coord_result.total_retired);
    println!("  ctx switches  : {}", coord_result.context_switches);
    println!("  wall time     : {:.3}s", coord_secs);
    println!();

    let speedup = coord_result.total_retired as f64 / base_ops as f64;
    let opc_ratio = coord_opc / base_opc;

    println!("── Results ──");
    println!("  Throughput  : {:.2}× (8-core vs 1-core)", speedup);
    println!("  ops/cycle   : {:.3} coordinated vs {:.3} baseline ({:.1}%)",
        coord_opc, base_opc, opc_ratio * 100.0);
    println!();

    println!("── Cache Crosstalk Model ──");
    println!("  SMT pairs share L1/L2 → CSEND/CRECV hit shared cache");
    println!("  CBAR(count=2) syncs pair locally, not global 8-way");
    println!("  Temporal: prefetch fills L1 4 SIWs before CRECV reads");
    println!();

    // Gate check
    let gate_opc = coord_opc;
    if gate_opc >= STRETCH_OPS_CYCLE {
        println!("✅ W23 STRETCH GOAL: {:.3} ops/cycle ≥ {}", gate_opc, STRETCH_OPS_CYCLE);
    } else if gate_opc >= TARGET_OPS_CYCLE {
        println!("✅ W23 GATE PASSED: {:.3} ops/cycle ≥ {}", gate_opc, TARGET_OPS_CYCLE);
    } else {
        println!("❌ W23 GATE: {:.3} ops/cycle (target: {TARGET_OPS_CYCLE})", gate_opc);
        println!("   Note: cooperative SMT (single-thread interleave) has known");
        println!("   overhead from context switching. The Phase 0 baseline shows");
        println!("   {:.3} ops/cycle. Coordination cost = {:.1}%", base_opc,
            (1.0 - opc_ratio) * 100.0);
    }

    const STRETCH_OPS_CYCLE: f64 = 3.0;
    const TARGET_OPS_CYCLE: f64 = 2.5;
}
