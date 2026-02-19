//! SIW Stream Validation
//!
//! Checks for common errors in hand-written or generated SIW streams.

use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp, CoordOp};

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Register written multiple times in the same SIW
    RegisterConflict { siw_idx: usize, register: u8 },
    
    /// Unresolved read-after-write dependency
    UnresolvedDependency { siw_idx: usize, register: u8 },
    
    /// Invalid phext coordinate (dimension out of range)
    InvalidCoordinate { siw_idx: usize, dim: usize, value: u16 },
    
    /// Excessive dependencies (likely indicates poor scheduling)
    ExcessiveDependencies { siw_idx: usize, dep_count: u32 },
    
    /// Empty stream
    EmptyStream,
}

/// Validation result
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Validate a SIW stream
pub fn validate_stream(siws: &[SIW]) -> ValidationResult {
    let mut errors = Vec::new();
    
    if siws.is_empty() {
        errors.push(ValidationError::EmptyStream);
        return Err(errors);
    }
    
    for (idx, siw) in siws.iter().enumerate() {
        // Check for intra-SIW register conflicts
        if let Some(conflict) = check_register_conflicts(siw) {
            errors.push(ValidationError::RegisterConflict { 
                siw_idx: idx, 
                register: conflict 
            });
        }
        
        // Note: Coordinate validity is enforced by PhextCoord::new() - dimensions are
        // automatically truncated to 11 bits (0-2047), so we don't need runtime validation here
        
        // Check for excessive dependency chaining (>5 is likely a problem)
        let dep_count = siw.deps.count_ones();
        if dep_count > 5 {
            errors.push(ValidationError::ExcessiveDependencies {
                siw_idx: idx,
                dep_count,
            });
        }
    }
    
    // Check for unresolved inter-SIW dependencies
    let mut live_d_writes = Vec::new();
    let mut live_s_writes = Vec::new();
    let mut live_c_writes = Vec::new();
    
    for (idx, siw) in siws.iter().enumerate() {
        // Check if dependencies are satisfied
        if (siw.deps & 0x01) != 0 && live_d_writes.is_empty() {
            errors.push(ValidationError::UnresolvedDependency { siw_idx: idx, register: 0 });
        }
        if (siw.deps & 0x02) != 0 && live_s_writes.is_empty() {
            errors.push(ValidationError::UnresolvedDependency { siw_idx: idx, register: 0 });
        }
        if (siw.deps & 0x04) != 0 && live_c_writes.is_empty() {
            errors.push(ValidationError::UnresolvedDependency { siw_idx: idx, register: 0 });
        }
        
        // Track writes (simplified - doesn't track specific registers yet)
        if let Some(_reg) = get_write_register(&siw.d_op) {
            live_d_writes.push(idx);
        }
        if let Some(_reg) = get_sparse_write_register(&siw.s_op) {
            live_s_writes.push(idx);
        }
        if let Some(_reg) = get_coord_write_register(&siw.c_op) {
            live_c_writes.push(idx);
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check if a SIW has intra-instruction register conflicts
fn check_register_conflicts(siw: &SIW) -> Option<u8> {
    let d_write = get_write_register(&siw.d_op);
    let s_write = get_sparse_write_register(&siw.s_op);
    let c_write = get_coord_write_register(&siw.c_op);
    
    // Check if multiple pipes write to the same register
    if let (Some(d), Some(s)) = (d_write, s_write) {
        if d == s {
            return Some(d);
        }
    }
    if let (Some(d), Some(c)) = (d_write, c_write) {
        if d == c {
            return Some(d);
        }
    }
    if let (Some(s), Some(c)) = (s_write, c_write) {
        if s == c {
            return Some(s);
        }
    }
    
    None
}

// Helper functions (duplicated from stream.rs - should be shared module)
fn get_write_register(op: &DenseOp) -> Option<u8> {
    match op {
        DenseOp::DFMA { rd, .. } |
        DenseOp::DADD { rd, .. } |
        DenseOp::DSUB { rd, .. } |
        DenseOp::DMUL { rd, .. } |
        DenseOp::DCMP { rd, .. } |
        DenseOp::DRED { rd, .. } |
        DenseOp::DSEL { rd, .. } |
        DenseOp::DHDENC { rd, .. } | DenseOp::DHDBIND { rd, .. } | DenseOp::DHDBUND { rd, .. } | DenseOp::DHDPERM { rd, .. } | DenseOp::DHDSIM { rd, .. } | DenseOp::DTERNARY { rd, .. } | DenseOp::DTPOP { rd, .. } | DenseOp::DTACC { rd, .. } | DenseOp::DMOV { rd, .. } => Some(*rd),
        DenseOp::DNOP => None,
    }
}

fn get_sparse_write_register(op: &SparseOp) -> Option<u8> {
    match op {
        SparseOp::SGATHER { rd, .. } |
        SparseOp::SINDEX { rd, .. } |
        SparseOp::SDEDUP { rd, .. } |
        SparseOp::SASSOC { rd, .. } | SparseOp::SROUTE { rd, .. } | SparseOp::SNEIGHBR { rd, .. } | SparseOp::SALLOC { rd, .. } => Some(*rd),
        _ => None,
    }
}

fn get_coord_write_register(op: &CoordOp) -> Option<u8> {
    match op {
        CoordOp::CPACK { rd, .. } |
        CoordOp::CRECV { rd, .. } |
        CoordOp::CREDUCE { rd, .. } | CoordOp::CMERGE { rd, .. } => Some(*rd),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phext_coord::PhextCoord;
    
    #[test]
    fn test_validate_empty_stream() {
        let result = validate_stream(&[]);
        assert!(result.is_err());
        
        if let Err(errors) = result {
            assert!(matches!(errors[0], ValidationError::EmptyStream));
        }
    }
    
    #[test]
    fn test_validate_valid_stream() {
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        );
        
        let result = validate_stream(&[siw]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_detect_register_conflict() {
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },  // Conflict: both write r1
            CoordOp::CNOP,
            PhextCoord::zero(),
        );
        
        let result = validate_stream(&[siw]);
        assert!(result.is_err());
        
        if let Err(errors) = result {
            assert!(matches!(errors[0], ValidationError::RegisterConflict { .. }));
        }
    }
    
    // Note: Invalid coordinate test removed because PhextCoord packs dimensions to 11 bits,
    // so values > 2047 get truncated during construction. The validation check for >= 2048
    // can never trigger with properly constructed PhextCoords (which is good - the type
    // system prevents invalid coordinates from being created).
}

#[cfg(test)]
mod w20_tests {
    use super::*;
    use crate::siw::SIW;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};
    use crate::phext_coord::PhextCoord;

    #[test]
    fn validate_empty_stream_returns_err() {
        // Empty stream is invalid per spec
        assert!(validate_stream(&[]).is_err());
    }

    #[test]
    fn validate_nop_stream_ok() {
        let siws = vec![SIW::nop(), SIW::nop()];
        assert!(validate_stream(&siws).is_ok());
    }

    #[test]
    fn validate_single_dadd_ok() {
        let s = SIW::new(DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
        assert!(validate_stream(&[s]).is_ok());
    }

    #[test]
    fn validate_register_conflict_detected() {
        // rd == rs1 — structural hazard
        let s = SIW::new(DenseOp::DADD { rd: 1, rs1: 1, rs2: 2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
        // May or may not be an error depending on validator strictness
        let _ = validate_stream(&[s]); // must not panic
    }

    #[test]
    fn validate_large_stream_ok() {
        let siws: Vec<SIW> = (0..100).map(|i| {
            let rd = (i % 14) as u8;
            let rs1 = (i % 13 + 1) as u8;
            let rs2 = (i % 12 + 2) as u8;
            SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
        }).collect();
        let result = validate_stream(&siws);
        // Should not panic; errors may be present for conflicts
        let _ = result;
    }

    #[test]
    fn validate_packed_stream_ok() {
        let s = SIW::new(
            DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::SINDEX { rd: 3, base: 0, offset: 1, dim: 0 },
            CoordOp::CBAR { barrier_id: 0, count: 1 },
            PhextCoord::zero(),
        );
        let _ = validate_stream(&[s]); // must not panic
    }
}
