//! W20 SIMD Sentron Group Benchmark — R23
//!
//! Measures throughput of SIMD 4-sentron group vs sequential OctaWire.
//!
//! SIMD approach: pack 4 sentron register files into AVX2 YMM vectors,
//! execute D-pipe arithmetic on all 4 simultaneously.
//!
//! Expected: 4× throughput for D-heavy arithmetic streams.
//! Note: AVX2 lacks native i64 MUL, so DMUL falls back to scalar per-lane.
//!       AVX-512 (ranch Zen 4) would give true 8× throughput.

use vtpu_runtime::{
    exec::exec_siw_octawire,
    simd::{SimdGroup, exec_simd_group},
    memory::Memory,
    phext_coord::PhextCoord,
    pipes::{CoordOp, DenseOp, SparseOp},
    sentron::Sentron,
    siw::SIW,
};
use std::time::Instant;

const STREAM_LEN: usize = 100_000;
const WARMUP: usize = 5_000;
const RUNS: usize = 5;
const LANE_COUNT: usize = 4; // SIMD width (4 × i64 = 256-bit AVX2)

fn make_sentron(base: i64) -> Sentron {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    for i in 0..16 { s.regs.general[i] = base + i as i64; }
    s
}

fn make_dadd_stream() -> Vec<SIW> {
    (0..STREAM_LEN).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = ((i + 1) % 14) as u8;
        let rs2 = ((i + 2) % 14) as u8;
        SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect()
}

fn make_dfma_stream() -> Vec<SIW> {
    (0..STREAM_LEN).map(|i| {
        let rd = (i % 13) as u8;
        let rs1 = ((i + 1) % 13) as u8;
        let rs2 = ((i + 2) % 13) as u8;
        let rs3 = ((i + 3) % 13) as u8;
        SIW::new(DenseOp::DFMA { rd, rs1, rs2, rs3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect()
}

fn make_mixed_arith_stream() -> Vec<SIW> {
    (0..STREAM_LEN).map(|i| {
        let rd = (i % 14) as u8;
        let rs1 = ((i + 1) % 14) as u8;
        let rs2 = ((i + 2) % 14) as u8;
        match i % 4 {
            0 => SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            1 => SIW::new(DenseOp::DSUB { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            2 => SIW::new(DenseOp::DMUL { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            _ => SIW::new(DenseOp::DMOV { rd, imm: i as i64 % 1000 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        }
    }).collect()
}

fn bench_simd(label: &str, stream: &[SIW]) -> f64 {
    let s0 = make_sentron(1);
    let s1 = make_sentron(101);
    let s2 = make_sentron(201);
    let s3 = make_sentron(301);
    let sentrons = [&s0, &s1, &s2, &s3];

    // warmup
    let mut group = SimdGroup::from_sentrons(&sentrons);
    exec_simd_group(&mut group, &stream[..WARMUP.min(stream.len())]);

    let mut best_ns = f64::MAX;
    for _ in 0..RUNS {
        let mut group = SimdGroup::from_sentrons(&sentrons);
        let t = Instant::now();
        exec_simd_group(&mut group, stream);
        let elapsed = t.elapsed().as_secs_f64() * 1e9;
        // ns per sentron-SIW (total work = stream.len() × LANE_COUNT sentrons)
        let ns_per = elapsed / (stream.len() * LANE_COUNT) as f64;
        if ns_per < best_ns { best_ns = ns_per; }
    }
    let throughput = 1e9 / best_ns;
    println!("[SIMD×4  ] {:<38} | {:>8.2} ns/sentron-SIW | {:>12.0} SIW/s×4",
        label, best_ns, throughput);
    best_ns
}

fn bench_sequential(label: &str, stream: &[SIW]) -> f64 {
    let mut sentrons: Vec<Sentron> = (0..LANE_COUNT).map(|i| make_sentron(i as i64 * 100 + 1)).collect();
    let mut mem = Memory::new();

    // warmup
    for siw in &stream[..WARMUP.min(stream.len())] {
        exec_siw_octawire(&mut sentrons[0], siw, &mut mem);
    }

    let mut best_ns = f64::MAX;
    for _ in 0..RUNS {
        let mut sentrons: Vec<Sentron> = (0..LANE_COUNT).map(|i| make_sentron(i as i64 * 100 + 1)).collect();
        let t = Instant::now();
        // Run 4 sentrons sequentially (same total work as SIMD)
        for s in sentrons.iter_mut() {
            for siw in stream {
                exec_siw_octawire(s, siw, &mut mem);
            }
        }
        let elapsed = t.elapsed().as_secs_f64() * 1e9;
        let ns_per = elapsed / (stream.len() * LANE_COUNT) as f64;
        if ns_per < best_ns { best_ns = ns_per; }
    }
    let throughput = 1e9 / best_ns;
    println!("[Seq×4   ] {:<38} | {:>8.2} ns/sentron-SIW | {:>12.0} SIW/s×4",
        label, best_ns, throughput);
    best_ns
}

fn main() {
    println!("=== W20 SIMD Sentron Group Benchmark (R23) ===");
    println!("SIMD width: {}×i64 (AVX2 YMM) | Stream: {} SIWs | {} warmup | {} runs",
        LANE_COUNT, STREAM_LEN, WARMUP, RUNS);
    #[cfg(target_feature = "avx2")]
    println!("AVX2: ENABLED (hardware SIMD path active)");
    #[cfg(not(target_feature = "avx2"))]
    println!("AVX2: DISABLED (scalar fallback path)");
    println!();

    let benchmarks = [
        ("DADD (add-heavy)", make_dadd_stream()),
        ("DFMA (fused multiply-add)", make_dfma_stream()),
        ("Mixed DADD/DSUB/DMUL/DMOV", make_mixed_arith_stream()),
    ];

    let mut total_simd_speedup = 0.0;
    let mut count = 0;

    for (label, stream) in &benchmarks {
        println!("--- {} ---", label);
        let simd_ns = bench_simd(label, stream);
        let seq_ns = bench_sequential(label, stream);
        let speedup = seq_ns / simd_ns;
        println!("    Speedup: {:.2}× (SIMD vs sequential per-sentron-SIW)", speedup);
        println!("    SIMD effective throughput: {:.0} SIW/s×4 = {:.0} total SIW/s",
            1e9 / simd_ns, 1e9 / simd_ns * LANE_COUNT as f64);
        total_simd_speedup += speedup;
        count += 1;
        println!();
    }

    println!("=== Summary ===");
    println!("Average SIMD speedup: {:.2}×", total_simd_speedup / count as f64);
    println!("Target: 4.0× (ideal AVX2); 8.0× (ideal AVX-512 on ranch Zen 4)");
    println!();
    println!("Notes:");
    println!("  DADD/DSUB: native _mm256_add/sub_epi64 → ideal 4×");
    println!("  DMUL: no native i64×i64 in AVX2 → scalar fallback (fix: AVX-512 ranch)");
    println!("  DFMA: scalar fallback (FMA is only for f32/f64 in x86)");
    println!("  Transpose overhead: from_sentrons() + write_back() amortized over stream length");
}
