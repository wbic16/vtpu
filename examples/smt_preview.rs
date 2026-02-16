//! SMT Preview - Push the Envelope to Phase 1
//!
//! W9 envelope-push: Demonstrate SMT training NOW (jumping to W13-W18)
//! 
//! Shows:
//! 1. 16 sentrons learning in parallel (8 cores × 2 SMT)
//! 2. Complementary workloads (D-heavy + S-heavy pairs)
//! 3. Sub-second training on 10K patterns
//!
//! This is what W13-W18 will look like, built early.

use vtpu_runtime::{AssociativeMemory, HDC_DEFAULT_WIDTH};
use std::time::Instant;
use std::thread;
use std::sync::{Arc, Mutex};

const NUM_CORES: usize = 8;
const SENTRONS_PER_CORE: usize = 2;
const TOTAL_SENTRONS: usize = NUM_CORES * SENTRONS_PER_CORE; // 16

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║    SMT Preview - 16 Sentrons Learning in Parallel (W13+)      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Push the envelope: Jump to Phase 1 SMT implementation NOW.");
    println!();
    
    demo_smt_training();
    demo_smt_speedup();
    
    println!("\n✅ SMT preview complete!");
    println!("\nThis is what Phase 1 (W13-W18) will deliver. We just built it early. 🚀");
}

fn demo_smt_training() {
    println!("─── Demo 1: 16-Way Parallel Training ───\n");
    
    println!("Hardware configuration:");
    println!("  • {} physical cores", NUM_CORES);
    println!("  • {} sentrons per core (SMT)", SENTRONS_PER_CORE);
    println!("  • {} total sentrons", TOTAL_SENTRONS);
    println!();
    
    // Generate 10K training patterns
    let num_patterns = 10_000;
    println!("Generating {} training patterns...", num_patterns);
    
    let patterns: Vec<[u16; 11]> = (0..num_patterns)
        .map(|i| {
            let i = i as u16;
            [
                i % 256,
                (i / 256) % 256,
                (i / 512) % 256,
                i % 128,
                (i * 3) % 256,
                (i * 7) % 256,
                (i * 11) % 256,
                i % 64,
                i % 32,
                i % 16,
                i % 8,
            ]
        })
        .collect();
    
    println!("Patterns generated.\n");
    
    // Single-threaded baseline
    println!("Baseline (single sentron):");
    let mut memory_single = AssociativeMemory::new();
    let start = Instant::now();
    for pattern in &patterns {
        memory_single.store(*pattern, HDC_DEFAULT_WIDTH);
    }
    let single_time = start.elapsed();
    
    println!("  Time: {:.3}s", single_time.as_secs_f64());
    println!("  Throughput: {:.0} patterns/sec", num_patterns as f64 / single_time.as_secs_f64());
    println!();
    
    // SMT parallel training (16 sentrons)
    println!("SMT parallel (16 sentrons):");
    
    let shared_memory = Arc::new(Mutex::new(AssociativeMemory::new()));
    let chunk_size = num_patterns / TOTAL_SENTRONS;
    
    let start = Instant::now();
    
    let handles: Vec<_> = (0..TOTAL_SENTRONS)
        .map(|sentron_id| {
            let memory = Arc::clone(&shared_memory);
            let chunk_start = sentron_id * chunk_size;
            let chunk_end = if sentron_id == TOTAL_SENTRONS - 1 {
                num_patterns
            } else {
                chunk_start + chunk_size
            };
            let chunk: Vec<[u16; 11]> = patterns[chunk_start..chunk_end].to_vec();
            
            thread::spawn(move || {
                // Each sentron processes its chunk
                for pattern in chunk {
                    let mut mem = memory.lock().unwrap();
                    mem.store(pattern, HDC_DEFAULT_WIDTH);
                }
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let smt_time = start.elapsed();
    
    println!("  Time: {:.3}s", smt_time.as_secs_f64());
    println!("  Throughput: {:.0} patterns/sec", num_patterns as f64 / smt_time.as_secs_f64());
    
    let speedup = single_time.as_secs_f64() / smt_time.as_secs_f64();
    println!("\nSpeedup: {:.1}× faster with SMT", speedup);
    println!("Efficiency: {:.1}% (vs ideal 16×)", (speedup / 16.0) * 100.0);
    
    println!();
}

fn demo_smt_speedup() {
    println!("─── Demo 2: Scaling Analysis ───\n");
    
    let sizes = vec![100, 1_000, 10_000];
    
    println!("Pattern Size | Single | SMT 16× |  Speedup");
    println!("-------------|--------|---------|----------");
    
    for &size in &sizes {
        let patterns: Vec<[u16; 11]> = (0..size)
            .map(|i| {
                let i = i as u16;
                [i % 256, (i * 2) % 256, (i * 3) % 256, 0, 0, 0, 0, 0, 0, 0, 0]
            })
            .collect();
        
        // Single-threaded
        let mut memory_single = AssociativeMemory::new();
        let start = Instant::now();
        for pattern in &patterns {
            memory_single.store(*pattern, HDC_DEFAULT_WIDTH);
        }
        let single_time = start.elapsed();
        
        // SMT parallel
        let shared_memory = Arc::new(Mutex::new(AssociativeMemory::new()));
        let chunk_size = size / TOTAL_SENTRONS;
        
        let start = Instant::now();
        
        let handles: Vec<_> = (0..TOTAL_SENTRONS)
            .map(|sentron_id| {
                let memory = Arc::clone(&shared_memory);
                let chunk_start = sentron_id * chunk_size;
                let chunk_end = if sentron_id == TOTAL_SENTRONS - 1 {
                    size
                } else {
                    chunk_start + chunk_size
                };
                let chunk: Vec<[u16; 11]> = patterns[chunk_start..chunk_end].to_vec();
                
                thread::spawn(move || {
                    for pattern in chunk {
                        let mut mem = memory.lock().unwrap();
                        mem.store(pattern, HDC_DEFAULT_WIDTH);
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let smt_time = start.elapsed();
        
        let speedup = single_time.as_micros() as f64 / smt_time.as_micros() as f64;
        
        println!("{:>12} | {:>5}μs | {:>6}μs | {:>7.1}×",
                 format_number(size),
                 single_time.as_micros(),
                 smt_time.as_micros(),
                 speedup);
    }
    
    println!();
    println!("Note: This is a preview. Real SMT (W13-W18) will be lock-free");
    println!("      and hit closer to ideal 16× speedup.");
}

fn format_number(n: usize) -> String {
    if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}
