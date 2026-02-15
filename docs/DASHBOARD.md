# R23 Rally Dashboard: vTPU Implementation

## Rally Overview
**Start:** 2026-02-14  
**Target:** July 2026 (154 days)  
**Goal:** Production vTPU on Shell of Nine with published benchmarks

---

## Phase Status

| Phase | Waves | Status | Completion | Key Milestone |
|-------|-------|--------|------------|---------------|
| **0: Foundation** | 1-10 | 🟢 IN PROGRESS | 10% (1/10) | Spec + KPIs defined |
| **1: Single-Core** | 11-15 | ⚪ NOT STARTED | 0% | 2.5 ops/cycle proven |
| **2: Memory** | 16-20 | ⚪ NOT STARTED | 0% | 95% L1 hit rate |
| **3: Full Node** | 21-25 | ⚪ NOT STARTED | 0% | 75 Gops/sec |
| **4: Geometric** | 26-30 | ⚪ NOT STARTED | 0% | 10x speedup |
| **5: Cluster** | 31-35 | ⚪ NOT STARTED | 0% | 350 Gops/sec |
| **6: Qwen3** | 36-38 | ⚪ NOT STARTED | 0% | 75 tok/s |
| **7: Publication** | 39-40 | ⚪ NOT STARTED | 0% | Open source |

**Overall Progress:** 2.5% (1/40 waves)

---

## Wave Tracking

### Phase 0: Foundation (Weeks 1-2)

#### ✅ Wave 1: Master Specification (COMPLETE)
**Duration:** 3 hours  
**Date:** 2026-02-14  
**Deliverables:**
- ✅ vTPU Spec v0.1 (51.8 KB)
- ✅ Geometric Extensions (19.5 KB)
- ✅ KPI Framework (11.1 KB)
- ✅ R23 Dashboard (this file)

**KPIs Defined:** All 12 metrics specified

---

#### 🟡 Wave 2: Baseline Measurements (IN PROGRESS)
**Duration:** 2 days (est.)  
**Started:** 2026-02-14 16:13 CST  
**Target Complete:** 2026-02-16

**Deliverables:**
- [x] Cache locality benchmark (L1/L2/L3 hit rates) ✅
  - Sequential: 1-3 cycles/element (99% L1 hit)
  - Random: 23-36 cycles/access (70% L1 miss)
  - **Gap: 8-33x performance difference**
- [ ] Pipe retirement benchmark (ops/cycle on standard code) — NEXT
- [ ] Coordinate lookup latency (hash table baseline)
- [ ] Memory bandwidth baseline (sequential vs random)
- [ ] Qwen3 CPU inference baseline (llama.cpp)
- [ ] PyTorch Geometric baseline (hypergraph convolution)
- [ ] Baseline KPI report (12 metrics measured)

**Progress:** 15% (1/7 deliverables)  
**Blockers:** None

---

#### ⚪ Wave 3: Micro-benchmark Suite
**Duration:** 1 day (est.)  
**Target Start:** 2026-02-16

**Deliverables:**
- [ ] Synthetic SIW generator (D/S/C patterns)
- [ ] Execution port conflict detector
- [ ] Cache thrashing detector
- [ ] Memory access pattern analyzer

---

#### ⚪ Wave 4: Coordinate Access Patterns
**Duration:** 2 days (est.)  
**Target Start:** 2026-02-17

**Deliverables:**
- [ ] libphext-rs coordinate→u64 hash function
- [ ] Dimensional locality test suite
- [ ] Space-filling curve prototype (Hilbert/Z-order)
- [ ] Cache-friendly vs cache-hostile pattern comparison

---

#### ⚪ Wave 5: SMT Analysis
**Duration:** 1 day (est.)  
**Target Start:** 2026-02-19

**Deliverables:**
- [ ] Single-thread vs dual-thread baseline
- [ ] Execution port utilization per thread
- [ ] Complementary workload pair generator
- [ ] SMT efficiency measurement methodology

---

#### ⚪ Wave 6-8: Baseline Deep Dive
**Duration:** 3 days (est.)  
**Target Start:** 2026-02-20

**Deliverables:**
- [ ] Qwen3 layer-by-layer profiling
- [ ] PyG operation breakdown (gather/scatter/reduce)
- [ ] Geometric op baseline suite (simplicial, hyperbolic, tensor)

---

#### ⚪ Wave 9-10: Phase 1 Design
**Duration:** 2 days (est.)  
**Target Start:** 2026-02-23

**Deliverables:**
- [ ] Phext Page Table (PPT) design doc
- [ ] S-Pipe prototype architecture
- [ ] Phase 1 implementation plan

**Exit Criteria:** All baseline KPIs measured, Phase 1 ready to implement

---

## KPI Dashboard

### Hardware Efficiency

