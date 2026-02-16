//! W17 Affinity Benchmark — Pinned vs unpinned SMT performance
//!
//! Measures the real impact of CPU affinity on SMT complementary workloads.
//! Pin both threads to the same physical core → guaranteed L1/L2 sharing.

use vtpu_runtime::*;
use vtpu_runtime::pool::SentronPool;
#[allow(unused_imports)]
use vtpu_runtime::affinity::*;
use std::time::Instant;
use std::thread;
use std::sync::Arc;
use std::sync::Barrier;

const FLEET: usize = 360;
const ROUNDS: usize = 2000;

fn main() {
    let logical = num_cpus();
    let physical = num_physical_cores();
    println!("═══════════════════════════════════════════════");
    println!("  vTPU W17 Affinity Benchmark");
    println!("  {} logical cores, {} physical (SMT={})",
        logical, physical, if logical > physical { "on" } else { "off" });
    println!("═══════════════════════════════════════════════\n");

    let unpinned = bench_dual(None, None, "Unpinned");
    let pinned_same = bench_dual(Some(0), Some(0 + physical), "Pinned SMT pair (core 0)");
    let pinned_diff = if physical >= 2 {
        bench_dual(Some(0), Some(1), "Pinned diff cores (0,1)")
    } else {
        0.0
    };

    println!("\n  ─── Results ───");
    println!("  Unpinned:         {:.1}M/sec", unpinned / 1_000_000.0);
    println!("  Pinned SMT pair:  {:.1}M/sec ({:.2}x vs unpinned)",
        pinned_same / 1_000_000.0, pinned_same / unpinned);
    if physical >= 2 {
        println!("  Pinned diff cores: {:.1}M/sec ({:.2}x vs unpinned)",
            pinned_diff / 1_000_000.0, pinned_diff / unpinned);
    }
    println!("\n═══════════════════════════════════════════════");
}

fn bench_dual(pin_a: Option<usize>, pin_b: Option<usize>, label: &str) -> f64 {
    let barrier = Arc::new(Barrier::new(2));

    let b1 = Arc::clone(&barrier);
    let handle_a = thread::spawn(move || {
        if let Some(core) = pin_a {
            let _ = pin_thread(&CpuSet::single(core));
        }
        let mut pool = SentronPool::new(FLEET / 2, 4);
        let mut mem = Memory::new();
        let program = [SIW::new(
            DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
        )];
        b1.wait();
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
        (FLEET / 2 * ROUNDS) as f64 / start.elapsed().as_secs_f64()
    });

    let b2 = Arc::clone(&barrier);
    let handle_b = thread::spawn(move || {
        if let Some(core) = pin_b {
            let _ = pin_thread(&CpuSet::single(core));
        }
        let mut pool = SentronPool::new(FLEET / 2, 4);
        let mut mem = Memory::new();
        let program = [SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP, PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]),
        )];
        b2.wait();
        let start = Instant::now();
        for _ in 0..ROUNDS {
            for _ in 0..FLEET / 2 {
                let idx = pool.checkout().unwrap();
                pool.load_program(idx, &program);
                exec::run(pool.get_mut(idx), &mut mem);
                pool.checkin(idx);
            }
        }
        (FLEET / 2 * ROUNDS) as f64 / start.elapsed().as_secs_f64()
    });

    let a = handle_a.join().unwrap();
    let b = handle_b.join().unwrap();
    let combined = a + b;
    println!("  {}: {:.1}M/sec (A={:.1}M, B={:.1}M)",
        label, combined / 1_000_000.0, a / 1_000_000.0, b / 1_000_000.0);
    combined
}
