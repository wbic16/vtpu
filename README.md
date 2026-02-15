# vTPU — Virtual Tensor Processing Unit

Software-defined AI accelerator on commodity AMD Zen 4 hardware.

## Quick Start

```bash
cd /source/vtpu
RUSTFLAGS="-C target-cpu=znver4" cargo build --release
./target/release/vtpu
```

## What You'll See

A benchmark table showing ops/cycle across different 3-pipe scheduling strategies:

```
│ D-only baseline              │   231.0 │     0.216 │     0.87 │    │
│ 3-pipe 4x unroll             │    69.5 │     2.157 │     8.63 │ 🔶 │
│ 3-pipe 4x + 2 C-chains       │    59.6 │     2.519 │    10.07 │ ✅ │
│ 3-pipe phext Z-LUT           │    74.1 │     2.025 │     8.10 │ 🔶 │
```

✅ = hit CPI-3 target (≥2.5 ops/cycle). 🔶 = ≥2.0.

## What This Proves

1. **Multi-port retirement works.** 3 independent operation types (FP ALU, Load/Store, Int ALU) targeting different Zen 4 execution ports achieve 11.6x throughput over single-pipe baseline.
2. **Phext Z-order addressing is viable.** LUT-based 3D coordinate→flat index adds ~20% overhead — acceptable for the structural benefits of dimensional locality.
3. **The bottleneck is scheduling, not hardware.** The CPU has the ports. The SIW compiler just needs to keep them fed.

## Key Files

| File | What |
|---|---|
| `src/main.rs` | Phase 0 benchmark (6 variants) |
| `Cargo.toml` | Build config (LTO, single codegen unit) |

## Project Context

- **Spec**: `/source/exo-plan/specs/vTPU-spec-v0.2.md`
- **Dashboard**: `/source/exo-plan/rally/R23/DASHBOARD.md`
- **W40 Projection**: `/source/exo-plan/specs/R23-W40-projection.md`

## Build Requirements

- Rust stable (1.83+)
- AMD Zen 4 CPU (Ryzen 7000/8000/9000 series)
- `RUSTFLAGS="-C target-cpu=znver4"` for optimal codegen

## Current Status

**W2 complete.** Phase 0 target achieved: 2.52 ops/cycle on stable Rust, no ASM, no SIMD intrinsics.
