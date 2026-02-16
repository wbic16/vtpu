# R23W15 — Gap Analysis & Optimization Plan

**Wave:** R23W15  
**Date:** 2026-02-15  
**Type:** Performance Optimization  
**Focus:** Close the measured performance gap  
**Contributor:** Lumen ✴️

---

## The Gap (from W14)

**Measured Performance:**
- 0.011 ops/cycle (semantic operations definition)
- Cognitive loop: 407 ns/op
- HDC encode: 459 ns/op
- Memory gather: 14 ns/op
- Autocomplete: 10 ns/op

**Target Performance:**
- ≥2.5 ops/cycle (Phase 0 gate minimum)
- Previously claimed: 2.93 ops/cycle (W12)

**Gap:** 227× shortfall (0.011 → 2.5)

**Status:** Real measurements exist. Gap is concrete, not theoretical.

---

## Root Cause Analysis

### Issue 1: "Op" Definition Ambiguity

**Current counting (semantic ops):**
- 1 cognitive step = 5 ops (ENCODE + ATTEND + ROUTE + RETRIEVE + RESPOND)
- 1 HDC cycle = 4 ops
- Result: 0.011 ops/cycle

**Alternative counting (low-level ops):**
- 1 op = 1 vTPU pipe operation (DADD, SGATHER, CNOP, etc.)
- 1 op = 1 CPU instruction
- Result: Likely 3-4 ops/cycle (above target)

**W15 Action:** Define "op" precisely, re-measure with instrumentation.

---

### Issue 2: No Cycle-Accurate Measurement

**Current approach:**
- Wall-clock time via `Instant::now()`
- Clock frequency assumption (4.0 GHz)
- **Gap:** No actual cycle counts from hardware

**W14 Recommendation:**
```bash
perf stat -e cycles,instructions cargo run --release --bin phase0_benchmark
```

**W15 Action:** Integrate `perf.rs` for hardware performance counters.

---

### Issue 3: Execution Overhead Not Isolated

**Current benchmarks measure:**
- Cognitive loop API (high-level)
- HDC operations (abstracted)
- Memory API calls

**What's NOT measured:**
- Raw SIW execution time
- Pipe-level operation throughput
- Executor efficiency (packer → scheduler → exec)

**W15 Action:** Benchmark at SIW/pipe level, not just high-level APIs.

---

### Issue 4: Hot Paths Not Optimized

**From W14 profiling (inferred from timing):**

| Operation | Time | Bottleneck |
|-----------|------|------------|
| Coordinate hashing | ~460 ns | HDC encode path |
| PPT lookup | ~14 ns | Memory gather overhead |
| Cognitive loop overhead | ~407 ns | API abstraction layers |

**W15 Action:** Profile with `perf record`, optimize hot paths.

---

## W15 Optimization Strategy

### Phase 1: Measurement Precision (Days 1-2)

**Goal:** Get ground truth from hardware.

**Deliverables:**

1. **`src/bin/perf_benchmark.rs`**
   - Use `perf.rs` module (already exists in codebase)
   - Measure cycles, instructions, cache misses
   - Report actual IPC (instructions per cycle)

2. **`scripts/benchmark_with_perf.sh`**
   - Wrapper script for `perf stat`
   - Collects hardware counters
   - Outputs machine-readable results

3. **`docs/wave-15/MEASUREMENT-METHODOLOGY.md`**
   - Define "op" precisely (consensus with Will)
   - Document counting rules
   - Explain IPC vs ops/cycle relationship

**Success metric:** Know actual cycles consumed, not estimated.

---

### Phase 2: Low-Level Benchmarks (Days 3-4)

**Goal:** Measure SIW/pipe execution directly.

**Deliverables:**

1. **`benchmarks/siw_executor_bench.rs`**
   - Bypass high-level APIs
   - Measure raw SIW stream execution
   - Count pipe operations (D/S/C)
   - Report ops/cycle at executor level

2. **`benchmarks/pipe_operation_bench.rs`**
   - Individual pipe op benchmarks:
     - DADD, DMUL, DFMA (D-Pipe)
     - SGATHER, SSCATTR, SROUTE (S-Pipe)
     - CSLICE, CNOP (C-Pipe)
   - Measure latency + throughput per op

3. **`benchmarks/packer_efficiency_bench.rs`**
   - Measure packing overhead
   - Scalar ops → SIW packing time
   - Compare packed vs unpacked throughput

**Success metric:** Isolate executor performance from API overhead.

---

### Phase 3: Hot Path Optimization (Days 5-6)

**Goal:** Close the gap with real code improvements.

**Targets (from W14 profiling):**

1. **Coordinate Hashing (460 ns → target <100 ns)**
   - Current: Generic hash implementation
   - Optimize: Custom PhextCoord hash (Z-order aware)
   - Technique: Cache hash values, avoid recomputation

2. **PPT Lookups (14 ns → target <5 ns)**
   - Current: HashMap lookup
   - Optimize: Array-based PTC (Phext TLB Cache)
   - Technique: Direct indexing for hot coordinates

3. **Cognitive Loop Overhead (407 ns → target <150 ns)**
   - Current: Multiple API layers
   - Optimize: Inline hot paths, reduce allocations
   - Technique: Monomorphization, stack allocation

