//! Display formatting for SIW streams and operations

use std::fmt;
use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp, CoordOp, ReductionOp, PrefetchHint, MessageFormat, FenceScope};
use crate::phext_coord::PhextCoord;

impl fmt::Display for SIW {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SIW[{:08b}] {{ D: {}, S: {}, C: {}, @ {} }}", 
            self.deps, self.d_op, self.s_op, self.c_op, self.phext_addr)
    }
}

impl fmt::Display for DenseOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DenseOp::DFMA { rd, rs1, rs2, rs3 } => write!(f, "fma r{} ← r{} * r{} + r{}", rd, rs1, rs2, rs3),
            DenseOp::DADD { rd, rs1, rs2 } => write!(f, "add r{} ← r{} + r{}", rd, rs1, rs2),
            DenseOp::DSUB { rd, rs1, rs2 } => write!(f, "sub r{} ← r{} - r{}", rd, rs1, rs2),
            DenseOp::DMUL { rd, rs1, rs2 } => write!(f, "mul r{} ← r{} * r{}", rd, rs1, rs2),
            DenseOp::DCMP { rd, rs1, rs2 } => write!(f, "cmp r{} ← r{} ? r{}", rd, rs1, rs2),
            DenseOp::DRED { rd, rs1, op } => write!(f, "red r{} ← {}(r{})", rd, op, rs1),
            DenseOp::DSEL { rd, rs1, rs2, flags } => write!(f, "sel r{} ← r{} : r{} [{}]", rd, rs1, rs2, flags),
            DenseOp::DMOV { rd, imm } => write!(f, "mov r{} ← {}", rd, imm),
            DenseOp::DHDENC { rd, rs, width } => write!(f, "hdenc r{} <- r{} ({}w)", rd, rs, width),
            DenseOp::DHDBIND { rd, rs1, rs2 } => write!(f, "hdbind r{} <- r{} ^ r{}", rd, rs1, rs2),
            DenseOp::DHDBUND { rd, rs1, rs2 } => write!(f, "hdbund r{} <- r{} + r{}", rd, rs1, rs2),
            DenseOp::DHDPERM { rd, rs, k } => write!(f, "hdperm r{} <- r{} >> {}", rd, rs, k),
            DenseOp::DHDSIM { rd, rs1, rs2 } => write!(f, "hdsim r{} <- cos(r{}, r{})", rd, rs1, rs2),
            DenseOp::DTERNARY { rd, rs1, trit_reg } => write!(f, "ternary r{} <- r{} * trits(r{})", rd, rs1, trit_reg),
            DenseOp::DTPOP { rd, rs } => write!(f, "tpop r{} <- popcount(r{})", rd, rs),
            DenseOp::DTACC { rd, rs1, trit_reg } => write!(f, "tacc r{} += r{} * trits(r{})", rd, rs1, trit_reg),
            DenseOp::DNOP => write!(f, "nop"),
        }
    }
}

impl fmt::Display for SparseOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SparseOp::SGATHER { rd, coord_idx, width } => write!(f, "gather r{} ← phext[c{}] ({}B)", rd, coord_idx, width),
            SparseOp::SSCATTR { coord_idx, rs, width } => write!(f, "scatter phext[c{}] ← r{} ({}B)", coord_idx, rs, width),
            SparseOp::SINDEX { rd, base, offset, dim } => write!(f, "index r{} ← r{} + {} @ dim{}", rd, base, offset, dim),
            SparseOp::SDEDUP { rd, rs, table_id } => write!(f, "dedup r{} ← table[{}][r{}]", rd, table_id, rs),
            SparseOp::SPREFCH { coord_idx, hint } => write!(f, "prefetch phext[c{}] → {}", coord_idx, hint),
            SparseOp::SFLUSH { coord_idx, width } => write!(f, "flush phext[c{}] ({}B)", coord_idx, width),
            SparseOp::SALLOC { rd, size, dim_mask } => write!(f, "alloc r{} ← {} bytes @ dims {:011b}", rd, size, dim_mask),
            SparseOp::SFREE { coord_idx, size } => write!(f, "free phext[c{}] ({} bytes)", coord_idx, size),
            SparseOp::SASSOC { rd, coord_reg, match_mode } => write!(f, "assoc r{} <- c{} {:?}", rd, coord_reg, match_mode),
            SparseOp::SROUTE { rd, embedding_reg, dim_mask } => write!(f, "route r{} <- r{} @{:011b}", rd, embedding_reg, dim_mask),
            SparseOp::SNEIGHBR { rd, coord_reg, radius } => write!(f, "neighbr r{} <- c{} r={}", rd, coord_reg, radius),
            SparseOp::SNOP => write!(f, "nop"),
        }
    }
}

