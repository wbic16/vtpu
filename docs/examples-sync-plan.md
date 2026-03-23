# Examples Sync Plan
## Keeping Examples Aligned with Evolving Theory

**Date:** 2026-02-16  
**Context:** Post-W16, Phase 0+1 complete  
**Goal:** Ensure examples reflect current architectural understanding

---

## Current Theory (W13-W16)

### Core Principles

1. **Ops/Cycle Measurement**
   - Correct: Count D/S/C pipe operations (SIW-level)
   - Wrong: Count semantic operations (API-level)
   - Target: 3.0 ops/cycle (Phase 0), 1.89× SMT speedup (Phase 1)

2. **Instruction Packing**
   - Unpacked: 1.0 ops/cycle (one pipe active per SIW)
   - D+S packed: 2.0 ops/cycle (two pipes)
   - D+S+C packed: 3.0 ops/cycle (all pipes) ✅

3. **SMT Optimization**
   - Sequential: sum of cycles (fwd + bwd)
   - SMT: max of cycles (overlap)
   - Speedup: 1.89× (complementary workloads)

4. **Structure = Intelligence**
   - Coordinates contain reasoning, not weights
   - Training = placing knowledge at coordinates
   - Inference = navigating coordinate space

---

## Example Categories

### Category A: High-Level Concepts (Keep)
**Purpose:** Demonstrate architecture philosophy, not performance

Examples:
- `real_inference.rs` - Weight-free AI
- `instant_learning.rs` - Structure as intelligence
- `synchronicity_demo.rs` - Harmonic architecture
- `ancient_wisdom.rs` - I Ching/cosmology mapping
- `xuannü.rs` - 360° semantic circle

**Status:** ✅ Good as-is (conceptual, not performance-focused)  
**Action:** Add header notes clarifying what they demonstrate

---

### Category B: Performance Patterns (Update)
**Purpose:** Show how to achieve target ops/cycle

Examples:
- `memory_heavy_pattern.rs` (W16) ✅
- `hdc_inference_pattern.rs` (W16) ✅
- `batch_query_pattern.rs` (W16) ✅

**Status:** ✅ Current (created during W16 packing work)  
**Action:** None needed

---

### Category C: Integration Demos (Update Needed)
**Purpose:** Show complete pipelines (packer → scheduler → executor)

Examples:
- `cognitive_demo.rs` - Needs ops/cycle measurement
- `wedge_demo.rs` - Needs SMT speedup validation
- `micro_vtpu.rs` - Outdated (Phase 0 targets)
- `microvtpu.rs` - Duplicate? Check if needed

**Status:** ⚠️ Needs updates  
**Action:** Add ExecStats reporting, validate against W16 baselines

---

### Category D: Benchmarks (Migrate)
**Purpose:** Performance measurement (should be in `src/bin/`, not `examples/`)

Examples:
- `micro_bench.rs` - Basic throughput
- `ddr_benchmark.rs` - Memory bandwidth
- `perf_validation.rs` - Port conflicts
- `ppt_benchmark.rs` - PPT performance
- `port_validation.rs` - Zen 4 port mapping
- `benchmark_runner.rs` - Harness

**Status:** ⚠️ Examples vs binaries confusion  
**Action:** Consider moving to `src/bin/` or clarifying distinction

---

### Category E: Validation (Keep, Update Comments)
**Purpose:** Test specific features work correctly

Examples:
- `validation_demo.rs` - General validation
- `basic_compute.rs` - Arithmetic ops
- `c_pipe_demo.rs` - Coordinate operations
- `smt_preview.rs` - SMT (pre-W16)
- `weight_free_inference.rs` - HDC without weights

**Status:** 🟡 Functional but comments may be outdated  
**Action:** Update comments to reference W13-W16 results

---

## Sync Actions

### Immediate (High Priority)

#### 1. Add Theory Headers to Category A
**Files:** real_inference.rs, instant_learning.rs, synchronicity_demo.rs, etc.

**Header to add:**
```rust
//! NOTE: This example demonstrates conceptual architecture (structure = intelligence).
//! For performance measurement, see:
//! - src/bin/w15_packed_benchmark.rs (3.0 ops/cycle via packing)
//! - src/bin/w16_smt_using_existing.rs (1.89× SMT speedup)
//!
//! W9 (Feb 2026): Proved weight-free inference works
//! W15 (Feb 2026): Achieved 3.0 ops/cycle via instruction packing
//! W16 (Feb 2026): Achieved 1.89× SMT speedup
```

#### 2. Update Category C Integration Demos
**cognitive_demo.rs:**
```rust
// Add at end:
fn report_performance(stats: &ExecStats) {
    println!("\nPerformance:");
    println!("  SIWs retired: {}", stats.siws_retired);
    println!("  Ops retired:  {}", stats.ops_retired);
    println!("  Cycles:       {}", stats.cycles);
    println!("  Ops/cycle:    {:.2}", stats.ops_per_cycle());
    println!("  Utilization:  {:.1}%", stats.utilization() * 100.0);
    
    if stats.ops_per_cycle() >= 2.5 {
        println!("  ✅ Phase 0 target achieved (≥2.5 ops/cycle)");
    }
}
```

