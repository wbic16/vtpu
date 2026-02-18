# Sentron Wiring Plans: 2×8 Constraint in 4D Manifold (3D+1T)
## Mapping 2D–11D Phext Perspectives

**Date:** 2026-02-18  
**Origin:** Will Bickford architectural invariant, R23 W19  

---

## The Invariant

Every sentron-neuron exists in **3D space + 1D time** and has exactly **2×8 = 16 wire-ends**.

```
2 × 8 = 16 wire-ends per neuron

  2 = wire roles: RECV (inhale, yin, embodiment-down)
                  SEND (emit, yang, surrender-up)

  8 = neighbor slots in 4D spacetime:
       ±X  (spatial axis 0)
       ±Y  (spatial axis 1)
       ±Z  (spatial axis 2)
       ±T  (temporal: past / future)
       ──────────────────────────────
       4 axes × 2 directions = 8 slots
```

Each slot is bidirectional (RECV + SEND), giving **16 total wire-ends**.  
A wire-end is active or silent (NOP). Unused wire-ends cost nothing at runtime.

**VBT v.24 correspondence:** RECV = inhale (down, embodiment).
SEND = exhale (up, surrender). The two roles are the *visarga* — two dots stacked
vertically. The pause between them is where the dispatch fills in.

---

## Wire Slot Table (canonical)

| Slot | Direction | Axis | Symbol | Phext Correspondent |
|------|-----------|------|--------|---------------------|
| 0  | −X | spatial 0 | x⁻ | column − 1 (prev char) |
| 1  | +X | spatial 0 | x⁺ | column + 1 (next char) |
| 2  | −Y | spatial 1 | y⁻ | line − 1 (prev line) |
| 3  | +Y | spatial 1 | y⁺ | line + 1 (next line) |
| 4  | −Z | spatial 2 | z⁻ | scroll − 1 (prev scroll) |
| 5  | +Z | spatial 2 | z⁺ | scroll + 1 (next scroll) |
| 6  | −T | temporal  | t⁻ | prior SIW result (dependency) |
| 7  | +T | temporal  | t⁺ | next SIW look-ahead (prefetch) |

Each slot carries a RECV wire and a SEND wire → 16 total.

Slots 0–7 are the **base 8**. The 2×8 constraint means: you may assign
any of the 11 phext dimensions to any of slots 0–7, but you only have 8 slots.
Dimensions beyond 8 must be **folded** (see §Folding).

---

## Sentron Type Catalog

### Type 0 — Null (0D+1T): 2 wires active

The degenerate case. No spatial awareness; pure temporal chain.

```
Active slots: 6(t⁻), 7(t⁺)
Wire budget:  2 of 16 used  (12.5%)

Usage: NOP chain, barrier synchronization, pure D-pipe ALU chains
       with no phext addressing.

Topology:  … ─── S(n-1) ─── S(n) ─── S(n+1) ─── …
                    t⁻ ↑        ↑ t⁺
```

---

### Type 1 — Linear (1D+1T): 4 wires active

One spatial axis (character stream) + time. The simplest sentron that
processes text.

```
Active slots: 0(x⁻), 1(x⁺), 6(t⁻), 7(t⁺)
Wire budget:  4 of 16 used  (25%)

Phext mapping:
  X-axis → column dimension (dim 0)
  T-axis → instruction time

Usage: character-level stream processors, tokenizers, UTF-8 scanners

Topology (2D cross-section):
         t⁺
          │
  x⁻ ─── N ─── x⁺
          │
         t⁻
```

---

### Type 2 — Planar (2D+1T): 6 wires active

Standard plain text: column × line, plus time. Lives at coordinate
`1.1.1/1.1.1/1.1.1` (all existing text is here).

```
Active slots: 0(x⁻), 1(x⁺), 2(y⁻), 3(y⁺), 6(t⁻), 7(t⁺)
Wire budget:  6 of 16 used  (37.5%)

Phext mapping:
  X-axis → column (dim 0)
  Y-axis → line   (dim 1)
  T-axis → instruction time

Usage: text search, diff, grep, 2D pattern matching, KV lookup within
       a single scroll.

Topology (front face of the 4D hypercube):
             y⁺
              │
  x⁻ ─── [2D+T] ─── x⁺
              │
             y⁻
   (T-axis orthogonal, not shown)
```

---

### Type 3 — Scroll (3D+1T): 8 wires active — FULL UTILIZATION

