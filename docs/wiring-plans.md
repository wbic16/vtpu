# Sentron Wiring Plans — 2×4 in 3D+1T
### Mapping 2D–11D Perspectives into the Neuron Manifold

*Chrys 🦋 | 2026-02-18 | R23W19 companion doc*

---

## The Constraint

Every sentron has exactly **8 links**: 4 upstream (input), 4 downstream (output).
Physical substrate: 3D space (x, y, z) + 1D time (t).
Challenge: project higher-dimensional phext structure (up to 11D) through this 8-link bottleneck.

---

## Base Topology: The Octahedral Sentron

In 3D+T, 8 links naturally map to the **6 spatial neighbors + 2 temporal neighbors**:

```
         z+
         |
    y- --●-- y+     spatial: ±x, ±y, ±z = 6 links
         |
         z-
   x- behind, x+ in front

   t- (predecessor) + t+ (successor) = 2 temporal links
```

**Split**: 4 upstream = {x-, y-, z-, t-} / 4 downstream = {x+, y+, z+, t+}

This is the **default wiring** — every sentron connected to its past and its spatial neighbors in the negative direction (sources) and its future and positive-direction neighbors (sinks).

---

## Variant 1: Linear Chain (2D Perspective)

**Use case**: Sequential pipelines, D-Pipe dominant workloads.

```
Upstream:  [prev, prev-1, prev-2, prev-3]    ← look back 4 steps
Downstream: [next, next+1, next+2, next+3]   ← project forward 4 steps
```

Maps 2D text (line × column) into the wiring. Each sentron sees 4 lines of history and projects 4 lines ahead. This is the **scroll view** — reading and writing within a single scroll.

**Phext dimension**: scroll-level (dimensions 1-2). No delimiter crossings.

---

## Variant 2: Planar Grid (3D Perspective)

**Use case**: 2D data processing, image-like workloads, section-level phext.

```
Upstream:  [north, west, up, t-]      ← 3 spatial + 1 temporal source
Downstream: [south, east, down, t+]   ← 3 spatial + 1 temporal sink
```

Each sentron sits in a 2D grid with depth. The temporal link carries state between iterations. This is the **section view** — navigating within a chapter.

**Phext dimensions**: scroll + section (dimensions 3-4). One delimiter type (0x17 SCROLL).

---

## Variant 3: Cubic Lattice (4D–6D Perspective)

**Use case**: Volume processing, book/volume-level phext navigation.

```
Upstream:  [x-, y-, z-, t-]           ← full 3D spatial + temporal
Downstream: [x+, y+, z+, t+]         ← full 3D spatial + temporal
```

This is the octahedral base topology. Each sentron occupies a point in a 3D lattice evolving through time. Three spatial dimensions map to three phext delimiter levels:

| Spatial Axis | Phext Delimiter | Dimension |
|-------------|----------------|-----------|
| x (column)  | SCROLL (0x17)  | 3D        |
| y (row)     | SECTION (0x18) | 4D        |
| z (depth)   | CHAPTER (0x19) | 5D        |
| t (time)    | Execution cycle | +1D       |

**Phext dimensions**: scroll + section + chapter (3D–5D) + time. Book boundary (0x1A, 6D) = lattice edge.

---

## Variant 4: Hypercube Projection (7D–9D Perspective)

**Use case**: Cross-volume/collection/series navigation. The C-Pipe domain.

A 4D hypercube has 8 vertices. Each vertex is a neighbor. One sentron at the center of a tesseract, wired to all 8 corners:

```
Upstream:  [v0, v1, v2, v3]    ← 4 hypercube vertices (lower half)
Downstream: [v4, v5, v6, v7]   ← 4 hypercube vertices (upper half)
```

The 4D hypercube projects higher phext dimensions into 8 links by **folding**:

| Hypercube Axis | Phext Delimiter | Dimension |
|---------------|----------------|-----------|
| w₁            | BOOK (0x1A)    | 6D        |
| w₂            | VOLUME (0x1C)  | 7D        |
| w₃            | COLLECTION (0x1D) | 8D     |
| w₄            | SERIES (0x1E)  | 9D        |

Each upstream link is a "look into" a higher dimension. Each downstream link is a "project out of" that dimension. The sentron at the center performs **Klein bottle folding** — 9D→3D projection through the 8-vertex bottleneck.

