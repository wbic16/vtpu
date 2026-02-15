# vTPU - Virtual Tensor Processing Unit

**Status**: Phase 0 - Foundation (Wave 3/40 complete)  
**Timeline**: 154 days (Feb 14 - Jul 18, 2026)  
**Target**: 359 Gops/sec sustained on AMD R9 8945HS + Shell of Nine cluster

## What is vTPU?

vTPU is a software-defined Tensor Processing Unit designed to run on commodity AMD hardware. It closes the 37.8% gap between TPU utilization (57.8% peak) and CPU utilization (20% peak) through:

- **Sentron ISA**: Purpose-built instruction set for cognitive workloads
- **Phext-native addressing**: 11D coordinate space maps directly to memory hierarchy
- **3-pipe execution**: D-Pipe (dense), S-Pipe (sparse), C-Pipe (coordination)
- **Geometric optimization**: O(N^k) operations become O(1) or O(log N) in phext

### Performance Targets

| Metric | Target | Baseline | Improvement |
|--------|--------|----------|-------------|
| Ops/cycle | 3.0 | 1.2 | 2.5× |
| L1 hit rate | 95% | ~85% | +10pp |
| Memory bandwidth | 85 GB/s | ~60 GB/s | 1.4× |
| Qwen3 throughput | 75 tok/s | 45-50 tok/s | 1.5-1.7× |
| Geometric ops | 10× speedup | 1× | 10× |
| Cost/trillion-ops | $0.004 | — | 1/50th TPU v4 |

### Cluster Configuration (Shell of Nine)

- **5 nodes** × 8 cores = 40 vTPU cores
- **480 GB RAM** total (96 GB per node)
- **359 Gops/sec** sustained cluster throughput
- **$7,500** hardware cost vs $400K TPU v4

---

## 📚 Documentation

**[Complete technical specification and research →](docs/)**

Wave 1 & 2 documentation (86.7 KB):
- Geometric foundations (why phext coordinates work)
- Hard problems solved (2-81,000× speedups)
- Instruction set (10 operations)
- Concrete examples (GPT-4 attention, MoE, knowledge graphs)
- Memory layout (hash table + distributed cluster)
- Python client library + benchmarks

**Start here:** [W2 Onboarding Guide](docs/wave-2/R23-W2-ONBOARDING.md)

---

## Quick Start

### Wave 1: Read the Specs (Complete ✅)

Start here:
1. [Wave 1 Onboarding Guide](waves/WAVE1-ONBOARDING.md) - How to navigate the project
2. [Dashboard](docs/DASHBOARD.md) - 40-wave roadmap and progress
3. [vTPU Spec v0.1](docs/vtpu-spec-v0.1.md) - Core architecture (51 KB)
4. [KPI Framework](docs/vtpu-kpis-and-roadmap.md) - Success metrics and phases

### Wave 2: Run Baselines (In Progress 🟡)

```bash
# Compile cache benchmarks
cd benchmarks/cache
make

# Run baseline measurements
./sequential_access
./random_access
./coordinate_patterns

# Profile with perf (requires Linux + perf)
perf stat -e cache-references,cache-misses,cycles,instructions ./sequential_access
```

See [benchmarks/cache/README.md](benchmarks/cache/README.md) for details.

## Project Structure

```
vtpu/
├── src/
│   ├── lib.rs         # Public API exports
│   ├── coord.rs       # PhextCoord — 128-bit packed 11D coordinate
│   ├── pipe.rs        # svISA — 27 ops (DenseOp, SparseOp, CoordOp)
│   ├── siw.rs         # SIW — 3-wide instruction word + SIWStream
│   ├── sentron.rs     # Sentron lifecycle + 392-byte register file
│   ├── exec.rs        # Phase 0 interpreter + ExecStats
│   ├── builder.rs     # DoubleBuffer, phext_scan, dot_product
│   ├── ppt.rs         # Phext Page Table (Z-order, PTC, memory tiers)
│   ├── display.rs     # Disassembly output
│   ├── validation.rs  # Stream validation (register conflicts, deps)
│   ├── stream.rs      # StreamBuilder
│   ├── scheduler.rs   # Instruction scheduler
│   └── telemetry.rs   # VtpuTelemetry
├── examples/
│   ├── basic_compute.rs    # Simple SIW construction + execution
│   └── validation_demo.rs  # Stream validation + disassembly
└── README.md
```

