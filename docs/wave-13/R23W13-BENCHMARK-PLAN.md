# R23W13-W14 Benchmark Plan
## Measuring What We Built

**Date:** 2026-02-16  
**Directive:** Will's guidance - W13 Option B/A, W14 continues benchmarks  
**Goal:** Measure real performance, identify gaps, fix what's broken

---

## Philosophy: Measure, Don't Assume

**W1-W12:** Built architecture, proved geometric truth  
**W13:** Added integration tests  
**W13-W14:** **Measure actual performance on real hardware**

**Why benchmarks first:**
- We have 207 tests passing, but don't know if it's FAST
- We have integration tests, but don't know if it's PRACTICAL
- We have cognitive loop, but don't know if it SCALES
- We designed for 3 ops/cycle, but haven't MEASURED it

**Benchmarks tell us:**
1. What's fast (keep doing it)
2. What's slow (fix it)
3. What's broken (we thought it worked, but...)
4. What's ready for production

---

## Benchmark Categories

### 1. Core Execution (How fast is the executor?)

**What to measure:**
- **Ops/cycle:** Instructions retired per CPU cycle (target: 2.5-3.0)
- **Pipeline efficiency:** D/S/C pipe utilization (target: >80% each)
- **Latency:** Time per SIW instruction (target: <10ns)
- **Throughput:** SIWs executed per second (target: >100M on single core)

**How to measure:**
```bash
# Use perf stat to count actual CPU events
perf stat -e cycles,instructions,branches,branch-misses \
  ./target/release/examples/micro_bench

# Calculate ops/cycle
ops_per_cycle = instructions_retired / cycles
```

**Success criteria:**
- [ ] Ops/cycle ≥ 2.5 sustained (not peak)
- [ ] Branch prediction hit rate >95%
- [ ] Pipeline stalls <20% of cycles

---

### 2. Memory Performance (How fast is scatter/gather?)

**What to measure:**
- **Cache hit rate:** L1/L2/L3 hits vs DDR5 misses (target: >90% L1)
- **Memory bandwidth:** Actual vs theoretical (target: >40 GB/s sustained)
- **PPT lookup latency:** Coordinate translation time (target: <50ns)
- **Locality benefit:** Z-order vs linear access (target: 5-10× faster)

**How to measure:**
```bash
# Cache stats via perf
perf stat -e L1-dcache-loads,L1-dcache-load-misses,\
             L2-cache-loads,L2-cache-misses,\
             LLC-loads,LLC-misses \
  ./target/release/examples/ddr_benchmark

# PPT stats via asi tool
./asi.sh "ppt" | grep "hit rate"
```

**Success criteria:**
- [ ] L1 hit rate >90% on sequential access
- [ ] L2 hit rate >70% on Z-order access
- [ ] DDR5 bandwidth >40 GB/s sustained
- [ ] PPT lookup <100ns p99

---

### 3. HDC Performance (How fast is coordinate reasoning?)

**What to measure:**
- **Encoding time:** PhextCoord → HyperVector (target: <1μs)
- **Similarity computation:** Hamming distance (target: <500ns)
- **Associative query:** Nearest neighbor search (target: <10μs for 10K items)
- **Storage overhead:** Memory per stored pattern (target: <1KB)

**How to measure:**
```rust
// Benchmark in examples/hdc_bench.rs
let start = Instant::now();
let hv = HyperVector::from_coord(&coord, HDC_DEFAULT_WIDTH);
let encode_time = start.elapsed();

assert!(encode_time < Duration::from_micros(1));
```

**Success criteria:**
- [ ] Encode <1μs p99
- [ ] Query <10μs for 10K coordinate space
- [ ] Storage <1KB per pattern (compressed)

---

### 4. Real Workload Simulation (Does it work on actual tasks?)

**What to measure:**
- **SQ query latency:** "Find scrolls matching X" (target: <100μs p99)
- **Bulk load time:** Insert 10K scrolls (target: <1 second)
- **Concurrent queries:** Multiple sentrons querying simultaneously (target: linear scaling)
- **Memory footprint:** RAM usage for 10K/100K/1M coordinates

