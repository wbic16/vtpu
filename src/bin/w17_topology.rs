//! R23W17 - CPU Topology Discovery
//!
//! Detects and displays CPU topology for OS scheduler coordination.
//!
//! Usage: cargo run --release --bin w17_topology

use vtpu_runtime::scheduler::CpuTopology;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           R23W17: CPU Topology Discovery                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    let topo = CpuTopology::discover();
    
    println!("Hardware Threads: {}", topo.hardware_threads);
    println!("Physical Cores:   {}", topo.physical_cores);
    println!("SMT Enabled:      {}", if topo.has_smt() { "Yes" } else { "No" });
    println!();
    
    if topo.has_smt() {
        println!("SMT Sibling Pairs:");
        let mut seen = vec![false; topo.hardware_threads];
        
        for (cpu, sibling_opt) in &topo.siblings {
            if seen[*cpu] {
                continue;
            }
            
            if let Some(sibling) = sibling_opt {
                println!("  CPU {:2} ↔ CPU {:2} (same physical core)", cpu, sibling);
                seen[*cpu] = true;
                seen[*sibling] = true;
            } else {
                println!("  CPU {:2} (no SMT sibling)", cpu);
                seen[*cpu] = true;
            }
        }
        println!();
    }
    
    println!("Cache Hierarchy:");
    println!("  L1 Data:        {} KB per core", topo.cache.l1d_kb);
    println!("  L1 Instruction: {} KB per core", topo.cache.l1i_kb);
    println!("  L2:             {} KB per core", topo.cache.l2_kb);
    println!("  L3:             {} KB shared", topo.cache.l3_kb);
    println!();
    
    if !topo.numa_nodes.is_empty() {
        println!("NUMA Topology:");
        for node in &topo.numa_nodes {
            println!("  Node {}:", node.id);
            println!("    CPUs:   {:?}", node.cpus);
            println!("    Memory: {:.1} GB", node.memory_gb);
        }
        println!();
        
        println!("NUMA Recommendation:");
        println!("  Pin threads + memory to same node for best performance.");
        println!("  Local memory = 1.0× latency");
        println!("  Remote memory = 1.5-2.0× latency");
    } else {
        println!("NUMA: Single node (or not detected)");
    }
    println!();
    
    println!("Optimal SMT Pairing Strategy:");
    if topo.has_smt() {
        println!("  For 1.8× SMT speedup:");
        println!("  - Thread A (D-Pipe heavy) → CPU 0");
        println!("  - Thread B (S-Pipe heavy) → CPU {} (SMT sibling)", 
                 topo.smt_sibling(0).unwrap_or(1));
        println!("  - Complementary workloads minimize port conflicts");
        println!("  - Shared L1/L2 cache enables zero-copy handoff");
    } else {
        println!("  No SMT detected — use different physical cores");
        println!("  - Thread A → CPU 0");
        println!("  - Thread B → CPU 1");
    }
    println!();
    
    #[cfg(target_os = "linux")]
    {
        println!("Platform: Linux");
        println!("  CPU pinning: Supported (sched_setaffinity)");
        println!("  Thread naming: Supported (pthread_setname_np)");
        println!("  NUMA: {}", if !topo.numa_nodes.is_empty() { "Detected" } else { "Not detected" });
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("Platform: macOS");
        println!("  CPU pinning: Not supported (best-effort hints only)");
        println!("  Thread naming: Supported");
        println!("  NUMA: Not applicable (single node)");
    }
    
    #[cfg(target_os = "windows")]
    {
        println!("Platform: Windows");
        println!("  CPU pinning: Supported (SetThreadAffinityMask)");
        println!("  Thread naming: Supported (SetThreadDescription)");
        println!("  NUMA: Detection not yet implemented");
    }
    
    println!();
    println!("Next: Run w17_smt_pinned.rs with actual thread pinning");
}
