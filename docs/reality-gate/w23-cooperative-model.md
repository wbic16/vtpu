# W23 execution and measurement contract

**Scope:** `src/bin/bench_w23_8core.rs`, `src/coop_smt.rs`, and `src/coop_fleet.rs`.

W23 is an **eight-virtual-sentron, one-OS-thread cooperative model**. It does
not establish physical eight-core throughput, simultaneous multithreading
efficiency, hardware instructions-per-cycle, or cache-crosstalk speedups.

The original W23 harness passed a C-pipe messaging program to
`coop_execute`. That executor schedules sentrons but does not route `CSEND`
outboxes to `CRECV` inboxes; a missing message yields zero. Its simulated
cycle count increments once per SIW. It is not a physical CPU cycle counter.

The corrected coordinated harness calls `coop_fleet_execute`, which drains
outboxes into recipient inboxes after each quantum. Its delivery counter must
include messages queued in sentron outboxes *before* a fleet flush; counting
only `Fleet::pending_messages()` initially missed those messages. The focused
tests verify message delivery and the accounting correction.

## Measurement labels

| W23 quantity | Interpretation |
| --- | --- |
| Wall time and SIWs/s | Measured on the current host for this synthetic workload; no hardware comparability without a run manifest |
| 5.0 GHz | Assumed conversion factor, **not observed clock frequency** |
| SIWs/assumed cycle | Virtual SIW rate divided by wall time × nominal frequency; **not hardware IPC** |
| Eight sentrons | Virtual execution contexts interleaved on one OS thread; **not eight cores or four SMT pairs** |
| Relative SIWs/s | Descriptive comparison of two **different** synthetic workloads; **not an eight-core speedup** |
| `messages_delivered` | Message queue deliveries at fleet flush; does not establish that all `CRECV`s received a value |
| `CBAR(count=2)` | Nonblocking marker in the current OctaWire path; **not paired synchronization** |
| S-pipe prefetch | Execution of prefetch hints; **not proof of L1 residency or cache sharing** |

The historical W23 thresholds of 2.5 and 3.0 virtual operations/cycle are
goals, **not exit gates that W23 can certify**. A valid physical experiment
requires a named host, pinned CPU threads, measured PMU cycles when available,
thermal/frequency provenance, and an equivalent workload baseline (RG-4).

## Reproduction

```bash
cargo test --lib coop_fleet::tests::send_recv_across_fleet
cargo test --bin bench_w23_8core
cargo test --all-targets
```

No extra crates, privileged counters, or real cluster services are required.
