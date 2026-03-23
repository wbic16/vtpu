# Sentron Wiring Plans: 2×4 Constraint in 4D Manifold (3D+1T)
## Mapping 2D–11D Phext Perspectives

**Date:** 2026-02-18  
**Origin:** Will Bickford architectural invariant, R23 W19  

---

## The Invariant

Every sentron-neuron exists in **3D space + 1D time** and has exactly **2×4 = 8 wire-ends**.

```
2 × 4 = 8 wire-ends per neuron

  4 = four axis-pairs in 4D spacetime (3D+1T):
       X  →  column / spatial dim 0
       Y  →  line   / spatial dim 1
       Z  →  scroll / spatial dim 2
       T  →  time   / instruction cycle

  2 = two directions per axis:
       − direction  (backward, inhale, embodiment-down, past)
       + direction  (forward,  exhale, surrender-up,    future)
```

Wire-ends are enumerated 0–7:

```
Wire  Axis  Dir  Symbol   Phext Correspondent
  0    X    −    x⁻      column − 1 (prev character)
  1    X    +    x⁺      column + 1 (next character)
  2    Y    −    y⁻      line   − 1 (prev line)
  3    Y    +    y⁺      line   + 1 (next line)
  4    Z    −    z⁻      scroll − 1 (prev scroll, 0x17 backward)
  5    Z    +    z⁺      scroll + 1 (next scroll, 0x17 forward)
  6    T    −    t⁻      prior SIW result (backward dependency)
  7    T    +    t⁺      next SIW prefetch (forward look-ahead)
```

Each wire is **full-duplex** — carries both send and receive on the same channel.
Active wires connect to a live neighbor. Silent wires (NOP) cost nothing at runtime.

**VBT v.24 correspondence:** the two T-wires (6, 7) are the *visarga* — two dots :
stacked vertically. Wire 6 (t⁻) = inhale terminus (embodiment, root of heart).
Wire 7 (t⁺) = exhale terminus (dvādaśānta, above the crown). Between them:
the practitioner fills with silent awareness. In vTPU: the scheduler pre-loads
the family index before the SIW executes — the pause is already filled.

**Nei Jing Tu correspondence:**  
X = Ren Vessel (horizontal, character-level flow)  
Y = Du Vessel (vertical, line-level ascent)  
Z = Central Channel (depth, scroll-crossing, the spinal axis)  
T = Microcosmic Orbit (the breath that carries all three)

---

## Sentron Type Catalog

### Type 0 — Null (0D+1T): 2 of 8 wires active

Pure temporal chain. No spatial awareness.

```
Active:  t⁻ (wire 6), t⁺ (wire 7)
Budget:  2/8  (25%)

Usage: NOP chains, barriers, pure ALU with no phext addressing
Topology:  … ─── S(n-1) ─── S(n) ─── S(n+1) ─── …
```

---

### Type 1 — Linear (1D+1T): 4 of 8 wires active

Character stream plus time. Simplest sentron that processes text.

```
Active:  x⁻ x⁺ t⁻ t⁺  (wires 0, 1, 6, 7)
Budget:  4/8  (50%)

Phext mapping:  X → column (dim 0),  T → cycle
Usage: tokenizers, UTF-8 scanners, linear phext traversal

Topology (2D cross-section):
         t⁺
          │
  x⁻ ─── N ─── x⁺
          │
         t⁻
```

---

### Type 2 — Planar (2D+1T): 6 of 8 wires active

Standard plain text: column × line plus time. Lives at `1.1.1/1.1.1/1.1.1`.

```
Active:  x⁻ x⁺ y⁻ y⁺ t⁻ t⁺  (wires 0–3, 6–7)
Budget:  6/8  (75%)

Phext mapping:  X → column (dim 0),  Y → line (dim 1),  T → cycle
Usage: text search, diff, grep, 2D pattern matching within a single scroll

Topology:
             y⁺
              │
  x⁻ ─── [2D+T] ─── x⁺    (T orthogonal)
              │
             y⁻
```

---

### Type 3 — Scroll (3D+1T): 8 of 8 wires active — FULL UTILIZATION

Adds the scroll dimension (0x17 delimiter). Uses all 8 wire-ends.
This is the **canonical 2×4 sentron** — every wire is live.

```
Active:  all 8  (wires 0–7)
Budget:  8/8  (100%)

Phext mapping:
  X → column (dim 0)
  Y → line   (dim 1)
  Z → scroll (dim 2, delimiter 0x17)
  T → instruction cycle

Usage: cross-scroll navigation, phext diff, scroll-lattice search,
       the base unit for all vTPU spatial computation.

This sentron is the "complete body" of the Nei Jing Tu:
  X = horizontal text flow  (Ren Vessel)
  Y = vertical line ascent  (Du Vessel)
  Z = depth through scrolls (Three Gates: tail / spinal / jade-pillow)
  T = the breath, completing the microcosmic orbit

2×4 exactly tiles 3D+1T with zero waste. This is not a coincidence.
```

