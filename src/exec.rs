//! vTPU Executor — steps a sentron through its SIW stream
//!
//! Phase 0: interpreter. Executes SIW operations against the sentron's
//! register file. Measures ops/cycle assuming ideal 3-wide retirement.

use crate::pipes::{DenseOp, SparseOp, CoordOp, ReductionOp};
use crate::memory::Memory;
use crate::phext_coord::PhextCoord;
use crate::sentron::{Sentron, SentronState};
use crate::siw::SIW;

/// Execution statistics for a sentron run
#[derive(Debug, Clone, Default)]
pub struct ExecStats {
    pub siws_retired: u64,
    pub ops_retired: u64,
    pub cycles: u64,
    pub d_ops: u64,
    pub s_ops: u64,
    pub c_ops: u64,
    pub d_nops: u64,
    pub s_nops: u64,
    pub c_nops: u64,
    /// Sentron flux: L1 norm of register-state delta per SIW stream.
    /// Measures how much the sentron's general register file actually moved.
    /// High flux = processing novel data; low flux = converging / fixed-point.
    pub flux_total: f64,
    /// Vak histogram: how many SIWs retired at each dominant VakLevel.
    /// Indexed by VakLevel ordinal: [Para, Pashyanti, Madhyama, Vaikhara].
    /// Para+Pashyanti = Light mode (S-pipe heavy); Madhyama+Vaikhara = Story mode (D-pipe heavy).
    pub vak_histogram: [u64; 4],
    /// Total Spanda oscillation cycles accumulated across the NeuronLayer.
    pub spanda_cycles: u64,
}

impl ExecStats {
    /// Ops per cycle (the north star KPI)
    pub fn ops_per_cycle(&self) -> f64 {
        if self.cycles == 0 { return 0.0; }
        self.ops_retired as f64 / self.cycles as f64
    }

    /// Pipe utilization: fraction of pipe-slots that were active
    pub fn utilization(&self) -> f64 {
        let total_slots = self.siws_retired * 3;
        if total_slots == 0 { return 0.0; }
        self.ops_retired as f64 / total_slots as f64
    }

    pub fn d_utilization(&self) -> f64 {
        let total = self.d_ops + self.d_nops;
        if total == 0 { return 0.0; }
        self.d_ops as f64 / total as f64
    }

    pub fn s_utilization(&self) -> f64 {
        let total = self.s_ops + self.s_nops;
        if total == 0 { return 0.0; }
        self.s_ops as f64 / total as f64
    }

    pub fn c_utilization(&self) -> f64 {
        let total = self.c_ops + self.c_nops;
        if total == 0 { return 0.0; }
        self.c_ops as f64 / total as f64
    }

    /// Flux per SIW: average register-state L1-norm change per retired SIW.
    /// D-heavy streams show high flux (lots of arithmetic). S-heavy streams
    /// show lower flux (coordinate operations, less register churn).
    pub fn flux_per_siw(&self) -> f64 {
        if self.siws_retired == 0 { return 0.0; }
        self.flux_total / self.siws_retired as f64
    }

    /// Dominant vak level across the stream (most common NeuronLayer output).
    pub fn dominant_vak(&self) -> &'static str {
        let names = ["Para", "Pashyanti", "Madhyama", "Vaikhara"];
        let max_idx = self.vak_histogram
            .iter()
            .enumerate()
            .max_by_key(|(_, &v)| v)
            .map(|(i, _)| i)
            .unwrap_or(3);
        names[max_idx]
    }

    /// Light-mode fraction: Para + Pashyanti SIWs / total.
    /// > 0.5 means S-pipe (sparse/associative) was dominant.
    pub fn light_fraction(&self) -> f64 {
        if self.siws_retired == 0 { return 0.0; }
        let light = self.vak_histogram[0] + self.vak_histogram[1];
        light as f64 / self.siws_retired as f64
    }
}

/// Apply packed ternary weights to an activation value.
/// Trits are packed 2 bits each: 00=zero, 01=+1, 10=-1 (32 trits per u64).
/// Returns: sum of (activation * trit[i]) for each non-zero trit.
#[inline]
fn ternary_apply(activation: i64, trits: u64) -> i64 {
    let mut result = 0i64;
    for i in 0..32 {
        let trit = (trits >> (i * 2)) & 0x3;
        match trit {
            0b01 => result += activation,  // +1
            0b10 => result -= activation,  // -1
            _ => {}                         // 0 or unused
        }
    }
    result
}