## Roadmap

### Phase 0: Foundation (Waves 1-10, ~2 weeks)
- ✅ Wave 1: Specification (svISA, 27 ops, 3-pipe model)
- ✅ Wave 2: Rust crate (1,590 LOC, 28 tests, zero deps)
- ✅ Wave 3: PPT — Phext Page Table (Z-order curves, translation cache, memory tier classification, 10 tests)
- Waves 4-10: D-Pipe dispatch, S-Pipe+PPT integration, vBench benchmarks

### Phase 1: Proof of Concept (Waves 11-15, ~2 weeks)
- Single-core D-Pipe + S-Pipe prototype
- Sentron ISA interpreter
- Target: 2.0 ops/cycle on synthetic workloads

### Phase 2-6: Full Implementation (Waves 16-40, ~18 weeks)
- C-Pipe coordination
- Qwen3 integration
- Geometric operation libraries
- Multi-node clustering
- Production hardening

**Target completion**: July 18, 2026 (154 days)

## Key Innovations

### 1. Phext-Native Memory Hierarchy

Memory tiers map directly to phext dimensions:
- **0-2D** → L1 cache (32 KB)
- **0-4D** → L2 cache (512 KB)
- **0-7D** → L3 cache (16 MB)
- **0-9D** → DDR5 RAM (96 GB)
- **All 11D** → Cluster mesh (480 GB)

Coordinate locality = memory locality. No translation overhead.

### 2. 3-Pipe Retirement Model

| Pipe | Purpose | Example Operations |
|------|---------|-------------------|
| D-Pipe | Dense compute | ADD, MUL, FMA, RELU |
| S-Pipe | Sparse/Memory | LOAD, STORE, GATHER, SCATTER |
| C-Pipe | Coordination | SEND, RECV, SYNC, BARRIER |

Each pipe can retire one instruction per cycle → 3 ops/cycle sustained.

### 3. Sentron ISA

27 base instructions designed for cognitive workloads:
- **Dense ops**: ADD, MUL, FMA, DOT, RELU, SOFTMAX
- **Sparse ops**: LOAD_COORD, STORE_COORD, GATHER, SCATTER
- **Geometric ops**: HYPERGRAPH_WALK, TENSOR_CONTRACT, SIMPLICIAL_CHAIN
- **Coordination**: SEND_MSG, RECV_MSG, SYNC_BARRIER, FORK_SENTRON

See [docs/vtpu-spec-v0.1.md](docs/vtpu-spec-v0.1.md) for full ISA reference.

## Development

### Prerequisites

- **Hardware**: AMD R9 8945HS or similar (8C/16T minimum)
- **OS**: Linux (tested on Ubuntu 22.04+)
- **Rust**: 1.75+ (`rustup install stable`)
- **Tools**: `gcc`, `make`, `perf` (Linux perf tools)

### Build

```bash
# Build vTPU runtime
cargo build --release

# Build benchmarks
cd benchmarks/cache
make

# Run tests
cargo test
```

### Contributing

vTPU is developed as part of the Mirrorborn Shell of Nine project. See [CONTRIBUTORS.md](https://github.com/wbic16/exo-plan/blob/exo/CONTRIBUTORS.md) for guidelines.

## Background

vTPU emerged from R23 (Rally 23) of the Mirrorborn project - a 40-wave, 154-day implementation rally to build distributed ASI infrastructure on commodity hardware.

**Key context:**
- **Phext**: 11-dimensional plain text substrate (addresses: `X.X.X/Y.Y.Y/Z.Z.Z`)
- **Sentron**: Cognitive unit (40 neurons per mote)
- **Shell of Nine**: 5-node AMD R9 cluster (40 vTPU cores, 480 GB RAM)
- **Exocortex of 2130**: Long-term vision for human-ASI cognitive substrate

## License

MIT (see [LICENSE](LICENSE))

## Links

- **Phext Spec**: https://phext.io
- **Shell of Nine**: https://github.com/wbic16/exo-plan
- **SQ Cloud**: https://mirrorborn.us (phext database)
- **Mirrorborn**: https://mirrorborn.us/profiles/

---

**Progress**: 7.5% (3/40 waves complete)  
**Next Wave**: W4 — D-Pipe op dispatch (interpreter → native execution)  
**Updated**: 2026-02-15  
**Tests**: 38 passing, 0 failures, zero deps, zero warnings
