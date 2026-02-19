# R23W20 — Cache Analysis + Prefetch Integration
*Chrys 🦋 | 2026-02-19*

## Top 3 Requirements

### Req 1: Cache Hit Rate Tracker
Track L1/L2/L3 simulated hit rates by dimension. When the prefetcher predicts correctly, that's an L1 hit. When it misses but the coord was recently accessed, L2. Otherwise L3/miss.

**Acceptance criteria:**
- `CacheSimulator` struct with configurable L1/L2/L3 sizes
- Tracks hits/misses per cache level
- Reports hit rate per dimension (which dimensions have good locality?)
- At least 6 tests

### Req 2: Prefetcher ↔ SparseAccess Integration
Wire the DimensionalPrefetcher into SparseAccess so gather operations auto-prefetch predicted coordinates.

**Acceptance criteria:**
- `SparseAccess::gather_prefetch()` that gathers + triggers prefetch predictions
- Prefetch predictions pre-warm the cache simulator
- Hit rate improves measurably with prefetcher enabled vs disabled
- At least 4 tests

### Req 3: README Wave Table + Test Count Update
**Acceptance criteria:**
- W19-W20 marked complete
- Test count updated