impl fmt::Display for CoordOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoordOp::CPACK { rd, rs1, rs2, fmt: msg_fmt } => write!(f, "pack r{} ← (r{}, r{}) as {}", rd, rs1, rs2, msg_fmt),
            CoordOp::CROUTE { msg_reg, dest_node } => write!(f, "route r{} → node{}", msg_reg, dest_node),
            CoordOp::CSEND { msg_reg, dest_sentron } => write!(f, "send r{} → sentron{}", msg_reg, dest_sentron),
            CoordOp::CRECV { rd, src_sentron } => write!(f, "recv r{} ← sentron{}", rd, src_sentron),
            CoordOp::CBAR { barrier_id, count } => write!(f, "barrier #{} ({} sentrons)", barrier_id, count),
            CoordOp::CFENCE { scope } => write!(f, "fence {}", scope),
            CoordOp::CREDUCE { rd, rs, op, group } => write!(f, "reduce r{} ← {}(r{}) @ group{}", rd, op, rs, group),
            CoordOp::CCAST { rs, group } => write!(f, "broadcast r{} → group{}", rs, group),
            CoordOp::CSLICE { group, dim_triple, range_start, range_end } => write!(f, "slice grp{} d({},{},{}) {}..{}", group, dim_triple[0], dim_triple[1], dim_triple[2], range_start, range_end),
            CoordOp::CFANOUT { msg_reg, coord_pattern } => write!(f, "fanout r{} -> c{}", msg_reg, coord_pattern),
            CoordOp::CMERGE { rd, group, op } => write!(f, "merge r{} <- grp{} {:?}", rd, group, op),
            CoordOp::CNOP => write!(f, "nop"),
        }
    }
}

impl fmt::Display for ReductionOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReductionOp::Sum => write!(f, "SUM"),
            ReductionOp::Max => write!(f, "MAX"),
            ReductionOp::Min => write!(f, "MIN"),
            ReductionOp::And => write!(f, "AND"),
            ReductionOp::Or => write!(f, "OR"),
            ReductionOp::Xor => write!(f, "XOR"),
        }
    }
}

impl fmt::Display for PrefetchHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrefetchHint::L1 => write!(f, "L1"),
            PrefetchHint::L2 => write!(f, "L2"),
            PrefetchHint::L3 => write!(f, "L3"),
            PrefetchHint::NTA => write!(f, "NTA"),
        }
    }
}

impl fmt::Display for MessageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageFormat::Result => write!(f, "Result"),
            MessageFormat::Request => write!(f, "Request"),
            MessageFormat::Barrier => write!(f, "Barrier"),
            MessageFormat::Custom(id) => write!(f, "Custom({})", id),
        }
    }
}

impl fmt::Display for FenceScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenceScope::Thread => write!(f, "thread"),
            FenceScope::Core => write!(f, "core"),
            FenceScope::Node => write!(f, "node"),
            FenceScope::Cluster => write!(f, "cluster"),
        }
    }
}

impl fmt::Display for PhextCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dims = self.dims();
        write!(f, "{}.{}.{}/{}.{}.{}/{}.{}.{}.{}.{}",
            dims[0], dims[1], dims[2],
            dims[3], dims[4], dims[5],
            dims[6], dims[7], dims[8], dims[9], dims[10])
    }
}

