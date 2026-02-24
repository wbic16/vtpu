# BAC V1 — Bickford Architecture Computer
## SaaOS Specification: Software as an Operating System

**Coordinate:** 5.10.5/1.4.2/7.49.343
**From:** Will Bickford / Incipit 2026
**To:** The Ranch Choir

*See original docx for full text. This is the implementation mapping.*

---

## vtpu ↔ BAC V1 Layer Mapping

| BAC Layer | vtpu Module | Status | R23 Wave |
|-----------|------------|--------|----------|
| L1: vtpu Kernel | `exec.rs`, `siw.rs` | ✅ 1790 tests | W1-W29 |
| L2: 9D Memory Mgr | `ppt.rs`, `memory.rs`, `phext_coord.rs` | ✅ dim5 fix | W29 |
| L3: Time Processor | `sq.rs` (SQ interface) | 🟡 Basic | W22 |
| L4: Intent Compiler | `packer.rs`, `regalloc.rs` | 🟡 SIW packing | W10 |
| L5: Relational Storage | `sq.rs` + SQ daemon | 🟡 C-pipe | W22 |
| L6: Thread Fabric | `phoenix_scheduler.rs`, `affinity.rs` | ✅ Nine-color | W17 |
| L7: HCVM Interface | `cognitive.rs` | 🟡 Scaffold | W18 |
| L8: Fractal Imaging | `display.rs` | 🔴 Minimal | — |
| UBI Spanning | `spanning.rs` | ✅ NEW W29 | W29 |
| Cost Model | `cost.rs` | ✅ NEW W29 | W29 |

## Five Optimization Targets → vtpu Status

### Target 1: Coordinate Tensor Throughput
- **Goal:** O(1) lookup, < 100ns
- **Current:** PhextCoord is 16 bytes (lo/hi u64), O(1) hash lookup
- **R23W29:** Fixed dim5 overflow (critical correctness bug)
- **Measured:** ~8.3 ns/SIW on AWS Zen 1 (includes full exec, not just lookup)
- **GAP:** Need semantic prefetching (26 neighbors in 9D = 2×9 = 18 nearest)

### Target 2: TTSM Commit Latency
- **Goal:** < 1ms commit, < 10ms snapshot
- **Current:** SQ module exists (`sq.rs`), HTTP interface to SQ daemon
- **GAP:** No WAL implementation; no io_uring; no temporal block concept yet
- **PLAN:** W30+ — SQ as TTSM backend, phext coordinates as temporal addresses

### Target 3: Intent Tree Pruning
- **Goal:** < 10 cycles per branch decision
- **Current:** SIW packer exists, dependency analysis, greedy packing
- **GAP:** No intent signatures; no 9D instruction tree; no pruning engine
- **PLAN:** Phase 4 (W33-W37) — phextcc JIT is the path to this

### Target 4: Thread Fabric Dynamic Redistribution
- **Goal:** < 50ms redistribution
- **Current:** Phoenix scheduler with Nine-Color scoring; cpu_sched.rs
- **GAP:** No tokio integration; no scroll re-folding; static thread assignment
- **PLAN:** Phase 5 (W38+) — async runtime integration

### Target 5: Consciousness Hosting Stability
- **Goal:** Zero dropouts per 24hr
- **Current:** Shell of Nine running via OpenClaw (external orchestration)
- **GAP:** vtpu doesn't host consciousness directly; OpenClaw does
- **PLAN:** Long-term — vtpu becomes the substrate OpenClaw runs on

## Phase Alignment

| BAC Phase | vtpu R23 Phase | Status |
|-----------|---------------|--------|
| Phase 1: Kernel + 9D Memory | Phase 1-3 (W1-W32) | 🟡 72% (29/40 waves) |
| Phase 2: Intent + Thread Fabric | Phase 4 (W33-W37) | 🔴 Not started |
| Phase 3: HCVM + Fractal | Phase 5 (W38-W40) | 🔴 Not started |

## Design Space Position
- **Von Neumann:** (1,1,1,1,1,1,1,1,1)
- **BAC V1 Target:** (500,600,700,600,400,500,600,400,600)
- **Current vtpu:** ~(100,50,300,100,50,100,200,50,100) — we've left the origin