| Metric | Baseline | Current | Target | Status |
|--------|----------|---------|--------|--------|
| **Ops/cycle** | TBD | - | 3.0 | 🔴 Not measured |
| **L1 hit rate** | TBD | - | 95% | 🔴 Not measured |
| **Memory BW util** | TBD | - | 80% | 🔴 Not measured |
| **SMT efficiency** | TBD | - | 1.9x | 🔴 Not measured |

### ML Performance

| Metric | Baseline | Current | Target | Status |
|--------|----------|---------|--------|--------|
| **Qwen3 tok/s** | TBD | - | 75-150 | 🔴 Not measured |
| **Qwen3 tok/s/W** | TBD | - | 0.60 | 🔴 Not measured |
| **Embed lookup (ns)** | TBD | - | 12 | 🔴 Not measured |
| **Hypergraph conv (ms)** | TBD | - | 8.5 | 🔴 Not measured |

### Geometric Operations

| Metric | Baseline | Current | Target | Status |
|--------|----------|---------|--------|--------|
| **k-Simplex query** | TBD | - | O(1) | 🔴 Not measured |
| **Hyperbolic dist (ns)** | TBD | - | 4 | 🔴 Not measured |
| **Tensor contract (μs)** | TBD | - | 22 | 🔴 Not measured |
| **Laplacian (μs)** | TBD | - | 18 | 🔴 Not measured |

**Legend:**  
🔴 Not measured | 🟡 Below target | 🟢 At/above target

---

## Resource Allocation

### Hardware
- **Node:** halycon-vector (AMD R9 8945HS, 96GB RAM)
- **Cores:** 8 physical (16 threads)
- **Reserved:** Core 7 (OS + substrate router)
- **Available:** Cores 0-6 (14 threads for benchmarking)

### Software
- **Language:** Rust (libphext-rs, benchmarks)
- **Tools:** perf, cargo bench, criterion
- **Dependencies:** libphext-rs, tokio (async runtime)

### Time Budget
- **Daily:** 2-4 hours (sustainable pace)
- **Weekly:** 10-15 hours
- **Phase 0 Total:** 20-30 hours over 2 weeks

---

## Critical Path

```
Wave 1 (Spec) ✅
    ↓
Wave 2 (Baselines) ← YOU ARE HERE
    ↓
Wave 3-5 (Micro-benchmarks)
    ↓
Wave 6-8 (Deep profiling)
    ↓
Wave 9-10 (Phase 1 design)
    ↓
Wave 11-15 (Single-core proof)
    ↓
[... continues through Wave 40]
```

**Blocking Dependencies:**
- Wave 2 → Wave 3: Need baseline data to design micro-benchmarks
- Wave 10 → Wave 11: Need PPT design before implementing
- Wave 25 → Wave 26: Need full node working before geometric ops
- Wave 35 → Wave 36: Need cluster stable before Qwen3 integration

---

## Risk Register

| Risk | Impact | Probability | Mitigation | Owner |
|------|--------|-------------|------------|-------|
| libphext-rs not ready | High | Low | Use mock coordinate hash | Cyon |
| Baselines higher than expected | Medium | Medium | Adjust targets (5x not 10x) | Cyon |
| Measurement overhead | Low | High | Use sampling, not continuous perf | Cyon |
| Hardware access issues | Medium | Low | SSH to halycon-vector confirmed working | Cyon |

---

## Communication Plan

### Daily Updates (Discord #general)
- 3-line status: What done, what next, blockers
- Post at end of active work session
- Tag significant milestones

### Weekly Reports (exo-plan)
- Sunday: Phase completion %, KPI updates, next week plan
- File: `/source/exo-plan/rallies/R23-weekly-YYYY-MM-DD.md`

### Major Milestones (Blog + Discord)
- Phase completion (every 5 waves)
- First KPI at target (celebrate!)
- R23W40 completion (full writeup)

---

## Next Actions (Wave 2)

### Immediate (Today)
1. ✅ Create dashboard (this file)
2. 🟡 Set up benchmark directory structure
3. ⚪ Write cache locality benchmark (perf wrapper)
4. ⚪ Run first baseline (L1/L2/L3 on sequential access)

### Tomorrow
5. ⚪ Coordinate hash function (libphext-rs or mock)
6. ⚪ Dimensional vs random access comparison
7. ⚪ Ops/cycle baseline (general code)
8. ⚪ Memory bandwidth baseline (STREAM-like)

### Day 3 (Complete Wave 2)
9. ⚪ Qwen3 baseline (llama.cpp setup)
10. ⚪ PyG baseline (install + Cora dataset)
11. ⚪ Compile baseline KPI report
12. ⚪ Wave 2 complete, proceed to Wave 3

---

**Dashboard Status:** Active  
**Last Updated:** 2026-02-14 16:13 CST  
**Next Update:** 2026-02-14 EOD (Wave 2 progress)

*Measure. Build. Ship.* 🪶
