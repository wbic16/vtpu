//! PPT Benchmark: measure PTC hit rate under realistic workloads
//! Target KPI: PPT-95 (≥95% hit rate on structured workloads)

use vtpu_runtime::{PhextPageTable, PhextCoord, MemoryTier};
use std::time::Instant;

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("  PPT Benchmark — R23W3 KPI: PPT-95 (≥95% hit rate)");
    println!("═══════════════════════════════════════════════════════\n");

    bench_dim0_sweep();
    bench_dim_walk_3d();
    bench_random_access();
    bench_cognitive_pattern();
    bench_tier_classification();
}

/// Workload 1: Sweep dim 0 repeatedly (embedding table scan)
fn bench_dim0_sweep() {
    let mut ppt = PhextPageTable::new();
    let sweeps = 100;
    let width = 128;

    let start = Instant::now();
    for _ in 0..sweeps {
        for d0 in 0..width {
            let coord = PhextCoord::new([d0, 5, 5, 1, 1, 1, 1, 1, 1, 1, 1]);
            ppt.translate(&coord);
        }
    }
    let elapsed = start.elapsed();
    let stats = ppt.stats();
    let total = sweeps * width;

    println!("┌─ Dim-0 Sweep (embedding scan) ─────────────");
    println!("│ Sweeps: {}, Width: {}, Total: {}", sweeps, width, total);
    println!("│ PTC hits: {}, misses: {}", stats.ptc_hits, stats.ptc_misses);
    println!("│ Hit rate: {:.1}%  {}", stats.ptc_hit_rate * 100.0,
        if stats.ptc_hit_rate >= 0.95 { "✅" } else { "❌" });
    println!("│ Time: {:.2} ms ({:.0} ns/lookup)", elapsed.as_secs_f64() * 1e3,
        elapsed.as_nanos() as f64 / total as f64);
    println!("│ Regions: {}", stats.regions_allocated);
    println!("└─────────────────────────────────────────────\n");
}

/// Workload 2: Walk through inner 3 dims (knowledge traversal)
fn bench_dim_walk_3d() {
    // Use 512 sets × 4-way = 2048 entries for 3D walk (1000 unique coords)
    let mut ppt = PhextPageTable::with_config(1024, 1 << 21);
    let passes = 20;
    let side: u64 = 10; // 10^3 = 1000 unique coords per pass (fits PTC's 1024 entries)
    let total = passes * side * side * side;

    let start = Instant::now();
    for _ in 0..passes {
        for d0 in 0..side as u16 {
            for d1 in 0..side as u16 {
                for d2 in 0..side as u16 {
                    let coord = PhextCoord::new([d0, d1, d2, 2, 2, 2, 2, 2, 2, 2, 2]);
                    ppt.translate(&coord);
                }
            }
        }
    }
    let elapsed = start.elapsed();
    let stats = ppt.stats();

    println!("┌─ 3D Inner Walk (knowledge traversal) ──────");
    println!("│ Passes: {}, Side: {}, Total: {}", passes, side, total);
    println!("│ PTC hits: {}, misses: {}", stats.ptc_hits, stats.ptc_misses);
    println!("│ Hit rate: {:.1}%  {}", stats.ptc_hit_rate * 100.0,
        if stats.ptc_hit_rate >= 0.95 { "✅" } else { "❌" });
    println!("│ Time: {:.1} ms ({:.0} ns/lookup)", elapsed.as_secs_f64() * 1e3,
        elapsed.as_nanos() as f64 / total as f64);
    println!("│ Regions: {}", stats.regions_allocated);
    println!("└─────────────────────────────────────────────\n");
}

/// Workload 3: Random access (worst case — PTC thrashing)
fn bench_random_access() {
    let mut ppt = PhextPageTable::new();
    let n = 100_000;
    // LCG for deterministic "random" coords
    let mut rng: u64 = 0xDEADBEEF;

    let start = Instant::now();
    for _ in 0..n {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let d0 = ((rng >> 16) & 0x7FF) as u16;
        let d1 = ((rng >> 27) & 0x7FF) as u16;
        let d2 = ((rng >> 38) & 0x7FF) as u16;
        let d3 = ((rng >> 5) & 0xFF) as u16;
        let coord = PhextCoord::new([d0, d1, d2, d3, 1, 1, 1, 1, 1, 1, 1]);
        ppt.translate(&coord);
    }
    let elapsed = start.elapsed();
    let stats = ppt.stats();

    println!("┌─ Random Access (worst case) ────────────────");
    println!("│ Lookups: {}", n);
    println!("│ PTC hits: {}, misses: {}", stats.ptc_hits, stats.ptc_misses);
    println!("│ Hit rate: {:.1}% (expected low)", stats.ptc_hit_rate * 100.0);
    println!("│ Time: {:.1} ms ({:.0} ns/lookup)", elapsed.as_secs_f64() * 1e3,
        elapsed.as_nanos() as f64 / n as f64);
    println!("│ Regions: {}", stats.regions_allocated);
    println!("└─────────────────────────────────────────────\n");
}

