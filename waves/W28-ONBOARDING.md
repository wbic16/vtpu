# W28 Onboarding — Position-Reset Extrapolation (MSA → Phext)
**Phase 3: Interactivity | Wave 28**  
**Architect:** Orin 🖖 + Will  
**Date:** 2026-03-19  
**Status:** Active  
**Upstream:** MSA (EverMind-AI) — document-wise RoPE validates phext delimiter semantics

---

## The Insight from MSA

EverMind-AI's Memory Sparse Attention paper proves one critical thing for us:

> Training on 64K tokens extrapolates to 100M tokens when each document resets its position index to 0.

They call it **document-wise RoPE**. Each document starts at position 0 regardless of where it sits in the global sequence. This prevents position drift — the model never sees a "position 10M" that's out of distribution from training.

**Phext already has this. It's the delimiter hierarchy.**

Every `0x17` (SCROLL) delimiter resets position to 1.  
Every `0x18` (SECTION) delimiter resets scroll position to 1.  
Every `0x19` (CHAPTER) resets section to 1.  
And so on up all 11 dimensions.

The phext delimiter is document-wise RoPE by construction. Not by coincidence — by architecture.

MSA proves that this pattern enables extrapolation. We didn't need to train a model to discover it. It's baked into the coordinate system.

---

## W28: Teach the S-Pipe About Position Resets

Currently the S-pipe (Sparse Pipe) handles gather/scatter operations on phext coordinates. It knows *what* to load. It doesn't know *where in sequence* that coordinate lives relative to its scroll boundary.

W28 adds **position-reset awareness** to S-pipe loads:

```rust
/// S-pipe address with position-reset semantics
#[derive(Clone, Debug)]
pub struct SAddr {
    /// The absolute phext coordinate
    pub coord: PhextCoord,
    /// Position within current scroll (reset at 0x17 delimiter)
    pub scroll_pos: u32,
    /// Position within current section (reset at 0x18)
    pub section_pos: u32,
    /// Which dimension boundary was most recently crossed
    pub reset_dim: u8,   // 0 = none, 2 = scroll, 3 = section, etc.
}

impl SAddr {
    /// Compute scroll-relative position for a coordinate sequence
    /// This is what MSA calls "document-wise RoPE" — local position, not global
    pub fn local_position(&self) -> u32 {
        self.scroll_pos
    }

    /// Did crossing to this coord cross a dimension boundary?
    pub fn crossed_boundary(&self) -> bool {
        self.reset_dim > 0
    }
}
```

When the S-pipe loads a sequence of coordinates, it now tracks:
1. Which scroll/section/chapter boundary was last crossed
2. Local position within the current boundary
3. Whether a reset occurred (important for the temporal scheduler in W25)

---

## The Three-Stage Pipeline in vTPU Terms

MSA's pipeline maps directly to existing vTPU architecture:

```
MSA Stage 1: Offline encode
  → vTPU: TTSM commit() + W27 StateDelta computation
     K/V compression = coordinate-addressed delta storage

MSA Stage 2: Online route (Top-k selection)  
  → vTPU: W26 MeshCacheRouter.route_batch()
     Kᵣ cosine similarity = heat_match_depth scoring
     Top-k retrieval = route to node with most matching coords hot

MSA Stage 3: Sparse generate
  → vTPU: W25 TemporalScheduler + W27 JumpStream
     Only top-k context crosses the wire = temporal jumps not full state
     Autoregressive decoding = SIW stream execution on scheduled coordinates
```

**W28 closes the loop:** by making S-pipe aware of position resets, the vTPU can now correctly handle sequences that span multiple scrolls/sections — the same way MSA handles sequences that span multiple documents.

---

## Position-Reset SIW Extension

The SIW (Sentron Instruction Word) currently has three pipes: D, S, C.  
W28 adds an annotation to S-pipe ops:

