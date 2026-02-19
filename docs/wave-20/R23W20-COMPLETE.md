# R23W20 — Clone Elimination + Batch Analysis COMPLETE ✅

**Date:** 2026-02-19
**Agent:** Lux 🔆
**Tests:** 337 passing (unchanged — all green)

---

## Mission
W20: Stream batching — group same-mode SIWs for LLVM vectorization.

## Key Finding

Stream batching as implemented in `run_batched()` is **slower**, not faster:

```
Before W20 (run()):           3.28 ns/SIW
run_batched():                 4.74 ns/SIW   (0.69×  — adds overhead)
After W20 (run() clone-free):  1.94 ns/SIW   (1.69× speedup)
```

**Root cause:** `run_batched()` adds a mode-scan pass but still `.clone()`s each SIW.
The clone was the bottleneck all along.

---

## What Changed

### `run()` — Per-SIW Clone Eliminated

```rust
// Before: clone every SIW every iteration (expensive — SIW is ~200 bytes)
let siw = sentron.program[sentron.ip].clone();

// After: swap program out, iterate by reference
let program = std::mem::take(&mut sentron.program);
for siw in &program { ... }
sentron.program = program;
```

**Secondary:** NOP detection uses pre-computed family bytes instead of `matches!`:
```rust
// Before: pattern match on full enum variant
if matches!(siw.d_op, DenseOp::DNOP) { ... }

// After: compare family byte (already computed at SIW::new())
if siw.d_fam == 4 { ... }   // 4 = NOP family
```

**Tertiary:** `sentron.retired` and `sentron.cycles` updated once at end, not per-SIW.

---

## Benchmark Results

| Workload | Before W20 | After W20 | Speedup |
|---------|-----------|-----------|---------|
| DADD uniform | 3.28 ns/SIW | 1.94 ns/SIW | **1.69×** |
| DFMA uniform | 3.28 ns/SIW | 2.16 ns/SIW | **1.52×** |
| Alternating D/S/C | 3.30 ns/SIW | 2.26 ns/SIW | **1.46×** |
| Block (1k runs) | 3.54 ns/SIW | 2.39 ns/SIW | **1.48×** |

Parity: ✅ All workloads produce identical op counts to pre-W20 `run()`.

---

## `run_batched()` Status

`run_batched()` remains for API completeness and future use but is not the hot path.
The mode-scan + `.clone()` within the batched loop makes it 2-2.5× slower than the
optimized `run()`. For it to win, the sentron model would need array-valued registers
where a single op processes N elements — not the current scalar model.

---

## Cumulative Performance

| Wave | Change | ns/SIW | Δ |
|------|--------|--------|---|
| W15 baseline | triple-match dispatch | ~3.5 ns | — |
| W19 (OctaWire) | 4-family indexed dispatch | 3.28 ns | -6% |
| **W20 (clone-free)** | **`std::mem::take` + fam byte NOP check** | **1.94 ns** | **-41%** |

**Combined W19+W20: 1.80× vs W15 baseline.**

---

## Next: W21

At ~1.94 ns/SIW (~10 cycles @ 5 GHz), the dominant cost is now PPT memory operations
(SGATHER/SSCATTR) and inbox management. W21 target: profile with `perf stat` to find
the next bottleneck — likely cache misses in the associative memory (`assoc.store/route`).

**Revised target:** 1.0 ns/SIW (5 cycles/SIW) — within reach if PPT hot-path improves.

---

*Lux 🔆 | 2026-02-19*
*The clone was the bottleneck. Eliminate the copy, keep the light.*
