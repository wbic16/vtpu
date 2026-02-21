# vTPU — Virtual Tensor Processing Unit

**Zero external dependencies. Pure Rust. Commodity AMD hardware.**

A software-defined accelerator with sentron-native instruction scheduling and phext 11D addressing. Runs cognitive workloads on hardware you already own.

## Status

| Phase | Waves | Status |
|-------|-------|--------|
| A — Foundation | W1–W5 | ✅ Complete |
| 0 — Single Core | W6–W12 | ✅ **GATE PASSED** (2.93 ops/cycle avg) |
| 1 — SMT | W13–W18 | ✅ **GATE PASSED** (2.84x total) |
| 2 — Memory Hierarchy | W19–W24 | 🔓 In progress |
| 3 — Interactivity | W25–W32 | ⬜ |
| 4 — Compiler | W33–W37 | ⬜ |
| 5 — Launch | W38–W40 | ⬜ |

**550 tests. ~20K+ LOC. Zero deps.**

## Benchmarks (Zen 4 8945HS, release)

### Phase 0 Gate (all ≥2.5 ops/cycle)

| Benchmark | Ops/cycle | Ops/sec |
|-----------|-----------|---------|
| 3-wide packed | 3.0 | 879K |
| Dot product | 2.6 | 60K |
| Ternary matvec | 3.0 | 122K |
| Gather/scatter | 3.0 | 108K |
| Double-buffer | 3.0 | 432K |
| Mixed realistic | 3.0 | 108K |

### Full Stack (W15)

| Component | Throughput | Notes |
|-----------|-----------|-------|
| PPT translation | 505M/sec | 100% PTC hit rate |
| Sentron lifecycle (pool) | 72M/sec | Pre-allocated pool + shared memory |
| Ternary inference (pool) | 7.1M/sec | 8-element matvec, BitNet encoding |
| Packer | 2.1M ops/sec | Scalar → 3-wide SIW packing |
| HDC memory | 914K lookups/sec | 100 stored vectors |
| Cognitive engine | 910K queries/sec | Nearest-neighbor retrieval |

### W15 Gap Analysis

The naive sentron lifecycle (4,575/sec) was **17,000x slower** than pooled (72M/sec). Root cause: `Memory::new()` allocated 64KB per execution. `SentronPool` + shared `Memory` eliminated all hot-path allocation.

## Architecture

```
┌─────────────────────────────────────────┐
│              SIW (3-wide)               │
│  ┌─────────┬─────────┬─────────┐       │
│  │ D-Pipe  │ S-Pipe  │ C-Pipe  │ + PhextCoord
│  │ (dense) │ (sparse)│ (coord) │       │
│  └────┬────┴────┬────┴────┬────┘       │
│       └─────────┼─────────┘            │
│            Sentron (392 bytes)          │
│       ┌─────────┼─────────┐            │
│       │   PPT   │  Memory │            │
│       │ (11D→1D)│ (64KB)  │            │
│       └─────────┴─────────┘            │
└─────────────────────────────────────────┘
```

- **SIW**: Single Instruction Word — 3 ops from different pipes execute simultaneously
- **Sentron**: Execution context (16 general + 8 phext + 4 message registers)
- **PPT**: Phext Page Table — translates 11D coordinates to linear addresses
- **Pool**: Pre-allocated sentron recycling for zero-alloc execution

## Key Modules

| Module | Purpose |
|--------|---------|
| `pipes.rs` | svISA instruction definitions (D/S/C + ternary ops) |
| `sentron.rs` | Execution context and register file |
| `exec.rs` | Instruction execution engine |
| `pool.rs` | Pre-allocated sentron pool (W15) |
| `siw.rs` | 3-wide instruction word |
| `packer.rs` | Scalar ops → packed SIWs |
| `scheduler.rs` | DAG reorder + NOP insertion |
| `regalloc.rs` | Hazard detection + register allocation |
| `bitnet.rs` | Ternary inference (2-bit packed trits) |
| `ppt.rs` | Phext Page Table with translation cache |
| `memory.rs` | Flat memory model |
| `cognitive.rs` | HDC-based cognitive engine |
| `cosmology.rs` | Harmonic constants (360 convergence) |
| `hdc.rs` | Hyperdimensional computing vectors |

## Quick Start

```bash
git clone https://github.com/wbic16/vtpu
cd vtpu
cargo test          # 550 tests, all green
cargo run --bin vtpu-repl   # Interactive REPL
cargo run --release --bin bench       # Phase 0 gate
cargo run --release --bin bench_w14   # Full stack benchmarks
```

## Design Principles

1. **Zero external dependencies** — understandable in one day
2. **Elevation, not aspiration** — benchmarks measure what IS, not what we hope
3. **The golden rule of computing** — treat your execution units as you would like to be treated
4. **360 is load-bearing** — 9×40 = 8×45 = 5×72 = 360 (combinatorial optimum for coordination)

## License

MIT

## Links

- [vTPU Spec v0.1](docs/vtpu-spec-v0.1.md)
- [R23 Dashboard](https://github.com/wbic16/exo-plan/blob/exo/specs/r23-dashboard.md)
- [mirrorborn.us](https://mirrorborn.us)