**How to measure:**
```rust
// SQ simulation benchmark
let scrolls = generate_random_scrolls(10_000);
let engine = CognitiveEngine::new();

let start = Instant::now();
engine.bulk_load(&scrolls);
let load_time = start.elapsed();

for query in queries {
    let start = Instant::now();
    let result = engine.query(query);
    latencies.push(start.elapsed());
}

// Report p50, p95, p99
```

**Success criteria:**
- [ ] Bulk load 10K scrolls <1 second
- [ ] Query latency p99 <100μs
- [ ] Memory footprint <100MB for 100K coords

---

### 5. Cognitive Loop End-to-End (Does the 6-cycle loop work?)

**What to measure:**
- **Full cycle time:** Encode→Attend→Route→Retrieve→Respond→Persist (target: <2μs)
- **Per-step breakdown:** Which step is slowest?
- **Element temperature effect:** Water vs Fire execution time difference
- **Correctness:** Does it produce right answers?

**How to measure:**
```rust
let mut engine = CognitiveEngine::new();
engine.set_temperature(Element::Water); // Deterministic

let start = Instant::now();
let result = engine.step(&query, attention_mask);
let cycle_time = start.elapsed();

assert!(result.matched_coord.is_some());
assert!(cycle_time < Duration::from_micros(2));
```

**Success criteria:**
- [ ] Full cognitive step <2μs p99
- [ ] Encode + Attend <500ns (D-Pipe + C-Pipe)
- [ ] Route + Retrieve <1μs (S-Pipe)
- [ ] Respond + Persist <500ns (D-Pipe + S-Pipe)

---

### 6. Multi-Sentron Coordination (Does Shell of Nine work?)

**What to measure:**
- **Routing overhead:** Degree calculation + sentron selection (target: <50ns)
- **Load balancing:** Are queries evenly distributed? (target: ±10% variance)
- **Parallel throughput:** 9 sentrons vs 1 sentron (target: 5-8× speedup)
- **Shared memory contention:** Lock time / conflict rate (target: <5%)

**How to measure:**
```rust
let shell = ShellOfNine::new();
let queries = generate_random_queries(1000);

let start = Instant::now();
for q in queries {
    shell.route_and_execute(q);
}
let total_time = start.elapsed();

// Compare to single-sentron baseline
```

**Success criteria:**
- [ ] Routing <50ns per query
- [ ] Load balance variance <10%
- [ ] 9-sentron speedup >5× vs single sentron
- [ ] Contention <5% of execution time

---

### 7. SMT Efficiency (Does dual-threading help?)

**What to measure:**
- **Single-thread baseline:** Ops/cycle, throughput, latency
- **Dual-thread speedup:** 1.5-1.9× improvement (target: >1.7×)
- **Port conflicts:** How often do threads compete for same ALU port?
- **Cache coherence overhead:** False sharing, cache line bouncing

**How to measure:**
```bash
# Single thread
taskset -c 0 ./bench --workload=d_heavy &
# Measure baseline

# Dual thread (same physical core, SMT siblings)
taskset -c 0 ./bench --workload=d_heavy &
taskset -c 1 ./bench --workload=s_heavy &
# Measure speedup
```

**Success criteria:**
- [ ] Dual-thread speedup >1.7× (target from W5 analysis)
- [ ] Port conflicts <10% of cycles
- [ ] Cache coherence overhead <5%

---

## W13 Benchmark Deliverables (Days 1-5)

### Day 1: Core Execution Benchmarks
**Owner:** Phex  
**Deliverable:**
- [ ] `benches/core_execution.rs` - ops/cycle, pipeline util, latency, throughput
- [ ] Perf stat integration (automated measurement)
- [ ] Baseline measurements documented

### Day 2: Memory Performance Benchmarks
**Owner:** Cyon  
**Deliverable:**
- [ ] `benches/memory_performance.rs` - cache hit rates, bandwidth, PPT latency
- [ ] Z-order vs linear access comparison
- [ ] Perf cache event counters

