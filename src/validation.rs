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
        DenseOp::DMOV { rd, .. } => Some(*rd),
        DenseOp::DNOP => None,
    }
}

fn get_sparse_write_register(op: &SparseOp) -> Option<u8> {
    match op {
        SparseOp::SGATHER { rd, .. } |
        SparseOp::SINDEX { rd, .. } |
        SparseOp::SDEDUP { rd, .. } |
        SparseOp::SALLOC { rd, .. } => Some(*rd),
        _ => None,
    }
}

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
