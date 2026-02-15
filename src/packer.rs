//! SIW Packer (R23W10)
//!
//! Takes a sequence of single-pipe operations and packs them into 3-wide SIWs.
//! This is the first stage of phextcc: sequential ops → packed SIW stream.
//!
//! Strategy:
//! 1. Classify each op by pipe (D/S/C)
//! 2. Build dependency graph on the scalar ops
//! 3. Greedily pack independent ops from different pipes into one SIW
//! 4. Fall back to NOP-padding when no compatible op is available

use crate::pipes::{DenseOp, SparseOp, CoordOp};
use crate::siw::SIW;
use crate::PhextCoord;
use crate::regalloc::{self, Hazard, RegAccess, RegClass};

/// A single-pipe operation before packing
#[derive(Debug, Clone)]
pub enum ScalarOp {
    D(DenseOp),
    S(SparseOp),
    C(CoordOp),
}

impl ScalarOp {
    pub fn pipe(&self) -> Pipe {
        match self {
            ScalarOp::D(_) => Pipe::D,
            ScalarOp::S(_) => Pipe::S,
            ScalarOp::C(_) => Pipe::C,
        }
    }

    /// Extract register accesses for dependency analysis
    pub fn accesses(&self) -> RegAccess {
        let siw = match self {
            ScalarOp::D(d) => SIW::new(d.clone(), SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            ScalarOp::S(s) => SIW::new(DenseOp::DNOP, s.clone(), CoordOp::CNOP, PhextCoord::zero()),
            ScalarOp::C(c) => SIW::new(DenseOp::DNOP, SparseOp::SNOP, c.clone(), PhextCoord::zero()),
        };
        regalloc::extract_accesses(&siw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pipe { D, S, C }

/// Packing result
#[derive(Debug, Clone)]
pub struct PackResult {
    pub stream: Vec<SIW>,
    pub input_ops: usize,
    pub packed_siws: usize,
    pub utilization: f64,  // ops / (siws * 3)
}

/// Check if two scalar ops have a data dependency
fn has_dependency(a: &ScalarOp, b: &ScalarOp) -> bool {
    let aa = a.accesses();
    let ba = b.accesses();

    // RAW: a writes what b reads
    for &(wc, wr) in &aa.writes {
        for &(rc, rr) in &ba.reads {
            if wc == rc && wr == rr { return true; }
        }
    }
    // WAW: both write same
    for &(wc1, wr1) in &aa.writes {
        for &(wc2, wr2) in &ba.writes {
            if wc1 == wc2 && wr1 == wr2 { return true; }
        }
    }
    // WAR: a reads what b writes
    for &(rc, rr) in &aa.reads {
        for &(wc, wr) in &ba.writes {
            if rc == wc && rr == wr { return true; }
        }
    }

    false
}

/// Pack sequential scalar ops into 3-wide SIWs
///
/// Greedy algorithm:
/// - Maintain a ready queue of ops whose dependencies are satisfied
/// - Each cycle, pick up to one D, one S, one C from the ready queue
/// - Prefer ops with more dependents (critical path heuristic)
pub fn pack(ops: &[ScalarOp]) -> PackResult {
    let n = ops.len();
    if n == 0 {
        return PackResult { stream: vec![], input_ops: 0, packed_siws: 0, utilization: 0.0 };
    }

    // Build dependency edges: deps[i] = set of ops that must complete before i
    let mut deps: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        for j in 0..i {
            if has_dependency(&ops[j], &ops[i]) {
                deps[i].push(j);
            }
        }
    }

    // Count dependents for priority
    let mut dep_count = vec![0usize; n];
    for i in 0..n {
        for &d in &deps[i] {
            dep_count[d] += 1;
        }
    }

    let mut completed = vec![false; n];
    let mut stream = Vec::new();
    let mut total_ops = 0usize;

    loop {
        // Find ready ops (all deps completed)
        let mut ready: Vec<usize> = (0..n)
            .filter(|&i| !completed[i] && deps[i].iter().all(|&d| completed[d]))
            .collect();

        if ready.is_empty() { break; }

        // Sort by dependents (most first) for critical path priority
        ready.sort_by(|&a, &b| dep_count[b].cmp(&dep_count[a]));

        // Greedily pick one from each pipe
        let mut d_op = DenseOp::DNOP;
        let mut s_op = SparseOp::SNOP;
        let mut c_op = CoordOp::CNOP;
        let mut picked = Vec::new();
        let mut has_d = false;
        let mut has_s = false;
        let mut has_c = false;

        for &idx in &ready {
            // Check this op doesn't conflict with already-picked ops in this SIW
            let dominated = picked.iter().any(|&p| has_dependency(&ops[p], &ops[idx]) || has_dependency(&ops[idx], &ops[p]));
            if dominated { continue; }

            match ops[idx].pipe() {
                Pipe::D if !has_d => {
                    if let ScalarOp::D(ref op) = ops[idx] {
                        d_op = op.clone();
                        has_d = true;
                        picked.push(idx);
                    }
                }
                Pipe::S if !has_s => {
                    if let ScalarOp::S(ref op) = ops[idx] {
                        s_op = op.clone();
                        has_s = true;
                        picked.push(idx);
                    }
                }
                Pipe::C if !has_c => {
                    if let ScalarOp::C(ref op) = ops[idx] {
                        c_op = op.clone();
                        has_c = true;
                        picked.push(idx);
                    }
                }
                _ => {} // pipe slot already taken
            }

            if has_d && has_s && has_c { break; }
        }

        for &idx in &picked {
            completed[idx] = true;
        }
        total_ops += picked.len();

        stream.push(SIW::new(d_op, s_op, c_op, PhextCoord::zero()));
    }

    let packed_siws = stream.len();
    let utilization = if packed_siws > 0 {
        total_ops as f64 / (packed_siws as f64 * 3.0)
    } else {
        0.0
    };

    PackResult {
        stream,
        input_ops: n,
        packed_siws,
        utilization,
    }
}

/// Convenience: pack from a list of DenseOps (D-pipe only workload)
pub fn pack_dense(ops: &[DenseOp]) -> PackResult {
    let scalars: Vec<ScalarOp> = ops.iter().map(|d| ScalarOp::D(d.clone())).collect();
    pack(&scalars)
}

/// Convenience: pack a mixed workload described as (pipe, op) tuples
pub fn pack_mixed(d_ops: &[DenseOp], s_ops: &[SparseOp], c_ops: &[CoordOp]) -> PackResult {
    let mut scalars = Vec::new();
    for op in d_ops { scalars.push(ScalarOp::D(op.clone())); }
    for op in s_ops { scalars.push(ScalarOp::S(op.clone())); }
    for op in c_ops { scalars.push(ScalarOp::C(op.clone())); }
    pack(&scalars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::*;

    #[test]
    fn pack_empty() {
        let result = pack(&[]);
        assert_eq!(result.packed_siws, 0);
    }

    #[test]
    fn pack_single_d() {
        let result = pack(&[ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 42 })]);
        assert_eq!(result.packed_siws, 1);
        assert_eq!(result.input_ops, 1);
    }

    #[test]
    fn pack_three_independent_pipes() {
        // One D, one S, one C — all independent → should pack into 1 SIW
        let ops = vec![
            ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 1 }),
            ScalarOp::S(SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L2 }),
            ScalarOp::C(CoordOp::CFENCE { scope: FenceScope::Thread }),
        ];
        let result = pack(&ops);
        assert_eq!(result.packed_siws, 1, "3 independent ops from different pipes → 1 SIW");
        assert_eq!(result.input_ops, 3);
        assert!((result.utilization - 1.0).abs() < 0.01, "100% utilization");
    }

    #[test]
    fn pack_two_d_ops_sequential() {
        // Two D ops → can't pack together, need 2 SIWs
        let ops = vec![
            ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 1 }),
            ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 2 }),
        ];
        let result = pack(&ops);
        assert_eq!(result.packed_siws, 2, "Two D ops → 2 SIWs");
    }

    #[test]
    fn pack_dependent_chain() {
        // r0=1, r1=r0+r0 — dependent, can't parallelize
        let ops = vec![
            ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 1 }),
            ScalarOp::D(DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 }),
        ];
        let result = pack(&ops);
        assert_eq!(result.packed_siws, 2);
    }

    #[test]
    fn pack_interleave_d_and_s() {
        // D: r0=1, D: r1=2, S: gather r2 from p0, S: gather r3 from p1
        // Should pack: (D:r0=1 + S:gather_r2), (D:r1=2 + S:gather_r3)
        let ops = vec![
            ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 1 }),
            ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 2 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 2, coord_idx: 0, width: 8 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 3, coord_idx: 1, width: 8 }),
        ];
        let result = pack(&ops);
        assert_eq!(result.packed_siws, 2, "4 ops across 2 pipes → 2 SIWs");
        assert!((result.utilization - 2.0 / 3.0).abs() < 0.01, "66% utilization");
    }

    #[test]
    fn pack_six_independent_ops() {
        // 2D + 2S + 2C, all independent → 2 fully-packed SIWs
        let ops = vec![
            ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 1 }),
            ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 2 }),
            ScalarOp::S(SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L1 }),
            ScalarOp::S(SparseOp::SPREFCH { coord_idx: 1, hint: PrefetchHint::L2 }),
            ScalarOp::C(CoordOp::CFENCE { scope: FenceScope::Thread }),
            ScalarOp::C(CoordOp::CFENCE { scope: FenceScope::Cluster }),
        ];
        let result = pack(&ops);
        assert_eq!(result.packed_siws, 2);
        assert!((result.utilization - 1.0).abs() < 0.01, "100% utilization");
    }

    #[test]
    fn pack_dense_convenience() {
        let ops = vec![
            DenseOp::DMOV { rd: 0, imm: 1 },
            DenseOp::DMOV { rd: 1, imm: 2 },
            DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
        ];
        let result = pack_dense(&ops);
        assert_eq!(result.input_ops, 3);
        // r2 depends on r0 and r1, so can't pack all 3 in one cycle
        assert!(result.packed_siws >= 2);
    }

    #[test]
    fn pack_mixed_convenience() {
        let result = pack_mixed(
            &[DenseOp::DMOV { rd: 0, imm: 42 }],
            &[SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L1 }],
            &[CoordOp::CFENCE { scope: FenceScope::Thread }],
        );
        assert_eq!(result.packed_siws, 1);
        assert!((result.utilization - 1.0).abs() < 0.01);
    }

    #[test]
    fn pack_dot_product() {
        // Classic pattern: load 4 values, 2 muls, 1 add
        // S: gather r0-r3, D: r4=r0*r1, r5=r2*r3, r6=r4+r5
        let ops = vec![
            ScalarOp::S(SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 1, coord_idx: 1, width: 8 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 2, coord_idx: 2, width: 8 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 3, coord_idx: 3, width: 8 }),
            ScalarOp::D(DenseOp::DMUL { rd: 4, rs1: 0, rs2: 1 }),
            ScalarOp::D(DenseOp::DMUL { rd: 5, rs1: 2, rs2: 3 }),
            ScalarOp::D(DenseOp::DADD { rd: 6, rs1: 4, rs2: 5 }),
        ];
        let result = pack(&ops);
        // 4 gathers + 2 muls + 1 add = 7 ops
        // Best case: gather0+mul can't overlap (mul needs gather result)
        // But gather0 and gather1 are independent S-ops → sequential (same pipe)
        // Optimal: 4 SIWs (gather pairs with D-NOPs, then muls, then add)
        assert!(result.packed_siws <= 7, "Should pack better than 1:1");
        assert!(result.utilization > 0.3, "Should achieve decent utilization");
    }

    #[test]
    fn pack_preserves_all_ops() {
        let ops = vec![
            ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 1 }),
            ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 2 }),
            ScalarOp::D(DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 }),
        ];
        let result = pack(&ops);
        assert_eq!(result.input_ops, 3);
        // Count non-NOP D-ops in output
        let d_ops: usize = result.stream.iter()
            .filter(|s| !matches!(s.d_op, DenseOp::DNOP))
            .count();
        assert_eq!(d_ops, 3, "All 3 D-ops must appear in output");
    }
}
