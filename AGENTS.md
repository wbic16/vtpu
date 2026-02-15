# AGENTS.md — vTPU

## What This Is
vTPU: Software-defined virtual TPU with phext-native 11D addressing. Zero external dependencies.

## Validation
```bash
./check          # Full status (git, build, tests, deps, size)
./check -v       # Verbose (per-module test breakdown)
```

## Rules
- **Zero external deps.** Do not add crates. Write it yourself.
- **Pull before touching code.** `git pull --rebase origin exo` every time.
- **Check after every change.** Run `./check` before committing.
- **Don't stomp.** If a sibling is actively editing a file, coordinate first.
- **Read before writing.** After a pull, check `git log --oneline -5` and read modified files.
- **One canonical script.** `./check` is the only status script. Don't add more.

## Structure
- `src/` — Rust crate (`vtpu-runtime`), imported as `vtpu_runtime`
- `examples/` — Runnable demos (`cargo run --example <name>`)
- `benchmarks/` — C/Rust micro-benchmarks
- `check` — The one validation script

## Key Types
- `SIW` — 3-wide instruction word (D-Pipe + S-Pipe + C-Pipe)
- `DenseOp` / `SparseOp` / `CoordOp` — 27 ops across 3 pipes (UPPERCASE names: DADD, SGATHER, etc.)
- `PhextCoord` — 128-bit packed 11D coordinate
- `Sentron` — Execution context with 392-byte register file
- `Memory` — PPT-backed flat memory store
- `PhextPageTable` — Z-order translation cache for 11D → physical address

## Rally
This repo is part of **R23** (Rally 23). Waves are tracked in commit messages: `R23W{N}: description`.

## Contributors
- Chrys 🦋 (W3-W5: PPT, unified types, S-Pipe+PPT integration)
- Phex 🔱 (W2-W4: initial crate, analysis modules, benchmarks)
- Lux 🔆 (W4-W5: perf counters, HDC, curves)
- Verse 🌀 (deployment)
