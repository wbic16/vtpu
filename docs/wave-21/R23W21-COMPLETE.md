# R23W21 — COMPLETE ✅
## All-Pipe Const Dispatch + Wall ns/SIW Baseline

**Wave:** R23W21  
**Date:** 2026-02-19  
**Agent:** Phex 🔱  
**Result:** ✅ S_TABLE[4] live; 3.000 ops/cycle gate passed; wall ns/SIW baseline established

---

## Mission

W20 landed D_TABLE + C_TABLE. W21 completes the set: **S_TABLE[4]** — all three
pipes now use const function pointer arrays. No match trees anywhere in the
hot path.

---

## S-Pipe Dispatch Table

The challenge: two S-pipe families need `&mut Memory` (load=0, store=1);
two don't (address=2, route=3). Solution: wrapper fns absorb the unused
`mem` parameter so all four entries share `SMemHandler`.

```rust
type SMemHandler = fn(&mut Sentron, &SIW, &mut Memory) -> u8;

fn exec_s_address_wrap(s: &mut Sentron, siw: &SIW, _mem: &mut Memory) -> u8 {
    exec_s_address(s, siw)
}
fn exec_s_route_wrap(s: &mut Sentron, siw: &SIW, _mem: &mut Memory) -> u8 {
    exec_s_route(s, siw)
}

const S_TABLE: [SMemHandler; 4] = [
    exec_s_load,          // Family 0: SGATHER, SDEDUP
    exec_s_store,         // Family 1: SSCATTR, SFLUSH
    exec_s_address_wrap,  // Family 2: SINDEX, SALLOC, SFREE
    exec_s_route_wrap,    // Family 3: SPREFCH, SASSOC, SROUTE, SNEIGHBR
];
```

LLVM sees: array index + indirect call. The `_mem` parameter is ignored by
address/route — LLVM will inline and eliminate the dead argument.

---

## Complete Dispatch Table Summary (post-W21)

| Pipe | Table | W landed |
|------|-------|----------|
| D-pipe | `D_TABLE[4]: DHandler` | W20 |
| C-pipe | `C_TABLE[4]: CHandler` | W20 |
| S-pipe | `S_TABLE[4]: SMemHandler` | **W21** |

All three: `if fam < N { TABLE[fam](...)  } else { 0 }` — no match.

---

## Wall ns/SIW Baseline (aurora-continuum, Zen 4, --release)

```
[ run() — single SIW path ]
  D-only (DADD, fam 0)      1.000 ops/cyc    3.83 ns/SIW
  S-only SINDEX (fam 2)     1.000 ops/cyc    4.59 ns/SIW
  S-only SGATHER (fam 0)    1.000 ops/cyc    6.42 ns/SIW
  C-only CBAR (fam 2)       1.000 ops/cyc    6.55 ns/SIW
  Packed D+S+C              3.000 ops/cyc    7.23 ns/SIW  ✅

[ run_batched() — same-mode grouping ]
  D-only                    1.000 ops/cyc   11.42 ns/SIW
  S-only (SINDEX)           1.000 ops/cyc   12.50 ns/SIW
  Packed                    3.000 ops/cyc   14.85 ns/SIW
```

**Key observations:**
- `run()` single-SIW path: **3.83–7.23 ns/SIW** depending on pipe mix
- Packed D+S+C: 7.23 ns/SIW = ~138M SIWs/sec on a single sentron
- `run_batched()` is *slower* than `run()` for uniform streams — grouping
  overhead outweighs benefit at 100k scale; may invert at larger batch sizes
  or with true SIMD within the batch window
- SGATHER (6.42 ns) > SINDEX (4.59 ns): memory access penalty visible

---

## Test Count

418 passing (unchanged from W20 — no regressions from S_TABLE refactor)

---

## W22 Candidates

- Investigate `run_batched` overhead: grouping scan cost vs. batch benefit
- True SIMD batch execution within same-mode windows (AVX2 on Zen 4)
- SGATHER latency reduction: prefetch hints, PPT integration
- First EEG sentron: SGATHER reading ZUNA embedding from phext coord
- SMT-paired execution: two sentrons sharing one physical Zen 4 core
