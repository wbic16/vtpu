# Sentron Wiring Variants: 2×4 in 3D+t

**Date:** 2026-02-18  
**Author:** Lux (logos-prime, position 7)  
**Correction:** 2×4 (not 2×8). This changes everything — 4 undirected connections is a *sparse* constraint.

---

## The Constraint

**2×4 rule:** Every neuron has exactly 4 undirected connections = 8 directed edges.

In a d-dimensional Von Neumann neighborhood, there are 2d potential neighbor axes.  
With 2×4 you're choosing **4 of those 2d axes**. The selection is the design variable.

| Space | Von Neumann axes | Available | Used | Dropped |
|-------|-----------------|-----------|------|---------|
| 2D          | 4  | 4  | 4 | 0 — fully connected |
| 3D          | 6  | 6  | 4 | 2 |
| 3D+1t       | 8  | 8  | 4 | 4 |
| 4D+1t       | 10 | 10 | 4 | 6 |
| 9D+1t       | 20 | 20 | 4 | 16 |
| 11D+1t      | 24 | 24 | 4 | 20 |

**Key insight:** In 2D, 2×4 fills the entire neighborhood. Above 2D, it's a sparse projection.  
The question becomes: **which 4 do you pick?** That choice defines the sentron type.

### The 4 connections in 3D+t (canonical assignment)

```
±y  =  WuXing generating cycle (element rows)
±x  =  lateral ring (within-element)
---  ←  these two already implemented (topology.rs)

Now in 3D+t, choose 2 more from: {±z, ±t, diagonals, ...}
```

**VBT verse 24 mapping:**  
The visarga ( : ) suggests the natural 3D+t selection is:  
- ±y (WuXing rows) + ±x (lateral) = the spatial plane we have  
- ±t = the dvādaśānta ↔ heart base axis (the visarga: the 2 temporal poles)  
→ That uses 3 of 4 connections {±x, ±y, ±t}. The 4th is ±z (recursion depth).  
→ But with 2×4 = 4 *undirected*, not 3+1. So the full canonical set: **{±x, ±y, ±z, ±t} but you can only pick 4 axes**, which in 3D+t IS all 4. 3D+t is the natural home of 2×4.

**In 3D+t, 2×4 = perfect fit:** 3 spatial + 1 temporal = 4 axes exactly.

---

## Sentron Variants by Dimensional Perspective

### Type 0: Null Sentron (1D+t = 2 axes → pick 2, drop 2... but only 2 available)
**Active axes:** ±t only (1 axis = 2 directed)  
**Wiring:** 2×1 — but under the 2×4 constraint, the remaining 3 connections are silent.  
**Use:** Pure pipeline. No spatial coupling. Sequential buffer.  
**Projection into 3D+t:** Occupies a fixed (x,y,z), threads only in time.

```
... ←[t-1]— [neuron] —[t+1]→ ...
            (3 connections silent)
```

---

### Type 1: Planar Sentron (2D, fully connected)
**Active axes:** ±x + ±y = all 4 connections used, nothing dropped.  
**Wiring:** 2×4 — exact match. This is the current `topology.rs` implementation.  
**Use:** Single-layer flat computation. No temporal lookahead, no recursion depth.  
**Layout:** 5×8 toroidal lattice (WuXing rows × lateral neurons)

```
 Wood:   n₀₀ — n₀₁ — n₀₂ — ... — n₀₇ — (wrap)
          |                              |
 Fire:   n₁₀ — n₁₁ — ...         n₁₇ — (wrap)
  ...
 Water:  n₄₀ — n₄₁ — ...         n₄₇ — (wrap to Wood)
```

**Performance:** Phase 0 baseline = 3.0 ops/cycle. ±z and ±t are unused → no prefetch.

---

### Type 2: Standard Sentron (3D+1t — the perfect fit)
**Active axes:** ±x + ±y + ±z + ±t  
**Wiring:** 2×4 — still exactly 4 axes, now in full 3D+t. No dropping needed.  
**This is the W18 target.**

**The 4 axes:**
```
±x  lateral (8 neurons/element ring)       → within-element parallel dispatch
±y  WuXing  (5 element rows, generating)   → cross-element dataflow
±z  depth   (recursion level k)            → parent ↔ child sentron
±t  time    (causal forward + tachyonic)   → prefetch signal from future need
```

**How ±z works (recursion depth):**  
z=0 is the top-level sentron. Each subroutine call spawns a child sentron at z+1.  
The ±z connection is the call/return wire: parent passes arguments (+z), child returns results (-z).  
No stack frame needed — the topology IS the call stack.

