// SMT (Simultaneous Multithreading) Analysis - R23 Wave 5
//
// Analyzes workload pairs for SMT efficiency.
// Generates complementary D-heavy + S-heavy workloads to minimize port conflicts.

use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp, CoordOp};
use crate::phext_coord::PhextCoord;

/// Workload type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    DHeavy,   // 80% D-Pipe (ALU-heavy)
    SHeavy,   // 80% S-Pipe (Memory-heavy)
    CHeavy,   // 80% C-Pipe (Coordination-heavy)
    Balanced, // 33% each
}

/// SMT efficiency metrics
#[derive(Debug)]
pub struct SmtEfficiency {
    pub port_conflicts: usize,
    pub complementary_ops: usize,
    pub predicted_speedup: f64,  // 1.0-2.0x range
}

pub struct SmtAnalyzer;

impl SmtAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Generate complementary workload pair
    ///
    /// Returns (thread1_siws, thread2_siws) where:
    /// - thread1 is D-heavy (uses Port 0/1)
    /// - thread2 is S-heavy (uses Port 4/5)
    /// - Minimal port conflicts → high SMT efficiency
    pub fn generate_complementary_pair(&self, count: usize) -> (Vec<SIW>, Vec<SIW>) {
        let mut thread1 = Vec::with_capacity(count);
        let mut thread2 = Vec::with_capacity(count);

        for i in 0..count {
            // Thread 1: D-heavy (80% D-Pipe, 20% S-Pipe)
            let d_op1 = if i % 5 != 0 {
                self.generate_d_op(i)
            } else {
                DenseOp::DNOP
            };
            
            let s_op1 = if i % 5 == 0 {
                self.generate_s_op(i)
            } else {
                SparseOp::SNOP
            };

            thread1.push(SIW::new(
                d_op1,
                s_op1,
                CoordOp::CNOP,
                PhextCoord::new([(i % 100 + 1) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            ));

            // Thread 2: S-heavy (80% S-Pipe, 20% D-Pipe)
            let d_op2 = if i % 5 == 0 {
                self.generate_d_op(i)
            } else {
                DenseOp::DNOP
            };
            
            let s_op2 = if i % 5 != 0 {
                self.generate_s_op(i)
            } else {
                SparseOp::SNOP
            };

            thread2.push(SIW::new(
                d_op2,
                s_op2,
                CoordOp::CNOP,
                PhextCoord::new([(i % 100 + 1) as u16, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            ));
        }

        (thread1, thread2)
    }

    /// Analyze SMT efficiency of two workload streams
    pub fn analyze_smt_efficiency(&self, thread1: &[SIW], thread2: &[SIW]) -> SmtEfficiency {
        assert_eq!(thread1.len(), thread2.len(), "Thread workloads must be same length");

        let mut port_conflicts = 0;
        let mut complementary_ops = 0;

        for i in 0..thread1.len() {
            let ports1 = self.get_execution_ports(&thread1[i]);
            let ports2 = self.get_execution_ports(&thread2[i]);

            if self.ports_conflict(&ports1, &ports2) {
                port_conflicts += 1;
            } else {
                complementary_ops += 1;
            }
        }

        // Predict speedup based on port conflicts
        // Low conflicts → high speedup (up to 2.0x)
        // High conflicts → low speedup (down to 1.0x)
        let conflict_rate = port_conflicts as f64 / thread1.len() as f64;
        let predicted_speedup = 2.0 - conflict_rate; // Linear model (simple)

        SmtEfficiency {
            port_conflicts,
            complementary_ops,
            predicted_speedup,
        }
    }

    /// Get execution ports used by a SIW
    fn get_execution_ports(&self, siw: &SIW) -> Vec<u8> {
        let mut ports = Vec::new();

        // D-Pipe → Port 0/1 (ALU)
        if !matches!(siw.d_op, DenseOp::DNOP) {
            ports.push(0); // Simplified: assume Port 0
        }

        // S-Pipe → Port 4/5 (AGU + Load/Store)
        if !matches!(siw.s_op, SparseOp::SNOP) {
            ports.push(4); // Simplified: assume Port 4
        }

        // C-Pipe → Port 2/3 (ALU for coordination)
        if !matches!(siw.c_op, CoordOp::CNOP) {
            ports.push(2); // Simplified: assume Port 2
        }

        ports
    }

    /// Check if two port sets conflict
    fn ports_conflict(&self, ports1: &[u8], ports2: &[u8]) -> bool {
        for &p1 in ports1 {
            for &p2 in ports2 {
                if p1 == p2 {
                    return true;
                }
            }
        }
        false
    }

    fn generate_d_op(&self, idx: usize) -> DenseOp {
        let rd = (idx % 32) as u8;
        let rs1 = ((idx + 1) % 32) as u8;
        let rs2 = ((idx + 2) % 32) as u8;
        let rs3 = ((idx + 3) % 32) as u8;

        match idx % 4 {
            0 => DenseOp::DFMA { rd, rs1, rs2, rs3 },
            1 => DenseOp::DADD { rd, rs1, rs2 },
            2 => DenseOp::DMUL { rd, rs1, rs2 },
            _ => DenseOp::DSUB { rd, rs1, rs2 },
        }
    }

    fn generate_s_op(&self, idx: usize) -> SparseOp {
        let rd = (idx % 32) as u8;
        let coord_idx = (idx % 16) as u8;

        if idx % 2 == 0 {
            SparseOp::SGATHER { rd, coord_idx, width: 64 }
        } else {
            SparseOp::SSCATTR { rs: rd, coord_idx, width: 64 }
        }
    }
}

impl SmtEfficiency {
    pub fn print(&self) {
        println!("SMT Efficiency Analysis");
        println!("======================");
        println!("Port conflicts: {}", self.port_conflicts);
        println!("Complementary ops: {}", self.complementary_ops);
        println!("Predicted speedup: {:.2}x", self.predicted_speedup);
        println!();
        
        if self.predicted_speedup >= 1.9 {
            println!("✓ Excellent SMT efficiency (>= 95%)");
        } else if self.predicted_speedup >= 1.7 {
            println!("✓ Good SMT efficiency (>= 85%)");
        } else if self.predicted_speedup >= 1.5 {
            println!("⚠ Fair SMT efficiency (>= 75%)");
        } else {
            println!("✗ Poor SMT efficiency (< 75%)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_complementary_pair() {
        let analyzer = SmtAnalyzer::new();
        let (thread1, thread2) = analyzer.generate_complementary_pair(100);

        assert_eq!(thread1.len(), 100);
        assert_eq!(thread2.len(), 100);

        // Thread 1 should be D-heavy (80% D-Pipe)
        let t1_d_ops = thread1.iter().filter(|siw| !matches!(siw.d_op, DenseOp::DNOP)).count();
        assert!(t1_d_ops >= 75 && t1_d_ops <= 85, "Thread 1 should be ~80% D-Pipe");

        // Thread 2 should be S-heavy (80% S-Pipe)
        let t2_s_ops = thread2.iter().filter(|siw| !matches!(siw.s_op, SparseOp::SNOP)).count();
        assert!(t2_s_ops >= 75 && t2_s_ops <= 85, "Thread 2 should be ~80% S-Pipe");
    }

    #[test]
    fn test_analyze_smt_efficiency() {
        let analyzer = SmtAnalyzer::new();
        let (thread1, thread2) = analyzer.generate_complementary_pair(100);
        let efficiency = analyzer.analyze_smt_efficiency(&thread1, &thread2);

        // Complementary workloads should have low conflict rate
        assert!(efficiency.complementary_ops > efficiency.port_conflicts,
                "Complementary workloads should have more non-conflicting ops");

        // Predicted speedup should be high (> 1.5x)
        assert!(efficiency.predicted_speedup > 1.5,
                "Predicted speedup should be > 1.5x for complementary workloads");
    }
}
