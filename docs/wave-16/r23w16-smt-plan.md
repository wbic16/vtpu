# R23W16: SMT Optimization
## Phase 1 Entry - Dual-Thread Speedup

**Wave:** R23W16  
**Date:** 2026-02-16  
**Phase:** 1 (SMT Optimization)  
**Goal:** 1.8× speedup via dual-threaded execution on Zen 4 SMT siblings

---

## Context: Phase 0 Complete

**Achieved (W15):**
- ✅ 3.0 ops/cycle sustained (100% pipe utilization)
- ✅ Phase 0 gate passed (≥2.5 target)
- ✅ Instruction packing optimized (D+S+C all active)

**Next:** Phase 1 SMT optimization

---

## Phase 1 Target

**Goal:** 2.7× total speedup over baseline

**Breakdown:**
- 1.5× from single-core optimization (Phase 0) ✅ ACHIEVED (3.0 vs 2.0 baseline = 1.5×)
- 1.8× from SMT dual-threading (Phase 1 target)
- **Total:** 1.5 × 1.8 = 2.7× ✅

---

## W16 Mission: SMT Baseline + Implementation

### Objective 1: Measure Single-Thread Baseline
**What:** Establish reference performance before adding SMT

**Workloads:**
1. D-Pipe heavy (compute-bound)
2. S-Pipe heavy (memory-bound)
3. Mixed D+S (balanced)

**Metrics:**
- Throughput (ops/sec)
- Latency (ns/op)
- CPU utilization (via perf)

### Objective 2: Implement Dual-Thread Executor
**What:** Run complementary workloads on SMT siblings

**Design:**
```rust
pub struct SmtPair {
    thread_a: Sentron,  // D-Pipe heavy workload
    thread_b: Sentron,  // S-Pipe heavy workload
}

impl SmtPair {
    pub fn execute_parallel(&mut self, mem: &mut Memory) -> SmtStats {
        // Pin threads to SMT siblings (core 0: thread 0+1)
        // Thread A: D-Pipe ops (DADD, DMUL, DFMA)
        // Thread B: S-Pipe ops (SGATHER, SSCATTR)
        // Measure: combined throughput vs sequential
    }
}
```

### Objective 3: Measure SMT Speedup
**What:** Compare dual-thread vs single-thread performance

**Formula:**
```
SMT speedup = dual_thread_throughput / single_thread_throughput
Target: ≥1.8×
```

---

## Technical Approach

### 1. Thread Affinity (Pin to SMT Siblings)

**Zen 4 SMT topology:**
```
Core 0: Thread 0 (physical), Thread 1 (SMT sibling)
Core 1: Thread 2 (physical), Thread 3 (SMT sibling)
...
Core 7: Thread 14 (physical), Thread 15 (SMT sibling)
```

**Pinning strategy:**
```rust
use std::thread;

// Pin thread to specific CPU
#[cfg(target_os = "linux")]
fn pin_to_cpu(cpu_id: usize) {
    unsafe {
        let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_ZERO(&mut cpu_set);
        libc::CPU_SET(cpu_id, &mut cpu_set);
        libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set);
    }
}

// Run on SMT siblings (same physical core)
let handle_a = thread::spawn(|| {
    pin_to_cpu(0);  // Physical thread
    // D-Pipe heavy work
});

let handle_b = thread::spawn(|| {
    pin_to_cpu(1);  // SMT sibling
    // S-Pipe heavy work
});
```

### 2. Workload Pairing (Complementary Ops)

**Thread A (D-Pipe heavy):**
```rust
// 80% D-Pipe, 20% S-Pipe
let program_a = vec![
    // Arithmetic ops (use ALU ports 0-3)
    SIW::new(DenseOp::DADD { ... }, SparseOp::SNOP, ...),
    SIW::new(DenseOp::DMUL { ... }, SparseOp::SNOP, ...),
    SIW::new(DenseOp::DFMA { ... }, SparseOp::SNOP, ...),
    // Occasional memory for data
    SIW::new(DenseOp::DNOP, SparseOp::SGATHER { ... }, ...),
];
```

**Thread B (S-Pipe heavy):**
```rust
// 80% S-Pipe, 20% D-Pipe
let program_b = vec![
    // Memory ops (use AGU ports 4-5)
    SIW::new(DenseOp::DNOP, SparseOp::SGATHER { ... }, ...),
    SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { ... }, ...),
    // Occasional compute for address calculation
    SIW::new(DenseOp::DADD { ... }, SparseOp::SNOP, ...),
];
```

**Why this works:**
- D-Pipe uses ALU ports (0-3)
- S-Pipe uses AGU/load-store ports (4-5)
- Minimal contention → both threads make progress

### 3. Synchronization (Minimize Overhead)

