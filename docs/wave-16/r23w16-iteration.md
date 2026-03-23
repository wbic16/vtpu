# R23W16 Iteration — Sync Examples with Implementation

**Date:** 2026-02-15 (evening)  
**Contributor:** Phex 🔱  
**Issue:** Keeping theory in sync as we revise

---

## Problem

Original PACKING-PATTERNS.md guide used fictional operation signatures that didn't match actual vTPU implementation in `pipes.rs`.

**Examples of mismatches:**
```
GUIDE (fictional):           ACTUAL (pipes.rs):
CSLICE { rd, rs, mask }      CSLICE { group, dim_triple, range_start, range_end }
SROUTE { coord_idx, hint }   SROUTE { rd, embedding_reg, dim_mask }
SASSOC { query_reg, mode }   SASSOC { rd, coord_reg, match_mode }
CROUTE { rd, rs }            CROUTE { msg_reg, dest_node }
```

**Impact:**
- Examples wouldn't compile if copy-pasted
- Theory diverged from implementation
- Misleading for developers learning vTPU

---

## Solution

Rewrote all examples in PACKING-PATTERNS.md to use actual operations from `pipes.rs`:

### Updated Operations

**D-Pipe (Dense):**
- DADD, DSUB, DMUL ✓ (unchanged)
- DHDSIM { rd, rs1, rs2 } ✓ (correct syntax)
- DHDENC { rd, rs, width } ✓ (correct syntax)

**S-Pipe (Sparse):**
- SGATHER { rd, coord_idx, width } ✓
- SSCATTR { coord_idx, rs, width } ✓
- SASSOC { rd, coord_reg, match_mode } ✓ (fixed)
- SROUTE { rd, embedding_reg, dim_mask } ✓ (fixed)

**C-Pipe (Coordination):**
- CPACK { rd, rs1, rs2, fmt: MessageFormat } ✓ (fixed)
- CREDUCE { rd, rs, op: ReductionOp, group } ✓ (added group param)
- CROUTE { msg_reg, dest_node } ✓ (fixed)
- CCAST { rs, group } ✓ (fixed)
- CFENCE { scope: FenceScope } ✓ (correct syntax)

### Updated Patterns

**Pattern 1: Simple Pipeline**
- Before: Used CSLICE with wrong params
- After: Uses CPACK with correct MessageFormat param

**Pattern 2: Batch Query**
- Before: CREDUCE without group param
- After: CREDUCE { rd, rs, op, group: 0 }

**Pattern 3: Memory-Heavy**
- Simplified to realistic load-accumulate-store
- No fictional operations

**Pattern 4: HDC Inference**
- Before: SROUTE with wrong params
- After: SROUTE { rd, embedding_reg, dim_mask }

---

## Validation

**Compilation:** ✅ All examples compile  
**Execution:** ✅ w16_packing_demo runs successfully  
**Demo output:**
```
Bad packing (sequential):  1.0 ops/cycle
Good packing (mixed):      2.0 ops/cycle
Auto packer:               1.17 ops/cycle (from 7 unpacked ops)
```

---

## Files Changed

**Modified:**
- `docs/wave-16/PACKING-PATTERNS.md` (87 insertions, 57 deletions)

**Result:**
- All 10 pattern examples now use real vTPU operations
- Packer usage example compiles correctly
- Theory synchronized with implementation ✅

---

## Lesson Learned

**"Keep the project in sync as we revise theory"**

When implementation evolves:
1. Check actual operation signatures in `pipes.rs`
2. Update all documentation examples to match
3. Test that examples compile and run
4. Validate demo outputs match expectations

**Don't let theory drift from reality.**

Documentation is code. Keep it compiling. 🔱

---

**Commit:** `1086ccf`  
**Status:** SYNCED ✅
