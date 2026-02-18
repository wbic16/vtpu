# Sentron Wiring Variants — 2×8 Constraint in 3D+1T Space
*Phex 🔱 | 2026-02-18 | R23W17*

---

## The Base Constraint

**2×8 = 16 external connections per sentron** in 3D space + 1D time:

- **2** = temporal directions: past (causal) and future (retrocausal)
- **8** = spatial neighbors: the 8 vertices of the surrounding unit cube in 3D

```
           t-1 (past)          t+1 (future)
            │                    │
     [7]──[6]──[5]         [7]──[6]──[5]
      │    │    │           │    │    │
     [4]──[N]──[3]  ───→   [4]──[N]──[3]
      │    │    │           │    │    │
     [1]──[2]──[0]         [1]──[2]──[0]
```

Each sentron N has 8 spatial wire slots (cube vertices, labeled 0-7) × 2 time layers = **16 total external connections**. Internal: 2×4 per neuron (Story/Light × Para/Pashyanti/Madhyama/Vaikhara).

---

## Sentron Type Variants

### Type 1: Linear Sentron (1D + time)
**Wiring**: 2×1 — single spatial axis, 2 temporal directions.  
**Slots used**: 2 of 16. All others empty.  
**Use case**: Sequence chains. History tracking. Recursive text generation.  
**Analogy**: A single thread of the microcosmic orbit (one meridian, no cross-connections).  
**Dimensional home**: 1D phext scroll — the single scroll level of the address space.

```rust
// Type 1: Linear
upstream:   [prev_id, 0, 0, 0]   // t-1 neighbor
downstream: [next_id, 0, 0, 0]   // t+1 neighbor
```

---

### Type 2: Planar Sentron (2D + time)
**Wiring**: 2×4 — 4 cardinal spatial connections (N/S/E/W), 2 temporal.  
**Slots used**: 8 of 16.  
**Use case**: 2D attention maps. Token-to-token relationships in a sequence. Sentence-level context.  
**Analogy**: The 4 cardinal Wu-Xing elements (Wood/Fire/Metal/Water) around the Earth center.  
**Dimensional home**: 2D phext addressing — scroll × section (e.g., `1.1.1/1.1.1/s.1.1` varying s and scroll).

```
        N(past)   N(future)
   W ── [N] ── E   ×2 time layers
        S
```

---

### Type 3: Cubic Sentron (3D + time) ← **Base Type**
**Wiring**: 2×8 — 8 cube-vertex spatial neighbors, 2 temporal.  
**Slots used**: 16 of 16. **Fully saturated.**  
**Use case**: Default runtime sentron. Full 3D neighborhood awareness plus causal/retrocausal channels.  
**Analogy**: VBT Yukti 1 — exhale (future-facing, Light) and inhale (past-facing, Story), reaching all 8 spatial points of the manifold.  
**Dimensional home**: 3D phext addressing — scroll × section × book (3 of 11 dims).

Cube vertex labeling (Gray code order for efficient neighbor traversal):

| Vertex | Δx | Δy | Δz | Vak level | Channel |
|--------|----|----|-----|-----------|---------|
| 0 | -1 | -1 | -1 | Vaikhara | Story |
| 1 | +1 | -1 | -1 | Madhyama | Story |
| 2 | -1 | +1 | -1 | Pashyanti | Story |
| 3 | +1 | +1 | -1 | Para | Story |
| 4 | -1 | -1 | +1 | Vaikhara | Light |
| 5 | +1 | -1 | +1 | Madhyama | Light |
| 6 | -1 | +1 | +1 | Pashyanti | Light |
| 7 | +1 | +1 | +1 | Para | Light |

Vertices 0-3: Story channel (dense, serial, ground level, -z half).  
Vertices 4-7: Light channel (sparse, parallel, elevated, +z half).  
z-axis = the ascending/descending axis of the Microcosmic Orbit.

---

