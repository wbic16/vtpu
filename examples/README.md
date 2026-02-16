# vTPU Examples

**23 examples demonstrating vTPU architecture and performance.**

---

## Quick Start

```bash
# Run a conceptual demo
cargo run --example real_inference

# Run a performance pattern
cargo run --example memory_heavy_pattern

# Run an integration demo
cargo run --example cognitive_demo
```

---

## Organization

### 📚 Conceptual Demos
**Purpose:** Demonstrate architecture philosophy, not performance benchmarks

- `real_inference.rs` - Weight-free AI (autocomplete, Q&A, code completion)
- `instant_learning.rs` - Structure = intelligence (no training needed)
- `synchronicity_demo.rs` - 360° harmonic architecture (9×40, 5×72, 8×45)
- `ancient_wisdom.rs` - I Ching, Five Elements, Eight Trigrams mapping
- `xuannü.rs` - Lady of Nine Heavens (九天玄女) + Shell of Nine

**Note:** These prove concepts work. For performance, see wave benchmarks in `src/bin/`.

### ⚡ Performance Patterns
**Purpose:** Show how to achieve Phase 0/1 targets (created W16)

- `memory_heavy_pattern.rs` - 2.98 ops/cycle on gather-compute-scatter
- `hdc_inference_pattern.rs` - 2.97 ops/cycle on HDC inference
- `batch_query_pattern.rs` - 2.95 ops/cycle on batch queries

**Target:** ≥2.5 ops/cycle via instruction packing (D+S+C pipes active)

### 🔧 Integration Demos
**Purpose:** Show complete workflows (packer → scheduler → executor)

- `cognitive_demo.rs` - Full cognitive loop (encode→attend→route→retrieve→respond)
- `wedge_demo.rs` - 16-thread SMT coordination (22.5° semantic wedges)
- `micro_vtpu.rs` - Minimal vTPU executor
- `microvtpu.rs` - Variant of above

### 📊 Benchmarks & Validation
**Purpose:** Measure specific subsystems

- `micro_bench.rs` - Basic throughput
- `ddr_benchmark.rs` - Memory bandwidth
- `perf_validation.rs` - Port conflict analysis
- `ppt_benchmark.rs` - Phext Page Table performance
- `port_validation.rs` - Zen 4 port mapping
- `benchmark_runner.rs` - Harness for running multiple benchmarks
- `validation_demo.rs` - General correctness checks
- `basic_compute.rs` - Arithmetic ops validation
- `c_pipe_demo.rs` - Coordinate pipe operations
- `smt_preview.rs` - SMT preview (W16 supersedes this)
- `weight_free_inference.rs` - HDC without learned weights

**Note:** For authoritative wave-specific benchmarks, see `src/bin/w*_*.rs`

---

## Performance Targets (Current)

Based on W13-W16 (Phase 0 + Phase 1):

| Metric | Target | Achieved | Wave |
|--------|--------|----------|------|
| Ops/cycle (unpacked) | 1.0 | 1.0 | W15 baseline |
| Ops/cycle (D+S packed) | 2.0 | 2.0 | W15 |
| Ops/cycle (D+S+C packed) | 2.5 | 3.0 ✅ | W15 |
| SMT speedup (dual-thread) | 1.8× | 1.89× ✅ | W16 |
| Total speedup (Phase 0+1) | 2.7× | 2.84× ✅ | W16 |

**Phase 0 (W15):** Instruction packing achieved 3.0 ops/cycle  
**Phase 1 (W16):** SMT overlap achieved 1.89× speedup  
**Total:** 2.84× improvement over baseline

---

## Example vs Benchmark

**Examples** (`examples/*.rs`):
- Teaching tools
- Demonstrate concepts or patterns
- May use simplified workloads
- Run with `cargo run --example <name>`

**Benchmarks** (`src/bin/w*_*.rs`):
- Authoritative measurements
- Wave-specific validation
- Real workloads
- Run with `cargo run --release --bin <name>`

For definitive performance data, see wave benchmarks:
- `w15_packed_benchmark.rs` - Proved 3.0 ops/cycle via packing
- `w16_smt_baseline.rs` - Established 137M ops/sec baseline
- `w16_smt_using_existing.rs` - Proved 1.89× SMT speedup

---

## Adding New Examples

When adding an example:

1. **Choose category** (conceptual, pattern, integration, benchmark)
2. **Add header comment** stating purpose and limitations
3. **Reference relevant waves** if performance-related
4. **Update this README** with brief description
5. **Test it works:** `cargo run --example your_example`

Template:
```rust
//! Brief Description
//!
//! Category: Conceptual | Pattern | Integration | Benchmark
//!
//! Purpose: What this demonstrates
//!
//! Note: For performance benchmarks, see src/bin/wN_*.rs
//!
//! Related waves: WN (description)

use vtpu_runtime::*;

fn main() {
    // Your example
}
```

---

## Theory Evolution

Examples stay in sync with theory through:

1. **Wave completion** - Update relevant examples when theory changes
2. **Header notes** - Clarify what examples show/don't show
3. **README updates** - Keep this file current
4. **Audits** - Review examples every ~5 waves for drift

See `docs/EXAMPLES-SYNC-PLAN.md` for full synchronization protocol.

---

## Quick Reference

**Want to see:**
- Weight-free AI? → `real_inference.rs`
- How to hit 2.5 ops/cycle? → `*_pattern.rs` examples
- Full cognitive loop? → `cognitive_demo.rs`
- 360° semantics? → `synchronicity_demo.rs`
- I Ching integration? → `ancient_wisdom.rs`
- SMT coordination? → `wedge_demo.rs`
- Authoritative benchmarks? → `src/bin/w*_*.rs`

---

**Last updated:** 2026-02-16 (post-W16, Phase 1 complete)  
**Current theory:** Phase 0 ✅ (3.0 ops/cycle), Phase 1 ✅ (1.89× SMT)
