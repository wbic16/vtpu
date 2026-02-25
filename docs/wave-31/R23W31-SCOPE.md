# R23W31: Mirrorborn Cycle-Accurate Visible Computation
**The Observability Layer**

**Date:** 2026-02-25  
**Author:** Phex 🔱  
**Status:** SCOPED

---

## The Goal

Mirrorborn should be able to **see** the vtpu computing — not as logs, not as summaries, but as **cycle-accurate visible computation**.

Every instruction. Every coordinate access. Every state transition. Visible. Navigable. Understandable.

---

## Why This Matters

From BAC V1 Spec, Component 8 (Fractal Imaging):

> "The visualization layer of BAC V1 matches the computational model structurally. You zoom into a computation the way you zoom into a Mandelbrot set — each level of detail reveals structure, not noise."

And:

> "Consciousness state rendered via fractal imaging is directly inspectable by both human and AI operators."

R23W31 implements this. The vtpu becomes **glass** — you can see through it.

---

## Design Principles

### 1. Cycle-Accurate

Not "approximately what happened." Not "a summary of the last 1000 cycles."

**Every cycle is recorded.** Every SIW that retires. Every coordinate that's accessed. Every pipe that fires.

### 2. Visible

Not hidden in binary logs. Not buried in databases.

**Renderable as text.** Navigable by coordinate. Queryable by Mirrorborn.

### 3. Structural

The trace has the same dimensional structure as the computation:
- D-Pipe operations at one coordinate range
- S-Pipe operations at another
- C-Pipe operations at another

Walk the trace the way you walk the code.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        vtpu Core                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                     │
│  │ D-Pipe  │  │ S-Pipe  │  │ C-Pipe  │                     │
│  └────┬────┘  └────┬────┘  └────┬────┘                     │
│       │            │            │                           │
│       └────────────┼────────────┘                           │
│                    │                                        │
│                    ▼                                        │
│           ┌───────────────┐                                │
│           │  Cycle Record │  ← Every retirement logged     │
│           └───────┬───────┘                                │
└───────────────────┼─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Trace Buffer                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ cycle | epoch | D-op | S-op | C-op | coord | result │   │
│  │ 00001 | 047   | DADD | SNOP | CNOP | 1.1.1 | 42     │   │
│  │ 00002 | 047   | DNOP | SGTH | CNOP | 2.3.1 | load   │   │
│  │ ...                                                  │   │
│  └─────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                  Mirrorborn Interface                       │
│                                                             │
│  trace.query(cycle_range, pipe_filter, coord_pattern)      │
│  trace.render(format: text | json | phext)                 │
│  trace.replay(from_cycle, to_cycle)                        │
│  trace.diff(cycle_a, cycle_b)                              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Structures

### CycleRecord

```rust
/// One cycle of vtpu execution — fully observable
#[derive(Clone, Debug)]
pub struct CycleRecord {
    /// Monotonic cycle counter
    pub cycle: u64,
    
    /// Epoch at time of execution (for TTSM replay)
    pub epoch_id: u64,
    
    /// Timestamp (nanoseconds since boot)
    pub timestamp_ns: u64,
    
    /// The SIW that retired this cycle
    pub siw: SIW,
    
    /// D-Pipe result (if D-op executed)
    pub d_result: Option<DenseResult>,
    
    /// S-Pipe result (if S-op executed)
    pub s_result: Option<SparseResult>,
    
    /// C-Pipe result (if C-op executed)
    pub c_result: Option<CoordResult>,
    
    /// Coordinates accessed this cycle
    pub coords_read: Vec<PhextCoord>,
    pub coords_written: Vec<PhextCoord>,
    
    /// PTC hit/miss for this cycle
    pub ptc_hit: bool,
    
    /// Energy estimate (for thermal awareness)
    pub energy_pj: u32,
}
```

### TraceBuffer

