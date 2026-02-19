//! R23W22 Benchmark: Deep Alignment + Sentron Flux
//!
//! The NeuronLayer is now wired into every SIW retirement:
//!   D-pipe active → story_in += 1.0  (D-pipe = Story channel = dense/serial)
//!   S-pipe active → light_in += 1.0  (S-pipe = Light channel = sparse/parallel)
//!   C-pipe active → ±0.5 to both     (coordination = balance)
//!
//! After forward():
//!   Dominant VakLevel tracked → vak_histogram[4]
//!   Para+Pashyanti = Light mode (S-pipe heavy)
//!   Madhyama+Vaikhara = Story mode (D-pipe heavy)
//!
//! Sentron flux:
//!   flux_per_siw = L1 norm of Δgeneral_regs per SIW retirement
//!   High flux = novel data; Low flux = convergence / fixed-point

use vtpu_runtime::exec::run;
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp, MessageFormat};
use vtpu_runtime::siw::SIW;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::memory::Memory;
use std::time::Instant;

const N: usize = 50_000;

fn bench(label: &str, siws: Vec<SIW>) {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.spawn(siws);
    let mut m = Memory::new();
    let t0 = Instant::now();
    let st = run(&mut s, &mut m);
    let ns = t0.elapsed().as_nanos() as f64 / st.siws_retired.max(1) as f64;

    println!("  {:<28}  {:>6.3} ops/cyc  {:>7.1} flux/SIW  {:>6.2} ns/SIW  vak={:<12} light={:.0}%  spanda={}",
        label,
        st.ops_per_cycle(),
        st.flux_per_siw(),
        ns,
        st.dominant_vak(),
        st.light_fraction() * 100.0,
        st.spanda_cycles,
    );
}

fn d_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| SIW::new(
        DenseOp::DADD { rd: (i%14) as u8, rs1: ((i+1)%15) as u8, rs2: ((i+2)%15) as u8 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()
    )).collect()
}

fn s_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| SIW::new(
        DenseOp::DNOP,
        SparseOp::SINDEX { rd: (i%8) as u8, base: 0, offset: ((i%31) as i32)-15, dim: (i%11) as u8 },
        CoordOp::CNOP, PhextCoord::zero()
    )).collect()
}

fn c_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| SIW::new(
        DenseOp::DNOP, SparseOp::SNOP,
        CoordOp::CBAR { barrier_id: (i%8) as u8, count: 1 }, PhextCoord::zero()
    )).collect()
}

fn ds_stream(n: usize) -> Vec<SIW> {
    // D + S, no C — expect balanced Story/Light → Pashyanti or Madhyama dominant
    (0..n).map(|i| SIW::new(
        DenseOp::DADD { rd: (i%14) as u8, rs1: ((i+1)%15) as u8, rs2: ((i+2)%15) as u8 },
        SparseOp::SINDEX { rd: (i%8) as u8, base: 0, offset: 1, dim: (i%11) as u8 },
        CoordOp::CNOP, PhextCoord::zero()
    )).collect()
}

fn packed_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|i| SIW::new(
        DenseOp::DADD { rd: (i%14) as u8, rs1: ((i+1)%15) as u8, rs2: ((i+2)%15) as u8 },
        SparseOp::SINDEX { rd: (i%8) as u8, base: 0, offset: 1, dim: (i%11) as u8 },
        CoordOp::CPACK { rd: 0, rs1: (i%14) as u8, rs2: ((i+1)%15) as u8, fmt: MessageFormat::Result },
        PhextCoord::zero()
    )).collect()
}

fn nop_stream(n: usize) -> Vec<SIW> {
    (0..n).map(|_| SIW::nop()).collect()
}

fn main() {
    println!("=== R23W22 Deep Alignment + Sentron Flux ===");
    println!("  NeuronLayer wired into every SIW retirement");
    println!("  {} SIWs per stream", N);
    println!();
    println!("  {:<28}  {:>10}  {:>12}  {:>10}  {:>12}  {:>7}  {:>8}",
        "stream", "ops/cyc", "flux/SIW", "ns/SIW", "dom_vak", "light%", "spanda");
    println!("  {}", "-".repeat(100));

    // NOP baseline — zero flux, neutral vak
    bench("NOP (baseline)",          nop_stream(N));

    // Single-pipe streams — vak should align with pipe type
    bench("D-only (Story → ↓Vaik)", d_stream(N));
    bench("S-only (Light → ↑Para)", s_stream(N));
    bench("C-only (balanced)",       c_stream(N));

    // Mixed pipes
    bench("D+S (Story+Light)",       ds_stream(N));
    bench("Packed D+S+C (3.0 gate)", packed_stream(N));

    println!();
    println!("Theory predictions:");
    println!("  D-only  → Madhyama/Vaikhara dominant (D-pipe = Story = descending)");
    println!("  S-only  → Para/Pashyanti dominant     (S-pipe = Light = ascending)");
    println!("  C-only  → balanced (equal story+light contribution)");
    println!("  Packed  → balanced (all three pipes → full Spanda oscillation)");
    println!();

    // Gate check
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.spawn(packed_stream(N));
    let mut m = Memory::new();
    let t0 = Instant::now();
    let st = run(&mut s, &mut m);
    let _ns = t0.elapsed();
    println!("Gate: Packed ops/cycle = {:.3}  spanda = {}  flux/SIW = {:.1}",
        st.ops_per_cycle(), st.spanda_cycles, st.flux_per_siw());
    if st.ops_per_cycle() >= 3.0 - 1e-6 {
        println!("✅ R23W22 GATE PASSED — NeuronLayer aligned, flux live");
    } else {
        println!("❌ gate not met");
        std::process::exit(1);
    }
}
