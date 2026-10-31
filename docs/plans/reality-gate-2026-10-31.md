# vTPU Reality Gate — Independent Engineering Plan

**Proposal:** RG-001  
**Author:** ChatGPT (independent plan for Will Bickford)  
**Date:** 2026-10-31  
**Status:** Proposed; no implementation or performance claims are made by this document  
**Target:** `wbic16/vtpu`, base branch `exo`  
**Related reviews:** Grok's GitHub comment and separate Emise/Argon attachments; see [review addendum](reality-gate-review-addendum-2026-10-31.md) for provenance and corrections. Standalone reviewer plans have not been supplied.

## Executive decision

Make vTPU a *measurable, reproducible, correct* execution substrate before
extrapolating its throughput or relying on its temporal semantics for persistent
cognition. This is a verification-and-enablement campaign, not an architecture
replacement. Keep phext-native coordinates, the three-pipe SIW model, the
sentron abstraction, Rust, and the lightweight/zero-dependency core as design
constraints.

There are two near-term leverage points:

1. **Hardware truth.** Current benchmarks mix virtual pipe-operations,
   wall-clock throughput, and modeled cycles. Hardware-level claims require a
   separate measurement contract and actual CPU measurements.
2. **Durable temporal truth.** The TTSM's commit/fork/replay structure is a
   strong starting point; today its `replay()` walks block metadata, while
   `commit()` retains a hash/size but does not persist recoverable state bytes.
   Establish a crash-recoverable state contract before promising durable replay.

The long-term 75 Gops/s single-node goal remains a **target**, not a passing
gate or an achieved result. A failure to hit that goal is valid evidence.

## Evidence snapshot and limitations

This proposal is based on source inspection, **not** a local build, test run,
profiling session, or hardware audit. Revalidate every observation at the
working branch head before editing the associated code. The subsequent review
found a material error in the original W29 assessment; the corrected row below
and the [review addendum](reality-gate-review-addendum-2026-10-31.md) supersede
that earlier claim without changing implementation code.

| Area | Evidence in `exo` | Hypothesis or limitation to test |
| --- | --- | --- |
| Performance accounting | `src/bin/bench_w23_8core.rs` computes `total_retired / (wall_seconds * 5e9)` and describes sentron pairs as a model of SMT siblings | This is an effective rate using an assumed frequency, not a measured hardware IPC or proof of physical 8-core parallelism |
| Unified benchmark — **invalid pending repair** | `src/bin/bench_w29.rs` assigns programs to `Dormant` sentrons without calling `spawn`; `exec::run()` immediately returns unless `Running`. Its timed loops therefore retire **zero SIWs** at this revision. Separately, `src/cost.rs::bench_d_pipe` times program cloning/assignment without invoking execution. | Quarantine W29 and cost-benchmark throughput claims; repair lifecycle, require nonzero retirement and architectural effects, and rerun before interpreting GOPS. |
| Temporal state | `src/ttsm.rs` stores `TemporalBlock` metadata in a `HashMap` and reconstructs a metadata lineage on replay | Reopen/replay of state bytes, crash consistency, and divergent-writer handling need proof |
| Epoch mapping | `src/epoch_ppt.rs` has immutable `EpochView` snapshots and epoch-tagged PTC entries, but its page-table translation truncates 11-bit coordinate dimensions to 7/8 bits; `PhextCoord::set_flags` overlaps dim10 | Address injectivity, flag integrity, cross-epoch stability, and no-allocation-on-replay must be tested adversarially |
| Validation | `src/validation.rs` explicitly uses simplified dependency tracking without specific register provenance | Construct differential tests for RAW/WAR/WAW and all three pipes |
| Documentation | `readme.md`, `onboarding.md`, `vtpu.dass`, and historical wave notes report different counts/statuses | Generate current facts from executable artifacts; label historical targets clearly |
| Intent | `src/intent.rs` has deterministic parsing and `IntentSignature` types | Distinguish implemented parsing from compiler/code-generation promises |

Repository links: [readme](../../readme.md),
[onboarding](../../onboarding.md), [vTPU DASS](../../vtpu.dass),
[TTSM](../../src/ttsm.rs), [epoch PPT](../../src/epoch_ppt.rs),
[validation](../../src/validation.rs),
[W23 benchmark](../../src/bin/bench_w23_8core.rs),
[W29 benchmark](../../src/bin/bench_w29.rs),
[intent](../../src/intent.rs).

## Non-negotiable invariants

