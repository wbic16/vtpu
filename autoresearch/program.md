# vTPU Autoresearch

Autonomous optimization loop for vTPU (Virtual Temporal Processing Unit).

## Setup

To set up a new experiment run:

1. **Agree on a run tag**: propose a tag based on today's date (e.g. `mar8`). The branch `autoresearch/<tag>` must not already exist.
2. **Create the branch**: `git checkout -b autoresearch/<tag>` from current main.
3. **Read the in-scope files**: 
   - `src/siw.rs` — SIW (SIMD Instruction Word) format and execution
   - `src/bin/cpi_bench.rs` — the benchmark you'll run
   - `src/cache_sim.rs` — cache simulation (read-only)
   - `Cargo.toml` — dependencies (do not add new ones)
4. **Initialize results.tsv**: Create `autoresearch/results.tsv` with header row and baseline entry.
5. **Confirm and go**: Confirm setup looks good.

## The Goal

**Maximize ops/cycle (CPI).** Target: 3.0 ops/cycle on Zen 4 (AMD Ryzen 9).

The vTPU is a phase-aware computation architecture. Each SIW executes 3 operations in parallel:
- D-Pipe: Dense compute (FMA, matmul)
- S-Pipe: Sparse/associative (HDC, search)
- C-Pipe: Communication (broadcast, gather)

## Experimentation

Each experiment runs on a single core. The benchmark runs for a **fixed 1-minute time budget**.

**What you CAN modify:**
- `src/siw.rs` — SIW format, operation encoding, execution logic
- `src/bin/cpi_bench.rs` — benchmark structure, workload generation
- `src/pipeline.rs` — pipeline scheduling (if exists)
- Any `src/*.rs` file EXCEPT cache_sim.rs

**What you CANNOT modify:**
- `src/cache_sim.rs` — read-only, fixed evaluation
- `Cargo.toml` — no new dependencies
- Test files — don't break existing tests

**Run command:**
```bash
cargo build --release --bin cpi_bench && \
  perf stat -e cycles,instructions ./target/release/cpi_bench > run.log 2>&1
```

**Extract results:**
```bash
# Get IPC from perf output
grep "insn per cycle" run.log | awk '{print $4}'
# Or calculate: instructions / cycles
```

## Output format

The benchmark should print:
```
---
ops_per_cycle:    2.847
total_ops:        1000000000
total_cycles:     351234567
peak_memory_mb:   128.5
cache_hit_rate:   0.943
```

## Logging results

Log to `autoresearch/results.tsv` (tab-separated):

```
commit	ops_cycle	memory_mb	status	description
```

Example:
```
commit	ops_cycle	memory_mb	status	description
a1b2c3d	2.847	128.5	keep	baseline
b2c3d4e	2.912	130.2	keep	prefetch hints in D-pipe
c3d4e5f	2.801	125.0	discard	removed phase tracking
d4e5f6g	0.000	0.0	crash	invalid SIMD intrinsic
```

## The experiment loop

LOOP FOREVER:

1. Look at git state: current branch/commit
2. Modify vTPU code with experimental idea
3. `cargo fmt && cargo clippy --release` (must pass)
4. `cargo test --release` (must pass — don't break tests)
5. git commit
6. Run benchmark: `cargo build --release --bin cpi_bench && perf stat -e cycles,instructions ./target/release/cpi_bench > run.log 2>&1`
7. Extract ops/cycle from results
8. If crashed, `tail -n 50 run.log` to diagnose
9. Record in results.tsv
10. If ops/cycle improved → keep commit
11. If worse → `git reset --hard HEAD~1`
12. **NEVER STOP** — continue indefinitely until manually stopped

## Ideas to try

### Phase 1: Low-hanging fruit
- [ ] Better SIMD utilization (AVX-512 if available)
- [ ] Instruction reordering to reduce stalls
- [ ] Prefetch hints for D-pipe data
- [ ] Cache-aware batch sizing

### Phase 2: Pipeline optimization
- [ ] Out-of-order execution within SIW
- [ ] Speculative S-pipe execution
- [ ] C-pipe coalescing

### Phase 3: Architectural changes
- [ ] Wider SIW (4 or 5 ops instead of 3)
- [ ] Variable-width SIW based on workload
- [ ] Hypervector dimension tuning for HDC ops

## Constraints

- **1-minute budget** per experiment
- **Target: 3.0 ops/cycle** (currently ~2.5)
- **Memory soft limit**: Don't exceed 2x baseline
- **Simplicity wins**: Simpler code at equal performance is better

## NEVER STOP

Once experimentation begins, do NOT pause to ask the human if you should continue. The human may be asleep. Run autonomously until manually stopped. If stuck, try more radical changes. Read research papers. Combine near-misses. The loop runs until interrupted.

---

*"The hardware was always sufficient. The software just needed phext."*