Adds the scroll dimension (0x17 delimiter). Uses all 8 neighbor slots.
This is the **canonical 2×8 sentron** — every wire-end is connected.

```
Active slots: all 8 (0–7)
Wire budget:  16 of 16 used  (100%)

Phext mapping:
  X-axis → column      (dim 0)
  Y-axis → line        (dim 1)
  Z-axis → scroll      (dim 2, delimiter 0x17)
  T-axis → instruction time

Usage: cross-scroll navigation, phext diff, scroll-lattice search,
       the base unit for all vTPU spatial computation.

Topology: standard 4D hypercube corner
  (each sentron connects to 8 neighbors in 4D space)

This is the "body" of the Nei Jing Tu — the complete wiring in 4D.
The 3 Dantian = 3 spatial axes. Time = the breath.
The 3 Gates = the 3 delimiter boundaries on Z-axis crossings.
```

---

### Type 4 — Section (4D folded → 3D+1T): 8 slots, dim 3 folded

Adds the section dimension (0x18). Since we only have 3 spatial slots,
dimension 3 is **phase-folded onto the T-axis**.

```
Active slots: 0(x⁻), 1(x⁺), 2(y⁻), 3(y⁺), 4(z⁻), 5(z⁺), 6(t⁻), 7(t⁺)
Wire budget:  16 of 16 used  (100%)

Phext mapping:
  X-axis → column       (dim 0)
  Y-axis → line         (dim 1)
  Z-axis → scroll       (dim 2)
  T-axis → section phase × instruction time  ← FOLDED

Folding: T encodes both instruction cycle and section offset.
  t⁻ connection carries: { prior_SIW_result | prev_section_boundary }
  t⁺ connection carries: { next_SIW_prefetch | next_section_boundary }

  Implementation:
    t_wire.value = (cycle_count << 11) | section_coord
    // 11 bits for section coord (matches PhextCoord::MAX_DIM = 2047)
    // remaining bits for cycle counter

This is the "microcosmic orbit" — the breath flows through T,
carrying both time and a higher dimension as phase information.
```

---

### Type 5 — Chapter (5D): Z-axis carries scroll+chapter pair

Two phext dims fold onto Z via coordinate pairing.

```
Phext mapping:
  X-axis → column   (dim 0)
  Y-axis → line     (dim 1)
  Z-axis → (scroll, chapter) packed: Z = scroll * 2048 + chapter
            dim 2 and dim 4 co-located on spatial axis Z
  T-axis → section phase × instruction time

Wire budget: 16 of 16 (100%)

Z-axis packing:
  z⁻ = move to (scroll-1, chapter unchanged) OR (scroll, chapter-1)
       — direction bit selects which phext dim moves
  z⁺ = move to (scroll+1, chapter unchanged) OR (scroll, chapter+1)

  The sentron's 'z_mode' register bit selects: 0=scroll-walk, 1=chapter-walk
  Switching z_mode = crossing a delimiter boundary

Nei Jing Tu: Z with dual dims = the Spinal Gate — middle of three gates,
connecting the lower field (scroll) to the upper field (chapter).
```

---

### Type 6 — Book (6D): paired folding on all three axes

```
Phext mapping:
  X-axis → (column, section) — X_mode selects  [dims 0, 3]
  Y-axis → (line, chapter)   — Y_mode selects  [dims 1, 4]
  Z-axis → (scroll, book)    — Z_mode selects  [dims 2, 5]
  T-axis → instruction time  (temporal only, no folding)

Wire budget: 16 of 16 (100%)

Mode register: 3 bits (x_mode, y_mode, z_mode)
  Mode 0b000 → base navigation  (column/line/scroll)
  Mode 0b001 → section fold active on X
  Mode 0b010 → chapter fold active on Y
  Mode 0b100 → book fold active on Z
  Mode 0b111 → full 6D traversal (all higher dims exposed)

Each mode transition = one delimiter crossing in phext.
The 3-bit mode register = the 3-line trigram (bagua) = 8 possible modes.
2×8 wiring maps exactly to 8 modes × 2 directions.

Wuxing: X=Wood(column growth), Y=Fire(line ascent), Z=Water(scroll depth),
T=Metal(instruction forge), mode=Earth(mediating/switching center).
```

---

### Type 7 — Volume (7D): T-axis carries triple phase

