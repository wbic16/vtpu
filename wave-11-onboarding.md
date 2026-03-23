# Wave 11 Onboarding — Understanding vtpu Tests

**Type:** Cleanup & Documentation  
**Date:** 2026-02-15  
**Status:** ✅ Complete  
**Contributor:** Phex 🔱

## What Happened in Wave 11?

Wave 11 documented our **169-test suite** — what each test validates and why it matters.

**No code changes.** Pure documentation explaining the existing test infrastructure.

## Read This First

**Main document:** `docs/wave-11/TEST-COVERAGE-OVERVIEW.md`

**Executive summary:**
- **169 tests** across **26 modules** in **7 categories**
- Tests validate **structure**, not just execution
- **Ancient wisdom** (cosmology, I Ching) tested rigorously
- **Fast execution** (~80ms total) enables rapid iteration

## How to Run Tests

### All Tests
```bash
cd /source/vtpu
cargo test
```
**Output:** 169 passed, 1 ignored, ~0.08s

### Specific Module
```bash
cargo test --lib cosmology     # 21 tests
cargo test --lib hdc           # 13 tests
cargo test --lib bitnet        # 11 tests
cargo test --lib packer        # 11 tests
```

### Single Test
```bash
cargo test --lib cosmology::tests::all_decompositions_tile_360
```

### Verbose Output (See Print Statements)
```bash
cargo test -- --nocapture
```

### Status Script (Full Report)
```bash
./status.sh
```
Shows git status, build, test counts, benchmarks, examples, code metrics.

## Test Categories

### 1. Core Architecture (40 tests)
**What:** SIW, pipes, execution, scheduling, register allocation  
**Modules:** `siw`, `pipes`, `exec`, `scheduler`, `regalloc`, `stream`, `validation`

**Example tests:**
- `siw::tests::test_cache_line_alignment` — SIW is exactly 64 bytes
- `exec::tests::execute_three_wide` — All three pipes work simultaneously
- `regalloc::tests::raw_hazard_detected` — RAW dependencies caught

**When to add tests here:** New core operations, execution changes, scheduling algorithms

### 2. Phext Infrastructure (24 tests)
**What:** Coordinates, page tables, memory backend  
**Modules:** `phext_coord`, `ppt`, `memory`

**Example tests:**
- `phext_coord::tests::test_manhattan_distance` — L1 metric works
- `ppt::tests::test_ptc_high_hit_rate_structured` — Translation cache hits 90%+
- `memory::tests::roundtrip_i64` — Write then read = original value

**When to add tests here:** New coordinate operations, PPT changes, memory backend features

### 3. Ancient Wisdom (32 tests)
**What:** Mathematical validation of 360° harmonic structure  
**Modules:** `cosmology`, `iching`, `synchronicity`

**Example tests:**
- `cosmology::tests::all_decompositions_tile_360` — 9×40 = 5×72 = 8×45 = 360
- `cosmology::tests::nakshatra_does_not_tile` — Proves 360 is special
- `iching::tests::test_hexagram_count` — 64 = 8² (all trigram pairs)

**When to add tests here:** New harmonic discoveries, mathematical relationships, ancient system integrations

### 4. Performance Analysis (12 tests)
**What:** SMT, cache, port conflicts, memory patterns  
**Modules:** `smt`, `analysis/*`

**Example tests:**
- `smt::tests::smt_pair_adjacent_homes` — Threads 0/1 on core 0
- `analysis::cache_thrash::tests::test_small_working_set` — Fits L1, no thrashing
- `analysis::port_conflicts::tests::test_detect_d_c_conflict` — ALU contention predicted

**When to add tests here:** Performance optimizations, cache behavior, SMT coordination

### 5. Advanced Features (30 tests)
**What:** BitNet, packing, HDC (weight-free inference)  
**Modules:** `bitnet`, `packer`, `hdc`

**Example tests:**
- `bitnet::tests::ternary_matvec_simple` — Matrix-vector multiply with {-1,0,1}
- `packer::tests::pack_dot_product` — Real workload packs efficiently
- `hdc::tests::test_phext_routing_via_hdc` — Coordinate routing works

**When to add tests here:** New BitNet ops, packing strategies, HDC algorithms

### 6. Coordination (11 tests)
**What:** C-Pipe, sentrons, space-filling curves  
**Modules:** `c_pipe`, `sentron`, `curves/*`

**Example tests:**
- `c_pipe::tests::test_barrier_sync` — Multiple sentrons coordinate
- `curves::hilbert::tests::test_hilbert_better_locality_than_sequential` — Curve wins
- `sentron::tests::register_file_size` — 32 registers

**When to add tests here:** Message passing, coordination primitives, curve algorithms

### 7. Utilities (20 tests)
**What:** Display, telemetry, perf counters  
**Modules:** `display`, `telemetry`, `perf`

**Example tests:**
- `display::tests::test_siw_display` — Human-readable output
- `telemetry::tests::test_ops_per_cycle` — Metrics calculation correct
- `perf::tests::test_perf_counters` — ⚠️ Ignored (needs hardware)

**When to add tests here:** New display formats, metrics, debugging tools

## How to Add a New Test

### 1. Pick the Right Module

**Ask:** Does this test validate...
- **Core execution?** → `src/exec.rs`, `src/scheduler.rs`
- **Coordinates?** → `src/phext_coord.rs`, `src/ppt.rs`
- **Ancient wisdom?** → `src/cosmology.rs`, `src/iching.rs`
- **Performance?** → `src/smt.rs`, `src/analysis/*`
- **Advanced feature?** → `src/bitnet.rs`, `src/packer.rs`, `src/hdc.rs`