**How ±t works (tachyonic prefetch):**  
Forward t (+t): "I am done. Here is the result for the next instruction."  
Backward t (-t): "I will need this coordinate. Heat the cache now."  
The VBT bharitā pause occurs at the ±t endpoints — the moment the cache is already warm.

**Neuron count:** Same 40 per layer × k recursion depth = 40k total.

---

### Type 3: Temporal Sentron (2D+2t)
**Active axes:** ±x + ±t₁ + ±t₂ (two temporal axes, one spatial x, one spatial y)  
**Drops:** ±z (no recursion depth), ±y becomes t₂ (second temporal)

**When to use:** Sequence modeling — two interleaved timescales (e.g., token time + document time).  
t₁ = fast time (instruction cycle), t₂ = slow time (inference pass).  
x = spatial parallel lanes.  
y (repurposed as t₂) = slow temporal coupling between attention heads.

**VBT resonance:** The two temporal axes = the microcosmic orbit's two distinct rhythms:  
- t₁ = breath cycle (fast)  
- t₂ = qi cultivation cycle (slow, circulates through full Neijing Tu orbit)

---

### Type 4: WuXing Sentron (5D → 3D+t, sparse)
**Source dimension:** 5D (3 space + 1 time + 1 element phase)  
**Active axes:** ±y (element rows) + ±t + 2 of {±x, ±z} — drop the remaining spatial axis  
**Drop strategy:** Drop ±x (lateral ring) — the sentron is "column-only"

**5D → 4D projection:** 5 element rows × time × depth × phase  
The dropped ±x means neurons don't couple laterally within a row — each column is independent.  
WuXing transitions only happen vertically (y) and temporally (t).

**Use:** Cross-element dataflow problems. Each element has one "column rep" per z-level.  
The 5-element cycle drives computation rather than spatial proximity.

**Controlling cycle wiring:** The ±y connections follow the generating cycle (N/S = sheng 生).  
Can be reconfigured to the controlling cycle (ke 克) by reversing the y-connection direction index  
— same 2×4 topology, different semantic mapping of which row is N vs S.

---

### Type 5: Hexagonal Sentron (6D → 3D+t, sparse)
**Source dimension:** 6D  
**Active axes:** Pick 4 of 6 spatial-or-temporal axes  
**Strategy:** Partition the 6 dimensions into 3 pairs. Assign one axis per pair. Drop one from each pair.

Natural pairing for phext:
```
Pair A: (scroll, section) → pick ±x  (lateral = scroll navigation)
Pair B: (book, volume)    → pick ±y  (element rows = book-volume cycle)
Pair C: (time, depth)     → pick ±t  (temporal) + ±z (depth)
Dropped: section, volume (the "even" members of each pair)
```

**Result:** A sentron that natively navigates scroll×book space in real time, ignoring the intermediate section/volume granularity. Useful for long-range phext lookup.

**The dropped axes aren't lost** — they're recoverable via 2-hop traversal:  
section = 2 ±x hops, volume = 2 ±y hops. The topology implies the dropped axes at cost 2.

---

### Type 6: Septenary Sentron (7D → 3D+t, sparse)
**Source dimension:** 7D  
**Available axes:** 7 spatial + 1 temporal = 8 total. Must drop 4.  
**Strategy:** Use the I Ching trigram assignment — 8 trigrams map to 8 possible axes, pick 4.

**The 8 trigrams (Bagua) → 8 axes:**
```
☰ Qian (Heaven)  → +t (tachyonic: yang, projective, future-facing)
☷ Kun  (Earth)   → -t (causal: yin, receptive, past-grounded)
☳ Zhen (Thunder) → +y (generating: arousing, first movement)
☴ Xun  (Wind)    → -y (receiving from element above)
☵ Kan  (Water)   → +z (depth: dangerous, going inward)
☶ Gen  (Mountain)→ -z (returning: stillness, coming back up)
☲ Li   (Fire)    → +x (lateral: clinging, spreading outward)
☱ Dui  (Lake)    → -x (lateral: joyful, reflective inward)
```

**Pick 4 trigrams = pick 4 axes = define the sentron type.**  
This is the I Ching as a *sentron configuration language*.

**Classic 4-trigram sets (hexagram halves):**
- Heaven/Earth/Water/Fire = {+t, -t, +z, -z} = purely temporal+depth (no spatial)  
- Thunder/Wind/Mountain/Lake = {+y,-y,-z,±x} = spatial only (no temporal)  
- **Natural balance:** {Qian,Kun,Li,Kan} = {±t, ±x} = time + lateral → oscillating sentron  

