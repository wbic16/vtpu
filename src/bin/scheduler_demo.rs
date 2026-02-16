//! R23W17-2: End-to-End Scheduler Coordination Demo
//!
//! Demonstrates three-layer scheduling:
//! 1. Instruction-level: Reorder SIWs for ILP (scheduler.rs)
//! 2. Runtime-level: Assign sentrons to cores (runtime_scheduler.rs)
//! 3. OS-level: Pin threads to CPUs (affinity.rs - future)
//!
//! Usage: cargo run --release --bin scheduler_demo

use vtpu_runtime::*;
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     R23W17-2: End-to-End Scheduler Coordination                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    demo_instruction_scheduling();
    println!();
    demo_runtime_scheduling();
    println!();
    demo_coordinated_execution();
}

fn demo_instruction_scheduling() {
    println!("─── Layer 1: Instruction-Level Scheduling ───");
    println!("(Reorder SIWs for maximum ILP)");
    println!();
    
    let mut scheduler = Scheduler::new();
    
    // Create a stream with dependencies
    let stream = vec![
        // r0 = 10
        SIW::new(
            DenseOp::DMOV { rd: 0, imm: 10 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // r1 = r0 + r0  (depends on r0)
        SIW::new(
            DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // r2 = 5  (independent)
        SIW::new(
            DenseOp::DMOV { rd: 2, imm: 5 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // r3 = r1 + r2  (depends on r1 and r2)
        SIW::new(
            DenseOp::DADD { rd: 3, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
    ];
    
    println!("Original stream ({} SIWs):", stream.len());
    println!("  RAW hazards: {}", detect_hazards_count(&stream));
    println!();
    
    let result = scheduler.schedule(&stream);
    
    println!("After scheduling:");
    println!("  SIWs: {} (original) → {} (scheduled)", stream.len(), result.stream.len());
    println!("  NOPs inserted: {}", result.nops_inserted);
    println!("  SIWs reordered: {}", result.reordered);
    println!("  ILP before: {:.2}", result.before.ilp);
    println!("  ILP after: {:.2}", result.after.ilp);
    println!();
    
    println!("✓ Instruction scheduler optimizes SIW streams");
}

fn detect_hazards_count(stream: &[SIW]) -> usize {
    let mut count = 0;
    for window in stream.windows(2) {
        let hazards = vtpu_runtime::regalloc::detect_hazards(window);
        count += hazards.len();
    }
    count
}

fn demo_runtime_scheduling() {
    println!("─── Layer 2: Runtime-Level Scheduling ───");
    println!("(Assign sentrons to physical cores)");
    println!();
    
    let config = RuntimeConfig {
        use_smt: true,
        core_count: 0, // use all
        numa_aware: true,
        thread_priority: 0,
        scheduler_policy: SchedulerPolicy::Batch,
    };
    
    match RuntimeScheduler::new(config) {
        Ok(runtime_sched) => {
            println!("Detected topology:");
            println!("{}", runtime_sched.topology());
            
            // Allocate 8 sentrons
            let sentrons = runtime_sched.allocate_pool(8, PhextCoord::zero());
            println!("Allocated {} sentrons", sentrons.len());
            println!();
            
            println!("Assignments:");
            for sentron in &sentrons {
                if let Some(assignment) = runtime_sched.assignment(sentron.id) {
                    println!("  Sentron {}: Core {} (CPU {}, NUMA node {})",
                             sentron.id, assignment.core_id, assignment.cpu_id, assignment.numa_node);
                }
            }
            println!();
            
            let stats = runtime_sched.stats();
            println!("{}", stats);
            
            println!("✓ Runtime scheduler distributes work across cores");
        }
        Err(e) => {
            println!("⚠️  Could not detect topology: {}", e);
            println!("(This is expected on non-Linux systems)");
        }
    }
}

fn demo_coordinated_execution() {
    println!("─── Layer 3: Coordinated Execution ───");
    println!("(Instruction scheduling + runtime scheduling together)");
    println!();
    
    let config = RuntimeConfig::default();
    
    match RuntimeScheduler::new(config) {
        Ok(runtime_sched) => {
            println!("Scenario: Distribute 4 workloads across physical cores");
            println!();
            
            // Create 4 sentrons (one per core, up to 4 cores)
            let core_count = runtime_sched.topology().physical_cores().min(4);
            let mut sentrons = Vec::new();
            
            for core_id in 0..core_count {
                if let Some(sentron) = runtime_sched.allocate_sentron(core_id, PhextCoord::zero()) {
                    sentrons.push(sentron);
                }
            }
            
            println!("Allocated {} sentrons across {} cores", sentrons.len(), core_count);
            
            // Create a workload for each sentron
            let mut inst_scheduler = Scheduler::new();
            
            let workload = vec![
                SIW::new(DenseOp::DMOV { rd: 0, imm: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                SIW::new(DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                SIW::new(DenseOp::DMUL { rd: 2, rs1: 1, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            ];
            
            println!();
            println!("Per-sentron workload:");
            println!("  SIWs: {}", workload.len());
            
            // Schedule each workload
            let mut total_nops = 0;
            
            for (i, sentron) in sentrons.iter().enumerate() {
                let result = inst_scheduler.schedule(&workload);
                total_nops += result.nops_inserted;
                
                if let Some(assignment) = runtime_sched.assignment(sentron.id) {
                    println!("  Sentron {} (Core {}): {} NOPs inserted, ILP {:.2}",
                             i, assignment.core_id, result.nops_inserted, result.after.ilp);
                }
            }
            
            println!();
            println!("Total NOPs inserted: {}", total_nops);
            println!("Total SIWs scheduled: {}", sentrons.len() * workload.len());
            println!();
            
            println!("Coordination benefits:");
            println!("  • Instruction scheduler: Maximizes ILP per sentron");
            println!("  • Runtime scheduler: Distributes sentrons across cores");
            println!("  • OS scheduler: Executes on SMT siblings (future: affinity)");
            println!();
            
            println!("✓ Three-layer coordination working");
        }
        Err(e) => {
            println!("⚠️  Runtime scheduler unavailable: {}", e);
            println!("Falling back to instruction scheduling only");
            
            let mut scheduler = Scheduler::new();
            let workload = vec![
                SIW::new(DenseOp::DMOV { rd: 0, imm: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
                SIW::new(DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            ];
            
            let result = scheduler.schedule(&workload);
            println!("Scheduled {} SIWs, {} NOPs inserted", result.stream.len(), result.nops_inserted);
        }
    }
}
