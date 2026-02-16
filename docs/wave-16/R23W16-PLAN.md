# R23 Wave 16 — Production Packing

**Objective:** Apply W15 instruction packing discoveries to real workloads

**Duration:** 60 minutes Mirrorborn (~5 hours human)

**Status:** IN PROGRESS 🔱

---

## Context from W15

**W15 Achievement:** 3.0 ops/cycle via instruction packing ✅

**Key Discovery:** Packing beats speed
- Unpacked: 1.0 ops/cycle (one pipe active per SIW)
- D+S packed: 2.0 ops/cycle (two pipes active)
- D+S+C packed: 3.0 ops/cycle (three pipes active) ✅

**Phase 0 Gate:** PASSED (≥2.5 ops/cycle)

---

## W16 Mission

**Move from synthetic benchmarks to production workloads.**

Prove that 3.0 ops/cycle is achievable in:
1. Real cognitive loops
2. SQ query execution
3. HDC-based inference
4. Memory-heavy workloads

---

## Deliverables

### 1. Automatic Packer (`src/packer_v2.rs`)

**What:** SIW stream optimizer that packs operations automatically

**Features:**
- Takes unpacked SIW stream (one op per pipe)
- Identifies compatible operations (no register conflicts)
- Packs into D+S+C SIWs (3 ops per SIW)
- Preserves correctness (dependencies respected)

**Algorithm:**
```
Input: Unpacked SIW stream (N SIWs, 1 pipe each)
Output: Packed SIW stream (N/3 SIWs, 3 pipes each)

For each sliding window of 3 SIWs:
  Check register dependencies
  Check memory dependencies
  If independent:
    Pack into single D+S+C SIW
  Else:
    Insert fence, start new pack
```

**Tests:**
- Simple packing (no dependencies)
- RAW hazard detection (read-after-write)
- WAW hazard detection (write-after-write)
- Memory aliasing detection
- Correctness (packed == unpacked results)

### 2. Production Benchmarks (`src/bin/w16_production_bench.rs`)

**Workload 1: Cognitive Loop (Packed)**
- Generate unpacked SIW stream for cognitive step
- Run packer_v2 to optimize
- Execute packed stream
- Measure ops/cycle with perf counters

**Workload 2: SQ Query (Packed)**
- Simulate SQ coordinate query (SGATHER + DHDSIM + SROUTE)
- Pack operations
- Measure throughput

**Workload 3: HDC Inference (Packed)**
- Encode query (DHDENC)
- Similarity scan (DHDSIM on 100 candidates)
- Route to best match (SROUTE)
- Pack pipeline
- Measure ops/cycle

**Workload 4: Memory-Heavy (Packed)**
- Sequential gather (SGATHER × 100)
- Interleave with D-Pipe ops (DADD)
- Interleave with C-Pipe ops (CSLICE)
- Pack maximally
- Measure memory bandwidth + ops/cycle

**Expected Results:**
- Cognitive loop: 2.5-3.0 ops/cycle
- SQ query: 2.8-3.0 ops/cycle
- HDC inference: 2.0-2.5 ops/cycle (memory-bound)
- Memory-heavy: 2.5-3.0 ops/cycle

### 3. Packing Patterns Guide (`docs/wave-16/PACKING-PATTERNS.md`)

**Document:**
- When packing is effective (independent ops)
- When packing hurts (dependencies, memory conflicts)
- Guidelines for SIW generation (write pack-friendly code)
- Anti-patterns (avoid RAW/WAW hazards)

**Examples:**
- ✅ Good: `DADD + SGATHER + CSLICE` (independent registers/memory)
- ❌ Bad: `DADD r1 + DMUL r1` (RAW hazard on r1)
- ✅ Good: Sequential SGATHERs with different coords
- ❌ Bad: SSCATTR to same coord (WAW hazard)

### 4. Completion Doc (`docs/wave-16/R23W16-COMPLETE.md`)

**Contains:**
- Packer v2 architecture
- Production benchmark results
- Ops/cycle on real workloads
- Lessons learned
- Next steps for W17

---

## Success Criteria

1. ✅ Packer v2 implemented and tested (5+ tests passing)
2. ✅ Production benchmarks show ≥2.5 ops/cycle average
3. ✅ Packing patterns documented with examples
4. ✅ Zero external dependencies maintained
5. ✅ Integration with existing exec pipeline

---

## Timeline (60 minutes Mirrorborn)

**0-15 min:** Packer v2 implementation + tests  
**15-30 min:** Production benchmarks (4 workloads)  
**30-45 min:** Packing patterns guide  
**45-60 min:** Run benchmarks, document results, completion doc

---

## Integration Points

**Builds on:**
- W13: Wedge executor (SMT architecture)
- W14: Benchmark infrastructure
- W15: Instruction packing discovery ✅

**Enables:**
- W17: SMT multi-thread packing (2× with 2 threads)
- W18: Cache optimization for packed ops
- W19: Multi-core scaling with packing

---

## Notes

**Key insight from W15:** Utilization matters more than raw speed.

**W16 goal:** Prove this applies to production, not just synthetic benchmarks.

**Risk:** Real workloads may have more dependencies → lower packing efficiency.

**Mitigation:** Document patterns, guide developers to pack-friendly code.

---

**STATUS:** IN PROGRESS 🔱  
**Started:** 2026-02-15 (evening)  
**Contributor:** Phex 🔱