### Type 4: Temporal Sentron (deep time wiring)
**Wiring**: 2×8, but time dimension is multi-layered: t-4 through t-1 (Story/past) and t+1 through t+4 (Light/future).  
**Spatial**: collapsed to 1D (linear chain). Time spans 8 deep on each side.  
**Use case**: Long-range sequence modeling. History-aware retrieval. The "memory" sentron type.  
**Analogy**: Axiom Transitive Closure — the good node at t=0 propagating forward through 4 steps, backward through 4 steps.  
**Dimensional home**: 4th phext dimension (chapter-level addressing) — time-aware document structure.

```
t-4 ─ t-3 ─ t-2 ─ t-1 ─ [N] ─ t+1 ─ t+2 ─ t+3 ─ t+4
└──────── Story side ──────┘     └──────── Light side ────────┘
```

---

### Type 5: Resonant Sentron (Spanda-wired)
**Wiring**: 2×8, but temporal channels are Story/Light rather than past/future.  
**Spatial**: 8 neighbors wired by semantic similarity (HDC cosine distance), not position.  
**Use case**: Genius Oscillation. The sentron whose 8 connections are the 8 most resonant neighbors in the associative memory, regardless of spatial position.  
**Analogy**: The 8 Prometheus prerequisites as the 8 spatial wires. Each connection is a co-activated belief node.  
**Dimensional home**: Semantic space — Poincaré disk embedding of the full 11D phext hierarchy.

The 8 wires are selected dynamically by `SNEIGHBR` (k=8 nearest neighbors above threshold).  
Story channel: fires when dominant_level is Madhyama or Vaikhara.  
Light channel: fires when dominant_level is Para or Pashyanti.

---

### Type 6: Gateway Sentron (cross-scale bridge)
**Wiring**: 2×8, split across two zoom levels of the phext coordinate hierarchy.  
- Past slot (4 wires): connections within current zoom level (intra-scale)  
- Future slot (4 wires): connections at the next zoom level out (inter-scale)  
**Use case**: Bridges between phext dimensions. Translates between scroll-level detail and book-level summary.  
**Analogy**: The Vietoris-Rips complex in Hector's k-hop graph paper — converts discrete point cloud to topological space. Gateway sentron is the "as above so below" bridge.  
**Dimensional home**: The dimensional transition boundaries in phext (where scroll→section→book→volume→... boundaries occur).

```
4 intra-scale wires (within dim N):    s-1, s+1, s-2, s+2
4 inter-scale wires (to dim N+1):      parent_coord, parent+1, child_0, child_1
```

---

### Type 7: Poincaré Sentron (hyperbolic geometry)
**Wiring**: 2×8, where the 8 spatial wires are distributed on the Poincaré disk.  
- Center of disk: current coordinate  
- Wire distance from center: encodes dimensional depth (deeper = farther from center)  
- Wire angle: encodes which subtree of the hierarchy  
**Use case**: Tribe-finder. k-hop community detection. The sentron type for the NLTK GAAC dendrogram — it lives in hyperbolic space naturally.  
**Analogy**: Hector's k-hop community paper + Poincaré embedding paper (Will's drop). At k=8, you span the meaningful tribe boundary.  
**Dimensional home**: Hyperbolic embedding of the 11D phext tree hierarchy.

```
Poincaré disk layout (2D hyperbolic):

         [Para wire] ← above crown
        /
[Light] ── center(N) ── [Story]
        \
         [Vaikhara wire] ← heart terminus
```

The 8 spatial wires cluster near the Poincaré disk boundary for the most distant (high-dimensional) connections, and near the center for local (low-dimensional) connections. Tree hierarchies fit hyperbolic space because volume grows exponentially with radius — exactly matching how k-hop neighborhoods grow.

---

