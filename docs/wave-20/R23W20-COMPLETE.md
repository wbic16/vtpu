# R23W20 — COMPLETE ✅
## Const Dispatch Tables + 400 Tests

**Wave:** R23W20
**Date:** 2026-02-19
**Agent:** Phex 🔱
**Result:** ✅ 400 passing tests, const function pointer dispatch tables live

---

## Mission

1. Replace `match siw.d_fam/c_fam` with const function pointer arrays (true O(1) dispatch)
2. Boost test coverage to 400 passing tests
3. Prepare clean baseline for W21

---

## Deliverables

### 1. Const Dispatch Tables (`src/exec.rs`)

```rust
type DHandler = fn(&mut Sentron, &SIW) -> u8;
const D_TABLE: [DHandler; 4] = [
    exec_d_arithmetic,   // Family 0: DADD, DSUB, DMUL, DFMA, DCMP, DSEL, DMOV
    exec_d_reduce,       // Family 1: DRED
    exec_d_hdc,          // Family 2: DHDENC, DHDBIND, DHDBUND, DHDPERM, DHDSIM
    exec_d_ternary,      // Family 3: DTERNARY, DTPOP, DTACC
];

type CHandler = fn(&mut Sentron, &SIW) -> u8;
const C_TABLE: [CHandler; 4] = [
    exec_c_pack,         // Family 0: CPACK
    exec_c_send,         // Family 1: CSEND, CRECV, CROUTE
    exec_c_barrier,      // Family 2: CBAR, CFENCE
    exec_c_reduce,       // Family 3: CREDUCE, CCAST
];
```

LLVM sees array index + single indirect call instead of a match tree.
S-pipe retains match (needs `&mut Memory` arg — different type signature).

### 2. Test Coverage: 400 Passing

| Module | Before W20 | After W20 |
|--------|-----------|-----------|
| display | 3 | 8 |
| telemetry | 3 | 9 |
| stream | 4 | 11 |
| c_pipe | 4 | 10 |
| memory | 5 | 11 |
| siw | 4 | 12 |
| pool | 5 | 13 |
| validation | 3 | 9 |
| exec (dispatch tables) | — | 9 |
| **Total** | **337** | **400** |

### 3. Bug: SINDEX coordinate accumulation fixed
Phext coordinates are 1-indexed, 11-bit max (2047).
`exec_s_address` now clamps SINDEX result to `[1, 2047]` — prevents panic on long-running streams.

---

## Test Count History
| Wave | Tests |
|------|-------|
| W18  | 309   |
| W19  | 337   |
| W20  | 400 ✅ |

---

## W21 Candidates
- S-pipe function pointer table (requires wrapper to absorb `&mut Memory`)
- `run_batched` SIMD optimization within same-mode runs
- Prefetcher integration into `run()` / `run_batched()`
- SMT-paired sentron execution (two sentrons sharing one physical core)
