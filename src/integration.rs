//! Integration Tests — R23W13
//!
//! End-to-end tests that prove the vTPU works as a system,
//! not just as individual modules passing unit tests.
//!
//! Each test builds a real workload: encode → store → query → route → verify.

use crate::hdc::{HyperVector, AssociativeMemory, HDC_DEFAULT_WIDTH};
use crate::phext_coord::PhextCoord;
use crate::memory::Memory;
use crate::sentron::Sentron;
use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp, CoordOp};
use crate::exec;
use crate::packer::{ScalarOp, pack};
use crate::bitnet;

// ═══════════════════════════════════════════════════════════════
// Test 1: End-to-end scroll storage and retrieval
// ═══════════════════════════════════════════════════════════════

/// Store a value at a phext coordinate, retrieve it, verify it matches.
/// This is the most basic "does the system work" test.
#[cfg(test)]
fn store_and_retrieve(coord: [u16; 11], value: i64) -> bool {
    let pc = PhextCoord::new(coord);
    let mut mem = Memory::new();

    // Store
    mem.scatter_i64(&pc, value);

    // Retrieve
    let got = mem.gather_i64(&pc);
    got == value
}

// ═══════════════════════════════════════════════════════════════
// Test 2: HDC encode → associative recall
// ═══════════════════════════════════════════════════════════════

/// Encode 10 phext coordinates as hypervectors, store in associative memory,
/// query with a known coordinate, verify we get the right one back.
#[cfg(test)]
fn hdc_associative_recall() -> bool {
    let mut amem = AssociativeMemory::new();
    let coords: Vec<[u16; 11]> = (1..=10).map(|i| {
        let mut c = [1u16; 11];
        c[0] = i;
        c
    }).collect();

    for &c in &coords {
        amem.store(c, HDC_DEFAULT_WIDTH);
    }

    // Query: coord [5,1,1,...] should return itself as nearest
    let query_coord = [5u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    let query_hv = HyperVector::from_coord(&query_coord, HDC_DEFAULT_WIDTH);
    let result = amem.query_nearest(&query_hv);

    match result {
        Some((found, sim)) => found == query_coord && sim > 0.9,
        None => false,
    }
}

// ═══════════════════════════════════════════════════════════════
// Test 3: Full sentron execution pipeline
// ═══════════════════════════════════════════════════════════════

/// Build a program that:
/// 1. Loads two values from phext coordinates (SGATHER)
/// 2. Adds them (DADD)
/// 3. Stores the result (SSCATTR)
/// 4. Verifies the stored result
#[cfg(test)]
fn sentron_load_add_store() -> bool {
    let coord_a = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    let coord_b = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]);
    let coord_out = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3]);

    let mut mem = Memory::new();
    mem.scatter_i64(&coord_a, 42);
    mem.scatter_i64(&coord_b, 58);

    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    // Set phext registers for coordinate addressing
    sentron.regs.phext[0] = coord_a;
    sentron.regs.phext[1] = coord_b;
    sentron.regs.phext[2] = coord_out;

    let program = vec![
        // Gather value from coord_a → r1
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // Gather value from coord_b → r2
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 2, coord_idx: 1, width: 64 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // Add r1 + r2 → r3
        SIW::new(
            DenseOp::DADD { rd: 3, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // Scatter r3 → coord_out
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 2, rs: 3, width: 64 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
    ];

    sentron.spawn(program);
    let stats = exec::run(&mut sentron, &mut mem);

    // Verify
    let result = mem.gather_i64(&coord_out);
    result == 100 && stats.siws_retired == 4
}

// ═══════════════════════════════════════════════════════════════
// Test 4: SIW packer → executor round-trip
// ═══════════════════════════════════════════════════════════════

/// Generate scalar ops, pack into SIWs, execute, verify result.
/// This tests the compiler → executor pipeline.
#[cfg(test)]
fn pack_and_execute() -> bool {
    // Scalar ops: r1 = 10, r2 = 20, r3 = r1 + r2
    let ops = vec![
        ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 10 }),
        ScalarOp::D(DenseOp::DMOV { rd: 2, imm: 20 }),
        ScalarOp::D(DenseOp::DADD { rd: 3, rs1: 1, rs2: 2 }),
    ];

    let packed = pack(&ops);

    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.spawn(packed.stream);
    let _stats = exec::run_standalone(&mut sentron);

    sentron.regs.general[3] == 30
}

// ═══════════════════════════════════════════════════════════════
// Test 5: Ternary inference — BitNet matmul through executor
// ═══════════════════════════════════════════════════════════════

