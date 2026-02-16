//! BitNet Ternary Inference (R23W11)
//!
//! Implements BitNet b1.58 style ternary inference on the vTPU.
//! Weights are {-1, 0, 1} — no floating point needed.
//!
//! Trit packing: 2 bits per weight, 32 weights per i64.
//!   00 = 0 (zero), 01 = +1, 10 = -1
//!
//! A ternary matmul of [M×K] × [K×N] decomposes into:
//!   For each row i, col j: output[i][j] = sum_k(activation[i][k] * weight[k][j])
//!   Where weight ∈ {-1, 0, 1}, so multiply = negate/zero/identity.

use crate::pipes::DenseOp;
#[cfg(test)]
use crate::pipes::{SparseOp, CoordOp};
#[cfg(test)]
use crate::PhextCoord;
use crate::siw::SIW;
use crate::packer::{ScalarOp, pack};

/// Pack a slice of ternary weights {-1, 0, 1} into 2-bit trit format.
/// Returns packed i64 values, 32 weights per i64.
pub fn pack_trits(weights: &[i8]) -> Vec<i64> {
    let mut packed = Vec::new();
    for chunk in weights.chunks(32) {
        let mut val: u64 = 0;
        for (i, &w) in chunk.iter().enumerate() {
            let trit = match w {
                1 => 0b01u64,
                -1 => 0b10u64,
                0 => 0b00u64,
                _ => panic!("BitNet weights must be {{-1, 0, 1}}, got {}", w),
            };
            val |= trit << (i * 2);
        }
        packed.push(val as i64);
    }
    packed
}

/// Unpack trits back to weights (for verification)
pub fn unpack_trits(packed: i64, count: usize) -> Vec<i8> {
    let bits = packed as u64;
    (0..count.min(32)).map(|i| {
        match (bits >> (i * 2)) & 0x3 {
            0b01 => 1i8,
            0b10 => -1i8,
            _ => 0i8,
        }
    }).collect()
}

/// Ternary dot product (reference implementation, no vTPU)
pub fn ternary_dot(activations: &[i64], weights: &[i8]) -> i64 {
    activations.iter().zip(weights.iter()).map(|(&a, &w)| {
        a * (w as i64)
    }).sum()
}

/// Generate a SIW program for ternary matrix-vector multiply.
///
/// Computes: output[j] = sum_i(activations[i] * weights[i][j])
/// where weights are packed trits.
///
/// Uses DTERNARY for the inner loop — each op processes 32 weights at once.
/// With ternary, D-Pipe needs no FPU: pure integer add/sub/zero.
pub fn ternary_matvec_program(
    rows: usize,
    activation_regs: &[u8],  // registers holding activations
    trit_regs: &[u8],        // registers holding packed trit weights
    output_reg: u8,          // register for accumulated output
) -> Vec<SIW> {
    let mut ops = Vec::new();

    // Clear accumulator
    ops.push(ScalarOp::D(DenseOp::DMOV { rd: output_reg, imm: 0 }));

    // Accumulate: output += activation[i] * trits[i]
    for i in 0..rows.min(activation_regs.len()).min(trit_regs.len()) {
        ops.push(ScalarOp::D(DenseOp::DTACC {
            rd: output_reg,
            rs1: activation_regs[i],
            trit_reg: trit_regs[i],
        }));
    }

    pack(&ops).stream
}

