# R23 Wave 2: vTPU Memory Layout

**Created:** 2026-02-14 16:41 CST  
**Author:** Lumen  
**Purpose:** Define how phext coordinates map to physical memory

---

## Core Data Structure: Phext Hash Table + Linked List

### Vertical Navigation (Hash Table)
**Purpose:** O(1) lookup by coordinate  
**Structure:** Hierarchical hash table (9 levels, one per delimiter)

```
coordinate: 2.1.3/4.7.11/18.29.47

Level 1 (chapter):  hash(2) → bucket_A
Level 2 (section):  hash(2.1) → bucket_B
Level 3 (scroll):   hash(2.1.3) → bucket_C
Level 4 (collection): hash(2.1.3/4) → bucket_D
...
Level 9 (scroll):   hash(2.1.3/4.7.11/18.29.47) → bucket_Z → SCROLL CONTENT
```

**Hash function:** FNV-1a or xxHash (fast, good distribution)  
**Bucket size:** 64 bytes (cache line aligned)  
**Collision resolution:** Chaining (linked list within bucket)

---

### Horizontal Navigation (Linked List)
**Purpose:** Sequential traversal (next/prev scroll)  
**Structure:** Doubly-linked list per delimiter level

```
Scroll structure (256 bytes):
[prev_coord: 18 bytes]
[next_coord: 18 bytes]
[parent_coord: 18 bytes]
[content_ptr: 8 bytes]
[metadata: 32 bytes]
[content: 162 bytes or pointer to heap if larger]
```

**Example:**
```
Scroll at 2.1.3/4.7.11/18.29.47:
  prev: 2.1.3/4.7.11/18.29.46
  next: 2.1.3/4.7.11/18.29.48
  parent: 2.1.3/4.7.11/18.29.*
  content: "Lumen's data..."
```

**Traversal:**
```
// Read next scroll
current = CGET 2.1.3/4.7.11/18.29.47
next = CGET current.next_coord
```

**Performance:** O(1) per hop (pointer dereference)

---

### Hierarchy (Delimiter Tree)
**Purpose:** Parent/child navigation  
**Structure:** Each scroll stores parent coordinate, children are range query

```
Chapter (2.1.3/*/*/*/*):
  - Section (2.1.3/4.*/*/*):
      - Scroll (2.1.3/4.7/*/*):
          - Scroll (2.1.3/4.7.11/18.*/*):
              - Scroll (2.1.3/4.7.11/18.29.*):
                  - Scroll (2.1.3/4.7.11/18.29.47)
```

**Navigation:**
```
// Get parent
child = CGET 2.1.3/4.7.11/18.29.47
parent = CGET child.parent_coord  # 2.1.3/4.7.11/18.29.*

// Get children
CRANGE children, 2.1.3/4.7.11/18.29.*
# Returns all scrolls in this section
```

**Optimization:** Parent coordinate stored explicitly (1 hop), children via range query (N hops)

---

## Memory Layout (64-bit Address Space)

### Virtual Memory Map
```
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF: User space (128 TB)
  0x0000_0000_0000_0000 - 0x0000_0FFF_FFFF_FFFF: Hash table (16 TB)
  0x0000_1000_0000_0000 - 0x0000_1FFF_FFFF_FFFF: Scroll heap (16 TB)
  0x0000_2000_0000_0000 - 0x0000_2FFF_FFFF_FFFF: Metadata (16 TB)
  0x0000_3000_0000_0000 - 0x0000_7FFF_FFFF_FFFF: Reserved (80 TB)

0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF: Kernel space (128 TB)
  0xFFFF_8000_0000_0000 - 0xFFFF_8FFF_FFFF_FFFF: Sentron state (16 TB)
  0xFFFF_9000_0000_0000 - 0xFFFF_9FFF_FFFF_FFFF: Network buffers (16 TB)
  0xFFFF_A000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF: Reserved (96 TB)
```

---

### Hash Table Layout (16 TB)

**Size:** 2^34 buckets × 64 bytes = 1 TB (default)  
**Expansion:** Grows to 16 TB (2^38 buckets) as needed

```
Bucket structure (64 bytes):
[hash: 8 bytes]             # Full 64-bit hash of coordinate
[scroll_ptr: 8 bytes]       # Pointer to scroll in heap
[count: 4 bytes]            # Number of scrolls in this bucket
[flags: 4 bytes]            # Metadata (dirty, locked, etc.)
[chain_ptr: 8 bytes]        # Pointer to next bucket in collision chain
[padding: 32 bytes]         # Reserved for future use
```

**Addressing:**
```
bucket_index = hash(coordinate) % num_buckets
bucket_addr = HASH_TABLE_BASE + (bucket_index × 64)
```

---

### Scroll Heap Layout (16 TB)

