//! D-Pipe, S-Pipe, C-Pipe operation definitions (svISA)
//!
//! Three pipes, three execution domains, zero intra-SIW dependencies.

/// Register index for the sentron register file
pub type Reg = u8;       // r0-r15 (general), p0-p7 (phext), m0-m3 (message)
pub type BarrierId = u8;
pub type GroupId = u8;
pub type TableId = u8;
pub type NodeId = u8;
pub type SentronId = u16;
pub type MsgFormat = u8;

/// Cache prefetch hint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheHint {
    L1,
    L2,
    L3,
}

/// Reduction operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReduceOp {
    Sum,
    Max,
    Min,
    And,
    Or,
    Xor,
}

/// Memory fence scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceScope {
    /// Fence within this sentron's core
    Local,
    /// Fence across all cores on this node
    Node,
    /// Fence across the entire cluster
    Cluster,
}

// ─── D-Pipe (Dense) ────────────────────────────────────────────

/// Dense pipeline operations — maps to ALU Port 0/1
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DenseOp {
    /// rd = rs1 * rs2 + rs3 (fused multiply-add)
    Fma { rd: Reg, rs1: Reg, rs2: Reg, rs3: Reg },
    /// rd = rs1 + rs2
    Add { rd: Reg, rs1: Reg, rs2: Reg },
    /// rd = rs1 - rs2
    Sub { rd: Reg, rs1: Reg, rs2: Reg },
    /// rd = rs1 * rs2
    Mul { rd: Reg, rs1: Reg, rs2: Reg },
    /// rd = compare(rs1, rs2), sets status flags
    Cmp { rd: Reg, rs1: Reg, rs2: Reg },
    /// rd = reduce(rs1, op) — local reduction
    Red { rd: Reg, rs1: Reg, op: ReduceOp },
    /// rd = select(rs1, rs2, flags) — branchless conditional
    Sel { rd: Reg, rs1: Reg, rs2: Reg, flags: Reg },
    /// rd = immediate value
    Mov { rd: Reg, imm: i64 },
    /// No operation (retires in zero cycles, maintains 3-wide invariant)
    Nop,
}

// ─── S-Pipe (Sparse) ──────────────────────────────────────────

/// Sparse pipeline operations — maps to AGU Port 0/1 (address generation + load/store)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparseOp {
    /// rd = gather(phext[coord], width bytes)
    /// coord comes from a phext register (p0-p7)
    Gather { rd: Reg, coord_reg: Reg, width: u16 },
    /// phext[coord] = scatter(rs, width bytes)
    Scatter { coord_reg: Reg, rs: Reg, width: u16 },
    /// rd = index into phext along dimension dim from base
    Index { rd: Reg, base_reg: Reg, offset: i16, dim: u8 },
    /// rd = deduplicated lookup in embedding table
    Dedup { rd: Reg, rs: Reg, table: TableId },
    /// Prefetch phext region into cache level
    Prefetch { coord_reg: Reg, hint: CacheHint },
    /// Flush modified phext region to DDR5
    Flush { coord_reg: Reg, width: u16 },
    /// rd = allocate phext region across specified dimensions (bitmask)
    Alloc { rd: Reg, size: u32, dim_mask: u16 },
    /// Free phext region
    Free { coord_reg: Reg, size: u32 },
    /// No operation
    Nop,
}

// ─── C-Pipe (Coordination) ────────────────────────────────────

/// Coordination pipeline operations — maps to ALU Port 2/3 or FP unit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordOp {
    /// Pack rs1, rs2 into message register rd with format fmt
    Pack { rd: Reg, rs1: Reg, rs2: Reg, fmt: MsgFormat },
    /// Route message register to destination node
    Route { msg_reg: Reg, dest_node: NodeId },
    /// Send message to specific sentron (intra-node = register move)
    Send { msg_reg: Reg, dest: SentronId },
    /// Receive from sentron (blocks until available)
    Recv { rd: Reg, src: SentronId },
    /// Barrier synchronization across count sentrons
    Barrier { id: BarrierId, count: u8 },
    /// Memory fence at specified scope
    Fence { scope: FenceScope },
    /// All-reduce across sentron group
    Reduce { rd: Reg, rs: Reg, op: ReduceOp, group: GroupId },
    /// Broadcast to sentron group
    Cast { rs: Reg, group: GroupId },
    /// No operation
    Nop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_op_sizes() {
        // DenseOp should be small — fits in cache line with SIW
        assert!(std::mem::size_of::<DenseOp>() <= 24);
    }

    #[test]
    fn sparse_op_sizes() {
        assert!(std::mem::size_of::<SparseOp>() <= 16);
    }

    #[test]
    fn coord_op_sizes() {
        assert!(std::mem::size_of::<CoordOp>() <= 8);
    }
}
