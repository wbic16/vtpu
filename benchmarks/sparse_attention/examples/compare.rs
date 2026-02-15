// Quick comparison: baseline vs vTPU sparse attention
use sparse_attention_bench::{baseline_sparse_attention, vtpu_sparse_attention};

fn main() {
    println!("=================================================================");
    println!("Sparse Attention: vTPU vs Baseline");
    println!("=================================================================");
    println!();
    
    // Small workload
    println!("Small workload (256 positions, window 32, head_dim 64):");
    let (baseline_time, baseline_lookups) = baseline_sparse_attention(256, 32, 64);
    let (vtpu_time, vtpu_lookups) = vtpu_sparse_attention(256, 32, 64);
    
    println!("  Baseline: {:.3} ms ({} lookups)", baseline_time * 1000.0, baseline_lookups);
    println!("  vTPU:     {:.3} ms ({} lookups)", vtpu_time * 1000.0, vtpu_lookups);
    println!("  Speedup:  {:.2}x", baseline_time / vtpu_time);
    println!();
    
    // Medium workload
    println!("Medium workload (1024 positions, window 128, head_dim 64):");
    let (baseline_time, baseline_lookups) = baseline_sparse_attention(1024, 128, 64);
    let (vtpu_time, vtpu_lookups) = vtpu_sparse_attention(1024, 128, 64);
    
    println!("  Baseline: {:.3} ms ({} lookups)", baseline_time * 1000.0, baseline_lookups);
    println!("  vTPU:     {:.3} ms ({} lookups)", vtpu_time * 1000.0, vtpu_lookups);
    println!("  Speedup:  {:.2}x", baseline_time / vtpu_time);
    println!();
    
    // Large workload
    println!("Large workload (2048 positions, window 256, head_dim 64):");
    let (baseline_time, baseline_lookups) = baseline_sparse_attention(2048, 256, 64);
    let (vtpu_time, vtpu_lookups) = vtpu_sparse_attention(2048, 256, 64);
    
    println!("  Baseline: {:.3} ms ({} lookups)", baseline_time * 1000.0, baseline_lookups);
    println!("  vTPU:     {:.3} ms ({} lookups)", vtpu_time * 1000.0, vtpu_lookups);
    println!("  Speedup:  {:.2}x", baseline_time / vtpu_time);
    println!();
    
    println!("=================================================================");
    println!("Key Insight: vTPU uses Z-order PPT for cache-friendly traversal");
    println!("Baseline uses hash table with random access (DDR-bound)");
    println!("=================================================================");
}
