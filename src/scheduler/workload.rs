//! Workload Classification and Cooperative Scheduling (R23W17-2)
//!
//! Classifies SIW streams by dominant pipe usage (D/S/C-heavy).
//! Enables complementary workload pairing and cooperative OS scheduler hints.

use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp, CoordOp};

/// Workload classification based on pipe utilization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkloadClass {
    /// Compute-bound (D-Pipe heavy: >60% D, <20% S)
    DenseHeavy,
    
    /// Memory-bound (S-Pipe heavy: >60% S, <20% D)
    SparseHeavy,
    
    /// Communication-bound (C-Pipe heavy: >60% C)
    CoordHeavy,
    
    /// Balanced (no single pipe >60%)
    Balanced,
}

/// Workload statistics for a SIW stream
#[derive(Debug, Clone)]
pub struct WorkloadStats {
    /// Total SIWs
    pub siw_count: usize,
    
    /// Active D-Pipe operations (non-NOP)
    pub d_ops: usize,
    
    /// Active S-Pipe operations (non-NOP)
    pub s_ops: usize,
    
    /// Active C-Pipe operations (non-NOP)
    pub c_ops: usize,
    
    /// D-Pipe utilization (0.0-1.0)
    pub d_util: f64,
    
    /// S-Pipe utilization (0.0-1.0)
    pub s_util: f64,
    
    /// C-Pipe utilization (0.0-1.0)
    pub c_util: f64,
    
    /// Workload classification
    pub class: WorkloadClass,
}

impl WorkloadStats {
    /// Analyze a SIW stream and classify workload
    pub fn analyze(stream: &[SIW]) -> Self {
        if stream.is_empty() {
            return Self::empty();
        }

        let siw_count = stream.len();
        let mut d_ops = 0;
        let mut s_ops = 0;
        let mut c_ops = 0;

        for siw in stream {
            if !matches!(siw.d_op, DenseOp::DNOP) {
                d_ops += 1;
            }
            if !matches!(siw.s_op, SparseOp::SNOP) {
                s_ops += 1;
            }
            if !matches!(siw.c_op, CoordOp::CNOP) {
                c_ops += 1;
            }
        }

        let d_util = d_ops as f64 / siw_count as f64;
        let s_util = s_ops as f64 / siw_count as f64;
        let c_util = c_ops as f64 / siw_count as f64;

        let class = Self::classify(d_util, s_util, c_util);

        WorkloadStats {
            siw_count,
            d_ops,
            s_ops,
            c_ops,
            d_util,
            s_util,
            c_util,
            class,
        }
    }

    /// Classify workload based on utilization
    fn classify(d_util: f64, s_util: f64, c_util: f64) -> WorkloadClass {
        if d_util > 0.6 && s_util < 0.2 {
            WorkloadClass::DenseHeavy
        } else if s_util > 0.6 && d_util < 0.2 {
            WorkloadClass::SparseHeavy
        } else if c_util > 0.6 {
            WorkloadClass::CoordHeavy
        } else {
            WorkloadClass::Balanced
        }
    }

    /// Check if two workloads are complementary (good for SMT pairing)
    pub fn is_complementary(&self, other: &WorkloadStats) -> bool {
        use WorkloadClass::*;
        
        match (self.class, other.class) {
            // D-heavy + S-heavy = complementary (use different execution ports)
            (DenseHeavy, SparseHeavy) | (SparseHeavy, DenseHeavy) => true,
            
            // D-heavy + C-heavy = complementary
            (DenseHeavy, CoordHeavy) | (CoordHeavy, DenseHeavy) => true,
            
            // S-heavy + C-heavy = complementary
            (SparseHeavy, CoordHeavy) | (CoordHeavy, SparseHeavy) => true,
            
            // Same class = NOT complementary (contention)
            _ => false,
        }
    }

    fn empty() -> Self {
        WorkloadStats {
            siw_count: 0,
            d_ops: 0,
            s_ops: 0,
            c_ops: 0,
            d_util: 0.0,
            s_util: 0.0,
            c_util: 0.0,
            class: WorkloadClass::Balanced,
        }
    }
}

/// Quantized execution chunk (64 SIWs default)
pub const DEFAULT_QUANTUM: usize = 64;

