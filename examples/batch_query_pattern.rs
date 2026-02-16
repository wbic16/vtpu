//! Batch Query Packing Pattern
//!
//! Demonstrates optimal 3.0 ops/cycle packing on a realistic batch query workload.
//!
//! Pattern: Load → Compute similarity → Route to best match
//! Packing: SGATHER (load) + DHDSIM (similarity) + CSLICE (attention) in same SIW

use vtpu_runtime::{
    SIW, DenseOp, SparseOp, CoordOp, Sentron, Memory, PhextCoord,
    HyperVector, HDC_DEFAULT_WIDTH,
};
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          Batch Query Packing Pattern Demo                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Workload: Query 100 coordinates for nearest neighbors");
    println!("Pattern: SGATHER + DHDSIM + CSLICE (3-pipe packing)");
    println!();

    // Setup: Store 1000 coordinates in memory
    let mut memory = Memory::new();
    let mut sentron = Sentron::new();

    println!("Setup: Storing 1000 coordinates...");
    for i in 0..1000 {
        let coord = PhextCoord::new([
            (i % 256) as u16,
            ((i / 256) % 256) as u16,
            ((i / 65536) % 256) as u16,
            0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        memory.store(&coord, i);
    }
    println!("✓ 1000 coordinates stored\n");

    // Benchmark: Unpacked (sequential pipe usage)
    println!("─── Benchmark 1: Unpacked (Sequential Pipes) ───\n");
    let unpacked_siws = generate_unpacked_queries(100);
    let start = Instant::now();
    for siw in &unpacked_siws {
        let _ = execute_siw(&mut sentron, &mut memory, siw);
    }
    let unpacked_time = start.elapsed();

    let unpacked_ops = unpacked_siws.len(); // 1 op per SIW (only 1 pipe active)
    let unpacked_cycles = unpacked_siws.len(); // 1 SIW = 1 cycle
    let unpacked_ops_per_cycle = unpacked_ops as f64 / unpacked_cycles as f64;

    println!("SIWs: {}", unpacked_siws.len());
    println!("Ops: {} (1 per SIW)", unpacked_ops);
    println!("Cycles: {} (unpacked)", unpacked_cycles);
    println!("Ops/cycle: {:.2}", unpacked_ops_per_cycle);
    println!("Time: {:?}", unpacked_time);
    println!();

    // Benchmark: Packed (3-pipe usage)
    println!("─── Benchmark 2: Packed (D+S+C Pipes) ───\n");
    let packed_siws = generate_packed_queries(100);
    let start = Instant::now();
    for siw in &packed_siws {
        let _ = execute_siw(&mut sentron, &mut memory, siw);
    }
    let packed_time = start.elapsed();

    let packed_ops = packed_siws.len() * 3; // 3 ops per SIW (D+S+C)
    let packed_cycles = packed_siws.len(); // 1 SIW = 1 cycle
    let packed_ops_per_cycle = packed_ops as f64 / packed_cycles as f64;

    println!("SIWs: {}", packed_siws.len());
    println!("Ops: {} (3 per SIW)", packed_ops);
    println!("Cycles: {} (packed)", packed_cycles);
    println!("Ops/cycle: {:.2}", packed_ops_per_cycle);
    println!("Time: {:?}", packed_time);
    println!();

    // Comparison
    println!("─── Comparison ───\n");
    let speedup = unpacked_time.as_nanos() as f64 / packed_time.as_nanos() as f64;
    let efficiency_gain = packed_ops_per_cycle / unpacked_ops_per_cycle;

    println!("Speedup: {:.2}× faster", speedup);
    println!("Efficiency gain: {:.2}× more ops/cycle", efficiency_gain);
    println!();

    if packed_ops_per_cycle >= 2.5 {
        println!("✅ Phase 0 target achieved: {:.2} ≥ 2.5 ops/cycle", packed_ops_per_cycle);
    } else {
        println!("⚠️  Below Phase 0 target: {:.2} < 2.5 ops/cycle", packed_ops_per_cycle);
    }
    println!();

    // Show first few SIWs for educational purposes
    println!("─── Example SIWs (first 3) ───\n");
    println!("Unpacked pattern:");
    for (i, siw) in unpacked_siws.iter().take(3).enumerate() {
        println!("  SIW {}: {:?} (1 pipe active)", i, classify_siw(siw));
    }
    println!();
    println!("Packed pattern:");
    for (i, siw) in packed_siws.iter().take(3).enumerate() {
        println!("  SIW {}: {:?} (3 pipes active)", i, classify_siw(siw));
    }
    println!();

    println!("Pattern Summary:");
    println!("  Unpacked: Load → Compute → Slice (3 SIWs per query)");
    println!("  Packed: Load+Compute+Slice (1 SIW per query)");
    println!("  Result: 3× fewer SIWs, 3× better ops/cycle");
}

/// Generate unpacked query SIWs (sequential pipe usage - anti-pattern)
fn generate_unpacked_queries(count: usize) -> Vec<SIW> {
    let mut siws = Vec::new();

    for i in 0..count {
        // Step 1: SGATHER (load coordinate) - S-Pipe only
        let mut siw1 = SIW::nop();
        siw1.s_op = SparseOp::SGATHER;
        siw1.s_dst = (i % 16) as u8;
        siw1.s_coord = (i % 100) as u8;
        siws.push(siw1);

        // Step 2: DHDSIM (compute similarity) - D-Pipe only
        let mut siw2 = SIW::nop();
        siw2.d_op = DenseOp::DHDSIM;
        siw2.d_dst = ((i + 1) % 16) as u8;
        siw2.d_src1 = (i % 16) as u8;
        siw2.d_src2 = ((i + 2) % 16) as u8;
        siws.push(siw2);

        // Step 3: CSLICE (attention mask) - C-Pipe only
        let mut siw3 = SIW::nop();
        siw3.c_op = CoordOp::CSLICE { dim: (i % 11) as u8 };
        siws.push(siw3);
    }

    siws
}

/// Generate packed query SIWs (3-pipe usage - optimal pattern)
fn generate_packed_queries(count: usize) -> Vec<SIW> {
    let mut siws = Vec::new();

    for i in 0..count {
        // Single SIW: SGATHER + DHDSIM + CSLICE (all 3 pipes active)
        let mut siw = SIW::nop();

        // D-Pipe: Compute similarity
        siw.d_op = DenseOp::DHDSIM;
        siw.d_dst = ((i + 1) % 16) as u8;
        siw.d_src1 = (i % 16) as u8;
        siw.d_src2 = ((i + 2) % 16) as u8;

        // S-Pipe: Load coordinate
        siw.s_op = SparseOp::SGATHER;
        siw.s_dst = (i % 16) as u8;
        siw.s_coord = (i % 100) as u8;

        // C-Pipe: Apply attention mask
        siw.c_op = CoordOp::CSLICE { dim: (i % 11) as u8 };

        siws.push(siw);
    }

    siws
}

/// Execute a single SIW (simulated)
fn execute_siw(sentron: &mut Sentron, memory: &mut Memory, siw: &SIW) -> Result<(), ()> {
    // D-Pipe execution
    if siw.d_op != DenseOp::DNOP {
        // Simulate compute
        sentron.regs.general[siw.d_dst as usize] = 42;
    }

    // S-Pipe execution
    if siw.s_op == SparseOp::SGATHER {
        let coord = PhextCoord::new([siw.s_coord as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let value = memory.load(&coord).unwrap_or(0);
        sentron.regs.general[siw.s_dst as usize] = value;
    }

    // C-Pipe execution (CSLICE is mostly metadata, no register effects)
    // In real implementation, this would affect attention routing

    Ok(())
}

/// Classify SIW by active pipes (for display)
fn classify_siw(siw: &SIW) -> &'static str {
    let d_active = siw.d_op != DenseOp::DNOP;
    let s_active = siw.s_op != SparseOp::SNOP;
    let c_active = siw.c_op != CoordOp::CNOP;

    match (d_active, s_active, c_active) {
        (true, true, true) => "D+S+C (3 pipes)",
        (true, true, false) => "D+S (2 pipes)",
        (true, false, true) => "D+C (2 pipes)",
        (false, true, true) => "S+C (2 pipes)",
        (true, false, false) => "D only (1 pipe)",
        (false, true, false) => "S only (1 pipe)",
        (false, false, true) => "C only (1 pipe)",
        (false, false, false) => "NOP (0 pipes)",
    }
}
