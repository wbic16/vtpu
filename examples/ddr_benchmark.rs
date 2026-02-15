// DDR5 Memory Benchmark - R23 Wave 4
//
// Characterize DDR5 performance on AMD R9 8945HS ranch nodes:
// - Sequential bandwidth (read/write streaming)
// - Random access latency (cache miss penalty)
// - Scatter/gather patterns (sparse workload)
// - Working set scaling (L1 → L2 → L3 → DDR transitions)
//
// Target: Identify where DDR becomes bottleneck for vTPU operations

use std::time::Instant;
use vtpu_runtime::PhextCoord;

const KB: usize = 1024;
const MB: usize = 1024 * KB;

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("  DDR5 Memory Benchmark — R23W4");
    println!("  Target: AMD R9 8945HS + DDR5-5600 (92 GB)");
    println!("═══════════════════════════════════════════════════════\n");

    // Measure cache sizes (via latency transitions)
    measure_cache_hierarchy();
    
    // Sequential bandwidth (best case)
    measure_sequential_bandwidth();
    
    // Random access latency (worst case for cache)
    measure_random_latency();
    
    // Scatter/gather (sparse access pattern)
    measure_scatter_gather();
    
    // Working set scaling (vTPU workload sizes)
    measure_working_set_scaling();
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("  DDR5 Benchmark Complete");
    println!("═══════════════════════════════════════════════════════");
}

/// Detect L1/L2/L3 cache sizes by measuring latency transitions
fn measure_cache_hierarchy() {
    println!("┌─ Cache Hierarchy Detection ─────────────────────────");
    
    // Test sizes from 4 KB to 128 MB
    let sizes = [
        ("4 KB", 4 * KB),
        ("8 KB", 8 * KB),
        ("16 KB", 16 * KB),
        ("32 KB", 32 * KB),     // Expected L1d boundary
        ("64 KB", 64 * KB),
        ("128 KB", 128 * KB),
        ("256 KB", 256 * KB),
        ("512 KB", 512 * KB),   // Expected L2 boundary
        ("1 MB", 1 * MB),
        ("2 MB", 2 * MB),
        ("4 MB", 4 * MB),
        ("8 MB", 8 * MB),
        ("16 MB", 16 * MB),     // Expected L3 boundary
        ("32 MB", 32 * MB),
        ("64 MB", 64 * MB),
        ("128 MB", 128 * MB),   // DDR5 territory
    ];
    
    for (name, size) in &sizes {
        let latency_ns = measure_latency_at_size(*size);
        let tier = classify_tier(latency_ns);
        println!("│ {:<8}: {:>6.1} ns  {}", name, latency_ns, tier);
    }
    
    println!("└──────────────────────────────────────────────────────\n");
}

/// Measure average access latency for given working set size
fn measure_latency_at_size(size: usize) -> f64 {
    // Allocate array larger than size to avoid prefetch benefits
    let count = size / 8;  // 8 bytes per u64
    let mut data: Vec<u64> = vec![0; count];
    
    // Initialize with pointer-chase pattern to defeat prefetcher
    for i in 0..count {
        data[i] = ((i * 7919) % count) as u64;  // Prime stride
    }
    
    // Warm up
    let mut idx = 0usize;
    for _ in 0..100 {
        idx = data[idx] as usize;
    }
    
    // Measure
    let iterations = 10_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
        idx = data[idx] as usize;
    }
    
    let elapsed = start.elapsed();
    
    // Prevent optimization
    std::hint::black_box(idx);
    
    // Return average latency per access
    elapsed.as_nanos() as f64 / (iterations * 10) as f64
}

/// Classify memory tier based on latency
fn classify_tier(latency_ns: f64) -> &'static str {
    if latency_ns < 2.0 {
        "L1 (< 2 ns)"
    } else if latency_ns < 10.0 {
        "L2 (< 10 ns)"
    } else if latency_ns < 30.0 {
        "L3 (< 30 ns)"
    } else if latency_ns < 100.0 {
        "DDR5 (< 100 ns)"
    } else {
        "DDR5 + TLB miss"
    }
}

/// Measure sequential read/write bandwidth
fn measure_sequential_bandwidth() {
    println!("┌─ Sequential Bandwidth (Best Case) ───────────────────");
    
    // Test different sizes to see bandwidth scaling
    let sizes = [
        ("1 MB", 1 * MB),
        ("16 MB", 16 * MB),      // Fits in L3
        ("64 MB", 64 * MB),      // Exceeds L3
        ("256 MB", 256 * MB),    // Pure DDR5
    ];
    
    for (name, size) in &sizes {
        measure_bandwidth_at_size(*name, *size);
    }
    
    println!("└──────────────────────────────────────────────────────\n");
}

fn measure_bandwidth_at_size(name: &str, size: usize) {
    let count = size / 8;
    let data: Vec<u64> = vec![42; count];
    
    // Read bandwidth
    let iterations = 100;
    let start = Instant::now();
    let mut sum = 0u64;
    
    for _ in 0..iterations {
        for &val in &data {
            sum = sum.wrapping_add(val);
        }
    }
    
    let elapsed = start.elapsed();
    std::hint::black_box(sum);
    
    let bytes_read = size * iterations;
    let bandwidth_gbps = (bytes_read as f64 / elapsed.as_secs_f64()) / 1e9;
    
    println!("│ {:<8} Read:  {:>6.2} GB/s", name, bandwidth_gbps);
    
    // Write bandwidth
    let mut data_mut: Vec<u64> = vec![0; count];
    let start = Instant::now();
    
    for _ in 0..iterations {
        for val in &mut data_mut {
            *val = 42;
        }
    }
    
    let elapsed = start.elapsed();
    let bytes_written = size * iterations;
    let bandwidth_gbps = (bytes_written as f64 / elapsed.as_secs_f64()) / 1e9;
    
    println!("│ {:<8} Write: {:>6.2} GB/s", name, bandwidth_gbps);
}