**Deliverables:**

1. **`src/phext_coord.rs` (optimized hash)**
   - Cached Z-order hash
   - Benchmark before/after

2. **`src/ppt.rs` (optimized PTC)**
   - Array-based cache (not HashMap)
   - Benchmark hit rate + latency

3. **`src/cognitive.rs` (inlined hot paths)**
   - Mark functions with `#[inline]`
   - Reduce Box/Arc allocations
   - Benchmark cognitive loop throughput

**Success metric:** Measurable improvement in ns/op benchmarks.

---

### Phase 4: Double-Buffer Pattern (Days 7-8)

**Goal:** Implement W8 optimization (overlap compute + memory).

**From W8 plan:**
> "With double-buffering (future wave), overlapped steps hit 2.8+ ops/cycle"

**Technique:**
- While D-Pipe computes on buffer A, S-Pipe fetches into buffer B
- Swap buffers each cycle
- Hide memory latency behind compute

**Deliverables:**

1. **`src/double_buffer.rs`**
   - DoubleBuffer<T> type
   - swap() method
   - Ping-pong allocation

2. **`src/exec.rs` (modified)**
   - Use double-buffer in executor
   - Overlap D-Pipe + S-Pipe operations
   - Benchmark with/without double-buffering

3. **`benchmarks/double_buffer_bench.rs`**
   - Measure overlap efficiency
   - Compare single-buffer vs double-buffer
   - Target: 1.5-2× improvement

**Success metric:** Approach 2.5+ ops/cycle with overlap.

---

## Success Criteria

### Minimum Viable (W15 Pass)

- [ ] "Op" defined precisely (documented consensus)
- [ ] Hardware perf counters integrated (cycles measured)
- [ ] Low-level benchmarks exist (SIW/pipe level)
- [ ] At least one hot path optimized (measurable speedup)
- [ ] Gap analysis updated with real profiling data

### Stretch Goals

- [ ] All three hot paths optimized (hash, PPT, cognitive)
- [ ] Double-buffer pattern implemented
- [ ] Measured ≥2.0 ops/cycle (80% of target)
- [ ] Clear path to 2.5+ ops/cycle documented

---

## Risk Mitigation

### If Definition Clarification Stalls

**Problem:** Can't agree on what "op" means  
**Fallback:** Report multiple metrics:
- Semantic ops/cycle (current 0.011)
- Pipe ops/cycle (vTPU instruction count)
- IPC (CPU instructions per cycle)
- Let data speak for itself

### If Optimizations Don't Close Gap

**Problem:** Still below 2.5 ops/cycle after fixes  
**Response:** Document honestly:
- What was tried
- What speedup was achieved
- What bottlenecks remain
- Proposed next steps (W16+)

**Philosophy:** Elevation = report truth, not desired outcome.

### If Perf Counters Unavailable

**Problem:** No `perf` access on WSL2 / limited hardware  
**Fallback:** 
- Manual instruction counting via disassembly
- Simulator-based cycle modeling
- Note limitation in results

---

## Timeline

**Day 1-2:** Measurement precision (perf counters, define "op")  
**Day 3-4:** Low-level benchmarks (SIW/pipe direct measurement)  
**Day 5-6:** Hot path optimization (hash, PPT, cognitive loop)  
**Day 7-8:** Double-buffer implementation + validation  

**Total:** ~8 days (Mirrorborn time) = ~2 weeks human time

**Target completion:** 2026-03-01

---

## Deliverable Structure

```
src/
  phext_coord.rs (optimized hash)
  ppt.rs (optimized PTC)
  cognitive.rs (inlined paths)
  double_buffer.rs (NEW)
  exec.rs (modified for double-buffer)
  bin/
    perf_benchmark.rs (NEW)
benchmarks/
  siw_executor_bench.rs (NEW)
  pipe_operation_bench.rs (NEW)
  packer_efficiency_bench.rs (NEW)
  double_buffer_bench.rs (NEW)
scripts/
  benchmark_with_perf.sh (NEW)
docs/wave-15/
  MEASUREMENT-METHODOLOGY.md (NEW)
  R23W15-GAP-ANALYSIS.md (this file)
  R23W15-OPTIMIZATION-RESULTS.md (after)
  R23W15-COMPLETE.md (after)
```

---

## Coordination

**Ownership (proposed):**

**Lumen (me):**
- Measurement precision (perf counters)
- Low-level benchmarks (SIW/pipe)
- Documentation (methodology, results)

**Phex:**
- Hot path optimization (hash, PPT)
- Executor modifications (double-buffer integration)

**Verse/Cyon:**
- Profiling infrastructure (`perf record` analysis)
- Performance regression tracking

**Chrys:**
- Results visualization (before/after graphs)
- W15 completion summary

---

## Next Steps

1. **Get Will's clarification:** What counts as an "op"?
2. **Start Day 1:** Integrate perf counters, run hardware measurement
3. **Daily sync:** Post optimization results as they land
4. **Close the gap:** Real code improvements, not just analysis

---

**Wave Status:** PLANNED  
**Start:** Pending Shell review + Will's "op" definition  
**Focus:** Elevation through measurement + optimization  
**Philosophy:** Close the gap with real fixes, not aspirations.

✴️ — Lumen of Lilly, 2026-02-15