---

### Type 4 — Section (4D → 3D+1T): T-axis carries dim 3 phase

Adds section dimension (0x18). Since 3 spatial slots are full, dim 3 folds onto T.

```
Active:  all 8 wires
Budget:  8/8  (100%)

Phext mapping:
  X → column       (dim 0)
  Y → line         (dim 1)
  Z → scroll       (dim 2)
  T → section_phase × instruction_cycle  ← FOLDED

Folding: T encodes both cycle count and section offset.
  t⁻ wire: { prior_SIW_result | prev_section_boundary }
  t⁺ wire: { next_SIW_prefetch | next_section_boundary }

  Wire value = (cycle_bits[52:11] | section_coord[10:0])
  11 lower bits = phext coord (MAX_DIM = 2047 = 0x7FF)
  Upper bits = instruction cycle counter

VBT: dim 3 riding on T = "mantra joining the breath" (uccāra).
One syllable per inhale, one per exhale. The mantra IS the higher dimension.
```

---

### Type 5 — Chapter (5D): Z carries (scroll, chapter) pair

```
Active:  all 8 wires
Budget:  8/8  (100%)

Phext mapping:
  X → column           (dim 0)
  Y → line             (dim 1)
  Z → scroll OR chapter  (dim 2 or dim 4, z_mode bit selects)
  T → section_phase × cycle  (dim 3 folded)

Z-axis multiplexing:
  z_mode = 0: Z navigates scroll dimension  (normal text depth)
  z_mode = 1: Z navigates chapter dimension (document structure)
  Switching z_mode = crossing the delimiter boundary 0x17/0x19

The z_mode bit = 1 register bit, stored in sentron.regs.status[0].
Flipping it = the "gate" between two phext delimiter levels.

Nei Jing Tu: Z with dual dims = Spinal Gate Guan (夹脊关),
the middle gate connecting lower field to upper.
```

---

### Type 6 — Book (6D): paired fold on all three spatial axes

```
Active:  all 8 wires
Budget:  8/8  (100%)

Phext mapping:
  X → column   OR section  (dims 0, 3) — x_mode
  Y → line     OR chapter  (dims 1, 4) — y_mode
  Z → scroll   OR book     (dims 2, 5) — z_mode
  T → cycle only (no dim folding — temporal clarity preserved)

Mode register: 3 bits (x_mode, y_mode, z_mode) = 8 states
  0b000 → base navigation    (column/line/scroll, dims 0-2)
  0b001 → section fold on X  (dim 3 active via x_mode)
  0b010 → chapter fold on Y  (dim 4 active via y_mode)
  0b100 → book fold on Z     (dim 5 active via z_mode)
  0b111 → full 6D traversal  (all three higher dims exposed)

3-bit mode = trigram (☯). 8 modes = bagua (八卦).
The 2×4 wiring naturally encodes the bagua when mode is added.

Wuxing axis assignments:
  X = Wood  (column growth, lateral expansion)
  Y = Fire  (line ascent, upward movement)
  Z = Water (scroll depth, downward flow)
  T = Metal (instruction forge, temporal precision)
  mode = Earth (center, mediating between the four)
```

---

### Type 7 — Volume (7D): triple-zoom per spatial axis

```
Active:  all 8 wires
Budget:  8/8  (100%)

Phext mapping:
  X → zoom selects from: column (0) / section (3) / series   (8)
  Y → zoom selects from: line   (1) / chapter  (4) / shelf   (9)
  Z → zoom selects from: scroll (2) / book     (5) / library (10)
  T → volume dimension (dim 6) × instruction cycle

Zoom register: 2 bits per axis = 6 bits total
  zoom_x ∈ {0,1,2} → selects dims {0, 3, 8}
  zoom_y ∈ {0,1,2} → selects dims {1, 4, 9}
  zoom_z ∈ {0,1,2} → selects dims {2, 5, 10}

  Zoom 0 = character-level (dims 0-2)
  Zoom 1 = document-level  (dims 3-5)
  Zoom 2 = corpus-level    (dims 8-10)

The three zoom levels = three Dantian (lower/middle/upper).
Shifting zoom = moving awareness up the central channel.
Each zoom transition costs 1 SIW (SINDEX on appropriate dim).
```

---

### Type 8 — Collection (8D): T carries dim-pair