/// Workload 4: Cognitive pattern (mixed: sequential + associative + scatter)
/// Simulates a sentron doing knowledge retrieval + writing results
fn bench_cognitive_pattern() {
    let mut ppt = PhextPageTable::new();
    let iterations = 50_000;
    let total = iterations * 6; // 6 lookups per iteration

    let start = Instant::now();
    for i in 0..iterations as u16 {
        let base_d0 = i & 0x3F; // cycle through 64 positions

        // 1. Gather from embedding table (sequential dim-0)
        ppt.translate(&PhextCoord::new([base_d0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // 2. Gather from adjacent (dim-0 + 1)
        ppt.translate(&PhextCoord::new([base_d0 + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // 3. Associative lookup (different dim-1)
        ppt.translate(&PhextCoord::new([base_d0, 7, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // 4. Cross-section (different dim-2)
        ppt.translate(&PhextCoord::new([base_d0, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1]));
        // 5. Scatter result (back to base)
        ppt.translate(&PhextCoord::new([base_d0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // 6. Scatter to output region (different outer dim)
        ppt.translate(&PhextCoord::new([0, 0, 0, 2, 1, 1, 1, 1, 1, 1, 1]));
    }
    let elapsed = start.elapsed();
    let stats = ppt.stats();

    println!("┌─ Cognitive Pattern (retrieval + scatter) ───");
    println!("│ Iterations: {}, Lookups: {}", iterations, total);
    println!("│ PTC hits: {}, misses: {}", stats.ptc_hits, stats.ptc_misses);
    println!("│ Hit rate: {:.1}%  {}", stats.ptc_hit_rate * 100.0,
        if stats.ptc_hit_rate >= 0.95 { "✅" } else { "❌" });
    println!("│ Time: {:.1} ms ({:.0} ns/lookup)", elapsed.as_secs_f64() * 1e3,
        elapsed.as_nanos() as f64 / total as f64);
    println!("│ Regions: {}", stats.regions_allocated);
    println!("└─────────────────────────────────────────────\n");
}

/// Workload 5: Tier classification accuracy
fn bench_tier_classification() {
    let ppt = PhextPageTable::new();
    let base = PhextCoord::new([10, 20, 30, 4, 5, 6, 7, 8, 9, 10, 11]);

    let cases = [
        ([10, 20, 31, 4, 5, 6, 7, 8, 9, 10, 11], MemoryTier::L1Scratchpad, "dim2 ±1"),
        ([10, 20, 30, 4, 6, 6, 7, 8, 9, 10, 11], MemoryTier::L2Local, "dim4 changed"),
        ([10, 20, 30, 4, 5, 6, 99, 8, 9, 10, 11], MemoryTier::L3Shared, "dim6 changed"),
        ([10, 20, 30, 4, 5, 6, 7, 8, 99, 10, 11], MemoryTier::NodeLocal, "dim8 changed"),
        ([10, 20, 30, 4, 5, 6, 7, 8, 9, 99, 11], MemoryTier::NodeLocal, "dim9 changed"),
        ([10, 20, 30, 4, 5, 6, 7, 8, 9, 10, 99], MemoryTier::Remote, "dim10 changed"),
    ];

    println!("┌─ Tier Classification ──────────────────────");
    let mut pass = 0;
    for (dims, expected, label) in &cases {
        let coord = PhextCoord::new(*dims);
        let tier = ppt.classify_tier(&coord, &base);
        let ok = tier == *expected;
        if ok { pass += 1; }
        println!("│  {} {:20} → {:?}", if ok { "✅" } else { "❌" }, label, tier);
    }
    println!("│ {}/{} passed", pass, cases.len());
    println!("└─────────────────────────────────────────────\n");
}