/// Measure random access latency (cache miss heavy)
fn measure_random_latency() {
    println!("┌─ Random Access Latency (Cache Misses) ───────────────");
    
    // Working set much larger than L3 (16 MB) to force DRAM access
    let size = 256 * MB;
    let count = size / 8;
    let mut data: Vec<u64> = vec![0; count];
    
    // Initialize with random jumps
    for i in 0..count {
        data[i] = ((i * 2654435761) % count) as u64;  // Knuth hash
    }
    
    // Measure pointer-chase latency
    let iterations = 100_000;
    let start = Instant::now();
    let mut idx = 0usize;
    
    for _ in 0..iterations {
        idx = data[idx] as usize;
    }
    
    let elapsed = start.elapsed();
    std::hint::black_box(idx);
    
    let avg_latency_ns = elapsed.as_nanos() as f64 / iterations as f64;
    
    println!("│ Working set: {} MB", size / MB);
    println!("│ Iterations: {}", iterations);
    println!("│ Avg latency: {:.1} ns/access", avg_latency_ns);
    println!("│ Expected: ~80-100 ns (DDR5-5600 typical)");
    println!("└──────────────────────────────────────────────────────\n");
}

/// Measure scatter/gather performance (sparse vTPU pattern)
fn measure_scatter_gather() {
    println!("┌─ Scatter/Gather Performance (Sparse Access) ─────────");
    
    // Simulate phext coordinate-based sparse access
    let num_coords = 10_000;
    let coords: Vec<PhextCoord> = (0..num_coords)
        .map(|i| {
            // Generate sparse coordinates (varied locality)
            let d0 = (i % 100) as u16;
            let d1 = ((i / 100) % 80) as u16;
            let d2 = ((i / 8000) % 10) as u16;
            PhextCoord::new([d0, d1, d2, 1, 1, 1, 1, 1, 1, 1, 1])
        })
        .collect();
    
    // Simulate memory backing (256 MB sparse array)
    let memory_size = 256 * MB / 8;
    let memory: Vec<u64> = vec![0; memory_size];
    
    // Gather operation: read from sparse coordinates
    let start = Instant::now();
    let mut sum = 0u64;
    
    for coord in &coords {
        // Hash coordinate to memory address (simplified)
        let hash = coord.hash() as usize % memory_size;
        sum = sum.wrapping_add(memory[hash]);
    }
    
    let gather_time = start.elapsed();
    std::hint::black_box(sum);
    
    let gather_ns = gather_time.as_nanos() as f64 / num_coords as f64;
    let gather_gbps = (num_coords * 8) as f64 / gather_time.as_secs_f64() / 1e9;
    
    println!("│ Coordinates: {}", num_coords);
    println!("│ Gather: {:.1} ns/coord  ({:.2} GB/s)", gather_ns, gather_gbps);
    
    // Scatter operation: write to sparse coordinates
    let mut memory_mut: Vec<u64> = vec![0; memory_size];
    let start = Instant::now();
    
    for coord in &coords {
        let hash = coord.hash() as usize % memory_size;
        memory_mut[hash] = 42;
    }
    
    let scatter_time = start.elapsed();
    let scatter_ns = scatter_time.as_nanos() as f64 / num_coords as f64;
    let scatter_gbps = (num_coords * 8) as f64 / scatter_time.as_secs_f64() / 1e9;
    
    println!("│ Scatter: {:.1} ns/coord  ({:.2} GB/s)", scatter_ns, scatter_gbps);
    println!("│");
    println!("│ Comparison to sequential:");
    println!("│   Sequential read: ~50-70 GB/s (DDR5-5600 theoretical: 44.8 GB/s)");
    println!("│   Sparse gather:   {:.2} GB/s ({:.1}× slowdown)", gather_gbps, 50.0 / gather_gbps);
    println!("└──────────────────────────────────────────────────────\n");
}

/// Measure how working set size impacts bandwidth (cache → DRAM transition)
fn measure_working_set_scaling() {
    println!("┌─ Working Set Scaling (Cache → DRAM) ─────────────────");
    
    let sizes = [
        ("16 KB", 16 * KB),      // L1
        ("256 KB", 256 * KB),    // L2
        ("8 MB", 8 * MB),        // L3
        ("32 MB", 32 * MB),      // L3 boundary
        ("128 MB", 128 * MB),    // DDR5
        ("512 MB", 512 * MB),    // DDR5
    ];
    
    for (name, size) in &sizes {
        let bandwidth = measure_working_set_bandwidth(*size);
        println!("│ {:<8}: {:>6.2} GB/s", name, bandwidth);
    }
    
    println!("│");
    println!("│ Expected transitions:");
    println!("│   L1 (32 KB):  ~200 GB/s");
    println!("│   L2 (512 KB): ~100 GB/s");
    println!("│   L3 (16 MB):  ~50 GB/s");
    println!("│   DDR5:        ~30-40 GB/s (actual bandwidth)");
    println!("└──────────────────────────────────────────────────────\n");
}

fn measure_working_set_bandwidth(size: usize) -> f64 {
    let count = size / 8;
    let data: Vec<u64> = vec![42; count];
    
    // Sequential read
    let iterations = 1000;
    let start = Instant::now();
    let mut sum = 0u64;
    
    for _ in 0..iterations {
        for &val in &data {
            sum = sum.wrapping_add(val);
        }
    }
    
    let elapsed = start.elapsed();
    std::hint::black_box(sum);
    
    (size * iterations) as f64 / elapsed.as_secs_f64() / 1e9
}