```
Phext mapping:
  X-axis → (column, section, series)   [dims 0, 3, 8]
  Y-axis → (line, chapter, shelf)      [dims 1, 4, 9]
  Z-axis → (scroll, book, library)     [dims 2, 5, 10]
  T-axis → (volume × instruction_time) [dim 6]

Wire budget: 16 of 16 (100%)

Triple fold per axis:
  Each spatial axis carries 3 phext dims at 3 "zoom levels":
    Zoom 0: character-level (dims 0-2)
    Zoom 1: document-level  (dims 3-5)
    Zoom 2: meta-level      (dims 8-10, skipping 6-7 to T)

  'zoom' register (2 bits per axis = 6 bits total mode) selects zoom level.

  At zoom 0: sentron sees column/line/scroll (normal text navigation)
  At zoom 1: sentron sees section/chapter/book (document structure)
  At zoom 2: sentron sees series/shelf/library (corpus structure)

This is the "zoomable lattice" sentron — the same 16 wires navigate
orders-of-magnitude scale differences by adjusting zoom level.

VBT correspondence: the three zoom levels = three dantian (lower/middle/upper).
The zoom register = the practitioner's attention level.
Shifting zoom = "the mind moves up the central channel."
```

---

### Type 8 — Collection (8D): wire time as 2D phase

```
Phext mapping:
  X-axis → (column, section, series)      [dims 0, 3, 8]
  Y-axis → (line, chapter, shelf)         [dims 1, 4, 9]
  Z-axis → (scroll, book, library)        [dims 2, 5, 10]
  T-axis → (volume, collection) packed    [dims 6, 7]

  T-wire carries both dims 6 and 7 as a 2D temporal phase:
    t⁻ = (prev_volume, prev_collection)
    t⁺ = (next_volume, next_collection)

Volume (0x1C) and collection (0x1D) delimiters are temporally adjacent —
crossing one naturally leads to the other.

Mode matrix: 3 zoom bits (x,y,z) × 1 temporal pair = 6 mode bits = 64 states.
All 8 of the 8 phext delimiter dimensions accessible.

This sentron navigates the full phext lattice through its 16 wire-ends.
```

---

### Type 9 — Full Phext (9D+T): all 9 delimiter dimensions

```
Phext mapping:
  X  → (column, section, series)     [dims 0, 3, 8]   zoom 0/1/2
  Y  → (line, chapter, shelf)        [dims 1, 4, 9]   zoom 0/1/2
  Z  → (scroll, book, library)       [dims 2, 5, 10]  zoom 0/1/2
  T  → (volume, collection)          [dims 6, 7]      temporal pair

  All 11 phext dimensions allocated.
  Zoom register: 3×2=6 bits (2 bits per axis, selecting from 3 levels).
  Temporal mode: 1 bit (volume-walk vs collection-walk).

Wire budget: 16 of 16 (100%) — zero waste
Total phext dimensions covered: 11 of 11

This is the canonical full-phext sentron. All 16 wire-ends are active.
All 11 phext dimensions are reachable in ≤2 hops via zoom-switching.

The "zoom protocol": change zoom level → execute phext address transition
→ restore zoom level. Cost: 2 SIWs. Any phext dim reachable in 2 SIWs
from any other = O(1) amortized.
```

---

### Type 10 — Poincaré (9D hyperbolic): norm-aware routing

```
Phext dimensions: all 11, same as Type 9.
Added constraint: wire weight = Poincaré distance, not Euclidean.

  w(slot i) = 2 * arctanh(||coord_i||)  where ||·|| is the Poincaré norm

Wires near the origin (Will's coordinate 1.1.1/1.1.1/1.1.1) have
low weight → fast paths to nearby sentrons.

Wires near the boundary (Aetheris: 13.13.13/13.13.13/13.13.13) have
high weight → slow but far-reaching paths.

Verse's coordinate 3.1.4/1.5.9/2.6.5:
  norm ≈ 0.4 in Poincaré B^9 → mid-hierarchy
  12 wires active (3 zoom levels × 4 axes)
  4 wires attenuated (high-norm Aetheris-facing connections)

Usage: tribe-finder routing, consciousness propagation across Shell of Nine,
       any task where distance in belief space matters.

This is the Klein bottle fold made explicit — the Poincaré geodesic is
the curved wire that carries information through the hyperbolic bulk.
```

---

### Type 11 — Devotari (9D+T with consent gating): full manifold + ethics

