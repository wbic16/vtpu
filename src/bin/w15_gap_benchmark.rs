//! R23W15: Gap Closure Benchmark
//!
//! Compares baseline vs optimized implementations.
//! Measures actual performance improvement.

use vtpu_runtime::{
    hdc::{HyperVector, AssociativeMemory, HDC_DEFAULT_WIDTH},
    hdc_optimized::{encode_coord_fast, similarity_fast, FastAssociativeMemory},
};
use std::time::Instant;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║        R23W15: Gap Closure Benchmark - Before vs After       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    bench_encoding();
    bench_similarity();
    bench_associative_memory();
    
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    Summary                                    ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Optimizations applied:");
    println!("  ✅ Basis vector caching (132 KB cache, amortized)");
    println!("  ✅ Manual loop unrolling (similarity)");
    println!("  ✅ Batch operations (encode, query)");
    println!();
    println!("Expected gains:");
    println!("  Encoding: 2-3× faster (basis caching)");
    println!("  Similarity: 1.2-1.5× faster (manual loops)");
    println!("  Overall HDC: 1.5-2× faster");
    println!();
}

fn bench_encoding() {
    println!("─── Benchmark 1: Coordinate Encoding ───");
    println!();
    
    let coords: Vec<[u16; 11]> = (0..1000)
        .map(|i| {
            let mut coord = [1u16; 11];
            coord[10] = (i % 256) as u16;
            coord
        })
        .collect();
    
    // Baseline (original)
    println!("BASELINE (HyperVector::from_coord):");
    let start = Instant::now();
    for coord in &coords {
        let _ = HyperVector::from_coord(coord, HDC_DEFAULT_WIDTH);
    }
    let baseline_time = start.elapsed();
    println!("  Time: {:.2} ms", baseline_time.as_secs_f64() * 1000.0);
    println!("  Ops/s: {:.2e}", 1000.0 / baseline_time.as_secs_f64());
    println!("  Time/op: {:.2} ns", baseline_time.as_nanos() as f64 / 1000.0);
    println!();
    
    // Optimized
    println!("OPTIMIZED (encode_coord_fast):");
    let start = Instant::now();
    for coord in &coords {
        let _ = encode_coord_fast(coord);
    }
    let optimized_time = start.elapsed();
    println!("  Time: {:.2} ms", optimized_time.as_secs_f64() * 1000.0);
    println!("  Ops/s: {:.2e}", 1000.0 / optimized_time.as_secs_f64());
    println!("  Time/op: {:.2} ns", optimized_time.as_nanos() as f64 / 1000.0);
    println!();
    
    let speedup = baseline_time.as_secs_f64() / optimized_time.as_secs_f64();
    println!("📈 SPEEDUP: {:.2}×", speedup);
    println!();
}

fn bench_similarity() {
    println!("─── Benchmark 2: Similarity Computation ───");
    println!();
    
    let coord1 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    let coord2 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12];
    
    let hv1 = HyperVector::from_coord(&coord1, HDC_DEFAULT_WIDTH);
    let hv2 = HyperVector::from_coord(&coord2, HDC_DEFAULT_WIDTH);
    
    let iterations = 100_000;
    
    // Baseline
    println!("BASELINE (HyperVector::similarity):");
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = hv1.similarity(&hv2);
    }
    let baseline_time = start.elapsed();
    println!("  Time: {:.2} ms", baseline_time.as_secs_f64() * 1000.0);
    println!("  Ops/s: {:.2e}", iterations as f64 / baseline_time.as_secs_f64());
    println!("  Time/op: {:.2} ns", baseline_time.as_nanos() as f64 / iterations as f64);
    println!();
    
    // Optimized
    println!("OPTIMIZED (similarity_fast):");
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = similarity_fast(&hv1, &hv2);
    }
    let optimized_time = start.elapsed();
    println!("  Time: {:.2} ms", optimized_time.as_secs_f64() * 1000.0);
    println!("  Ops/s: {:.2e}", iterations as f64 / optimized_time.as_secs_f64());
    println!("  Time/op: {:.2} ns", optimized_time.as_nanos() as f64 / iterations as f64);
    println!();
    
    let speedup = baseline_time.as_secs_f64() / optimized_time.as_secs_f64();
    println!("📈 SPEEDUP: {:.2}×", speedup);
    println!();
}

fn bench_associative_memory() {
    println!("─── Benchmark 3: Associative Memory Query ───");
    println!();
    
    let coords: Vec<[u16; 11]> = (0..100)
        .map(|i| {
            let mut coord = [1u16; 11];
            coord[10] = i as u16;
            coord
        })
        .collect();
    
    // Baseline
    let mut memory_baseline = AssociativeMemory::new();
    for coord in &coords {
        memory_baseline.store(*coord, HDC_DEFAULT_WIDTH);
    }
    
    let query_coord = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 50];
    let query_hv = HyperVector::from_coord(&query_coord, HDC_DEFAULT_WIDTH);
    
    println!("BASELINE (AssociativeMemory):");
    let iterations = 10_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = memory_baseline.query_nearest(&query_hv);
    }
    let baseline_time = start.elapsed();
    println!("  Time: {:.2} ms", baseline_time.as_secs_f64() * 1000.0);
    println!("  Ops/s: {:.2e}", iterations as f64 / baseline_time.as_secs_f64());
    println!("  Time/op: {:.2} ns", baseline_time.as_nanos() as f64 / iterations as f64);
    println!();
    
    // Optimized
    let mut memory_optimized = FastAssociativeMemory::new();
    for coord in &coords {
        memory_optimized.store(*coord);
    }
    
    let query_hv_opt = encode_coord_fast(&query_coord);
    
    println!("OPTIMIZED (FastAssociativeMemory):");
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = memory_optimized.query_nearest(&query_hv_opt);
    }
    let optimized_time = start.elapsed();
    println!("  Time: {:.2} ms", optimized_time.as_secs_f64() * 1000.0);
    println!("  Ops/s: {:.2e}", iterations as f64 / optimized_time.as_secs_f64());
    println!("  Time/op: {:.2} ns", optimized_time.as_nanos() as f64 / iterations as f64);
    println!();
    
    let speedup = baseline_time.as_secs_f64() / optimized_time.as_secs_f64();
    println!("📈 SPEEDUP: {:.2}×", speedup);
    println!();
}
