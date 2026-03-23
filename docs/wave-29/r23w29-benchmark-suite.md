# R23W29: Benchmark Suite & Cost Analysis

## Goal
Formalize performance measurement across the full vTPU stack.
Establish $/TFLOPS baseline and comparison against conventional approaches.

## Deliverables
1. **Unified benchmark binary** (`src/bin/bench_w29.rs`)
   - Single-sentron throughput (D/S/C pipes individually)
   - 40-sentron mote throughput (canonical node)
   - 200-sentron fleet throughput (5-node cluster sim)
   - Memory footprint validation (vs W28 estimates)
   - SIMD speedup measurement (with/without AVX2)

2. **Cost model** (`src/cost.rs`)
   - ops/cycle, ops/watt, ops/dollar metrics
   - RAM per sentron (measured, not estimated)
   - SIW throughput at current interpreter speed
   - Projected JIT improvement factor

3. **Comparison table** (in docs)
   - vTPU vs PyTorch Geometric (graph ops)
   - vTPU vs NetworkX (topology)
   - vTPU vs raw SIMD (compute density)

## Success Criteria
- `cargo run --bin bench_w29 --release` produces full report
- Measured sentron size matches W28 estimate (±10%)
- Cost model covers all 3 pipe types
- Tests: target 460+
