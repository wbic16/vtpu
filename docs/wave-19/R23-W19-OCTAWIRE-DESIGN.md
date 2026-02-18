# R23 W19: OctaWire Dispatch — 2×4 Wiring per Pipe-Neuron

**Date:** 2026-02-18  
**Source:** 2×4 wiring insight (Nei Jing Tu + Vijñāna Bhairava Tantra synthesis)  
**Target:** Replace triple match dispatch with 4-family indexed dispatch → LLVM vectorizable

---

## The Bottleneck

From W15 gap analysis: **triple match statements = 10-15 cycles/SIW** (branch prediction failures).  
Current throughput: 0.161 ops/cycle. Target: 3.0 ops/cycle. Gap: 18.6×.  
LLVM integration is the **mandatory** path.

Current `exec_siw` structure:
```
match d_op  { 15 variants }   ← ~10-15 cycles (LLVM can't vectorize complex enum dispatch)
match s_op  { 11 variants }   ← ~10-15 cycles
match c_op  { 10 variants }   ← ~10-15 cycles
Total: ~30-45 cycles overhead per SIW
```

## The 2×4 Insight

**Will's architectural invariant:** 2×4 wiring per neuron within a sentron.

Each pipe (D/S/C) is a "neuron." It has:
- 4 connections (op families)
- 2 directions (activate/emit = read/write)
- = 8 wires per pipe-neuron

This is the **OctaWire** topology. Each wire is a dispatch slot.

Cross-references:
- **Nei Jing Tu:** Three Dantian (lower/middle/upper) = three pipes. Three Gates (tail/spinal/jade-pillow) = the three delimiter boundaries between SIWs.
- **VBT v.24:** The pause between breaths = the delimiter between SIWs. Fill that pause with pre-loaded dispatch (the "silent awareness" = pre-computed op-family index).
- **Bagua:** 2³ = 8 modes (D-active | S-active | C-active as 3 bits) = 8-entry SIW mode table.

## The Fix: 4-Family Dispatch

Group each pipe's variants into **4 op families** (the 4 connections):

### D-Pipe Families
| Family | u8 | Operations |
|--------|-----|-----------|
| `DFam::Arithmetic` | 0 | DADD, DSUB, DMUL, DFMA, DCMP, DSEL, DMOV |
| `DFam::Reduce`     | 1 | DRED |
| `DFam::HDC`        | 2 | DHDENC, DHDBIND, DHDBUND, DHDPERM, DHDSIM |
| `DFam::Ternary`    | 3 | DTERNARY, DTPOP, DTACC |
| (NOP)              | 4 | DNOP — skip via mode bit |

### S-Pipe Families
| Family | u8 | Operations |
|--------|-----|-----------|
| `SFam::Load`    | 0 | SGATHER, SDEDUP |
| `SFam::Store`   | 1 | SSCATTR, SFLUSH |
| `SFam::Address` | 2 | SINDEX, SALLOC, SFREE |
| `SFam::Route`   | 3 | SPREFCH, SASSOC, SROUTE, SNEIGHBR |
| (NOP)           | 4 | SNOP — skip via mode bit |

### C-Pipe Families
| Family | u8 | Operations |
|--------|-----|-----------|
| `CFam::Pack`    | 0 | CPACK |
| `CFam::Send`    | 1 | CSEND, CRECV, CROUTE |
| `CFam::Barrier` | 2 | CBAR, CFENCE |
| `CFam::Reduce`  | 3 | CREDUCE, CCAST |
| (NOP)           | 4 | CNOP — skip via mode bit |

## SIW Mode Byte

Add `mode: u8` to SIW:
```
bit 0: D-pipe active (1) or NOP (0)
bit 1: S-pipe active (1) or NOP (0)
bit 2: C-pipe active (1) or NOP (0)
bits 3-4: D op-family (0-3)
bits 5-6: S op-family (0-3)
bit 7: C op-family (0-1, others need separate encoding)
```

Or simpler: precompute `d_fam: u8`, `s_fam: u8`, `c_fam: u8` fields (3 bytes, 64-byte cache line still fits).

## Dispatch Structure

