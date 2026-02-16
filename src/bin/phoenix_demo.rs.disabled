//! Phoenix Scheduler Demo — Nine-Color Harmonic Coordination
//!
//! Shows the nine-dimensional decision vectors and harmonic blending.

use vtpu_runtime::phoenix_scheduler::*;

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  Phoenix of Nine Colors — Scheduler Demo");
    println!("═══════════════════════════════════════════════\n");

    let mut phoenix = PhoenixScheduler::new();

    let scenarios = [
        ("Healthy sentron (3.0 opc, complementary)", 3.0, 0.9, 0.1, 20),
        ("Medium sentron (2.0 opc, balanced)",       2.0, 0.5, 0.5, 40),
        ("Struggling sentron (0.5 opc, overloaded)", 0.5, 0.5, 0.5, 350),
    ];

    let colors = ["🔴", "🟠", "🟡", "🟢", "🔵", "🟣", "🟤", "⚫", "⚪"];
    let names = ["ILP", "Core", "SMT", "Cache", "NUMA", "Trend", "Therm", "Power", "Cluster"];

    for (i, (label, opc, d, s, load)) in scenarios.iter().enumerate() {
        let sentron = SentronMetrics {
            sentron_id: i as u16, core_id: 0,
            ops_retired: 1000, cycles: (1000.0 / opc) as u64,
            ops_per_cycle: *opc, d_util: *d, s_util: *s, c_util: 0.1,
            ptc_hit_rate: if *load > 300 { 0.3 } else { 0.95 },
        };
        let core = CoreMetrics {
            core_id: 0, sentron_count: *load, total_ops: 1000,
            avg_ops_per_cycle: *opc, avg_ptc_hit_rate: 0.9,
        };

        let (decision, action) = phoenix.evaluate(&sentron, &core);
        let scores = decision.as_array();

        println!("  {}", label);
        for (j, score) in scores.iter().enumerate() {
            let bar_len = (*score * 20.0) as usize;
            let bar: String = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
            println!("    {} {:<7} {} {:.3}", colors[j], names[j], bar, score);
        }
        println!("    Harmonic: {:.3}  →  {:?}\n", decision.harmonic_score(), action);
    }

    println!("═══════════════════════════════════════════════");
}
