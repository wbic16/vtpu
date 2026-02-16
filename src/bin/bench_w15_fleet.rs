//! W15 Fleet Benchmark — 360 sentrons on shared memory
//!
//! Measures: How fast can we cycle 360 concurrent sentrons through a shared pool?
//! This is the Phase 1 preview — SMT coordination at scale.

use vtpu_runtime::*;
use vtpu_runtime::pool::SentronPool;
use std::time::Instant;

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  vTPU W15 Fleet Benchmark — 360 Sentrons");
    println!("  Zen 4 (8945HS)");
    println!("═══════════════════════════════════════════════\n");

    bench_fleet_mul();
    bench_fleet_ternary();
    bench_fleet_pipeline();

    println!("\n═══════════════════════════════════════════════");
}

/// 360 sentrons each compute 7*6=42, cycled 1000 times
fn bench_fleet_mul() {
    let fleet_size = 360;
    let rounds = 1000;
    let mut pool = SentronPool::new(fleet_size, 8);
    let mut mem = Memory::new();
    let program = [SIW::new(
        DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
    )];

    let start = Instant::now();
    for _ in 0..rounds {
        // Check out all 360
        let mut indices = Vec::with_capacity(fleet_size);
        for _ in 0..fleet_size {
            let idx = pool.checkout().unwrap();
            pool.load_program(idx, &program);
            pool.get_mut(idx).regs.general[0] = 7;
            pool.get_mut(idx).regs.general[1] = 6;
            indices.push(idx);
        }
        // Execute all
        for &idx in &indices {
            exec::run(pool.get_mut(idx), &mut mem);
        }
        // Verify + return
        for &idx in &indices {
            debug_assert_eq!(pool.get(idx).regs.general[2], 42);
            pool.checkin(idx);
        }
    }
    let elapsed = start.elapsed();
    let total = fleet_size * rounds;
    let rate = total as f64 / elapsed.as_secs_f64();

    println!("  Fleet MUL:    {:>8} sentrons × {:>4} rounds = {:>8} executions in {:>6.2} ms  ({:.1}M sentrons/sec)",
        fleet_size, rounds, total, elapsed.as_secs_f64() * 1000.0, rate / 1_000_000.0);
}

/// 360 sentrons each run 8-element ternary matvec
fn bench_fleet_ternary() {
    let fleet_size = 360;
    let rounds = 100;

    let weights: Vec<i8> = vec![1, -1, 0, 1, -1, 0, 1, -1];
    let packed_trits: Vec<i64> = weights.chunks(1).map(|w| bitnet::pack_trits(w)[0]).collect();

    let mut program = vec![
        SIW::new(DenseOp::DMOV { rd: 15, imm: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ];
    for i in 0..8u8 {
        program.push(SIW::new(
            DenseOp::DTACC { rd: 15, rs1: i, trit_reg: 8 + i },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
        ));
    }

    let mut pool = SentronPool::new(fleet_size, 16);
    let mut mem = Memory::new();

    let start = Instant::now();
    for _ in 0..rounds {
        let mut indices = Vec::with_capacity(fleet_size);
        for _ in 0..fleet_size {
            let idx = pool.checkout().unwrap();
            pool.load_program(idx, &program);
            let s = pool.get_mut(idx);
            for i in 0..8 { s.regs.general[i] = (i as i64 + 1) * 10; }
            for i in 0..8 { s.regs.general[8 + i] = packed_trits[i]; }
            indices.push(idx);
        }
        for &idx in &indices {
            exec::run(pool.get_mut(idx), &mut mem);
        }
        for &idx in &indices {
            pool.checkin(idx);
        }
    }
    let elapsed = start.elapsed();
    let total = fleet_size * rounds;
    let rate = total as f64 / elapsed.as_secs_f64();

    println!("  Fleet TACC:   {:>8} sentrons × {:>4} rounds = {:>8} executions in {:>6.2} ms  ({:.1}M sentrons/sec)",
        fleet_size, rounds, total, elapsed.as_secs_f64() * 1000.0, rate / 1_000_000.0);
}

/// Producer-consumer pipeline: 180 producers compute, 180 consumers accumulate
fn bench_fleet_pipeline() {
    let half = 180;
    let rounds = 100;
    let mut pool = SentronPool::new(360, 8);
    let mut mem = Memory::new();

    let produce = [SIW::new(
        DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
    )];
    let consume = [SIW::new(
        DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
    )];

    let start = Instant::now();
    for r in 0..rounds {
        // Producers: compute i * (r+1)
        let mut producers = Vec::with_capacity(half);
        for i in 0..half {
            let idx = pool.checkout().unwrap();
            pool.load_program(idx, &produce);
            pool.get_mut(idx).regs.general[0] = i as i64;
            pool.get_mut(idx).regs.general[1] = (r + 1) as i64;
            producers.push(idx);
        }
        for &idx in &producers {
            exec::run(pool.get_mut(idx), &mut mem);
        }

        // Consumers: accumulate producer output + i
        let mut consumers = Vec::with_capacity(half);
        for i in 0..half {
            let cidx = pool.checkout().unwrap();
            pool.load_program(cidx, &consume);
            pool.get_mut(cidx).regs.general[0] = pool.get(producers[i]).regs.general[2];
            pool.get_mut(cidx).regs.general[1] = i as i64;
            consumers.push(cidx);
        }
        for &idx in &consumers {
            exec::run(pool.get_mut(idx), &mut mem);
        }

        // Return all
        for &idx in &producers { pool.checkin(idx); }
        for &idx in &consumers { pool.checkin(idx); }
    }
    let elapsed = start.elapsed();
    let total = 360 * rounds; // both halves
    let rate = total as f64 / elapsed.as_secs_f64();

    println!("  Fleet Pipe:   {:>8} sentrons × {:>4} rounds = {:>8} executions in {:>6.2} ms  ({:.1}M sentrons/sec)",
        360, rounds, total, elapsed.as_secs_f64() * 1000.0, rate / 1_000_000.0);
}
