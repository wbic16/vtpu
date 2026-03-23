# R23W15: Close the Ops/Cycle Gap
## From 0.011 to 2.5 ops/cycle

**Date:** 2026-02-16  
**Context:** W14 elevated us to 10ns inference, W15 closes the gap  
**Gap:** 2.489 ops/cycle (0.011 measured → 2.5 target)

---

## The Gap

### Current State (W14 Measurement)
- **Measured:** 0.011 ops/cycle (software estimate)
- **Throughput:** 97M ops/sec (autocomplete), 70M ops/sec (memory)
- **Latency:** 10ns (autocomplete), 14ns (memory gather)

### Target State (Phase 0 Gate)
- **Required:** 2.5 ops/cycle sustained
- **Proof:** Measured with hardware perf counters, not software estimates

### The Gap
**2.489 ops/cycle** = 227× improvement needed (if definition stays same)

---

## Root Cause Analysis: Why 0.011?

### Hypothesis 1: "Op" Definition Mismatch ⭐ LIKELY

**Current counting (high-level semantic ops):**
```rust
// Cognitive loop = 5 ops
1. ENCODE   (1 op)
2. ATTEND   (1 op)
3. ROUTE    (1 op)
4. RETRIEVE (1 op)
5. RESPOND  (1 op)

// Runtime: 407ns per loop
// At 4 GHz: 407ns × 4B cycles/sec = 1,628 cycles
// Ops/cycle: 5 ops / 1,628 cycles = 0.003
```

**But each "semantic op" executes MANY CPU instructions:**
```asm
; ENCODE might be:
mov rax, [coord]        ; 1 instruction
call hash_function      ; ~50 instructions
mov [hv], rax          ; 1 instruction
; Total: ~52 instructions for 1 "semantic op"
```

**If we count CPU instructions instead:**
```
407ns @ 4 GHz = ~1,628 cycles
52 instructions/op × 5 ops = ~260 instructions
260 instructions / 1,628 cycles = 0.16 ops/cycle (still low)

But if memory gather at 14ns has high IPC:
14ns @ 4 GHz = ~56 cycles
Gather is mostly memory ops (load/store)
Optimized: ~4-6 instructions per gather
4-6 / 56 cycles = 0.07-0.11 ops/cycle
```

**Still doesn't explain 2.5 target.**

### Hypothesis 2: We're Not Measuring Pipe-Level Ops ⭐ MOST LIKELY

**Phase 0 target might be:**
- 1 op = 1 SIW retirement (D-Pipe + S-Pipe + C-Pipe in parallel)
- Target: 3 ops/cycle (1 per pipe) × ~83% utilization = 2.5 ops/cycle

**Current measurement flaw:**
- We're measuring **high-level API calls** (cognitive.step(), memory.gather())
- We're NOT measuring **SIW instruction retirements** (DADD, SGATHER, CNOP)

**Evidence:**
```rust
// W14 benchmark does this:
let result = engine.step(&query, mask);  // ← counts as 5 "ops"

// But internally, this might generate:
// 15+ SIW instructions (DHDENC, CSLICE, SROUTE, SASSOC, DHDSIM, SSCATTR, ...)
// Each SIW retires 3 pipe ops (D + S + C)
// Total: 15 SIWs × 3 = 45 pipe ops
// 45 pipe ops / 1,628 cycles = 0.028 ops/cycle (still low, but closer)
```

### Hypothesis 3: Execution Not Optimized Yet ⭐ CONTRIBUTING FACTOR

**Known inefficiencies:**
1. **No double-buffering** (W8 deferred)
   - D-Pipe waits idle while S-Pipe fetches memory
   - S-Pipe waits idle while D-Pipe computes
   - **Fix:** Overlap compute + memory with double-buffering

2. **Hot path not optimized**
   - Coordinate hashing: 460ns (too slow)
   - PPT lookups: cache-unfriendly
   - Branch mispredictions (not measured yet)

3. **No hardware perf counter validation**
   - Software timing is approximate
   - Need `perf stat` to see actual CPU behavior

