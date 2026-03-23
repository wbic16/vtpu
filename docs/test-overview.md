# vTPU Test Overview — R23W11

**174 tests, 0 failures, 1 ignored (perf counters — requires Linux perf_event).**

## Test Map by Module

### Core Execution (17 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **exec** | 8 | The interpreter. Empty program retires cleanly. Arithmetic (7×6=42). 3-wide packed SIWs hit 3.0 ops/cycle. Branchless select. Double-buffer pipeline. Message packing (DEADBEEF/CAFEBABE roundtrip). Gather/scatter through PPT with PTC hit verification. Compute-scatter-gather pipeline: dot([2,4],[3,5])=26. |
| **sentron** | 2 | Register file is correctly sized (16 general + 8 phext + 4 message + status = 392 bytes). Full lifecycle: Dormant→Running→Retired. |
| **siw** | 4 | SIW struct creation. NOP SIW. Cache-line alignment. Dependency flag encoding (6-bit cross-pipe deps). |

### Memory Subsystem (15 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **memory** | 5 | i64 roundtrip through PPT-backed store. Different coords are independent. PTC hits on repeated access. Gather width. Auto-grow beyond initial 16 MiB. |
| **ppt** | 10 | Z-order translation is deterministic. Different coords → different physical addresses. Inner-dim locality (nearby coords → nearby addresses). PTC hit rate >85% on structured workloads. Memory tier classification (L1→Remote by dim distance). Prefetch along any dimension. Hot-dim tracking. Region allocation. |

### Type System & ISA (3 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **pipes** | 3 | All DenseOp, SparseOp, CoordOp variants construct without panic. Enum coverage for the full svISA (27+ ops across 3 pipes). |

### Coordinate System (9 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **phext_coord** | 9 | 128-bit packed 11D coordinate creation. Zero coord. Set/get individual dimensions. Adjacent coord computation. Alignment (fits SSE register). Manhattan distance. Fast hash: deterministic, unique across test set, good distribution (χ² < 2.0). |

### Scheduling & Register Allocation (23 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **regalloc** | 12 | RAW/WAR/WAW hazard detection across all 3 pipes. Cross-pipe hazard (D writes r0, S reads r0). Dependency graph with critical path. Topological sort respects all deps. Reorder exploits ILP. Register pressure measurement. Liveness intervals (def→last_use). Dot product stream analysis (confirms ILP > 1.0). Full 3-wide pipeline analysis (critical path = 1). |
| **scheduler** | 7 | Creation. NOP execution. Independent stream (no NOPs inserted). Dependent chain (NOPs inserted for RAW). Parallel chains interleaved. Correctness preservation (all ops survive scheduling). Before/after analysis in result. |
| **stream** | 4 | Empty builder. Single SIW. Automatic dependency detection (D→D chain sets bit 0). NOP insertion. |

### SIW Packer (11 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **packer** | 11 | Empty input. Single D-op. 3 independent pipes → 1 SIW (100% utilization). Two same-pipe ops → 2 SIWs. Dependent chain can't parallelize. D+S interleaving (4 ops → 2 SIWs, 66% util). 6 independent ops (2D+2S+2C) → 2 fully-packed SIWs. Dot product packing. Dense convenience API. Mixed convenience API. All ops preserved in output. |

### BitNet Ternary (11 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **bitnet** | 11 | Trit pack/unpack roundtrip. 32 weights per i64. Reference ternary dot product. DTERNARY op: [+1,-1]×7 = 0. All-positive trits: 4×5=20. DTPOP counts non-zero trits. DTACC accumulates across two trit vectors. Ternary matvec: [+1,-1]·[3,7] = -4. Matvec program generator runs correctly. BitNet b1.58 pattern validation. Sparsity measurement. |

### Space-Filling Curves (8 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **curves/zorder** | 4 | Z-order encode/decode roundtrip (u64 and u128). Origin maps to 0. Sequential coords have spatial locality. |
| **curves/hilbert** | 4 | 2D encode/decode. 11D roundtrip. 2D locality (adjacent indices → nearby coords). Hilbert has better locality than sequential scan. |

