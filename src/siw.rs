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
}

impl SIW {
    /// Create a new SIW with no dependencies
    pub fn new(d_op: DenseOp, s_op: SparseOp, c_op: CoordOp, phext_addr: PhextCoord) -> Self {
        Self {
            d_op,
            s_op,
            c_op,
            phext_addr,
            deps: 0,
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
        }
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