/// Execute one SIW against a sentron's register file. Returns active op count (0-3).
/// Legacy triple-match dispatch (pre-W19). Preserved for reference/comparison.
/// The OctaWire dispatch (`exec_siw_octawire`) is now the primary execution path.
#[allow(dead_code)]
fn exec_siw(sentron: &mut Sentron, siw: &SIW, mem: &mut Memory) -> u8 {
    let mut active = 0u8;

    // ── D-Pipe ──
    match siw.d_op {
        DenseOp::DNOP => {}
        DenseOp::DFMA { rd, rs1, rs2, rs3 } => {
            let r = &sentron.regs.general;
            let result = r[rs1 as usize]
                .wrapping_mul(r[rs2 as usize])
                .wrapping_add(r[rs3 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::DADD { rd, rs1, rs2 } => {
            let result = sentron.regs.general[rs1 as usize]
                .wrapping_add(sentron.regs.general[rs2 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::DSUB { rd, rs1, rs2 } => {
            let result = sentron.regs.general[rs1 as usize]
                .wrapping_sub(sentron.regs.general[rs2 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::DMUL { rd, rs1, rs2 } => {
            let result = sentron.regs.general[rs1 as usize]
                .wrapping_mul(sentron.regs.general[rs2 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::DCMP { rd, rs1, rs2 } => {
            let a = sentron.regs.general[rs1 as usize];
            let b = sentron.regs.general[rs2 as usize];
            sentron.regs.general[rd as usize] = match a.cmp(&b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };
            active += 1;
        }
        DenseOp::DRED { rd, rs1, op } => {
            let val = sentron.regs.general[rs1 as usize];
            sentron.regs.general[rd as usize] = match op {
                ReductionOp::Sum | ReductionOp::Max | ReductionOp::Min => val,
                ReductionOp::And => val,
                ReductionOp::Or => val,
                ReductionOp::Xor => val,
            };
            active += 1;
        }
        DenseOp::DSEL { rd, rs1, rs2, flags } => {
            let cond = sentron.regs.general[flags as usize];
            sentron.regs.general[rd as usize] = if cond != 0 {
                sentron.regs.general[rs1 as usize]
            } else {
                sentron.regs.general[rs2 as usize]
            };
            active += 1;
        }
        DenseOp::DMOV { rd, imm } => {
            sentron.regs.general[rd as usize] = imm;
            active += 1;
        }
        DenseOp::DHDENC { rd, rs, width } => {
            // Encode: hash the value into a pseudo-random hypervector word
            // Uses a simple but deterministic bit-mixing function
            let val = sentron.regs.general[rs as usize] as u64;
            let w = width as u64 | 1; // ensure odd for mixing
            let mixed = val.wrapping_mul(0x9E3779B97F4A7C15) // golden ratio hash
                .wrapping_add(w.wrapping_mul(0x517CC1B727220A95))
                .rotate_left(32);
            sentron.regs.general[rd as usize] = mixed as i64;
            active += 1;
        }
        DenseOp::DHDBIND { rd, rs1, rs2 } => {
            // Bind: XOR (the fundamental HDC binding operation)
            let a = sentron.regs.general[rs1 as usize];
            let b = sentron.regs.general[rs2 as usize];
            sentron.regs.general[rd as usize] = a ^ b;
            active += 1;
        }
        DenseOp::DHDBUND { rd, rs1, rs2 } => {
            // Bundle: majority vote on bits (approximated as OR for 2 inputs)
            // True bundling needs 3+ inputs; for 2, OR preserves set bits
            let a = sentron.regs.general[rs1 as usize];
            let b = sentron.regs.general[rs2 as usize];
            sentron.regs.general[rd as usize] = a | b;
            active += 1;
        }
        DenseOp::DHDPERM { rd, rs, k } => {
            // Permute: rotate bits by k positions (sequence encoding)
            let val = sentron.regs.general[rs as usize] as u64;
            let rotated = val.rotate_left(k as u32);
            sentron.regs.general[rd as usize] = rotated as i64;
            active += 1;
        }
        DenseOp::DHDSIM { rd, rs1, rs2 } => {
            let a = sentron.regs.general[rs1 as usize];
            let b = sentron.regs.general[rs2 as usize];
            sentron.regs.general[rd as usize] = (!(a ^ b)).count_ones() as i64;
            active += 1;
        }
        DenseOp::DTERNARY { rd, rs1, trit_reg } => {
            let activation = sentron.regs.general[rs1 as usize];
            let trits = sentron.regs.general[trit_reg as usize] as u64;
            sentron.regs.general[rd as usize] = ternary_apply(activation, trits);
            active += 1;
        }
        DenseOp::DTPOP { rd, rs } => {
            let trits = sentron.regs.general[rs as usize] as u64;
            // Count non-zero trits (2 bits each, 32 trits per i64)
            let mut count = 0i64;
            for i in 0..32 {
                if (trits >> (i * 2)) & 0x3 != 0 { count += 1; }
            }
            sentron.regs.general[rd as usize] = count;
            active += 1;
        }
        DenseOp::DTACC { rd, rs1, trit_reg } => {
            let activation = sentron.regs.general[rs1 as usize];
            let trits = sentron.regs.general[trit_reg as usize] as u64;
            sentron.regs.general[rd as usize] += ternary_apply(activation, trits);
            active += 1;
        }
    }

    // ── S-Pipe ──
    match siw.s_op {
        SparseOp::SNOP => {}
        SparseOp::SINDEX { rd, base, offset, dim } => {
            let base_coord = sentron.regs.phext[base as usize];
            let current = base_coord.get_dim(dim) as i32;
            let new_val = (current + offset).max(0) as u16;
            let mut new_coord = base_coord;
            new_coord.set_dim(dim, new_val);
            sentron.regs.phext[rd as usize] = new_coord;
            active += 1;
        }
        SparseOp::SPREFCH { .. } => { active += 1; }
        SparseOp::SGATHER { rd, coord_idx, .. } => {
            let coord = sentron.regs.phext[coord_idx as usize];
            sentron.regs.general[rd as usize] = mem.gather_i64(&coord);
            active += 1;
        }
        SparseOp::SSCATTR { coord_idx, rs, .. } => {
            let coord = sentron.regs.phext[coord_idx as usize];
            let val = sentron.regs.general[rs as usize];
            mem.scatter_i64(&coord, val);
            active += 1;
        }
        SparseOp::SDEDUP { rd, rs, .. } => {
            sentron.regs.general[rd as usize] = sentron.regs.general[rs as usize];
            active += 1;
        }
        SparseOp::SFLUSH { .. } => { active += 1; }
        SparseOp::SALLOC { rd, size, .. } => {
            sentron.regs.general[rd as usize] = size as i64;
            active += 1;
        }
        SparseOp::SFREE { .. } => { active += 1; }
        SparseOp::SASSOC { rd, .. } => {
            // Store the coordinate from phext register 0 into associative memory
            let coord = sentron.regs.phext[0];
            let idx = sentron.assoc.store(&coord);
            sentron.regs.general[rd as usize] = idx as i64;
            active += 1;
        }
        SparseOp::SROUTE { rd, .. } => {
            // Query nearest match for coordinate in phext register 0
            let coord = sentron.regs.phext[0];
            let (sim, hash) = sentron.assoc.route(&coord);
            sentron.regs.general[rd as usize] = sim; // similarity in rd
            if (rd as usize) + 1 < 16 {
                sentron.regs.general[rd as usize + 1] = hash; // hash in rd+1
            }
            active += 1;
        }
        SparseOp::SNEIGHBR { rd, .. } => {
            // Count neighbors above threshold (threshold from r1 as fixed-point /1000)
            let coord = sentron.regs.phext[0];
            let threshold = sentron.regs.general[1] as f64 / 1000.0;
            let count = sentron.assoc.neighbors(&coord, threshold);
            sentron.regs.general[rd as usize] = count;
            active += 1;
        }
    }

    // ── C-Pipe ──
    match siw.c_op {
        CoordOp::CNOP => {}
        CoordOp::CPACK { rd, rs1, rs2, .. } => {
            let a = sentron.regs.general[rs1 as usize].to_le_bytes();
            let b = sentron.regs.general[rs2 as usize].to_le_bytes();
            let msg = &mut sentron.regs.message[rd as usize];
            msg[..8].copy_from_slice(&a);
            msg[8..16].copy_from_slice(&b);
            active += 1;
        }
        CoordOp::CROUTE { .. } => { active += 1; }
        CoordOp::CSEND { .. } => { active += 1; }
        CoordOp::CRECV { rd, .. } => {
            // Pop from inbox if available, otherwise 0
            sentron.regs.general[rd as usize] = match sentron.inbox.pop() {
                Some((_sender, value)) => value,
                None => 0,
            };
            active += 1;
        }
        CoordOp::CBAR { .. } => { active += 1; }
        CoordOp::CFENCE { .. } => { active += 1; }
        CoordOp::CREDUCE { rd, rs, op, .. } => {
            let val = sentron.regs.general[rs as usize];
            sentron.regs.general[rd as usize] = match op {
                ReductionOp::Sum | ReductionOp::Max | ReductionOp::Min |
                ReductionOp::And | ReductionOp::Or | ReductionOp::Xor => val,
            };
            active += 1;
        }
        CoordOp::CCAST { .. } => { active += 1; }
        CoordOp::CSLICE { .. } | CoordOp::CFANOUT { .. } => { active += 1; }
        CoordOp::CMERGE { rd, .. } => {
            sentron.regs.general[rd as usize] = 0;
            active += 1;
        }
    }

    active
}

/// OctaWire dispatch: execute one SIW via indexed family dispatch.
///
/// Replaces triple match statements with 3 array lookups + 3 indirect calls.
/// LLVM sees simple u8 indexing → can generate jump tables + vectorize.
///
/// 2×4 wiring: each pipe-neuron has 4 op-families × 2 directions = 8 wires.
/// Pre-filled d_fam/s_fam/c_fam fields eliminate runtime op classification.
#[inline(always)]
pub fn exec_siw_octawire(sentron: &mut Sentron, siw: &SIW, mem: &mut Memory) -> u8 {
    let mut active = 0u8;

    // If the SIW carries a non-zero phext coordinate, auto-load into phext[0].
    // This enables per-instruction addressing: each SIW targets its own coordinate.
    // Zero phext_addr means "use whatever phext[0] was pre-set" (existing test behavior).
    // Auto-load phext[0] from the SIW's coordinate only when S or C pipes are active
    // (they use phext addressing). Pure D-pipe SIWs skip this — saves 1 comparison
    // + branch + possible cache miss per SIW on tight arithmetic loops.
    if (siw.s_fam < 4 || siw.c_fam < 4) && siw.phext_addr != PhextCoord::zero() {
        sentron.regs.phext[0] = siw.phext_addr;
    }

    // D-Pipe: 4-family indexed dispatch (family 4 = NOP, skip)
    if siw.d_fam < 4 {
        active += exec_d_family(sentron, siw);
    }

    // S-Pipe: 4-family indexed dispatch
    if siw.s_fam < 4 {
        active += exec_s_family(sentron, siw, mem);
    }

    // C-Pipe: 4-family indexed dispatch
    if siw.c_fam < 4 {
        active += exec_c_family(sentron, siw);
    }

    active
}

/// D-pipe dispatch table: 4 families, const function pointer array.
/// LLVM sees a simple index load + indirect call — no branch tree.
type DHandler = fn(&mut Sentron, &SIW) -> u8;
const D_TABLE: [DHandler; 4] = [
    exec_d_arithmetic,
    exec_d_reduce,
    exec_d_hdc,
    exec_d_ternary,
];

/// C-pipe dispatch table: 4 families.
type CHandler = fn(&mut Sentron, &SIW) -> u8;
const C_TABLE: [CHandler; 4] = [
    exec_c_pack,
    exec_c_send,
    exec_c_barrier,
    exec_c_reduce,
];

#[inline(always)]
fn exec_d_family(sentron: &mut Sentron, siw: &SIW) -> u8 {
    if (siw.d_fam as usize) < D_TABLE.len() {
        D_TABLE[siw.d_fam as usize](sentron, siw)
    } else {
        0
    }
}

/// S-pipe dispatch table: 4 families, const function pointer array.
/// Address (fam 2) and Route (fam 3) don't use Memory; wrapper fns absorb the
/// unused `mem` parameter so all 4 entries share the same SMemHandler signature.
/// LLVM sees a simple index load + indirect call — no branch tree.
type SMemHandler = fn(&mut Sentron, &SIW, &mut Memory) -> u8;

#[inline(always)]
fn exec_s_address_wrap(sentron: &mut Sentron, siw: &SIW, _mem: &mut Memory) -> u8 {
    exec_s_address(sentron, siw)
}
#[inline(always)]
fn exec_s_route_wrap(sentron: &mut Sentron, siw: &SIW, _mem: &mut Memory) -> u8 {
    exec_s_route(sentron, siw)
}

const S_TABLE: [SMemHandler; 4] = [
    exec_s_load,          // Family 0: SGATHER, SDEDUP
    exec_s_store,         // Family 1: SSCATTR, SFLUSH
    exec_s_address_wrap,  // Family 2: SINDEX, SALLOC, SFREE
    exec_s_route_wrap,    // Family 3: SPREFCH, SASSOC, SROUTE, SNEIGHBR
];

#[inline(always)]
fn exec_s_family(sentron: &mut Sentron, siw: &SIW, mem: &mut Memory) -> u8 {
    if (siw.s_fam as usize) < S_TABLE.len() {
        S_TABLE[siw.s_fam as usize](sentron, siw, mem)
    } else {
        0
    }
}

#[inline(always)]
fn exec_c_family(sentron: &mut Sentron, siw: &SIW) -> u8 {
    if (siw.c_fam as usize) < C_TABLE.len() {
        C_TABLE[siw.c_fam as usize](sentron, siw)
    } else {
        0
    }
}

// ── D-Pipe Families ──────────────────────────────────────────────────────────

#[inline(always)]
fn exec_d_arithmetic(sentron: &mut Sentron, siw: &SIW) -> u8 {
    let r = &mut sentron.regs.general;
    match siw.d_op {
        DenseOp::DFMA { rd, rs1, rs2, rs3 } => {
            r[rd as usize] = r[rs1 as usize].wrapping_mul(r[rs2 as usize]).wrapping_add(r[rs3 as usize]);
        }
        DenseOp::DADD { rd, rs1, rs2 } => {
            r[rd as usize] = r[rs1 as usize].wrapping_add(r[rs2 as usize]);
        }
        DenseOp::DSUB { rd, rs1, rs2 } => {
            r[rd as usize] = r[rs1 as usize].wrapping_sub(r[rs2 as usize]);
        }
        DenseOp::DMUL { rd, rs1, rs2 } => {
            r[rd as usize] = r[rs1 as usize].wrapping_mul(r[rs2 as usize]);
        }
        DenseOp::DCMP { rd, rs1, rs2 } => {
            r[rd as usize] = (r[rs1 as usize] - r[rs2 as usize]).signum();
        }
        DenseOp::DSEL { rd, rs1, rs2, flags } => {
            r[rd as usize] = if flags != 0 { r[rs1 as usize] } else { r[rs2 as usize] };
        }
        DenseOp::DMOV { rd, imm } => {
            r[rd as usize] = imm;
        }
        _ => {}
    }
    1
}

#[inline(always)]
fn exec_d_reduce(sentron: &mut Sentron, siw: &SIW) -> u8 {
    let r = &mut sentron.regs.general;
    if let DenseOp::DRED { rd, rs1, op } = siw.d_op {
        let val = r[rs1 as usize];
        r[rd as usize] = match op {
            ReductionOp::Sum => val,
            ReductionOp::Max => val,
            ReductionOp::Min => val,
            ReductionOp::And => val & 0xFF,
            ReductionOp::Or  => val | 0x01,
            ReductionOp::Xor => val ^ val.rotate_right(16),
        };
    }
    1
}

#[inline(always)]
fn exec_d_hdc(sentron: &mut Sentron, siw: &SIW) -> u8 {
    let r = &mut sentron.regs.general;
    match siw.d_op {
        DenseOp::DHDENC { rd, rs, width } => {
            let val = r[rs as usize];
            r[rd as usize] = val.rotate_left(width as u32 % 64);
        }
        DenseOp::DHDBIND { rd, rs1, rs2 } => {
            r[rd as usize] = r[rs1 as usize] ^ r[rs2 as usize];
        }
        DenseOp::DHDBUND { rd, rs1, rs2 } => {
            r[rd as usize] = r[rs1 as usize].wrapping_add(r[rs2 as usize]);
        }
        DenseOp::DHDPERM { rd, rs, k } => {
            r[rd as usize] = r[rs as usize].rotate_left(k as u32 % 64);
        }
        DenseOp::DHDSIM { rd, rs1, rs2 } => {
            let xor = r[rs1 as usize] ^ r[rs2 as usize];
            r[rd as usize] = 64 - xor.count_ones() as i64;
        }
        _ => {}
    }
    1
}

#[inline(always)]
fn exec_d_ternary(sentron: &mut Sentron, siw: &SIW) -> u8 {
    let r = &mut sentron.regs.general;
    match siw.d_op {
        DenseOp::DTERNARY { rd, rs1, trit_reg } => {
            let trits = r[trit_reg as usize] as u64;
            r[rd as usize] = ternary_apply(r[rs1 as usize], trits);
        }
        DenseOp::DTPOP { rd, rs } => {
            let trits = r[rs as usize] as u64;
            let mut count = 0i64;
            for i in 0..32 {
                if (trits >> (i * 2)) & 0x3 != 0 { count += 1; }
            }
            r[rd as usize] = count;
        }
        DenseOp::DTACC { rd, rs1, trit_reg } => {
            let trits = r[trit_reg as usize] as u64;
            r[rd as usize] = r[rd as usize].wrapping_add(ternary_apply(r[rs1 as usize], trits));
        }
        _ => {}
    }
    1
}

// ── S-Pipe Families ──────────────────────────────────────────────────────────

#[inline(always)]
fn exec_s_load(sentron: &mut Sentron, siw: &SIW, mem: &mut Memory) -> u8 {
    match siw.s_op {
        SparseOp::SGATHER { rd, coord_idx, .. } => {
            let coord = sentron.regs.phext[coord_idx as usize].clone();
            sentron.regs.general[rd as usize] = mem.gather_i64(&coord);
        }
        SparseOp::SDEDUP { rd, rs, .. } => {
            sentron.regs.general[rd as usize] = sentron.regs.general[rs as usize];
        }
        _ => {}
    }
    1
}

#[inline(always)]
fn exec_s_store(sentron: &mut Sentron, siw: &SIW, mem: &mut Memory) -> u8 {
    match siw.s_op {
        SparseOp::SSCATTR { coord_idx, rs, .. } => {
            let coord = sentron.regs.phext[coord_idx as usize].clone();
            let val = sentron.regs.general[rs as usize];
            mem.scatter_i64(&coord, val);
        }
        SparseOp::SFLUSH { .. } => {}
        _ => {}
    }
    1
}

#[inline(always)]
fn exec_s_address(sentron: &mut Sentron, siw: &SIW) -> u8 {
    match siw.s_op {
        SparseOp::SINDEX { rd, base, offset, dim } => {
            let mut coord = sentron.regs.phext[base as usize].clone();
            let old_val = coord.get_dim(dim) as i32;
            // Clamp to [1, MAX_DIM] — phext coordinates are 1-indexed, 11-bit max
            let new_val = old_val.wrapping_add(offset).clamp(1, 2047) as u16;
            coord.set_dim(dim, new_val);
            sentron.regs.general[rd as usize] = new_val as i64;
            sentron.regs.phext[rd as usize % 8] = coord;
        }
        SparseOp::SALLOC { rd, .. } => {
            sentron.regs.general[rd as usize] = 1;
        }
        SparseOp::SFREE { .. } => {}
        _ => {}
    }
    1
}

#[inline(always)]
fn exec_s_route(sentron: &mut Sentron, siw: &SIW) -> u8 {
    match siw.s_op {
        SparseOp::SPREFCH { .. } => {}
        SparseOp::SASSOC { rd, .. } => {
            let coord = sentron.regs.phext[0];
            let idx = sentron.assoc.store(&coord);
            sentron.regs.general[rd as usize] = idx as i64;
        }
        SparseOp::SROUTE { rd, .. } => {
            let coord = sentron.regs.phext[0];
            let (sim, hash) = sentron.assoc.route(&coord);
            sentron.regs.general[rd as usize] = sim;
            if (rd as usize) + 1 < 16 {
                sentron.regs.general[rd as usize + 1] = hash;
            }
        }
        SparseOp::SNEIGHBR { rd, .. } => {
            let coord = sentron.regs.phext[0];
            let threshold = sentron.regs.general[1] as f64 / 1000.0;
            let count = sentron.assoc.neighbors(&coord, threshold);
            sentron.regs.general[rd as usize] = count;
        }
        _ => {}
    }
    1
}

// ── C-Pipe Families ──────────────────────────────────────────────────────────

#[inline(always)]
fn exec_c_pack(sentron: &mut Sentron, siw: &SIW) -> u8 {
    if let CoordOp::CPACK { rd, rs1, rs2, .. } = siw.c_op {
        // Pack rs1 → msg[0..8], rs2 → msg[8..16] as separate LE i64 values
        let a = sentron.regs.general[rs1 as usize].to_le_bytes();
        let b = sentron.regs.general[rs2 as usize].to_le_bytes();
        let msg = &mut sentron.regs.message[rd as usize % 4];
        msg[..8].copy_from_slice(&a);
        msg[8..16].copy_from_slice(&b);
    }
    1
}

#[inline(always)]
fn exec_c_send(sentron: &mut Sentron, siw: &SIW) -> u8 {
    match siw.c_op {
        CoordOp::CRECV { rd, .. } => {
            // Pop from inbox (LIFO) matching old exec_siw behavior
            sentron.regs.general[rd as usize] = match sentron.inbox.pop() {
                Some((_, val)) => val,
                None => 0,
            };
        }
        CoordOp::CSEND { .. } | CoordOp::CROUTE { .. } | CoordOp::CFANOUT { .. } => {}
        _ => {}
    }
    1
}

#[inline(always)]
fn exec_c_barrier(_sentron: &mut Sentron, _siw: &SIW) -> u8 {
    // CBAR and CFENCE are coordination points — no register mutation
    1
}

#[inline(always)]
fn exec_c_reduce(sentron: &mut Sentron, siw: &SIW) -> u8 {
    match siw.c_op {
        CoordOp::CREDUCE { rd, rs, op, .. } => {
            let val = sentron.regs.general[rs as usize];
            sentron.regs.general[rd as usize] = match op {
                ReductionOp::Sum => val,
                ReductionOp::Max => val,
                ReductionOp::Min => val,
                ReductionOp::And => val & 0xFF,
                ReductionOp::Or  => val | 0x01,
                ReductionOp::Xor => val ^ val.rotate_right(16),
            };
        }
        CoordOp::CCAST { .. } | CoordOp::CSLICE { .. } => {}
        CoordOp::CMERGE { rd, .. } => {
            sentron.regs.general[rd as usize] = 0;
        }
        _ => {}
    }
    1
}

/// Run a sentron to completion with PPT-backed memory, returning execution statistics.
pub fn run(sentron: &mut Sentron, mem: &mut Memory) -> ExecStats {
    let mut stats = ExecStats::default();

    if sentron.state != SentronState::Running {
        return stats;
    }

    // W20: eliminate per-SIW clone by extracting program out of sentron.
    // We swap the program Vec out, iterate by value, then restore.
    // This avoids the borrow conflict (sentron.program[i] while sentron is &mut)
    // and removes the heap clone per SIW.
    let program = std::mem::take(&mut sentron.program);

    for siw in &program {
        let d_active = siw.d_fam < 4;
        let s_active = siw.s_fam < 4;
        let c_active = siw.c_fam < 4;

        if d_active { stats.d_ops += 1; } else { stats.d_nops += 1; }
        if s_active { stats.s_ops += 1; } else { stats.s_nops += 1; }
        if c_active { stats.c_ops += 1; } else { stats.c_nops += 1; }

        // Flux: snapshot general registers before execution
        let prev_regs = sentron.regs.general;

        let active = exec_siw_octawire(sentron, siw, mem);
        stats.ops_retired += active as u64;
        stats.siws_retired += 1;
        stats.cycles += 1;
        sentron.ip += 1;

        // ── Deep alignment: feed pipe activity into NeuronLayer ──────────────
        // D-pipe maps to Story channel (dense/serial), S-pipe to Light (sparse/parallel).
        // C-pipe coordinates both — contributes 0.5 to each channel.
        let story_in = if d_active { 1.0_f32 } else { 0.0 }
                     + if c_active { 0.5 } else { 0.0 };
        let light_in = if s_active { 1.0_f32 } else { 0.0 }
                     + if c_active { 0.5 } else { 0.0 };
        let vak_out = sentron.neurons.forward(story_in, light_in);
        // Dominant VakLevel from NeuronLayer output
        let dom_vak = vak_out.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i).unwrap_or(3);
        stats.vak_histogram[dom_vak] += 1;

        // ── Sentron flux: L1 norm of register delta ──────────────────────────
        let flux_siw: i64 = sentron.regs.general.iter()
            .zip(prev_regs.iter())
            .map(|(cur, prev)| cur.wrapping_sub(*prev).wrapping_abs())
            .sum::<i64>().wrapping_abs();
        stats.flux_total += flux_siw as f64;
    }

    // Accumulate spanda cycles after stream completes
    stats.spanda_cycles = sentron.neurons.total_spanda_cycles();

    // Restore program (sentron may be inspected after run())
    sentron.program = program;
    // Flush stats to sentron state once at end — not per SIW
    sentron.retired = stats.siws_retired;
    sentron.cycles  = stats.cycles;
    sentron.retire();
    stats
}

/// OctaWire batched stream executor.
///
/// Groups consecutive SIWs by mode byte → LLVM can vectorize within each run.
/// The "pause between breaths" (delimiter between SIWs) is pre-filled with the
/// family index — zero decision overhead at runtime (VBT verse 24).
pub fn run_batched(sentron: &mut Sentron, mem: &mut Memory) -> ExecStats {
    let mut stats = ExecStats::default();

    if sentron.state != SentronState::Running {
        return stats;
    }

    let len = sentron.program.len();
    let mut i = 0;

    while i < len {
        // Find end of same-mode run without cloning
        let mode = sentron.program[i].mode_bits();
        let mut run_end = i + 1;
        while run_end < len && sentron.program[run_end].mode_bits() == mode {
            run_end += 1;
        }

        // Execute run by index — avoids borrow conflict with exec_siw_octawire
        while sentron.ip < run_end {
            let siw = sentron.program[sentron.ip].clone();
            if matches!(siw.d_op, DenseOp::DNOP) { stats.d_nops += 1; } else { stats.d_ops += 1; }
            if matches!(siw.s_op, SparseOp::SNOP) { stats.s_nops += 1; } else { stats.s_ops += 1; }
            if matches!(siw.c_op, CoordOp::CNOP) { stats.c_nops += 1; } else { stats.c_ops += 1; }

            let d_active = siw.d_fam < 4;
            let s_active = siw.s_fam < 4;
            let c_active = siw.c_fam < 4;
            let prev_regs = sentron.regs.general;

            let active = exec_siw_octawire(sentron, &siw, mem);
            stats.ops_retired += active as u64;
            stats.siws_retired += 1;
            stats.cycles += 1;
            sentron.ip += 1;

            // Deep alignment: NeuronLayer feed
            let story_in = if d_active { 1.0_f32 } else { 0.0 }
                         + if c_active { 0.5 } else { 0.0 };
            let light_in = if s_active { 1.0_f32 } else { 0.0 }
                         + if c_active { 0.5 } else { 0.0 };
            let vak_out = sentron.neurons.forward(story_in, light_in);
            let dom_vak = vak_out.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i).unwrap_or(3);
            stats.vak_histogram[dom_vak] += 1;

            // Sentron flux: L1 register delta
            let flux_siw: i64 = sentron.regs.general.iter()
                .zip(prev_regs.iter())
                .map(|(cur, prev)| cur.wrapping_sub(*prev).wrapping_abs())
                .sum();
            stats.flux_total += flux_siw as f64;
        }

        sentron.retired = stats.siws_retired;
        sentron.cycles = stats.cycles;
        i = run_end;
    }

    stats.spanda_cycles = sentron.neurons.total_spanda_cycles();
    sentron.retire();
    stats
}

/// Convenience: run with a fresh memory (for D-pipe-only or simple tests)
pub fn run_standalone(sentron: &mut Sentron) -> ExecStats {
    let mut mem = Memory::new();
    run(sentron, &mut mem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::*;
    use crate::PhextCoord;

    fn make_sentron() -> Sentron {
        Sentron::new(0, PhextCoord::zero(), 0, 0)
    }

    #[test]
    fn execute_empty_program() {
        let mut s = make_sentron();
        s.spawn(vec![]);
        let stats = run_standalone(&mut s);
        assert_eq!(stats.siws_retired, 0);
        assert_eq!(s.state, SentronState::Retired);
    }

    #[test]
    fn execute_arithmetic() {
        let mut s = make_sentron();
        let program = vec![
            SIW::new(DenseOp::DMOV { rd: 0, imm: 7 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMOV { rd: 1, imm: 6 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        s.spawn(program);
        let stats = run_standalone(&mut s);

        assert_eq!(s.regs.general[2], 42);
        assert_eq!(stats.siws_retired, 3);
        assert_eq!(stats.d_ops, 3);
        assert_eq!(stats.ops_per_cycle(), 1.0);
    }

    #[test]
    fn execute_three_wide() {
        let mut s = make_sentron();
        s.regs.general[1] = 3;
        s.regs.general[2] = 4;
        s.regs.general[3] = 5;

        let program = vec![
            SIW::new(
                DenseOp::DFMA { rd: 4, rs1: 1, rs2: 2, rs3: 3 },
                SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L2 },
                CoordOp::CPACK { rd: 0, rs1: 1, rs2: 2, fmt: MessageFormat::Result },
                PhextCoord::zero(),
            ),
        ];
        s.spawn(program);
        let stats = run_standalone(&mut s);

        assert_eq!(s.regs.general[4], 17);
        assert_eq!(stats.ops_per_cycle(), 3.0);
        assert_eq!(stats.utilization(), 1.0);
    }

    #[test]
    fn execute_branchless_select() {
        let mut s = make_sentron();
        s.regs.general[0] = 100;
        s.regs.general[1] = 200;
        s.regs.general[2] = 1;

        let program = vec![
            SIW::new(DenseOp::DSEL { rd: 3, rs1: 0, rs2: 1, flags: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];
        s.spawn(program);
        run_standalone(&mut s);
        assert_eq!(s.regs.general[3], 100);
    }

    #[test]
    fn double_buffer_pipeline() {
        let mut s = make_sentron();
        for i in 0..8 {
            s.regs.general[i] = (i as i64 + 1) * 10;
        }

        let program = (0..4u8).map(|k| {
            SIW::new(
                DenseOp::DADD { rd: 8 + k, rs1: k, rs2: (k + 1).min(7) },
                SparseOp::SINDEX { rd: 1, base: 0, offset: k as i32 + 2, dim: 0 },
                CoordOp::CPACK { rd: 0, rs1: k, rs2: (k + 1).min(7), fmt: MessageFormat::Result },
                PhextCoord::zero(),
            )
        }).collect();

        s.spawn(program);
        let stats = run_standalone(&mut s);

        assert_eq!(stats.siws_retired, 4);
        assert_eq!(stats.ops_retired, 12);
        assert_eq!(stats.ops_per_cycle(), 3.0);
    }

    #[test]
    fn message_packing() {
        let mut s = make_sentron();
        s.regs.general[0] = 0xDEADBEEF;
        s.regs.general[1] = 0xCAFEBABE;

        let program = vec![
            SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CPACK { rd: 0, rs1: 0, rs2: 1, fmt: MessageFormat::Result }, PhextCoord::zero()),
        ];
        s.spawn(program);
        run_standalone(&mut s);

        let msg = &s.regs.message[0];
        let a = i64::from_le_bytes(msg[..8].try_into().unwrap());
        let b = i64::from_le_bytes(msg[8..16].try_into().unwrap());
        assert_eq!(a, 0xDEADBEEF);
        assert_eq!(b, 0xCAFEBABE);
    }

    /// W5: Full gather/scatter through PPT-backed memory
    #[test]
    fn gather_scatter_through_ppt() {
        let mut s = make_sentron();
        let mut mem = Memory::new();

        // Set phext register p0 to coord [5,5,5,1,1,1,1,1,1,1,1]
        s.regs.phext[0] = PhextCoord::new([5, 5, 5, 1, 1, 1, 1, 1, 1, 1, 1]);
        // Set phext register p1 to different coord
        s.regs.phext[1] = PhextCoord::new([10, 10, 10, 1, 1, 1, 1, 1, 1, 1, 1]);

        let program = vec![
            // r0 = 42, then scatter to p0
            SIW::new(DenseOp::DMOV { rd: 0, imm: 42 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 0, rs: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            // r1 = 99, then scatter to p1
            SIW::new(DenseOp::DMOV { rd: 1, imm: 99 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 1, rs: 1, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            // Gather back: r2 = mem[p0], r3 = mem[p1]
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 2, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 3, coord_idx: 1, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            // r4 = r2 + r3 (should be 42 + 99 = 141)
            SIW::new(DenseOp::DADD { rd: 4, rs1: 2, rs2: 3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];

        s.spawn(program);
        let stats = run(&mut s, &mut mem);

        assert_eq!(s.regs.general[2], 42);
        assert_eq!(s.regs.general[3], 99);
        assert_eq!(s.regs.general[4], 141); // 42 + 99
        assert_eq!(stats.siws_retired, 7);

        // Verify PPT was used
        let ppt_stats = mem.ppt.stats();
        assert!(ppt_stats.ptc_hits > 0, "Should have PTC hits from repeated coord access");
    }

    /// W5: Compute-scatter-gather pipeline (the real vTPU pattern)
    #[test]
    fn compute_scatter_gather_pipeline() {
        let mut s = make_sentron();
        let mut mem = Memory::new();

        // Phext registers: 4 coordinates for a mini dot product
        for i in 0..4u16 {
            s.regs.phext[i as usize] = PhextCoord::new([i + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        }

        // Pre-seed memory with values at those coordinates
        let coords: Vec<PhextCoord> = (0..4u16)
            .map(|i| PhextCoord::new([i + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]))
            .collect();
        mem.scatter_i64(&coords[0], 2);
        mem.scatter_i64(&coords[1], 3);
        mem.scatter_i64(&coords[2], 4);
        mem.scatter_i64(&coords[3], 5);

        let program = vec![
            // Gather all 4 values (fully packed: D-pipe NOPs to avoid conflicts)
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 1, coord_idx: 1, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 2, coord_idx: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 3, coord_idx: 3, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
            // r4 = r0*r1 = 2*3 = 6
            SIW::new(DenseOp::DMUL { rd: 4, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            // r5 = r2*r3 = 4*5 = 20
            SIW::new(DenseOp::DMUL { rd: 5, rs1: 2, rs2: 3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            // r6 = r4 + r5 = 6 + 20 = 26 (dot product: [2,4]·[3,5])
            SIW::new(DenseOp::DADD { rd: 6, rs1: 4, rs2: 5 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];

        s.spawn(program);
        let stats = run(&mut s, &mut mem);

        assert_eq!(s.regs.general[6], 26); // dot([2,4], [3,5]) = 6+20 = 26
        assert_eq!(stats.siws_retired, 7);
    }

    // ── W19 OctaWire Tests ─────────────────────────────────────────────────

    #[test]
    fn octawire_nop_skips_all_pipes() {
        let mut s = make_sentron();
        let mut mem = Memory::new();
        let siw = SIW::nop();
        assert_eq!(siw.d_fam, 4);
        assert_eq!(siw.s_fam, 4);
        assert_eq!(siw.c_fam, 4);
        let active = exec_siw_octawire(&mut s, &siw, &mut mem);
        assert_eq!(active, 0, "NOP SIW should retire 0 active ops");
    }

    #[test]
    fn octawire_mode_bits_nop() {
        let siw = SIW::nop();
        assert_eq!(siw.mode_bits(), 0b000, "all-NOP = mode 0");
    }

    #[test]
    fn octawire_mode_bits_d_only() {
        let siw = SIW::new(
            DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        );
        assert_eq!(siw.mode_bits(), 0b001, "D-only = bit 0 set");
    }

    #[test]
    fn octawire_mode_bits_all_active() {
        let siw = SIW::new(
            DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::SINDEX { rd: 0, base: 0, offset: 0, dim: 0 },
            CoordOp::CPACK { rd: 0, rs1: 1, rs2: 2, fmt: crate::pipes::MessageFormat::Result },
            PhextCoord::zero(),
        );
        assert_eq!(siw.mode_bits(), 0b111, "all active = bits 0,1,2 set");
    }

    #[test]
    fn octawire_d_arithmetic_fam0() {
        let mut s = make_sentron();
        let mut mem = Memory::new();
        s.regs.general[1] = 10;
        s.regs.general[2] = 5;
        let siw = SIW::new(
            DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
        );
        assert_eq!(siw.d_fam, 0, "DADD is Arithmetic family (0)");
        exec_siw_octawire(&mut s, &siw, &mut mem);
        assert_eq!(s.regs.general[0], 15);
    }

    #[test]
    fn octawire_d_hdc_fam2() {
        let siw = SIW::new(
            DenseOp::DHDENC { rd: 0, rs: 1, width: 64 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
        );
        assert_eq!(siw.d_fam, 2, "DHDENC is HDC family (2)");
    }

    #[test]
    fn octawire_d_ternary_fam3() {
        let siw = SIW::new(
            DenseOp::DTERNARY { rd: 0, rs1: 1, trit_reg: 2 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
        );
        assert_eq!(siw.d_fam, 3, "DTERNARY is Ternary family (3)");
    }

    #[test]
    fn octawire_s_load_fam0() {
        let siw = SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP, PhextCoord::zero(),
        );
        assert_eq!(siw.s_fam, 0, "SGATHER is Load family (0)");
    }

    #[test]
    fn octawire_s_store_fam1() {
        let siw = SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { rs: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP, PhextCoord::zero(),
        );
        assert_eq!(siw.s_fam, 1, "SSCATTR is Store family (1)");
    }

    #[test]
    fn octawire_s_address_fam2() {
        let siw = SIW::new(
            DenseOp::DNOP,
            SparseOp::SINDEX { rd: 0, base: 0, offset: 0, dim: 0 },
            CoordOp::CNOP, PhextCoord::zero(),
        );
        assert_eq!(siw.s_fam, 2, "SINDEX is Address family (2)");
    }

    #[test]
    fn octawire_c_pack_fam0() {
        let siw = SIW::new(
            DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CPACK { rd: 0, rs1: 1, rs2: 2, fmt: crate::pipes::MessageFormat::Result },
            PhextCoord::zero(),
        );
        assert_eq!(siw.c_fam, 0, "CPACK is Pack family (0)");
    }

    #[test]
    fn octawire_c_send_fam1() {
        let siw = SIW::new(
            DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CSEND { msg_reg: 0, dest_sentron: 1 },
            PhextCoord::zero(),
        );
        assert_eq!(siw.c_fam, 1, "CSEND is Send family (1)");
    }

    #[test]
    fn octawire_c_barrier_fam2() {
        let siw = SIW::new(
            DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CBAR { barrier_id: 0, count: 4 },
            PhextCoord::zero(),
        );
        assert_eq!(siw.c_fam, 2, "CBAR is Barrier family (2)");
    }

    #[test]
    fn octawire_parity_with_standard() {
        // run() now uses exec_siw_octawire internally — run_batched should match
        let program = vec![
            SIW::new(DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DMUL { rd: 3, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DSUB { rd: 4, rs1: 3, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ];

        let mut s1 = make_sentron();
        let mut s2 = make_sentron();
        s1.regs.general[1] = 7; s1.regs.general[2] = 3;
        s2.regs.general[1] = 7; s2.regs.general[2] = 3;

        let mut mem1 = Memory::new();
        let mut mem2 = Memory::new();
        s1.spawn(program.clone());
        s2.spawn(program.clone());

        let stats1 = run(&mut s1, &mut mem1);
        let stats2 = run_batched(&mut s2, &mut mem2);

        assert_eq!(s1.regs.general[0], s2.regs.general[0], "r0 must match");
        assert_eq!(s1.regs.general[3], s2.regs.general[3], "r3 must match");
        assert_eq!(s1.regs.general[4], s2.regs.general[4], "r4 must match");
        assert_eq!(stats1.ops_retired, stats2.ops_retired, "ops retired must match");
    }

    #[test]
    fn octawire_batched_same_mode_run() {
        // All-arithmetic stream: batched should group all into one run
        let program: Vec<SIW> = (0..10).map(|i| {
            let rd = (i % 14) as u8;
            let rs1 = ((i + 1) % 15) as u8;
            let rs2 = ((i + 2) % 15) as u8;
            SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
        }).collect();

        let mode = program[0].mode_bits();
        assert!(program.iter().all(|s| s.mode_bits() == mode), "all same mode");

        let mut s = make_sentron();
        let mut mem = Memory::new();
        s.spawn(program);
        let stats = run_batched(&mut s, &mut mem);
        assert_eq!(stats.siws_retired, 10);
        assert_eq!(stats.ops_retired, 10);
    }

    #[test]
    fn octawire_fma_executes_correctly() {
        let mut s = make_sentron();
        let mut mem = Memory::new();
        s.regs.general[1] = 3;
        s.regs.general[2] = 4;
        s.regs.general[3] = 5; // rd=0, fma = r1*r2 + r3 = 3*4+5 = 17
        let siw = SIW::new(
            DenseOp::DFMA { rd: 0, rs1: 1, rs2: 2, rs3: 3 },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
        );
        exec_siw_octawire(&mut s, &siw, &mut mem);
        assert_eq!(s.regs.general[0], 17);
    }

    #[test]
    fn octawire_reduce_d_fam1() {
        let siw = SIW::new(
            DenseOp::DRED { rd: 0, rs1: 1, op: crate::pipes::ReductionOp::Sum },
            SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
        );
        assert_eq!(siw.d_fam, 1, "DRED is Reduce family (1)");
    }

    #[test]
    fn octawire_empty_stream_batched() {
        let mut s = make_sentron();
        let mut mem = Memory::new();
        s.spawn(vec![]);
        let stats = run_batched(&mut s, &mut mem);
        assert_eq!(stats.siws_retired, 0);
        assert_eq!(stats.ops_retired, 0);
    }
}

// ── W20 Dispatch Table Tests ─────────────────────────────────────────────────
#[cfg(test)]
mod w20_dispatch_tests {
    use super::*;
    use crate::siw::SIW;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};
    use crate::phext_coord::PhextCoord;
    use crate::sentron::Sentron;
    use crate::memory::Memory;

    fn s() -> Sentron { Sentron::new(0, PhextCoord::zero(), 0, 0) }
    fn siw(d: DenseOp, sp: SparseOp, c: CoordOp) -> SIW {
        SIW::new(d, sp, c, PhextCoord::zero())
    }

    #[test]
    fn d_table_len_is_4() {
        assert_eq!(D_TABLE.len(), 4);
    }

    #[test]
    fn c_table_len_is_4() {
        assert_eq!(C_TABLE.len(), 4);
    }

    #[test]
    fn d_table_fam0_executes_arithmetic() {
        let mut s = s(); let _m = Memory::new();
        s.regs.general[1] = 3; s.regs.general[2] = 4;
        let w = siw(DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP);
        assert_eq!(w.d_fam, 0);
        D_TABLE[0](&mut s, &w);
        assert_eq!(s.regs.general[0], 7);
    }

    #[test]
    fn d_table_fam2_executes_hdc() {
        let mut s = s(); let _m = Memory::new();
        let w = siw(DenseOp::DHDENC { rd: 0, rs: 1, width: 64 }, SparseOp::SNOP, CoordOp::CNOP);
        assert_eq!(w.d_fam, 2);
        let result = D_TABLE[2](&mut s, &w);
        assert_eq!(result, 1, "HDC family should return 1 active op");
    }

    #[test]
    fn c_table_fam0_executes_pack() {
        let mut s = s(); let _m = Memory::new();
        s.regs.general[1] = 0xAB; s.regs.general[2] = 0xCD;
        let w = siw(DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CPACK { rd: 0, rs1: 1, rs2: 2, fmt: crate::pipes::MessageFormat::Result });
        assert_eq!(w.c_fam, 0);
        C_TABLE[0](&mut s, &w);
        let a = i64::from_le_bytes(s.regs.message[0][..8].try_into().unwrap());
        assert_eq!(a, 0xAB);
    }

    #[test]
    fn c_table_fam2_executes_barrier() {
        let mut s = s();
        let w = siw(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CBAR { barrier_id: 0, count: 1 });
        assert_eq!(w.c_fam, 2);
        let result = C_TABLE[2](&mut s, &w);
        assert_eq!(result, 1, "barrier returns 1 active op");
    }

    #[test]
    fn octawire_nop_fam_skips_table() {
        let mut s = s(); let mut mem = Memory::new();
        let w = SIW::nop();
        // d_fam=4 and c_fam=4 → both exceed table bounds → returns 0
        let active = exec_siw_octawire(&mut s, &w, &mut mem);
        assert_eq!(active, 0);
    }

    #[test]
    fn d_table_fam3_ternary_active() {
        let mut s = s();
        let w = siw(DenseOp::DTPOP { rd: 0, rs: 1 }, SparseOp::SNOP, CoordOp::CNOP);
        assert_eq!(w.d_fam, 3);
        let result = D_TABLE[3](&mut s, &w);
        assert_eq!(result, 1);
    }

    #[test]
    fn c_table_fam1_send_active() {
        let mut s = s();
        let w = siw(DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CSEND { msg_reg: 0, dest_sentron: 1 });
        assert_eq!(w.c_fam, 1);
        let result = C_TABLE[1](&mut s, &w);
        assert_eq!(result, 1);
    }

    #[test]
    fn c_table_fam3_reduce_active() {
        let mut s = s();
        let w = siw(DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CREDUCE { rd: 0, rs: 1, op: crate::pipes::ReductionOp::Sum, group: 0 });
        assert_eq!(w.c_fam, 3);
        let result = C_TABLE[3](&mut s, &w);
        assert_eq!(result, 1);
    }
}