**I1 — Executable equivalence.** For the supported opcode subset, reference,
scheduled, packed, batched, and native-lowered execution produce the same
architectural state and externally visible effects, or return the same
specified fault. NOPs do not count as useful operations.

**I2 — Coordinate fidelity.** A coordinate is encoded, decoded, translated,
and replayed without unintended aliases or changes in dimensional meaning.
An absent or invalid mapping never silently resolves to another coordinate.

**I3 — Temporal immutability.** A committed block's payload, parent, and
translation view cannot change. A fork may evolve independently without
mutating ancestors. Replay must reproduce the exact committed payload.

**I4 — Recovery determinism.** After any simulated crash point, reopening
either sees the last durably committed transition or a valid prior state. It
never sees a partially committed transition, invents a block, or silently
reuses an epoch identifier.

**I5 — Measurement provenance.** Every reported performance value identifies
the unit of operation, denominator, CPU, core/thread count, frequency method,
software revision, input distribution, run configuration, and uncertainty.

**I6 — Admission boundary.** No privileged commands, production-data mutation,
networked cluster deployments, or live binary hot-swapping occur as side
effects of routine tests. They require a separate reviewed experiment.

## Work packages

### RG-0: Reproducible baseline and claims ledger

**Purpose:** Produce one ground-truth inventory of what currently builds,
passes, and runs. This package is first and read-only until findings are
reviewed.

Tasks:

1. Record branch head, toolchain, platform, CPU topology, kernel, build flags,
   and test command. Check build/test/clippy without changing repository state
   beyond ordinary local build artifacts.
2. Run a representative baseline including SIW interpreter, D/S/C pipeline,
   PPT, TTSM, SMT/cooperative scheduling, and at least one mixed workload.
3. Classify statements in current documentation as **measured**, **modeled**,
   **historical**, **target**, **unverified**, or **invalid**. Explicitly
   quarantine W29 and `cost::bench_d_pipe` results until execution is repaired.
   Never silently turn one classification into another.
4. Produce a machine-readable benchmark manifest and a concise human report.
5. Record the installed Rust toolchain and `target-cpu=native` configuration,
   resolve `Cargo.lock`/toolchain pinning, and reconcile literal `#[test]`
   counts (922 under `src/`, 1,470 under `tests/` at the reviewed commit)
   separately from the number of compiled and passing tests.

Deliverables: `docs/reality-gate/baseline.md` and an exportable
`baseline.json` sample; a proposed correction list for stale README/DASS
metrics. Do not rewrite historical wave records.

**Exit gate RG-0:** A reviewer can independently reproduce a passing/failing
test count and each baseline value from an exact revision and command. Any
unavailable hardware counter is explicitly marked unavailable.

### RG-1: Reference executor and differential SIW oracle

**Purpose:** Prove the fast paths preserve meaning under adversarial streams.

Tasks:

1. Define the supported opcode/state subset and a deliberately simple,
   obviously ordered reference interpreter; define overflow, fault, memory,
   message, and barrier behavior. Do not use optimized code as its own oracle.
2. Implement deterministic generators with fixed seeds, no external test
   dependency by default. Cover register hazards, mixed D/S/C SIWs,
   cross-SIW dependencies, pack/unpack, coordinate boundaries, absent memory,
   overflow, and interleaved sentrons.
3. Run generated streams through the reference, scheduler, packer, interpreter,
   and any other claimed equivalent route. Compare full register/memory/message
   state plus faults; minimize and save counterexamples.
4. Add model-based tests for structural validation and coordinate translation.
   Include seeded fuzz replays as normal regression tests.
5. Implement an initial **RG-1A D-pipe slice** covering all 16 active D-pipe
   opcodes with explicit semantics and a truly independent oracle. Exercise
   10,000 seeded streams, boundary values, per-opcode coverage, comparator
   mutation tests, and the observed `DSEL`/`DCMP` divergences. Keep the
   100,000 mixed-pipe streams as the full RG-1 exit criterion. Test `CSEND`
   register hazards and PPT coordinate aliasing as separate prerequisite gates.

Deliverables: `tests/reality_gate/` (or an idiomatic equivalent), seed corpus,
state comparator, and a fail-fast report specifying which execution path
disagreed and the minimal reproducible instruction stream.

**Exit gate RG-1:** At least 100,000 deterministic generated streams run with
zero unexplained divergences for the *declared supported subset*, plus
explicit regression tests for every discovered bug. Invalid inputs must be
specified as rejected or faulting; they may not be discarded to improve scores.

### RG-2: TTSM/epoch persistence and fault-injection contract

**Purpose:** Turn metadata lineage into recoverable temporal state while
preserving immutable history and epoch-aware translations.

