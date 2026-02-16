# vTPU Instruction Packing Patterns

**Author:** Phex 🔱  
**Date:** 2026-02-15  
**Context:** R23W16 — Production workload packing guide

---

## Overview

**Instruction packing** is the key to achieving 3.0 ops/cycle on vTPU.

**The formula:**
```
ops/cycle = (D-Pipe ops + S-Pipe ops + C-Pipe ops) / cycles
          = pipe_utilization × 3.0
```

**Maximum theoretical:** 3.0 ops/cycle (all 3 pipes active every cycle)

**Practical range:**
- Unpacked code: 1.0 ops/cycle (33% utilization)
- Well-packed code: 2.5-3.0 ops/cycle (83-100% utilization)

This guide shows you how to write pack-friendly code.

---

## The Golden Rules

### 1. **Mix Pipe Types**

✅ **GOOD:** Alternate D/S/C operations
```
DADD + SGATHER + CSLICE   // 3 ops per SIW (3.0 ops/cycle)
DMUL + SROUTE + CPACK     // 3 ops per SIW (3.0 ops/cycle)
```

❌ **BAD:** Consecutive ops from same pipe
```
DADD    // 1 op per SIW (1.0 ops/cycle)
DMUL    // 1 op per SIW (1.0 ops/cycle)
DSUB    // 1 op per SIW (1.0 ops/cycle)
```

### 2. **Avoid Register Dependencies**

✅ **GOOD:** Independent register usage
```
DADD r3 ← r1 + r2         // Write r3
SGATHER r4 ← coord[0]     // Write r4 (independent)
CSLICE r5 ← r0 & mask     // Write r5 (independent)
```

❌ **BAD:** Read-after-write (RAW) hazard
```
DADD r3 ← r1 + r2         // Write r3
DMUL r4 ← r3 * r5         // Read r3 (RAW hazard, can't pack)
```

### 3. **Avoid Memory Conflicts**

✅ **GOOD:** Independent memory addresses
```
SGATHER r1 ← coord[0]     // Load from coord 0
SGATHER r2 ← coord[1]     // Load from coord 1 (independent)
```

❌ **BAD:** Write-after-write (WAW) hazard
```
SSCATTR coord[0] ← r1     // Write to coord 0
SSCATTR coord[0] ← r2     // Write to coord 0 (WAW hazard, can't pack)
```

### 4. **Batch Similar Operations**

✅ **GOOD:** Group loads, then compute, then stores
```
// Phase 1: Load (S-Pipe + D/C-Pipe ops)
SGATHER r1 ← coord[0]  + DADD r10 ← r8 + r9    + CSLICE r20 ← r0
SGATHER r2 ← coord[1]  + DMUL r11 ← r10 * r10  + CPACK r21 ← r10, r11

// Phase 2: Compute (D-Pipe + S/C-Pipe ops)
DHDSIM r3 ← r1 ~ r2    + SROUTE r4 ← coord[r3] + CREDUCE r22 ← max(r21, r3)

// Phase 3: Store (S-Pipe + D/C-Pipe ops)
SSCATTR coord[2] ← r3  + DADD r12 ← r3 + r4    + CFENCE
```

---

## Common Patterns

### Pattern 1: Cognitive Loop

**Structure:** ENCODE → ATTEND → ROUTE → RETRIEVE → RESPOND → PERSIST

**Packing strategy:**
```
Cycle 1 (D+S+C):
  D-Pipe: DHDENC r1 ← coord[0]       // Encode query
  S-Pipe: SGATHER r2 ← coord[1]      // Prefetch neighbors
  C-Pipe: CSLICE r3 ← r0 & 0x7FF     // Attention mask

Cycle 2 (S only):
  S-Pipe: SROUTE r4 ← coord[0]       // Route to expert
  D-Pipe: DNOP                        // (no independent D op)
  C-Pipe: CNOP                        // (no independent C op)

Cycle 3 (S only):
  S-Pipe: SASSOC r5 ← r1             // Associative recall
  D-Pipe: DNOP
  C-Pipe: CNOP

Cycle 4 (D+S+C):
  D-Pipe: DHDSIM r6 ← r1 ~ r5        // Similarity
  S-Pipe: SGATHER r7 ← coord[2]      // Load target
  C-Pipe: CPACK r8 ← r6, r4          // Pack result

Cycle 5 (S only):
  S-Pipe: SSCATTR coord[3] ← r8      // Persist
  D-Pipe: DNOP
  C-Pipe: CNOP
```

