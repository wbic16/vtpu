//! vTPU Executor — steps a sentron through its SIW stream
//!
//! Phase 0 executor: interprets SIW operations against the sentron's
//! register file. No real cache hierarchy yet (that's W5/PPT).
//! This executor measures ops/cycle assuming ideal 3-wide retirement.

#[cfg(test)]
use crate::coord::PhextCoord;
use crate::pipe::{DenseOp, SparseOp, CoordOp, ReduceOp};
use crate::sentron::{Sentron, SentronState};
use crate::siw::SIW;

/// Execution statistics for a sentron run
#[derive(Debug, Clone, Default)]
pub struct ExecStats {
    /// Total SIWs retired
    pub siws_retired: u64,
    /// Total ops retired (active pipes only, not NOPs)
    pub ops_retired: u64,
    /// Total cycles consumed (1 cycle per SIW in ideal 3-wide)
    pub cycles: u64,
    /// D-Pipe ops executed
    pub d_ops: u64,
    /// S-Pipe ops executed
    pub s_ops: u64,
    /// C-Pipe ops executed
    pub c_ops: u64,
    /// D-Pipe NOPs
    pub d_nops: u64,
    /// S-Pipe NOPs
    pub s_nops: u64,
    /// C-Pipe NOPs
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

    /// Per-pipe utilization
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

/// Execute one SIW against a sentron's register file.
/// Returns the number of active ops (0-3).
fn exec_siw(sentron: &mut Sentron, siw: &SIW) -> u8 {
    let mut active = 0u8;

    // ── D-Pipe ──
    match siw.d_op {
        DenseOp::Nop => {}
        DenseOp::Fma { rd, rs1, rs2, rs3 } => {
            let r = &sentron.regs.general;
            let result = r[rs1 as usize]
                .wrapping_mul(r[rs2 as usize])
                .wrapping_add(r[rs3 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::Add { rd, rs1, rs2 } => {
            let result = sentron.regs.general[rs1 as usize]
                .wrapping_add(sentron.regs.general[rs2 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::Sub { rd, rs1, rs2 } => {
            let result = sentron.regs.general[rs1 as usize]
                .wrapping_sub(sentron.regs.general[rs2 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::Mul { rd, rs1, rs2 } => {
            let result = sentron.regs.general[rs1 as usize]
                .wrapping_mul(sentron.regs.general[rs2 as usize]);
            sentron.regs.general[rd as usize] = result;
            active += 1;
        }
        DenseOp::Cmp { rd, rs1, rs2 } => {
            let a = sentron.regs.general[rs1 as usize];
            let b = sentron.regs.general[rs2 as usize];
            // Store comparison result: -1, 0, or 1
            sentron.regs.general[rd as usize] = match a.cmp(&b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };
            active += 1;
        }
        DenseOp::Red { rd, rs1, op } => {
            // Local reduce on a single value is identity (meaningful with vectors)
            let val = sentron.regs.general[rs1 as usize];
            sentron.regs.general[rd as usize] = match op {
                ReduceOp::Sum | ReduceOp::Max | ReduceOp::Min => val,
                ReduceOp::And => val,
                ReduceOp::Or => val,
                ReduceOp::Xor => val,
            };
            active += 1;
        }
        DenseOp::Sel { rd, rs1, rs2, flags } => {
            let cond = sentron.regs.general[flags as usize];
            sentron.regs.general[rd as usize] = if cond != 0 {
                sentron.regs.general[rs1 as usize]
            } else {
                sentron.regs.general[rs2 as usize]
            };
            active += 1;
        }
        DenseOp::Mov { rd, imm } => {
            sentron.regs.general[rd as usize] = imm;
            active += 1;
        }
    }

    // ── S-Pipe ──
    match siw.s_op {
        SparseOp::Nop => {}
        SparseOp::Index { rd, base_reg, offset, dim } => {
            // Adjust a phext coordinate along one dimension
            let base = sentron.regs.phext[base_reg as usize];
            let current = base.dim(dim as usize) as i16;
            let new_val = (current + offset).max(0) as u16;
            sentron.regs.phext[rd as usize] = base.with_dim(dim as usize, new_val);
            active += 1;
        }
        SparseOp::Prefetch { .. } => {
            // Phase 0: prefetch is a hint, no-op in interpreter
            active += 1;
        }
        SparseOp::Gather { rd, coord_reg, width: _ } => {
            // Phase 0: no real memory backing. Store coord hash as placeholder.
            let coord = sentron.regs.phext[coord_reg as usize];
            sentron.regs.general[rd as usize] = coord.raw() as i64;
            active += 1;
        }
        SparseOp::Scatter { coord_reg: _, rs: _, width: _ } => {
            // Phase 0: no real memory backing. Count as executed.
            active += 1;
        }
        SparseOp::Dedup { rd, rs, table: _ } => {
            sentron.regs.general[rd as usize] = sentron.regs.general[rs as usize];
            active += 1;
        }
        SparseOp::Flush { .. } => { active += 1; }
        SparseOp::Alloc { rd, size, dim_mask: _ } => {
            // Phase 0: return size as placeholder address
            sentron.regs.general[rd as usize] = size as i64;
            active += 1;
        }
        SparseOp::Free { .. } => { active += 1; }
    }

    // ── C-Pipe ──
    match siw.c_op {
        CoordOp::Nop => {}
        CoordOp::Pack { rd, rs1, rs2, fmt: _ } => {
            // Pack two registers into message buffer
            let a = sentron.regs.general[rs1 as usize].to_le_bytes();
            let b = sentron.regs.general[rs2 as usize].to_le_bytes();
            let msg = &mut sentron.regs.message[rd as usize];
            msg[..8].copy_from_slice(&a);
            msg[8..16].copy_from_slice(&b);
            active += 1;
        }
        CoordOp::Route { .. } => { active += 1; } // Phase 0: no network
        CoordOp::Send { .. } => { active += 1; }  // Phase 0: no IPC
        CoordOp::Recv { rd, .. } => {
            // Phase 0: return 0 (no actual message passing)
            sentron.regs.general[rd as usize] = 0;
            active += 1;
        }
        CoordOp::Barrier { .. } => { active += 1; } // Phase 0: instant barrier
        CoordOp::Fence { .. } => { active += 1; }
        CoordOp::Reduce { rd, rs, op, .. } => {
            // Phase 0: single-sentron reduce = identity
            let val = sentron.regs.general[rs as usize];
            sentron.regs.general[rd as usize] = match op {
                ReduceOp::Sum => val,
                ReduceOp::Max => val,
                ReduceOp::Min => val,
                ReduceOp::And => val,
                ReduceOp::Or => val,
                ReduceOp::Xor => val,
            };
            active += 1;
        }
        CoordOp::Cast { .. } => { active += 1; }
    }

    active
}

/// Run a sentron to completion, returning execution statistics.
pub fn run(sentron: &mut Sentron) -> ExecStats {
    let mut stats = ExecStats::default();

    if sentron.state != SentronState::Running {
        return stats;
    }

    let program = match &sentron.program {
        Some(p) => p.instructions.clone(), // clone to avoid borrow issues
        None => return stats,
    };

    while sentron.ip < program.len() {
        let siw = &program[sentron.ip];

        // Count per-pipe activity
        if matches!(siw.d_op, DenseOp::Nop) { stats.d_nops += 1; } else { stats.d_ops += 1; }
        if matches!(siw.s_op, SparseOp::Nop) { stats.s_nops += 1; } else { stats.s_ops += 1; }
        if matches!(siw.c_op, CoordOp::Nop) { stats.c_nops += 1; } else { stats.c_ops += 1; }

        let active = exec_siw(sentron, siw);
        stats.ops_retired += active as u64;
        stats.siws_retired += 1;
        stats.cycles += 1; // 1 cycle per SIW in ideal 3-wide model

        sentron.ip += 1;
        sentron.retired = stats.siws_retired;
        sentron.cycles = stats.cycles;
    }

    sentron.retire();
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe::*;
    use crate::siw::{SIWStream, Deps};

    fn make_sentron() -> Sentron {
        Sentron::new(0, PhextCoord::BASE, 0, 0)
    }

    #[test]
    fn execute_empty_program() {
        let mut s = make_sentron();
        s.spawn(SIWStream::new(PhextCoord::BASE));
        let stats = run(&mut s);
        assert_eq!(stats.siws_retired, 0);
        assert_eq!(s.state, SentronState::Retired);
    }

    #[test]
    fn execute_arithmetic() {
        let mut s = make_sentron();
        let mut stream = SIWStream::new(PhextCoord::BASE);

        // r0 = 7
        stream.push(SIW::new(
            DenseOp::Mov { rd: 0, imm: 7 },
            SparseOp::Nop,
            CoordOp::Nop,
            PhextCoord::BASE,
            Deps::NONE,
        ));
        // r1 = 6
        stream.push(SIW::new(
            DenseOp::Mov { rd: 1, imm: 6 },
            SparseOp::Nop,
            CoordOp::Nop,
            PhextCoord::BASE,
            Deps::NONE,
        ));
        // r2 = r0 * r1 = 42
        stream.push(SIW::new(
            DenseOp::Mul { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::Nop,
            CoordOp::Nop,
            PhextCoord::BASE,
            Deps::NONE,
        ));

        s.spawn(stream);
        let stats = run(&mut s);

        assert_eq!(s.regs.general[2], 42);
        assert_eq!(stats.siws_retired, 3);
        assert_eq!(stats.d_ops, 3);
        assert_eq!(stats.s_nops, 3);
        assert_eq!(stats.ops_per_cycle(), 1.0); // only D-pipe active
    }

    #[test]
    fn execute_three_wide() {
        let mut s = make_sentron();
        let mut stream = SIWStream::new(PhextCoord::BASE);

        // Set up registers
        s.regs.general[1] = 3;
        s.regs.general[2] = 4;
        s.regs.general[3] = 5;
        s.regs.phext[0] = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);

        // Fully packed SIW: compute + prefetch + pack message
        stream.push(SIW::new(
            DenseOp::Fma { rd: 4, rs1: 1, rs2: 2, rs3: 3 }, // r4 = 3*4+5 = 17
            SparseOp::Prefetch { coord_reg: 0, hint: CacheHint::L2 },
            CoordOp::Pack { rd: 0, rs1: 1, rs2: 2, fmt: 0 },
            PhextCoord::BASE,
            Deps::NONE,
        ));

        s.spawn(stream);
        let stats = run(&mut s);

        assert_eq!(s.regs.general[4], 17); // 3*4+5
        assert_eq!(stats.ops_per_cycle(), 3.0); // perfect 3-wide
        assert_eq!(stats.utilization(), 1.0);
        assert_eq!(stats.d_utilization(), 1.0);
        assert_eq!(stats.s_utilization(), 1.0);
        assert_eq!(stats.c_utilization(), 1.0);
    }

    #[test]
    fn execute_phext_index() {
        let mut s = make_sentron();
        let mut stream = SIWStream::new(PhextCoord::BASE);

        // Start at 1.1.1/1.1.1/1.1.1
        s.regs.phext[0] = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);

        // Move along dimension 3 (book) by +5
        stream.push(SIW::new(
            DenseOp::Nop,
            SparseOp::Index { rd: 1, base_reg: 0, offset: 5, dim: 3 },
            CoordOp::Nop,
            PhextCoord::BASE,
            Deps::NONE,
        ));

        s.spawn(stream);
        let _stats = run(&mut s);

        // Book was 1 (0-based: 0), now should be 6 (0-based: 5)
        assert_eq!(s.regs.phext[1].dim(3), 5);
        // Other dims unchanged
        assert_eq!(s.regs.phext[1].dim(0), 0); // scroll
        assert_eq!(s.regs.phext[1].dim(2), 0); // chapter
    }

    #[test]
    fn execute_branchless_select() {
        let mut s = make_sentron();
        let mut stream = SIWStream::new(PhextCoord::BASE);

        s.regs.general[0] = 100; // option A
        s.regs.general[1] = 200; // option B
        s.regs.general[2] = 1;   // condition (true)

        // r3 = select(r0, r1, r2) — should pick r0 since r2 != 0
        stream.push(SIW::new(
            DenseOp::Sel { rd: 3, rs1: 0, rs2: 1, flags: 2 },
            SparseOp::Nop,
            CoordOp::Nop,
            PhextCoord::BASE,
            Deps::NONE,
        ));

        s.spawn(stream);
        run(&mut s);
        assert_eq!(s.regs.general[3], 100);

        // Now with false condition
        let mut s2 = make_sentron();
        let mut stream2 = SIWStream::new(PhextCoord::BASE);
        s2.regs.general[0] = 100;
        s2.regs.general[1] = 200;
        s2.regs.general[2] = 0; // false

        stream2.push(SIW::new(
            DenseOp::Sel { rd: 3, rs1: 0, rs2: 1, flags: 2 },
            SparseOp::Nop,
            CoordOp::Nop,
            PhextCoord::BASE,
            Deps::NONE,
        ));

        s2.spawn(stream2);
        run(&mut s2);
        assert_eq!(s2.regs.general[3], 200);
    }

    /// The double-buffer pattern from §7.3:
    /// SIW N:   D: compute data[K]     S: fetch data[K+2]    C: send result[K-1]
    /// SIW N+1: D: compute data[K+1]   S: fetch data[K+3]    C: send result[K]
    /// All three pipes active every cycle. Zero stalls.
    #[test]
    fn double_buffer_pipeline() {
        let mut s = make_sentron();
        let mut stream = SIWStream::new(PhextCoord::BASE);

        // Seed registers
        for i in 0..8 {
            s.regs.general[i] = (i as i64 + 1) * 10;
        }
        s.regs.phext[0] = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);

        // 4 cycles of double-buffered execution
        for k in 0..4u8 {
            stream.push(SIW::new(
                DenseOp::Add { rd: (8 + k) as Reg, rs1: k as Reg, rs2: (k + 1).min(7) as Reg },
                SparseOp::Index { rd: 1, base_reg: 0, offset: k as i16 + 2, dim: 0 },
                CoordOp::Pack { rd: 0, rs1: k as Reg, rs2: (k + 1).min(7) as Reg, fmt: 0 },
                PhextCoord::BASE,
                Deps::NONE, // double-buffer = no cross-SIW deps
            ));
        }

        s.spawn(stream);
        let stats = run(&mut s);

        assert_eq!(stats.siws_retired, 4);
        assert_eq!(stats.ops_retired, 12); // 4 × 3 = 12
        assert_eq!(stats.ops_per_cycle(), 3.0); // perfect
        assert_eq!(stats.d_nops, 0);
        assert_eq!(stats.s_nops, 0);
        assert_eq!(stats.c_nops, 0);
    }

    #[test]
    fn message_packing() {
        let mut s = make_sentron();
        let mut stream = SIWStream::new(PhextCoord::BASE);

        s.regs.general[0] = 0xDEADBEEF;
        s.regs.general[1] = 0xCAFEBABE;

        stream.push(SIW::new(
            DenseOp::Nop,
            SparseOp::Nop,
            CoordOp::Pack { rd: 0, rs1: 0, rs2: 1, fmt: 0 },
            PhextCoord::BASE,
            Deps::NONE,
        ));

        s.spawn(stream);
        run(&mut s);

        // Verify message buffer has both values packed
        let msg = &s.regs.message[0];
        let a = i64::from_le_bytes(msg[..8].try_into().unwrap());
        let b = i64::from_le_bytes(msg[8..16].try_into().unwrap());
        assert_eq!(a, 0xDEADBEEF);
        assert_eq!(b, 0xCAFEBABE);
    }
}