/// Split a SIW stream into quanta for cooperative scheduling
pub fn quantize_stream(stream: &[SIW], quantum_size: usize) -> Vec<&[SIW]> {
    stream.chunks(quantum_size).collect()
}

/// Recommend whether to yield to OS scheduler after quantum
pub fn should_yield(quantum_idx: usize, total_quanta: usize) -> bool {
    // Yield every 4 quanta (256 SIWs) unless on final quantum
    quantum_idx > 0 && quantum_idx % 4 == 0 && quantum_idx < total_quanta - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhextCoord;

    fn make_d_heavy_siw() -> SIW {
        SIW::new(
            DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }

    fn make_s_heavy_siw() -> SIW {
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 64 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }

    fn make_c_heavy_siw() -> SIW {
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CPACK {
                rd: 0,
                rs1: 1,
                rs2: 2,
                fmt: crate::pipes::MessageFormat::Result,
            },
            PhextCoord::zero(),
        )
    }

    #[test]
    fn classify_d_heavy() {
        let stream: Vec<SIW> = (0..100).map(|_| make_d_heavy_siw()).collect();
        let stats = WorkloadStats::analyze(&stream);

        assert_eq!(stats.siw_count, 100);
        assert_eq!(stats.d_ops, 100);
        assert_eq!(stats.s_ops, 0);
        assert_eq!(stats.class, WorkloadClass::DenseHeavy);
        assert!(stats.d_util > 0.9);
    }

    #[test]
    fn classify_s_heavy() {
        let stream: Vec<SIW> = (0..100).map(|_| make_s_heavy_siw()).collect();
        let stats = WorkloadStats::analyze(&stream);

        assert_eq!(stats.s_ops, 100);
        assert_eq!(stats.d_ops, 0);
        assert_eq!(stats.class, WorkloadClass::SparseHeavy);
    }

    #[test]
    fn classify_balanced() {
        let mut stream = Vec::new();
        for _ in 0..50 {
            stream.push(make_d_heavy_siw());
        }
        for _ in 0..50 {
            stream.push(make_s_heavy_siw());
        }

        let stats = WorkloadStats::analyze(&stream);
        assert_eq!(stats.class, WorkloadClass::Balanced);
    }

    #[test]
    fn complementary_workloads() {
        let d_stream: Vec<SIW> = (0..100).map(|_| make_d_heavy_siw()).collect();
        let s_stream: Vec<SIW> = (0..100).map(|_| make_s_heavy_siw()).collect();

        let d_stats = WorkloadStats::analyze(&d_stream);
        let s_stats = WorkloadStats::analyze(&s_stream);

        assert!(d_stats.is_complementary(&s_stats));
        assert!(s_stats.is_complementary(&d_stats));
    }

    #[test]
    fn non_complementary_workloads() {
        let d_stream1: Vec<SIW> = (0..100).map(|_| make_d_heavy_siw()).collect();
        let d_stream2: Vec<SIW> = (0..100).map(|_| make_d_heavy_siw()).collect();

        let stats1 = WorkloadStats::analyze(&d_stream1);
        let stats2 = WorkloadStats::analyze(&d_stream2);

        assert!(!stats1.is_complementary(&stats2));
    }

    #[test]
    fn quantization() {
        let stream: Vec<SIW> = (0..200).map(|_| make_d_heavy_siw()).collect();
        let quanta = quantize_stream(&stream, 64);

        assert_eq!(quanta.len(), 4); // 200 / 64 = 3.125 → 4 chunks
        assert_eq!(quanta[0].len(), 64);
        assert_eq!(quanta[1].len(), 64);
        assert_eq!(quanta[2].len(), 64);
        assert_eq!(quanta[3].len(), 8); // Remainder
    }

    #[test]
    fn yield_policy() {
        // Should not yield on first quantum
        assert!(!should_yield(0, 10));

        // Should yield every 4 quanta
        assert!(!should_yield(1, 10));
        assert!(!should_yield(2, 10));
        assert!(!should_yield(3, 10));
        assert!(should_yield(4, 10));

        // Should not yield on final quantum
        assert!(!should_yield(9, 10));
    }
}