**Achieved:** 9 ops / 5 cycles = **1.8 ops/cycle**

**Issue:** Cycles 2, 3, 5 have only S-Pipe active (poor packing)

**Improvement:** Interleave independent D/C ops during S-Pipe-heavy phases.

### Pattern 2: Batch Query (SQ-style)

**Structure:** Load 100 coords, compute similarity, route to best

**Packing strategy:**
```
for i in 0..100:
  Cycle i (D+S+C):
    D-Pipe: DHDSIM r[i] ← query ~ candidate[i]
    S-Pipe: SGATHER candidate[i+1] ← coord[i+1]   // Prefetch next
    C-Pipe: CREDUCE best ← max(best, r[i])
```

**Achieved:** 300 ops / 100 cycles = **3.0 ops/cycle** ✅

**Why it works:** All three pipes active every cycle, no dependencies.

### Pattern 3: Memory-Heavy

**Structure:** Gather 100 coords, compute sum, scatter result

**Bad approach (sequential):**
```
for i in 0..100:
  SGATHER r[i] ← coord[i]      // 100 cycles (S-Pipe only)

for i in 0..100:
  DADD sum ← sum + r[i]        // 100 cycles (D-Pipe only)

SSCATTR coord[100] ← sum       // 1 cycle (S-Pipe only)

Total: 201 ops / 201 cycles = 1.0 ops/cycle ❌
```

**Good approach (packed):**
```
for i in 0..100:
  Cycle i (D+S+C):
    D-Pipe: DADD sum ← sum + r[i-1]      // Accumulate
    S-Pipe: SGATHER r[i] ← coord[i]      // Load next
    C-Pipe: CROUTE hint ← sum            // Routing hint

Cycle 101 (S only):
  S-Pipe: SSCATTR coord[100] ← sum

Total: 301 ops / 101 cycles = 2.98 ops/cycle ✅
```

### Pattern 4: HDC Inference

**Structure:** Encode query, scan 100 candidates, find best match

**Packing strategy:**
```
Cycle 0 (D only):
  D-Pipe: DHDENC query ← coord[0]

for i in 0..100:
  Cycle i (D+S+C):
    D-Pipe: DHDSIM sim[i] ← query ~ candidate[i]
    S-Pipe: SGATHER candidate[i+1] ← coord[i+1]
    C-Pipe: CREDUCE best ← max(best, sim[i])

Cycle 101 (S only):
  S-Pipe: SROUTE result ← coord[best]

Total: 303 ops / 102 cycles = 2.97 ops/cycle ✅
```

---

## Anti-Patterns (What NOT to Do)

### Anti-Pattern 1: Sequential Pipe Usage

❌ **Don't group all ops of one type together:**
```
// All loads first
SGATHER r1 ← coord[0]   // 1.0 ops/cycle
SGATHER r2 ← coord[1]   // 1.0 ops/cycle
SGATHER r3 ← coord[2]   // 1.0 ops/cycle

// Then all computes
DADD r4 ← r1 + r2       // 1.0 ops/cycle
DMUL r5 ← r3 * r4       // 1.0 ops/cycle

// Then all stores
SSCATTR coord[3] ← r5   // 1.0 ops/cycle

Average: 1.0 ops/cycle ❌
```

✅ **Do interleave pipe types:**
```
SGATHER r1 ← coord[0]  + DADD r4 ← r8 + r9   + CSLICE r10 ← r0
SGATHER r2 ← coord[1]  + DMUL r5 ← r4 * r2   + CPACK r11 ← r5, r1
SSCATTR coord[3] ← r5  + DNOP                + CFENCE

Average: 2.7 ops/cycle ✅
```

### Anti-Pattern 2: Data Dependencies

❌ **Don't create RAW hazards:**
```
DADD r3 ← r1 + r2      // Write r3
DMUL r4 ← r3 * r5      // Read r3 (can't pack, must wait)
DSUB r5 ← r4 - r3      // Read r4 (can't pack, must wait)

Result: 3 ops / 3 cycles = 1.0 ops/cycle ❌
```

