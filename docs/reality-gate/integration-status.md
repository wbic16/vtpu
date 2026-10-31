# vTPU Reality Gate — integration checkpoint

**Date:** 2026-10-31

**Integration branch:** `codex/reality-gate-integration`

**Integration commit:** `7bddb99a435e29a8288229435ee623ab865f43f6`

**Base `exo` commit:** `26b1c7982068fec1fc2329f17567ab6dc64ee171` (merged documentation PR #1)
**Ship status:** **NOT READY**; this is a test integration of eight implementation PR heads, including drafts. No implementation PR was merged to `exo` by this checkpoint.

## Integrated inputs

| PR | Revision | Scope | PR status at checkpoint |
| --- | --- | --- | --- |
| [#6](https://github.com/wbic16/vtpu/pull/6) | `81bd3cd287b6faeae647c01b890b1ee9a0b5f8fa` | Toolchain pin, lockfile, native/portable build provenance | Open, reviewable |
| [#5](https://github.com/wbic16/vtpu/pull/5) | `7fe1d40404462c820d7da25f8d5a13342eab282e` | W29/cost benchmark retirement truth | Open, reviewable |
| [#3](https://github.com/wbic16/vtpu/pull/3) | `a78a5f7e95b33d48a278be0a8e37e91918042dc4` | CSEND register dependency hazards | Open, reviewable |
| [#8](https://github.com/wbic16/vtpu/pull/8) | `6ed840f55c90504c8aa1ce37236b4f1dec57dc77` | Small-fleet topology underflow | Open, reviewable |
| [#4](https://github.com/wbic16/vtpu/pull/4) | `cd01737c46479f13441492308ef21fc5786582c4` | Full coordinate injectivity/flags | Open, reviewable; raw-format migration decision |
| [#2](https://github.com/wbic16/vtpu/pull/2) | `116dc7ac3b05b63745a1a30bc6629c3e3d400aa3` | RG-1A D-pipe oracle | Draft; provisional semantic specification |
| [#7](https://github.com/wbic16/vtpu/pull/7) | `5bd330c2923d65762fb91e0b8937847519e550ac` | Truthful single-thread W23 cooperative messaging | Draft; CBAR not synchronized |
| [#9](https://github.com/wbic16/vtpu/pull/9) | `3e32c4f5ccd51ea2eea7415b9526e86a1553707a` | Experimental durable TTSM snapshot log | Draft; single-writer, no real power-loss soak |

The integration commit's source tree was produced by conflict-free local merges of all eight PR heads, validated, and uploaded as an exact SHA-matched tree with the eight PR commits as additional merge parents. This branch is **not** a substitute for reviewing and merging individual PRs to `exo`.

## Reproducible validation

- Host: isolated Codex workspace, **not** Will's AMD 8945HS cluster.
- Toolchain: Rust/Cargo 1.99.1, locked dependency set, existing native CPU flags.
- Clean target-directory command: `cargo test --all-targets --locked --quiet` (with `CARGO_TARGET_DIR=/tmp/vtpu-integration-fresh-20261031` and the workspace Rust toolchain selected).
- Result: **91 test target suites; 3,946 passed, 0 failed, 3 ignored.** Aggregate counts may include tests invoked in multiple targets.
- Release regressions: `cargo test --release --locked --test reality_gate_rg1a --test reality_gate_coord --test reality_gate_ttsm_durable --test rg1b_csend_hazards --quiet`: **22 passed, 0 failed, 0 ignored** across four targets.
- `git diff origin/exo HEAD --check`: clean on the locally merged tree.
- No GitHub CI status checks were returned for the integration commit at checkpoint time.

**Build-cache footnote:** An initial incremental test run after rapid consecutive local merges reported 18 missing TTSM APIs, despite those APIs being present in the source tree. Recompiling into a fresh `CARGO_TARGET_DIR` resolved the stale crate metadata; the clean-target all-targets suite passed. Future integration runs should use clean target directories when compiler metadata and source disagree.

## Outstanding release gates

- [ ] Freeze canonical semantics for `DRED`, HDC operations, and reserved ternary encodings. PR #2's oracle deliberately characterizes some current behavior provisionally.
- [ ] Reach the full RG-1 gate of **100,000 deterministic mixed D/S/C streams**, including scheduler, packer, memory/message state, saved failing seeds, mutation tests, and zero unexplained divergences.
- [ ] Decide and test migration/version policy for existing raw `PhextCoord` values containing nonzero flags after PR #4's bit-layout correction. Validate sparse-memory costs and coordinate replay consistency.
- [ ] Decide W23/`CBAR` synchronization semantics and demonstrate correct behavior. A single-thread cooperative simulation is not a measured eight-core/SMT result.
- [ ] Extend RG-2 TTSM beyond one in-process writer: conflict/locking semantics, randomized process-kill/crash-reopen soaks, fsync/power-loss evidence, epoch-ID reuse checks, and integrity under corruption. Keep no cryptographic-tamper claim from its current checksum.
- [ ] Run a preregistered real-hardware RG-4 test on one **named, approved** 8945HS node, with clocks, CPU affinity/SMT, DDR5 bandwidth, PMU permissions, power methodology, uncertainty and raw samples. No cluster experiments without separate authorization.
- [ ] Add/verify continuous CI for the integration branch, rerun after every merge, and obtain independent review of semantic and compatibility changes.

## Hourly continuation rule

Fetch current `exo`, integration, and PR heads; integrate only new commits by a non-forced, expected-head update; run a fresh all-targets suite and focused release tests; publish exact results and newly discovered regressions. If any gate fails, isolate a bounded fix on a reviewable PR. **Do not merge integration into `exo` or declare a release without Will's explicit approval.**
