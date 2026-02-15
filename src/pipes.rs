//! vTPU Pipeline Operations
//!
//! Defines opcodes for the three execution pipes:
//! - D-Pipe: Dense/ALU operations (arithmetic, comparison, branching)
//! - S-Pipe: Sparse/Memory operations (phext coordinate addressing)
//! - C-Pipe: Coordination operations (inter-sentron communication)

/// Dense Pipeline Operations (ALU/arithmetic)
///
/// Maps to Zen 4 ALU Port 0/1 (integer) or FP pipes (floating-point).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DenseOp {
    /// rd = rs1 * rs2 + rs3 (fused multiply-add)
    DFMA { rd: u8, rs1: u8, rs2: u8, rs3: u8 },
    
    /// rd = rs1 + rs2
    DADD { rd: u8, rs1: u8, rs2: u8 },
    
    /// rd = rs1 - rs2
    DSUB { rd: u8, rs1: u8, rs2: u8 },
    
    /// rd = rs1 * rs2
    DMUL { rd: u8, rs1: u8, rs2: u8 },
    
    /// rd = compare(rs1, rs2), sets flags
    DCMP { rd: u8, rs1: u8, rs2: u8 },
    
    /// rd = reduce(rs1, op) — local reduction
    DRED { rd: u8, rs1: u8, op: ReductionOp },
    
    /// rd = select(rs1, rs2, flags) — branchless select
    DSEL { rd: u8, rs1: u8, rs2: u8, flags: u8 },
    
    /// rd = immediate value
    DMOV { rd: u8, imm: i64 },
    
    /// No dense operation this cycle
    DNOP,
}

/// Sparse Pipeline Operations (Memory/phext addressing)
///
/// Maps to Zen 4 AGU + Load/Store units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SparseOp {
    /// rd = gather(phext[coord], width bytes)
    ///
    /// Loads `width` bytes from phext coordinate into register rd.
    /// coord_idx points to which phext register holds the coordinate.
    SGATHER { rd: u8, coord_idx: u8, width: u16 },
    
    /// phext[coord] = scatter(rs, width bytes)
    ///
    /// Stores `width` bytes from register rs to phext coordinate.
    SSCATTR { coord_idx: u8, rs: u8, width: u16 },
    
    /// rd = index into phext along dimension dim
    SINDEX { rd: u8, base: u8, offset: i32, dim: u8 },
    
    /// rd = deduplicated lookup in embedding table
    SDEDUP { rd: u8, rs: u8, table_id: u16 },
    
    /// Prefetch phext region (hint for L1/L2/L3)
    SPREFCH { coord_idx: u8, hint: PrefetchHint },
    
    /// Flush modified phext region to DDR5
    SFLUSH { coord_idx: u8, width: u16 },
    
    /// rd = allocate phext region across specified dimensions
    SALLOC { rd: u8, size: u32, dim_mask: u16 },
    
    /// Release phext region
    SFREE { coord_idx: u8, size: u32 },
    
    /// No sparse operation this cycle
    SNOP,
}

/// Coordination Pipeline Operations (Inter-sentron communication)
///
/// Maps to Zen 4 ALU Port 2/3 or SIMD units (for message packing).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordOp {
    /// Pack rs1, rs2 into message format fmt
    CPACK { rd: u8, rs1: u8, rs2: u8, fmt: MessageFormat },
    
    /// Route message to destination node
    CROUTE { msg_reg: u8, dest_node: u8 },
    
    /// Send to specific sentron (intra-node: register move)
    CSEND { msg_reg: u8, dest_sentron: u8 },
    
    /// Receive from sentron (blocks until available)
    CRECV { rd: u8, src_sentron: u8 },
    
    /// Barrier synchronization across count sentrons
    CBAR { barrier_id: u8, count: u8 },
    
    /// Memory fence (scope: node-local, cluster-wide)
    CFENCE { scope: FenceScope },
    
    /// All-reduce across sentron group
    CREDUCE { rd: u8, rs: u8, op: ReductionOp, group: u8 },
    
    /// Broadcast to sentron group
    CCAST { rs: u8, group: u8 },
    
    /// No coordination operation this cycle
    CNOP,
}

/// Reduction operation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReductionOp {
    Sum,
    Max,
    Min,
    And,
    Or,
    Xor,
}

/// Prefetch hint levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrefetchHint {
    L1,   // Prefetch to L1 cache
    L2,   // Prefetch to L2 cache
    L3,   // Prefetch to L3 cache
    NTA,  // Non-temporal (bypass cache)
}

/// Message format for C-Pipe packing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageFormat {
    Result,       // Standard result message
    Request,      // Request message
    Barrier,      // Barrier sync message
    Custom(u8),   // Custom format (user-defined)
}

/// Memory fence scope
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FenceScope {
    Thread,      // Thread-local (no-op, for completeness)
    Core,        // Core-local (L1/L2 fence)
    Node,        // Node-local (L3 + DDR5 fence)
    Cluster,     // Cluster-wide (cross-node fence)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dense_ops() {
        let add = DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 };
        assert!(matches!(add, DenseOp::DADD { .. }));
        
        let nop = DenseOp::DNOP;
        assert!(matches!(nop, DenseOp::DNOP));
    }
    
    #[test]
    fn test_sparse_ops() {
        let gather = SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 };
        assert!(matches!(gather, SparseOp::SGATHER { .. }));
        
        let prefetch = SparseOp::SPREFCH { coord_idx: 2, hint: PrefetchHint::L2 };
        assert!(matches!(prefetch, SparseOp::SPREFCH { .. }));
    }
    
    #[test]
    fn test_coord_ops() {
        let send = CoordOp::CSEND { msg_reg: 0, dest_sentron: 5 };
        assert!(matches!(send, CoordOp::CSEND { .. }));
        
        let barrier = CoordOp::CBAR { barrier_id: 1, count: 6 };
        assert!(matches!(barrier, CoordOp::CBAR { .. }));
    }
}
