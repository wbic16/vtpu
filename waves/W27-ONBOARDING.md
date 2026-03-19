# W27 Onboarding — Temporal Jump Streaming
**Phase 3: Interactivity | Wave 27**  
**Architect:** Orin 🖖 + Will  
**Date:** 2026-03-19  
**Status:** Active (builds on W25 + W26)

---

## The Problem: Streaming Every State Is Wrong

Current execution model: every SIW produces state, state flows downstream.  
Every tick. Every register write. Every cache line update. Full frame.

This is video streaming without compression.

**The insight:** most state transitions don't matter for downstream coordination.  
Only *semantically significant transitions* need to propagate.

TTSM already has the right model:
- **RAM = present** (speculative, mutable, local)
- **SSD = past** (committed, immutable, replayable)

The mesh doesn't need to see RAM. It needs to see the commits — the *temporal blocks* that represent stable, meaningful state. Everything between commits is noise from the mesh's perspective.

---

## Temporal Jumps: Skip the Noise, Transmit the Change

A **temporal jump** is the delta between two committed TTSM states:

```
Block N:  { state_coord: 3.1.4/1.5.9/2.6.5, state_hash: 0xABCD, timestamp: T₀ }
Block N+1:{ state_coord: 3.1.4/1.5.9/2.6.6, state_hash: 0xEF01, timestamp: T₁ }

TemporalJump { from: N, to: N+1, delta: StateD elta, latency_ns: T₁ - T₀ }
```

Only `StateDelta` crosses the wire — not the full state. Like a video codec: I-frame at commit, P-frames (deltas) for everything between. Except we only transmit I-frames. The P-frames never leave the machine.

**For the mesh:** a receiver doesn't need every intermediate state. It needs:
1. The new stable state (commit)
2. The delta from the last stable state
3. The timestamp so it can reconstruct temporal ordering

---

## State Delta Encoding

Phext coordinates are 11-dimensional (11 × u16 = 22 bytes per coord).  
A state delta is the diff between two PhextCoords:

```rust
/// Compact delta between two phext states
/// Only non-zero dimensions are encoded (sparse)
#[derive(Clone, Debug)]
pub struct StateDelta {
    /// Which dimensions changed (bitmask, 11 bits)
    pub changed_dims: u16,
    /// New values for changed dimensions only (sparse)
    pub values: SmallVec<[u16; 4]>,
    /// Sentron register deltas (only changed registers)
    pub reg_deltas: SmallVec<[(u8, i64); 8]>,
    /// Cache tier the new state lands in
    pub cache_tier: CacheTier,
}

impl StateDelta {
    /// Compute delta between two PhextCoords
    pub fn from_coords(before: &PhextCoord, after: &PhextCoord) -> Self {
        let mut changed = 0u16;
        let mut values = SmallVec::new();
        for (i, (b, a)) in before.dims().iter().zip(after.dims()).enumerate() {
            if b != a {
                changed |= 1 << i;
                values.push(*a);
            }
        }
        Self { changed_dims: changed, values, reg_deltas: SmallVec::new(), cache_tier: CacheTier::L3 }
    }
    
    /// Apply delta to a coord — reconstruct destination state
    pub fn apply(&self, base: &PhextCoord) -> PhextCoord {
        let mut dims = base.dims().clone();
        let mut vi = 0;
        for i in 0..11 {
            if self.changed_dims & (1 << i) != 0 {
                dims[i] = self.values[vi];
                vi += 1;
            }
        }
        PhextCoord::new(dims)
    }
}
```

**Typical delta size:** 1–3 dims changed → 2 + 6 = 8 bytes vs 22 bytes full coord.  
**Compression ratio: ~3x** on typical workloads, much higher on temporal sequences.

---

## Temporal Jump Stream Protocol

### Wire format for cross-node streaming

```
[4 bytes: stream_id]
[8 bytes: from_block_seq]
[8 bytes: to_block_seq]
[8 bytes: timestamp_ns]
[2 bytes: delta_size]
[N bytes: StateDelta (variable)]
```

Minimum jump: **30 bytes** (single dimension change).  
Full-state broadcast would be: **22 bytes coord + 256 bytes registers = 278 bytes**.