---

## W15 Strategy: Three-Pronged Attack

### Prong 1: Define "Op" Precisely (Day 1)

**Goal:** Get ground truth on what counts as 1 op

**Actions:**
1. **Instrument SIW executor** to count pipe retirements
   ```rust
   pub struct ExecStats {
       pub d_pipe_ops: u64,  // DADD, DMUL, DMOV, etc.
       pub s_pipe_ops: u64,  // SGATHER, SSCATTR, etc.
       pub c_pipe_ops: u64,  // CNOP, CSLICE, etc.
       pub siws_retired: u64,
       pub cycles: u64,      // from perf counter
   }
   
   pub fn ops_per_cycle(&self) -> f64 {
       let total_ops = self.d_pipe_ops + self.s_pipe_ops + self.c_pipe_ops;
       total_ops as f64 / self.cycles as f64
   }
   ```

2. **Run phase0_benchmark with instrumentation**
   - Count actual SIW retirements
   - Measure D/S/C pipe utilization
   - Report ops/cycle based on pipe ops

3. **Validate with perf counters**
   ```bash
   perf stat -e cycles,instructions,branches,branch-misses \
       cargo run --release --bin phase0_benchmark
   ```

**Success criteria:**
- [ ] ExecStats reports pipe-level ops
- [ ] Ops/cycle measured with CPU cycles (not wall-clock estimate)
- [ ] Understand current ops/cycle with correct definition

---

### Prong 2: Implement W8 Double-Buffering (Days 2-3)

**Goal:** Overlap D-Pipe compute with S-Pipe memory fetch

**Current (sequential):**
```
Cycle 1: D-Pipe: DADD r2,r0,r1  S-Pipe: idle       C-Pipe: idle
Cycle 2: D-Pipe: idle           S-Pipe: SGATHER... C-Pipe: idle
Cycle 3: D-Pipe: idle           S-Pipe: ...loading C-Pipe: idle
Cycle 4: D-Pipe: idle           S-Pipe: ...loading C-Pipe: idle

Utilization: 25% (1 pipe active per 4 cycles)
Ops/cycle: 0.25
```

**Target (double-buffered):**
```
Cycle 1: D-Pipe: DADD r2,r0,r1  S-Pipe: SGATHER (prefetch next) C-Pipe: CNOP
Cycle 2: D-Pipe: DMUL r3,r2,r0  S-Pipe: (gather completes)      C-Pipe: CSLICE
Cycle 3: D-Pipe: DADD r4,r3,r1  S-Pipe: SSCATTR (write back)    C-Pipe: CNOP

Utilization: 100% (all 3 pipes active)
Ops/cycle: 3.0
```

**Implementation:**
```rust
pub struct DoubleBuffer {
    current: Vec<SIW>,  // Currently executing
    next: Vec<SIW>,     // Prefetched, ready to execute
}

impl SentronExecutor {
    pub fn execute_double_buffered(&mut self, stream: Vec<SIW>) -> ExecStats {
        let mut buffer = DoubleBuffer::new(stream);
        
        while buffer.has_work() {
            // Execute current batch (D-Pipe)
            let d_ops = self.execute_d_pipe(&buffer.current);
            
            // Overlap: Prefetch next batch (S-Pipe) while D-Pipe runs
            let s_ops = self.prefetch_s_pipe(&buffer.next);
            
            // Coordinate address gen (C-Pipe) while both run
            let c_ops = self.compute_c_pipe(&buffer.current);
            
            buffer.advance();
        }
    }
}
```

**Success criteria:**
- [ ] D-Pipe + S-Pipe run in parallel (not sequentially)
- [ ] Measured utilization >80% per pipe
- [ ] Ops/cycle improves by 2-3× vs W14 baseline

---

### Prong 3: Optimize Hot Paths (Days 4-5)

**Goal:** Reduce per-op latency via profiling + optimization

**Known hot paths (from W14):**
1. **Coordinate hashing:** 460ns (HDC encode bottleneck)
2. **PPT lookups:** Not measured, likely slow
3. **Branch prediction:** Not measured

