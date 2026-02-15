//! vTPU Executor — steps a sentron through its SIW stream
//!
//! Phase 0: interpreter. Executes SIW operations against the sentron's
//! register file. Measures ops/cycle assuming ideal 3-wide retirement.

use crate::pipes::{DenseOp, SparseOp, CoordOp, ReductionOp};
use crate::memory::Memory;
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
}

/// Execute one SIW against a sentron's register file. Returns active op count (0-3).
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
        DenseOp::DHDENC { rd, rs, .. } |
        DenseOp::DHDBIND { rd, rs1: rs, .. } |
        DenseOp::DHDBUND { rd, rs1: rs, .. } |
        DenseOp::DHDPERM { rd, rs, .. } => {
            sentron.regs.general[rd as usize] = sentron.regs.general[rs as usize];
            active += 1;
        }
        DenseOp::DHDSIM { rd, rs1, rs2 } => {
            let a = sentron.regs.general[rs1 as usize];
            let b = sentron.regs.general[rs2 as usize];
            sentron.regs.general[rd as usize] = (!(a ^ b)).count_ones() as i64;
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
        SparseOp::SASSOC { rd, .. } | SparseOp::SROUTE { rd, .. } | SparseOp::SNEIGHBR { rd, .. } => {
            sentron.regs.general[rd as usize] = 0;
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
            sentron.regs.general[rd as usize] = 0;
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

/// Run a sentron to completion with PPT-backed memory, returning execution statistics.
pub fn run(sentron: &mut Sentron, mem: &mut Memory) -> ExecStats {
    let mut stats = ExecStats::default();

    if sentron.state != SentronState::Running {
        return stats;
    }

    while sentron.ip < sentron.program.len() {
        let siw = sentron.program[sentron.ip].clone();

        if matches!(siw.d_op, DenseOp::DNOP) { stats.d_nops += 1; } else { stats.d_ops += 1; }
        if matches!(siw.s_op, SparseOp::SNOP) { stats.s_nops += 1; } else { stats.s_ops += 1; }
        if matches!(siw.c_op, CoordOp::CNOP) { stats.c_nops += 1; } else { stats.c_ops += 1; }

        let active = exec_siw(sentron, &siw, mem);
        stats.ops_retired += active as u64;
        stats.siws_retired += 1;
        stats.cycles += 1;

        sentron.ip += 1;
        sentron.retired = stats.siws_retired;
        sentron.cycles = stats.cycles;
    }

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
}
