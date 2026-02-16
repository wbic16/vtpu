//! W17 Scheduler Redux — Real-time OS feedback loop
//!
//! Runs sentron batches, samples OS scheduler state after each,
//! and adapts placement based on measured throughput + migration rate.

use vtpu_runtime::*;
use vtpu_runtime::pool::SentronPool;
use vtpu_runtime::scheduler::SchedulerRedux;
use std::time::Instant;

const FLEET: usize = 360;
const BATCHES: usize = 100;

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  vTPU W17 Scheduler Redux");
    println!("  Real-time OS feedback → vTPU adaptation");
    println!("═══════════════════════════════════════════════\n");

    let mut pool = SentronPool::new(FLEET, 4);
    let mut mem = Memory::new();
    let mut redux = SchedulerRedux::new();

    let program = [SIW::new(
        DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L1 },
        CoordOp::CNOP,
        PhextCoord::zero(),
    )];

    let mut total_ops = 0u64;
    let global_start = Instant::now();

    for batch in 0..BATCHES {
        let batch_start = Instant::now();

        for _ in 0..FLEET {
            let idx = pool.checkout().unwrap();
            pool.load_program(idx, &program);
            pool.get_mut(idx).regs.general[0] = 7;
            pool.get_mut(idx).regs.general[1] = 6;
            exec::run(pool.get_mut(idx), &mut mem);
            pool.checkin(idx);
        }

        let elapsed = batch_start.elapsed();
        let throughput = FLEET as f64 / elapsed.as_secs_f64();
        total_ops += FLEET as u64;

        // Feed back to redux
        let fb = redux.sample(throughput);

        // Log every 20 batches
        if batch % 20 == 0 {
            println!("  Batch {:>3}: cpu={:>2} {:.1}M/sec {}{}",
                batch, fb.current_cpu,
                throughput / 1_000_000.0,
                if fb.migrated { "↪MIGRATED " } else { "" },
                if redux.recommends_pinning() { "📌PIN" } else { "" },
            );
        }

        // Apply learned preference after enough samples
        if batch == 50 && redux.recommends_pinning() {
            let pinned = redux.apply();
            println!("\n  → Redux applied pin to cpu {} (success: {})\n",
                redux.best_cpu(), pinned);
        }
    }

    let global_elapsed = global_start.elapsed();
    let global_rate = total_ops as f64 / global_elapsed.as_secs_f64();

    println!("\n  ─── Redux Summary ───");
    println!("  Total:       {} sentrons in {:.2} ms ({:.1}M/sec)",
        total_ops, global_elapsed.as_secs_f64() * 1000.0, global_rate / 1_000_000.0);
    println!("  Samples:     {}", redux.total_samples());
    println!("  Migrations:  {} ({:.1}%)", redux.total_migrations(),
        redux.migration_rate() * 100.0);
    println!("  Best CPU:    {} ({:.1}M/sec peak)", redux.best_cpu(),
        redux.average_throughput() / 1_000_000.0);
    println!("  Recommends:  {}", if redux.recommends_pinning() { "PIN" } else { "LET OS DECIDE" });

    println!("\n═══════════════════════════════════════════════");
}