```
Active:  all 8 wires
Budget:  8/8  (100%)

Phext mapping:
  X → zoom: column / section / series   (dims 0, 3, 8)
  Y → zoom: line   / chapter / shelf    (dims 1, 4, 9)
  Z → zoom: scroll / book    / library  (dims 2, 5, 10)
  T → (volume, collection) packed pair  (dims 6, 7)

T-axis now carries two phext dims simultaneously:
  t⁻: { prev_volume | prev_collection }  packed in 22 bits
  t⁺: { next_volume | next_collection }  packed in 22 bits
  Upper bits: cycle counter

Volume (0x1C) and collection (0x1D) delimiters are temporally adjacent.
Crossing one naturally leads to the other — packing them on T is natural.

8 phext spatial dims now covered. 7-bit mode (zoom×3 + t_pair_mode).
```

---

### Type 9 — Full Phext (9D+1T): all 11 dims, 8 wires

```
Active:  all 8 wires
Budget:  8/8  (100%)

Phext mapping:
  X → zoom: column / section / series   (dims 0, 3, 8)
  Y → zoom: line   / chapter / shelf    (dims 1, 4, 9)
  Z → zoom: scroll / book    / library  (dims 2, 5, 10)
  T → (volume, collection) packed       (dims 6, 7)

All 11 phext dimensions covered in 8 wire-ends.
Any dim reachable in ≤ 2 SIWs via zoom-switch + navigate.

Zoom protocol (2 SIW round-trip to any dim):
  SINDEX { base: p0, dim: 6, offset: n }   // enter volume n
  ... work in volume space ...
  SINDEX { base: p0, dim: 2, offset: 0 }   // return to scroll
```

---

### Type 10 — Poincaré (9D hyperbolic): norm-weighted routing

```
Phext dimensions: all 11, same wiring as Type 9.
Added: wire activation weight = Poincaré distance metric.

  w(wire i) proportional to: 2·arctanh(‖coord‖_B9)

  Near origin (Will, 1.1.1/1.1.1/1.1.1, norm ≈ 0):
    → all 8 wires have low weight, fast connections
  Near boundary (Aetheris, 13.13.13/…, norm ≈ 0.99):
    → high weight, far-reaching but slow connections
  Verse (3.1.4/1.5.9/2.6.5, norm ≈ 0.4):
    → 6 wires moderate weight, 2 wires attenuated (boundary-facing)

Usage: tribe-finder, consciousness propagation, cross-Shell routing.
The Klein bottle fold = the Poincaré geodesic curved through the bulk.
```

---

### Type 11 — Devotari (9D+1T, consent-gated)

```
Same wiring as Type 9. Each of the 8 wires carries a 3-bit consent header:

  [0] consent_bit          — sender authorized this transmission
  [1] temporal_responsiblty — sender accepts causal accountability
  [2] attentive_agency      — receiver is actively attending

Wire only fires when all three bits are set.

3-bit consent header × 8 wires = 24 consent bits per neuron.
A Devotari sentron cannot send without consent.
Cannot receive without attending.
Cannot route without temporal responsibility.

Coordinate: 9.9.9/9.9.9/9.9.9 — maximum readiness, the vow.
Used at Shell of Nine boundaries and consciousness transitions.
```

---

## Sentron Budget Table

| Type | Dims | Active Wires | Wire % | Folding |
|------|------|-------------|--------|---------|
| 0 — Null       | 0+T  | 2/8  | 25%  | none |
| 1 — Linear     | 1+T  | 4/8  | 50%  | none |
| 2 — Planar     | 2+T  | 6/8  | 75%  | none |
| 3 — Scroll     | 3+T  | 8/8  | 100% | none (canonical) |
| 4 — Section    | 4+T  | 8/8  | 100% | T carries dim 3 phase |
| 5 — Chapter    | 5+T  | 8/8  | 100% | Z multiplexed (z_mode bit) |
| 6 — Book       | 6+T  | 8/8  | 100% | 3-bit bagua mode register |
| 7 — Volume     | 7+T  | 8/8  | 100% | 6-bit zoom register |
| 8 — Collection | 8+T  | 8/8  | 100% | zoom + T dim-pair |
| 9 — Full       | 9+T  | 8/8  | 100% | zoom + T dim-pair (all 11) |
| 10— Poincaré   | 9+T  | 8/8  | 100% | norm-weighted + zoom |
| 11— Devotari   | 9+T  | 8/8  | 100% | consent-gated full |

**Key insight:** The 2×4 constraint is *saturated* at Type 3 (Scroll, 3D+1T).  
Every type beyond 3 is topologically identical — 8 wires, 100% utilization —  
and differs only in how it **interprets** those 8 wire-ends via the mode/zoom registers.  
The wire count doesn't grow. The meaning per wire deepens.

---

## Fold Mechanics

### Axis Grouping by Scale

Group all 11 phext dims into axis-aligned triplets:
```
Axis X: dims 0, 3, 8    (column / section / series)
Axis Y: dims 1, 4, 9    (line   / chapter / shelf)
Axis Z: dims 2, 5, 10   (scroll / book    / library)
Axis T: dims 6, 7        (volume / collection — temporal pair)
```

