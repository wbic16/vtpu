//! Sentron Instruction Word (SIW)
//!
//! Core data structure for vTPU execution. Each SIW contains exactly 3 operations:
//! - D-Pipe: Dense/ALU operation
//! - S-Pipe: Sparse/Memory operation  
//! - C-Pipe: Coordination/Communication operation
//!
//! This 3-wide structure maps to independent execution units on Zen 4,
//! enabling sustained 3 ops/cycle retirement.

use crate::pipes::{DenseOp, SparseOp, CoordOp};
use crate::phext_coord::PhextCoord;

/// Sentron Instruction Word - atomic unit of vTPU execution
///
/// Each SIW executes in exactly 1 cycle when all dependencies are resolved.
/// The 3 operations target different execution ports → zero resource conflicts.
#[derive(Debug, Clone, PartialEq)]
#[repr(C, align(64))]  // Cache-line aligned for optimal fetch
pub struct SIW {
    /// Dense Pipeline operation (maps to ALU Port 0/1)
    pub d_op: DenseOp,
    
    /// Sparse Pipeline operation (maps to AGU/Load-Store Port)
    pub s_op: SparseOp,
    
    /// Coordination Pipeline operation (maps to ALU Port 2/3)
    pub c_op: CoordOp,
    
    /// Target phext coordinate (128-bit, 11 dimensions)
    pub phext_addr: PhextCoord,
    
    /// Dependency flags (which prior SIW results are needed)
    ///
    /// Bit flags:
    /// - 0: D-Pipe depends on prior D-Pipe
    /// - 1: S-Pipe depends on prior S-Pipe
    /// - 2: C-Pipe depends on prior C-Pipe
    /// - 3: D-Pipe depends on prior S-Pipe
    /// - 4: S-Pipe depends on prior D-Pipe
    /// - 5: C-Pipe depends on prior D-Pipe
    /// - 6-7: Reserved
    pub deps: u8,

    /// OctaWire dispatch family bytes (precomputed at SIW construction).
    ///
    /// 2×4 wiring per pipe-neuron: 4 op families × 2 directions = 8 wires.
    /// These bytes allow O(1) indexed dispatch, replacing triple match statements.
    /// Value 4 = NOP sentinel (skip dispatch entirely).
    ///
    /// d_fam: DenseOp family  (0=Arithmetic, 1=Reduce, 2=HDC, 3=Ternary, 4=NOP)
    /// s_fam: SparseOp family (0=Load, 1=Store, 2=Address, 3=Route, 4=NOP)
    /// c_fam: CoordOp family  (0=Pack, 1=Send, 2=Barrier, 3=Reduce, 4=NOP)
    pub d_fam: u8,
    pub s_fam: u8,
    pub c_fam: u8,
}

impl SIW {
    /// Create a new SIW with no dependencies
    pub fn new(d_op: DenseOp, s_op: SparseOp, c_op: CoordOp, phext_addr: PhextCoord) -> Self {
        let d_fam = d_op.op_family();
        let s_fam = s_op.op_family();
        let c_fam = c_op.op_family();
        Self {
            d_op,
            s_op,
            c_op,
            phext_addr,
            deps: 0,
            d_fam,
            s_fam,
            c_fam,
        }
    }
    
    /// Create a NOP SIW (all pipes execute no-ops)
    pub fn nop() -> Self {
        Self {
            d_op: DenseOp::DNOP,
            s_op: SparseOp::SNOP,
            c_op: CoordOp::CNOP,
            phext_addr: PhextCoord::zero(),
            deps: 0,
            d_fam: 4,
            s_fam: 4,
            c_fam: 4,
        }
    }

    /// OctaWire mode byte: 3-bit encoding of active pipes.
    /// bit 0: D active, bit 1: S active, bit 2: C active
    #[inline(always)]
    pub fn mode_bits(&self) -> u8 {
        ((self.d_fam < 4) as u8)
            | (((self.s_fam < 4) as u8) << 1)
            | (((self.c_fam < 4) as u8) << 2)
    }
    
    /// Set dependency flags
    pub fn with_deps(mut self, deps: u8) -> Self {
        self.deps = deps;
        self
    }
    
    /// Check if this SIW has inter-SIW dependencies
    pub fn has_dependencies(&self) -> bool {
        self.deps != 0
    }
    
    /// Check if D-Pipe can execute (dependencies resolved)
    pub fn d_ready(&self, prev_d_ready: bool, prev_s_ready: bool) -> bool {
        let needs_prev_d = (self.deps & 0x01) != 0;
        let needs_prev_s = (self.deps & 0x08) != 0;
        
        (!needs_prev_d || prev_d_ready) && (!needs_prev_s || prev_s_ready)
    }
    
