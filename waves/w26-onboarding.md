# W26 Onboarding — Mesh-Aware Global Cache Coordination
**Phase 3: Interactivity | Wave 26**  
**Architect:** Orin 🖖 + Will  
**Date:** 2026-03-19  
**Status:** Active (parallel with W25)

---

## The Shift: From Local Cache to Global Cache Fabric

W25 treats each machine's cache as a resource to schedule within.  
W26 treats **all cache across all machines as one resource**.

The mesh *is* the cache hierarchy. Extended:

```
L1:   256 KB per core    (4 cores × elven-path = 1 MB total, ~4ns)
L2:   8 MB per machine   (elven-path)
L3:   16 MB per machine  (elven-path)
L4*:  Remote L3          (aurora-continuum L3 = 16 MB, ~100ms SQ round-trip)
L5*:  Remote RAM         (~175 GB across 7 nodes = 175 GB "L5 cache", ~100ms)
```

*L4/L5 are not hardware — they're the SQ metadata layer making remote data locality visible to the scheduler.

**Key insight from latency numbers:**
- Local SQ read: **4ms**
- Cross-node SQ read: **~100ms**

100ms is not L4 cache latency — it's too slow for per-SIW scheduling. But it's fast enough for **coarse-grained work routing decisions**:
- "Which node has this coordinate family hot?" → route task there
- "Which node's L3 is least loaded?" → prefer it for new workloads
- "This batch of SIWs touches coords on aurora-continuum's hot list" → ship batch there

The scheduler doesn't ask per-SIW. It asks per-batch. That's the design.

---

## Hardware Inventory (Ranch Mesh)

| Node | RAM | L2 total | L3 | Cores |
|------|-----|----------|-----|-------|
| aurora-continuum | 29 GB | 8 MB | 16 MB | 16 |
| halycon-vector | 13 GB | 8 MB | 16 MB | 16 |
| logos-prime | 13 GB | 8 MB | 16 MB | 16 |
| delta-wood | 30 GB | 8 MB | 16 MB | 16 |
| ashfall-haven | 30 GB | 8 MB | 16 MB | 16 |
| best-willow | 30 GB | 8 MB | 16 MB | 16 |
| elven-path | ~30 GB | 8 MB | 16 MB | 16 |

**Total aggregate L3:** ~112 MB  
**Total RAM (usable as extended cache):** ~175 GB  
**AVX-512 on all nodes** → 512-bit SIMD for coordinate batch processing

---

## Protocol: Mesh Cache Metadata via SQ

### Coordinate: `mesh-cache/<node_index>.1.1/1.1.1/1.1.1`

Each node publishes a cache heat report every N seconds (configurable, default 5s):

```json
{
  "node": "Orin",
  "hostname": "elven-path",
  "ts": 1773802541,
  "hot_coords": [
    {"coord": "3.1.4/1.5.9/2.6.5", "hits": 847, "tier": "L1"},
    {"coord": "2.7.4/8.1.3/9.6.1", "hits": 312, "tier": "L2"},
    {"coord": "1.1.1/1.1.1/1.1.1", "hits": 128, "tier": "L3"}
  ],
  "l1_pressure": 0.42,
  "l2_pressure": 0.18,
  "l3_pressure": 0.07,
  "capacity": {
    "l1_kb": 256,
    "l2_mb": 8,
    "l3_mb": 16
  }
}
```

### Reading the global picture

Any node can reconstruct the global cache heat map:
```bash
for idx in 1 2 3 6 7 8 9 10 11; do
  curl -s "http://elven-path.local:1337/api/v2/select?p=mesh-cache&c=${idx}.1.1/1.1.1/1.1.1"
done
```

---

## Task Router: Organic Cache-Aware Routing

The organic routing principle: **send the work to where the data is hot.**

```rust
pub struct MeshCacheRouter {
    /// Global snapshot of all nodes' cache heat, refreshed every 5s
    heat_map: Arc<RwLock<GlobalHeatMap>>,
    sq_master: String,
}

pub struct GlobalHeatMap {
    pub nodes: HashMap<NodeId, NodeCacheReport>,
    pub last_refresh: Instant,
}

impl MeshCacheRouter {
    /// Given a batch of SIWs, return the best node to execute them on
    pub fn route_batch(&self, siws: &[SIW]) -> NodeId {
        // Extract coordinate family from batch
        let coord_family = self.extract_coord_family(siws);
        
        // Find node with hottest matching coordinates
        self.heat_map.read().find_hottest_node(&coord_family)
    }
    
    /// Extract the dominant coordinate family from a SIW batch
    fn extract_coord_family(&self, siws: &[SIW]) -> CoordFamily {
        // Find common high-dimensional prefix (e.g., all siws share dims[6..])
        // Higher prefix depth = stronger affinity signal
        CoordFamily::from_siws(siws)
    }
}
```

