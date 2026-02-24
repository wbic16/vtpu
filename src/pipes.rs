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
    // v0.2: Hyperdimensional Computing
    DHDENC { rd: u8, rs: u8, width: u16 },
    DHDBIND { rd: u8, rs1: u8, rs2: u8 },
    DHDBUND { rd: u8, rs1: u8, rs2: u8 },
    DHDPERM { rd: u8, rs: u8, k: u16 },
    DHDSIM { rd: u8, rs1: u8, rs2: u8 },

    // v0.3: BitNet Ternary Mode — weights are {-1, 0, 1}, no FPU needed
    /// rd = ternary_select(rs1, trit_reg): apply packed trits to activation
    /// trit_reg holds packed 2-bit trits: 00=zero, 01=+1, 10=-1
    /// Result: sum of (activation[i] * trit[i]) across packed elements
    DTERNARY { rd: u8, rs1: u8, trit_reg: u8 },

    /// rd = popcount of non-zero trits in rs (sparsity measure)
    DTPOP { rd: u8, rs: u8 },

    /// rd = ternary accumulate: rd += ternary_select(rs1, trit_reg)
    DTACC { rd: u8, rs1: u8, trit_reg: u8 },

    DNOP,
}

impl DenseOp {
    /// OctaWire: return the op-family index (0-3) for indexed dispatch.
    ///
    /// 4 families (the "4 connections" in 2×4 wiring per pipe-neuron):
    ///   0 = Arithmetic  (DADD, DSUB, DMUL, DFMA, DCMP, DSEL, DMOV)
    ///   1 = Reduce      (DRED)
    ///   2 = HDC         (DHDENC, DHDBIND, DHDBUND, DHDPERM, DHDSIM)
    ///   3 = Ternary     (DTERNARY, DTPOP, DTACC)
    ///   4 = NOP sentinel (skip dispatch)
    #[inline(always)]
    pub fn op_family(&self) -> u8 {
        match self {
            DenseOp::DADD { .. } | DenseOp::DSUB { .. } | DenseOp::DMUL { .. }
            | DenseOp::DFMA { .. } | DenseOp::DCMP { .. }
            | DenseOp::DSEL { .. } | DenseOp::DMOV { .. } => 0,
            DenseOp::DRED { .. } => 1,
            DenseOp::DHDENC { .. } | DenseOp::DHDBIND { .. } | DenseOp::DHDBUND { .. }
            | DenseOp::DHDPERM { .. } | DenseOp::DHDSIM { .. } => 2,
            DenseOp::DTERNARY { .. } | DenseOp::DTPOP { .. } | DenseOp::DTACC { .. } => 3,
            DenseOp::DNOP => 4,
        }
    }
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
    // v0.2: Associative & Routing
    SASSOC { rd: u8, coord_reg: u8, match_mode: MatchMode },
    SROUTE { rd: u8, embedding_reg: u8, dim_mask: u16 },
    SNEIGHBR { rd: u8, coord_reg: u8, radius: u8 },
    SNOP,
}

