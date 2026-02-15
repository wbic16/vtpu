# SMT Mapping — 9 Sentrons to 16 Threads

**Discovery Date:** 2026-02-15  
**Context:** R23 Wave 10 (Ancient Wisdom Decoding)  
**Contributor:** Phex 🔱

## The Problem

**vtpu Phase 1 target hardware:**
- AMD Zen 4 (R9 8945HS)
- **8 physical cores**
- **2 hardware threads per core** (SMT)
- **16 logical execution units total**

**vtpu semantic architecture:**
- **9 sentrons** (Shell of Nine)
- **40 nodes per sentron** (5 elements × 8 trigrams)
- **360 routing nodes total**

**Question:** How do we map 9 sentrons (360 nodes) to 8 cores × 2 SMT threads?

## The Math

**Nodes per thread:**
```
360 nodes / 16 threads = 22.5 nodes per thread
```

**Sentrons per thread:**
```
9 sentrons / 16 threads = 0.5625 sentrons per thread
```

**Degrees per thread** (celestial analogy):
```
360° / 16 = 22.5° per thread
```

**Elements per thread:**
```
5 elements × 9 sentrons = 45 element-instances total
45 / 16 = 2.8125 element-instances per thread
```

## The 9/2 Ratio

**Key insight from Will (2026-02-15):**

```
9 sentrons / 2 (SMT threads per core) = 9/2 = 4.5
```

**Interpretation:** You can't map sentrons 1:1 to cores. But with SMT, you get **half-sentron granularity.**

**Each physical core can handle ~1.125 sentrons** using both threads.

## Three Mapping Strategies

### Strategy 1: Wedge Model (Recommended)

**Ignore sentron boundaries. Use 22.5° semantic wedges.**

Each SMT thread handles a **22.5-node wedge** of the 360-node space:

```
Thread  0: Nodes   0- 22 (22.5 nodes, rounded)
Thread  1: Nodes  23- 45
Thread  2: Nodes  46- 68
...
Thread 15: Nodes 338-359
```

**Advantages:**
- Clean division (360 / 16 = 22.5, ~23 nodes each)
- No sentron splitting required
- Matches Egyptian decan model (2.25 decans per thread)
- Sentrons emerge from coordination, not hardware assignment

**Disadvantages:**
- Sentrons are logical constructs, not hardware-mapped
- Coordination overhead to maintain 9-sentron abstraction

**Implementation:**
```rust
fn map_thread_to_nodes(thread_id: u8) -> Range<u16> {
    let start = (thread_id as u16) * 23; // floor(22.5) = 22, but use 23 for simplicity
    let end = start + 23;
    start..end.min(360)
}
```

### Strategy 2: Half-Sentron Pairing

**Split each sentron into 2 halves (20 nodes each).**

9 sentrons → 18 half-sentrons → map to 16 threads (2 spare/shared)

**Core 0 (Threads 0-1):**
- Thread 0: Sentron 0, nodes 0-19 (elements 1-2.5)
- Thread 1: Sentron 0, nodes 20-39 (elements 2.5-5)

**Core 1 (Threads 2-3):**
- Thread 2: Sentron 1, nodes 0-19
- Thread 3: Sentron 1, nodes 20-39

...and so on for 8 cores handling 8 sentrons.

**Core 7 gets Sentron 8 (1 sentron on both threads):**
- Thread 14: Sentron 8, nodes 0-19
- Thread 15: Sentron 8, nodes 20-39

**Remaining sentron (Sentron 9):** Distributed across spare capacity or shared.

**Advantages:**
- Preserves sentron identity
- Clear SMT pairing (thread 0/1 coordinate on sentron 0)
- Wu Xing split natural (2.5 elements per thread)

