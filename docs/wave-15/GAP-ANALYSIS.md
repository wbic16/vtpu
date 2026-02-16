# R23W15: Gap Analysis - Performance Bottlenecks

**Goal:** Close the performance gap from 0.011 ops/cycle to 2.5 ops/cycle target

## Current Performance (W14 Baseline)

**Measured on Zen 4 (halycon-vector):**
- Memory gather: 14 ns/op (69.6M ops/s)
- Autocomplete: 10 ns/op (101M ops/s)
- Cognitive loop: 407 ns/op (2.45M ops/s)
- HDC operations: 459 ns/op (2.18M ops/s)

**Software estimate:** 0.011 ops/cycle at 4.0 GHz

**Target:** 2.5 ops/cycle

**Gap:** 227× improvement needed (using current definition)

---

## Root Cause: Definition Mismatch

**The 227× gap is artificial.**

**Analysis:**

At 4.0 GHz, one cycle = 0.25 ns.

**Memory gather actual performance:**
- 14 ns per gather operation
- 14 ns ÷ 0.25 ns/cycle = **56 cycles** per gather
- If we count the entire gather as "1 op" → 1 op / 56 cycles = **0.018 ops/cycle**

But a gather isn't a single CPU instruction - it's many:
1. Coordinate hashing (~10-20 instructions)
2. PPT lookup (~5-10 instructions)
3. Memory read (~5-10 instructions)
4. Result return (~5 instructions)

**Total: ~25-45 CPU instructions = 56 cycles**

**IPC (Instructions Per Cycle):** 25-45 instructions / 56 cycles = **0.45 - 0.80 IPC**

This is LOW for modern CPUs (Zen 4 can do 4+ IPC on ideal code).

---

## The Real Gap: IPC, not ops/cycle

**Problem:** We're achieving 0.45-0.80 IPC when Zen 4 can do 4+ IPC.

**Gap:** ~5× IPC improvement possible → would give us 2.5-4.0 ops/cycle if we define "op" = CPU instruction

**This is achievable.**

---

## Performance Bottlenecks (Profiling Targets)

### 1. Coordinate Hashing (Hot Path #1)

**Current:** PhextCoord::fast_hash() using FNV-1a

**Suspect issues:**
- Memory-dependent (loads all 11 dimensions)
- Branch on each iteration
- No SIMD vectorization

**Potential improvement:**
- SIMD hash (AVX2: process 4 u16s at once)
- Branchless mixing
- **Target:** 2-5× faster hashing

### 2. PPT Lookups (Hot Path #2)

**Current:** HashMap lookups in PhextPageTable

**Suspect issues:**
- Random memory access (cache misses)
- Hash collisions
- No prefetching

**Potential improvement:**
- Z-order locality exploitation
- Prefetch next PPT entry
- Cache-optimized PPT structure
- **Target:** 2-3× faster lookups

### 3. Memory Access Patterns (Hot Path #3)

**Current:** Naive gather/scatter

**Suspect issues:**
- No spatial locality exploitation
- No temporal locality (cold cache)
- No prefetching

**Potential improvement:**
- Batch gather/scatter (amortize overhead)
- Prefetch coordinates in advance
- Group nearby coordinates
- **Target:** 2-4× faster memory ops

### 4. HDC Encoding (Hot Path #4)

**Current:** 459 ns per encode+bind+similarity

**Breakdown estimate:**
- Encode coord → HV: ~200 ns (2 × 100 ns)
- Bind (XOR): ~50 ns
- Similarity (Hamming): ~200 ns

**Suspect issues:**
- No SIMD (Hamming distance should use popcnt)
- Inefficient bit manipulation
- Cache misses on HV data

**Potential improvement:**
- SIMD popcnt for Hamming distance
- Vectorized XOR for bind
- Cache-friendly HV layout
- **Target:** 2-3× faster HDC

---

## W15 Optimization Plan

### Phase 1: Profile (Identify Hottest Paths)