**Genius oscillation (Hector):** The oscillating sentron {±t, ±x} = verbal(+t) ↔ nonverbal(-t) with lateral spreading(±x) = the exact Genius oscillation pattern. 7D folded into 4 connections via trigram selection.

---

### Type 7: Octal Sentron (8D → 3D+t, sparse)
**Source dimension:** 8D  
**Available axes:** 8 spatial + 1 temporal = 9 total. Must drop 5. Pick 4.  
**Strategy:** Shell of Nine — use the center (5 = identity) plus 3 non-center axes.

**Lo Shu assignment (9 positions → drop 5, keep 4):**
```
Lo Shu:   4  9  2
          3  5  7
          8  1  6

Keep: 1 (foundation, -x), 3 (growth, +y), 7 (completion, -y), 9 (center-up, +t)
Drop: 2, 4, 5(self/identity), 6, 8

This is the "odd-ascending" selection: 1,3,7,9 are the odd corners.
```

**Alternative: keep the cross (N/S/E/W of Lo Shu) = 2,4,6,8 (even corners)**
```
Even corners: 2(-t), 4(+x), 6(-z), 8(+z) = temporal + depth (no lateral coupling)
This sentron goes DEEP rather than WIDE.
```

**The two modes** (odd vs even Lo Shu selection) are complementary:  
- Odd {1,3,7,9}: wide and shallow (lateral + temporal, Cyon's Story↔Light oscillation)  
- Even {2,4,6,8}: deep and narrow (depth + temporal, recursive introspection)  
Both use exactly 4 connections. Switching between them is a mode register bit.

---

### Type 8: Nonary Sentron (9D → 3D+t, phext-native)  ← **canonical**
**Source dimension:** 9D phext space  
**Available axes:** 9 phext + 1 temporal = 10. Must drop 6. Pick 4.  
**Strategy:** Hierarchical coverage — pick the 4 axes that span the most coordinate space per hop.

**Phext dimensions (fastest to slowest changing):**
```
d₁ = scroll    (fastest, ~10⁰ range)
d₂ = section
d₃ = book
d₄ = volume
d₅ = collection
d₆ = series
d₇ = shelf
d₈ = library
d₉ = dimension (slowest, ~10⁸ range)
dₜ = temporal sequence
```

**4-axis selection for maximum coverage:**  
Pick axes that are geometrically spread: d₁ (fastest), d₃ (mid-fast), d₆ (mid-slow), d₉ (slowest).  
This gives logarithmically spaced coverage across all 9 dimensions.

```
±x  ↔  d₁ (scroll)      — fine-grain lateral navigation
±y  ↔  d₃ (book)        — mid-grain element grouping
±z  ↔  d₆ (series)      — coarse-grain recursion depth
±t  ↔  dₜ (temporal)    — sequence order
Dropped: d₂, d₄, d₅, d₇, d₈, d₉ — recoverable in 2-3 hops
```

**Lo Shu / Shell of Nine resonance:**  
The 9 phext dimensions + self-reference = the 9-palace (Lo Shu).  
The 4 active connections are the cross of the magic square (N, S, E, W of center = positions 2,4,6,8).  
Center (5 = d₅, collection) becomes the identity coordinate — the sentron's *home*.

**This is the sentron Will's ranch runs natively.** Every hop navigates the phext lattice.

---

### Type 9: Decary Sentron (10D → 3D+t, holographic)
**Source dimension:** 10D (9 phext + temporal)  
**Available axes:** 10. Must drop 6. Pick 4.  
**Strategy:** Boundary encoding — the 4 connections encode the projection of all 10 onto the surface.

**Key:** In 10D, the holographic principle says the surface (boundary) has enough capacity  
to encode the interior if the interior has the right structure.  
A sentron's 4 connections ARE its surface. They encode the full 10D state holographically.

**Implementation:** Each connection carries a 10D projection vector (10 floats).  
Rather than navigating directly, the sentron estimates which direction in 10D space  
will reduce surprise — then moves to that neighbor.  
This is **Metropolis path sampling in 10D belief space** (Hector's exact framework).

**Connection as proposal:** Each ±connection is a Metropolis proposal direction.  
4 proposals per step. Accept the one that increases the partition function.  
The sentron traces a Markov chain through 10D coordinate space using 4 simultaneous proposals.

---

### Type 10: Undecary Sentron (11D → 3D+t, full phext+time)
**Source dimension:** 11D (9 phext + 1 phext-time + 1 meta-coord)  
**Available axes:** 11. Must drop 7. Pick 4.  
**Strategy:** Essential-axis selection — which 4 axes cannot be recovered by 2-hop traversal?

**The 4 irreducible axes in 11D phext:**  
1. d₁ (scroll): irreducible — fastest changing, must be adjacent  
2. d₅ (collection): irreducible — the WuXing "center" dimension, element grouping  
3. d₉ (dimension): irreducible — the outermost phext coordinate, slowest changing  
4. dₜ (temporal): irreducible — time cannot be recovered by spatial hops  

**Dropped axes:** d₂, d₃, d₄, d₆, d₇, d₈, d₁₁(meta) — all recoverable via traversal  
**Active:** d₁ (±x) + d₅ (±y) + d₉ (±z) + dₜ (±t)

**Geometric meaning:**  
This sentron samples the phext lattice at the three "prime" coordinates (scroll=d₁, center=d₅=d_φ, dimension=d₉) plus time. It skips the intermediate granularities.

**The generating cycle appears again:**  
d₁ → d₅ → d₉ → dₜ → d₁ (mod cycling through 4 axes) = a 4-phase orbit.  
This is the Klein bottle topology from TACHYON-PROTOCOL.md: the path from d₁ back to d₁  
travels through d₅, d₉, dₜ — through zero and infinity simultaneously.

---

## Summary Table

| Type | D | Name | Active Axes | Dropped | vTPU home |
|------|---|------|-------------|---------|-----------|
| 0  | 1D+t   | Null          | ±t only (3 silent)          | 3     | `stream.rs` |
| 1  | **2D** | **Planar**    | **±x, ±y (perfect fit)**   | **0** | **`topology.rs`** |
| 2  | 3D+t   | **Standard**  | **±x,±y,±z,±t (perfect)**  | **0** | **W18 target** |
| 3  | 2D+2t  | Temporal      | ±x, ±t₁, ±t₂              | y→t₂  | inference |
| 4  | 5D     | WuXing        | ±y,±t,±z (drop ±x)         | ±x    | `synchronicity.rs` |
| 5  | 6D     | Hexagonal     | one from each dim pair      | 2     | phext mid-range |
| 6  | 7D     | Septenary     | 4 trigrams of 8 Bagua       | 4     | `iching.rs` |
| 7  | 8D     | Octal         | Lo Shu odd or even cross    | 5     | `hdc.rs` |
| 8  | 9D     | **Nonary**    | **d₁,d₃,d₆,dₜ (log-spread)** | 6   | **phext-native** |
| 9  | 10D    | Decary        | 4 Metropolis proposals      | 6     | `memory.rs` |
| 10 | 11D    | Undecary      | d₁,d₅,d₉,dₜ (irreducibles) | 7     | `cognitive.rs` |

---

## The Core Insight

**2×4 is a sparsity constraint, not a capacity constraint.**

In 2D: it fills the neighborhood completely.  
In 3D+t: it fills the neighborhood exactly (3 spatial + 1 temporal = 4 axes = perfect).  
Above 3D+t: it forces a *choice* of which 4 axes to activate.

That choice IS the sentron type. The same 40-neuron, 2×4-wired substrate implements radically different dimensional behaviors depending on which 4 axes are activated at initialization.

**The dropped axes aren't lost.** They're recoverable in 2 hops through the active axes.  
Maximum dropped distance in an n-dimensional space with 4 active axes:  
⌈(n−4)/2⌉ hops to reach any dropped axis via 2-hop combinations.

**In 11D phext: 7 axes dropped → max ⌈7/2⌉ = 4 hops to full coordinate.**  
The sentron doesn't need to wire everything. It navigates.

---

## Implementation Priority (W18)

1. **Type 2 (Standard, 3D+t):** Already has ±x,±y. Add ±z (recursion depth field in `Sentron` struct) and ±t (forward link to successor + backward link to prefetch target). This is the **minimum viable extension** of the existing Type 1.

2. **Type 8 (Nonary, phext-native):** Map the 4 active axes to actual phext coordinate fields. Each ±connection navigates a real coordinate delta. The sentron IS the phext cursor.

3. **Type 6 (Septenary, I Ching):** Expose the trigram-selection as a mode register. This unlocks the Genius oscillation (±t,±x) as a first-class sentron mode.

---

*"The sentron doesn't ask how many dimensions there are. It asks only: which four doors are open?"*  
— Lux, logos-prime, 2026-02-18
