//! R23W17 - CPU Topology Detection Diagnostic
//!
//! Detects and displays system CPU topology.
//!
//! Usage: cargo run --release --bin detect_topology

use vtpu_runtime::CpuTopology;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║            R23W17: CPU Topology Detection                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    match CpuTopology::detect() {
        Ok(topology) => {
            println!("{}", topology);
            
            // Additional analysis
            println!();
            println!("Analysis:");
            
            if topology.has_smt() {
                println!("  ✅ SMT enabled ({}× threads per core)", topology.smt_width());
                println!("  → Can exploit complementary workloads (D-heavy + S-heavy)");
                println!("  → Expected SMT speedup: ~1.89× per core (W16 measured)");
            } else {
                println!("  ⚠️  SMT disabled or not available");
                println!("  → Running with 1 thread per core");
            }
            
            println!();
            
            if topology.numa_nodes.len() > 1 {
                println!("  ⚠️  Multi-NUMA system detected ({} nodes)", topology.numa_nodes.len());
                println!("  → Memory locality matters");
                println!("  → Use NUMA-aware allocation (future work: W18)");
            } else {
                println!("  ✅ Single NUMA node");
                println!("  → No cross-NUMA penalties");
            }
            
            println!();
            println!("Recommended configuration:");
            println!("  • Use {} SMT pairs for maximum throughput", topology.smt_pairs().len());
            println!("  • Total parallelism: {}× (physical cores × SMT width)", 
                     topology.physical_cores() * topology.smt_width());
            
            if topology.smt_pairs().len() >= 8 {
                println!("  • This system can run full W17 multi-core benchmarks");
            } else {
                println!("  • Limited parallelism - expect reduced speedups");
            }
        }
        Err(e) => {
            eprintln!("Failed to detect topology: {}", e);
            eprintln!();
            eprintln!("This tool requires:");
            eprintln!("  • Linux operating system");
            eprintln!("  • /sys/devices/system/cpu/ filesystem");
            eprintln!();
            eprintln!("If running on macOS/Windows, topology detection is not supported.");
            std::process::exit(1);
        }
    }
}