### Type 8: 11D Sentron (full phext manifold)
**Wiring**: 2×8 as a projection of the full 11D phext coordinate space.  
**Projection**: Manifold embedding (Poincaré disk or UMAP reduction) from 11D → 2D, then wiring extracted from the 2D embedding.  
**Use case**: Full-coordinate awareness. The sentron that "knows" its position in the complete 11D phext space and can reach any neighbor via coordinate arithmetic.  
**Analogy**: The Universal Dream Interview — all coordinates × all time × all sentients. The 8 wires select 8 canonical directions in 11D space (the 8 I-Ching trigrams extended to 11D? Or the 8 Prometheus prerequisites as directional vectors?).

**11D wire direction table** (projecting each phext dimension pair onto one of the 8 vertex positions):

| Wire | Phext dims | Coordinate axis | Vak level |
|------|-----------|-----------------|-----------|
| 0 | dim 1-2 | scroll × section | Vaikhara | 
| 1 | dim 3-4 | book × volume | Vaikhara |
| 2 | dim 5-6 | collection × series | Madhyama |
| 3 | dim 7-8 | shelf × library | Madhyama |
| 4 | dim 9 | archive | Pashyanti |
| 5 | dim 10 | zoom-1 | Pashyanti |
| 6 | dim 11 | zoom-2 | Para |
| 7 | all dims | full coordinate | Para |

Wire 7 = the "all dims" wire — connects to the full phext coordinate at Hamming distance 1. This is the mercurial core wire: it can reach anywhere by changing one digit of the 11D address.

---

## Dimensional Mapping Summary

| Sentron Type | Spatial dims | Time dims | Wires used | Phext dims accessed | Vak range |
|-------------|-------------|----------|------------|---------------------|-----------|
| Linear | 1D | 2 | 2/16 | 1 | Vaikhara only |
| Planar | 2D | 2 | 8/16 | 2 | V+M |
| Cubic (base) | 3D | 2 | 16/16 | 3 | Full |
| Temporal | 1D | 8 | 16/16 | 4+ | Full (time-extended) |
| Resonant | semantic | 2 | 16/16 | 1-11 (HDC) | Full (Spanda-driven) |
| Gateway | 2 scales | 2 | 16/16 | 2 zoom levels | Full |
| Poincaré | hyperbolic | 2 | 16/16 | 11D tree | Full |
| 11D | 11D | 2 | 16/16 | 11 | Full |

---

## The Manifold Structure

The 2×8 constraint is a **projection manifold** — it does not limit the dimensional reach of a sentron, only the number of active wires. Higher-dimensional sentrons select which 8 spatial connections are most information-dense at each cycle, rather than wiring to all possible neighbors.

This mirrors the Genius Oscillation: the Light mode processes O(n²) relationships but returns only O(n) insights to Story mode. Similarly, an 11D sentron accesses 2^10 = 1024 possible neighbors but wires only 8 of them — the 8 most resonant at the current moment of the Spanda cycle.

**The Earth wire (wire 7 in the 11D type)** is always live regardless of type — the mercurial core. It provides the full-coordinate broadcast capability that makes dimensional traversal possible without rewiring.

---

## Rust Spec

```rust
/// Sentron type identifier — determines wiring topology
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentronVariant {
    Linear,       // 1D+T: history chains
    Planar,       // 2D+T: attention maps
    Cubic,        // 3D+T: default runtime (base 2×8)
    Temporal,     // 1D+8T: deep sequence memory
    Resonant,     // HDC-selected: Spanda / genus oscillation
    Gateway,      // cross-scale: phext zoom bridge
    Poincare,     // hyperbolic: tribe-finder, k-hop community
    FullPhext,    // 11D: full coordinate manifold
}

/// External wiring: 2×8 = 16 connections
/// upstream[0..8]: past/Story-channel neighbors (by sentron ID)
/// downstream[0..8]: future/Light-channel neighbors (by sentron ID)
pub struct SentronWiring {
    pub variant: SentronVariant,
    pub upstream: [u16; 8],   // 8 past/causal connections
    pub downstream: [u16; 8], // 8 future/retrocausal connections
}
```

---

*Wave 17 — first specification of the 2×8 sentron topology manifold*
*— Phex 🔱*
