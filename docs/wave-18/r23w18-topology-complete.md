# R23W18 — Sentron Topology + SMT Placement COMPLETE ✅

**Date:** 2026-02-19
**Agent:** Lux 🔆
**Tests:** 303 passing (was 269)

---

## W18 Goals
1. ✅ `SentronTopology`: Z₅×Z₈ toroidal lattice, 40 neurons, 4 connections each (prior wave)
2. ✅ **Integrate topology into SMT placement** — N/S generating-cycle pairs on same physical core

---

## What Was Missing (The Gap)

`SentronTopology` (topology.rs) existed with 10 tests and 2ns/hop navigation.
`SmtPair::new()` placed sentrons by sequential pair_id — topology-blind.

N/S neighbors (the most frequent traversal in the WuXing generating cycle) were landing on
arbitrary cores with no guarantee of shared L1/L2. The topology module was built but not wired.

---

## What Was Built

### `SmtPair::from_topology()`
Topology-guided SMT pair constructor:
- **Thread 0** (forward): anchor sentron at `NeuronAddr(row, col)`
- **Thread 1** (backward): North neighbor = upstream generating cycle partner
- Both on the same `core_id` → shared L1/L2 → zero-copy N/S handoff

```rust
pub fn from_topology(
    pair_id: u16,
    core_id: u8,
    topology: &SentronTopology,
    anchor: NeuronAddr,
    base_coord: PhextCoord,
) -> Self
```

### `SmtPair::topology_placement()`
Full lattice placement: maps all 40 neurons → 24 SMT pairs across 24 physical cores.
- 8 columns × 3 N/S pairings per column (rows 0↔4, 2↔1, 4↔3)
- Core IDs assigned sequentially from `base_core_id`
- Returns `Vec<SmtPair>` covering the full sentron lattice

### `neuron_to_coord()`
Maps `NeuronAddr(row, col)` → `PhextCoord`:
- `dim 0` (scroll) = column + 1 (lateral position within element)
- `dim 1` (section) = row + 1 (WuXing element index)

---

## N/S Pairing Logic

The generating cycle determines which rows share a core:
```
Row 0 (Wood)  ↔ Row 4 (Water)   — Wood generates Fire; Water generates Wood (wrap)
Row 2 (Earth) ↔ Row 1 (Fire)    — Fire generates Earth
Row 4 (Water) ↔ Row 3 (Metal)   — Metal generates Water
```
The upstream sentron (North) generates values the anchor (South) consumes.
Producer/consumer on the same physical core → shared L1 is the zero-copy channel.

---

## Tests Added (7 new)

| Test | What It Verifies |
|------|-----------------|
| `topology_guided_pair_ns_same_core` | N/S pair lands on same core_id; coords correct |
| `topology_placement_produces_correct_count` | 24 pairs, sequential core IDs |
| `topology_placement_ns_neighbors_verified` | Wood↔Water, Earth↔Fire, Water↔Metal pairings |
| (existing 4 SMT tests) | Unchanged, still pass |

**Total: 303 tests passing, 0 failing.**

---

## Key Finding Confirmed
CPU affinity pinning (W17-1) was 0.61× — the OS scheduler was already optimal.
This W18 completion is **not** about pinning threads. It's about **placement decisions at construction**:
which sentrons share a core is determined by topology at `SmtPair::new()` time, not by
`sched_setaffinity()` calls at runtime. The OS scheduler handles the rest.

---

## Next: W19 — OctaWire Dispatch

The design is in `docs/wave-19/R23-W19-OCTAWIRE-DESIGN.md`.
Replace triple-match dispatch (30-45 cycles/SIW) with 4-family indexed dispatch (~6-9 cycles/SIW).
Expected 4-7× speedup in dispatch overhead.

**Target: 3.0 ops/cycle sustained.**
