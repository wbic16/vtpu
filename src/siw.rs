//! Sentron Instruction Word (SIW) — the atomic unit of vTPU execution
//!
//! Every SIW contains exactly three operations: one D-op, one S-op, one C-op.
//! This is the scheduling contract. The compiler guarantees no intra-SIW
//! dependencies. The hardware sees three independent micro-ops per window.
//!
//! If a pipe has no work, it gets a NOP that retires in zero cycles
//! but maintains the 3-wide invariant.

use crate::coord::PhextCoord;
use crate::pipe::{DenseOp, SparseOp, CoordOp};

/// Dependency flags between SIWs.
/// Bit layout:
///   0: D depends on prior D
///   1: D depends on prior S
///   2: D depends on prior C
///   3: S depends on prior D
///   4: S depends on prior S
///   5: S depends on prior C
///   6: C depends on prior D
///   7: C depends on prior S
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Deps(u8);

impl Deps {
    pub const NONE: Self = Self(0);

    // Common patterns
    /// S-Pipe uses result from prior D-Pipe (e.g., computed coordinate)
    pub const S_FROM_D: Self = Self(1 << 3);
    /// C-Pipe uses result from prior D-Pipe (e.g., computed message payload)
    pub const C_FROM_D: Self = Self(1 << 6);
    /// D-Pipe uses result from prior S-Pipe (e.g., loaded data for computation)
    pub const D_FROM_S: Self = Self(1 << 1);
    /// The double-buffer pattern: D uses data from S two SIWs ago (no dep on prior)
    pub const DOUBLE_BUFFER: Self = Self(0); // explicitly no deps = perfect pipeline

    #[inline]
    pub fn has(self, dep: Self) -> bool {
        (self.0 & dep.0) != 0
    }

    #[inline]
    pub fn merge(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// True if this SIW has zero dependencies on the prior SIW
    #[inline]
    pub fn is_independent(self) -> bool {
        self.0 == 0
    }
}

/// A Sentron Instruction Word: the 3-wide atomic unit of vTPU execution.
///
/// Scheduling contract:
/// - Exactly one D-op, one S-op, one C-op per SIW
/// - No RAW/WAR/WAW hazards within a single SIW
/// - Cross-SIW dependencies explicit in `deps`
/// - phext_addr is the target coordinate for S-Pipe operations
#[derive(Debug, Clone)]
pub struct SIW {
    /// Dense pipeline operation (ALU)
    pub d_op: DenseOp,
    /// Sparse pipeline operation (Memory)
    pub s_op: SparseOp,
    /// Coordination pipeline operation (Communication)
    pub c_op: CoordOp,
    /// Target phext coordinate for this instruction
    pub phext_addr: PhextCoord,
    /// Dependency flags linking to prior SIW
    pub deps: Deps,
}

impl SIW {
    /// Create a fully-specified SIW
    pub fn new(
        d_op: DenseOp,
        s_op: SparseOp,
        c_op: CoordOp,
        phext_addr: PhextCoord,
        deps: Deps,
    ) -> Self {
        Self { d_op, s_op, c_op, phext_addr, deps }
    }

    /// Create a NOP SIW (all three pipes idle)
    pub fn nop() -> Self {
        Self {
            d_op: DenseOp::Nop,
            s_op: SparseOp::Nop,
            c_op: CoordOp::Nop,
            phext_addr: PhextCoord::BASE,
            deps: Deps::NONE,
        }
    }

    /// True if all three pipes are NOPs
    pub fn is_nop(&self) -> bool {
        matches!(self.d_op, DenseOp::Nop)
            && matches!(self.s_op, SparseOp::Nop)
            && matches!(self.c_op, CoordOp::Nop)
    }

    /// Count of active (non-NOP) pipes in this SIW
    pub fn active_pipes(&self) -> u8 {
        let d = if matches!(self.d_op, DenseOp::Nop) { 0 } else { 1 };
        let s = if matches!(self.s_op, SparseOp::Nop) { 0 } else { 1 };
        let c = if matches!(self.c_op, CoordOp::Nop) { 0 } else { 1 };
        d + s + c
    }
}

/// A stream of SIWs ready for execution by a sentron.
/// The instruction stream is itself addressable as a phext —
/// a flat array of SIWs at sequential coordinates.
#[derive(Debug, Clone)]
pub struct SIWStream {
    /// The instructions
    pub instructions: Vec<SIW>,
    /// Base phext coordinate where this stream is located
    pub base_coord: PhextCoord,
}

impl SIWStream {
    pub fn new(base_coord: PhextCoord) -> Self {
        Self {
            instructions: Vec::new(),
            base_coord,
        }
    }