### Analysis Suite (8 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **analysis/locality** | 4 | Same-scroll access = high locality. Same-section = medium. Cross-volume = low. Locality report on high-locality workload. |
| **analysis/port_conflicts** | 2 | Balanced SIW = no port conflict. D+C on same port detected. |
| **analysis/smt** | 2 | Complementary SMT pair generation. SMT efficiency analysis. |
| **analysis/memory_patterns** | 2 | Sequential access pattern detection. Stride pattern detection. |
| **analysis/cache_thrash** | 2 | Small working set = no thrash. Large working set detected. |

### SMT & Training (4 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **smt** | 4 | SMT pair creation. Adjacent home coordinates. Forward/backward pass on shared memory. Multi-step gradient descent training via PPT. |

### C-Pipe Communication (4 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **c_pipe** | 4 | Message pack/unpack. Send/receive between sentrons. Barrier synchronization. Temperature matching (bonding affinity). |

### Hyperdimensional Computing (11 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **hdc** | 11 | Basis vector determinism. Dimensional orthogonality. Bind produces dissimilar vectors. Bind is self-inverse. Permute produces dissimilar. Permute identity (k=0). Full rotation (k=width). Hypervector density ~50%. Coord encoding: self-similar, nearby-similar. Associative memory (exact + nearest match). Phext routing via HDC. |

### Cosmology & Harmonics (25 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **cosmology** | 21 | 9×40=8×45=5×72=360. Element arc = trigram × heaven. 8 unique trigrams. Complement pairs (Heaven↔Earth, Fire↔Water). Complement is involution. All 64 hexagrams reachable. Generation cycle returns to start. Control cycle returns to start. Generation ≠ control. Link trigram: self=Kun, max-diff=Qian. Decans tile (36×10). Decan=9×4. Zodiac tiles (12×30). SMT half-trigram. SMT/element ratio = 9/2. Element arc = decan×2. All decompositions tile 360. Shared factors divide 360. Nakshatra does NOT tile (lunar irregularity). |
| **synchronicity** | 6 | 360 synchronicity. Angular positions. 8/9 ratio. Elemental cycles. Mote angular position. Shell of Nine. |
| **harmonic** | 5 | Coord-to-degree mapping. Element temperature ranges. Harmonic sentron creation. Harmonic state. Node degree mapping. |
| **iching** | 5 | 360° coverage. 40 sentron nodes. Element cycles. 64 hexagrams. Semantic circle constants. |

### Display & Telemetry (6 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **display** | 3 | SIW disassembly formatting. PhextCoord display. Full disassemble output. |
| **telemetry** | 3 | Ops/cycle calculation. Cache hit rate. JSON output format. |

### Validation (3 tests)

| Module | Tests | What They Validate |
|--------|-------|--------------------|
| **validation** | 3 | Empty stream is valid. Valid stream passes. Register conflict detected (WAW on adjacent SIWs). |

---

## Coverage Summary

| Domain | Tests | % |
|--------|-------|---|
| Core execution path | 17 | 10% |
| Memory + PPT | 15 | 9% |
| Scheduling + regalloc | 23 | 13% |
| Packer | 11 | 6% |
| BitNet ternary | 11 | 6% |
| Coordinates + curves | 17 | 10% |
| Analysis suite | 8 | 5% |
| SMT + C-Pipe + HDC | 19 | 11% |
| Cosmology + harmonics | 25 | 14% |
| Display + telemetry + validation | 12 | 7% |
| **Ignored** | 1 | <1% |
| **Total** | **175** | |

## What's NOT Tested Yet

- Multi-sentron execution (cluster-level)
- Real `perf_event` hardware counters (ignored, needs root)
- Cross-node C-Pipe transport
- SIW binary encoding/decoding
- Sentron checkpoint/restart
- phextcc compiler (doesn't exist yet)