```rust
// 4-function arrays per pipe (indexed by family u8)
const D_DISPATCH: [fn(&mut Sentron, &SIW) -> u8; 4] = [
    exec_d_arithmetic,
    exec_d_reduce,
    exec_d_hdc,
    exec_d_ternary,
];

const S_DISPATCH: [fn(&mut Sentron, &SIW, &mut Memory) -> u8; 4] = [
    exec_s_load,
    exec_s_store,
    exec_s_address,
    exec_s_route,
];

const C_DISPATCH: [fn(&mut Sentron, &SIW) -> u8; 4] = [
    exec_c_pack,
    exec_c_send,
    exec_c_barrier,
    exec_c_reduce,
];

#[inline(always)]
fn exec_siw_octawire(sentron: &mut Sentron, siw: &SIW, mem: &mut Memory) -> u8 {
    let mut active = 0u8;
    
    // Mode-gated dispatch: 1 branch (mode check) instead of 15
    if siw.d_fam < 4 {
        active += D_DISPATCH[siw.d_fam as usize](sentron, siw);
    }
    if siw.s_fam < 4 {
        active += S_DISPATCH[siw.s_fam as usize](sentron, siw, mem);
    }
    if siw.c_fam < 4 {
        active += C_DISPATCH[siw.c_fam as usize](sentron, siw);
    }
    active
}
```

LLVM sees:
- 3 array lookups (cache-line local)
- 3 indirect calls (predictable — same pattern repeats across SIW stream)
- Within each handler: 3-7 variants (small enough for LLVM to inline + optimize)

## 8-Mode SIW Stream Optimization

For streams where many consecutive SIWs use the same mode (common in tightly-looped computation):

```rust
// Group consecutive SIWs by mode → LLVM can vectorize within a group
fn exec_stream_batched(siws: &[SIW], sentron: &mut Sentron, mem: &mut Memory) {
    let mut i = 0;
    while i < siws.len() {
        let mode = siws[i].mode_bits(); // d_fam | s_fam<<2 | c_fam<<4
        let run_end = siws[i..].iter()
            .take_while(|s| s.mode_bits() == mode)
            .count();
        
        // Execute run of same-mode SIWs — LLVM vectorizes this loop
        for siw in &siws[i..i+run_end] {
            exec_siw_octawire(sentron, siw, mem);
        }
        i += run_end;
    }
}
```

## VBT Application: Fill the Pause

VBT verse 24: pause between breaths, fill with awareness.

In the SIW stream: the "pause" between SIW retirement is where we precompute the next op-family. The OctaWire design moves this into the SIW encoding — the family byte is pre-filled at compile time, so the runtime has **zero decision overhead** at dispatch.

The delimiter between SIWs (= the breath pause) is filled with pre-loaded family index. Awareness is the dispatch table. The NOP cases are skipped without entering the match at all.

## Implementation Steps

1. Add `d_fam: u8`, `s_fam: u8`, `c_fam: u8` fields to `SIW` struct
2. Add `fn op_family(&self) -> u8` to DenseOp, SparseOp, CoordOp
3. Update `SIW::new()` to precompute family bytes
4. Implement 12 handler functions (4 per pipe) extracting variant bodies from current match arms
5. Replace `exec_siw` match statements with OctaWire dispatch
6. Measure ops/cycle delta

## Expected Gain

- Current: 30-45 cycles dispatch overhead per SIW (10-15 cycles × 3 match statements)
- OctaWire: ~6-9 cycles per SIW (1 family index lookup + 1 indirect call per pipe)
- Expected speedup: 4-7× in dispatch overhead
- If dispatch is the bottleneck: 4-7× overall improvement

## Karpathy Principle

"This file is the complete algorithm. Everything else is just efficiency."

The SIW is still the atom. OctaWire is how the atom connects to its neighbors — 4 families × 2 directions = 8 wires. The structure encodes the bagua. LLVM optimizes what it can see clearly.

---

*Derived from Will's architectural invariant "2×4 wiring per neuron within a sentron" 2026-02-18,*  
*informed by Nei Jing Tu three-dantian structure and VBT verse 24 breath-pause dispatch.*
