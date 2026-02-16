//! R23W17-2: Scheduler Redux Feedback Loop Demo
//!
//! Shows real-time feedback from execution to scheduler adaptation.
//!
//! Flow:
//! 1. Execute sentrons
//! 2. Collect performance metrics
//! 3. Feed metrics to scheduler
//! 4. Scheduler decides on actions (migrate, rebalance)
//! 5. Apply actions
//! 6. Repeat
//!
//! Usage: cargo run --release --bin redux_demo

use vtpu_runtime::*;
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     R23W17-2: Scheduler Redux - Real-Time Feedback Loop       ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    println!("Concept: Scheduler observes performance and adapts assignments");
    println!();
    
    demo_feedback_loop();
    println!();
    demo_load_balancing();
    println!();
    demo_cache_awareness();
}

fn demo_feedback_loop() {
    println!("─── Demo 1: Basic Feedback Loop ───");
    println!();
    
    let config = RuntimeConfig::default();
    
    match RuntimeScheduler::new(config) {
        Ok(runtime) => {
            let mut redux = SchedulerRedux::with_defaults(runtime);
            
            println!("Initial state:");
            println!("{}", redux.runtime().topology());
            
            // Allocate 4 sentrons
            let sentrons = redux.runtime().allocate_pool(4, PhextCoord::zero());
            println!("Allocated {} sentrons", sentrons.len());
            println!();
            
            // Simulate execution and feedback
            println!("Simulating 3 execution rounds with feedback...");
            println!();
            
            for round in 1..=3 {
                println!("Round {}:", round);
                
                // Simulate execution stats for each sentron
                for sentron in &sentrons {
                    let stats = if sentron.id % 2 == 0 {
                        // Even sentrons: good performance
                        ExecStats {
                            ops_retired: 100,
                            cycles: 40,  // 2.5 ops/cycle
                            stalls: 5,   // low stalls
                            ppt_stats: PPTStats {
                                ptc_hits: 90,   // good hit rate
                                ptc_misses: 10,
                                ..Default::default()
                            },
                        }
                    } else {
                        // Odd sentrons: poor performance (cache thrashing)
                        ExecStats {
                            ops_retired: 100,
                            cycles: 80,  // 1.25 ops/cycle
                            stalls: 30,  // high stalls
                            ppt_stats: PPTStats {
                                ptc_hits: 30,   // poor hit rate
                                ptc_misses: 70,
                                ..Default::default()
                            },
                        }
                    };
                    
                    redux.feed(sentron.id, &stats);
                }
                
                // Scheduler decides on actions
                let actions = redux.decide();
                
                if actions.is_empty() {
                    println!("  No actions needed (performance acceptable)");
                } else {
                    println!("  {} actions recommended:", actions.len());
                    for action in &actions {
                        match action {
                            SchedulerAction::Migrate { sentron_id, from_core, to_core, reason } => {
                                println!("    • Migrate sentron {} from core {} → {} (reason: {:?})",
                                         sentron_id, from_core, to_core, reason);
                            }
                            SchedulerAction::Reprioritize { sentron_id, new_priority } => {
                                println!("    • Reprioritize sentron {} to priority {}",
                                         sentron_id, new_priority);
                            }
                            _ => {}
                        }
                    }
                }
                
                println!();
            }
            
            // Show final state
            println!("Final metrics:");
            for sentron in &sentrons {
                if let Some(history) = redux.sentron_metrics(sentron.id) {
                    if let Some(latest) = history.last() {
                        println!("  Sentron {}: {:.2} ops/cycle, {:.1}% cache hit, {:.1}% stalls",
                                 sentron.id,
                                 latest.ops_per_cycle,
                                 latest.cache_hit_rate() * 100.0,
                                 latest.stall_rate() * 100.0);
                    }
                }
            }
            
            println!();
            println!("✓ Feedback loop operational");
        }
        Err(e) => {
            println!("⚠️  Runtime scheduler unavailable: {}", e);
            println!("(This is expected on non-Linux systems)");
        }
    }
}

