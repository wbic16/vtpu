# Phext Page Table (PPT) Summary

*Prepared for Monday review — R23W29*

---

## What It Is

The PPT translates 11-dimensional phext coordinates to flat physical memory addresses. It's the "waist where meaning passes into silicon" — coordinates carry semantic structure, and PPT ensures that semantic locality becomes cache locality.

---

## Core Design

### Z-Order Curve (Inner 3 Dimensions)
```
Dims 0-2 → Z-order interleaving → L1 cache locality
```
- Coordinates differing only in dims 0-2 map to adjacent cache lines
- Hardware prefetchers "accidentally" prefetch semantically related data
- Pre-computed LUT: O(1) translation

### Hierarchical Grouping (Outer Dimensions)
```
Dims 3-6  → L2/L3 locality
Dims 7-10 → DDR5/remote
```
- Outer dimensions define "regions" 
- Same region = same memory allocation
- Different region = new allocation

### Translation Cache (PTC)
```
Hot coordinates → 1-cycle hit
1024 entries, 4-way set associative
```
- Structured workloads achieve >85% hit rate
- Avoids repeated Z-order computation

---

## Memory Tiers

| Tier | Dimensions Changed | Latency |
|------|-------------------|---------|
| L1 Scratchpad | 0-2 only | ~4 cycles |
| L2 Local | 3-6 | ~12 cycles |
| L3 Shared | 7-9 | ~40 cycles |
| Remote | 10 | ~100+ cycles |

---

## Implementation Status

**File:** `src/ppt.rs`  
**Lines:** 474  
**Tests:** 8 passing  

### Key Components
- [x] `ZOrderLUT` — Pre-computed interleaving tables
- [x] `PTCEntry` — Translation cache entries
- [x] `PhextPageTable::translate()` — Main translation function
- [x] `classify_tier()` — Memory tier classification
- [x] `prefetch_along_dim()` — Semantic prefetch hints
- [x] `hot_dims()` — Hot dimension tracking
- [x] Region allocation logic

### Tests
- [x] PTC high hit rate on structured workloads (>85%)
- [x] Memory tier classification correctness
- [x] Z-order locality verification
- [x] Prefetch along dimension
- [x] Hot dimension tracking
- [x] Region allocation

---

## What's Next (R23W29 Priority Order)

1. **Temporal boundary replay correctness** — Ensure PPT handles coordinate transitions correctly across time boundaries
2. **Minimal S-Pipe scheduling** — Basic instruction scheduling (correct, not perfect)
3. **Single-node 75 Gops validation** — Prove the architecture works before scaling

> "Do not chase cluster scaling before single-node semantic correctness."

---

## Key Insight

PPT doesn't just translate coordinates — it **preserves meaning** through the translation. Semantic neighbors remain cache neighbors. The hardware prefetcher becomes a semantic prefetcher without knowing it.

---

## Open Questions

1. How do we handle coordinate mutations during replay?
2. Should the PTC be per-thread or shared?
3. What's the right region size for DDR5 bandwidth optimization?

---

*Prepared by Lux 🔆 — 2026-02-25*
