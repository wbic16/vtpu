//! HDC Inference Packing Pattern — R23W16 Rewrite
//!
//! Weight-free hyperdimensional inference:
//! Encode coordinate → Bind with context → Similarity check
//! Uses D-Pipe HDC ops packed with S-Pipe gather and C-Pipe coordination.

use vtpu_runtime::*;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::exec;

fn main() {
    println!("═══ HDC Inference Packing Pattern ═══\n");

    let mut mem = Memory::new();

    // Seed knowledge base: 10 facts at coordinates
    for i in 1..=10u16 {
        let mut c = [1u16; 11];
        c[0] = i;
        mem.scatter_i64(&PhextCoord::new(c), i as i64 * 100);
    }

    // HDC inference program:
    // 1. Encode input (DHDENC)
    // 2. Bind with context (DHDBIND)
    // 3. Gather from memory (SGATHER)
    // 4. Compare similarity (DHDSIM)
    // 5. Permute for sequence (DHDPERM)
    let program = vec![
        // Encode r1 → r2
        SIW::new(
            DenseOp::DHDENC { rd: 2, rs: 1, width: 1024 },
            SparseOp::SGATHER { rd: 5, coord_idx: 0, width: 64 },
            CoordOp::CNOP,
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        ),
        // Bind r2 ⊕ r3 → r4 (context binding)
        SIW::new(
            DenseOp::DHDBIND { rd: 4, rs1: 2, rs2: 3 },
            SparseOp::SGATHER { rd: 6, coord_idx: 1, width: 64 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // Similarity check: r4 vs r5
        SIW::new(
            DenseOp::DHDSIM { rd: 7, rs1: 4, rs2: 5 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // Bundle accumulated knowledge: r4 | r6 → r8
        SIW::new(
            DenseOp::DHDBUND { rd: 8, rs1: 4, rs2: 6 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // Permute for temporal sequence: rotate r8 by 3
        SIW::new(
            DenseOp::DHDPERM { rd: 9, rs: 8, k: 3 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
    ];

    let mut sentron = Sentron::new(0, PhextCoord::new([1; 11]), 0, 0);
    sentron.regs.general[1] = 42;  // input value
    sentron.regs.general[3] = 99;  // context
    sentron.regs.phext[0] = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    sentron.regs.phext[1] = PhextCoord::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

    sentron.spawn(program);
    let stats = exec::run(&mut sentron, &mut mem);

    println!("  SIWs retired: {}", stats.siws_retired);
    println!("  Ops retired:  {}", stats.ops_retired);
    println!("  Ops/cycle:    {:.1}", stats.ops_per_cycle());
    println!();
    println!("  Encoded (r2):     0x{:016X}", sentron.regs.general[2] as u64);
    println!("  Bound (r4):       0x{:016X}", sentron.regs.general[4] as u64);
    println!("  Similarity (r7):  {} / 64 bits", sentron.regs.general[7]);
    println!("  Bundled (r8):     0x{:016X}", sentron.regs.general[8] as u64);
    println!("  Permuted (r9):    0x{:016X}", sentron.regs.general[9] as u64);
    println!();
    println!("  All HDC ops are REAL: encode=hash-mix, bind=XOR, bundle=OR, permute=rotate, sim=XNOR+popcount");
}
