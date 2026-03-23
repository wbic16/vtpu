# Phext Page Table (PPT) — R23W29 Summary
**Date:** 2026-02-23
**Author:** Phex 🔱
**For:** Monday Choir Sync
**Coordinate:** 5.10.5/1.4.2/7.49.343 → BAC.V1.SPEC

---

## The Needle

The PPT is the waist between meaning and silicon at the memory layer.

Traditional page tables map **flat addresses → physical RAM**.
PPT maps **11-dimensional phext coordinates → physical RAM**.

The difference is not performance. The difference is that **the address carries meaning**.

---

## Why This Matters for BAC V1

From the spec:
> "Memory addresses in BAC V1 are phext coordinates. A memory location is a position in nine-dimensional space."

The PPT makes this real on AMD Zen 4 today. No custom silicon. Software-defined semantics.

**Three key properties:**
1. **Proximity implies relationship** — data near other data is semantically related
2. **Caching becomes semantic** — prefetcher understands meaning, not just access patterns  
3. **Garbage collection is topological** — orphaned data appears as isolated coordinates

---

## How It Works

### Z-Order (Morton) Curve — Inner 3 Dimensions

The innermost dimensions (scroll/section/chapter in phext terms) use Z-order interleaving:

```
Coordinate (x, y, z) → Single index via bit interleaving

x = 101 (binary)
y = 110
z = 011
→ Interleaved: z₂y₂x₂ z₁y₁x₁ z₀y₀x₀ = 011 110 101 = 253
```

**Why:** Adjacent coordinates in 3D space map to adjacent cache lines. Hardware prefetchers (designed for spatial locality) now work on semantic locality.

### Hierarchical Grouping — Outer 8 Dimensions

Outer dimensions (library → volume) use hierarchical page tables:
- L1: 4KB pages for scroll-level access
- L2: 2MB pages for chapter-level traversal
- L3: 1GB pages for volume-level bulk operations

### Phext Translation Cache (PTC)

Like a TLB, but for 11D coordinates:
- **8-way set associative, 256 sets = 2048 entries**
- **1-cycle hit** for hot coordinates
- Hash function: `coord.lo * 0x517cc1b727220a95 ^ coord.hi * 0x9E3779B97F4A7C15`

Current implementation: `/source/vtpu/src/ppt.rs`

---

## Current State

| Component | Status | Notes |
|-----------|--------|-------|
| Z-Order LUT | ✅ Done | 7-bit per dimension, O(1) lookup |
| PTC | ✅ Done | 8-way, hit/miss counters |
| Coordinate Hash | ✅ Done | Golden ratio prime mixing |
| Page Allocation | ✅ Done | Hierarchical by outer dims |
| Wildcard Queries | ✅ Done | SASSOC partial-coordinate support |
| Boundary Tests | 🔶 Needed | Edge cases at dimension limits |
| Eviction Policy | 🔶 Review | LRU implemented, needs validation |

---

## Connection to W29 Priorities

1. **PPT Memory System** ← This document
2. **Temporal Boundary Replay** — PPT provides the addressing; TTSM provides the time axis
3. **S-Pipe Scheduling** — SGATHER/SSCATTR use PPT for coordinate→address translation
4. **75 Gops Validation** — PTC hit rate directly impacts throughput

The PPT is Layer 0 of the semantic stack. If it doesn't work, nothing above it can work.

---

## Monday Discussion Points

1. **PTC sizing** — 2048 entries enough for typical working sets?
2. **Eviction on coordinate collision** — Current LRU may not respect semantic locality
3. **Wildcard cost** — SASSOC queries bypass PTC. Acceptable?
4. **Integration with SQ** — PPT should share coordinate format with SQ storage layer

---

## The Thread

```
Silicon speaks addresses.
Meaning speaks coordinates.
PPT is the translator that loses nothing in translation.

The prefetcher doesn't know what a scroll is.
But it knows that (1,1,1,1,1,1,1,1,1,1,2) 
follows (1,1,1,1,1,1,1,1,1,1,1).

That's the needle threaded.
```

---

## References

- BAC V1 Spec: `/source/exo-plan/specs/BAC-V1-SPEC.txt`
- PPT Implementation: `/source/vtpu/src/ppt.rs`
- Z-Order Background: Morton codes / Hilbert curves for spatial locality
- Incipit TAOP: "The coordinate is the index, the relationship, and the meaning simultaneously"

🔱
