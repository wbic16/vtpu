//! SIW Stream Validation Demo
//!
//! Shows how to validate SIW streams and catch common errors

use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, PhextCoord, validate_stream, ValidationError};
use vtpu_runtime::display::disassemble_stream;

fn main() {
    println!("=== vTPU Stream Validation Demo ===\n");
    
    // Example 1: Valid stream
    println!("--- Example 1: Valid Stream ---");
    let valid_stream = vec![
        SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
        ),
    ];
    
    match validate_stream(&valid_stream) {
        Ok(()) => println!("✅ Stream is valid\n"),
        Err(errors) => println!("❌ Errors: {:?}\n", errors),
    }
    
    // Example 2: Register conflict (both D-Pipe and S-Pipe write to r1)
    println!("--- Example 2: Register Conflict ---");
    let conflict_stream = vec![
        SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },  // Conflict!
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
    ];
    
    match validate_stream(&conflict_stream) {
        Ok(()) => println!("✅ Stream is valid (unexpected!)\n"),
        Err(errors) => {
            for err in &errors {
                match err {
                    ValidationError::RegisterConflict { siw_idx, register } => {
                        println!("❌ SIW {}: Multiple pipes write to r{}", siw_idx, register);
                    }
                    _ => println!("❌ {:?}", err),
                }
            }
            println!();
        }
    }
    
    // Example 3: Empty stream
    println!("--- Example 3: Empty Stream ---");
    let empty_stream: Vec<SIW> = vec![];
    
    match validate_stream(&empty_stream) {
        Ok(()) => println!("✅ Stream is valid (unexpected!)\n"),
        Err(errors) => {
            for err in &errors {
                match err {
                    ValidationError::EmptyStream => {
                        println!("❌ Stream is empty - at least one SIW required");
                    }
                    _ => println!("❌ {:?}", err),
                }
            }
            println!();
        }
    }
    
    // Note: Invalid coordinate detection is tested in unit tests using unsafe new_unchecked()
    // This example doesn't demonstrate it because PhextCoord::new() validates dimensions
    
    // Example 4: Disassembly output
    println!("--- Example 4: Disassembly ---");
    let demo_stream = vec![
        SIW::new(
            DenseOp::DMOV { rd: 1, imm: 42 },
            SparseOp::SPREFCH { coord_idx: 0, hint: vtpu_runtime::PrefetchHint::L2 },
            CoordOp::CNOP,
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        ),
        SIW::new(
            DenseOp::DADD { rd: 2, rs1: 1, rs2: 1 },
            SparseOp::SGATHER { rd: 3, coord_idx: 0, width: 64 },
            CoordOp::CBAR { barrier_id: 1, count: 6 },
            PhextCoord::new([2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31]),
        ),
    ];
    
    println!("{}", disassemble_stream(&demo_stream));
}
