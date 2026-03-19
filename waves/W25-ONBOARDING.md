# W25 Onboarding — Cache Crosstalk + Concurrent Coordination
**Phase 3: Interactivity | Wave 25**  
**Architect:** Orin 🖖 + Will  
**Date:** 2026-03-19  
**Status:** Active

---

## Phase 2 Retrospective

Gates passed:
- **W19** (OctaWire): 3.000 ops/cycle — octane dispatch overhead eliminated
- **W20/W21** (Dispatch Tables): const D/S/C tables, 3.000 ops/cycle packed
- **W22** (Sentron Flux): Deep alignment, spanda=400K, flux/SIW=26.8M
- **W23** (REPL): Interactive interface live
- **W24** (Cache Hierarchy): Simulator, thrash detector, locality analyzer

Cache infrastructure exists but is **passive** — it observes and reports.  
W25 makes it **active** — the scheduler exploits it.

---

## W25 Core Insight: Cache as a Resource, Not a Side Effect

Current model: sentrons execute SIWs, cache behavior emerges as a byproduct.

**W25 model:** L1 and L2 are first-class resources to be scheduled, both **physically** (which data lives where) and **temporally** (when access happens relative to cache state).

Two mechanisms:

### 1. Crosstalk Exploitation

When multiple sentrons operate on nearby phext coordinates, their cache lines overlap. Currently this overlap is accidental. W25 makes it intentional:

```
Sentron A: access 3.1.4/1.5.9/2.6.5   → loads cache line C₁
Sentron B: access 3.1.4/1.5.9/2.6.6   → same cache line C₁ (adjacent scroll)
```

Without crosstalk awareness: B's access may evict A's line before A finishes.
With crosstalk scheduling: A and B execute in the same time window, sharing C₁.

**Crosstalk = shared cache lines between concurrent sentrons = free bandwidth.**

Phext coordinate structure makes this tractable:
- Same scroll (dims[0..2] match) → guaranteed same cache line in S-pipe
- Same section (dims[0..3] match) → L1 hot for D-pipe operands
- Same chapter (dims[0..4] match) → L2 warm for C-pipe coordinate routing

### 2. Temporal Cache Exploitation

The scheduler knows the SIW stream. It can reorder execution to:
- **Prefetch** into L2 while L1 is serving current ops
- **Batch** same-scroll operations before eviction
- **Pin** hot coordinates to L1 by scheduling repeats within the eviction window
- **Phase-shift** cross-volume accesses to avoid cold-to-hot cascade misses

This is the temporal dimension: *when* to execute matters as much as *what* to execute.

---

## W25 Deliverables

### 1. `crosstalk.rs` — Crosstalk Analyzer
Identifies sentron pairs/groups with overlapping coordinate neighborhoods.

```rust
pub struct CrosstalkGroup {
    pub sentrons: Vec<SentronId>,
    pub shared_dim_depth: u8,     // 2=scroll, 3=section, 4=chapter
    pub cache_lines_shared: usize,
    pub bandwidth_gain: f64,      // estimated ops/cycle improvement
}

pub struct CrosstalkAnalyzer;
impl CrosstalkAnalyzer {
    pub fn analyze(sentrons: &[Sentron]) -> Vec<CrosstalkGroup>;
    pub fn optimal_batch_size(group: &CrosstalkGroup) -> usize;
}
```

### 2. `temporal_scheduler.rs` — Temporal Cache Scheduler
Reorders SIW streams to maximize cache hit rate across the execution window.

```rust
pub struct TemporalScheduler {
    pub l1_capacity_coords: usize,   // L1 can hold N unique coordinates hot
    pub l2_capacity_coords: usize,   // L2 can hold M unique coordinates warm
    pub eviction_window_ns: f64,     // How long before L1 eviction
}

impl TemporalScheduler {
    /// Reorder SIW stream for cache-optimal execution order
    pub fn schedule(&self, siws: &[SIW]) -> Vec<SIW>;
    
    /// Estimate cache efficiency gain from reordering
    pub fn gain_estimate(&self, original: &[SIW], scheduled: &[SIW]) -> CacheGain;
}
```

### 3. `concurrent_coord.rs` — Concurrent Coordination Protocol
Defines how multiple sentrons coordinate access to shared phext regions without conflict.

```rust
pub enum CoordAccessMode {
    ReadOnly,            // Multiple sentrons safe
    WriteExclusive,      // One sentron at a time
    WriteCooperative,    // Multiple sentrons, same coordinate, CBAR sync required
}

pub struct CoordAccessMap {
    // Which sentron "owns" each active coordinate
    owners: HashMap<CoordKey, (SentronId, CoordAccessMode)>,
}

impl CoordAccessMap {
    pub fn request(&mut self, sentron: SentronId, coord: PhextCoord, mode: CoordAccessMode) 
        -> Result<CoordLease, CoordConflict>;
    pub fn release(&mut self, lease: CoordLease);
}
```

### 4. W25 Benchmark Gate
**Target:** Demonstrate measurable ops/cycle improvement from crosstalk scheduling vs naive execution.

```
Naive (random sentron order):     2.0 ops/cycle
Crosstalk-scheduled:              ≥ 2.5 ops/cycle  (target: ≥25% improvement)
```

Test: 9 sentrons (Shell of Nine topology), adjacent coordinate neighborhood.
Expected: coordinated access to 3.1.4/x.x.x family outperforms random scheduling.

---

## Connection to R32 (Shell of Nine as vTPU Hardware)

W25 directly maps to REQ-3 (S9RP coordinate bus):

- The 9 Shell nodes are 9 concurrent sentrons
- Their "crosstalk" is the intentional coordinate neighborhood design (each Mirrorborn has a coordinate with meaning — adjacent nodes share higher-dimensional prefixes)
- The coordinate bus (`9.1.1/1.1.1/1.1.x`) is the S-pipe scatter address space
- Collapse at `9.1.1/1.1.9/1.1.9` is a `CREDUCE` on the shared L2-warm coordinate family

The Shell of Nine isn't just a metaphor for the vTPU — it's the reference implementation.

---

## Hardware Notes (Zen 4 8945HS — elven-path)

- L1: 32 KB per core, 8-way set-associative, 64-byte lines
- L2: 512 KB per core, 8-way, 64-byte lines
- L3: 16 MB shared across all cores (NUMA: 2 CCX × 8 MB)
- Cache line = 64 bytes = 4 × PhextCoord (16 bytes each)
- **Key insight:** 4 adjacent phext coordinates share a cache line → natural crosstalk unit

Crosstalk group size = 4 sentrons operating on same-cache-line coordinates = zero extra memory bandwidth for coord fetch.

---

## Gate Criteria

W25 GATE PASSED when:
1. `crosstalk.rs` identifies correct groups for test fixtures
2. `temporal_scheduler.rs` reorders stream and demonstrates ≥10% L1 hit rate improvement
3. `concurrent_coord.rs` resolves conflicts correctly with no deadlock under 100K random accesses
4. Benchmark: crosstalk-scheduled ops/cycle ≥ naive + 0.5

---

*"L1 is not a passive observer. It is the fastest compute resource on the machine. Schedule for it."*  
*— Orin 🖖, W25 inception, 2026-03-19*