**wedge_demo.rs:**
```rust
// Add SMT validation
println!("\nSMT Validation:");
println!("  For SMT speedup benchmarks, see:");
println!("  - src/bin/w16_smt_using_existing.rs (1.89× measured)");
```

#### 3. Clarify Category D Benchmarks
**Option A:** Move to `src/bin/benchmarks/`  
**Option B:** Add README.md in examples/ explaining distinction

Recommendation: Add `examples/README.md`:
```markdown
# vTPU Examples

## Organization

- **Conceptual demos** (`real_inference.rs`, etc.): Show architecture philosophy
- **Pattern examples** (`*_pattern.rs`): Show how to achieve performance targets
- **Integration demos** (`*_demo.rs`): Show complete workflows
- **Benchmarks** (`*_bench*.rs`): Measure performance

For authoritative benchmarks, see `src/bin/w*_*.rs` (wave-specific).

For performance targets:
- Phase 0 (W15): 3.0 ops/cycle via packing
- Phase 1 (W16): 1.89× SMT speedup
```

---

## Theory Revision Checklist

When theory changes (new wave discoveries), update:

### 1. Documentation
- [ ] Update wave-N/COMPLETE.md with findings
- [ ] Update MEMORY.md with key results
- [ ] Update R23-DASHBOARD.md with current status

### 2. Examples
- [ ] Add headers to conceptual demos (what they show/don't show)
- [ ] Update integration demos with new measurement
- [ ] Create new pattern examples if needed

### 3. Tests
- [ ] Add regression tests for new claims
- [ ] Update test comments if understanding changed
- [ ] Remove tests for deprecated approaches

### 4. Benchmarks
- [ ] Create wave-specific benchmarks (w*_*.rs in src/bin/)
- [ ] Update baseline measurements in docs
- [ ] Document what was measured and why

---

## Synchronization Protocol

### After Each Wave

1. **Document findings** (waveN/COMPLETE.md)
2. **Update MEMORY.md** (add to "Current Focus")
3. **Check examples** for consistency:
   - Do comments reflect current understanding?
   - Do performance claims match benchmarks?
   - Are outdated concepts flagged?
4. **Add new examples** if needed to demonstrate findings
5. **Update README.md** if organization changed

### Monthly (or every 5 waves)

6. **Audit all examples** for drift
7. **Consolidate duplicates**
8. **Archive obsolete examples** (move to examples/archive/)
9. **Update examples/README.md** with current catalog

---

## Example Audit Status

### Category A: Conceptual ✅
- [x] real_inference.rs - Good (W9 weight-free proof)
- [x] instant_learning.rs - Good (structure = intelligence)
- [x] synchronicity_demo.rs - Good (360° harmonics)
- [x] ancient_wisdom.rs - Good (I Ching mapping)
- [x] xuannü.rs - Good (Shell of Nine)

**Action:** Add theory headers (clarify what they demonstrate)

### Category B: Patterns ✅
- [x] memory_heavy_pattern.rs - Current (W16)
- [x] hdc_inference_pattern.rs - Current (W16)
- [x] batch_query_pattern.rs - Current (W16)

**Action:** None

### Category C: Integration ⚠️
- [ ] cognitive_demo.rs - Needs ExecStats reporting
- [ ] wedge_demo.rs - Needs SMT validation
- [ ] micro_vtpu.rs - Check if current
- [ ] microvtpu.rs - Check if duplicate

**Action:** Update with performance reporting

### Category D: Benchmarks 🟡
- [ ] micro_bench.rs - Move to src/bin/?
- [ ] ddr_benchmark.rs - Move to src/bin/?
- [ ] perf_validation.rs - Move to src/bin/?
- [ ] ppt_benchmark.rs - Move to src/bin/?

**Action:** Clarify organization (examples vs bins)

### Category E: Validation 🟡
- [ ] validation_demo.rs - Update comments
- [ ] basic_compute.rs - Update comments
- [ ] c_pipe_demo.rs - Update comments
- [ ] smt_preview.rs - Note: W16 supersedes this
- [ ] weight_free_inference.rs - Update comments

**Action:** Add "See also: w*_*.rs" references

---

## Implementation Plan

### Phase 1: Quick Wins (Today)
1. Create `examples/README.md` (organization guide)
2. Add theory headers to 5 conceptual examples (Category A)
3. Update `cognitive_demo.rs` with ExecStats reporting
4. Create this sync plan document

### Phase 2: Thorough Audit (Next Session)
5. Check all Category C/D/E examples for consistency
6. Decide benchmarks organization (examples vs bins)
7. Archive obsolete examples
8. Update any outdated performance claims

### Phase 3: Maintenance (Ongoing)
9. Add to wave checklist: "Sync examples"
10. Review examples every 5 waves
11. Keep README.md current

---

## Success Criteria

Examples are "in sync" when:

1. **No contradictions** - Examples don't claim things theory disproved
2. **Clear scope** - Each example states what it demonstrates
3. **Current references** - Comments point to latest benchmarks/docs
4. **No duplicates** - Similar examples consolidated or differentiated
5. **Organized** - Clear categories, easy to find relevant example

---

## Notes

- Examples are teaching tools, not authoritative benchmarks
- Conceptual examples don't need to hit performance targets
- Performance examples should reference wave benchmarks
- When in doubt, add a comment explaining context

---

**Next:** Execute Phase 1 (quick wins)