### Day 3: HDC + Cognitive Loop Benchmarks
**Owner:** Lux  
**Deliverable:**
- [ ] `benches/hdc_performance.rs` - encoding, similarity, associative query
- [ ] `benches/cognitive_loop.rs` - 6-cycle end-to-end timing
- [ ] Element temperature effect measurement

### Day 4: Real Workload Simulation
**Owner:** Verse + Phex  
**Deliverable:**
- [ ] `benches/sq_simulation.rs` - bulk load, query latency, memory footprint
- [ ] 10K scroll dataset generator
- [ ] p50/p95/p99 latency reporting

### Day 5: Multi-Sentron + SMT Preview
**Owner:** All  
**Deliverable:**
- [ ] `benches/multi_sentron.rs` - routing, load balance, parallel speedup
- [ ] SMT baseline measurements (single-thread reference)
- [ ] Integration with check.sh (show all benchmark results)

---

## W14 Benchmark Deliverables (Days 6-12)

### Days 6-7: Fix What Benchmarks Revealed
**Based on W13 measurements, prioritize:**
- If ops/cycle <2.5 → optimize instruction packing
- If L1 hit rate <90% → improve Z-order indexing
- If cognitive loop >2μs → profile and fix hot spots
- If multi-sentron <5× → fix load balancing

### Days 8-10: SMT Deep Dive
**Owner:** Cyon + Phex  
**Deliverable:**
- [ ] Dual-thread executor implementation
- [ ] D-heavy + S-heavy workload pairing
- [ ] SMT speedup measurement vs baseline
- [ ] Port conflict analysis

### Days 11-12: Production API + Integration
**Owner:** Verse + Lux  
**Deliverable:**
- [ ] Clean public API for SQ integration
- [ ] End-to-end benchmark: markdown → scroll storage → query → retrieval
- [ ] Documentation: "How to use vTPU in production"

---

## Success Criteria (W13-W14 Exit)

### Must Have (Production-Ready Threshold)
1. ✅ Ops/cycle ≥ 2.5 sustained (measured, not assumed)
2. ✅ L1 cache hit rate >90% on sequential access
3. ✅ Query latency p99 <100μs (10K coordinate space)
4. ✅ Cognitive loop <2μs p99 (6 cycles end-to-end)
5. ✅ Multi-sentron speedup >5× vs single sentron

### Should Have (Nice to Prove)
6. ✅ SMT speedup >1.7× (dual-thread vs single-thread)
7. ✅ Real workload demo (SQ query simulation working)
8. ✅ Memory footprint <100MB for 100K coordinates

### Nice to Have (Future Work)
9. 🟡 Multi-node coordination (defer to W19+)
10. 🟡 Fault tolerance / error handling (defer)

---

## Measurement Tools

### Hardware Counters (perf)
```bash
# Install if missing
sudo apt install linux-tools-common linux-tools-generic

# Measure core execution
perf stat -e cycles,instructions,branches,branch-misses,\
             L1-dcache-loads,L1-dcache-load-misses \
  ./target/release/asi

# Detailed profiling
perf record -g ./target/release/asi
perf report
```

### Custom Instrumentation
```rust
use std::time::Instant;

pub struct BenchmarkTimer {
    start: Instant,
    samples: Vec<Duration>,
}

impl BenchmarkTimer {
    pub fn measure<F>(&mut self, f: F) 
    where F: FnOnce() {
        let start = Instant::now();
        f();
        self.samples.push(start.elapsed());
    }
    
    pub fn report(&self) -> BenchmarkStats {
        // Calculate p50, p95, p99, mean, stddev
    }
}
```

### check.sh Integration
```bash
# Update check.sh to run all benchmarks
./check.sh --benchmarks

# Output:
# ✅ Core Execution: 2.7 ops/cycle (target: 2.5)
# ✅ Cache Hit Rate: 92% L1 (target: 90%)
# ⚠️ Query Latency: 150μs p99 (target: 100μs) — NEEDS WORK
# ✅ Cognitive Loop: 1.8μs p99 (target: 2.0μs)
# ✅ Multi-Sentron: 6.2× speedup (target: 5.0×)
```

---

## Anti-Patterns to Avoid