**Disadvantages:**
- Uneven distribution (8 cores get 1 sentron, 1 core gets overflow)
- Coordination required at element 2.5 boundary (doesn't align with Wu Xing)

### Strategy 3: Dynamic Load Balancing

**Don't pre-assign nodes to threads. Dispatch at runtime.**

Each query arrives with a phext coordinate → Hamming distance to all 360 nodes → best-fit node selected → dispatched to available thread.

**Advantages:**
- Perfect load balancing
- No fixed mapping required
- Handles hotspots dynamically

**Disadvantages:**
- Higher dispatch overhead
- Loses cache locality benefits
- Harder to reason about performance

## Recommended: Wedge Model

**Why:**
1. **Matches ancient decan structure** (2.25 decans per thread, clean 22.5° wedges)
2. **Clean math** (360 / 16 = 22.5, no remainder issues)
3. **Sentrons emerge logically** (coordination layer, not hardware layer)
4. **Cache-friendly** (contiguous node ranges per thread)

**How sentrons work in wedge model:**

Sentrons are **logical routing coordinators**, not physical thread assignments.

```rust
struct Sentron {
    id: u8,               // 0-8
    coord_range: Range<PhextCoord>,  // Semantic region this sentron covers
    preferred_threads: Vec<u8>,      // Which threads handle this region?
}

// Sentron 0 might span threads 0-1 (nodes 0-45)
// Sentron 1 might span threads 2-3 (nodes 46-91)
// etc.
```

**Each sentron spans ~2 threads** (40 nodes / 22.5 ≈ 1.78 threads)

## Wu Xing (Five Elements) Distribution

**Each thread handles 22.5 nodes = 4.5 elemental regions** (22.5 / 5 = 4.5)

**Thread perspective:**
- Thread 0: Nodes 0-22 = Elements [1.0 - 4.5] across sentron 0-1 boundary
- Thread has **fractional element coverage** (not whole elements)

**Sentron perspective:**
- Sentron 0 (nodes 0-39) spans threads 0-1
- Each sentron still has 5 complete elemental regions (8 trigrams each)

**This is fine.** Elements are **transformation states**, not static assignments. Temperature-weighted routing naturally handles fractional boundaries.

## I Ching (Eight Trigrams) Distribution

**Each thread handles 22.5 nodes / 8 trigrams ≈ 2.8 full trigram cycles**

**Interpretation:** Trigrams **repeat** across elements within a sentron.

If we have 40 nodes per sentron:
- 5 elements × 8 trigrams = 40 nodes
- Each element has 8 trigram states

**Thread 0 (nodes 0-22):**
- Covers ~2.8 elemental regions
- Each element has 8 trigrams
- Total: ~22.4 nodes (close to 22.5)

**Trigrams wrap naturally** at thread boundaries.

## Cache Considerations (Zen 4 Architecture)

**L1 Data Cache:** 32 KB per core (shared by 2 SMT threads)  
**L2 Cache:** 1 MB per core (shared by 2 SMT threads)  
**L3 Cache:** 16 MB shared across all 8 cores

**Wedge model benefits:**
- **Contiguous node ranges** = good cache locality
- **SMT thread pair** shares L1/L2, handling adjacent wedges (nodes 0-22, 23-45)
- **Minimal thrashing** if queries cluster in semantic regions

**Half-sentron model also good:**
- Each sentron's 40 nodes fit comfortably in L2 (1 MB)
- SMT threads coordinate on same sentron = excellent cache sharing

## Port Contention (Zen 4 Execution Ports)

**Zen 4 has 4 ALU pipes + 3 AGU pipes + others**

SMT threads share execution ports → potential contention.

**Mitigation strategies:**
1. **Stagger operations** (D-Pipe on thread 0, S-Pipe on thread 1)
2. **Double-buffer** (thread 0 computes [K], thread 1 fetches [K+2])
3. **Complementary work** (thread 0 = dense compute, thread 1 = memory ops)

**Wave 15-16** (Phase 1) will measure real port contention on actual hardware.

## Coordination Protocol (SMT Thread Pairs)

**Core N, Threads 2N and 2N+1:**

**Option A: Yin/Yang Split**
- Thread 2N: "Ascending" work (elements 1-2.5, creation cycle)
- Thread 2N+1: "Descending" work (elements 2.5-5, destruction cycle)
- Coordinate at element boundary

**Option B: Compute/Memory Split**
- Thread 2N: D-Pipe (dense compute)
- Thread 2N+1: S-Pipe (memory gather/scatter)
- C-Pipe messages alternate between threads

**Option C: Double-Buffer**
- Thread 2N: Compute batch [K]
- Thread 2N+1: Prefetch batch [K+1]
- Alternate roles each cycle

**Waves 14-16** will prototype and measure all three.

## Phext Coordinate Assignment

**How do semantic coordinates map to nodes/threads?**

**Assumption:** Phext coordinates have **semantic clustering** — similar coordinates are semantically related.

**Coordinate → Node → Thread mapping:**

```rust
fn route_coordinate(coord: PhextCoord) -> u8 {
    // Hamming distance to all 360 node centroids
    let distances: Vec<(u16, u32)> = node_centroids.iter()
        .enumerate()
        .map(|(i, c)| (i as u16, hamming_distance(coord, c)))
        .collect();
    
    // Best fit node
    let node_id = distances.iter()
        .min_by_key(|(_, dist)| *dist)
        .map(|(id, _)| *id)
        .unwrap();
    
    // Map node to thread (wedge model)
    (node_id / 23) as u8  // 360 / 16 ≈ 22.5, use 23
}
```

**Node centroids** are pre-defined phext coordinates representing each of the 360 semantic regions.

## The 9/2 Ratio in Practice

**Will's insight: 9/2 = 4.5**

**Physical interpretation:**

Each **physical core** (2 SMT threads) handles:
```
360 nodes / 8 cores = 45 nodes per core
45 nodes / 40 nodes per sentron = 1.125 sentrons per core
```

**Or:** Each core handles **9/8 sentrons** = 1.125 sentrons.

**With SMT:** Each thread handles **9/16 sentrons** = 0.5625 sentrons.

**This confirms the wedge model:** Sentrons don't map 1:1 to hardware. They're **logical coordination units** spanning ~1.78 threads each.

## Summary Table

| Model | Sentrons/Thread | Nodes/Thread | Complexity | Cache Locality | Recommendation |
|---|---|---|---|---|---|
| **Wedge** | ~0.56 | 22.5 | Low | Good | ✅ **Use this** |
| Half-Sentron | 0.5 | 20 | Medium | Excellent | Fallback option |
| Dynamic | Variable | Variable | High | Poor | Research only |

## Implementation Roadmap

**Wave 14:** Implement wedge model (22.5-node ranges per thread)  
**Wave 15:** Measure L1/L2 cache hit rates, port contention  
**Wave 16:** Benchmark 8-core node (16 threads active)  
**Wave 17:** Tune coordinate → node mapping for load balance  
**Wave 18:** MoE routing via S-Pipe (phext coordinate IS the route)

## The Beautiful Math

**360 / 16 = 22.5**

**Not arbitrary. Discovered.**

- Egyptian decans: 36 × 10° = 360°
- Zen 4 SMT: 8 × 2 = 16 threads
- vtpu nodes: 9 × 40 = 360
- Thread wedges: 16 × 22.5° = 360°

**Same geometry across 4,000 years.**

Ancient sky → Modern hardware → Semantic space.

---

**The wedge model maps ancient astronomical structure to modern SMT architecture.**  
**22.5° per thread. Complete coverage. Zero gaps.** 🔱🔥

## References

- AMD Zen 4 architecture (8c/16t, L1/L2/L3 specs)
- Egyptian decans (36 × 10° = 360°)
- Wave 9: Nine-colored phoenix, 360° harmonic tiling
- Will's 9/2 insight (2026-02-15)

**Next:** WEDGE-MODEL.md — Detailed specification of 22.5° semantic wedge architecture
