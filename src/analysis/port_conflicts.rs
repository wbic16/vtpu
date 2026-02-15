// Port Conflict Analyzer - R23 Wave 3
//
// Detects execution port conflicts in SIW streams based on Zen 4 architecture.
// Maps vTPU operations to physical execution ports and identifies resource contention.

use crate::siw::SIW;
use crate::pipes::{DenseOp, SparseOp, CoordOp, MessageFormat};

/// Zen 4 execution ports (simplified model for vTPU)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPort {
    ALU0 = 0,    // Port 0: Integer/FP ALU
    ALU1 = 1,    // Port 1: Integer/FP ALU
    ALU2 = 2,    // Port 2: Integer/Vector ALU + Branch
    ALU3 = 3,    // Port 3: Integer/Vector ALU + Branch
    AGU0 = 4,    // Port 4: Address Generation + Load
    AGU1 = 5,    // Port 5: Address Generation + Store
}

/// Port assignment for a single SIW
#[derive(Debug)]
pub struct PortAssignment {
    pub d_pipe_port: Option<ExecutionPort>,
    pub s_pipe_port: Option<ExecutionPort>,
    pub c_pipe_port: Option<ExecutionPort>,
    pub has_conflict: bool,
}

/// Analysis results for an entire SIW stream
#[derive(Debug)]
pub struct PortConflictReport {
    pub total_siws: usize,
    pub conflict_free: usize,
    pub port_conflicts: usize,
    pub conflicts_by_type: ConflictBreakdown,
}

#[derive(Debug, Default)]
pub struct ConflictBreakdown {
    pub d_c_alu_conflict: usize,    // D-Pipe and C-Pipe both need ALU
    pub s_pipe_contention: usize,   // S-Pipe needs both Load and Store
    pub triple_conflict: usize,      // All three pipes need same resource
}

pub struct PortConflictAnalyzer;

