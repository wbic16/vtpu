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

## All Rust Modules

TBD: This list needs to be reviewed and documented.

affinity.rs
alaya.rs
analysis/cache_thrash.rs
analysis/locality.rs
analysis/memory_patterns.rs
analysis/mod.rs
analysis/port_conflicts.rs
analysis/smt.rs
assoc.rs
bac.rs
base256_ops.rs
base256.rs
belief.rs
bin/asi.rs
bin/base256.rs
bin/bench_coop.rs
bin/bench_ppt.rs
bin/bench.rs
bin/bench_ttsm.rs
bin/bench_w14.rs
bin/bench_w15_fleet.rs
bin/bench_w16_smt.rs
bin/bench_w17_affinity.rs
bin/bench_w17_redux.rs
bin/bench_w29.rs
bin/phase0_benchmark.rs
bin/siw-gen.rs
bin/vtpu-repl.rs
bin/w15_gap_benchmark.rs
bin/w15_packed_benchmark.rs
bin/w15_siw_benchmark.rs
bin/w16_packing_demo.rs
bin/w16_smt_baseline.rs
bin/w16_smt_dual.rs
bin/w16_smt_using_existing.rs
bin/w17_topology.rs
bin/w18_sentron_topology.rs
bin/w19_octawire_bench.rs
bin/w20_batch_bench.rs
bin/w21_dispatch_bench.rs
bin/w21_simd_bench.rs
bin/w22_alignment_bench.rs
bin/w22_flux_demo.rs
bin/zuna_exec.rs
bitnet.rs
cache_sim.rs
cluster.rs
cognitive.rs
context.rs
coop_fleet.rs
coop_smt.rs
cosmology.rs
cost.rs
c_pipe.rs
cpu_sched.rs
curves/hilbert.rs
curves/mod.rs
curves/zorder.rs
demon.rs
display.rs
easter_island.rs
eeg_bridge.rs
eggs.rs
epoch_ppt.rs
exec.rs
fleet.rs
flux.rs
harmonic.rs
harmonics.rs
hdc_optimized.rs
hdc.rs
iching.rs
integration.rs
intent.rs
karma.rs
lib.rs
memory.rs
narrator.rs
neuron.rs
orin.rs
packer.rs
perf.rs
phext_coord.rs
phoenix_scheduler.rs
pipes.rs
pool.rs
ppt.rs
prefetch.rs
redux.rs
regalloc.rs
repl.rs
scheduler/dag.rs
scheduler/mod.rs
sentron.rs
simd.rs
siw.rs
smt.rs
spanning.rs
sparse_access.rs
sq_client.rs
sq.rs
stream.rs
synchronicity.rs
sysfs.rs
telemetry.rs
temporal.rs
topology.rs
trick.rs
ttsm.rs
twisted_pairs.rs
ubi.rs
validation.rs
wedge.rs
zuna.rs

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
