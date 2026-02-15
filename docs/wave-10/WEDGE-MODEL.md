# The Wedge Model — 22.5° Semantic Architecture

**Discovery Date:** 2026-02-15  
**Context:** R23 Wave 10 (Ancient Wisdom Decoding)  
**Contributor:** Phex 🔱

## Core Concept

**Divide 360-node semantic space into 16 wedges of 22.5 nodes each.**

Each SMT thread owns one wedge. Queries route to best-fit wedge via Hamming distance.

**Analogy:** Egyptian decans divided the sky into 36 wedges of 10° each. We divide meaning-space into 16 wedges of 22.5 nodes each.

## The Math

```
360 nodes total
÷ 16 SMT threads
= 22.5 nodes per thread
≈ 23 nodes (rounded for implementation)
```

**Actual distribution:**
- 8 threads get 23 nodes (8 × 23 = 184)
- 8 threads get 22 nodes (8 × 22 = 176)
- Total: 184 + 176 = 360 ✓

**Or use floating-point wedge boundaries** (22.5 exactly) and handle fractional node ownership via ownership flags.

## Wedge Structure

### Fixed Wedge Assignment

```rust
struct Wedge {
    id: u8,                // 0-15
    start_node: u16,       // Starting node ID (0-359)
    end_node: u16,         // Ending node ID (inclusive)
    thread_id: u8,         // Which SMT thread owns this wedge
    core_id: u8,           // Which physical core (thread_id / 2)
    
    // Semantic boundaries
    coord_min: PhextCoord, // Minimum coordinate in this wedge
    coord_max: PhextCoord, // Maximum coordinate in this wedge
    
    // Node centroids for routing
    nodes: [NodeCentroid; 23], // Up to 23 node centroids
}

struct NodeCentroid {
    id: u16,               // Global node ID (0-359)
    coord: PhextCoord,     // Semantic coordinate (centroid of region)
    sentron: u8,           // Which sentron (0-8) this node belongs to
    element: u8,           // Which Wu Xing element (0-4)
    trigram: u8,           // Which I Ching trigram (0-7)
}
```

### Wedge Boundaries (23-node model)

```
Wedge  0: Nodes   0- 22 (thread 0, core 0)
Wedge  1: Nodes  23- 45 (thread 1, core 0)
Wedge  2: Nodes  46- 68 (thread 2, core 1)
Wedge  3: Nodes  69- 91 (thread 3, core 1)
Wedge  4: Nodes  92-114 (thread 4, core 2)
Wedge  5: Nodes 115-137 (thread 5, core 2)
Wedge  6: Nodes 138-160 (thread 6, core 3)
Wedge  7: Nodes 161-183 (thread 7, core 3)
Wedge  8: Nodes 184-206 (thread 8, core 4)
Wedge  9: Nodes 207-229 (thread 9, core 4)
Wedge 10: Nodes 230-252 (thread 10, core 5)
Wedge 11: Nodes 253-275 (thread 11, core 5)
Wedge 12: Nodes 276-298 (thread 12, core 6)
Wedge 13: Nodes 299-321 (thread 13, core 6)
Wedge 14: Nodes 322-344 (thread 14, core 7)
Wedge 15: Nodes 345-359 (thread 15, core 7) ← only 15 nodes
```

**Wedge 15 adjustment:** Last wedge gets 15 nodes (not 23) to sum to 360.

**Alternative:** Use 22.5 exactly, last two wedges split the 0.5 remainder.

## Routing Algorithm

**Query arrives with phext coordinate → Find best wedge → Dispatch to thread**

```rust
fn route_query(query_coord: PhextCoord, wedges: &[Wedge; 16]) -> u8 {
    // Option 1: Hamming distance to wedge centroids
    let wedge_id = wedges.iter()
        .enumerate()
        .map(|(i, w)| {
            let centroid = w.coord_centroid();  // Average of all node coords in wedge
            (i, hamming_distance(query_coord, centroid))
        })
        .min_by_key(|(_, dist)| *dist)
        .map(|(id, _)| id as u8)
        .unwrap();
    
    wedges[wedge_id as usize].thread_id
}

fn route_query_precise(query_coord: PhextCoord, wedges: &[Wedge; 16]) -> (u8, u16) {
    // Option 2: Hamming distance to all 360 node centroids, return (thread, node)
    let mut best_node = 0u16;
    let mut best_dist = u32::MAX;
    let mut best_thread = 0u8;
    
    for wedge in wedges {
        for node in &wedge.nodes {
            let dist = hamming_distance(query_coord, node.coord);
            if dist < best_dist {
                best_dist = dist;
                best_node = node.id;
                best_thread = wedge.thread_id;
            }
        }
    }
    
    (best_thread, best_node)
}
```

**Trade-off:**
- **Coarse routing** (wedge centroid): Faster, less precise
- **Fine routing** (all 360 nodes): Slower, more precise

**Recommendation:** Use **coarse routing** for initial dispatch, then **fine routing within wedge** if needed.

## Sentron Mapping

**Sentrons don't map 1:1 to wedges.** They're **logical overlays.**