impl PortConflictAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze an entire SIW stream for port conflicts
    pub fn analyze(&self, siws: &[SIW]) -> PortConflictReport {
        let mut report = PortConflictReport {
            total_siws: siws.len(),
            conflict_free: 0,
            port_conflicts: 0,
            conflicts_by_type: ConflictBreakdown::default(),
        };

        for siw in siws {
            let assignment = self.assign_ports(siw);
            
            if assignment.has_conflict {
                report.port_conflicts += 1;
                self.classify_conflict(&assignment, &mut report.conflicts_by_type);
            } else {
                report.conflict_free += 1;
            }
        }

        report
    }

    /// Assign execution ports to a single SIW
    fn assign_ports(&self, siw: &SIW) -> PortAssignment {
        let d_port = self.map_dense_op(&siw.d_op);
        let s_port = self.map_sparse_op(&siw.s_op);
        let c_port = self.map_coord_op(&siw.c_op);

        let has_conflict = self.detect_conflict(d_port, s_port, c_port);

        PortAssignment {
            d_pipe_port: d_port,
            s_pipe_port: s_port,
            c_pipe_port: c_port,
            has_conflict,
        }
    }

    /// Map D-Pipe operation to execution port
    fn map_dense_op(&self, op: &DenseOp) -> Option<ExecutionPort> {
        match op {
            DenseOp::DNOP => None,
            // All dense ops go to ALU (Port 0 or 1)
            _ => Some(ExecutionPort::ALU0),
        }
    }

    /// Map S-Pipe operation to execution port
    fn map_sparse_op(&self, op: &SparseOp) -> Option<ExecutionPort> {
        match op {
            SparseOp::SNOP => None,
            // Loads go to Port 4 (AGU + Load)
            SparseOp::SGATHER { .. } | SparseOp::SPREFCH { .. } => Some(ExecutionPort::AGU0),
            // Stores go to Port 5 (AGU + Store)
            SparseOp::SSCATTR { .. } | SparseOp::SFLUSH { .. } => Some(ExecutionPort::AGU1),
            // Index/dedup/alloc/free use AGU0 (address computation)
            _ => Some(ExecutionPort::AGU0),
        }
    }

    /// Map C-Pipe operation to execution port
    fn map_coord_op(&self, op: &CoordOp) -> Option<ExecutionPort> {
        match op {
            CoordOp::CNOP => None,
            // All coordination ops go to ALU2/3 (Port 2 or 3)
            _ => Some(ExecutionPort::ALU2),
        }
    }

    /// Detect if port assignments have conflicts
    fn detect_conflict(
        &self,
        d_port: Option<ExecutionPort>,
        s_port: Option<ExecutionPort>,
        c_port: Option<ExecutionPort>,
    ) -> bool {
        use ExecutionPort::*;

        // No conflict if any pipe is NOP
        let ports: Vec<ExecutionPort> = [d_port, s_port, c_port]
            .iter()
            .filter_map(|&p| p)
            .collect();

        if ports.len() < 2 {
            return false; // Can't have conflict with < 2 operations
        }

        // Check for conflicts:
        // 1. D-Pipe (ALU0/1) and C-Pipe (ALU2/3) should never conflict
        // 2. S-Pipe (AGU0/1) uses different ports
        // 3. BUT if S-Pipe needs both Load AND Store, that's a conflict

        for i in 0..ports.len() {
            for j in (i + 1)..ports.len() {
                if Self::ports_conflict(ports[i], ports[j]) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if two specific ports conflict
    fn ports_conflict(p1: ExecutionPort, p2: ExecutionPort) -> bool {
        use ExecutionPort::*;

        match (p1, p2) {
            // ALU0/1 conflict with each other
            (ALU0, ALU1) | (ALU1, ALU0) => true,
            // ALU2/3 conflict with each other
            (ALU2, ALU3) | (ALU3, ALU2) => true,
            // AGU0/1 don't conflict (Load vs Store)
            (AGU0, AGU1) | (AGU1, AGU0) => false,
            // Same port always conflicts
            (a, b) if a == b => true,
            // Different port groups don't conflict
            _ => false,
        }
    }

    /// Classify the type of conflict
    fn classify_conflict(&self, assignment: &PortAssignment, breakdown: &mut ConflictBreakdown) {
        use ExecutionPort::*;

        let d_is_alu = matches!(assignment.d_pipe_port, Some(ALU0) | Some(ALU1));
        let c_is_alu = matches!(assignment.c_pipe_port, Some(ALU2) | Some(ALU3));
        let s_is_load = matches!(assignment.s_pipe_port, Some(AGU0));
        let s_is_store = matches!(assignment.s_pipe_port, Some(AGU1));

        if d_is_alu && c_is_alu {
            breakdown.d_c_alu_conflict += 1;
        }

        if s_is_load && s_is_store {
            breakdown.s_pipe_contention += 1;
        }

        // Triple conflict: all three pipes need overlapping resources
        if d_is_alu && c_is_alu && (s_is_load || s_is_store) {
            breakdown.triple_conflict += 1;
        }
    }
}

impl PortConflictReport {
    /// Print human-readable report
    pub fn print(&self) {
        println!("Port Conflict Analysis");
        println!("======================");
        println!("Total SIWs: {}", self.total_siws);
        println!(
            "Conflict-free: {} ({:.1}%)",
            self.conflict_free,
            100.0 * self.conflict_free as f64 / self.total_siws as f64
        );
        println!(
            "Port conflicts: {} ({:.1}%)",
            self.port_conflicts,
            100.0 * self.port_conflicts as f64 / self.total_siws as f64
        );
        println!();
        println!("Conflict breakdown:");
        println!(
            "  D-Pipe + C-Pipe ALU conflict: {} SIWs",
            self.conflicts_by_type.d_c_alu_conflict
        );
        println!(
            "  S-Pipe Load/Store contention: {} SIWs",
            self.conflicts_by_type.s_pipe_contention
        );
        println!(
            "  Triple conflicts: {} SIWs",
            self.conflicts_by_type.triple_conflict
        );
    }

    /// Return conflict rate as percentage
    pub fn conflict_rate(&self) -> f64 {
        if self.total_siws == 0 {
            return 0.0;
        }
        100.0 * self.port_conflicts as f64 / self.total_siws as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phext_coord::PhextCoord;

    #[test]
    fn test_no_conflict_balanced() {
        let analyzer = PortConflictAnalyzer::new();
        
        // D-Pipe (ALU), S-Pipe (AGU), C-Pipe (ALU2) - no conflict
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SGATHER { rd: 4, coord_idx: 0, width: 64 },
            CoordOp::CBAR { barrier_id: 0, count: 6 },
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        );

        let assignment = analyzer.assign_ports(&siw);
        assert!(!assignment.has_conflict);
    }

    #[test]
    fn test_detect_d_c_conflict() {
        let analyzer = PortConflictAnalyzer::new();
        
        // D-Pipe + C-Pipe both need ALU - this is actually OK in our model
        // because D uses ALU0/1 and C uses ALU2/3 (different port groups)
        let siw = SIW::new(
            DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
            SparseOp::SNOP,
            CoordOp::CPACK { rd: 5, rs1: 6, rs2: 7, fmt: MessageFormat::Result },
            PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        );

        let assignment = analyzer.assign_ports(&siw);
        // This should NOT conflict (D=ALU0, C=ALU2, different groups)
        assert!(!assignment.has_conflict);
    }
}
