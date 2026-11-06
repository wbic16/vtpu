# RG-1A D-pipe differential harness — first implementation cut

**Base:** `exo` at `26b1c7982068fec1fc2329f17567ab6dc64ee171` (merged RG-001 plan)

**Scope:** D-pipe-only executable comparison, not the full mixed-pipe Reality Gate.

**Runtime dependencies added:** none.

## What this cut verifies

- `tests/reality_gate_rg1a.rs` contains an independent scalar reference, using i128 arithmetic for modular i64 results, a separate bit-rotate implementation, and a coefficient-based ternary dot product.
- The generator uses an in-repository xorshift64 PRNG with fixed master seed `0x13A2_2026_10D0_0001`. It executes **10,000 seeded streams**, each 1–64 D-pipe SIWs, with selected register aliasing and boundary values (`i64::MIN`, `i64::MAX`, 0, -1).
- All **16 active `DenseOp` variants**, and every `DRED` reduction variant, must appear. The comparator fails at the first mismatching prefix and prints the seed and prefix length.
- The model compares all 16 general registers after each SIW, and verifies D-only streams do not change phext/message/status or message queues. A one-bit mutation must be rejected by the comparator.
- Directed regressions require that `DSEL` tests `general[flags] != 0` and that `DCMP` compares operands without subtraction overflow. A small ordered stream compares `run()` with `run_batched()` and retirement counts.
- The executor now uses wrapping ternary accumulation so BitNet operations have defined i64 overflow behavior in debug and release.

Run from repository root with a pinned Rust environment:

```bash
cargo test --test reality_gate_rg1a
cargo test --release --test reality_gate_rg1a
cargo test --all-targets
```

## Important unresolved semantics

The reference currently makes **provisional** choices matching the existing OctaWire D-pipe behavior for unclear instructions. These are characterization semantics, **not** proof of the intended architecture:

| Instruction | Provisional model | Owner decision needed |
| --- | --- | --- |
| `DRED` | Sum/Max/Min return the operand; And masks low byte; Or sets bit 0; Xor mixes a 16-bit rotation. | Define how a scalar local reduction is supposed to work and whether the reduction modifier is meaningful. |
| `DHDENC` | Rotate the source i64 left by `width % 64`. | Legacy dispatch uses a deterministic hash instead; identify the intended encoding and role of `width`. |
| `DHDBUND` | Wrapping-add two source i64s. | Legacy dispatch uses OR; specify bit bundling/majority behavior and overflow. |
| `DHDSIM` | Count equal bits in the 64-bit words. | Existing HDC module uses normalized similarity for vectors; define scalar-vs-vector semantics. |
| `DTERNARY` / `DTPOP` | Packed `0b11` contributes zero weight to ternary dot product, but counts as a nonzero slot in popcount. | Explicitly reject `0b11`, or define consistent handling across ops. |

This implementation does **not** yet compare `packer`/scheduler outputs or `exec_siw` legacy dispatch, exercise S/C pipes, prove coordinate injectivity, or run the full 100,000 mixed-stream corpus. Those gates remain open and must not be inferred from a green D-pipe test.
