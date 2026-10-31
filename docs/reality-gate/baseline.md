# RG-0A baseline: benchmark execution truth

**Captured:** 2026-10-31

**Starting revision:** `26b1c7982068fec1fc2329f17567ab6dc64ee171` (`exo`, merged RG-001 PR #1)

**Patch branch:** `codex/rg0a-benchmark-truth-20261031`

**Rust:** 1.99.1, zero new dependencies

**Scope:** Local Rust executor and benchmark accounting only. No real 8945HS hardware, CPU-cycle counter, power, or cluster result is claimed.

This is a first **source-and-test evidence ledger**, not the full RG-0 hardware inventory. Hardware topology, kernel, host identity, measured frequency, build pinning and raw sample distributions still need a separately preregistered run on an approved host.

The available container reports an **AMD EPYC 9V74 80-Core Processor**, **9 visible vCPUs**, Linux **6.18.57**, x86_64, and Rust **1.99.1**. That is an unapproved test environment, **not** a named Ryzen 9 8945HS RG-4 host; no physical-core mapping, actual clock cycles or power was established.

## Before/after executable baseline

| Check | Result | Interpretation |
| --- | --- | --- |
| Merged `exo`: `cargo test --all-targets --quiet` | **3,910 passed; 0 failed; 3 ignored** across 87 target-suite invocations (observed immediately before this branch by the coordinating run) | Sum across Cargo test targets; some tests may run in more than one target. This is **not** a unique-test census. |
| RG-0A patch: `cargo test --all-targets --quiet` | **3,916 passed; 0 failed; 3 ignored** across 87 target-suite invocations | Executed locally after code edits, with six added tests (four W29, two cost). |
| Literal source `#[test]` occurrences | 922 under `src/`, 1,470 under `tests/` at the review snapshot | Static text count, not proof each test compiles or passes. |
| RG-0A targeted unit tests | `cargo test --bin bench_w29 --quiet`: **4/4 pass**; `cargo test --lib cost::tests --quiet`: **10/10 pass** | Includes pre-existing cost tests and two new regression checks. |

## Single-container smoke run (not an RG-4 performance result)

`cargo run --release --bin bench_w29` completed successfully on the container above using the repository's `target-cpu=native` configuration. One wall-time sample gave the following **executed useful operations**; no warm/cold repetition, PMU cycles, confidence interval, or cross-framework control was collected:

| Workload | Observed useful pipe ops | Local wall-time ns/op (one sample) |
| --- | ---: | ---: |
| Single D-pipe accumulating DADD | 100,000 | 46.9 |
| Single S-pipe SGATHER | 100,000 | 52.7 |
| CNOP control | 0 | N/A (7.437 ms for 100,000 retired NOP SIWs) |
| 40 simulated sentrons, one process | 400,000 | 41.6 |
| 200 simulated sentrons, one process | 400,000 | 39.6 |

The benchmark also prints model-derived ops/cycle and cost/power based on its assumed AWS t3a.small parameters. **Those model values are not measurements of the test container**, nor proof of multicore or cluster scaling. The smoke-run numbers are diagnostic only; only the independently preregistered RG-4 harness can produce hardware throughput claims.

## Claims ledger

| Source / claim | At reviewed `exo` head | After this patch / remaining proof |
| --- | --- | --- |
| `src/bin/bench_w29.rs` reported D/S/C and 40/200-sentron throughput | **Invalid:** all timed `run()` calls returned from `Dormant` sentrons, retiring zero SIWs. | Calls `spawn` every iteration while moving the existing program allocation; counts `ExecStats::ops_retired` and `siws_retired`; asserts real state effects and retired counts. Output represents in-process wall-clock microbenchmarks, *not* physical IPC or distributed cluster throughput. |
| W29 CNOP baseline | **Invalid as useful GOPS:** NOP iterations were counted as if they were completed pipe operations. | Count **zero useful ops**, report elapsed time as a control baseline; no `ns/useful-op` or GOPS inference. |
| W29 hardware model | Wall-clock run mixed with assumed AWS cost, 2.5 GHz clock and TDP; historical hard-coded module/test counts. | Explicitly identifies assumed model; labels modeled ops/cycle, cost and power; removes hard-coded stale test-count banner. No auto-detection or PMU evidence. |
| `cost::bench_d_pipe` | **Invalid:** allocation-only loop, with no executor call. | Reuses preallocated SIW program, spawns actual sentrons, executes DADD, asserts accumulated architectural value, and reports actual retired ops. |
| W23 “8-core” benchmark | **Modeled / unverified:** assumes 5 GHz and runs cooperative simulation on one thread. | Not modified here; hardware and messaging claims remain unverified. |
| TTSM persistence | **Metadata only:** no durable payload byte replay on restart. | Not modified here; remains an RG-2 gate. |
| DSEL/DCMP/PPT/CSEND correctness | Source-level conflicts recorded in RG-001 review addendum. | Not modified here; separate RG-1A/coordinate/coordination gates. |

## Counting and scope conventions

“Useful operation” follows RG-001: one **executed** non-NOP D/S/C pipe operation. A retired NOP SIW is **not** one useful operation. The counted numerator comes from the executor's observed `ops_retired`, cross-checked against observed `siws_retired` and (for DADD/SGATHER) architectural register values. The denominator is measured **wall time of the local process**. Each simulated sentron runs in this process; the 200-sentron case is not a five-node cluster benchmark.

Per-iteration program reuse moves the existing vector through `Sentron::spawn` (no SIW clone/allocation in the timed loop), respecting the executor's `Dormant → Running → Retired` lifecycle. The executor itself may still allocate or perform non-pipe bookkeeping; these microbenchmarks are end-to-end interpreter timings, not isolated CPU instruction latency.

## Pending full RG-0 acceptance

- Pin and record `Cargo.lock`, toolchain, build flags and target CPU mode. Current `.cargo/config.toml` uses `target-cpu=native` and `.gitignore` ignores `Cargo.lock`.
- Run an approved named Ryzen 9 8945HS host with `perf_event_paranoid`, wall-time distribution, PMU cycles/instructions, thermal state, DDR5 bandwidth and power methodology recorded.
- Publish raw benchmark samples, their exact source SHA, input sizes, distribution and uncertainty; never substitute assumed GHz for measured CPU cycles.
- Treat historical W29 and cost numbers as **invalid**, not retroactively repaired by the existence of this patch. Reproduce fresh results before using them comparatively.