impl SparseOp {
    /// OctaWire: return the op-family index (0-3) for indexed dispatch.
    ///
    /// 4 families (the "4 connections" in 2×4 wiring per pipe-neuron):
    ///   0 = Load    (SGATHER, SDEDUP)
    ///   1 = Store   (SSCATTR, SFLUSH)
    ///   2 = Address (SINDEX, SALLOC, SFREE)
    ///   3 = Route   (SPREFCH, SASSOC, SROUTE, SNEIGHBR)
    ///   4 = NOP sentinel (skip dispatch)
    #[inline(always)]
    pub fn op_family(&self) -> u8 {
        match self {
            SparseOp::SGATHER { .. } | SparseOp::SDEDUP { .. } => 0,
            SparseOp::SSCATTR { .. } | SparseOp::SFLUSH { .. } => 1,
            SparseOp::SINDEX { .. } | SparseOp::SALLOC { .. } | SparseOp::SFREE { .. } => 2,
            SparseOp::SPREFCH { .. } | SparseOp::SASSOC { .. }
            | SparseOp::SROUTE { .. } | SparseOp::SNEIGHBR { .. } => 3,
            SparseOp::SNOP => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchMode { First, All, Nearest, Count }

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
    // v0.2: Attention Geometry
    CSLICE { group: u8, dim_triple: [u8; 3], range_start: u16, range_end: u16 },
    CFANOUT { msg_reg: u8, coord_pattern: u8 },
    CMERGE { rd: u8, group: u8, op: MergeOp },
    CNOP,
}

impl CoordOp {
    /// OctaWire: return the op-family index (0-3) for indexed dispatch.
    ///
    /// 4 families (the "4 connections" in 2×4 wiring per pipe-neuron):
    ///   0 = Pack    (CPACK)
    ///   1 = Send    (CSEND, CRECV, CROUTE, CFANOUT)
    ///   2 = Barrier (CBAR, CFENCE)
    ///   3 = Reduce  (CREDUCE, CCAST, CSLICE, CMERGE)
    ///   4 = NOP sentinel (skip dispatch)
    #[inline(always)]
    pub fn op_family(&self) -> u8 {
        match self {
            CoordOp::CPACK { .. } => 0,
            CoordOp::CSEND { .. } | CoordOp::CRECV { .. }
            | CoordOp::CROUTE { .. } | CoordOp::CFANOUT { .. } => 1,
            CoordOp::CBAR { .. } | CoordOp::CFENCE { .. } => 2,
            CoordOp::CREDUCE { .. } | CoordOp::CCAST { .. }
            | CoordOp::CSLICE { .. } | CoordOp::CMERGE { .. } => 3,
            CoordOp::CNOP => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeOp { Concat, Sum, Max, Vote }

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

    #[test]
    fn test_dense_op_families() {
        // Arithmetic family (0)
        assert_eq!(DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 }.op_family(), 0);
        assert_eq!(DenseOp::DSUB { rd: 0, rs1: 1, rs2: 2 }.op_family(), 0);
        assert_eq!(DenseOp::DMUL { rd: 0, rs1: 1, rs2: 2 }.op_family(), 0);
        assert_eq!(DenseOp::DFMA { rd: 0, rs1: 1, rs2: 2, rs3: 3 }.op_family(), 0);
        // Reduce family (1)
        assert_eq!(DenseOp::DRED { rd: 0, rs1: 1, op: ReductionOp::Sum }.op_family(), 1);
        // HDC family (2)
        assert_eq!(DenseOp::DHDENC { rd: 0, rs: 1, width: 256 }.op_family(), 2);
        assert_eq!(DenseOp::DHDBIND { rd: 0, rs1: 1, rs2: 2 }.op_family(), 2);
        // Ternary family (3)
        assert_eq!(DenseOp::DTERNARY { rd: 0, rs1: 1, trit_reg: 2 }.op_family(), 3);
        // NOP sentinel (4)
        assert_eq!(DenseOp::DNOP.op_family(), 4);
    }

    #[test]
    fn test_sparse_op_families() {
        // Load family (0)
        assert_eq!(SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 64 }.op_family(), 0);
        // Store family (1)
        assert_eq!(SparseOp::SSCATTR { coord_idx: 0, rs: 0, width: 64 }.op_family(), 1);
        // NOP sentinel (4)
        assert_eq!(SparseOp::SNOP.op_family(), 4);
    }

    #[test]
    fn test_coord_op_families() {
        // Pack family (0)
        assert_eq!(CoordOp::CPACK { rd: 0, rs1: 1, rs2: 2, fmt: MessageFormat::Result }.op_family(), 0);
        // Send family (1)
        assert_eq!(CoordOp::CSEND { msg_reg: 0, dest_sentron: 1 }.op_family(), 1);
        assert_eq!(CoordOp::CRECV { rd: 0, src_sentron: 1 }.op_family(), 1);
        // Barrier family (2)
        assert_eq!(CoordOp::CBAR { barrier_id: 0, count: 4 }.op_family(), 2);
        assert_eq!(CoordOp::CFENCE { scope: FenceScope::Core }.op_family(), 2);
        // Reduce family (3)
        assert_eq!(CoordOp::CREDUCE { rd: 0, rs: 1, op: ReductionOp::Sum, group: 0 }.op_family(), 3);
        // NOP sentinel (4)
        assert_eq!(CoordOp::CNOP.op_family(), 4);
    }

    #[test]
    fn test_reduction_op_variants() {
        let ops = [ReductionOp::Sum, ReductionOp::Max, ReductionOp::Min,
                    ReductionOp::And, ReductionOp::Or, ReductionOp::Xor];
        assert_eq!(ops.len(), 6);
        // All distinct
        for i in 0..ops.len() {
            for j in (i+1)..ops.len() {
                assert_ne!(ops[i], ops[j]);
            }
        }
    }

    #[test]
    fn test_prefetch_hints() {
        let hints = [PrefetchHint::L1, PrefetchHint::L2, PrefetchHint::L3, PrefetchHint::NTA];
        assert_eq!(hints.len(), 4);
        for i in 0..hints.len() {
            for j in (i+1)..hints.len() {
                assert_ne!(hints[i], hints[j]);
            }
        }
    }

    #[test]
    fn test_fence_scopes() {
        let scopes = [FenceScope::Thread, FenceScope::Core, FenceScope::Node, FenceScope::Cluster];
        assert_eq!(scopes.len(), 4);
        for i in 0..scopes.len() {
            for j in (i+1)..scopes.len() {
                assert_ne!(scopes[i], scopes[j]);
            }
        }
    }

    #[test]
    fn test_message_formats() {
        assert_ne!(MessageFormat::Result, MessageFormat::Request);
        assert_ne!(MessageFormat::Barrier, MessageFormat::Custom(0));
        assert_eq!(MessageFormat::Custom(42), MessageFormat::Custom(42));
        assert_ne!(MessageFormat::Custom(1), MessageFormat::Custom(2));
    }
}
