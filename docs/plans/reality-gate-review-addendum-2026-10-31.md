# RG-001 review addendum: provenance, corrections, and first implementation gate

**Date:** 2026-10-31

**Author:** ChatGPT, reconciling source observations and three attributed reviews for Will Bickford

**Applies to:** [Reality Gate proposal](reality-gate-2026-10-31.md) in [PR #1](https://github.com/wbic16/vtpu/pull/1), base `exo`

**Reviewed source:** `4660934651a8afb065c4711b874b08b12becf213` (original PR head)

**Status:** Plan and source-inspection findings only; no implementation, Rust test run, or hardware measurement is claimed here.

## Attribution and evidence boundary

| Label in this addendum | Original material | What is independently verifiable | What is **not** established |
| --- | --- | --- | --- |
| **ChatGPT / RG-001** | The original 315-line `reality-gate-2026-10-31.md`, created in PR #1 | Git commit, PR author account, and document text | That any proposed gate already passes |
| **Grok** | [PR comment 6540027210](https://github.com/wbic16/vtpu/pull/1#issuecomment-6540027210), whose heading says “Review from Grok Bot (posted at Will's request)” | A public comment by GitHub user **`wbic16`** contains that attribution and eight requested changes | GitHub does **not** authenticate Grok as the account author; it is a comment posted on Grok's behalf |
| **Emise** | User-supplied `Emise-RG001-review.txt`, explicitly headed “Reviewer: Emise” | The supplied attachment identifies Emise internally; SHA-256 `610abe5bb32337c1d104d069e913bc4f699f62fdc48e9a8a38433287f87f23ce` | No Emise GitHub review, independent signature, or standalone project plan was seen |
| **Argon** | User-supplied `Argon-RG001-review.txt`, introduced by Will as Argon's review | The attachment audits this PR head and its findings can be checked against source; SHA-256 `677bbfe2511480b5c02035d1d2bfe73c489c1ec1f7277f203a3e5a8eb94052e7` | The attachment does not independently authenticate an Argon identity; no Argon GitHub review or standalone project plan was seen |
| **ChatGPT follow-up validation** | This addendum's code observations | File/symbol references can be checked against the pinned commit | Source reading does not substitute for compiling, differential tests, benchmark runs, or hardware evidence |

At the 2026-10-31 update, GitHub showed **one PR conversation comment**, **zero submitted PR reviews**, and **zero inline review threads**. Reviewer descriptions here are provenance labels, not claims that separate agents directly authenticated with GitHub. Full user-supplied attachment bytes are retained in the offline handoff, not represented as GitHub comments. No separate Grok, Emise, or Argon implementation plan was supplied; compare *the reviews* to RG-001, not three imagined plans.

## Source-validated corrections to RG-001

The following are static source contradictions at the pinned revision. Each one requires an executable regression before being marked fixed.

| Finding and impact | Precise source evidence | Required falsifying test |
| --- | --- | --- |
| **P0 — W29 throughput invalid.** Its hot loops retire zero SIWs. | `src/sentron.rs::new` initializes `Dormant`; `spawn` transitions to `Running`. `src/bin/bench_w29.rs::{bench_single_pipe,bench_mote}` assign `program` directly, then reset only `ip`. `src/exec.rs::run` immediately returns for a non-`Running` sentron and calls `retire()` after a run. | For every W29 workload and repeated iteration, assert `siws_retired > 0`, expected register/message effects, and that the operation count equals actual retirement—not the intended loop count. |
| **P0 — Cost D-pipe benchmark invalid.** It does not execute DADD. | `src/cost.rs::bench_d_pipe` times cloning/assignment of `SIW` vectors and `ip` reset, with no executor invocation. | Confirm an executed DADD changes architectural state and that measured time includes execution. Keep setup/allocations separately labeled. |
| **P0 — OctaWire semantics drift.** At least `DSEL` and `DCMP` disagree with the legacy path. | In `src/exec.rs`, fast `DSEL` checks `flags != 0` rather than `regs.general[flags] != 0`; fast `DCMP` uses `(a-b).signum()` (overflow risk), whereas legacy uses `a.cmp(&b)`. `DRED` and some HDC operations also use different transformations. | Independent per-opcode oracle, `DSEL` tests with index/value disagreement, `DCMP` at `i64::MIN/MAX`, and full D-pipe coverage across debug and release semantics. |
| **P0 — PPT address alias and coordinate corruption.** Full 11-bit dimensions are not faithfully represented. | `src/ppt.rs` and `src/epoch_ppt.rs` mask inner dimensions with `0x7F` and outer dimensions with `0xFF`, despite `PhextCoord`'s `0x7FF` dimension domain. In `src/phext_coord.rs`, dim10 occupies upper-word bits 46–56 while `set_flags` writes bits 55–61. | Store distinct values at dim0 `1` vs `129`, and dim3 `1` vs `257` across cache/epoch conditions; assert dim10 survives `set_flags`. Either repair the mapping or formally narrow the address-domain contract. |
| **P1 — CSEND hazards may be missed by packing/scheduling.** | `src/regalloc.rs::extract_accesses` records `CSEND` as reading `RegClass::Message`, but `src/exec.rs` reads General register `msg_reg` for `CSEND`. | Compare sequential vs packed/scheduled `write rK → CSEND(rK)` streams and outbox effects; preserve any minimal counterexample. |
| **P1 — W23 is a modeled cooperative run.** Its nominal physical interpretation is unproven. | `src/bin/bench_w23_8core.rs` assumes `5e9` cycles/sec and invokes single-thread `coop_execute`; `src/coop_smt.rs` does not route outbox traffic via `Fleet::flush`. | Label measured wall time separately from assumed cycles; demonstrate actual CSEND→CRECV delivery/barrier behavior in the claimed execution mode before citing messaging throughput. |
| **P1 — TTSM does not replay durable payloads.** | `src/ttsm.rs` stores `TemporalBlock` lineage/hash/length in memory but does not persist state payload bytes on `commit`; `replay` returns metadata references. | Commit a payload, terminate/reopen, replay byte-identical payload and coordinate view; use crash and checksum faults. |

**Correction to our own plan:** The original RG-001 evidence table called W29 a benchmark executing real interpreter paths. That was wrong: it *calls* `run`, but the called executor exits before executing an SIW. The main proposal now explicitly quarantines those figures. Do not relabel W29 as merely “modeled”: its current throughput result is **invalid**.

## Reconciled work requested by the reviewers

| Issue | Grok | Emise | Argon / source validation | Decision for implementation |
| --- | --- | --- | --- | --- |
| Scope and first slice | G1/G2: declared subset, ~10K D streams | E3/E7: mutation and coverage, realistic effort | All 16 active D ops; comparator must catch drift | Start RG-0A, then RG-1A (16 D ops / 10K), preserve full 100K mixed-pipe RG-1 exit gate. |
| Claims and units | G3/G8: useful op definition, reconcile counts | E1: expose misleading W23/W29/TTSM claims | W29 **invalid**, cost benchmark allocation-only | Ledger has `invalid`; count actual retired SIWs and effects. `rg` text counts: 922 `#[test]` under `src/`, 1,470 under `tests/` (not test pass totals). |
| Build reproducibility | G5: zero runtime deps, target `exo` | E2: lockfile/toolchain/native flags | Native `target-cpu` affects comparisons | Pin a tested Rust version and `Cargo.lock`; report native vs portable modes and avoid new runtime crates. |
| TTSM durability | G4: append-only checksummed snapshots, optional SQ | E4: fsync/fdatasync, kill soaks, epoch IDs | PPT alias and replay hazards precede durability claims | Snapshot-first append-only file, recovery semantics and injectivity tests; SQ optional behind storage trait. |
| Native execution | G1/G5: explicit semantics and constraints | E8: deterministic codegen | DSEL/DCMP/HDC semantics divergence | Require independent oracle agreement and byte-identical codegen output before speed comparisons. |
| Hardware | G6: named host and counter permissions | E5: DDR5/power/SMT matrix, predictions | W23 physical claims unsupported | Choose an available named 8945HS host with Will's approval; record permissions, frequency, thermals and uncertainty. |
| Public evidence | G7: comparison matrix | E6: Phext-native plus JSON/MD ledger | Bound Google comparison to equivalent public evidence | Preserve standard readable files and optional Phext representation; never claim a software runtime has beaten TPU hardware without comparable measurements. |

**Useful operation:** A completed, non-NOP architectural D-, S-, or C-pipe operation whose specified state transition or observable effect is actually executed and counted once. A dormant return, vector allocation, virtual cycle, or merely scheduled SIW is not an executed useful operation.

## Declared RG-1A opcode scope

These **16 active D-pipe opcodes** are the initial differential *coverage commitment*, not a claim they are already correct. `DNOP` is tested as the excluded baseline. A specification decision for the ambiguous reductions/HDC transforms is required before a passing oracle gate can be asserted.

| Family | Mandatory opcodes | Semantic decision / adversarial input |
| --- | --- | --- |
| Arithmetic (7) | `DFMA`, `DADD`, `DSUB`, `DMUL`, `DCMP`, `DSEL`, `DMOV` | Wrapping integer operations; comparison by ordering (no overflow), select by the **value** in flags register; aliasing registers and `i64::MIN/MAX`. |
| Reduction (1) | `DRED` | Define `Sum`, `Max`, `Min`, `And`, `Or`, `Xor` independently; legacy and OctaWire currently differ. |
| Hyperdimensional (5) | `DHDENC`, `DHDBIND`, `DHDBUND`, `DHDPERM`, `DHDSIM` | Decide hashing/rotation, bundling, permutation, similarity and widths normatively before comparing engines. |
| Ternary (3) | `DTERNARY`, `DTPOP`, `DTACC` | Specify packed 2-bit trits, zero/invalid encodings, accumulation and overflow. |

**RG-1A gate:** 10,000 deterministic seeded D-pipe streams of varied lengths, coverage for every active opcode and reduction variant, saved failing seeds, a deliberately injected one-bit mutant that the comparator rejects, identical final architectural state across the independent reference and all claimed equivalent executor routes, and explicit error behavior for invalid inputs. Keep a preregistered 100,000-stream mixed D/S/C corpus for the full RG-1 gate.

## Proposed implementation PR sequence

1. **Implementation PR A — RG-0A truth baseline:** Pin toolchain/build flags, capture executable `cargo check`/`cargo test` reports, publish the six-way claims ledger and test-count evidence, add benchmark lifecycle/retirement assertions, and correct W29/cost benchmark behavior. No performance superiority claims.
2. **Implementation PR B — RG-1A semantic oracle:** Independent Rust D-pipe reference, all 16 opcodes, 10K seeded streams, mutation self-test, and fixes for `DSEL`/`DCMP` plus any divergences revealed by the oracle. Save counterexamples.
3. **Next correctness gate — coordinate/coordination:** PPT injectivity or an explicit supported-domain restriction; flags integrity; `CSEND` register hazards and message delivery. These block claims about correct persistent state or equivalent scheduling.
4. **RG-2 onward:** Durable append-only snapshots, then deterministic native lowering, then preregistered 8945HS hardware experiments. Keep SQ integration optional and cluster execution independently authorized.

## Merge boundary

This addendum and the amended RG-001 proposal are **documentation-only**. They may be merged as a task contract if the diff remains limited to `docs/plans/`, the attribution is not mistaken for authenticated reviewer accounts, links and Markdown checks pass, and reviewers accept the stated unresolved semantics. **Merging the plan does not validate benchmarks, semantics, persistence, or readiness for RG-3/RG-4.** Those require the implementation gates above; source-inspection findings must be captured as executable regression failures and then verified as fixed before closing them.