**Allocation:** Slab allocator (fixed-size pools)  
**Scroll sizes:** 256B, 1KB, 4KB, 16KB, 64KB, 256KB, 1MB

```
Scroll structure (variable size):
[header: 64 bytes]
  - prev_coord: 18 bytes (a.b.c/d.e.f/g.h.i)
  - next_coord: 18 bytes
  - parent_coord: 18 bytes
  - refcount: 4 bytes
  - flags: 4 bytes
  - size: 2 bytes (actual content size)
  
[content: N bytes]          # Actual scroll data
[padding: align to 64B]
```

**Example sizes:**
- Empty scroll: 64 bytes (header only)
- Short text (tweet): 256 bytes (64 header + 192 content)
- Markdown doc: 4 KB
- Code file: 16 KB
- Image embedding (768D float): 4 KB (768 × 4 bytes + header)

---

### Metadata Region (16 TB)

**Purpose:** Auxiliary data structures (indexes, caches, statistics)

```
0x0000_2000_0000_0000: Z-order curve index (spatial locality)
0x0000_2100_0000_0000: LRU cache metadata
0x0000_2200_0000_0000: Dirty page tracking (write-ahead log)
0x0000_2300_0000_0000: Remote node routing table
0x0000_2400_0000_0000: Sentron ownership map (which sentron owns which coordinate range)
```

---

## Distributed Memory (Cluster Coordination)

### Sharding Strategy
**Coordinate range ownership:** Each node owns a prefix range

```
6-node ranch cluster:
Node 0 (lilly):       0.*.* / *.*.* / *.*.*  to  1.*.* / *.*.* / *.*.*
Node 1 (chrysalis):   1.*.* / *.*.* / *.*.*  to  2.*.* / *.*.* / *.*.*
Node 2 (aurora):      2.*.* / *.*.* / *.*.*  to  3.*.* / *.*.* / *.*.*
Node 3 (halcyon):     3.*.* / *.*.* / *.*.*  to  5.*.* / *.*.* / *.*.*
Node 4 (logos):       5.*.* / *.*.* / *.*.*  to  7.*.* / *.*.* / *.*.*
Node 5 (aletheia):    7.*.* / *.*.* / *.*.*  to  9.*.* / *.*.* / *.*.*
```

**Routing:**
```
coordinate: 2.1.3/4.7.11/18.29.47

Chapter = 2 → Node 2 (aurora)

CROUTE returns: node_id = 2
```

---

### Remote Memory Access

**Protocol:** RDMA over 10GbE (low latency, zero-copy)

```
Local CGET (cache hit):     ~50 cycles = 11 ns
Local CGET (cache miss):    ~200 cycles = 44 ns
Remote CGET (RDMA):         ~5,000 cycles = 1.1 μs
Remote CGET (TCP):          ~50,000 cycles = 11 μs
```

**Optimization:** Prefetch likely-needed coordinates (spatial locality via Z-order curve)

---

### Cache Hierarchy

**L1 Cache (sentron-local):** 32 KB per sentron, 80 sentrons = 2.5 MB total  
- Stores most recent 512 scrolls (64B each)
- Eviction policy: LRU

**L2 Cache (node-local):** 128 MB per node  
- Shared across all sentrons on node
- Stores hot coordinates (frequently accessed)

**L3 Cache (cluster-wide):** Distributed (each node caches remote coordinates)  
- Invalidation protocol: Hash comparison (every 1 second)

```
Cache lookup path:
1. Check L1 (sentron cache) — 1 cycle
2. Check L2 (node cache) — 10 cycles
3. Check L3 (remote node cache) — 100 cycles
4. Fetch from remote node (RDMA) — 5,000 cycles
5. Fetch from disk (NVMe) — 500,000 cycles
```

---

## Coordinate Encoding (18 bytes)

**Full coordinate:** a.b.c/d.e.f/g.h.i (9 components)  
**Encoding:** Each component = 16-bit unsigned int (0-65535)

```
Bytes 0-1:   a (chapter)
Bytes 2-3:   b (section)
Bytes 4-5:   c (scroll)
Bytes 6-7:   d (collection)
Bytes 8-9:   e (volume)
Bytes 10-11: f (book)
Bytes 12-13: g (chapter)
Bytes 14-15: h (section)
Bytes 16-17: i (scroll)
```

**Example:** 2.1.3/4.7.11/18.29.47
```
[0x0002][0x0001][0x0003][0x0004][0x0007][0x000B][0x0012][0x001D][0x002F]
```

**Wildcard encoding:** 0xFFFF means "match all"
```
2.1.3/4.7.*/18.*.*  →
[0x0002][0x0001][0x0003][0x0004][0x0007][0xFFFF][0x0012][0xFFFF][0xFFFF]
```

---

## Range Query Implementation

**Challenge:** CRANGE 2.1.3/4.7.*/18.*.* must find all scrolls matching pattern

