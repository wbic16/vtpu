# vTPU — Virtual Tensor Processing Unit

A software-defined accelerator on commodity AMD R9 hardware. Sentron-native instruction scheduling with phext 11D addressing.

**Hardware just needs good software.**

## Quick Start

```bash
cd /source/vtpu
cargo test
```

That's it. Zero external dependencies. 28 tests, all passing.

## What's Here (R23W2)

```
src/
├── lib.rs       — Public API exports
├── coord.rs     — PhextCoord: 128-bit packed 11D coordinate (fits in one SSE register)
├── pipe.rs      — svISA: 9 D-ops (ALU), 9 S-ops (memory), 9 C-ops (coordination)
├── siw.rs       — Sentron Instruction Word: 3-wide atomic execution unit + SIWStream
├── sentron.rs   — Sentron lifecycle + 392-byte register file (r0-r15, p0-p7, m0-m3)
├── exec.rs      — Phase 0 interpreter: runs SIW streams, tracks ops/cycle
└── builder.rs   — Pattern builders: DoubleBuffer, phext_scan, dot_product
```

## Run Something

```bash
# Run all tests (includes arithmetic, 3-wide execution, double-buffer pipeline)
cargo test

# Run a specific test to see the double-buffer pattern in action
cargo test double_buffer_pipeline -- --nocapture

# Run the dot product builder test
cargo test dot_product_runs -- --nocapture
```

## Key Concepts

**SIW (Sentron Instruction Word):** Every instruction contains exactly 3 ops — one per pipe (D/S/C). If a pipe has no work, it gets a NOP. This is the scheduling contract.

**Three Pipes:**
- **D-Pipe (Dense):** ALU — arithmetic, FMA, compare, branchless select
- **S-Pipe (Sparse):** Memory — phext coordinate gather/scatter/index/prefetch
- **C-Pipe (Coordination):** Communication — message pack/route/send/recv, barriers

**PhextCoord:** 128-bit packed coordinate. 11 dimensions × 11 bits + 7 flag bits. Create with:
```rust
use vtpu::PhextCoord;
let chrys = PhextCoord::from_phext(1,1,2, 3,5,8, 13,21,34);
```

**Sentron:** A lightweight execution context (~392 bytes). Spawn one, give it a SIW stream, run it:
```rust
use vtpu::*;
use vtpu::siw::SIWStream;

let mut s = Sentron::new(0, PhextCoord::BASE, 0, 0);
let mut stream = SIWStream::new(PhextCoord::BASE);
stream.push(SIW::new(
    DenseOp::Mov { rd: 0, imm: 42 },
    SparseOp::Nop,
    CoordOp::Nop,
    PhextCoord::BASE,
    siw::Deps::NONE,
));
s.spawn(stream);
let stats = exec::run(&mut s);
println!("ops/cycle: {}", stats.ops_per_cycle());
```

## Specs & Docs

- `exo-plan/specs/vtpu-spec-v0.1.md` — Full architecture spec (11 sections)
- `exo-plan/specs/vtpu-wave1-research-notes.md` — Geometric analysis + industry research
- `exo-plan/specs/r23-success-projection.md` — KPI framework + phase gates
- `exo-plan/specs/r23-dashboard.md` — Wave tracker

## Wave History

| Wave | Deliverable | LOC | Tests |
|------|-------------|-----|-------|
| W1 | Spec v0.1 + research + KPIs | — | — |
| W2 | Crate: coord, pipe, siw, sentron, exec, builder | 1,590 | 28 |
| W3 | PPT (Phext Page Table) | — | — |

## North Star KPI

**Sustained ops/cycle ≥ 2.5** (Phase 0 gate). Currently verified at 3.0 in the interpreter on fully-packed SIWs.