Tasks:

1. Specify a minimal committed-block serialization with version, coordinate,
   epoch, parent/fork identity, payload length, checksum, and integrity check.
   Define storage limits and error behavior before implementation.
2. Begin with an **append-only snapshot log with per-block checksums**;
   keep SQ as an optional adapter behind a storage trait, not a requirement
   for local tests. Preserve zero external dependencies for the core. No real
   user data in tests.
3. Implement replay of **state bytes**, not only metadata, with verification of
   hashes and parent linkage. The first durable format stores full snapshots;
   delta compression is a separately justified optimization.
4. Define and test concurrent-owner semantics: either optimistic head checks
   and explicit conflict errors, or an equally explicit merge/branch policy.
5. Inject failures before/after append, flush, manifest/head update, and reopen.
   Include torn/truncated writes and checksums. Test fork isolation and
   repeated recovery.
6. Audit epoch-tagged PTC invalidation and ensure replay cannot allocate a
   missing historical translation.

Deliverables: durable TTSM prototype, recovery runner, format/version note,
and boundary tests. Avoid claiming SSD persistence if only RAM is exercised.

**Exit gate RG-2:** 100+ seeded fault-injection/recovery scenarios produce
only allowable outcomes; immutable prior blocks remain byte-identical;
replay of any committed test block reproduces its payload and translation
epoch. Repeated reopen is idempotent. Conflict policy is enforced.

### RG-3: Native execution proof of concept

**Purpose:** Establish whether lowering SIW streams can materially outperform
interpretation without changing their semantics.

Tasks:

1. Start with an isolated, deterministic AOT-generated kernel (a small D-pipe
   integer/ternary subset); benchmark it against the reference interpreter,
   existing optimized interpreter, and hand-written native equivalent.
2. Expand to a controlled S-pipe gather/scatter subset after coordinate
   equivalence is proven. Include bounds, alignment, and memory safety tests.
3. Evaluate JIT only after the AOT baseline establishes correctness and a
   speedup opportunity. Any executable-memory path requires a security and
   memory-safety review.
4. Keep compiler/toolchain experiments outside the zero-dependency runtime:
   **no new runtime crates**. A lightweight codegen option is welcome;
   importing LLVM is not assumed. Require byte-identical native output for
   the same canonical SIW stream and build configuration.

Deliverables: a minimal source-to-native lowering prototype, deterministic
equivalence tests, compile-time and runtime cost measurements, and a documented
decision on whether to extend, redesign, or stop the approach.

**Exit gate RG-3:** Native and reference results agree on RG-1's applicable
corpus; the observed speedup (including zero or negative) is reported with
confidence and compilation overhead. No 75 Gops/s claim is inferred from
modeled 3-wide virtual retirement.

### RG-4: Real-hardware performance and scaling experiment

**Purpose:** Measure useful work on an AMD Ryzen 9 8945HS-class node; extend
to the larger cluster only after the single-node experiment is credible.

Tasks:

1. Obtain maintainer approval for one specifically named 8945HS host before
   testing (Grok suggested `elven-path`, not yet confirmed as the test host).
   Pin threads explicitly and record logical CPU → physical core/SMT mapping,
   `perf_event_paranoid`, and counter access. Separate 1-thread,
   1-core/2-thread, 8-core, and 16-thread runs.
2. Capture wall-clock throughput, actual CPU cycles/instructions (when PMU
   access is available), cache misses, memory traffic, energy/power when
   measurable, and thermal/frequency behavior. Do not require root to run a
   non-privileged correctness suite.
3. Use multiple input classes: D-heavy, S-heavy, C-heavy, mixed realistic,
   high-locality, and random-coordinate. Include equivalent non-vTPU baselines.
4. Report median and variability over repeated warm/cold runs, not just the
   highest observed number. Separate compiler time and allocation costs.
5. After single-node review, specify a **separate** authorized experiment for
   2 nodes and then the 25-node AMD cluster, including USB4/network costs,
   energy, and cross-node latency.

Deliverables: `reality-gate` benchmark report with raw samples, repro
commands, hardware metadata, and a roofline/bottleneck analysis.

**Exit gate RG-4:** Each throughput claim has a reproducible hardware trace
or explicit counter limitation. The report distinguishes SIW/s, useful
D/S/C operations/s, hardware instructions/cycle, and application output
(e.g. tokens/s), with no unit substitution.

## Measurement contract

Suggested record schema (illustrative, not imposed on implementation):

