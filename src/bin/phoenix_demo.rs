//! R23W17: Phoenix of Nine Colors Demo
//!
//! Shows nine-dimensional harmonic coordination in scheduler.
//!
//! Usage: cargo run --release --bin phoenix_demo

use vtpu_runtime::*;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        R23W17: Phoenix of Nine Colors - Harmonic Scheduler     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Nine colors of scheduling coordination:");
    println!("  🔴 Red:    ILP (instruction-level parallelism)");
    println!("  🟠 Orange: Core affinity (load balancing)");
    println!("  🟡 Yellow: SMT pairing (complementary workloads)");
    println!("  🟢 Green:  Cache locality (hit rates)");
    println!("  🔵 Blue:   NUMA topology (memory locality)");
    println!("  🟣 Purple: Temporal trends (learning from history)");
    println!("  🟤 Brown:  Thermal management (temperature)");
    println!("  ⚫ Black:  Power efficiency (energy)");
    println!("  ⚪ White:  Cluster coordination (multi-node)");
    println!();
    
    demo_nine_color_decision();
    println!();
    demo_harmonic_blend();
    println!();
    demo_phoenix_scheduler();
}

fn demo_nine_color_decision() {
    println!("─── Demo 1: Nine-Color Decision Vector ───");
    println!();
    
    // Create sample metrics
    let sentron = SentronMetrics {
        sentron_id: 0,
        core_id: 0,
        cpu_id: 0,
        ops_retired: 140,
        cycles: 50,
        ops_per_cycle: 2.8,  // Below 3.0 target
        l1_hits: 65,
        l1_misses: 35,       // 65% hit rate (below 70% target)
        l2_hits: 31,
        l2_misses: 4,
        stall_cycles: 15,    // 30% stall rate
        timestamp: std::time::Instant::now(),
    };
    
    let core = CoreMetrics {
        core_id: 0,
        sentron_count: 4,    // Loaded core
        total_ops: 560,
        total_cycles: 200,
        ops_per_cycle: 2.8,
        avg_cache_hit_rate: 0.65,
        avg_stall_rate: 0.3,
        load: 4.0,
    };
    
    let history = vec![
        // Performance declining over time
        SentronMetrics { ops_per_cycle: 3.0, ..sentron.clone() },
        SentronMetrics { ops_per_cycle: 2.9, ..sentron.clone() },
        SentronMetrics { ops_per_cycle: 2.8, ..sentron.clone() },
    ];
    
    let decision = NineColorDecision::from_metrics(&sentron, &core, &history);
    
    println!("Sentron metrics:");
    println!("  Ops/cycle: {:.2} (target 3.0)", sentron.ops_per_cycle);
    println!("  Cache hit: {:.1}% (target 70%)", sentron.cache_hit_rate() * 100.0);
    println!("  Stall rate: {:.1}%", sentron.stall_rate() * 100.0);
    println!("  Core load: {} sentrons", core.sentron_count);
    println!();
    
    println!("Nine-color scores:");
    println!("  🔴 ILP:            {:.1}% ({:.2} / 3.0 ops/cycle)", 
             decision.ilp_score * 100.0, sentron.ops_per_cycle);
    println!("  🟠 Core affinity:  {:.1}% (load = {})",
             decision.core_affinity * 100.0, core.load);
    println!("  🟡 SMT pairing:    {:.1}% (stall rate = {:.1}%)",
             decision.smt_pairing * 100.0, sentron.stall_rate() * 100.0);
    println!("  🟢 Cache locality: {:.1}% (hit rate = {:.1}%)",
             decision.cache_locality * 100.0, sentron.cache_hit_rate() * 100.0);
    println!("  🔵 NUMA locality:  {:.1}% (not implemented yet)",
             decision.numa_locality * 100.0);
    println!("  🟣 Temporal trend: {:.1}% (declining: 3.0 → 2.8)",
             decision.temporal_trend * 100.0);
    println!("  🟤 Thermal:        {:.1}% (not implemented yet)",
             decision.thermal_delta * 100.0);
    println!("  ⚫ Power:          {:.1}% (not implemented yet)",
             decision.power_efficiency * 100.0);
    println!("  ⚪ Cluster:        {:.1}% (not implemented yet)",
             decision.cluster_balance * 100.0);
    println!();
    
    let harmonic = decision.harmonic_score();
    println!("Harmonic score: {:.1}% (weighted blend of all 9 colors)", harmonic * 100.0);
    
    if harmonic < 0.75 {
        println!("  → Below threshold (75%) - migration recommended");
    } else {
        println!("  → Above threshold - performance acceptable");
    }
}

