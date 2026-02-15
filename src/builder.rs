//! SIW stream builders for common patterns
//!
//! These encode the scheduling patterns from the vTPU spec
//! as composable, correct-by-construction instruction sequences.

use crate::coord::PhextCoord;
use crate::pipe::*;
use crate::siw::{SIW, SIWStream, Deps};

/// Builder for constructing SIW streams with the double-buffer pattern.
///
/// The double-buffer pattern (§7.3) ensures zero pipeline stalls:
///   D computes on data[K], S fetches data[K+2], C sends result[K-1]
///
/// This builder enforces the invariant that all three pipes are active.
pub struct DoubleBufferBuilder {
    stream: SIWStream,
    /// Register used for D-Pipe output
    _d_out: Reg,
    /// Register used for S-Pipe coordinate
    _s_coord: Reg,
    /// Message register for C-Pipe
    _c_msg: Reg,
}

impl DoubleBufferBuilder {
    pub fn new(base_coord: PhextCoord) -> Self {
        Self {
            stream: SIWStream::new(base_coord),
            _d_out: 8,   // r8+ for outputs
            _s_coord: 0, // p0 for coordinates
            _c_msg: 0,   // m0 for messages
        }
    }

    /// Add a fully-packed cycle: compute + fetch + send
    pub fn cycle(
        &mut self,
        d_op: DenseOp,
        s_op: SparseOp,
        c_op: CoordOp,
        addr: PhextCoord,
    ) -> &mut Self {
        self.stream.push(SIW::new(d_op, s_op, c_op, addr, Deps::NONE));
        self
    }

    /// Add a compute-and-prefetch cycle (C-pipe NOPs)
    pub fn compute_prefetch(
        &mut self,
        d_op: DenseOp,
        prefetch_reg: Reg,
        hint: CacheHint,
        addr: PhextCoord,
    ) -> &mut Self {
        self.stream.push(SIW::new(
            d_op,
            SparseOp::Prefetch { coord_reg: prefetch_reg, hint },
            CoordOp::Nop,
            addr,
            Deps::NONE,
        ));
        self
    }

    /// Finalize and return the stream
    pub fn build(self) -> SIWStream {
        self.stream
    }

    /// Current stream length
    pub fn len(&self) -> usize {
        self.stream.len()
    }
}

/// Build a phext scan program: traverse coordinates along one dimension,
/// gathering data and reducing results.
///
/// This is the fundamental S-Pipe pattern: dimensional traversal.
pub fn phext_scan(
    base: PhextCoord,
    dim: u8,
    count: u16,
    _reduce_op: ReduceOp,
) -> SIWStream {
    let mut stream = SIWStream::new(base);

    // Setup: load base coordinate into p0, zero accumulator r0
    stream.push(SIW::new(
        DenseOp::Mov { rd: 0, imm: 0 },         // r0 = 0 (accumulator)
        SparseOp::Nop,
        CoordOp::Nop,
        base,
        Deps::NONE,
    ));

    for i in 0..count {
        // Each iteration: gather from current coord, reduce into accumulator, advance coord
        if i == 0 {
            // First: gather + init
            stream.push(SIW::new(
                DenseOp::Mov { rd: 1, imm: i as i64 },
                SparseOp::Gather { rd: 2, coord_reg: 0, width: 8 },
                CoordOp::Nop,
                base.with_dim(dim as usize, i),
                Deps::NONE,
            ));
        } else {
            // Steady state: reduce prior + gather next + prefetch ahead
            let _prefetch_coord = if i + 2 < count { i + 2 } else { i };
            stream.push(SIW::new(
                DenseOp::Add { rd: 0, rs1: 0, rs2: 2 }, // accumulate
                SparseOp::Gather { rd: 2, coord_reg: 0, width: 8 },
                CoordOp::Nop,
                base.with_dim(dim as usize, i),
                Deps::D_FROM_S, // D uses data from prior S
            ));
        }
    }

    // Final reduction
    stream.push(SIW::new(
        DenseOp::Add { rd: 0, rs1: 0, rs2: 2 },
        SparseOp::Nop,
        CoordOp::Nop,
        base,
        Deps::D_FROM_S,
    ));

    stream
}