```json
{
  "schema": "vtpu-reality-gate/v1",
  "revision": "<full-git-sha>",
  "workload": "mixed-coordinate-v1",
  "operation_definition": "completed non-NOP architectural pipe op",
  "runtime": "interpreter|aot|jit|reference",
  "host": { "cpu": "<model>", "cores": 8, "threads": 16 },
  "measurement": { "wall_seconds": 1.0, "useful_ops": 0,
    "cycles_source": "perf|rdtsc|unavailable", "actual_cycles": null },
  "repetitions": 10,
  "seed": 42,
  "correctness": { "oracle_agreement": true, "failed_cases": 0 }
}
```

Keep separate columns for: virtual SIWs retired, non-NOP D/S/C operations,
measured physical CPU cycles, and wall time. The arithmetic target
`8 cores × 5 GHz × 2 ops/cycle ≈ 80 Gops/s` is an optimistic model with
perfect sustained clocks and useful-work occupancy, **not a benchmark**.

**Useful operation:** A completed non-NOP architectural D-, S-, or C-pipe
operation whose defined state transition or externally observable effect is
executed and counted once; program setup, dormant returns, and simulated
cycles are not useful operations.

## Execution order, gates, and contribution policy

| Sequence | Depends on | Merge gate | Proposed token allowance |
| --- | --- | --- | ---: |
| RG-0: baseline | None | Reproducible evidence, no hidden assumptions | 50K |
| RG-1: oracle | RG-0 | 100K generated streams, zero unexplained divergence | 150K |
| RG-2: persistence | RG-0; RG-1 where relevant | Crash/reopen, immutable payload replay | 100K |
| RG-3: native | RG-1 | Differential equivalence and honest speedup | 150K |
| RG-4: scaling | RG-0 and RG-3 for native comparison | Actual hardware data and reproducible units | 50K |
| **Total** | | | **~500K** |

These are model-work planning allowances, not quotas or billable-token
predictions. Additional investigation is justified by discovered failures,
not by token expenditure alone.

**PR boundaries:**

1. Proposal/documentation and attributed review addendum (this PR; no runtime
   behavior change).
2. RG-0A: pinned baseline, claims ledger, benchmark retirement checks/fixes,
   and structured results.
3. RG-1A: independent all-D-opcode reference model, 10K seeded streams,
   differential generators, and counterexample corpus.
4. Durable TTSM/epoch work with recovery tests.
5. Native-lowering prototype with equivalence and benchmarks.
6. Single-node results and optional independently approved cluster plan.

Each implementation PR must include: exact baseline commit, test command and
result, semantic invariants affected, benchmark before/after (or `N/A`),
counterexamples addressed, and rollback/reversal notes. Keep independent
reviewers' plans and their evidence distinct until there is an explicit
comparison/merge decision. Avoid concurrent edits to the same architectural
files across plans without an agreed owner.

## Stop conditions and risk register

- **Semantic drift:** Stop optimization work if the reference executor and
  accelerated path disagree; preserve the failing seed before repair.
- **Misleading throughput:** Stop publishing hardware conclusions when only
  modeled cycles or assumed frequency are available; label them modeled.
- **Temporal ambiguity:** Stop persistence work until snapshot/delta and
  concurrent-writer semantics are explicit.
- **Portability:** Keep x86-specific code isolated; do not break interpreter
  correctness on non-x86 platforms.
- **Permissions/security:** Do not relax `perf_event_paranoid`, run privileged
  commands, touch production SQ data, or enable writable/executable memory
  without the operator's authorization and a separate review.
- **Scope creep:** No speculative consciousness claims, automatic hot-swapping,
  autonomous distributed deployments, or broad rewrite of vTPU in this gate.

## Decision requested from Will / reviewers

1. Accept the invariants and measurement definitions as the common comparison
   contract (or record precise disagreements).
2. Decide whether durable TTSM should use an in-repo append-only backend, SQ,
   or both behind a trait; benchmark and failure semantics should stay common.
3. Confirm the first AMD host available for RG-4; record access restrictions
   separately from the test plan.
4. Compare this submission with Grok's, Emise's, and Argon's **supplied
   reviews** by evidence quality, novelty, dependencies, and executable exit
   gates. Do not attribute standalone plans that have not been provided.
   Merge compatible work, retaining provenance for distinct proposals.

**Definition of success:** A skeptical engineer can clone a specific commit,
run a finite set of tests and benchmarks, reproduce the reported results, and
identify exactly which vTPU properties have been *proved by tests*, *measured
on hardware*, *modeled*, or *not yet demonstrated*. Then improvements compound
on evidence.