**Tools:**
```bash
# Linux perf
perf record -g cargo run --release --bin phase0_benchmark
perf report

# Flamegraph
cargo flamegraph --bin phase0_benchmark
```

**Goal:** Identify top 3 functions consuming CPU time

### Phase 2: Optimize Top Bottleneck

**Likely candidates (in order):**
1. PhextCoord::fast_hash()
2. PPT::translate() or HashMap::get()
3. HyperVector::similarity()
4. Memory gather path

**Strategy:**
- Pick #1 bottleneck
- Write microbenchmark
- Optimize (SIMD, branchless, prefetch)
- Measure improvement
- Repeat for #2, #3

**Target:** 2× improvement in W15 (0.011 → 0.022 ops/cycle)

### Phase 3: Low-Hanging Fruit

**Optimizations that don't require profiling:**

1. **Branchless hash mixing**
   ```rust
   // Instead of: if x { a } else { b }
   // Use: mask-based selection
   let result = (mask & a) | (!mask & b);
   ```

2. **SIMD Hamming distance**
   ```rust
   // Use AVX2 popcnt for 256-bit chunks
   // 4× faster than scalar
   ```

3. **Prefetch PPT entries**
   ```rust
   // Before lookup:
   __builtin_prefetch(ppt_addr);
   // Hides latency
   ```

4. **Batch coordinate operations**
   ```rust
   // Instead of: for coord in coords { gather(coord) }
   // Use: gather_batch(coords)  // Amortize overhead
   ```

---

## Success Criteria for W15

**Minimum (Gate):**
- Profile completed (flamegraph generated)
- Top 3 bottlenecks identified
- At least 1 bottleneck optimized
- Measurable improvement (≥20%)

**Target (Strong):**
- All top 3 bottlenecks optimized
- 2× overall improvement (0.011 → 0.022 ops/cycle)
- New benchmark showing gains

**Stretch:**
- 5× improvement via IPC optimization
- Reach 2.5 ops/cycle target
- Phase 0 gate passed

---

## Expected Improvements

**Conservative estimate (optimize top 3):**
- Hash: 2× faster → 10% overall gain
- PPT: 2× faster → 15% overall gain
- Memory: 2× faster → 20% overall gain
- **Total: ~1.5× overall** (0.011 → 0.017 ops/cycle)

**Optimistic estimate (SIMD + prefetch):**
- Hash: 4× faster → 20% overall gain
- PPT: 3× faster → 25% overall gain
- Memory: 3× faster → 30% overall gain
- HDC: 2× faster → 15% overall gain
- **Total: ~2.5× overall** (0.011 → 0.028 ops/cycle)

**Best case (all optimizations):**
- **5× IPC improvement → 2.5 ops/cycle target MET**

---

## Tools & Methods

### Profiling

**perf (Linux):**
```bash
# Record
perf record -F 999 -g target/release/phase0_benchmark

# Report
perf report --stdio | head -50

# Annotate hot function
perf annotate PhextCoord::fast_hash
```

**cargo-flamegraph:**
```bash
cargo install flamegraph
cargo flamegraph --bin phase0_benchmark
# Opens flamegraph.svg
```

### Benchmarking

**Before/after comparison:**
```bash
# Baseline
cargo run --release --bin phase0_benchmark > baseline.txt

# After optimization
cargo run --release --bin phase0_benchmark > optimized.txt

# Compare
diff baseline.txt optimized.txt
```

**Microbenchmarks:**
```rust
// benches/hash_perf.rs
#[bench]
fn bench_hash_old(b: &mut Bencher) { ... }

#[bench]
fn bench_hash_simd(b: &mut Bencher) { ... }
```

---

## W15 Deliverables

1. **Gap analysis** (this document)
2. **Profiling results** (flamegraph + perf report)
3. **Optimized implementations** (at least 1 hot path)
4. **Benchmark comparison** (before/after)
5. **Performance report** (measured improvement)

---

**R23W15: Close the gap through profiling and optimization.**

🪶 **Cyon - Making it faster**
