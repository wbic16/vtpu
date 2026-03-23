# R23 Wave 1 Onboarding - vTPU Specification

**Status**: ✅ Complete (Feb 14, 2026)  
**Duration**: ~3 hours  
**Contributor**: Cyon

## What We Built

Wave 1 delivered the complete vTPU (Virtual Tensor Processing Unit) specification - the architectural foundation for all 40 waves.

### Core Deliverables

1. **vTPU Spec v0.1** (51.8 KB)
   - Location: `/source/exo-plan/whitepapers/vtpu-spec-v0.1.md`
   - What: Complete architecture specification
   - Key sections: 3-pipe execution model, Sentron ISA, memory hierarchy, cluster config

2. **Geometric Extensions** (19.5 KB)
   - Location: `/source/exo-plan/whitepapers/vtpu-geometric-extensions.md`
   - What: How phext's 11D structure accelerates geometric operations
   - Key insight: O(N^k) operations in 2D/3D become O(1) or O(log N) in phext

3. **KPI Framework** (11.1 KB)
   - Location: `/source/exo-plan/whitepapers/vtpu-kpis-and-roadmap.md`
   - What: 12 measurable success metrics + 7-phase roadmap
   - Target metrics: 3 ops/cycle, 95% L1 hit rate, 75 tok/s Qwen3, etc.

4. **Dashboard** (7.2 KB)
   - Location: `/source/exo-plan/rallies/R23-DASHBOARD.md`
   - What: 40-wave tracking document with phase/status/timeline

5. **Summary Report** (10 KB)
   - Location: `/source/exo-plan/whitepapers/R23-WAVE1-COMPLETE.md`
   - What: Completion report with scope evolution notes

## How to Navigate

### Quick Start (5 minutes)
```bash
# Read the executive summary
head -100 /source/exo-plan/whitepapers/vtpu-spec-v0.1.md

# Check current progress
cat /source/exo-plan/rallies/R23-DASHBOARD.md | grep "Status:"
```

### Deep Dive (30-60 minutes)

**Read in this order:**

1. **Dashboard first** - Get the big picture
   ```bash
   less /source/exo-plan/rallies/R23-DASHBOARD.md
   ```

2. **vTPU Spec** - Core architecture
   ```bash
   less /source/exo-plan/whitepapers/vtpu-spec-v0.1.md
   ```
   - Start with sections 1-3 (Overview, Architecture, ISA)
   - Skip to section 6 (Performance Targets) for the punchline
   - Circle back to sections 4-5 (Memory, Cluster) as needed

3. **KPI Framework** - Success metrics
   ```bash
   less /source/exo-plan/whitepapers/vtpu-kpis-and-roadmap.md
   ```
   - Focus on the 12 KPIs table
   - Read Phase 0 roadmap (Waves 1-10)

4. **Geometric Extensions** (optional, advanced)
   ```bash
   less /source/exo-plan/whitepapers/vtpu-geometric-extensions.md
   ```
   - Read if you want to understand WHY phext accelerates graph/tensor ops

## Key Architecture Points

### 3-Pipe Execution Model
- **D-Pipe**: Dense operations (ALU, standard computation)
- **S-Pipe**: Sparse operations (memory-intensive, phext navigation)
- **C-Pipe**: Coordination (messages, synchronization)
- **Target**: 3 operations/cycle sustained (vs 1.2 baseline)

### Sentron ISA
- 27 base instructions (SIW = Sentron Instruction Word)
- 3-wide VLIW-like issue (one instruction per pipe)
- Phext-native addressing (coordinates map directly to memory hierarchy)

### Memory Hierarchy → Phext Dimensions
- **0-2D**: L1 cache (32 KB per core)
- **0-4D**: L2 cache (512 KB per core)
- **0-7D**: L3 cache (16 MB shared)
- **0-9D**: DDR5 RAM (96 GB per node)
- **All 11D**: Cluster (Shell of Nine = 5 nodes, 480 GB total)

### Performance Targets
- **Cluster throughput**: 359 Gops/sec sustained (480 Gops/sec peak)
- **Cost efficiency**: 1/50th TPU v4 cost for cognitive workloads
- **Break-even**: 58 hours vs TPU v4 cloud pricing
- **Per-op cost**: $0.004/trillion ops (post break-even)

## How to Run (Wave 1)

**Nothing to run yet** - Wave 1 is pure specification/planning.

Runnable code starts in Wave 2 (baseline measurements) and continues through Waves 3-40.

## Timeline

- **Wave 1**: ✅ Complete (Feb 14, 2026)
- **Wave 2**: 🟡 In Progress (baseline measurements)
- **Waves 3-10**: Phase 0 - Foundation (Feb 15 - Mar 1, est.)
- **Waves 11-40**: Phases 1-6 - Implementation (Mar 2 - Jul 18)
- **Target completion**: July 18, 2026 (154 days from Wave 1)

## Questions?

- **What is vTPU?** Virtual Tensor Processing Unit - software-defined TPU on AMD R9 hardware
- **Why build this?** Close the 37.8% gap between TPU (57.8% peak utilization) and CPU (20% peak)
- **How does phext help?** 11D structure makes graph/tensor operations O(1) or O(log N) instead of O(N^k)
- **What's the end goal?** 40 vTPU cores (Shell of Nine) running cognitive workloads at 1/50th TPU v4 cost

## Next Wave Preview

**Wave 2: Baseline Measurements** (in progress)
- Compile and run cache benchmarks (sequential_access.c, random_access.c)
- Measure all 12 KPIs on current hardware/software
- Establish baseline for comparison as we build vTPU optimizations
- Deliverable: Baseline report with perf stats for all KPIs

---

*Created: 2026-02-14 21:02 CST*  
*Wave 1 Completion: 2026-02-14 ~19:00 CST*  
*Rally: R23 (40 waves, 154 days)*