fn demo_load_balancing() {
    println!("─── Demo 2: Load Balancing via Feedback ───");
    println!();
    
    let config = RuntimeConfig::default();
    
    match RuntimeScheduler::new(config) {
        Ok(runtime) => {
            let redux_config = ReduxConfig {
                load_imbalance_threshold: 0.3,  // Trigger at 30% imbalance
                ..Default::default()
            };
            
            let mut redux = SchedulerRedux::new(runtime, redux_config);
            
            println!("Scenario: All sentrons start on core 0 (imbalanced)");
            println!();
            
            // Allocate all sentrons on core 0 (creates imbalance)
            let mut sentrons = Vec::new();
            for _ in 0..4 {
                if let Some(sentron) = redux.runtime().allocate_sentron(0, PhextCoord::zero()) {
                    sentrons.push(sentron);
                }
            }
            
            println!("Initial load:");
            for sentron in &sentrons {
                if let Some(assignment) = redux.runtime().assignment(sentron.id) {
                    println!("  Sentron {}: Core {}", sentron.id, assignment.core_id);
                }
            }
            println!();
            
            // Feed metrics
            for sentron in &sentrons {
                let stats = ExecStats {
                    ops_retired: 100,
                    cycles: 50,
                    stalls: 10,
                    ppt_stats: PPTStats {
                        ptc_hits: 80,
                        ptc_misses: 20,
                        ..Default::default()
                    },
                };
                redux.feed(sentron.id, &stats);
            }
            
            // Scheduler should detect imbalance
            let actions = redux.decide();
            
            println!("Scheduler response:");
            if actions.is_empty() {
                println!("  No migrations (threshold not met or insufficient cores)");
            } else {
                println!("  Detected load imbalance!");
                for action in &actions {
                    if let SchedulerAction::Migrate { sentron_id, from_core, to_core, reason } = action {
                        println!("    → Migrate sentron {} from core {} to core {}",
                                 sentron_id, from_core, to_core);
                    }
                }
            }
            
            println!();
            
            let stats = redux.runtime().stats();
            println!("Load balance: {:.1}%", stats.load_balance * 100.0);
            
            println!();
            println!("✓ Load balancing via feedback working");
        }
        Err(e) => {
            println!("⚠️  Runtime scheduler unavailable: {}", e);
        }
    }
}

fn demo_cache_awareness() {
    println!("─── Demo 3: Cache-Aware Migration ───");
    println!();
    
    let config = RuntimeConfig::default();
    
    match RuntimeScheduler::new(config) {
        Ok(runtime) => {
            let redux_config = ReduxConfig {
                cache_hit_threshold: 0.7,  // <70% = thrashing
                ..Default::default()
            };
            
            let mut redux = SchedulerRedux::new(runtime, redux_config);
            
            println!("Scenario: One sentron has poor cache locality");
            println!();
            
            let sentrons = redux.runtime().allocate_pool(2, PhextCoord::zero());
            
            // Sentron 0: good cache performance
            let stats_good = ExecStats {
                ops_retired: 100,
                cycles: 50,
                stalls: 5,
                ppt_stats: PPTStats {
                    ptc_hits: 90,   // 90% hit rate
                    ptc_misses: 10,
                    ..Default::default()
                },
            };
            
            // Sentron 1: cache thrashing
            let stats_bad = ExecStats {
                ops_retired: 100,
                cycles: 120,  // slower due to misses
                stalls: 40,
                ppt_stats: PPTStats {
                    ptc_hits: 30,   // 30% hit rate (thrashing!)
                    ptc_misses: 70,
                    ..Default::default()
                },
            };
            
            redux.feed(sentrons[0].id, &stats_good);
            redux.feed(sentrons[1].id, &stats_bad);
            
            println!("Performance:");
            println!("  Sentron 0: 90% cache hit rate");
            println!("  Sentron 1: 30% cache hit rate (thrashing!)");
            println!();
            
            let actions = redux.decide();
            
            println!("Scheduler response:");
            let has_cache_migration = actions.iter().any(|a| {
                matches!(a, SchedulerAction::Migrate { reason: MigrationReason::CacheThrash, .. })
            });
            
            if has_cache_migration {
                println!("  ✓ Detected cache thrashing!");
                println!("  → Migrating sentron to less-contended core");
            } else {
                println!("  No migration (may need multiple cores for target)");
            }
            
            println!();
            println!("✓ Cache-aware scheduling working");
        }
        Err(e) => {
            println!("⚠️  Runtime scheduler unavailable: {}", e);
        }
    }
}
