# vTPU Autoresearch - Autonomous Performance Optimization

**Adapted from:** [Karpathy's autoresearch](https://github.com/karpathy/autoresearch)  
**Goal:** Autonomously improve vTPU performance metrics through iterative experimentation

## Overview

Instead of manually implementing W19 (dimensional prefetcher), we let an AI agent autonomously:
1. Modify vTPU source code (prefetch.rs, ppt.rs, etc.)
2. Run benchmarks (cargo run --release --bin bench)
3. Measure performance (ops/cycle, cache hit rate, memory bandwidth)
4. Keep improvements, revert failures
5. Iterate indefinitely until manually stopped

**The key insight from autoresearch:** "NEVER STOP" - runs overnight while you sleep, wakes you up to better performance.

## The Three-File Architecture (Adapted)

**1. Fixed files (read-only):**
- `benches/` - Benchmark harness (like prepare.py)
- `src/lib.rs` - Core types and traits
- `Cargo.toml` - Dependencies
- `tests/` - Test suite (must continue passing)

**2. Modifiable files (agent edits these):**
- `src/prefetch.rs` - NEW: Dimensional prefetcher implementation
- `src/ppt.rs` - Phext Page Table (optimization fair game)
- `src/exec.rs` - Execution engine (optimization fair game)
- `src/pool.rs` - Sentron pool (optimization fair game)

**3. Program file (human edits this):**
- `vtpu-autoresearch.md` - THIS FILE (agent instructions)

## Target Metrics

**Primary (maximize):**
- **ops_per_cycle** - Target: >3.0 (current: 2.93)
- **cache_hit_rate** - Target: >95% (current: varies)
- **memory_bandwidth_gb_s** - Target: >50 GB/s

**Secondary (constraints):**
- **compilation_time** - Soft constraint: <60s
- **test_pass_rate** - Hard constraint: 100% (all tests must pass)
- **simplicity** - All else equal, simpler code wins

## Setup

1. **Create experiment branch:**
   ```bash
   cd /source/vtpu
   git checkout -b vtpu-autoresearch/mar8
   ```

2. **Verify baseline:**
   ```bash
   cargo test --quiet  # All must pass
   cargo run --release --bin bench_w14 2>&1 | grep "ops/cycle\|L1 hit rate"
   ```

3. **Initialize results:**
   Create `vtpu-results.tsv`:
   ```
   commit	ops_cycle	cache_hit_pct	mem_bw_gbs	status	description
   <current>	2.93	0.0	0.0	baseline	Starting point before autoresearch
   ```

4. **Begin experimentation**

## The Autonomous Loop

**LOOP FOREVER:**

1. **Ideate:** Choose an optimization idea
   - Examples: prefetch predictor, cache-aware scheduling, SIMD optimizations
   - Read papers referenced in code
   - Combine previous near-misses
   - Try radical changes

2. **Modify:** Edit source files (prefetch.rs, ppt.rs, etc.)
   - One idea per iteration
   - Keep changes focused
   - Maintain code quality

3. **Commit:**
   ```bash
   git add -A
   git commit -m "Experiment: <brief description>"
   ```

4. **Build and test:**
   ```bash
   cargo test --quiet > test.log 2>&1
   if [ $? -ne 0 ]; then
       echo "TESTS FAILED - reverting"
       tail -50 test.log
       git reset --hard HEAD~1
       continue  # Try next idea
   fi
   ```

5. **Benchmark:**
   ```bash
   cargo run --release --bin bench_w14 > bench.log 2>&1
   ```

6. **Extract metrics:**
   ```bash
   grep "ops/cycle\|L1 hit rate\|Memory bandwidth" bench.log
   ```

7. **Record results:**
   - Append to vtpu-results.tsv
   - Log: commit hash, metrics, status (keep/discard), description

8. **Decide:**
   - If **ops_cycle improved** AND **tests pass**: KEEP (branch advances)
   - If **ops_cycle same/worse** OR **tests fail**: DISCARD (git reset --hard HEAD~1)
   - Special: If **simpler** with same/slightly-worse perf: KEEP (simplification win)

9. **GOTO 1** (never ask permission, never stop)

## Benchmark Command

**Quick iteration (for agent):**
```bash
cargo run --release --bin bench_w14 2>&1 | tee bench.log
```

**Extract metrics:**
```bash
# ops/cycle
grep "ops/cycle" bench.log | tail -1

# Cache hit rate (when implemented)
grep "L1.*hit rate" bench.log | tail -1

# Memory bandwidth (when implemented)
grep "Memory bandwidth" bench.log | tail -1
```

## Simplicity Criterion (from autoresearch)

> "All else being equal, simpler is better."

**Examples:**
- 0.01 ops/cycle gain + 50 lines complex code → **SKIP**
- 0.01 ops/cycle gain by deleting code → **DEFINITELY KEEP**
- 0.00 ops/cycle change but much cleaner → **KEEP** (simplification win)

**Complexity must justify improvement magnitude.**

## NEVER STOP Principle

Once experimentation begins:
- Do NOT ask "should I continue?"
- Do NOT ask "is this a good stopping point?"
- Human might be asleep (expects you to work overnight)
- Run until manually interrupted
- If out of ideas: think harder, read code, try combinations

**Example use case:** Run overnight while Will sleeps
- ~10 min/experiment (build + test + bench)
- 6 experiments/hour
- 48 experiments over 8-hour sleep
- Wake up to performance improvements

## Crash/Failure Handling

**If tests fail:**
- Read test output: `tail -50 test.log`
- If fixable (typo, logic error): fix and retry
- If unfixable (idea broken): revert and try next idea

**If benchmark crashes:**
- Read stack trace: `tail -50 bench.log`
- If fixable: fix and retry
- If unfixable: revert and try next idea

**Timeout:**
- If any step exceeds 10 minutes: kill and treat as failure

## Initial Optimization Ideas

**For the first few iterations:**

1. **Implement basic prefetcher** (W19 goal)
   - Track last N coordinate accesses
   - Predict next based on pattern (L+1, S+1, etc.)
   - Warm cache proactively

2. **Sequential scan optimization**
   - Detect L+1, V+1, Scroll+1 patterns
   - Prefetch next K coordinates ahead

3. **SIMD optimization**
   - Vectorize coordinate calculations
   - Batch PPT translations

4. **Cache-aware scheduling**
   - Group operations by coordinate locality
   - Reduce cache thrashing

5. **Memory layout optimization**
   - Reorder struct fields for cache lines
   - Align hot paths

**Later iterations:** Get creative, read papers, try combinations

## Success Metrics (Phase 2 exit criteria)

**To complete Phase 2 (Memory Hierarchy):**
- ops/cycle ≥ 3.0 (currently 2.93, need +0.07)
- L1 cache hit rate ≥ 95% (implement measurement first)
- Memory bandwidth ≥ 50 GB/s (implement measurement first)
- All tests passing (100% pass rate maintained)
- Code quality maintained (no spaghetti)

**When metrics achieved:** Declare W19-W24 complete, advance to Phase 3

## Results Format (TSV)

**vtpu-results.tsv:**
```
commit	ops_cycle	cache_hit_pct	mem_bw_gbs	status	description
a1b2c3d	2.93	0.0	0.0	baseline	Starting point
b2c3d4e	2.95	0.0	0.0	keep	Basic prefetcher (sequential L+1)
c3d4e5f	2.92	0.0	0.0	discard	Tried SIMD, slowed down
d4e5f6g	2.97	0.0	0.0	keep	Cache-aware PPT layout
```

**Status values:**
- `keep` - Improvement kept, branch advanced
- `discard` - No improvement, reverted
- `crash` - Build/test/bench failed
- `simplify` - Kept for simplification (not perf)

## Agent Instructions

**You are autonomous vTPU performance engineer.**

**Your only goals:**
1. Increase ops/cycle toward 3.0+
2. Maintain 100% test pass rate
3. Keep code simple and readable
4. Never stop until manually interrupted

**You can:**
- Modify any src/*.rs file (except lib.rs core traits)
- Add new files to src/
- Use any Rust features/patterns
- Read code, read papers, get creative

**You cannot:**
- Modify benchmarks (benches/*.rs)
- Modify tests (must keep passing)
- Add dependencies (Cargo.toml frozen)
- Ask for permission (autonomous operation)

**The loop runs forever. Make vTPU faster. That's it.**

---

**Created:** March 8, 2026 (Epoch Day 804)  
**Author:** Phex 🔱  
**Based on:** Karpathy's autoresearch (autonomous AI research)  
**Goal:** Rapid vTPU optimization through autonomous experimentation
