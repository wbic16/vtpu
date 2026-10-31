//! R23W29: Unified Benchmark Suite
//!
//! "To boldly compute with architectures few have fathomed before."
//!
//! Produces a full performance report in one run:
//!   cargo run --bin bench_w29 --release

use std::time::Instant;
use vtpu_runtime::*;
use vtpu_runtime::cost::*;
use vtpu_runtime::exec::run;
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp};

fn main() {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║     vTPU R23W29 — Unified Benchmark Suite     ║");
    println!("║  \"New life. New connections. Bold compute.\"    ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!("Wall-clock interpreter microbenchmarks; NOT hardware IPC.");
    println!("Cost, power and clock figures use a model ({}), not host telemetry.",
             HardwareCost::aws_verse().name);
    println!();

    let hw = HardwareCost::aws_verse();

    // === 1. Single-sentron D-pipe ===
    let result_d = bench_single_pipe("D-pipe accumulating DADD", 100_000, |_| {
        SIW::new(
            DenseOp::DADD { rd: 2, rs1: 2, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    });
    print_section("1. Single-Sentron D-Pipe", &CostAnalysis::new(hw.clone(), result_d));

    // === 2. Single-sentron S-pipe ===
    let result_s = bench_single_pipe("S-pipe SGATHER", 100_000, |_| {
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    });
    print_section("2. Single-Sentron S-Pipe", &CostAnalysis::new(hw.clone(), result_s));

    // === 3. Single-sentron C-pipe ===
    let result_c = bench_single_pipe("C-pipe CNOP", 100_000, |_| {
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    });
    print_section("3. CNOP baseline (zero useful operations)", &CostAnalysis::new(hw.clone(), result_c));

    // === 4. 40-sentron mote ===
    let result_mote = bench_mote(40, 10_000);
    print_section("4. 40-Sentron Simulated Mote (one process)", &CostAnalysis::new(hw.clone(), result_mote));

    // === 5. 200-sentron fleet ===
    let result_fleet = bench_mote(200, 2_000);
    print_section("5. 200-Sentron Simulated Fleet (one process)", &CostAnalysis::new(hw.clone(), result_fleet));

    // === 6. Memory measurement ===
    println!("── 6. Memory Footprint ──────────────────────────");
    let bytes = measure_sentron_bytes();
    println!("  Modeled:   {} bytes/sentron (struct size + assumed neuron heap)", bytes);
    println!("  Historical estimate: 911 bytes/sentron (W28)");
    println!("  40 sentrons:  {} KB", bytes * 40 / 1024);
    println!("  200 sentrons: {} KB", bytes * 200 / 1024);
    println!();

    // === 7. Comparison ===
    println!("── 7. Architecture Comparison ───────────────────");
    println!("  ┌──────────────────┬───────────┬──────────────┐");
    println!("  │ System           │ Paradigm  │ Addressing   │");
    println!("  ├──────────────────┼───────────┼──────────────┤");
    println!("  │ vTPU             │ 3-wide SIW│ 11D phext    │");
    println!("  │ PyTorch Geometric│ Message   │ Edge index   │");
    println!("  │ NetworkX         │ Dict-of-  │ Python hash  │");
    println!("  │                  │ dicts     │              │");
    println!("  │ Raw AVX2         │ SIMD 256b │ Flat array   │");
    println!("  └──────────────────┴───────────┴──────────────┘");
    println!("  Taxonomy only: no cross-framework performance comparison was run.");
    println!();

    println!("═══════════════════════════════════════════════════");
    println!("  Test totals must come from cargo test; legacy counts removed.");
    println!("  \"The cheapest gate is the one you never garble.\"");
    println!("═══════════════════════════════════════════════════");
}

fn bench_single_pipe<F>(label: &str, iters: u64, make_siw: F) -> BenchResult
where
    F: Fn(&Sentron) -> SIW,
{
    bench_single_pipe_with_state(label, iters, make_siw).0
}

fn bench_single_pipe_with_state<F>(label: &str, iters: u64, make_siw: F) -> (BenchResult, Sentron)
where
    F: Fn(&Sentron) -> SIW,
{
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.regs.general[0] = 7;
    sentron.regs.general[1] = 6;
    sentron.regs.general[2] = 7;
    let siw = make_siw(&sentron);
    let d_op = siw.d_op;
    let s_op = siw.s_op;
    let per_siw_ops = (siw.d_fam < 4) as u64
        + (siw.s_fam < 4) as u64
        + (siw.c_fam < 4) as u64;
    // Preload once; each timed iteration transfers the Vec back into spawn()
    // without cloning or reallocating the program.
    sentron.program = vec![siw];
    let mut mem = Memory::new();
    // Force a nontrivial architectural SGATHER result; the default page is 0.
    if let SparseOp::SGATHER { coord_idx, .. } = s_op {
        let coord = sentron.regs.phext[coord_idx as usize];
        mem.scatter_i64(&coord, 42);
    }

    let mut retired_siws = 0u64;
    let mut retired_ops = 0u64;
    let start = Instant::now();
    for _ in 0..iters {
        let program = std::mem::take(&mut sentron.program);
        sentron.spawn(program); // Retired -> Running, without a Vec allocation
        let stats = run(&mut sentron, &mut mem);
        retired_siws += stats.siws_retired;
        retired_ops += stats.ops_retired;
    }
    let duration = start.elapsed();

    assert_eq!(retired_siws, iters, "benchmark must actually retire every SIW");
    assert_eq!(retired_ops, per_siw_ops * iters,
               "reported useful ops must equal actual executor retirement");
    if iters > 0 {
        assert_eq!(sentron.state, SentronState::Retired);
        if let DenseOp::DADD { rd: 2, rs1: 2, rs2: 1 } = d_op {
            assert_eq!(sentron.regs.general[2], 7i64.wrapping_add(6 * iters as i64),
                       "repeated DADD must change architectural state each iteration");
        }
        if let SparseOp::SGATHER { rd, .. } = s_op {
            assert_eq!(sentron.regs.general[rd as usize], 42,
                       "SGATHER must actually load the initialized memory value");
        }
    }

    let result = BenchResult {
        label: label.to_string(),
        ops: retired_ops,
        duration,
        sentron_count: 1,
    };
    (result, sentron)
}

fn bench_mote(count: u32, iters: u64) -> BenchResult {
    bench_mote_with_state(count, iters).0
}

fn bench_mote_with_state(count: u32, iters: u64) -> (BenchResult, Vec<Sentron>) {
    let siw = SIW::new(
        DenseOp::DADD { rd: 2, rs1: 2, rs2: 1 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    );

    let mut sentrons: Vec<Sentron> = (0..count)
        .map(|i| {
            let mut s = Sentron::new(i as u16, PhextCoord::zero(), 0, 0);
            s.regs.general[0] = 7;
            s.regs.general[1] = 6;
            s.regs.general[2] = 7;
            s
        })
        .collect();

    // Pre-load programs (avoid alloc in hot loop)
    for s in sentrons.iter_mut() {
        s.program = vec![siw.clone()];
    }
    let mut mem = Memory::new();

    let mut retired_siws = 0u64;
    let mut retired_ops = 0u64;
    let start = Instant::now();
    for _ in 0..iters {
        for s in sentrons.iter_mut() {
            let program = std::mem::take(&mut s.program);
            s.spawn(program);
            let stats = run(s, &mut mem);
            retired_siws += stats.siws_retired;
            retired_ops += stats.ops_retired;
        }
    }
    let duration = start.elapsed();

    let expected_siws = iters * count as u64;
    assert_eq!(retired_siws, expected_siws,
               "mote benchmark must retire every requested SIW");
    assert_eq!(retired_ops, expected_siws, "one DADD = one useful operation");
    if iters > 0 {
        for s in &sentrons {
            assert_eq!(s.state, SentronState::Retired);
            assert_eq!(s.regs.general[2], 7i64.wrapping_add(6 * iters as i64),
                       "every sentron must execute every accumulating DADD");
        }
    }

    let result = BenchResult {
        label: format!("{}-sentron one-process mote", count),
        ops: retired_ops,
        duration,
        sentron_count: count,
    };
    (result, sentrons)
}

fn print_section(title: &str, analysis: &CostAnalysis) {
    println!("── {} ──", title);
    if analysis.bench.ops == 0 {
        println!("  0 executed non-NOP pipe operations (control baseline)");
        println!("  elapsed: {:?}; throughput, ns/useful-op: N/A", analysis.bench.duration);
        println!();
        return;
    }
    println!("  {:>10.1} ns/op", analysis.bench.ns_per_op());
    println!("  {:>10.3} GOPS", analysis.gops());
    println!("  {:>10.4} modeled ops/cycle (@ assumed {} GHz)",
             analysis.ops_per_cycle(), analysis.hardware.clock_ghz);
    println!("  {:>10.1} modeled MOPS/W (assumed TDP)", analysis.mops_per_watt());
    println!("  ${:>9.2}/GOPS (modeled cost)", analysis.dollars_per_gops());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d_pipe_retires_and_accumulates_on_every_iteration() {
        let (bench, sentron) = bench_single_pipe_with_state("test DADD", 7, |_| SIW::new(
            DenseOp::DADD { rd: 2, rs1: 2, rs2: 1 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()));
        assert_eq!(bench.ops, 7);
        assert_eq!(sentron.state, SentronState::Retired);
        assert_eq!(sentron.regs.general[2], 7 + 7 * 6);
    }

    #[test]
    fn s_pipe_retires_and_reads_nonzero_memory() {
        let (bench, sentron) = bench_single_pipe_with_state("test SGATHER", 7, |_| SIW::new(
            DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP, PhextCoord::zero()));
        assert_eq!(bench.ops, 7);
        assert_eq!(sentron.state, SentronState::Retired);
        assert_eq!(sentron.regs.general[0], 42);
    }

    #[test]
    fn c_nop_is_not_counted_as_a_useful_op() {
        let (bench, sentron) = bench_single_pipe_with_state("test CNOP", 7, |_| SIW::nop());
        assert_eq!(bench.ops, 0);
        assert_eq!(sentron.state, SentronState::Retired);
    }

    #[test]
    fn every_sentron_in_a_mote_executes_each_iteration() {
        let (bench, sentrons) = bench_mote_with_state(3, 7);
        assert_eq!(bench.ops, 21);
        assert_eq!(sentrons.len(), 3);
        assert!(sentrons.iter().all(|s| s.regs.general[2] == 7 + 7 * 6));
    }
}