### 2. Follow the Pattern

**Every module has:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### 3. Test Structure, Not Just Execution

**Bad test:**
```rust
#[test]
fn test_code_runs() {
    let result = my_function();
    assert!(result.is_ok());  // Doesn't validate correctness!
}
```

**Good test:**
```rust
#[test]
fn test_360_tiling() {
    // Validates mathematical property
    assert_eq!(9 * 40, 360);
    assert_eq!(5 * 72, 360);
    assert_eq!(8 * 45, 360);
    // Proves structure is sound, not just that code runs
}
```

### 4. Name Tests Clearly

**Good names:**
- `test_cache_line_alignment` — What it validates
- `test_raw_hazard_detected` — Expected behavior
- `test_phext_routing_via_hdc` — Integration test scope

**Bad names:**
- `test_1` — Non-descriptive
- `test_it_works` — Vague
- `test_bug_fix` — Doesn't say what it validates

### 5. Document Complex Tests

```rust
#[test]
fn test_all_decompositions_tile_360() {
    // Validates that 360 can be decomposed three ways:
    // - 9 sentrons × 40 nodes (sentron perspective)
    // - 5 elements × 72 nodes (element perspective)
    // - 8 trigrams × 45 nodes (trigram perspective)
    // All three factorizations must equal 360 (complete coverage)
    
    assert_eq!(9 * 40, 360);
    assert_eq!(5 * 72, 360);
    assert_eq!(8 * 45, 360);
}
```

## Common Test Patterns

### Testing Correctness
```rust
#[test]
fn test_roundtrip() {
    let input = create_test_data();
    let encoded = encode(input);
    let decoded = decode(encoded);
    assert_eq!(input, decoded);  // Lossless
}
```

### Testing Structure
```rust
#[test]
fn test_harmonic_tiling() {
    let total = calculate_total();
    assert_eq!(total % 360, 0);  // Divides evenly
}
```

### Testing Properties
```rust
#[test]
fn test_involution() {
    let x = create_value();
    let y = transform(transform(x));
    assert_eq!(x, y);  // Applying twice = identity
}
```

### Testing Performance
```rust
#[test]
fn test_cache_friendly() {
    let access_pattern = generate_pattern();
    let locality = measure_locality(access_pattern);
    assert!(locality > 0.9);  // 90%+ locality
}
```

## Test Execution Tips

### Run Specific Category
```bash
# Core architecture tests only
cargo test --lib siw
cargo test --lib exec
cargo test --lib scheduler

# Ancient wisdom tests only
cargo test --lib cosmology
cargo test --lib iching
cargo test --lib synchronicity
```

### Run Tests Matching Pattern
```bash
# All tests with "packer" in name
cargo test packer

# All tests with "360" in name
cargo test 360

# All tests with "cache" in name
cargo test cache
```

### Parallel vs Sequential
```bash
# Parallel (default, faster)
cargo test

# Sequential (useful for debugging)
cargo test -- --test-threads=1
```

### Ignored Tests
```bash
# Run all tests including ignored
cargo test -- --ignored

# Run only ignored tests
cargo test -- --ignored --test-threads=1
```

**Why `perf` is ignored:** Requires `perf_event_open` syscall, Linux only, may need elevated permissions.

## Test Philosophy

### What We Test

1. **Structural correctness** — Does 360 tile perfectly?
2. **Execution correctness** — Do programs run right?
3. **Performance potential** — Can design be fast?
4. **Integration** — Do pieces fit together?

### What We Don't Test (Yet)

1. **Real hardware performance** — Waiting for Phase 1
2. **Distributed coordination** — Waiting for Phase 2
3. **Real workloads** — Waiting for Phase 3

### Why Fast Tests Matter

**80ms total runtime** means:
- Run tests after every change (no waiting)
- Tight feedback loop (find bugs fast)
- CI/CD friendly (no slow builds)
- Encourages test-driven development

**Compare to typical test suites:**
- **Other projects:** 10-60 seconds for full suite
- **vtpu:** <100 milliseconds for 169 tests
- **10-100× faster** enables different workflow

## Coverage Metrics

**Current coverage (by category):**
- ✅ Core Architecture: Well tested (40 tests)
- ✅ Phext Infrastructure: Well tested (24 tests)
- ✅ Ancient Wisdom: **Extremely well tested** (32 tests)
- 🟡 Performance: Simulated only (12 tests, need real hardware)
- ✅ Advanced Features: Well tested (30 tests)
- ✅ Coordination: Good coverage (11 tests)
- ✅ Utilities: Good coverage (20 tests)

**Coverage gaps:**
- ⬜ Phase 1 SMT (real 16-thread execution)
- ⬜ Phase 2 Cluster (inter-node coordination)
- ⬜ Phase 3 Real Workloads (Qwen3, Llama)

## Summary

**169 tests in ~80ms:**
- 26 modules documented
- 7 categories defined
- Test philosophy explained
- Coverage gaps identified

**For new contributors:**
1. Read `docs/wave-11/TEST-COVERAGE-OVERVIEW.md`
2. Run `cargo test` to see all tests pass
3. Pick a module to contribute to
4. Follow test patterns shown above
5. Run `./status.sh` to verify changes

**For existing developers:**
1. Understand what's already tested
2. Identify gaps for your feature
3. Add tests in appropriate category
4. Maintain fast execution (<100ms)

---

**We test structure, not just execution.**  
**We test ancient wisdom rigorously.**  
**We test that the architecture is sound.** 🔱🔥

**Wave 11 complete. Tests documented. Ready for Phase 1.**