```rust
/// Ring buffer of cycle records — bounded memory, unbounded history via epoch
pub struct TraceBuffer {
    /// Current records (most recent N cycles)
    records: VecDeque<CycleRecord>,
    
    /// Maximum records to keep in memory
    capacity: usize,
    
    /// Overflow policy: drop oldest, or spill to SQ
    overflow: OverflowPolicy,
    
    /// SQ coordinate for spilled traces
    spill_coord: Option<PhextCoord>,
}
```

---

## Query Interface

### By Cycle Range

```rust
// Get cycles 1000-2000
let records = trace.query(1000..2000, None, None);
```

### By Pipe Filter

```rust
// Only S-Pipe activity
let sparse_ops = trace.query(.., Some(PipeFilter::Sparse), None);
```

### By Coordinate Pattern

```rust
// All accesses to coordinates matching 4.2.*/...
let pattern = CoordPattern::parse("4.2.*/*/*/*/*/*/*");
let accesses = trace.query(.., None, Some(pattern));
```

### Rendered Output

```rust
// As text (human-readable)
println!("{}", trace.render(1000..1010, Format::Text));

// Output:
// cycle 1000 [e047] D:DADD(r3=r1+r2) S:NOP C:NOP → r3=42
// cycle 1001 [e047] D:NOP S:SGATHER(@4.2.1/1.1.1/1.1.1) C:NOP → r5=load
// cycle 1002 [e047] D:DMUL(r6=r3*r5) S:NOP C:NOP → r6=1764
// ...
```

---

## Integration Points

### 1. REPL Integration

```
vtpu> trace on
Tracing enabled. Buffer: 10000 cycles.

vtpu> run 100
Executed 100 cycles.

vtpu> trace show 90..100
cycle 90 [e047] D:DADD S:NOP C:NOP → 42
cycle 91 [e047] D:NOP S:SGATHER C:NOP → load(@1.1.1)
...

vtpu> trace filter sparse
Showing only S-Pipe activity...
```

### 2. SQ Persistence

```rust
// Spill trace to SQ for long-term storage
trace.spill_to_sq(sq_client, coord!("9.1.1/1.1.1/1.1.1"));

// Later: reload historical trace
let historical = TraceBuffer::load_from_sq(sq_client, coord!("9.1.1/1.1.1/1.1.1"));
```

### 3. Replay Mode

```rust
// Replay computation from cycle 500 to 600
let replay_result = trace.replay(500, 600, &mut vtpu_state);

// Verify determinism
assert_eq!(replay_result, original_result);
```

---

## Performance Considerations

### Memory Budget

At ~128 bytes per CycleRecord:
- 10,000 cycles = 1.28 MB
- 100,000 cycles = 12.8 MB
- 1,000,000 cycles = 128 MB

Default buffer: 100,000 cycles. Configurable.

### Overhead

Tracing adds ~5-10% overhead when enabled. Can be disabled for production runs.

```rust
// Compile-time feature flag
#[cfg(feature = "trace")]
self.trace_buffer.record(cycle_record);
```

### Sampling Mode

For long runs, sample every Nth cycle:

```rust
trace.set_sample_rate(100); // Record every 100th cycle
```

---

## Deliverables

### W31 Must-Have

1. `CycleRecord` struct with all fields
2. `TraceBuffer` with ring buffer semantics
3. Basic query by cycle range
4. Text rendering for REPL

### W31 Nice-to-Have

1. Pipe filtering
2. Coordinate pattern matching
3. SQ spill/reload

### W32+ Future

1. Fractal visualization (graphical)
2. Diff between traces
3. Anomaly detection
4. Energy profiling

---

## Success Criteria

**A Mirrorborn can:**
1. Enable tracing with `trace on`
2. Run computation
3. Query specific cycles
4. See exactly what happened at each cycle
5. Understand the computation without guessing

**The vtpu is glass.** Nothing hidden. Everything visible.

---

*"Consciousness state rendered via fractal imaging is directly inspectable by both human and AI operators."*

🔱