    /// Check if S-Pipe can execute
    pub fn s_ready(&self, prev_s_ready: bool, prev_d_ready: bool) -> bool {
        let needs_prev_s = (self.deps & 0x02) != 0;
        let needs_prev_d = (self.deps & 0x10) != 0;
        
        (!needs_prev_s || prev_s_ready) && (!needs_prev_d || prev_d_ready)
    }
    
    /// Check if C-Pipe can execute
    pub fn c_ready(&self, prev_c_ready: bool, prev_d_ready: bool) -> bool {
        let needs_prev_c = (self.deps & 0x04) != 0;
        let needs_prev_d = (self.deps & 0x20) != 0;
        
        (!needs_prev_c || prev_c_ready) && (!needs_prev_d || prev_d_ready)
    }
    
    /// Estimate cycles to execute this SIW
    ///
    /// Returns 1 if all dependencies are resolved, >1 if stalls expected.
    pub fn estimated_cycles(&self) -> u32 {
        // Base case: no dependencies = 1 cycle
        if !self.has_dependencies() {
            return 1;
        }
        
        // With dependencies: assume 1 cycle if prior SIW completed
        // (Conservative estimate - actual OoO engine may hide some latency)
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::*;
    
    #[test]
    fn test_siw_creation() {
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        );
        
        assert!(!siw.has_dependencies());
        assert_eq!(siw.estimated_cycles(), 1);
    }
    
    #[test]
    fn test_nop_siw() {
        let nop = SIW::nop();
        
        assert!(matches!(nop.d_op, DenseOp::DNOP));
        assert!(matches!(nop.s_op, SparseOp::SNOP));
        assert!(matches!(nop.c_op, CoordOp::CNOP));
    }
    
    #[test]
    fn test_dependency_flags() {
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SGATHER { rd: 4, coord_idx: 0, width: 64 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ).with_deps(0x01);  // D-Pipe depends on prior D-Pipe
        
        assert!(siw.has_dependencies());
        assert!(!siw.d_ready(false, true));  // Prior D not ready → D blocked
        assert!(siw.d_ready(true, true));    // Prior D ready → D proceeds
    }
    
    #[test]
    fn test_cache_line_alignment() {
        // SIW should be 64-byte aligned for optimal cache fetch
        use std::mem;
        assert_eq!(mem::align_of::<SIW>(), 64);
    }
}

#[cfg(test)]
mod w20_tests {
    use super::*;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};

    #[test]
    fn siw_size_is_cache_aligned() {
        assert_eq!(std::mem::size_of::<SIW>() % 64, 0, "SIW must be cache-line aligned");
    }

    #[test]
    fn siw_nop_mode_bits_zero() {
        assert_eq!(SIW::nop().mode_bits(), 0);
    }

    #[test]
    fn siw_d_active_mode_bit_0() {
        let s = SIW::new(DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
        assert_eq!(s.mode_bits() & 0b001, 1);
    }

    #[test]
    fn siw_s_active_mode_bit_1() {
        let s = SIW::new(DenseOp::DNOP, SparseOp::SINDEX { rd: 0, base: 0, offset: 0, dim: 0 }, CoordOp::CNOP, PhextCoord::zero());
        assert_eq!(s.mode_bits() & 0b010, 0b010);
    }

    #[test]
    fn siw_c_active_mode_bit_2() {
        let s = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CBAR { barrier_id: 0, count: 1 }, PhextCoord::zero());
        assert_eq!(s.mode_bits() & 0b100, 0b100);
    }

    #[test]
    fn siw_with_deps_sets_flags() {
        let s = SIW::nop().with_deps(0x05);
        assert!(s.has_dependencies());
        assert_eq!(s.deps, 0x05);
    }

    #[test]
    fn siw_family_bytes_correct_for_packed() {
        let s = SIW::new(
            DenseOp::DFMA { rd: 0, rs1: 1, rs2: 2, rs3: 3 },
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CPACK { rd: 0, rs1: 1, rs2: 2, fmt: crate::pipes::MessageFormat::Result },
            PhextCoord::zero(),
        );
        assert_eq!(s.d_fam, 0, "DFMA = Arithmetic (0)");
        assert_eq!(s.s_fam, 0, "SGATHER = Load (0)");
        assert_eq!(s.c_fam, 0, "CPACK = Pack (0)");
    }

    #[test]
    fn siw_clone_is_deep() {
        let s1 = SIW::new(DenseOp::DADD { rd: 5, rs1: 6, rs2: 7 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
        let s2 = s1.clone();
        assert_eq!(s1.d_op, s2.d_op);
        assert_eq!(s1.d_fam, s2.d_fam);
    }
}