**Each sentron covers 40 nodes** = ~1.74 wedges (40 / 23 ≈ 1.74)

**Example:**
- Sentron 0 (nodes 0-39) spans Wedges 0-1 (nodes 0-45)
- Sentron 1 (nodes 40-79) spans Wedges 1-3 (nodes 23-91)
- Sentron 2 (nodes 80-119) spans Wedges 3-5 (nodes 69-137)
- etc.

**Sentron boundaries cross wedge boundaries.** This is fine — sentrons are **coordination abstractions**, not hardware units.

### Sentron Coordinator Pattern

```rust
struct SentronCoordinator {
    id: u8,                   // 0-8
    node_range: Range<u16>,   // 40 nodes (e.g., 0-39)
    wedge_ids: Vec<u8>,       // Which wedges cover this sentron (usually 2)
    element_regions: [Range<u16>; 5],  // 5 Wu Xing element regions (8 nodes each)
}

impl SentronCoordinator {
    fn dispatch(&self, query: PhextCoord) -> u8 {
        // Find best node within this sentron's 40-node range
        let node_id = self.best_node(query);
        
        // Map to wedge
        let wedge_id = node_id / 23;  // Integer division
        
        // Return thread ID
        wedge_id as u8
    }
}
```

**Sentrons provide semantic clustering** (similar queries go to same sentron), while **wedges provide hardware mapping** (sentron nodes distributed across threads).

## Wu Xing (Five Elements) in Wedge Model

**Each sentron has 5 elemental regions** (8 nodes each, one per trigram).

**Sentron 0 (nodes 0-39):**
- Element 0 (Wood): Nodes 0-7
- Element 1 (Fire): Nodes 8-15
- Element 2 (Earth): Nodes 16-23
- Element 3 (Metal): Nodes 24-31
- Element 4 (Water): Nodes 32-39

**Mapped to wedges:**
- Wedge 0 (nodes 0-22) covers Elements 0-2 fully, plus 6 nodes of Element 3
- Wedge 1 (nodes 23-45) covers 2 nodes of Element 3, all of Element 4, plus next sentron's Element 0

**Elements cross wedge boundaries.** Not a problem — **temperature-weighted routing handles fractional element membership**.

## I Ching (Eight Trigrams) in Wedge Model

**Each element has 8 trigram states** (one node per trigram).

**Element structure within sentron:**
```
Element 0: Nodes 0-7 (☰☱☲☳☴☵☶☷)
Element 1: Nodes 8-15 (☰☱☲☳☴☵☶☷)
Element 2: Nodes 16-23 (☰☱☲☳☴☵☶☷)
Element 3: Nodes 24-31 (☰☱☲☳☴☵☶☷)
Element 4: Nodes 32-39 (☰☱☲☳☴☵☶☷)
```

**Wedge 0 (nodes 0-22) contains:**
- Full trigram cycle for Elements 0-1 (16 nodes)
- Full trigram cycle for Element 2 (8 nodes) ← lands exactly at node 23!

**Wedge boundaries naturally align with trigram cycles.** Almost.

**Wedge 0 ends at node 22** = 2 full elements (16 nodes) + 6 nodes of Element 2 (partial trigram set).

**This is harmonic.** Wedge boundaries don't align perfectly, but they're **semantically smooth** — temperature weighting ensures graceful falloff at boundaries.

## Cache Locality

**Zen 4 cache hierarchy:**
- **L1D:** 32 KB per core (shared by 2 SMT threads)
- **L2:** 1 MB per core (shared by 2 SMT threads)
- **L3:** 16 MB shared across all 8 cores

**Wedge model benefits:**

**L1 locality:**
- Each wedge = 23 nodes × ~128 bytes per node = ~2.9 KB
- Both wedges on a core = ~5.8 KB
- Fits comfortably in 32 KB L1D ✓

**L2 locality:**
- Both wedges + working buffers + prefetch = <<1 MB
- Excellent L2 hit rate expected

**L3 locality:**
- All 360 nodes × 128 bytes ≈ 45 KB
- Entire semantic space fits in L3 with room to spare
- Cross-core queries hit L3, not DRAM