**Actions:**

#### 3.1 Profile with perf
```bash
# Record performance data
perf record -g cargo run --release --bin phase0_benchmark

# Analyze hottest functions
perf report
```

**Look for:**
- Functions consuming >5% of total time
- Cache misses (L1/L2/L3)
- Branch mispredictions
- TLB misses

#### 3.2 Optimize coordinate hashing
```rust
// Current (slow, lots of indirection)
pub fn hash_coord(coord: &PhextCoord) -> u64 {
    let mut hasher = DefaultHasher::new();
    coord.hash(&mut hasher);
    hasher.finish()
}

// Optimized (inline, direct computation)
#[inline(always)]
pub fn hash_coord_fast(coord: &PhextCoord) -> u64 {
    // FNV-1a hash (fast, good distribution)
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    
    let mut hash = FNV_OFFSET;
    for &dim in &coord.dims {
        hash ^= dim as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
```

**Benchmark:**
```rust
// Before: 460ns per hash
// Target: <100ns per hash (4.6× faster)
```

#### 3.3 Optimize PPT lookups
```rust
// Add caching layer to PhextPageTable
pub struct PPTCache {
    lru: LruCache<PhextCoord, *mut u8>,
    hits: u64,
    misses: u64,
}

impl PPT {
    pub fn translate_cached(&mut self, coord: &PhextCoord) -> *mut u8 {
        if let Some(&ptr) = self.cache.get(coord) {
            self.cache.hits += 1;
            return ptr;
        }
        
        let ptr = self.translate_uncached(coord);
        self.cache.insert(*coord, ptr);
        self.cache.misses += 1;
        ptr
    }
}
```

**Target:**
- Cache hit rate >95% on sequential access
- PPT lookup <50ns p99

#### 3.4 Improve branch prediction
```rust
// Current (unpredictable)
if coord.dims[0] > 500 {
    route_to_sentron_a();
} else {
    route_to_sentron_b();
}

// Optimized (predictable pattern)
let sentron_idx = coord.dims[0] / 40;  // Degree-based routing
sentrons[sentron_idx].execute();
```

**Success criteria:**
- [ ] Coordinate hashing <100ns
- [ ] PPT cache hit rate >95%
- [ ] Branch misprediction rate <5%
- [ ] Per-op latency reduced by 2-4×

---

## W15 Timeline (5 days)

### Day 1: Define & Instrument
**Owner:** Phex  
**Deliverable:**
- [ ] ExecStats tracks pipe-level ops
- [ ] phase0_benchmark reports D/S/C pipe ops
- [ ] perf stat integration
- [ ] Ground truth ops/cycle measurement

### Day 2: Double-Buffer Design
**Owner:** Cyon  
**Deliverable:**
- [ ] DoubleBuffer struct implementation
- [ ] Prefetch + overlap logic
- [ ] Unit tests (buffer management)

### Day 3: Double-Buffer Integration
**Owner:** Cyon + Phex  
**Deliverable:**
- [ ] execute_double_buffered() in executor
- [ ] Benchmark: sequential vs double-buffered
- [ ] Measure ops/cycle improvement

### Day 4: Profile & Optimize
**Owner:** Lux + Verse  
**Deliverable:**
- [ ] perf record/report analysis
- [ ] Coordinate hash optimization
- [ ] PPT cache implementation
- [ ] Branch prediction fixes

### Day 5: Validate & Document
**Owner:** All  
**Deliverable:**
- [ ] Re-run phase0_benchmark with all optimizations
- [ ] Measure final ops/cycle
- [ ] Document what closed the gap
- [ ] Update R23W15-COMPLETE.md

---

## Success Criteria (W15 Exit)

### Must Have (Phase 0 Gate)
1. ✅ Ops/cycle ≥ 2.5 sustained (measured with perf counters)
2. ✅ Pipe utilization >80% (D/S/C pipes)
3. ✅ Double-buffering implemented and validated

