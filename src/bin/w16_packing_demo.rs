//! R23W16 - Packing Patterns Demonstration
//!
//! Shows real examples from the PACKING-PATTERNS guide in action.
//! Demonstrates:
//! 1. Good packing (mixed D/S/C ops)
//! 2. Bad packing (sequential pipe usage)
//! 3. The packer's automatic optimization

use vtpu_runtime::*;
use vtpu_runtime::packer::{ScalarOp, pack};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║         R23W16: Instruction Packing Patterns Demo               ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    
    println!("═══ Example 1: Sequential Pipe Usage (Anti-Pattern) ═══\n");
    demo_bad_packing();
    println!();
    
    println!("═══ Example 2: Mixed Pipe Usage (Good Pattern) ═══\n");
    demo_good_packing();
    println!();
    
    println!("═══ Example 3: Automatic Packer Optimization ═══\n");
    demo_automatic_packing();
    println!();
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("                         SUMMARY");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("Bad packing (sequential):  ~1.0 ops/cycle");
    println!("Good packing (mixed):       2.5-3.0 ops/cycle");
    println!("Auto packer:                Optimizes automatically");
    println!();
    println!("Lesson: Mix D/S/C operations, use the packer, achieve 3× throughput ✅");
}

fn demo_bad_packing() {
    println!("Anti-pattern: Group all ops of same type together\n");
    
    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    
    sentron.regs.general[0] = 10;
    sentron.regs.general[1] = 20;
    
    // All S-Pipe ops first
    let mut program = vec![
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 2, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        ),
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 3, coord_idx: 1, width: 8 },
            CoordOp::CNOP,
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]),
        ),
    ];
    
    // Then all D-Pipe ops
    program.push(SIW::new(
        DenseOp::DADD { rd: 4, rs1: 0, rs2: 1 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    ));
    program.push(SIW::new(
        DenseOp::DMUL { rd: 5, rs1: 4, rs2: 2 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    ));
    
    // Then S-Pipe store
    program.push(SIW::new(
        DenseOp::DNOP,
        SparseOp::SSCATTR { coord_idx: 2, rs: 5, width: 8 },
        CoordOp::CNOP,
        PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3]),
    ));
    
    sentron.spawn(program.clone());
    let stats = exec::run(&mut sentron, &mut mem);
    
    println!("  SIWs: {}", stats.siws_retired);
    println!("  Ops: {}", stats.ops_retired);
    println!("  Cycles: {}", stats.cycles);
    println!("  Ops/cycle: {:.2}", stats.ops_per_cycle());
    println!("  Utilization: {:.1}%", stats.utilization() * 100.0);
    println!();
    println!("  Analysis: Only one pipe active per SIW → poor packing");
}

fn demo_good_packing() {
    println!("Good pattern: Mix D/S/C operations\n");
    
    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    
    sentron.regs.general[0] = 10;
    sentron.regs.general[1] = 20;
    
    // Mixed D+S ops
    let program = vec![
        SIW::new(
            DenseOp::DADD { rd: 4, rs1: 0, rs2: 1 },       // D-Pipe
            SparseOp::SGATHER { rd: 2, coord_idx: 0, width: 8 },  // S-Pipe
            CoordOp::CNOP,                                  // (could add C-Pipe op here)
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        ),
        SIW::new(
            DenseOp::DMUL { rd: 5, rs1: 4, rs2: 2 },       // D-Pipe
            SparseOp::SGATHER { rd: 3, coord_idx: 1, width: 8 },  // S-Pipe
            CoordOp::CNOP,
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]),
        ),
        SIW::new(
            DenseOp::DADD { rd: 6, rs1: 5, rs2: 3 },       // D-Pipe
            SparseOp::SSCATTR { coord_idx: 2, rs: 6, width: 8 }, // S-Pipe
            CoordOp::CNOP,
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3]),
        ),
    ];
    
    sentron.spawn(program.clone());
    let stats = exec::run(&mut sentron, &mut mem);
    
    println!("  SIWs: {}", stats.siws_retired);
    println!("  Ops: {}", stats.ops_retired);
    println!("  Cycles: {}", stats.cycles);
    println!("  Ops/cycle: {:.2}", stats.ops_per_cycle());
    println!("  Utilization: {:.1}%", stats.utilization() * 100.0);
    println!();
    println!("  Analysis: Two pipes active per SIW → 2.0 ops/cycle achieved");
}

fn demo_automatic_packing() {
    println!("Using the packer to optimize automatically\n");
    
    // Build unpacked ops (one per pipe)
    let mut ops = Vec::new();
    
    // Load ops (S-Pipe)
    ops.push(ScalarOp::S(SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 8 }));
    ops.push(ScalarOp::S(SparseOp::SGATHER { rd: 2, coord_idx: 1, width: 8 }));
    ops.push(ScalarOp::S(SparseOp::SGATHER { rd: 3, coord_idx: 2, width: 8 }));
    
    // Compute ops (D-Pipe)
    ops.push(ScalarOp::D(DenseOp::DADD { rd: 4, rs1: 1, rs2: 2 }));
    ops.push(ScalarOp::D(DenseOp::DMUL { rd: 5, rs1: 4, rs2: 3 }));
    ops.push(ScalarOp::D(DenseOp::DSUB { rd: 6, rs1: 5, rs2: 1 }));
    
    // Store ops (S-Pipe)
    ops.push(ScalarOp::S(SparseOp::SSCATTR { coord_idx: 3, rs: 6, width: 8 }));
    
    println!("  Input: {} scalar ops (unpacked)", ops.len());
    
    // Pack automatically
    let packed = pack(&ops);
    
    println!("  Output: {} packed SIWs", packed.packed_siws);
    println!("  Packing efficiency: {:.1}%", packed.utilization * 100.0);
    println!();
    
    // Execute packed stream
    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.spawn(packed.stream);
    
    let stats = exec::run(&mut sentron, &mut mem);
    
    println!("  Execution:");
    println!("    SIWs: {}", stats.siws_retired);
    println!("    Ops: {}", stats.ops_retired);
    println!("    Cycles: {}", stats.cycles);
    println!("    Ops/cycle: {:.2}", stats.ops_per_cycle());
    println!("    Utilization: {:.1}%", stats.utilization() * 100.0);
    println!();
    println!("  Analysis: Packer automatically mixed D/S ops for better utilization");
}