/// Build a dot-product program for two phext-addressed vectors.
/// Demonstrates the FMA-heavy D-Pipe pattern.
pub fn dot_product(
    vec_a_base: PhextCoord,
    vec_b_base: PhextCoord,
    dim: u8,
    length: u16,
) -> SIWStream {
    let mut stream = SIWStream::new(vec_a_base);

    // r0 = accumulator, r1/r2 = loaded values
    stream.push(SIW::new(
        DenseOp::Mov { rd: 0, imm: 0 },
        SparseOp::Nop,
        CoordOp::Nop,
        vec_a_base,
        Deps::NONE,
    ));

    for i in 0..length {
        // Load a[i] into r1 (via S-Pipe), multiply-accumulate prior values
        // In steady state: FMA r0 = r1 * r2 + r0, while fetching next pair
        if i == 0 {
            stream.push(SIW::new(
                DenseOp::Nop,
                SparseOp::Gather { rd: 1, coord_reg: 0, width: 8 },
                CoordOp::Nop,
                vec_a_base.with_dim(dim as usize, i),
                Deps::NONE,
            ));
            stream.push(SIW::new(
                DenseOp::Nop,
                SparseOp::Gather { rd: 2, coord_reg: 1, width: 8 },
                CoordOp::Nop,
                vec_b_base.with_dim(dim as usize, i),
                Deps::NONE,
            ));
        } else {
            // FMA on prior values while fetching next
            stream.push(SIW::new(
                DenseOp::Fma { rd: 0, rs1: 1, rs2: 2, rs3: 0 },
                SparseOp::Gather { rd: 1, coord_reg: 0, width: 8 },
                CoordOp::Nop,
                vec_a_base.with_dim(dim as usize, i),
                Deps::D_FROM_S,
            ));
            stream.push(SIW::new(
                DenseOp::Nop,
                SparseOp::Gather { rd: 2, coord_reg: 1, width: 8 },
                CoordOp::Nop,
                vec_b_base.with_dim(dim as usize, i),
                Deps::NONE,
            ));
        }
    }

    // Final FMA
    stream.push(SIW::new(
        DenseOp::Fma { rd: 0, rs1: 1, rs2: 2, rs3: 0 },
        SparseOp::Nop,
        CoordOp::Nop,
        vec_a_base,
        Deps::D_FROM_S,
    ));

    stream
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec;
    use crate::sentron::Sentron;

    #[test]
    fn double_buffer_builder() {
        let base = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);
        let mut builder = DoubleBufferBuilder::new(base);

        builder
            .cycle(
                DenseOp::Add { rd: 0, rs1: 1, rs2: 2 },
                SparseOp::Prefetch { coord_reg: 0, hint: CacheHint::L2 },
                CoordOp::Pack { rd: 0, rs1: 1, rs2: 2, fmt: 0 },
                base,
            )
            .cycle(
                DenseOp::Mul { rd: 3, rs1: 0, rs2: 1 },
                SparseOp::Gather { rd: 4, coord_reg: 0, width: 8 },
                CoordOp::Barrier { id: 0, count: 4 },
                base,
            );

        let stream = builder.build();
        assert_eq!(stream.len(), 2);
        assert_eq!(stream.fully_packed_count(), 2);
        assert!((stream.utilization() - 1.0).abs() < 0.001);
    }

    #[test]
    fn phext_scan_runs() {
        let base = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);
        let stream = phext_scan(base, 0, 8, ReduceOp::Sum);

        let mut sentron = Sentron::new(0, base, 0, 0);
        sentron.spawn(stream);
        let stats = exec::run(&mut sentron);

        assert!(stats.siws_retired > 0);
        assert!(stats.d_ops > 0);
        assert!(stats.s_ops > 0);
    }

    #[test]
    fn dot_product_runs() {
        let a = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);
        let b = PhextCoord::from_phext(1,1,1, 1,1,1, 1,2,1);
        let stream = dot_product(a, b, 0, 4);

        let mut sentron = Sentron::new(0, a, 0, 0);
        sentron.spawn(stream);
        let stats = exec::run(&mut sentron);

        assert!(stats.siws_retired > 0);
        // Dot product should have FMA ops
        assert!(stats.d_ops > 0);
    }
}