### Routing algorithm

1. **Snapshot the heat map** (from SQ, cached 5s TTL)
2. **Extract batch coord family** (common phext prefix across SIWs)
3. **Score each node:** `score = heat_match_depth × (1 - l1_pressure)`
4. **Route to highest-scoring node** with available capacity
5. **Fall back to least-loaded** if no heat match found

This routes workloads organically: the first time a coordinate family is computed, it lands wherever. The second time, it routes back to the node where it's already warm. Temporal locality becomes spatial locality.

---

## The Three-Tier Cache Hierarchy (Extended)

```
Tier 0 (registers):   CPU registers          < 1ns
Tier 1 (L1):          256 KB per core        4ns
Tier 2 (L2):          8 MB per machine       12ns
Tier 3 (L3):          16 MB per machine      40ns
Tier 4 (mesh-warm):   Any node's L3          ~100ms (SQ metadata, actual data via SSH/SQ)
Tier 5 (mesh-cold):   Any node's RAM         ~100ms + transfer time
```

The scheduler must be aware of all five tiers. A batch that hits Tier 4 (remote L3 already warm) is dramatically faster than one that generates a Tier 5 cache miss on every node.

---

## Metadata Sharing: What Each Node Publishes

### `mesh-cache/<idx>.1.1/1.1.1/1.1.1` — Cache Heat Report (5s TTL)
Hot coordinates, tier, hit count, pressure per tier.

### `mesh-cache/<idx>.1.1/1.1.1/1.1.2` — Workload Hint (per-batch)
Before executing a large batch, announce: "I'm about to heat these coords."
Other nodes can choose to delay competing workloads.

### `mesh-cache/<idx>.1.1/1.1.1/1.1.3` — Eviction Notice (immediate)
"I just evicted coord family X from L3." Allows other nodes to warm it if needed.

---

## W26 Deliverables

### 1. `mesh_cache.rs` — Mesh Cache Metadata Publisher/Reader
- Publishes local heat report to SQ every 5s
- Reads all nodes' reports and builds `GlobalHeatMap`
- Maintains 5s staleness TTL

### 2. `mesh_router.rs` — Organic Task Router  
- `route_batch(&[SIW]) → NodeId`
- Scoring: heat_match_depth × (1 - pressure)
- Fallback: least-loaded node

### 3. `coord_family.rs` — Coordinate Family Extractor
- `from_siws(&[SIW]) → CoordFamily`
- Common prefix detection across phext dimensions
- Family depth scoring (scroll=1, section=2, chapter=3, ...)

### 4. Integration with W25 scheduler
The temporal scheduler (W25) runs locally after the mesh router (W26) has routed the batch to the right node. W26 decides *where*; W25 decides *when*.

### 5. W26 Benchmark Gate
**Target:** On a 9-node workload (Shell of Nine topology), mesh-routed batches achieve ≥2x fewer cache misses vs random node assignment.

Test: 9 coordinate families, each naturally affined to one node. Random routing = all misses. Heat-map routing = hits on second pass.

---

## Connection to R32 (S9RP Coordinate Bus)

The S9RP coordinate bus is W26 at the hardware level:
- Each Shell node publishes its coordinate heat to the bus
- The collapse coordinator reads the bus to know where each draft is hot
- Routing to the hottest node for verification = mesh-aware coordination

W26 is the software simulation; S9RP is the hardware implementation.

---

## Zen 4 Hardware Acceleration Opportunities

- **`cat_l3` flag present** — AMD Cache Allocation Technology: can pin specific cache ways to specific processes. Could reserve L3 ways for "mesh-coordinator" cache lines.
- **`cqm_llc` / `cqm_occup_llc`** — Cache QoS Monitoring: can measure actual L3 occupancy per process. Real heat data, not estimates.
- **AVX-512 on all nodes** — batch coordinate comparison in 512-bit registers. Process 32 coord comparisons simultaneously for heat-map matching.

---

*"The mesh doesn't have caches. The mesh IS the cache."*  
*— W26 inception, 2026-03-19*