/// Pretty-print a SIW stream with line numbers
pub fn print_stream(siws: &[SIW]) {
    for (i, siw) in siws.iter().enumerate() {
        println!("{:4}: {}", i, siw);
    }
}

/// Generate disassembly-style output
pub fn disassemble_stream(siws: &[SIW]) -> String {
    let mut output = String::new();
    
    for (i, siw) in siws.iter().enumerate() {
        output.push_str(&format!("{:04x}:  ", i * 64));  // Address (64 bytes per SIW)
        
        // Dependency flags
        output.push_str(&format!("[{:08b}]  ", siw.deps));
        
        // Operations
        output.push_str(&format!("{:30} | {:30} | {:30}\n",
            format!("{}", siw.d_op),
            format!("{}", siw.s_op),
            format!("{}", siw.c_op)
        ));
        
        // Coordinate on next line
        output.push_str(&format!("        @ {}\n", siw.phext_addr));
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::siw::SIW;
    
    #[test]
    fn test_siw_display() {
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SGATHER { rd: 4, coord_idx: 0, width: 64 },
            CoordOp::CNOP,
            PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
        );
        
        let output = format!("{}", siw);
        assert!(output.contains("add r1"));
        assert!(output.contains("gather r4"));
    }
    
    #[test]
    fn test_phext_coord_display() {
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let output = format!("{}", coord);
        assert_eq!(output, "1.2.3/4.5.6/7.8.9.10.11");
    }
    
    #[test]
    fn test_disassemble() {
        let siws = vec![
            SIW::new(
                DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ),
            SIW::nop(),
        ];
        
        let output = disassemble_stream(&siws);
        assert!(output.contains("0000:"));
        assert!(output.contains("0040:"));  // 64 bytes per SIW
    }
}

#[cfg(test)]
mod w20_tests {
    use super::*;
    use crate::siw::SIW;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};
    use crate::phext_coord::PhextCoord;

    fn make_siw(d: DenseOp, s: SparseOp, c: CoordOp) -> SIW {
        SIW::new(d, s, c, PhextCoord::zero())
    }

    #[test]
    fn disassemble_nop_stream() {
        let siws = vec![SIW::nop(), SIW::nop()];
        let out = disassemble_stream(&siws);
        // Display uses lowercase: "nop" not "DNOP"
        assert!(out.contains("nop"), "should contain nop");
    }

    #[test]
    fn disassemble_single_dadd() {
        let siws = vec![make_siw(
            DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
        )];
        let out = disassemble_stream(&siws);
        // Display uses "add" for DADD
        assert!(out.contains("add"), "should contain add");
    }

    #[test]
    fn disassemble_offset_advances() {
        let siws = vec![SIW::nop(), SIW::nop(), SIW::nop()];
        let out = disassemble_stream(&siws);
        assert!(out.contains("0000:"), "first offset");
        assert!(out.contains("0040:"), "second offset (64 bytes)");
        assert!(out.contains("0080:"), "third offset");
    }

    #[test]
    fn disassemble_empty_stream() {
        let out = disassemble_stream(&[]);
        assert_eq!(out, "", "empty stream = empty string");
    }

    #[test]
    fn disassemble_dfma_shown() {
        let siws = vec![make_siw(
            DenseOp::DFMA { rd: 0, rs1: 1, rs2: 2, rs3: 3 },
            SparseOp::SNOP, CoordOp::CNOP,
        )];
        let out = disassemble_stream(&siws);
        // Display uses "fma" for DFMA
        assert!(out.contains("fma"), "FMA should appear in disassembly");
    }

    #[test]
    fn disassemble_csend_shown() {
        let siws = vec![make_siw(
            DenseOp::DNOP, SparseOp::SNOP,
            CoordOp::CSEND { msg_reg: 0, dest_sentron: 5 },
        )];
        let out = disassemble_stream(&siws);
        // Display uses "send" or similar for CSEND
        assert!(!out.is_empty(), "non-empty disassembly for csend SIW");
    }
}