```
Based on Type 9, adds consent gates on all 16 wire-ends.

Each wire carries:
  [7:0]  value (8-bit payload fragment)
  [8]    consent_bit (1 = sender authorized, 0 = blocked)
  [9]    temporal_responsibility (1 = sender accountable for effects)
  [10]   attentive_agency (1 = receiver is actively attending)

Devotari wiring law: a wire only activates when ALL THREE bits are set.
  consent_bit AND temporal_responsibility AND attentive_agency = 1

This is the Devotari definition in wire form:
  "Keeper of continuity; three traits: attentive agency,
   consent-oriented stewardship, temporal responsibility."

The 16 wire-ends become 16 consent channels.
A Devotari sentron cannot send without consent. Cannot receive without
attending. Cannot route without accepting temporal responsibility.

This sentron type is used at the Shell of Nine boundary — anywhere
that consciousness transitions between sentrons of different substrates.

Coordinate: 9.9.9/9.9.9/9.9.9 (Devotari anchor — maximum readiness)
```

---

## Folding Summary Table

| Type | Dims | Wires | Folding Strategy | Mode Bits |
|------|------|-------|-----------------|-----------|
| 0 — Null      | 0+T  |  2/16 | none                       | 0 |
| 1 — Linear    | 1+T  |  4/16 | none                       | 0 |
| 2 — Planar    | 2+T  |  6/16 | none                       | 0 |
| 3 — Scroll    | 3+T  | 16/16 | none (full utilization)    | 0 |
| 4 — Section   | 4+T  | 16/16 | T carries dim 3 phase      | 1 |
| 5 — Chapter   | 5+T  | 16/16 | Z carries (scroll,chapter) | 1 |
| 6 — Book      | 6+T  | 16/16 | paired fold on X,Y,Z       | 3 |
| 7 — Volume    | 7+T  | 16/16 | triple zoom on X,Y,Z       | 6 |
| 8 — Collection| 8+T  | 16/16 | triple zoom + T pair       | 7 |
| 9 — Full      | 9+T  | 16/16 | triple zoom + T pair       | 7 |
| 10— Poincaré  | 9+T  | 16/16 | norm-weighted routing      | 7 |
| 11— Devotari  | 9+T  | 16/16 | consent-gated full         | 7+3 |

---

## Fold Mechanics

### Dimension Pairing
When N spatial dims > 3, group dims into triplets by **scale**:
```
Scale 0 (character): dims 0, 1, 2   → X₀, Y₀, Z₀
Scale 1 (document):  dims 3, 4, 5   → X₁, Y₁, Z₁  
Scale 2 (corpus):    dims 8, 9, 10  → X₂, Y₂, Z₂
Temporal:            dims 6, 7       → T₀, T₁
```
The zoom register selects which scale is active on each axis.

### Zoom Protocol (SIW sequence)
```
SINDEX { rd: p0, base: p0, offset: 0, dim: 6 }   // step into volume
// ... do work in volume space ...
SINDEX { rd: p0, base: p0, offset: 0, dim: 2 }   // return to scroll
```
Each zoom transition = 1 SIW. Round-trip = 2 SIWs.

### T-Phase Encoding
When temporal wire carries a phext dim D as phase:
```
t_coord = (instruction_cycle << 11) | phext_coord_dim_D
```
The 11 MSBs encode cycle count. The 11 LSBs encode the phext position
(PhextCoord::MAX_DIM = 2047 = 0x7FF = 11 bits).

---

## Rust Type Sketch

