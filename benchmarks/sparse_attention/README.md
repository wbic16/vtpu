# Sparse Attention Benchmark - R23W6

## Goal

Prove 10× speedup claim: vTPU coordinate-native sparse attention vs baseline scatter/gather.

## Setup

**Workload:** GPT-style sparse attention over 1024 tokens
- Query: 1024 positions
- Keys: 1024 positions  
- Attention pattern: Sliding window (attend to previous 128 tokens)
- Head dimension: 64
- Total ops: ~8.4M FLOPs (simplified)

## Comparison

### Baseline: Hash Table Lookup
```
for each query position q:
    for each key position k in window:
        coord_hash = hash(layer, head, q, k)
        value = hash_table[coord_hash]  // Random access, DDR-bound
        accumulate attention score
```

**Expected:** ~2 GB/s (DDR5 random access, 27× penalty vs sequential)

### vTPU: Phext Coordinate Range
```
for each query position q:
    coord_range = (layer.head.q / 0.0.k / 0.0.0)  // k in [q-128, q]
    values = CRANGE(coord_range)  // Sequential scan, cache-friendly
    accumulate attention scores
```

**Expected:** ~50-140 GB/s (L1-L3 cache hits via Z-order PPT)

## Running

```bash
cd /source/vtpu
cargo bench --bench sparse_attention
```

## Metrics

- Baseline latency (ms)
- vTPU latency (ms)
- Speedup (baseline / vTPU)
- Memory bandwidth utilization
- Cache hit rate (perf counters)

## Success Criteria

✅ **10× speedup** (or better)  
✅ **95% L1 hit rate** (with Z-order PPT)  
✅ **Reproducible** (multiple runs, consistent results)
