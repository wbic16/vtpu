// vTPU Synthetic SIW Generator - R23 Wave 3
//
// Generates configurable SIW streams for micro-benchmarking:
// - Pipe ratios (D/S/C balance)
// - Dependency depth (independent vs chained)
// - Coordinate locality (same-scroll vs cross-volume)

use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use vtpu_runtime::{CoordOp, DenseOp, PhextCoord, SparseOp, SIW, PrefetchHint, ReductionOp, MessageFormat, FenceScope};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Pattern {
    /// 80% D-Pipe, 10% S-Pipe, 10% C-Pipe
    DHeavy,
    /// 10% D-Pipe, 80% S-Pipe, 10% C-Pipe
    SHeavy,
    /// 10% D-Pipe, 10% S-Pipe, 80% C-Pipe
    CHeavy,
    /// 33% D-Pipe, 33% S-Pipe, 33% C-Pipe
    Balanced,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Locality {
    /// All accesses in same scroll (3D = constant)
    High,
    /// Accesses across scrolls in same section (4D varies)
    Medium,
    /// Accesses across chapters/volumes (6D+ varies)
    Low,
}

#[derive(Parser, Debug)]
#[command(name = "siw-gen")]
#[command(about = "Generate synthetic vTPU SIW streams for benchmarking")]
struct Args {
    /// SIW generation pattern (d-heavy, s-heavy, c-heavy, balanced)
    #[arg(short, long, value_enum, default_value = "balanced")]
    pattern: Pattern,

    /// Number of SIWs to generate
    #[arg(short = 'n', long, default_value = "1000")]
    count: usize,

    /// Coordinate locality (high, medium, low)
    #[arg(short, long, value_enum, default_value = "medium")]
    locality: Locality,

    /// Dependency chain depth (0 = all independent)
    #[arg(short, long, default_value = "2")]
    dep_depth: u8,

    /// Output file path
    #[arg(short, long)]
    output: PathBuf,

    /// Generate text output instead of binary
    #[arg(short = 't', long)]
    text: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let siws = generate_siw_stream(&args);

    if args.text {
        write_text_stream(&siws, &args.output)?;
    } else {
        write_binary_stream(&siws, &args.output)?;
    }

    println!("Generated {} SIWs to {:?}", args.count, args.output);
    println!("  Pattern: {:?}", args.pattern);
    println!("  Locality: {:?}", args.locality);
    println!("  Dependency depth: {}", args.dep_depth);

    Ok(())
}

fn generate_siw_stream(args: &Args) -> Vec<SIW> {
    let mut siws = Vec::with_capacity(args.count);
    let mut rng_state = 0x123456789ABCDEFu64; // Simple PRNG state

    // Pipe probabilities based on pattern
    let (d_prob, s_prob, _c_prob) = match args.pattern {
        Pattern::DHeavy => (0.80, 0.10, 0.10),
        Pattern::SHeavy => (0.10, 0.80, 0.10),
        Pattern::CHeavy => (0.10, 0.10, 0.80),
        Pattern::Balanced => (0.33, 0.33, 0.34),
    };

    for i in 0..args.count {
        // Simple LCG for pseudo-random numbers
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand_val = (rng_state >> 32) as f64 / u32::MAX as f64;

        // Select pipe operations based on pattern
        let d_op = if rand_val < d_prob {
            generate_dense_op(i, args.dep_depth, &mut rng_state)
        } else {
            DenseOp::DNOP
        };

        let s_op = if rand_val >= d_prob && rand_val < d_prob + s_prob {
            generate_sparse_op(i, &mut rng_state)
        } else {
            SparseOp::SNOP
        };

        let c_op = if rand_val >= d_prob + s_prob {
            generate_coord_op(i, &mut rng_state)
        } else {
            CoordOp::CNOP
        };

        let coord = generate_coordinate(i, args.locality, &mut rng_state);

        siws.push(SIW::new(d_op, s_op, c_op, coord));
    }

    siws
}

fn generate_dense_op(idx: usize, dep_depth: u8, rng: &mut u64) -> DenseOp {
    *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
    let op_type = (*rng >> 32) % 8;

    let rd = (idx % 32) as u8;
    let rs1 = if dep_depth > 0 && idx >= dep_depth as usize {
        ((idx - dep_depth as usize) % 32) as u8
    } else {
        ((idx + 1) % 32) as u8
    };
    let rs2 = ((idx + 2) % 32) as u8;
    let rs3 = ((idx + 3) % 32) as u8;

    match op_type {
        0 => DenseOp::DFMA { rd, rs1, rs2, rs3 },
        1 => DenseOp::DADD { rd, rs1, rs2 },
        2 => DenseOp::DSUB { rd, rs1, rs2 },
        3 => DenseOp::DMUL { rd, rs1, rs2 },
        4 => DenseOp::DCMP { rd, rs1, rs2 },
        5 => DenseOp::DRED { rd, rs1, op: ReductionOp::Sum },
        6 => DenseOp::DSEL { rd, rs1, rs2, flags: 0 },
        _ => DenseOp::DMOV { rd, imm: (idx as i64) },
    }
}

fn generate_sparse_op(idx: usize, rng: &mut u64) -> SparseOp {
    *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
    let op_type = (*rng >> 32) % 8;

    let rd = (idx % 32) as u8;
    let coord_idx = (idx % 16) as u8;

    match op_type {
        0 => SparseOp::SGATHER {
            rd,
            coord_idx,
            width: 64,
        },
        1 => SparseOp::SSCATTR {
            rs: rd,
            coord_idx,
            width: 64,
        },
        2 => SparseOp::SINDEX {
            rd,
            base: coord_idx,
            offset: (idx as i32) % 100,
            dim: 3,
        },
        3 => SparseOp::SDEDUP {
            rd,
            rs: coord_idx,
            table_id: (idx % 256) as u16,
        },
        4 => SparseOp::SPREFCH {
            coord_idx,
            hint: PrefetchHint::L2,
        },
        5 => SparseOp::SFLUSH {
            coord_idx,
            width: 64,
        },
        6 => SparseOp::SALLOC {
            rd,
            size: 64 * 1024,
            dim_mask: 0x7,
        },
        _ => SparseOp::SFREE {
            coord_idx,
            size: 64 * 1024,
        },
    }
}

fn generate_coord_op(idx: usize, rng: &mut u64) -> CoordOp {
    *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
    let op_type = (*rng >> 32) % 8;

    let rd = (idx % 32) as u8;
    let rs1 = ((idx + 1) % 32) as u8;
    let rs2 = ((idx + 2) % 32) as u8;

    match op_type {
        0 => CoordOp::CPACK {
            rd,
            rs1,
            rs2,
            fmt: MessageFormat::Result,
        },
        1 => CoordOp::CROUTE {
            msg_reg: rd,
            dest_node: (idx % 5) as u8,
        },
        2 => CoordOp::CSEND {
            msg_reg: rd,
            dest_sentron: (idx % 40) as u8,
        },
        3 => CoordOp::CRECV {
            rd,
            src_sentron: (idx % 40) as u8,
        },
        4 => CoordOp::CBAR {
            barrier_id: (idx % 16) as u8,
            count: 6,
        },
        5 => CoordOp::CFENCE {
            scope: FenceScope::Node,
        },
        6 => CoordOp::CREDUCE {
            rd,
            rs: rs1,
            op: ReductionOp::Sum,
            group: (idx % 8) as u8,
        },
        _ => CoordOp::CCAST {
            rs: rs1,
            group: (idx % 8) as u8,
        },
    }
}

fn generate_coordinate(idx: usize, locality: Locality, rng: &mut u64) -> PhextCoord {
    *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);

    match locality {
        Locality::High => {
            // All in same scroll (3D = constant, vary 1D and 2D)
            let scroll = 1;
            let section = 1;
            let chapter = 1;
            PhextCoord::new([
                ((idx % 100) + 1) as u16,
                ((idx / 100) % 80 + 1) as u16,
                scroll,
                section,
                chapter,
                1,
                1,
                1,
                1,
                1,
                1,
            ])
        }
        Locality::Medium => {
            // Vary scroll within same section (4D = constant)
            let section = 1;
            let chapter = 1;
            PhextCoord::new([
                ((idx % 100) + 1) as u16,
                ((idx / 100) % 80 + 1) as u16,
                ((idx / 8000) % 10 + 1) as u16,
                section,
                chapter,
                1,
                1,
                1,
                1,
                1,
                1,
            ])
        }
        Locality::Low => {
            // Random across all dimensions
            let r1 = ((*rng >> 0) % 100 + 1) as u16;
            let r2 = ((*rng >> 10) % 80 + 1) as u16;
            let r3 = ((*rng >> 20) % 10 + 1) as u16;
            let r4 = ((*rng >> 30) % 8 + 1) as u16;
            let r5 = ((*rng >> 40) % 6 + 1) as u16;
            PhextCoord::new([r1, r2, r3, r4, r5, 1, 1, 1, 1, 1, 1])
        }
    }
}

fn write_binary_stream(siws: &[SIW], path: &PathBuf) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for siw in siws {
        // Write as raw 64-byte binary (SIW struct layout)
        let bytes = unsafe {
            std::slice::from_raw_parts(
                siw as *const SIW as *const u8,
                std::mem::size_of::<SIW>(),
            )
        };
        writer.write_all(bytes)?;
    }

    writer.flush()?;
    Ok(())
}

fn write_text_stream(siws: &[SIW], path: &PathBuf) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for (i, siw) in siws.iter().enumerate() {
        writeln!(writer, "{:04}: {}", i, siw)?;
    }

    writer.flush()?;
    Ok(())
}