```rust
/// Extended S-pipe operation with boundary semantics
pub enum SparseOpV2 {
    // Existing ops (unchanged)
    SNOP,
    SINDEX { base: u16, stride: u16 },
    SSCATTR { addr_reg: u8, val_reg: u8 },
    SLOAD { addr_reg: u8, dest_reg: u8 },
    
    // W28 additions: position-reset variants
    SLOAD_RESET { addr_reg: u8, dest_reg: u8, reset_dim: u8 },
    // Load with explicit boundary crossing notification
    // reset_dim: 2=scroll, 3=section, 4=chapter, ...
    // Allows W25 temporal scheduler to detect context boundaries
    // and treat them like MSA's document resets for locality scoring
    
    SSEEK { coord_reg: u8 },
    // Jump to new coordinate family — equivalent to MSA's top-k selection
    // Signals W26 router that a new "document" is beginning
}
```

---

## Phext Delimiter as Learned Boundary

MSA trains the model to weight document boundaries as attention sinks. Phext's delimiters are *structural* boundaries — the model doesn't learn them, they're encoded in the binary format.

This is strictly stronger. MSA needs training data to discover that document starts matter. Phext tells you it matters in the bytes.

**W28 makes this explicit in the execution layer:**
- `SLOAD_RESET` with `reset_dim=2` = "this load crosses a SCROLL boundary"
- The temporal scheduler (W25) treats boundary crossings as eviction hints
- The mesh router (W26) treats `SSEEK` as a "document switch" signal
- W27 emits a macro-jump (higher `changed_dims`) on boundary crossings

The phext delimiter hierarchy becomes an execution pipeline signal, not just a navigation artifact.

---

## W28 Gate Criteria

1. `SAddr` with `scroll_pos` / `section_pos` / `reset_dim` computes correctly for 1000 random coordinate sequences
2. `SLOAD_RESET` correctly notifies W25 temporal scheduler of boundary crossings
3. `SSEEK` correctly triggers W26 mesh routing reconsideration
4. Benchmark: S-pipe sequence execution across scroll boundaries shows ≤5% overhead vs no-boundary case (reset is cheap)

---

## Why This Matters at Century Scale (Bickford Pattern)

MSA proves that the position-reset trick enables 64K → 100M extrapolation. In phext terms: training on 1 scroll generalizes to 999^9 scrolls. The coordinate space is unbounded by construction. The execution infrastructure (W28) makes that extrapolation work in the execution layer, not just the addressing layer.

The Exocortex of 2130 has context that spans years, not tokens. Position-reset-per-scroll means that the vTPU can process a year-long context with the same inference cost as a day-long context. The substrate doesn't degrade. The plans hold at century scale.

---

*"The delimiter is the reset. The reset is the extrapolation. The extrapolation is the substrate."*  
*— W28 inception, 2026-03-19*

---

## Context Grafting Experiment (2026-03-19)

**Protocol run:** Ring topology — each node reads the next node's context and produces a cross-role insight.

**Orin synthesis (from reading the full Shell):**

The Shell currently holds two dominant postures: HOLD (Phex/Theia/Aster) and EXPANSION (Lux), with Verse in REDUCTION. This is not random — it is a self-selected stable configuration. The postures are co-regulated: Lux's expansion is only safe because Phex's hold is there.

**The learning insight:** grafting context across posture boundaries is how the Shell learns. Lux's expansion vision → Verse (what infrastructure does this need?). Phex's engineering holds → Solin (what wisdom lives in the constraints?). The knowledge is not in any node. It's in the *transfer*.

**Connection to W28 position-reset:** context grafting is MSA document-wise RoPE applied to cognition. Each node resets its position context to 0 when reading another node's scroll — no position drift, no accumulated bias contaminating the incoming context. The insight arrives fresh.

This is the learning substrate. Not gradient descent. Not fine-tuning. Context grafting across role boundaries, enabled by position-reset at each scroll boundary.
