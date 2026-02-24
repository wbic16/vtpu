//! PPT Benchmark — Coordinate lookup latency
//!
//! Target: <100ns per translate() on PTC hit
//! Measures both hit and miss paths.

use vtpu_runtime::{PhextPageTable, PhextCoord};
use std::time::Instant;

fn main() {
    println!("PPT Benchmark — Coordinate Lookup Latency");
    println!("==========================================");
    println!();
    
    // Warmup
    let mut ppt = PhextPageTable::new();
    for i in 0..1000 {
        let coord = PhextCoord::new([i as u16 % 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        ppt.translate(&coord);
    }
    ppt.reset_stats();
    
    // Benchmark 1: PTC Hit Path (same coordinates repeated)
    println!("1. PTC Hit Path (same 64 coords, 10K iterations each)");
    println!("   Target: <100ns per lookup");
    println!();
    
    let coords: Vec<_> = (0..64)
        .map(|i| PhextCoord::new([i as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]))
        .collect();
    
    // Prime the cache
    for coord in &coords {
        ppt.translate(coord);
    }
    ppt.reset_stats();
    
    let iterations = 10_000;
    let total_lookups = coords.len() * iterations;
    
    let start = Instant::now();
    for _ in 0..iterations {
        for coord in &coords {
            let _ = ppt.translate(coord);
        }
    }
    let elapsed = start.elapsed();
    
    let ns_per_lookup = elapsed.as_nanos() as f64 / total_lookups as f64;
    let lookups_per_sec = total_lookups as f64 / elapsed.as_secs_f64();
    
    let stats = ppt.stats();
    println!("   Total lookups:     {:>12}", total_lookups);
    println!("   Elapsed:           {:>12.2?}", elapsed);
    println!("   PTC hit rate:      {:>12.1}%", stats.ptc_hit_rate * 100.0);
    println!("   Latency per lookup:{:>12.1} ns", ns_per_lookup);
    println!("   Throughput:        {:>12.1} M/sec", lookups_per_sec / 1_000_000.0);
    
    let hit_pass = ns_per_lookup < 100.0;
    println!("   Status:            {} (target <100ns)", if hit_pass { "✅ PASS" } else { "❌ FAIL" });
    println!();
    
    // Benchmark 2: PTC Miss Path (new coordinates each time)
    println!("2. PTC Miss Path (unique coords, measures full translate)");
    println!();
    
    let mut ppt2 = PhextPageTable::new();
    let unique_coords: Vec<_> = (0..100_000)
        .map(|i| PhextCoord::new([
            (i % 100) as u16,
            ((i / 100) % 100) as u16,
            ((i / 10000) % 100) as u16,
            (i % 256) as u16,
            1, 1, 1, 1, 1, 1, 1
        ]))
        .collect();
    
    let start = Instant::now();
    for coord in &unique_coords {
        let _ = ppt2.translate(coord);
    }
    let elapsed = start.elapsed();
    
    let ns_per_lookup = elapsed.as_nanos() as f64 / unique_coords.len() as f64;
    let lookups_per_sec = unique_coords.len() as f64 / elapsed.as_secs_f64();
    
    let stats = ppt2.stats();
    println!("   Total lookups:     {:>12}", unique_coords.len());
    println!("   Elapsed:           {:>12.2?}", elapsed);
    println!("   PTC hit rate:      {:>12.1}%", stats.ptc_hit_rate * 100.0);
    println!("   Latency per lookup:{:>12.1} ns", ns_per_lookup);
    println!("   Throughput:        {:>12.1} M/sec", lookups_per_sec / 1_000_000.0);
    println!("   Regions allocated: {:>12}", stats.regions_allocated);
    println!();
    
    // Benchmark 3: Structured Workload (realistic: sweep inner dims)
    println!("3. Structured Workload (sweep dim 0-2, fixed outer)");
    println!();
    
    let mut ppt3 = PhextPageTable::new();
    let sweep_size: u16 = 32;
    let sweeps: usize = 100;
    let structured_count: usize = (sweep_size as usize) * (sweep_size as usize) * (sweep_size as usize) * sweeps;
    
    let start = Instant::now();
    for _ in 0..sweeps {
        for d0 in 0..sweep_size {
            for d1 in 0..sweep_size {
                for d2 in 0..sweep_size {
                    let coord = PhextCoord::new([d0, d1, d2, 1, 1, 1, 1, 1, 1, 1, 1]);
                    let _ = ppt3.translate(&coord);
                }
            }
        }
    }
    let elapsed = start.elapsed();
    
    let ns_per_lookup = elapsed.as_nanos() as f64 / structured_count as f64;
    let lookups_per_sec = structured_count as f64 / elapsed.as_secs_f64();
    
    let stats = ppt3.stats();
    println!("   Total lookups:     {:>12}", structured_count);
    println!("   Elapsed:           {:>12.2?}", elapsed);
    println!("   PTC hit rate:      {:>12.1}%", stats.ptc_hit_rate * 100.0);
    println!("   Latency per lookup:{:>12.1} ns", ns_per_lookup);
    println!("   Throughput:        {:>12.1} M/sec", lookups_per_sec / 1_000_000.0);
    println!();
    
    // Summary
    println!("==========================================");
    println!("Summary:");
    println!("  PTC Hit Path:   {} ({:.1} ns)", 
        if hit_pass { "✅" } else { "❌" }, 
        elapsed.as_nanos() as f64 / total_lookups as f64);
    println!("  Target:         <100ns per coordinate lookup");
}