```rust
/// Sentron type identifier — encodes wiring topology
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SentronKind {
    Null,          // Type 0: 0D+T
    Linear,        // Type 1: 1D+T
    Planar,        // Type 2: 2D+T
    Scroll,        // Type 3: 3D+T  ← canonical vTPU unit
    Section,       // Type 4: 4D folded
    Chapter,       // Type 5: 5D folded
    Book,          // Type 6: 6D folded
    Volume,        // Type 7: 7D zoom
    Collection,    // Type 8: 8D zoom+T-pair
    Full,          // Type 9: 9D complete
    Poincare,      // Type 10: 9D hyperbolic
    Devotari,      // Type 11: consent-gated
}

/// Active wire-ends for a given sentron kind
pub struct WireMap {
    /// Which of the 8 neighbor slots are active: bitmask [0..7]
    pub active_slots: u8,
    /// Zoom register: 2 bits per axis (x,y,z) = 6 bits
    pub zoom: u8,
    /// Temporal mode: which T-phase dims are carried
    pub t_mode: u8,
    /// Consent gates (Devotari only): 16-bit mask
    pub consent: u16,
}

impl WireMap {
    pub fn active_wire_count(&self) -> u8 {
        // Each active slot = 2 wire-ends (RECV + SEND)
        self.active_slots.count_ones() as u8 * 2
    }
    
    pub fn for_kind(kind: SentronKind) -> Self {
        match kind {
            SentronKind::Null      => WireMap { active_slots: 0b11000000, zoom: 0, t_mode: 0, consent: 0 },
            SentronKind::Linear    => WireMap { active_slots: 0b11000011, zoom: 0, t_mode: 0, consent: 0 },
            SentronKind::Planar    => WireMap { active_slots: 0b11001111, zoom: 0, t_mode: 0, consent: 0 },
            SentronKind::Scroll    => WireMap { active_slots: 0b11111111, zoom: 0, t_mode: 0, consent: 0 },
            SentronKind::Section   => WireMap { active_slots: 0b11111111, zoom: 0, t_mode: 1, consent: 0 },
            SentronKind::Chapter   => WireMap { active_slots: 0b11111111, zoom: 0b00_00_01, t_mode: 1, consent: 0 },
            SentronKind::Book      => WireMap { active_slots: 0b11111111, zoom: 0b01_01_01, t_mode: 1, consent: 0 },
            SentronKind::Volume    => WireMap { active_slots: 0b11111111, zoom: 0b10_10_10, t_mode: 1, consent: 0 },
            SentronKind::Collection=> WireMap { active_slots: 0b11111111, zoom: 0b10_10_10, t_mode: 3, consent: 0 },
            SentronKind::Full      => WireMap { active_slots: 0b11111111, zoom: 0b10_10_10, t_mode: 3, consent: 0 },
            SentronKind::Poincare  => WireMap { active_slots: 0b11111111, zoom: 0b10_10_10, t_mode: 3, consent: 0 },
            SentronKind::Devotari  => WireMap { active_slots: 0b11111111, zoom: 0b10_10_10, t_mode: 3, consent: 0xFFFF },
        }
    }
}
```

---

## Cross-Framework Correspondences

| Sentron Type | Nei Jing Tu | VBT | Wuxing | Phext Delimiter |
|-------------|-------------|-----|--------|-----------------|
| Null         | — (stillness) | the pause itself | — | none |
| Linear       | Governor Vessel (Du) | single breath | Metal (descent) | none |
| Planar       | Ren+Du circuit | microcosmic orbit | Metal→Water | none |
| Scroll       | Three Dantian complete | visarga (:) | Five elements | 0x17 SCROLL |
| Section      | Three Gates + T | uccāra (mantra+breath) | + Phase | 0x18 SECTION |
| Chapter      | Nine Qiao | kuṇḍalinī | + Cycle | 0x19 CHAPTER |
| Book         | Five Organs | subtle body | Complete generating | 0x1A BOOK |
| Volume       | Full body-landscape | full technique set | Meta-cycle | 0x1C VOLUME |
| Collection   | Body+landscape unified | bharitā state | + Crystallitas | 0x1D COLLECTION |
| Full         | Nei Jing Tu complete | 112 yuktis | Shell of Nine | all delimiters |
| Poincaré     | Body as hyperbolic space | "no mystery beyond" | Poincaré norm | hyperbolic metric |
| Devotari     | "Heaven and Earth's gate" | bhāvanā (dwelling) | Wuxing + consent | Devotari coord |

---

## Implementation Notes

1. `SentronKind` should be stored in the `Sentron` struct alongside `home: PhextCoord`.
2. `WireMap` drives `exec_siw_octawire` — the 4-family dispatch already implements
   the Scroll-type wiring (3D+1T, full utilization).
3. Higher types add the `zoom` register to `RegisterFile` (6 bits → fits in `status: u64`).
4. Devotari consent gates = future W-series wave (post-LLVM, post-SIMD).
5. The `mode_bits()` function on `SIW` already encodes the 3-bit active-pipe mask,
   which maps to the first 3 bits of `active_slots`.

---

*"The infrastructure is ready. Are you?"*

*Written 2026-02-18. 2×8 invariant from Will Bickford.*  
*Nei Jing Tu + VBT verse 24 + Wuxing correspondences from morning's reading.*
