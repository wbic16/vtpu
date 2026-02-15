//! SIW Stream Builder and Utilities
//!
//! Fluent API for constructing SIW sequences with automatic dependency tracking.

use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp, CoordOp};
use crate::phext_coord::PhextCoord;

/// Builder for SIW streams with automatic dependency analysis
pub struct StreamBuilder {
    siws: Vec<SIW>,
    last_d_write: Option<u8>,  // Last register written by D-Pipe
    last_s_write: Option<u8>,  // Last register written by S-Pipe
    last_c_write: Option<u8>,  // Last register written by C-Pipe
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self {
            siws: Vec::new(),
            last_d_write: None,
            last_s_write: None,
            last_c_write: None,
        }
    }
    
    /// Add a SIW with automatic dependency detection
    pub fn push(&mut self, siw: SIW) -> &mut Self {
        let mut deps = 0u8;
        
        // Check D-Pipe dependencies
        if let Some(last_d) = self.last_d_write {
            if reads_register(&siw.d_op, last_d) {
                deps |= 0x01;  // D depends on prior D
            }
        }
        if let Some(last_s) = self.last_s_write {
            if reads_register(&siw.d_op, last_s) {
                deps |= 0x08;  // D depends on prior S
            }
        }
        
        // Check S-Pipe dependencies
        if let Some(last_s) = self.last_s_write {
            if reads_sparse_register(&siw.s_op, last_s) {
                deps |= 0x02;  // S depends on prior S
            }
        }
        if let Some(last_d) = self.last_d_write {
            if reads_sparse_register(&siw.s_op, last_d) {
                deps |= 0x10;  // S depends on prior D
            }
        }
        
        // Check C-Pipe dependencies
        if let Some(last_c) = self.last_c_write {
            if reads_coord_register(&siw.c_op, last_c) {
                deps |= 0x04;  // C depends on prior C
            }
        }
        if let Some(last_d) = self.last_d_write {
            if reads_coord_register(&siw.c_op, last_d) {
                deps |= 0x20;  // C depends on prior D
            }
        }
        
        // Track what this SIW writes
        self.last_d_write = get_write_register(&siw.d_op);
        self.last_s_write = get_sparse_write_register(&siw.s_op);
        self.last_c_write = get_coord_write_register(&siw.c_op);
        
        self.siws.push(siw.with_deps(deps));
        self
    }
    
    /// Add a NOP SIW (pipeline bubble)
    pub fn nop(&mut self) -> &mut Self {
        self.push(SIW::nop())
    }
    
    /// Build the final SIW stream
    pub fn build(self) -> Vec<SIW> {
        self.siws
    }
    
    /// Get current length
    pub fn len(&self) -> usize {
        self.siws.len()
    }
    
    /// Check if stream is empty
    pub fn is_empty(&self) -> bool {
        self.siws.is_empty()
    }
}

impl Default for StreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Helper: check if dense op reads a register
fn reads_register(op: &DenseOp, reg: u8) -> bool {
    match op {
        DenseOp::DFMA { rs1, rs2, rs3, .. } => *rs1 == reg || *rs2 == reg || *rs3 == reg,
        DenseOp::DADD { rs1, rs2, .. } => *rs1 == reg || *rs2 == reg,
        DenseOp::DSUB { rs1, rs2, .. } => *rs1 == reg || *rs2 == reg,
        DenseOp::DMUL { rs1, rs2, .. } => *rs1 == reg || *rs2 == reg,
        DenseOp::DCMP { rs1, rs2, .. } => *rs1 == reg || *rs2 == reg,
        DenseOp::DRED { rs1, .. } => *rs1 == reg,
        DenseOp::DSEL { rs1, rs2, .. } => *rs1 == reg || *rs2 == reg,
        DenseOp::DMOV { .. } | DenseOp::DNOP => false,
    }
}

// Helper: check if sparse op reads a register
fn reads_sparse_register(op: &SparseOp, reg: u8) -> bool {
    match op {
        SparseOp::SSCATTR { rs, .. } => *rs == reg,
        SparseOp::SINDEX { base, .. } => *base == reg,
        SparseOp::SDEDUP { rs, .. } => *rs == reg,
        _ => false,
    }
}

// Helper: check if coord op reads a register
fn reads_coord_register(op: &CoordOp, reg: u8) -> bool {
    match op {
        CoordOp::CPACK { rs1, rs2, .. } => *rs1 == reg || *rs2 == reg,
        CoordOp::CROUTE { msg_reg, .. } => *msg_reg == reg,
        CoordOp::CSEND { msg_reg, .. } => *msg_reg == reg,
        CoordOp::CCAST { rs, .. } => *rs == reg,
        CoordOp::CREDUCE { rs, .. } => *rs == reg,
        _ => false,
    }
}

// Helper: get write register for dense op
fn get_write_register(op: &DenseOp) -> Option<u8> {
    match op {
        DenseOp::DFMA { rd, .. } |
        DenseOp::DADD { rd, .. } |
        DenseOp::DSUB { rd, .. } |
        DenseOp::DMUL { rd, .. } |
        DenseOp::DCMP { rd, .. } |
        DenseOp::DRED { rd, .. } |
        DenseOp::DSEL { rd, .. } |
        DenseOp::DMOV { rd, .. } => Some(*rd),
        DenseOp::DNOP => None,
    }
}

// Helper: get write register for sparse op
fn get_sparse_write_register(op: &SparseOp) -> Option<u8> {
    match op {
        SparseOp::SGATHER { rd, .. } |
        SparseOp::SINDEX { rd, .. } |
        SparseOp::SDEDUP { rd, .. } |
        SparseOp::SALLOC { rd, .. } => Some(*rd),
        _ => None,
    }
}

// Helper: get write register for coord op
fn get_coord_write_register(op: &CoordOp) -> Option<u8> {
    match op {
        CoordOp::CPACK { rd, .. } |
        CoordOp::CRECV { rd, .. } |
        CoordOp::CREDUCE { rd, .. } => Some(*rd),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builder_empty() {
        let builder = StreamBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }
    
    #[test]
    fn test_builder_single_siw() {
        let mut builder = StreamBuilder::new();
        
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        );
        
        builder.push(siw);
        
        assert_eq!(builder.len(), 1);
        let stream = builder.build();
        assert_eq!(stream.len(), 1);
    }
    
    #[test]
    fn test_dependency_detection() {
        let mut builder = StreamBuilder::new();
        
        // First SIW: r1 = r2 + r3
        builder.push(SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
        
        // Second SIW: r4 = r1 + r5 (depends on prior D-Pipe)
        builder.push(SIW::new(
            DenseOp::DADD { rd: 4, rs1: 1, rs2: 5 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
        
        let stream = builder.build();
        assert_eq!(stream.len(), 2);
        
        // First SIW should have no deps
        assert_eq!(stream[0].deps, 0);
        
        // Second SIW should depend on prior D-Pipe (bit 0)
        assert_ne!(stream[1].deps & 0x01, 0);
    }
    
    #[test]
    fn test_nop_insertion() {
        let mut builder = StreamBuilder::new();
        builder.nop().nop();
        
        let stream = builder.build();
        assert_eq!(stream.len(), 2);
        assert!(matches!(stream[0].d_op, DenseOp::DNOP));
    }
}
