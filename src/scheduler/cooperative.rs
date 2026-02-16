//! Cooperative Thread Scheduler (R23W17-2)
//!
//! Executes SIW streams with OS scheduler cooperation via quantized execution
//! and strategic yield points.

use crate::siw::SIW;
use crate::Memory;
use crate::Sentron;
use crate::exec::{run, ExecStats};
use super::workload::{WorkloadStats, quantize_stream, should_yield, DEFAULT_QUANTUM};

/// Cooperative execution result
#[derive(Debug, Clone)]
pub struct CoopResult {
    /// Execution statistics
    pub stats: ExecStats,
    
    /// Workload classification
    pub workload: WorkloadStats,
    
    /// Number of quanta executed
    pub quanta_executed: usize,
    
    /// Number of yields to OS scheduler
    pub yields: usize,
}

/// Execute a SIW stream cooperatively with OS scheduler
///
/// Breaks stream into quanta (default 64 SIWs) and yields at natural boundaries.
/// This allows other threads to make progress and reduces SMT contention.
///
/// # Arguments
/// * `stream` - SIW instructions to execute
/// * `sentron` - Execution context
/// * `memory` - Memory subsystem
/// * `quantum_size` - SIWs per quantum (None = default 64)
///
/// # Returns
/// Cooperative execution result with stats and workload classification
pub fn execute_cooperative(
    stream: &[SIW],
    sentron: &mut Sentron,
    memory: &mut Memory,
    quantum_size: Option<usize>,
) -> CoopResult {
    let quantum = quantum_size.unwrap_or(DEFAULT_QUANTUM);
    
    // Analyze workload
    let workload = WorkloadStats::analyze(stream);
    
    // Quantize stream
    let quanta = quantize_stream(stream, quantum);
    let total_quanta = quanta.len();
    
    // Execute quanta with cooperative yields
    let mut total_stats = ExecStats::default();
    let mut yields = 0;
    
    for (idx, quantum_slice) in quanta.iter().enumerate() {
        // Load quantum into sentron and execute
        sentron.spawn(quantum_slice.to_vec());
        let quantum_stats = run(sentron, memory);
        
        // Accumulate stats
        total_stats.ops_retired += quantum_stats.ops_retired;
        total_stats.cycles += quantum_stats.cycles;
        
        // Yield if appropriate
        if should_yield(idx, total_quanta) {
            std::thread::yield_now();
            yields += 1;
        }
    }
    
    CoopResult {
        stats: total_stats,
        workload,
        quanta_executed: total_quanta,
        yields,
    }
}

/// Execute two streams cooperatively as complementary workloads
///
/// If workloads are complementary (D-heavy + S-heavy), they can benefit from
/// running on separate cores (not SMT siblings, which showed 0.61× on Zen 4).
///
/// Note: This is a sequential cooperative executor. For actual parallel execution,
/// use separate threads with shared Memory via Arc<Mutex<>>.
///
/// # Returns
/// (result_a, result_b)
pub fn execute_pair_cooperative(
    stream_a: &[SIW],
    stream_b: &[SIW],
    sentron_a: &mut Sentron,
    sentron_b: &mut Sentron,
    memory: &mut Memory,
    quantum_size: Option<usize>,
) -> (CoopResult, CoopResult) {
    // Analyze both workloads
    let stats_a = WorkloadStats::analyze(stream_a);
    let stats_b = WorkloadStats::analyze(stream_b);
    
    // Log complementarity
    if stats_a.is_complementary(&stats_b) {
        eprintln!("Note: Workloads are complementary ({:?} + {:?})", stats_a.class, stats_b.class);
        eprintln!("      Should benefit from running on separate cores (not SMT siblings)");
    } else {
        eprintln!("Warning: Workloads are NOT complementary (both {:?})", stats_a.class);
        eprintln!("         May experience port contention");
    }
    
    // Execute sequentially with cooperative yields
    // (Real parallel execution requires thread spawning with shared memory)
    let result_a = execute_cooperative(stream_a, sentron_a, memory, quantum_size);
    let result_b = execute_cooperative(stream_b, sentron_b, memory, quantum_size);
    
    (result_a, result_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PhextCoord, DenseOp, SparseOp, CoordOp};
    
    fn make_d_heavy_stream(count: usize) -> Vec<SIW> {
        (0..count)
            .map(|i| {
                SIW::new(
                    DenseOp::DADD { rd: 0, rs1: 1, rs2: 2 },
                    SparseOp::SNOP,
                    CoordOp::CNOP,
                    PhextCoord::zero(),
                )
            })
            .collect()
    }
    
    #[test]
    fn test_cooperative_execution() {
        // Need >5 quanta to trigger a yield
        // Quantum 4 yields if it's not the final quantum
        let stream = make_d_heavy_stream(400); // 400 / 64 = 6.25 → 7 quanta
        let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let mut memory = Memory::new();
        
        let result = execute_cooperative(&stream, &mut sentron, &mut memory, Some(64));
        
        assert_eq!(result.workload.siw_count, 400);
        assert_eq!(result.quanta_executed, 7); // 400 / 64 = 7 quanta
        assert!(result.yields > 0); // Should have yielded at quantum 4
    }
    
    #[test]
    fn test_workload_classification() {
        let stream = make_d_heavy_stream(100);
        let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
        let mut memory = Memory::new();
        
        let result = execute_cooperative(&stream, &mut sentron, &mut memory, None);
        
        use super::super::workload::WorkloadClass;
        assert_eq!(result.workload.class, WorkloadClass::DenseHeavy);
    }
}