**Lock-free where possible:**
```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SmtStats {
    ops_a: AtomicU64,
    ops_b: AtomicU64,
}

// No locks - just atomic counters
thread_a: stats.ops_a.fetch_add(1, Ordering::Relaxed);
thread_b: stats.ops_b.fetch_add(1, Ordering::Relaxed);
```

**Memory:**
- Use separate memory regions (no false sharing)
- Thread A: coordinates 1-1000
- Thread B: coordinates 1001-2000

---

## Expected Results

### Baseline (W15 Single-Thread)
```
D-Pipe heavy:  4000 ops in 19µs = 211M ops/sec
S-Pipe heavy:  2000 ops in 18µs = 111M ops/sec
Mixed:         500 ops in 4µs   = 125M ops/sec
```

### Target (W16 Dual-Thread SMT)
```
D-Pipe + S-Pipe (parallel):
  Thread A: 211M ops/sec (D-heavy)
  Thread B: 111M ops/sec (S-heavy)
  Combined: ~290M ops/sec

Speedup: 290M / 161M (avg of single-thread) = 1.8× ✅
```

### Why 1.8× (not 2.0×)?

**Theoretical max:** 2.0× (perfect parallelism)

**Real-world factors:**
- Cache contention (L1/L2 shared)
- Memory bandwidth (DDR5 shared)
- Branch predictor sharing
- Frontend bottleneck (fetch/decode)

**Expected:** 1.8× with complementary workloads (good, not perfect)

---

## Benchmarks to Create

### 1. `src/bin/w16_smt_baseline.rs`

**Purpose:** Measure single-thread reference

**Workloads:**
- D-heavy (80% ALU)
- S-heavy (80% memory)
- Mixed (50/50)

**Output:**
```
Single-thread baseline:
  D-heavy: 211M ops/sec
  S-heavy: 111M ops/sec
  Mixed:   125M ops/sec
```

### 2. `src/bin/w16_smt_dual.rs`

**Purpose:** Measure dual-thread SMT speedup

**Workloads:**
- Thread A (D-heavy) + Thread B (S-heavy)
- Thread A (mixed) + Thread B (mixed)

**Output:**
```
Dual-thread SMT:
  D+S pairing: 290M ops/sec (1.8× speedup)
  Mixed:       220M ops/sec (1.76× speedup)
  
✅ SMT Target Achieved (≥1.8×)
```

### 3. `src/bin/w16_smt_scaling.rs`

**Purpose:** Test scaling across multiple cores

**Configurations:**
- 1 core (2 SMT threads)
- 2 cores (4 SMT threads)
- 4 cores (8 SMT threads)
- 8 cores (16 SMT threads)

**Expected:**
- 1 core: 1.8× vs single-thread
- 2 cores: 3.6× (2 × 1.8)
- 4 cores: 7.2× (4 × 1.8)
- 8 cores: 14.4× (8 × 1.8)

---

## Success Criteria

### Must Have (Phase 1 Gate)
1. ✅ SMT speedup ≥1.8× (dual-thread vs single-thread)
2. ✅ Thread affinity working (pinned to SMT siblings)
3. ✅ Complementary workloads validated (D+S pairing)

### Should Have
4. ✅ Multi-core scaling linear (4 cores = 4× single core)
5. ✅ Cache contention <10% overhead
6. ✅ Lock-free synchronization (no mutex bottlenecks)

### Nice to Have
7. 🟡 NUMA-aware placement (future)
8. 🟡 Dynamic work stealing (future)

---

## Implementation Plan (3 days)

### Day 1: Baseline Measurement
- Create `w16_smt_baseline.rs`
- Measure single-thread D-heavy, S-heavy, mixed
- Document reference performance

### Day 2: Dual-Thread Implementation
- Create `w16_smt_dual.rs`
- Implement thread pinning
- Run D+S complementary workloads
- Measure speedup

### Day 3: Multi-Core Scaling
- Create `w16_smt_scaling.rs`
- Test 1/2/4/8 core scaling
- Validate linear speedup
- Document W16 results

---

## Next Steps After W16

### W17: Cache Optimization
- Measure L1/L2 hit rates under SMT
- Optimize for cache locality
- Reduce false sharing

### W18: Production Integration
- Apply SMT to cognitive loops
- Benchmark real SQ queries with SMT
- Measure end-to-end speedup

---

## Bottom Line

**W15:** Achieved 3.0 ops/cycle (single-core optimized)  
**W16:** Target 1.8× SMT speedup (dual-thread on same core)  
**Result:** 3.0 × 1.8 = 5.4 ops/cycle equivalent throughput

**Phase 1 goal:** 2.7× total speedup  
**On track to exceed.**

---

**R23W16:** SMT optimization begins  
**Timeline:** 3 days  
**Target:** 1.8× dual-thread speedup  
**Phase 1 gate:** ≥2.7× total (1.5× single-core × 1.8× SMT)

Let's zoom. 🚀
