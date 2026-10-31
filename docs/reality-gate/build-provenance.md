# RG-0A build provenance: toolchain, lockfile, native vs portable

**Reviewed baseline:** merged `exo` commit `26b1c7982068fec1fc2329f17567ab6dc64ee171` (2026-10-31).

**Scope:** Build configuration and evidence only. No runtime semantics, hardware measurements, or performance claims are changed by this step.

## Reproducibility contract

- `rust-toolchain.toml` pins **Rust 1.99.1** with the minimal profile. This is a tested build toolchain, **not** a claim that 1.99.1 is the minimum compatible Rust version.
- `Cargo.lock` is tracked, lockfile version 4, containing only the `vtpu-runtime` package. Cargo's dependency sections remain empty; this PR adds no third-party crates.
- Always pass `--locked` to fail when the package graph no longer matches the committed lockfile.
- The existing `.cargo/config.toml` sets `-C target-cpu=native` for **every build profile**, including debug tests. Native artifacts are host-specific. This PR preserves that historical default while documenting an explicit portable variant.

## Two distinct build modes

From the repository root with the pinned toolchain installed:

| Mode | Validation command | Release build command | Compilation target |
| --- | --- | --- | --- |
| **Native** (existing default) | `cargo test --all-targets --locked --quiet` | `cargo build --release --locked` | `-C target-cpu=native` from `.cargo/config.toml`; tuned for *this* CPU only |
| **Portable** | `RUSTFLAGS='' cargo test --all-targets --locked --quiet` | `RUSTFLAGS='' cargo build --release --locked` | Default Rust target CPU, no host-native flag; CPU support depends on the selected target triple |

The explicitly empty `RUSTFLAGS` overrides Cargo's configured `build.rustflags`; this was verified against the real `rustc` invocations emitted by `cargo check --locked --lib -v`: the native invocation contained `-C target-cpu=native`, while the portable invocation did not. Do not use a plain `cargo test` result from one host to claim portability to an unrelated CPU.

If either mode is to be compared quantitatively, record CPU model, target triple, kernel, toolchain version/hash, revision, exact command and flags, sample count, benchmark inputs and timing denominator. Prefer separate build artifact directories or fully distinguish Cargo fingerprints; never compare stale binaries built under different flag modes.

## Evidence capture (2026-10-31)

The environment used for this build work is a **non-RG-4 container**, not an approved Ryzen 9 8945HS benchmark host. No throughput, CPU cycles, or energy result is asserted.

| Check | Observed result |
| --- | --- |
| `rustc --version` | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| `cargo --version` | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| Source literal `#[test]` counts | 922 under `src/`, 1,470 under `tests/` at the reviewed commit; not compiled/pass totals |
| Canonical native `cargo test --all-targets --locked --quiet` | **Passed:** 87 test-target invocations, 3,910 passed, 0 failed, 3 ignored |
| Portable `RUSTFLAGS='' cargo test --all-targets --locked --quiet` | **Passed:** 87 test-target invocations, 3,910 passed, 0 failed, 3 ignored |
| Native `cargo build --release --locked` | **Passed**, existing compiler warnings only |
| Portable `RUSTFLAGS='' cargo build --release --locked` | **Passed**, existing compiler warnings only |
| `./check.sh` | **Exit 0**, four nonfatal warnings: working tree uncommitted at test time, three ignored tests, `sparse_attention` optional benchmark build issue, `workloads` lacking a build system |

The source-count figures are simple textual counts, whereas actual `cargo test` counts can include repeated invocations across targets. Report both separately and retain raw command evidence when practical.