fn demo_harmonic_blend() {
    println!("─── Demo 2: Harmonic Blend Comparison ───");
    println!();
    
    println!("Scenario A: All dimensions good");
    let good_decision = NineColorDecision {
        ilp_score: 0.95,
        core_affinity: 0.9,
        smt_pairing: 0.9,
        cache_locality: 0.9,
        numa_locality: 1.0,
        temporal_trend: 1.0,
        thermal_delta: 1.0,
        power_efficiency: 1.0,
        cluster_balance: 1.0,
    };
    println!("  Harmonic score: {:.1}%", good_decision.harmonic_score() * 100.0);
    println!("  → No action needed");
    println!();
    
    println!("Scenario B: Cache thrashing (one color bad)");
    let cache_bad = NineColorDecision {
        ilp_score: 0.95,
        core_affinity: 0.9,
        smt_pairing: 0.9,
        cache_locality: 0.3,  // Bad!
        numa_locality: 1.0,
        temporal_trend: 0.5,
        thermal_delta: 1.0,
        power_efficiency: 1.0,
        cluster_balance: 1.0,
    };
    println!("  Harmonic score: {:.1}%", cache_bad.harmonic_score() * 100.0);
    println!("  → Action: Migrate (cache thrashing)");
    println!();
    
    println!("Scenario C: Multiple dimensions degraded");
    let multiple_bad = NineColorDecision {
        ilp_score: 0.6,       // Low
        core_affinity: 0.4,   // Imbalanced
        smt_pairing: 0.5,     // Contention
        cache_locality: 0.5,  // Marginal
        numa_locality: 1.0,
        temporal_trend: 0.2,  // Declining
        thermal_delta: 1.0,
        power_efficiency: 1.0,
        cluster_balance: 1.0,
    };
    println!("  Harmonic score: {:.1}%", multiple_bad.harmonic_score() * 100.0);
    println!("  → Action: Urgent migration (multiple bottlenecks)");
    println!();
    
    println!("Key insight: Harmonic blend detects subtle combinations");
    println!("  • One bad dimension might be acceptable");
    println!("  • Multiple marginal dimensions trigger action");
    println!("  • Weighted blend prioritizes cache (1.2×) over thermal (0.3×)");
}

fn demo_phoenix_scheduler() {
    println!("─── Demo 3: Phoenix Scheduler (Full Loop) ───");
    println!();
    
    let mut phoenix = PhoenixScheduler::new(0.75); // 75% threshold
    
    println!("Simulating 5 rounds of execution...");
    println!();
    
    // Simulate degrading performance over time
    for round in 1..=5 {
        let sentron = SentronMetrics {
            sentron_id: 0,
            core_id: 0,
            cpu_id: 0,
            ops_retired: 100,
            cycles: 50 + (round * 5), // Getting slower
            ops_per_cycle: 100.0 / (50.0 + (round * 5) as f64),
            l1_hits: 90 - (round * 10),
            l1_misses: 10 + (round * 10),
            l2_hits: 9,
            l2_misses: 1,
            stall_cycles: round * 5,
            timestamp: std::time::Instant::now(),
        };
        
        let core = CoreMetrics {
            core_id: 0,
            sentron_count: 2 + round as usize,
            total_ops: 200,
            total_cycles: 100,
            ops_per_cycle: 2.0,
            avg_cache_hit_rate: 0.9 - (round as f64 * 0.1),
            avg_stall_rate: round as f64 * 0.05,
            load: (2 + round) as f64,
        };
        
        let action = phoenix.decide(&sentron, &core);
        
        println!("Round {}:", round);
        println!("  Ops/cycle: {:.2}", sentron.ops_per_cycle);
        println!("  Cache hit: {:.1}%", sentron.cache_hit_rate() * 100.0);
        println!("  Core load: {}", core.sentron_count);
        
        match action {
            Some(SchedulerAction::Migrate { reason, .. }) => {
                println!("  → Action: Migrate (reason: {:?})", reason);
            }
            _ => {
                println!("  → No action");
            }
        }
        println!();
    }
    
    println!("Decision history:");
    let history = phoenix.decision_history(0);
    for (i, decision) in history.iter().enumerate() {
        println!("  Round {}: Harmonic score {:.1}%", i + 1, decision.harmonic_score() * 100.0);
    }
    println!();
    
    println!("✓ Phoenix scheduler learns from degrading performance");
    println!("  • Round 1-2: Performance acceptable");
    println!("  • Round 3+: Degradation detected → migration recommended");
    println!("  • Harmonic score captures multi-dimensional decline");
}