/// Pack ternary weights, execute DTERNARY ops, verify output.
#[cfg(test)]
fn ternary_inference_pipeline() -> bool {
    // Weights: [1, -1, 1, 0, 1] (padded to 32)
    let mut weights = vec![0i8; 32];
    weights[0] = 1;
    weights[1] = -1;
    weights[2] = 1;
    weights[3] = 0;
    weights[4] = 1;

    let packed = bitnet::pack_trits(&weights);
    assert_eq!(packed.len(), 1);

    // Activation = 7
    // Expected: 7*1 + 7*(-1) + 7*1 + 7*0 + 7*1 = 7 - 7 + 7 + 0 + 7 = 14
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.regs.general[1] = 7;           // activation
    sentron.regs.general[2] = packed[0];   // packed trits

    let program = vec![
        SIW::new(
            DenseOp::DTERNARY { rd: 3, rs1: 1, trit_reg: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
    ];

    sentron.spawn(program);
    exec::run_standalone(&mut sentron);

    sentron.regs.general[3] == 14
}

// ═══════════════════════════════════════════════════════════════
// Test 6: Multi-sentron coordinate partitioning
// ═══════════════════════════════════════════════════════════════

/// Two sentrons work on different coordinate regions.
/// Sentron 0 sums values in library 1, sentron 1 sums values in library 2.
/// Verifies they don't interfere.
#[cfg(test)]
fn multi_sentron_partition() -> bool {
    let mut mem = Memory::new();

    // Seed data: library 1 has values 1-3, library 2 has values 10-30
    let coords_1: Vec<PhextCoord> = (1..=3).map(|i| {
        let mut c = [1u16; 11];
        c[10] = i; // vary scroll dimension
        PhextCoord::new(c)
    }).collect();

    let coords_2: Vec<PhextCoord> = (1..=3).map(|i| {
        let mut c = [2u16; 11]; // library 2
        c[10] = i;
        PhextCoord::new(c)
    }).collect();

    for (i, c) in coords_1.iter().enumerate() {
        mem.scatter_i64(c, (i + 1) as i64);
    }
    for (i, c) in coords_2.iter().enumerate() {
        mem.scatter_i64(c, ((i + 1) * 10) as i64);
    }

    // Sentron 0: gather and sum library 1
    let mut s0 = Sentron::new(0, PhextCoord::new([1; 11]), 0, 0);
    s0.regs.phext[0] = coords_1[0];
    s0.regs.phext[1] = coords_1[1];
    s0.regs.phext[2] = coords_1[2];
    s0.spawn(vec![
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 2, coord_idx: 1, width: 64 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 3, coord_idx: 2, width: 64 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 4, rs1: 1, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 5, rs1: 4, rs2: 3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ]);
    exec::run(&mut s0, &mut mem);

    // Sentron 1: gather and sum library 2
    let mut s1 = Sentron::new(1, PhextCoord::new([2; 11]), 0, 1);
    s1.regs.phext[0] = coords_2[0];
    s1.regs.phext[1] = coords_2[1];
    s1.regs.phext[2] = coords_2[2];
    s1.spawn(vec![
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 2, coord_idx: 1, width: 64 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 3, coord_idx: 2, width: 64 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 4, rs1: 1, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 5, rs1: 4, rs2: 3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ]);
    exec::run(&mut s1, &mut mem);

    // Library 1: 1+2+3 = 6, Library 2: 10+20+30 = 60
    s0.regs.general[5] == 6 && s1.regs.general[5] == 60
}

// ═══════════════════════════════════════════════════════════════
// Test 7: HDC similarity via DHDSIM instruction
// ═══════════════════════════════════════════════════════════════

/// Verify DHDSIM computes real hamming similarity on register values.
#[cfg(test)]
fn dhdsim_real_similarity() -> bool {
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    // Identical values → max similarity (all bits match)
    sentron.regs.general[1] = 0x_DEAD_BEEF_CAFE_BABE_u64 as i64;
    sentron.regs.general[2] = 0x_DEAD_BEEF_CAFE_BABE_u64 as i64;

    let program = vec![
        SIW::new(
            DenseOp::DHDSIM { rd: 3, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
    ];

    sentron.spawn(program);
    exec::run_standalone(&mut sentron);

    // Identical → XNOR is all 1s → popcount = 64
    sentron.regs.general[3] == 64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e2e_store_retrieve() {
        assert!(store_and_retrieve([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], 42));
        assert!(store_and_retrieve([2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1], 2130));
        assert!(store_and_retrieve([9, 9, 9, 1, 5, 2, 7, 7, 7, 1, 1], -1));
    }

    #[test]
    fn e2e_hdc_recall() {
        assert!(hdc_associative_recall());
    }

    #[test]
    fn e2e_sentron_load_add_store() {
        assert!(sentron_load_add_store());
    }

    #[test]
    fn e2e_pack_and_execute() {
        assert!(pack_and_execute());
    }

    #[test]
    fn e2e_ternary_inference() {
        assert!(ternary_inference_pipeline());
    }

    #[test]
    fn e2e_multi_sentron_partition() {
        assert!(multi_sentron_partition());
    }

    #[test]
    fn e2e_dhdsim_similarity() {
        assert!(dhdsim_real_similarity());
    }
}