    pub fn push(&mut self, siw: SIW) {
        self.instructions.push(siw);
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Calculate utilization: ratio of active pipe-slots to total pipe-slots
    pub fn utilization(&self) -> f64 {
        if self.instructions.is_empty() {
            return 0.0;
        }
        let active: u32 = self.instructions.iter()
            .map(|siw| siw.active_pipes() as u32)
            .sum();
        let total = self.instructions.len() as u32 * 3; // 3 pipes per SIW
        active as f64 / total as f64
    }

    /// Count of SIWs with all 3 pipes active (the target state)
    pub fn fully_packed_count(&self) -> usize {
        self.instructions.iter()
            .filter(|siw| siw.active_pipes() == 3)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe::*;

    #[test]
    fn nop_siw() {
        let nop = SIW::nop();
        assert!(nop.is_nop());
        assert_eq!(nop.active_pipes(), 0);
    }

    #[test]
    fn three_wide_siw() {
        let siw = SIW::new(
            DenseOp::Add { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::Prefetch { coord_reg: 0, hint: CacheHint::L2 },
            CoordOp::Pack { rd: 0, rs1: 1, rs2: 2, fmt: 0 },
            PhextCoord::from_phext(3,1,4, 1,5,9, 2,6,5),
            Deps::NONE,
        );
        assert_eq!(siw.active_pipes(), 3);
        assert!(!siw.is_nop());
    }

    #[test]
    fn stream_utilization() {
        let mut stream = SIWStream::new(PhextCoord::BASE);

        // One fully packed SIW
        stream.push(SIW::new(
            DenseOp::Mov { rd: 0, imm: 42 },
            SparseOp::Prefetch { coord_reg: 0, hint: CacheHint::L1 },
            CoordOp::Nop,
            PhextCoord::BASE,
            Deps::NONE,
        ));
        // 2 active out of 3 = 66.7%
        assert!((stream.utilization() - 0.6667).abs() < 0.01);

        // Add a fully packed SIW
        stream.push(SIW::new(
            DenseOp::Add { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::Gather { rd: 3, coord_reg: 0, width: 64 },
            CoordOp::Barrier { id: 0, count: 6 },
            PhextCoord::BASE,
            Deps::NONE,
        ));
        // 5 active out of 6 = 83.3%
        assert!((stream.utilization() - 0.8333).abs() < 0.01);
        assert_eq!(stream.fully_packed_count(), 1);
    }

    #[test]
    fn deps_patterns() {
        assert!(Deps::NONE.is_independent());
        assert!(!Deps::S_FROM_D.is_independent());
        assert!(Deps::S_FROM_D.has(Deps::S_FROM_D));
        assert!(!Deps::S_FROM_D.has(Deps::C_FROM_D));

        let merged = Deps::S_FROM_D.merge(Deps::C_FROM_D);
        assert!(merged.has(Deps::S_FROM_D));
        assert!(merged.has(Deps::C_FROM_D));
    }

    /// The double-buffer pipeline pattern from §7.3 of the spec:
    /// D computes on data[K], S fetches data[K+2], C sends result[K-1]
    /// All three SIWs should have Deps::NONE (fully independent)
    #[test]
    fn double_buffer_pattern() {
        let coord_k = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);
        let coord_k2 = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,3);

        let siw_n = SIW::new(
            DenseOp::Fma { rd: 2, rs1: 3, rs2: 4, rs3: 1 },
            SparseOp::Prefetch { coord_reg: 0, hint: CacheHint::L2 },
            CoordOp::Send { msg_reg: 0, dest: 17 },
            coord_k2,
            Deps::DOUBLE_BUFFER,
        );

        assert_eq!(siw_n.active_pipes(), 3);
        assert!(siw_n.deps.is_independent());
    }
}
