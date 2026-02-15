// Criterion benchmark for sparse attention
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sparse_attention_bench::{baseline_sparse_attention, vtpu_sparse_attention};

fn bench_sparse_attention(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse_attention");
    
    // Small workload (fast iteration)
    group.bench_function("baseline_256_32", |b| {
        b.iter(|| {
            black_box(baseline_sparse_attention(256, 32, 64))
        });
    });
    
    group.bench_function("vtpu_256_32", |b| {
        b.iter(|| {
            black_box(vtpu_sparse_attention(256, 32, 64))
        });
    });
    
    // Medium workload (realistic)
    group.bench_function("baseline_1024_128", |b| {
        b.iter(|| {
            black_box(baseline_sparse_attention(1024, 128, 64))
        });
    });
    
    group.bench_function("vtpu_1024_128", |b| {
        b.iter(|| {
            black_box(vtpu_sparse_attention(1024, 128, 64))
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_sparse_attention);
criterion_main!(benches);