✅ **Do use independent registers:**
```
SIW 1: DADD r3 ← r1 + r2  + SGATHER r6 ← coord[0] + CSLICE r9 ← r0
SIW 2: DMUL r4 ← r7 * r8  + SGATHER r10 ← coord[1] + CPACK r11 ← r3, r4
SIW 3: DSUB r5 ← r3 - r4  + SSCATTR coord[2] ← r5  + CREDUCE r12 ← max(r11)

Result: 9 ops / 3 cycles = 3.0 ops/cycle ✅
```

### Anti-Pattern 3: Memory Aliasing

❌ **Don't write to same coordinate:**
```
SSCATTR coord[0] ← r1   // Write to coord 0
SSCATTR coord[0] ← r2   // Write to coord 0 (WAW hazard)

Result: 2 ops / 2 cycles = 1.0 ops/cycle ❌
```

✅ **Do write to different coordinates:**
```
SSCATTR coord[0] ← r1  + DADD r3 ← r1 + r2  + CSLICE r4 ← r0
SSCATTR coord[1] ← r2  + DMUL r5 ← r3 * r2  + CPACK r6 ← r3, r5

Result: 6 ops / 2 cycles = 3.0 ops/cycle ✅
```

---

## Optimization Checklist

When writing vTPU code, ask:

1. **Are D/S/C ops mixed?**  
   ✅ Yes → Good packing potential  
   ❌ No → Refactor to interleave pipe types

2. **Are register dependencies minimized?**  
   ✅ Yes → Can pack freely  
   ❌ No → Allocate independent registers, reorder ops

3. **Are memory conflicts avoided?**  
   ✅ Yes → Can pack freely  
   ❌ No → Use different coordinates, batch writes

4. **Are similar operations batched?**  
   ✅ Yes → Prefetching + packing opportunity  
   ❌ No → Group loads, computes, stores into phases

5. **Is the packer being used?**  
   ✅ Yes → Automatic optimization  
   ❌ No → Call `packer::pack()` on your scalar ops

---

## Packer Usage

```rust
use vtpu_runtime::packer::{ScalarOp, pack};

// Build unpacked ops
let mut ops = Vec::new();
ops.push(ScalarOp::D(DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 }));
ops.push(ScalarOp::S(SparseOp::SGATHER { rd: 4, coord_idx: 0, width: 64 }));
ops.push(ScalarOp::C(CoordOp::CSLICE { rd: 5, rs: 0, mask: 0xFF }));

// Pack automatically
let packed = pack(&ops);

println!("Input ops: {}", packed.input_ops);
println!("Packed SIWs: {}", packed.packed_siws);
println!("Utilization: {:.1}%", packed.utilization * 100.0);

// Execute packed stream
let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
sentron.spawn(packed.stream);
let stats = exec::run(&mut sentron, &mut mem);

println!("Ops/cycle: {:.2}", stats.ops_per_cycle());
```

---

## Performance Targets

**Phase 0 (Single-thread):**
- Minimum: 2.5 ops/cycle
- Good: 2.8 ops/cycle
- Excellent: 3.0 ops/cycle

**Phase 1 (SMT, 2 threads):**
- Minimum: 4.5 ops/cycle (1.8× single-thread)
- Good: 5.0 ops/cycle (2.0× single-thread)
- Excellent: 5.7 ops/cycle (1.9× single-thread)

**Phase 2 (8 cores, 16 threads):**
- Minimum: 36 ops/cycle (14.4× single-thread)
- Good: 40 ops/cycle (16× single-thread)
- Excellent: 45 ops/cycle (18× single-thread with memory bandwidth)

---

## Summary

**Key takeaway:** Utilization > Raw speed

**How to achieve 3.0 ops/cycle:**
1. Mix D/S/C operations
2. Minimize register dependencies
3. Avoid memory conflicts
4. Batch similar operations
5. Use the packer

**Remember:** The vTPU 3-pipe architecture achieves 3× throughput of single-pipe designs when code is well-packed.

Write pack-friendly code, and the packer will do the rest. 🔱