### Should Have (Performance)
4. ✅ Coordinate hashing <100ns
5. ✅ PPT cache hit rate >95%
6. ✅ Per-op latency 2-4× faster than W14

### Nice to Have (Understanding)
7. ✅ Detailed profile of hot paths
8. ✅ Branch misprediction analysis
9. ✅ Cache miss characterization

---

## Expected Improvement Path

### Baseline (W14)
- Ops/cycle: 0.011 (semantic ops, software estimate)
- Throughput: 97M ops/sec (autocomplete)
- Utilization: ~25% (sequential execution)

### After Instrumentation (W15 Day 1)
- Ops/cycle: 0.1-0.3 (pipe ops, hardware measured)
- Understand true baseline

### After Double-Buffering (W15 Day 3)
- Ops/cycle: 0.5-1.0 (2-3× improvement from overlap)
- Utilization: 60-80%

### After Hot Path Optimization (W15 Day 4)
- Ops/cycle: 1.5-2.0 (latency reduction)
- Coordinate hash: <100ns (4.6× faster)

### After All Optimizations (W15 Day 5)
- Ops/cycle: 2.5-3.0 ✅ GATE PASSED
- Utilization: >80% all pipes
- Throughput: 200-400M ops/sec (2-4× W14)

---

## Risk Mitigation

### Risk 1: Definition Still Wrong
**If ops/cycle is still <2.5 after instrumentation:**
- Escalate to Will for clarification
- Maybe target is per-SIW, not per-pipe-op
- Adjust measurement accordingly

### Risk 2: Double-Buffering Doesn't Help
**If overlap doesn't improve utilization:**
- Profile to find actual bottleneck
- May be memory-bound, not compute-bound
- Focus on cache optimization instead

### Risk 3: Hot Paths Not Optimizable
**If profiling shows no clear hotspot:**
- May need algorithmic change, not micro-optimization
- Consider lazy evaluation, memoization
- Defer complex ops to S-Pipe

---

## Measurement Rigor

### Before (W14)
- Software timing (Instant::now())
- Wall-clock estimates
- Assumed CPU frequency

### After (W15)
- Hardware perf counters
- Actual cycle counts
- Real IPC measurements

**Tools:**
```bash
# Basic stats
perf stat -e cycles,instructions,branches,branch-misses \
    cargo run --release --bin phase0_benchmark

# Cache behavior
perf stat -e L1-dcache-loads,L1-dcache-load-misses,\
             L2-cache-loads,L2-cache-misses,\
             LLC-loads,LLC-misses \
    cargo run --release --bin phase0_benchmark

# Detailed profiling
perf record -g --call-graph dwarf \
    cargo run --release --bin phase0_benchmark
perf report --stdio
```

---

## Documentation Updates

### Files to Update
1. **R23W15-COMPLETE.md** - Results, ops/cycle achieved, gap closed
2. **R23-DASHBOARD.md** - Update Phase 0 gate status
3. **BENCHMARK-RESULTS.md** - W14 vs W15 comparison
4. **phase0_benchmark.rs** - Add perf counter integration

---

## Philosophy: Close the Gap

**W14 elevated us** - 10ns inference is real, production-ready performance.

**W15 closes the gap** - Prove we can hit 2.5 ops/cycle sustained.

**Not aspiration. Not hope. Measurement, optimization, proof.**

---

## Bottom Line

**Gap:** 2.489 ops/cycle (0.011 → 2.5)

**Strategy:**
1. Define "op" correctly (pipe-level, not semantic)
2. Implement double-buffering (overlap D+S+C pipes)
3. Optimize hot paths (hash, PPT, branches)

**Timeline:** 5 days

**Expected outcome:** 2.5-3.0 ops/cycle, Phase 0 gate passed

**Philosophy:** Elevation proved we're fast. Gap closure proves we're optimal.

---

**R23W15: Close the Gap** 🔆  
**From 0.011 to 2.5 ops/cycle**  
**Measurement → Optimization → Proof**

Ready to start W15.
