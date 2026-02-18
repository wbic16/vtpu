//! OctaWire Dispatch Benchmark — R23 W19
//!
//! Measures throughput of OctaWire (4-family indexed) vs legacy (triple match) dispatch.
//! 2×4 wiring insight: 4 op families × 2 directions = 8 wires per pipe-neuron.

use vtpu_runtime::{
    exec::{self, exec_siw_octawire},
    memory::Memory,
    phext_coord::PhextCoord,
    pipes::{CoordOp, DenseOp, MessageFormat, SparseOp},
    sentron::{Sentron, SentronState},
    siw::SIW,
};
use std::time::Instant;

const STREAM_LEN: usize = 100_000;
const WARMUP: usize = 10_000;

fn make_arithmetic_stream() -> Vec<SIW> {
    (0..STREAM_LEN)
        .map(|i| {
            let rd = (i % 14) as u8;
            let rs1 = ((i + 1) % 14) as u8;
            let rs2 = ((i + 2) % 14) as u8;
            SIW::new(
                DenseOp::DADD { rd, rs1, rs2 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            )
        })
        .collect()
}

fn make_full_stream() -> Vec<SIW> {
    (0..STREAM_LEN)
        .map(|i| {
            let rd = (i % 14) as u8;
            let rs1 = ((i + 1) % 14) as u8;
            let rs2 = ((i + 2) % 14) as u8;
            SIW::new(
                DenseOp::DADD { rd, rs1, rs2 },
                SparseOp::SINDEX {
                    rd: rd % 8,
                    base: rs1 % 8,
                    offset: (i as i32 % 16) - 8,
                    dim: (i % 9) as u8,
                },
                CoordOp::CPACK {
                    rd: rd % 4,
                    rs1,
                    rs2,
                    fmt: MessageFormat::Result,
                },
                PhextCoord::zero(),
            )
        })
        .collect()
}

fn make_mixed_stream() -> Vec<SIW> {
    (0..STREAM_LEN)
        .map(|i| {
            let rd = (i % 14) as u8;
            let rs1 = ((i + 1) % 14) as u8;
            let rs2 = ((i + 2) % 14) as u8;
            match i % 8 {
                0 => SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                1 => SIW::new(DenseOp::DMUL { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                2 => SIW::new(DenseOp::DHDBIND { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                3 => SIW::new(DenseOp::DTACC { rd, rs1, trit_reg: rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                4 => SIW::new(DenseOp::DNOP, SparseOp::SINDEX { rd: rd%8, base: rs1%8, offset: 1, dim: 0 }, CoordOp::CNOP, PhextCoord::zero()),
                5 => SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CPACK { rd: rd%4, rs1, rs2, fmt: MessageFormat::Result }, PhextCoord::zero()),
                6 => SIW::new(DenseOp::DRED { rd, rs1, op: vtpu_runtime::pipes::ReductionOp::Sum }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                _ => SIW::nop(),
            }
        })
        .collect()
}

fn bench_octawire(name: &str, siws: &[SIW]) {
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.state = SentronState::Running;
    let mut mem = Memory::new();

    // Warmup
    for siw in &siws[..WARMUP] {
        exec_siw_octawire(&mut sentron, siw, &mut mem);
    }

    let t0 = Instant::now();
    let mut total_active = 0u64;
    for siw in siws {
        total_active += exec_siw_octawire(&mut sentron, siw, &mut mem) as u64;
    }
    let elapsed = t0.elapsed();

    let siws_per_sec = siws.len() as f64 / elapsed.as_secs_f64();
    let ns_per_siw = elapsed.as_nanos() as f64 / siws.len() as f64;
    let utilization = total_active as f64 / (siws.len() as f64 * 3.0);

    println!(
        "[OctaWire] {:30} | {:8.1} ns/SIW | {:10.0} SIW/s | util {:.1}%",
        name, ns_per_siw, siws_per_sec, utilization * 100.0
    );
}

fn bench_legacy(name: &str, siws: &[SIW]) {
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.state = SentronState::Running;
    let mut mem = Memory::new();

    // Warmup via legacy run
    sentron.spawn(siws[..WARMUP].to_vec());
    exec::run(&mut sentron, &mut mem);

    // Reset and benchmark
    sentron.spawn(siws.to_vec());
    let t0 = Instant::now();
    let stats = exec::run(&mut sentron, &mut mem);
    let elapsed = t0.elapsed();

    let siws_per_sec = stats.siws_retired as f64 / elapsed.as_secs_f64();
    let ns_per_siw = elapsed.as_nanos() as f64 / stats.siws_retired.max(1) as f64;

    println!(
        "[Legacy  ] {:30} | {:8.1} ns/SIW | {:10.0} SIW/s | util {:.1}%",
        name, ns_per_siw, siws_per_sec, stats.utilization() * 100.0
    );
}

fn main() {
    println!("=== OctaWire Dispatch Benchmark (R23 W19) ===");
    println!("Stream length: {} SIWs | Warmup: {} SIWs", STREAM_LEN, WARMUP);
    println!("2×4 wiring: 4 op families × 2 directions per family = 8 wire-ends per pipe-neuron");
    println!();

    let arithmetic = make_arithmetic_stream();
    let full = make_full_stream();
    let mixed = make_mixed_stream();

    // OctaWire dispatch
    bench_octawire("D-only arithmetic", &arithmetic);
    bench_octawire("D+S+C full 3-wide", &full);
    bench_octawire("Mixed 8-mode", &mixed);
    println!();

    // Legacy dispatch (via exec::run)
    bench_legacy("D-only arithmetic", &arithmetic);
    bench_legacy("D+S+C full 3-wide", &full);
    bench_legacy("Mixed 8-mode", &mixed);
    println!();

    // Mode bit distribution
    println!("=== SIW Mode Distribution ===");
    let mode_counts: [usize; 8] = mixed.iter().fold([0usize; 8], |mut acc, s| {
        acc[s.mode_bits() as usize] += 1;
        acc
    });
    for (mode, count) in mode_counts.iter().enumerate() {
        if *count > 0 {
            println!(
                "  mode {:03b} (D:{} S:{} C:{}) — {} SIWs ({:.1}%)",
                mode,
                (mode & 1) != 0,
                (mode & 2) != 0,
                (mode & 4) != 0,
                count,
                *count as f64 / mixed.len() as f64 * 100.0
            );
        }
    }
}