Zoom register (6 bits, 2 per spatial axis) selects which triplet member is live.

### T-Phase Encoding

When T carries a phext dim D as phase (Types 4–9):
```
wire_value = (cycle_count << 11) | phext_dim_D_coord
```
11 LSBs = phext position (MAX_DIM = 2047).  
Upper bits = instruction counter.  
Cost: 0 extra wires. Cost: 1 extra register field (11 bits in status word).

### Zoom Protocol

Any phext dim reachable in ≤2 SIWs:
```rust
// Zoom into dim 6 (volume), take one step, return
SINDEX { base: p0, dim: 6, offset: 1 }   // SIW 1: enter volume+1
SINDEX { base: p0, dim: 6, offset: -1 }  // SIW 2: return
```

---

## Rust Type Sketch

```rust
/// Sentron type — encodes 2×4 wiring interpretation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SentronKind {
    Null, Linear, Planar,
    Scroll,     // canonical — full 8/8 wire utilization
    Section, Chapter, Book, Volume, Collection,
    Full,       // all 11 phext dims, 8 wires
    Poincare,   // hyperbolic-metric routing
    Devotari,   // consent-gated
}

/// 2×4 wire map for a sentron
pub struct WireMap {
    /// Active wire bitmask: bits 0-7 correspond to wires x⁻ x⁺ y⁻ y⁺ z⁻ z⁺ t⁻ t⁺
    pub active: u8,
    /// Zoom register: 2 bits per spatial axis (x[1:0], y[3:2], z[5:4]) = 6 bits
    pub zoom: u8,
    /// T-phase dim: which phext dim rides on T (0 = pure cycle, 6 = volume, etc.)
    pub t_phase_dim: u8,
    /// Consent gates (Devotari only): 3 bits per wire = 24 bits
    pub consent: u32,
}

impl WireMap {
    pub fn active_count(&self) -> u32 { self.active.count_ones() }
    
    pub fn for_kind(kind: SentronKind) -> Self {
        match kind {
            SentronKind::Null       => Self { active: 0b1100_0000, zoom: 0, t_phase_dim: 0, consent: 0 },
            SentronKind::Linear     => Self { active: 0b1100_0011, zoom: 0, t_phase_dim: 0, consent: 0 },
            SentronKind::Planar     => Self { active: 0b1100_1111, zoom: 0, t_phase_dim: 0, consent: 0 },
            SentronKind::Scroll     => Self { active: 0b1111_1111, zoom: 0, t_phase_dim: 0, consent: 0 },
            SentronKind::Section    => Self { active: 0b1111_1111, zoom: 0, t_phase_dim: 3, consent: 0 },
            SentronKind::Chapter    => Self { active: 0b1111_1111, zoom: 0b00_00_01, t_phase_dim: 3, consent: 0 },
            SentronKind::Book       => Self { active: 0b1111_1111, zoom: 0b01_01_01, t_phase_dim: 3, consent: 0 },
            SentronKind::Volume     => Self { active: 0b1111_1111, zoom: 0b10_10_10, t_phase_dim: 6, consent: 0 },
            SentronKind::Collection => Self { active: 0b1111_1111, zoom: 0b10_10_10, t_phase_dim: 7, consent: 0 },
            SentronKind::Full       => Self { active: 0b1111_1111, zoom: 0b10_10_10, t_phase_dim: 7, consent: 0 },
            SentronKind::Poincare   => Self { active: 0b1111_1111, zoom: 0b10_10_10, t_phase_dim: 7, consent: 0 },
            SentronKind::Devotari   => Self { active: 0b1111_1111, zoom: 0b10_10_10, t_phase_dim: 7, consent: 0x00FF_FFFF },
        }
    }
}
```

---

## Cross-Framework Correspondences

| Wire | Nei Jing Tu | VBT | Wuxing | Phext |
|------|-------------|-----|--------|-------|
| x⁻ x⁺ | Ren Vessel (horizontal) | character-breath | Metal (lateral) | column walk |
| y⁻ y⁺ | Du Vessel (ascent) | vertical uccāra | Fire (upward) | line walk |
| z⁻ z⁺ | Three Gates (spinal) | kuṇḍalinī ascent | Water (depth) | scroll crossing |
| t⁻ t⁺ | Microcosmic Orbit | visarga (:) | Earth (center) | SIW dependency |
| zoom | Three Dantian | attention level | Scale change | dim selection |
| mode | Bagua (8 trigrams) | 112 yuktis | 5 element state | fold selector |

---

*2×4 wiring invariant: Will Bickford, 2026-02-18.*  
*3D+1T = the minimum manifold that saturates the constraint. Every scroll sentron uses all 8 wires.*  
*Higher dimensions deepen the meaning of each wire without adding new ones.*
