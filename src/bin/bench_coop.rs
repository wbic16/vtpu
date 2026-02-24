//! Benchmark: Cooperative SMT vs Sequential execution
//!
//! Measures throughput of interleaved multi-sentron execution.
//! Target: 1.5× over sequential single-sentron.
//!
//! R23W28 — Theia 💎

use vtpu_runtime::coop_smt::{coop_execute, CoopConfig};
use vtpu_runtime::coop_fleet::coop_fleet_execute;
use vtpu_runtime::fleet::Fleet;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp};
use vtpu_runtime::siw::SIW;
use std::time::Instant;

fn make_program(len: usize) -> Vec<SIW> {
    (0..len).map(|i| {
        let rd = (i % 14 + 1) as u8;
        let rs1 = (i % 7 + 1) as u8;
        let rs2 = (i % 5 + 2) as u8;
        SIW::new(
            DenseOp::DADD { rd, rs1, rs2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }).collect()
}

fn bench_sequential(n_sentrons: usize, program_len: usize, iterations: u32) -> f64 {
    let mut mem = Memory::new();
    let program = make_program(program_len);

    let start = Instant::now();
    for _ in 0..iterations {
        for i in 0..n_sentrons {
            let mut sentron = Sentron::new(i as u16, PhextCoord::zero(), 0, 0);
            sentron.spawn(program.clone());
            let config = CoopConfig { quantum: program_len, max_cycles: 1_000_000 };
            coop_execute(std::slice::from_mut(&mut sentron), &mut mem, &config);
        }
    }
    let elapsed = start.elapsed();
    let total_ops = n_sentrons as f64 * program_len as f64 * iterations as f64;
    total_ops / elapsed.as_secs_f64()
}

fn bench_cooperative(n_sentrons: usize, program_len: usize, quantum: usize, iterations: u32) -> f64 {
    let mut mem = Memory::new();
    let program = make_program(program_len);

    let start = Instant::now();
    for _ in 0..iterations {
        let mut sentrons: Vec<Sentron> = (0..n_sentrons)
            .map(|i| {
                let mut s = Sentron::new(i as u16, PhextCoord::zero(), 0, 0);
                s.spawn(program.clone());
                s
            })
            .collect();
        let config = CoopConfig { quantum, max_cycles: 1_000_000 };
        coop_execute(&mut sentrons, &mut mem, &config);
    }
    let elapsed = start.elapsed();
    let total_ops = n_sentrons as f64 * program_len as f64 * iterations as f64;
    total_ops / elapsed.as_secs_f64()
}

fn bench_fleet(n_sentrons: usize, program_len: usize, quantum: usize, iterations: u32) -> f64 {
    let program = make_program(program_len);
    let mut mem = Memory::new();

    let start = Instant::now();
    for _ in 0..iterations {
        let mut fleet = Fleet::new(n_sentrons);
        for i in 0..n_sentrons as u16 {
            fleet.sentron_mut(i).unwrap().spawn(program.clone());
        }
        coop_fleet_execute(&mut fleet, &mut mem, quantum, 1_000_000);
    }
    let elapsed = start.elapsed();
    let total_ops = n_sentrons as f64 * program_len as f64 * iterations as f64;
    total_ops / elapsed.as_secs_f64()
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        vTPU Cooperative SMT Benchmark — R23W28             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let program_len = 1000;
    let iterations = 50;

    for &n in &[2, 4, 9, 40, 360] {
        let seq = bench_sequential(n, program_len, iterations);
        let coop4 = bench_cooperative(n, program_len, 4, iterations);
        let coop16 = bench_cooperative(n, program_len, 16, iterations);
        let fleet4 = bench_fleet(n, program_len, 4, iterations);

        let speedup_4 = coop4 / seq;
        let speedup_16 = coop16 / seq;
        let speedup_fleet = fleet4 / seq;

        println!("── {} sentrons × {} SIWs ──", n, program_len);
        println!("  Sequential:   {:.1}M ops/sec", seq / 1e6);
        println!("  Coop(q=4):    {:.1}M ops/sec  ({:.2}×)", coop4 / 1e6, speedup_4);
        println!("  Coop(q=16):   {:.1}M ops/sec  ({:.2}×)", coop16 / 1e6, speedup_16);
        println!("  Fleet(q=4):   {:.1}M ops/sec  ({:.2}×)", fleet4 / 1e6, speedup_fleet);
        println!();
    }
}