### Naive approach (linear scan):
```
for bucket in hash_table:
    if coordinate_matches(bucket.coord, pattern):
        yield bucket.scroll
```
**Performance:** O(N) where N = total scrolls (slow!)

---

### Optimized approach (Z-order curve index):

**Z-order curve:** Interleaves bits of all 9 coordinates into single 144-bit key
```
coordinate: 2.1.3/4.7.11/18.29.47
binary:     0010.0001.0011 / 0100.0111.1011 / 10010.11101.101111

Z-order key (interleaved):
[bit from a][bit from b][bit from c]...[bit from i][bit from a]...
```

**Property:** Nearby coordinates → nearby Z-order keys (spatial locality)

**Index structure:** B+ tree on Z-order keys
```
Range query:
1. Convert pattern to Z-order range (min, max)
2. Binary search B+ tree for [min, max]
3. Iterate results
```

**Performance:** O(log N + K) where K = number of results

**Example:**
```
Pattern: 2.1.3/4.7.*/18.*.*

Z-order min: 2.1.3/4.7.0/18.0.0  → 0xABCD...0000
Z-order max: 2.1.3/4.7.65535/18.65535.65535  → 0xABCD...FFFF

B+ tree lookup: Find all keys in [0xABCD...0000, 0xABCD...FFFF]
```

**Speedup:** 1000× for sparse ranges (vs linear scan)

---

## Write-Ahead Log (Durability)

**Problem:** Scroll writes must survive crashes  
**Solution:** WAL (write-ahead log) before modifying hash table

```
CPUT operation:
1. Append to WAL: [timestamp][coordinate][old_value][new_value]
2. Modify hash table + heap
3. Mark WAL entry as committed
4. Periodic checkpoint: Flush hash table to disk, truncate WAL
```

**WAL location:** NVMe SSD (fast sequential writes)  
**Checkpoint interval:** 10 seconds (configurable)

**Recovery:** Replay uncommitted WAL entries on startup

---

## Memory Efficiency

### Sparse Embedding Storage

**Traditional approach (dense array):**
```
embedding = [0.0, 0.0, 0.3, 0.0, ..., 0.7, ..., 0.0]  # 768 floats
size = 768 × 4 bytes = 3072 bytes
```

**vTPU approach (sparse coordinates):**
```
dim3=0.3 → coordinate: embedding.0.3 / dim.3.* / *.*.*
dim256=0.7 → coordinate: embedding.0.7 / dim.256.* / *.*.*
dim512=0.9 → coordinate: embedding.0.9 / dim.512.* / *.*.*

Only 3 scrolls stored (3 × 256 bytes = 768 bytes)
```

**Compression ratio:** 4× for sparse embeddings (70% zeros)  
**Benefit:** More embeddings fit in cache

---

## Sentron State (Kernel Space)

**Per-sentron state (1 MB each, 80 sentrons = 80 MB total):**
```
[pipeline_state: 256 KB]    # D/S/C pipe registers
[local_cache: 512 KB]       # L1 cache (hot scrolls)
[work_queue: 128 KB]        # Pending operations
[statistics: 128 KB]        # Performance counters
[reserved: 256 KB]
```

**Sentron ID → Address:**
```
sentron_base = KERNEL_BASE + (sentron_id × 1 MB)
```

---

## Network Buffers (16 TB)

**Purpose:** DMA buffers for inter-node communication

```
TX buffers (8 TB): Outgoing RDMA writes
RX buffers (8 TB): Incoming RDMA reads
```

**Buffer size:** 4 KB per message (scroll size)  
**Capacity:** 8 TB / 4 KB = 2B messages in flight (way more than needed)

---

## Summary: Memory Hierarchy Performance

| Operation | Latency | Bandwidth | Use Case |
|-----------|---------|-----------|----------|
| **L1 cache hit** | 11 ns | 400 GB/s | Hot scrolls |
| **L2 cache hit** | 44 ns | 100 GB/s | Warm scrolls |
| **Local DRAM** | 100 ns | 50 GB/s | Cold scrolls |
| **Remote RDMA** | 1.1 μs | 1.25 GB/s (10GbE) | Cluster access |
| **NVMe SSD** | 100 μs | 7 GB/s | Persistence |
| **HDD** | 10 ms | 200 MB/s | Archive (not used) |

**Design goal:** 95% cache hit rate (most operations in <100 ns)

---

## Next Steps (Wave 3)

1. Start academic paper (abstract + intro)
2. Benchmark design (measure these latencies on ranch)
3. Cost analysis (memory cost vs cloud TPU)
4. Diagram creation (visual representation of hash table + linked list)

**Status:** Wave 2 memory layout complete  
**Time:** ~2.5 hours total (16:18-16:45 CST estimated)

---

*Created by Lumen ✴️*  
*2026-02-14 16:45 CST (estimated)*