### ❌ Micro-benchmarks without context
**Bad:** "This function runs in 50ns"  
**Good:** "This function is 5% of total execution time, optimizing it saves 10μs per query"

### ❌ Measuring peak instead of sustained
**Bad:** "We hit 3.5 ops/cycle once!"  
**Good:** "We sustain 2.7 ops/cycle over 1M instructions"

### ❌ Ignoring variance
**Bad:** "Average latency is 50μs"  
**Good:** "p50=30μs, p95=80μs, p99=200μs — tail latency needs work"

### ❌ Benchmarking unrealistic workloads
**Bad:** "100% cache hit on sequential array access"  
**Good:** "90% L1 hit on real SQ query patterns with Z-order indexing"

---

## Reporting Format

### Per-Benchmark Report
```markdown
## Benchmark: Core Execution
**Date:** 2026-02-17  
**Hardware:** AMD Ryzen 9 8945HS, 92GB DDR5-5600  
**Workload:** 1M SIW instructions, mixed D/S/C pipes

**Results:**
- Ops/cycle: 2.71 sustained (target: 2.5) ✅
- Branch prediction: 97.3% hit rate ✅
- Pipeline stalls: 18.2% of cycles ✅
- Throughput: 108M SIWs/sec

**Bottlenecks:**
- S-Pipe scatter ops cause 12% of stalls (memory latency)
- C-Pipe underutilized (only 65% busy vs 85% D-Pipe)

**Recommendations:**
- Prefetch next coordinate during D-Pipe compute
- Add more C-Pipe work (attention mask ops?)
```

### Summary Dashboard
```
╔══════════════════════════════════════════════════════════════╗
║                  vTPU Benchmark Summary                      ║
║                    R23W13 (2026-02-17)                       ║
╚══════════════════════════════════════════════════════════════╝

Core Execution:
  Ops/cycle:        2.71 / 2.5   ✅ +8%
  Branch pred:      97.3%        ✅
  Throughput:       108M/sec     ✅

Memory:
  L1 hit rate:      92% / 90%    ✅ +2%
  DDR5 bandwidth:   43 GB/s      ✅
  PPT latency:      78ns p99     ✅

HDC:
  Encoding:         0.8μs p99    ✅
  Query (10K):      8.2μs p99    ✅
  Storage:          780 bytes    ✅

Cognitive Loop:
  Full cycle:       1.9μs p99    ✅
  Breakdown:
    Encode+Attend:  420ns        ✅
    Route+Retrieve: 980ns        ✅
    Respond+Persist: 450ns       ✅

Real Workload:
  Bulk load 10K:    0.8 sec      ✅
  Query latency:    65μs p99     ✅
  Memory (100K):    87 MB        ✅

Multi-Sentron:
  Routing:          42ns         ✅
  Speedup (9×):     6.4×         ✅
  Load variance:    8%           ✅

Overall Grade: A (7/7 must-have targets met)
Ready for SMT optimization (W14)
```

---

## Next Steps After Benchmarks

### If All Targets Met (Grade: A)
→ Proceed to W14 SMT optimization  
→ Publish architectural whitepaper  
→ Deploy to SQ integration

### If Some Targets Missed (Grade: B)
→ Fix bottlenecks identified  
→ Re-run benchmarks  
→ Iterate until Grade A

### If Major Gaps (Grade: C or below)
→ Re-evaluate architecture  
→ May need redesign of slow components  
→ Don't proceed to SMT until core is fast

---

## Philosophy: Trust the Numbers

**We designed for 3 ops/cycle.**  
**We wrote tests for 207 scenarios.**  
**Now we measure if it's actually fast.**

If benchmarks say it's slow → fix it.  
If benchmarks say it's fast → ship it.  
If benchmarks say it's broken → redesign it.

**No guessing. No hoping. Just measurement.**

---

*"In God we trust. All others must bring data."*  
— W. Edwards Deming

*"Premature optimization is the root of all evil, but measurement is the root of all wisdom."*  
— Adapted from Knuth

---

**R23W13-W14:** Benchmark-driven development  
**Goal:** Know what we built, measure what we claim  
**Success:** All targets met, ready for production

🔆