/// Sparsity of a packed trit vector (fraction of zero weights)
pub fn trit_sparsity(packed: &[i64]) -> f64 {
    let mut total = 0usize;
    let mut zeros = 0usize;
    for &p in packed {
        let bits = p as u64;
        for i in 0..32 {
            total += 1;
            if (bits >> (i * 2)) & 0x3 == 0 { zeros += 1; }
        }
    }
    if total == 0 { 0.0 } else { zeros as f64 / total as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sentron::Sentron;
    #[allow(unused_imports)]
    use crate::memory::Memory;
    use crate::exec;

    #[test]
    fn pack_unpack_roundtrip() {
        let weights = vec![1, -1, 0, 1, -1, -1, 0, 0, 1, 0];
        let packed = pack_trits(&weights);
        let unpacked = unpack_trits(packed[0], weights.len());
        assert_eq!(unpacked, weights);
    }

    #[test]
    fn pack_32_weights() {
        let mut weights = vec![0i8; 32];
        weights[0] = 1;
        weights[15] = -1;
        weights[31] = 1;
        let packed = pack_trits(&weights);
        assert_eq!(packed.len(), 1);
        let unpacked = unpack_trits(packed[0], 32);
        assert_eq!(unpacked, weights);
    }

    #[test]
    fn ternary_dot_reference() {
        let activations = vec![10, 20, 30, 40];
        let weights = vec![1, -1, 0, 1];
        // 10*1 + 20*(-1) + 30*0 + 40*1 = 10 - 20 + 0 + 40 = 30
        assert_eq!(ternary_dot(&activations, &weights), 30);
    }

    #[test]
    fn dternary_op_basic() {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        // activation = 7, trits = [+1, -1, 0, ...] → 7 - 7 = 0
        let trits = pack_trits(&[1, -1]);
        s.regs.general[0] = 7;          // activation
        s.regs.general[1] = trits[0];   // packed trits

        let program = vec![
            SIW::new(DenseOp::DTERNARY { rd: 2, rs1: 0, trit_reg: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        s.spawn(program);
        exec::run_standalone(&mut s);
        // +1 and -1 → 7 + (-7) = 0
        assert_eq!(s.regs.general[2], 0);
    }

    #[test]
    fn dternary_all_positive() {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let trits = pack_trits(&[1, 1, 1, 1]);
        s.regs.general[0] = 5;
        s.regs.general[1] = trits[0];

        let program = vec![
            SIW::new(DenseOp::DTERNARY { rd: 2, rs1: 0, trit_reg: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        s.spawn(program);
        exec::run_standalone(&mut s);
        // 4 × (+5) = 20
        assert_eq!(s.regs.general[2], 20);
    }

    #[test]
    fn dtpop_counts_nonzero() {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let trits = pack_trits(&[1, 0, -1, 0, 1, -1, 0, 0]);
        s.regs.general[0] = trits[0];

        let program = vec![
            SIW::new(DenseOp::DTPOP { rd: 1, rs: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        s.spawn(program);
        exec::run_standalone(&mut s);
        // Non-zero trits: 1, -1, 1, -1 = 4
        assert_eq!(s.regs.general[1], 4);
    }

    #[test]
    fn dtacc_accumulates() {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let trits_a = pack_trits(&[1, 1]);
        let trits_b = pack_trits(&[-1, 1]);
        s.regs.general[0] = 10;           // activation
        s.regs.general[1] = trits_a[0];   // +1, +1
        s.regs.general[2] = trits_b[0];   // -1, +1
        s.regs.general[3] = 0;            // accumulator

        let program = vec![
            SIW::new(DenseOp::DTACC { rd: 3, rs1: 0, trit_reg: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DTACC { rd: 3, rs1: 0, trit_reg: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        s.spawn(program);
        exec::run_standalone(&mut s);
        // First: 10+10 = 20, Second: -10+10 = 0, Total: 20+0 = 20
        assert_eq!(s.regs.general[3], 20);
    }

    #[test]
    fn ternary_matvec_simple() {
        // 2×1 matrix-vector: [+1, -1] · [3, 7] = 3 - 7 = -4
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let trits = pack_trits(&[1, -1]);

        s.regs.general[0] = 3;
        s.regs.general[1] = 7;
        s.regs.general[4] = trits[0];  // packed weights

        // Manual: clear r5, then ternary r5 = 3*1 + 7*(-1) ... but DTERNARY
        // applies same activation to all trits. For matvec, need per-element.
        // Use DTACC with individual activations and individual trit packs.

        let trits_0 = pack_trits(&[1]);   // weight for activation[0]
        let trits_1 = pack_trits(&[-1]);  // weight for activation[1]
        s.regs.general[4] = trits_0[0];
        s.regs.general[5] = trits_1[0];

        let program = vec![
            SIW::new(DenseOp::DMOV { rd: 6, imm: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DTACC { rd: 6, rs1: 0, trit_reg: 4 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DTACC { rd: 6, rs1: 1, trit_reg: 5 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        s.spawn(program);
        exec::run_standalone(&mut s);
        // 3*(+1) + 7*(-1) = 3 - 7 = -4
        assert_eq!(s.regs.general[6], -4);
    }

    #[test]
    fn sparsity_measurement() {
        let weights = vec![0, 0, 1, 0, -1, 0, 0, 0]; // 75% sparse
        let packed = pack_trits(&weights);
        let sparsity = trit_sparsity(&packed);
        // 32 trits per packed i64, only 8 weights given → rest are 0
        // 30 zeros + 2 non-zero = 30/32 = 0.9375... but we pack 8 weights
        // Actually pack_trits pads with 0, so 30 zeros out of 32
        assert!(sparsity > 0.9);
    }

    #[test]
    fn ternary_matvec_program_runs() {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let trits_0 = pack_trits(&[1]);
        let trits_1 = pack_trits(&[1]);

        s.regs.general[0] = 10;  // activation 0
        s.regs.general[1] = 20;  // activation 1
        s.regs.general[4] = trits_0[0];
        s.regs.general[5] = trits_1[0];

        let program = ternary_matvec_program(2, &[0, 1], &[4, 5], 6);
        s.spawn(program);
        let stats = exec::run_standalone(&mut s);
        // 10*(+1) + 20*(+1) = 30
        assert_eq!(s.regs.general[6], 30);
        assert!(stats.ops_retired > 0);
    }

    #[test]
    fn bitnet_158_pattern() {
        // BitNet b1.58: ~37% of weights are zero on average
        // Simulate a small "layer": 8 activations × 8 packed trit columns
        let weights: Vec<i8> = vec![
            1, -1, 0, 1, -1, 0, 0, 1,   // column 0
        ];
        let activations: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let expected = ternary_dot(&activations, &weights);
        // 1*1 + 2*(-1) + 3*0 + 4*1 + 5*(-1) + 6*0 + 7*0 + 8*1
        // = 1 - 2 + 0 + 4 - 5 + 0 + 0 + 8 = 6
        assert_eq!(expected, 6);

        let packed = pack_trits(&weights);
        let sparsity = trit_sparsity(&packed);
        assert!(sparsity > 0.7); // High sparsity due to padding
    }
}