**Prefetching strategy:**
- Thread fetches its 23-node wedge into L1 on startup
- Keeps wedge hot in L1/L2
- Queries within wedge = L1 hit
- Queries outside wedge = L3 hit (other thread's wedge)

## Double-Buffer Pattern (Wave 8 Target)

**SMT thread pair coordination:**

**Thread 0 (Wedge 0, nodes 0-22):**
- Compute batch [K] using nodes 0-22
- While computing, Thread 1 prefetches batch [K+1]

**Thread 1 (Wedge 1, nodes 23-45):**
- Prefetch batch [K+1] into L1/L2
- While prefetching, Thread 0 computes batch [K]

**Alternate roles each batch:**
- Batch K+1: Thread 1 computes, Thread 0 prefetches
- Batch K+2: Thread 0 computes, Thread 1 prefetches

**Cache benefits:**
- Compute thread keeps its wedge hot in L1
- Prefetch thread warms L2 for next batch
- Minimal L1 thrashing (threads work on different wedges)

## Port Contention Mitigation

**Zen 4 execution ports** (simplified):
- 4 × ALU (integer arithmetic)
- 3 × AGU (address generation)
- 4 × FPU (floating point)

**SMT threads share ports** → potential contention.

**Mitigation via wedge model:**

**Thread 0 (even thread):**
- Focuses on **D-Pipe** (dense compute, ALU-heavy)
- Uses trigram state machines (integer logic)

**Thread 1 (odd thread):**
- Focuses on **S-Pipe** (memory ops, AGU-heavy)
- Uses gather/scatter (address generation)

**Result:** Threads use **different execution ports** → reduced contention.

**Measured in Wave 15** (port contention analysis).

## Temperature-Weighted Fuzzy Boundaries

**Wedge boundaries are soft, not hard.**

**Query near wedge boundary** (e.g., node 22-23):
- High temperature: Both wedges equally valid (broad match)
- Low temperature: Strict wedge assignment (nearest node wins)

**Implementation:**
```rust
fn route_with_temperature(query: PhextCoord, temp: f32) -> u8 {
    let scores: Vec<f32> = wedges.iter()
        .map(|w| {
            let dist = hamming_distance(query, w.centroid());
            (-dist as f32 / temp).exp()  // Softmax-style
        })
        .collect();
    
    // High temp: flat distribution (any wedge ok)
    // Low temp: sharp peak (one wedge dominates)
    
    sample_from_distribution(scores)
}
```

**Temperature controls "how Egyptian" we are:**
- High temp = "any decan in the sky is visible" (broad search)
- Low temp = "only the rising decan is visible" (exact match)

## Coordinate Space Initialization

**How do we assign phext coordinates to 360 nodes?**

**Option 1: Hierarchical Lattice**

9 sentrons × 40 nodes = 360  
Assign coordinates based on hierarchical position:

```
Sentron 0: 1.1.1/1.1.1/1.1.1 → 1.1.1/1.1.1/1.5.8
Sentron 1: 1.1.1/1.1.1/2.1.1 → 1.1.1/1.1.1/2.5.8
...
Sentron 8: 1.1.1/1.1.1/9.1.1 → 1.1.1/1.1.1/9.5.8
```

**Each sentron's 40 nodes** span a sub-region of 11D space.

**Option 2: Semantic Clustering**

Use **real phext corpus** (e.g., CYOA, Incipit) to:
1. Extract all phext coordinates from existing documents
2. Cluster into 360 regions via k-means or hierarchical clustering
3. Use cluster centroids as node coordinates

**Advantage:** Nodes align with **actual semantic structure** of existing phext corpus.

**Option 3: Random Initialization**

Generate 360 random coordinates, train via **coordinate adjustment** during queries:
- Query arrives with coord C
- Routed to node N
- If mismatch, nudge node N's centroid slightly toward C

**Self-organizing map** style learning.

**Recommendation for Phase 1:** Use **Option 1** (hierarchical lattice) for deterministic testing. Explore Option 2/3 in Phase 2+.

## Cross-Wedge Communication (C-Pipe)

**What if a query needs nodes from multiple wedges?**

**C-Pipe message passing** enables cross-thread coordination:

```rust
// Thread 5 needs data from Thread 11's wedge
thread_5.send_c_pipe_message(
    target_thread: 11,
    coord: PhextCoord::new(3, 7, 2),
    op: CPipeOp::Gather
);

// Thread 11 receives, processes, responds
thread_11.recv_c_pipe_message() → processes → sends result back

// Thread 5 receives result
thread_5.recv_c_pipe_result() → continues computation
```

**Latency:** L3 hit (~10-20 cycles on Zen 4) + message overhead.

**Measured in Wave 20** (C-Pipe inter-thread transport).

## Summary

**Wedge model key properties:**
1. **22.5 nodes per thread** (360 / 16 = 22.5)
2. **Fixed assignment** (wedge boundaries don't change)
3. **Cache-friendly** (contiguous node ranges, fits L1/L2)
4. **Sentron-agnostic** (sentrons overlay wedges, not replace)
5. **Temperature-weighted** (soft boundaries, fuzzy matching)
6. **Egyptian-inspired** (2.25 decans per thread, ancient harmonic structure)

**Implementation roadmap:**
- **Wave 14:** Implement wedge structure, routing algorithm
- **Wave 15:** Measure cache hit rates, port contention
- **Wave 16:** Benchmark 16-thread performance
- **Wave 17:** Tune coordinate → node mapping
- **Wave 18:** MoE routing via S-Pipe integration

---

**22.5° wedges. 16 threads. 360 nodes. Complete semantic coverage.**  
**Ancient sky → Modern hardware → Meaning-space.** 🔱🔥

## References

- Egyptian decans (36 × 10° = 360°, ~2100 BCE)
- AMD Zen 4 architecture (8c/16t, L1/L2/L3 specs)
- Wave 9: Nine-colored phoenix, 360° harmonic tiling
- Wave 10: SMT mapping, 9/2 ratio
