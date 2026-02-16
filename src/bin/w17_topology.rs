//! R23W17 — CPU Topology Discovery (zero deps)

use vtpu_runtime::affinity;

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  vTPU W17: CPU Topology (zero deps)");
    println!("═══════════════════════════════════════════════\n");

    let logical = affinity::num_cpus();
    let physical = affinity::num_physical_cores();
    let (l1, l2, l3) = affinity::cache_sizes();

    println!("  Physical cores:   {}", physical);
    println!("  Logical threads:  {}", logical);
    println!("  SMT:              {}", if logical > physical { "on" } else { "off" });
    println!("  L1: {}K  L2: {}K  L3: {}K", l1, l2, l3);

    let pairs = affinity::smt_pairs();
    if !pairs.is_empty() {
        println!("\n  SMT pairs:");
        for (a, b) in &pairs {
            println!("    CPU {} ↔ CPU {}", a, b);
        }
    }

    let topo = affinity::detect_topology();
    println!("\n  Sysfs topology ({} entries):", topo.len());
    for &(cpu, core, numa) in &topo {
        println!("    cpu{}: core={}, numa={}", cpu, core, numa);
    }

    println!("\n═══════════════════════════════════════════════");
}
