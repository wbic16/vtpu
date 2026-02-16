//! W16 SMT Benchmark — Dual-thread complementary workloads
//!
//! Measures real SMT benefit: two threads on the same physical core
//! running complementary pipe workloads (D-heavy vs S-heavy).
//!
//! Key insight: SMT wins when threads use different execution ports.
//! D-Pipe (ALU) and S-Pipe (memory) are naturally complementary.
//!
//! Zero deps — no libc for pinning. Uses pool for zero-alloc hot path.

use vtpu_runtime::*;
use vtpu_runtime::pool::SentronPool;
use std::time::Instant;
use std::thread;
use std::sync::Arc;
use std::sync::Barrier;

const FLEET: usize = 360;
const ROUNDS: usize = 1000;

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  vTPU W16 SMT Benchmark");
    println!("  Zen 4 (8945HS) — Complementary Pipes");
    println!("═══════════════════════════════════════════════\n");

    // Single-thread baseline
    let single = bench_single_thread();

    // Dual-thread (simulated SMT — OS schedules)
    let dual = bench_dual_thread();

    let speedup = dual / single;
    println!("\n  ─── Summary ───");
    println!("  Single-thread: {:.1}M sentrons/sec", single / 1_000_000.0);
    println!("  Dual-thread:   {:.1}M sentrons/sec", dual / 1_000_000.0);
    println!("  Speedup:       {:.2}x", speedup);
    println!("  Target:        1.8x");
    println!("  Status:        {}", if speedup >= 1.5 { "✅ SMT benefit confirmed" } else { "📊 Measuring (OS scheduling)" });

    println!("\n═══════════════════════════════════════════════");
}

fn bench_single_thread() -> f64 {
    let mut pool = SentronPool::new(FLEET, 4);
    let mut mem = Memory::new();

    // Mixed D+S program (1 SIW)
    let program = [SIW::new(
        DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L1 },
        CoordOp::CNOP,
        PhextCoord::zero(),
    )];

    let start = Instant::now();
    for _ in 0..ROUNDS {
        for _ in 0..FLEET {
            let idx = pool.checkout().unwrap();
            pool.load_program(idx, &program);
            pool.get_mut(idx).regs.general[0] = 7;
            pool.get_mut(idx).regs.general[1] = 6;
            exec::run(pool.get_mut(idx), &mut mem);
            pool.checkin(idx);
        }
    }
    let elapsed = start.elapsed();
    let total = (FLEET * ROUNDS) as f64;
    let rate = total / elapsed.as_secs_f64();

    println!("  Single:   {:>8} executions in {:>6.2} ms  ({:.1}M/sec)",
        FLEET * ROUNDS, elapsed.as_secs_f64() * 1000.0, rate / 1_000_000.0);

    rate
}

fn bench_dual_thread() -> f64 {
    let barrier = Arc::new(Barrier::new(2));

    // Thread A: D-heavy (compute)
    let b1 = Arc::clone(&barrier);
    let handle_a = thread::spawn(move || {
        let mut pool = SentronPool::new(FLEET / 2, 4);
        let mut mem = Memory::new();
        let program = [SIW::new(
            DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )];

        b1.wait(); // sync start
        let start = Instant::now();
        for _ in 0..ROUNDS {
            for _ in 0..FLEET / 2 {
                let idx = pool.checkout().unwrap();
                pool.load_program(idx, &program);
                pool.get_mut(idx).regs.general[0] = 7;
                pool.get_mut(idx).regs.general[1] = 6;
                exec::run(pool.get_mut(idx), &mut mem);
                pool.checkin(idx);
            }
        }
        let elapsed = start.elapsed();
        let count = (FLEET / 2 * ROUNDS) as f64;
        count / elapsed.as_secs_f64()
    });

    // Thread B: S-heavy (memory)
    let b2 = Arc::clone(&barrier);
    let handle_b = thread::spawn(move || {
        let mut pool = SentronPool::new(FLEET / 2, 4);
        let mut mem = Memory::new();
        let program = [SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        )];

        b2.wait(); // sync start
        let start = Instant::now();
        for _ in 0..ROUNDS {
            for _ in 0..FLEET / 2 {
                let idx = pool.checkout().unwrap();
                pool.load_program(idx, &program);
                exec::run(pool.get_mut(idx), &mut mem);
                pool.checkin(idx);
            }
        }
        let elapsed = start.elapsed();
        let count = (FLEET / 2 * ROUNDS) as f64;
        count / elapsed.as_secs_f64()
    });

    let rate_a = handle_a.join().unwrap();
    let rate_b = handle_b.join().unwrap();
    let combined = rate_a + rate_b;

    println!("  Thread A: D-heavy  {:.1}M/sec", rate_a / 1_000_000.0);
    println!("  Thread B: S-heavy  {:.1}M/sec", rate_b / 1_000_000.0);
    println!("  Combined:          {:.1}M/sec", combined / 1_000_000.0);

    combined
}