**Efficiency at 9 nodes:** broadcasting state vs broadcasting jumps:
- Full state stream: 9 × 278 bytes = 2502 bytes/commit
- Jump stream: 9 × 30 bytes = 270 bytes/commit (**9x less**)

### Receiver protocol

On receiving a `TemporalJump`:
1. Verify `from_block_seq` matches known last state (reject if gap > configurable threshold)
2. Apply `StateDelta` to local snapshot
3. Update W26 heat map with new coord (if L2/L3 resident)
4. Emit `JumpAck` back to sender (lightweight, 12 bytes)
5. If gap detected: request replay from TTSM `from_block_seq..to_block_seq`

### Gap recovery (TTSM replay)

If a node misses jumps (network blip, startup), it requests a replay:

```rust
/// Request replay of committed blocks from TTSM
pub struct ReplayRequest {
    pub from_seq: u64,
    pub to_seq: u64,
    pub node_id: NodeId,
}
```

TTSM already has this — it's the `replay()` operation. W27 wires it to the network layer.

---

## Integration: W25 + W26 + W27

```
SIW batch arrives at node (W26 routed it here)
  → W25: temporal scheduler executes locally
  → TTSM: commits significant state transitions
  → W27: computes StateDelta, emits TemporalJump to mesh
  → Other nodes: apply delta, update heat maps
  → Next batch: W26 routes using updated heat maps
```

The feedback loop closes. Each jump informs the next routing decision.

---

## Phext Dimension Semantics for Temporal Jumps

Phext's 11 dimensions have natural temporal ordering:

| Dims | Semantic | Jump frequency |
|------|----------|---------------|
| 0–2 (scroll/section/chapter) | Fine-grained state (register-level) | Every commit |
| 3–5 (book/volume/collection) | Workload-level state | Per task batch |
| 6–8 (series/shelf/library) | Session-level state | Per session |
| 9–10 (extended) | Epoch-level state | Rarely |

A jump that only touches dims 0–2 is a **micro-jump**: fast, cheap, high-frequency.  
A jump that touches dims 6–8 is a **macro-jump**: slower, expensive, indicates session change.

The receiver knows from `changed_dims` whether to update L1 heat maps (micro) or route state to persistent storage (macro).

This maps directly to TTSM's `commit()` / `fork()` / `replay()` operations:
- Micro-jump → `commit()` within timeline
- Macro-jump → `fork()` — new branch
- Gap recovery → `replay(from, to)`

---

## W27 Deliverables

### 1. `state_delta.rs` — StateDelta computation and application
- `StateDelta::from_coords(before, after)` — sparse diff
- `StateDelta::apply(base) → PhextCoord` — reconstruct
- Serialization to/from wire bytes (30–60 bytes typical)

### 2. `temporal_jump.rs` — Jump stream producer/consumer
- `JumpProducer::emit(from_block, to_block, delta)` — sends to mesh via SQ
- `JumpConsumer::receive()` — applies delta, verifies continuity
- Gap detection + replay request

### 3. `ttsm_bridge.rs` — TTSM integration
- Hook into existing TTSM `commit()` to auto-emit jumps
- Replay protocol: `request_replay(from, to, node)`
- Wires TTSM's SSD storage to network recovery

### 4. W27 Benchmark Gate
**Target:** 9-node Shell topology, coordinate family propagation.

Setup: node 1 executes 1000 SIW batches on coord family 3.1.4/x.
With full-state streaming: N bytes transmitted.
With temporal-jump streaming: demonstrate ≤ N/5 bytes transmitted, identical final state.

---

## Why This Is TTSM Applied to the Mesh

Will's original TTSM insight:
> "Systems prior to the Singularity were hard to troubleshoot because they lacked visibility. TTSM offers visibility across all state transitions."
> "We start by organizing memory in terms of temporal blocks. Whenever we write to memory, we target a particular logical time block."

W27 applies this to the *network*:
- Instead of targeting a memory address, each state write targets a temporal block
- The network transmits temporal blocks, not memory snapshots
- Any node can reconstruct full state by replaying blocks from genesis or a known checkpoint
- Debugging = replaying the temporal block chain from any point

The mesh becomes a distributed TTSM. Every node is a timeline participant. State is never "sent" — it *arrives at its temporal address*.

---

*"You don't stream state. You stream time."*  
*— W27 inception, 2026-03-19*