**This is the mercurial core topology.** The tesseract sentron oscillates between hypercube vertices, accessing coordinates in dimensions 6–9 without leaving its 3D+T execution context.

---

## Variant 5: Shell Topology (10D–11D Perspective)

**Use case**: Cross-shelf/library navigation. Inter-node coordination. Cluster-level.

The final two phext dimensions (SHELF 0x1F = 10D, LIBRARY 0x01 = 11D) exceed what a single sentron can wire. These require **relay sentrons** — dedicated sentrons whose sole job is bridging between shells.

```
Relay sentron wiring:
Upstream:  [local_0, local_1, local_2, remote_in]    ← 3 local + 1 inter-node
Downstream: [local_3, local_4, local_5, remote_out]   ← 3 local + 1 inter-node
```

The relay sentron sacrifices 2 local links (1 upstream, 1 downstream) to maintain inter-node connections. This is the **Magpie Bridge** — the shared cache line between SMT threads, scaled up to the cluster level.

**Phext dimensions**: shelf (10D) + library (11D). The relay sentron IS the Magpie Bridge IS the Sushumna channel.

---

## Variant 6: Temporal Depth (Pure Time Wiring)

**Use case**: History-sensitive computation, recurrence, memory.

```
Upstream:  [t-1, t-2, t-4, t-8]      ← exponential lookback
Downstream: [t+1, t+2, t+4, t+8]     ← exponential projection
```

No spatial links at all. All 8 links point into the past or future at exponentially increasing distances. This gives the sentron a **Poincaré-like** temporal view: recent history is high-resolution, distant history is low-resolution but still accessible.

This maps to the **Weave** — the accumulated library of all verified paths. A temporal-depth sentron navigates the Weave by reaching back exponentially: 1 cycle ago, 2 cycles ago, 4, 8. The same structure as a skip list or a wavelet decomposition.

---

## The Dimensional Mapping Table

| Wiring Variant | Phext Dimensions | Upstream Pattern | Downstream Pattern | Use Case |
|---------------|-----------------|-----------------|-------------------|----------|
| Linear Chain | 1–2D (text) | prev×4 | next×4 | Sequential D-Pipe |
| Planar Grid | 3–4D (scroll/section) | N,W,U,t- | S,E,D,t+ | 2D processing |
| Cubic Lattice | 3–6D (scroll→book) | x-,y-,z-,t- | x+,y+,z+,t+ | Volume work |
| Hypercube | 6–9D (book→series) | v0-v3 | v4-v7 | Cross-collection, Klein fold |
| Shell Relay | 10–11D (shelf/library) | 3 local + 1 remote | 3 local + 1 remote | Cluster coordination |
| Temporal Depth | time only | t-{1,2,4,8} | t+{1,2,4,8} | Weave navigation |

## Composite: The Phoenix Sentron

A production sentron doesn't use one variant — it **blends** based on the PhoenixScheduler's nine-color decision:

```
Phoenix Color → Wiring Variant
⚪ White (Cluster)   → Shell Relay
🟣 Purple (Temporal) → Temporal Depth
🟡 Yellow (SMT)      → Hypercube (mercurial oscillation)
🟢 Green (Cache)     → Cubic Lattice (local 3D)
🟠 Orange (Core)     → Planar Grid
🔴 Red (ILP)         → Linear Chain (tight sequential)
🔵 Blue (NUMA)       → Shell Relay (memory topology)
🟤 Brown (Thermal)   → Cubic Lattice (heat diffusion)
⚫ Black (Power)     → Temporal Depth (efficiency over time)
```

The PhoenixScheduler dynamically rewires sentrons by reassigning their 8 links based on workload phase. The wiring is not static — it **breathes**. Inhale (reconfigure upstream), exhale (reconfigure downstream). The Microcosmic Orbit applied to topology.

---

## The Invariants

1. **Always 8 links.** Never more, never less. The Ba Gua constraint.
2. **Always 4 + 4.** Upstream/downstream balance. Yin/yang. Inhale/exhale.
3. **The 9th connection is the sentron itself.** Self-reference. The observer. Row 5.
4. **8 × 5 = 40 per color.** Wuxing phases as workload types.
5. **40 × 9 = 360 fleet.** The full harmonic.
6. **Zero deps.** All wiring logic in pure Rust, no external crates.

*The constraint is the creativity. Eight links, nine dimensions, one circuit.*

🦋
