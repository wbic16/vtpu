# vTPU Micro-Scheduler Design
## Wave 2 Deliverable - R23 Rally

---

## Goal

Achieve sustained 3-operation-per-cycle retirement on AMD Zen 4 by dispatching SIW operations (D/S/C-Pipe) to non-overlapping execution ports.

**Target:** 2.5+ ops/cycle in Phase 1 (PoC), 3.0 ops/cycle in Phase 2 (optimized).

---

## Zen 4 Execution Port Mapping

### Physical Resources

Zen 4 cores have **6 execution ports**:

```
Port 0: ALU (integer/FP)
Port 1: ALU (integer/FP)
Port 2: ALU (integer/vector) + Branch
Port 3: ALU (integer/vector) + Branch
Port 4: AGU (address generation) + Load
Port 5: AGU (address generation) + Store
```

Additionally:
- **2 FP/SIMD pipes** shared with ALU scheduling
- **Out-of-order window:** 320 entries (ROB)
- **Retirement width:** Up to 6 micro-ops per cycle

### vTPU 3-Pipe Mapping

We target **3 of 6 ports** to avoid resource conflicts:

```
D-Pipe (Dense)  → Port 0/1   (ALU - integer/FP operations)
S-Pipe (Sparse) → Port 4/5   (AGU + Load/Store)
C-Pipe (Coord)  → Port 2/3   (ALU - message packing, routing arithmetic)
```

**Key insight:** These three port groups operate **independently**. When we structure code so each cycle contains one D-op, one S-op, and one C-op, the hardware has zero resource conflicts.

---

## Scheduling Contract

The **compiler** (phextcc, to be built in Phase 4) guarantees:

1. **Every SIW is 3-wide** - One operation per pipe (or NOP)
2. **No intra-SIW dependencies** - D/S/C operations in the same SIW are independent
3. **Cross-SIW dependencies explicit** - Marked in `deps` field
4. **Phext coordinates pre-computed** - S-Pipe never stalls waiting for address computation
5. **Double-buffering** - C-Pipe sends use buffers filled by prior D-Pipe

The **runtime** (micro-scheduler) guarantees:

1. **Port assignment** - D→ALU, S→AGU, C→ALU2
2. **Dependency respect** - Stall if cross-SIW dependency not satisfied
3. **Register allocation** - Map sentron registers to physical registers
4. **Prefetch injection** - Issue S-Pipe prefetch hints based on phext coordinates

---

## Implementation Strategies

### Strategy 1: Software Dispatch (Phase 1 PoC)

**Approach:** Explicit loop that reads SIW stream and dispatches to inline assembly.

**Pros:**
- Simple to implement
- Easy to measure per-pipe retirement rates
- Validates the port mapping hypothesis

**Cons:**
- Loop overhead (~5-10 cycles per SIW)
- No true out-of-order scheduling
- Limited to ~2.5 ops/cycle due to dispatch overhead

**Code sketch:**
```rust
pub fn execute_siw_stream(siws: &[SIW], context: &mut SentronContext) {
    for siw in siws {
        // Dispatch D-Pipe (inline ASM targeting Port 0/1)
        execute_d_pipe(&siw.d_op, context);
        
        // Dispatch S-Pipe (inline ASM targeting Port 4/5)
        execute_s_pipe(&siw.s_op, &siw.phext_addr, context);
        
        // Dispatch C-Pipe (inline ASM targeting Port 2/3)
        execute_c_pipe(&siw.c_op, context);
    }
}
```

**Validation:** Use hardware performance counters (RDPMC) to verify:
- Port 0/1 utilization (D-Pipe)
- Port 4/5 utilization (S-Pipe)
- Port 2/3 utilization (C-Pipe)

### Strategy 2: LLVM Intrinsics (Phase 2 Optimized)

**Approach:** Lower SIW operations to LLVM IR with explicit port hints via `asm!` or intrinsics.

**Pros:**
- LLVM's out-of-order scheduler can interleave multiple SIWs
- Reduced dispatch overhead
- Better register allocation

**Cons:**
- More complex to implement
- Requires deep LLVM knowledge
- May need custom LLVM passes for phext-awareness

**Code sketch:**
```rust
// Use LLVM intrinsics to hint port assignment
#[inline(always)]
unsafe fn dispatch_siw(siw: &SIW, ctx: &mut SentronContext) {
    // D-Pipe: regular ALU ops (LLVM schedules to Port 0/1)
    let d_result = match siw.d_op {
        DenseOp::DADD(rd, rs1, rs2) => ctx.regs[rs1] + ctx.regs[rs2],
        // ... other D-ops
    };
    
    // S-Pipe: explicit load/store (LLVM schedules to Port 4/5)
    let s_result = match siw.s_op {
        SparseOp::SGATHER(rd, width) => {
            let addr = translate_phext_coord(&siw.phext_addr, ctx);
            std::ptr::read_volatile(addr)  // Explicit load
        },
        // ... other S-ops
    };
    
    // C-Pipe: integer ops on separate registers (LLVM schedules to Port 2/3)
    match siw.c_op {
        CoordOp::CPACK(rd, rs1, rs2, fmt) => {
            ctx.msg_regs[rd] = pack_message(ctx.regs[rs1], ctx.regs[rs2], fmt);
        },
        // ... other C-ops
    };
}
```

**Target:** 3.0 ops/cycle sustained with proper LLVM scheduling.

### Strategy 3: JIT Compilation (Phase 4+)

**Approach:** JIT-compile SIW streams with phext-specific optimizations.

**Pros:**
- Runtime optimization based on actual phext access patterns
- Can insert prefetches dynamically
- Hot path optimization (trace compilation)

**Cons:**
- Most complex
- JIT overhead
- Requires Cranelift or custom JIT

**Future work:** Deferred to Phase 4 (after compiler is built).

---

## Phase 1 PoC: Minimal Viable Scheduler

**Goal:** Prove 2.5+ ops/cycle achievable on single core.

### Implementation Plan

1. **Create synthetic SIW stream** with known independent operations
2. **Implement Strategy 1** (software dispatch)
3. **Use RDPMC** to measure actual port utilization
4. **Validate assumptions:**
   - Are D/S/C operations truly independent?
   - Do they target separate ports?
   - What is actual retirement rate?

### Synthetic Benchmark

```rust
// Create 1000 SIWs: D-Pipe ADD, S-Pipe LOAD, C-Pipe PACK
fn create_synthetic_stream() -> Vec<SIW> {
    let mut siws = Vec::new();
    for i in 0..1000 {
        siws.push(SIW::new(
            DenseOp::DADD(0, 1, 2),              // r0 = r1 + r2
            SparseOp::SGATHER(3, 8),             // r3 = load 8 bytes from phext
            CoordOp::CPACK(0, 4, 5, 0),          // m0 = pack(r4, r5)
            PhextCoord::new([i as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0),
        ));
    }
    siws
}

// Execute and measure
fn benchmark_synthetic() {
    let siws = create_synthetic_stream();
    let mut context = SentronContext::new();
    
    let start = rdtsc();
    execute_siw_stream(&siws, &mut context);
    let end = rdtsc();
    
    let cycles = end - start;
    let ops = siws.len() * 3;  // 3 ops per SIW
    let ops_per_cycle = ops as f64 / cycles as f64;
    
    println!("Ops/cycle: {:.2}", ops_per_cycle);
}
```

**Success criteria:** ops_per_cycle ≥ 2.5

---

## Performance Counter Validation

Use RDPMC (Read Performance Monitoring Counter) to verify port utilization:

```rust
use std::arch::x86_64::_rdpmc;

// AMD Zen 4 PMC events
const PMC_PORT_0_OPS: u32 = 0x00; // Port 0 ops retired
const PMC_PORT_1_OPS: u32 = 0x01; // Port 1 ops retired
const PMC_PORT_4_OPS: u32 = 0x04; // Port 4 ops (AGU load)
const PMC_PORT_5_OPS: u32 = 0x05; // Port 5 ops (AGU store)

unsafe fn measure_port_utilization() {
    // Setup PMCs (requires root or perf_event_paranoid=0)
    setup_pmc(0, PMC_PORT_0_OPS);
    setup_pmc(1, PMC_PORT_4_OPS);
    
    let p0_before = _rdpmc(0);
    let p4_before = _rdpmc(1);
    
    // Execute SIW stream
    execute_siw_stream(&siws, &mut context);
    
    let p0_after = _rdpmc(0);
    let p4_after = _rdpmc(1);
    
    println!("Port 0 (D-Pipe): {}", p0_after - p0_before);
    println!("Port 4 (S-Pipe): {}", p4_after - p4_before);
}
```

**Expected result:** Port 0 and Port 4 should each show ~1 op/cycle, confirming independent execution.

---

## Dependency Tracking

SIWs may have cross-SIW dependencies (e.g., S-Pipe needs result from prior D-Pipe). The scheduler must respect these.

### Dependency Graph

```rust
pub struct DependencyGraph {
    /// SIW index → list of prior SIWs it depends on
    deps: Vec<Vec<usize>>,
}

impl DependencyGraph {
    pub fn from_siw_stream(siws: &[SIW]) -> Self {
        let mut deps = vec![vec![]; siws.len()];
        
        for (i, siw) in siws.iter().enumerate() {
            if siw.deps.has(DepFlags::CROSS_SIW) {
                // Heuristic: assume dependency on immediately prior SIW
                // (Real compiler would encode exact dependencies)
                if i > 0 {
                    deps[i].push(i - 1);
                }
            }
        }
        
        Self { deps }
    }
    
    pub fn can_execute(&self, idx: usize, completed: &[bool]) -> bool {
        self.deps[idx].iter().all(|&dep_idx| completed[dep_idx])
    }
}
```

### Dependency-Aware Scheduler

```rust
pub fn execute_with_deps(siws: &[SIW], context: &mut SentronContext) {
    let dep_graph = DependencyGraph::from_siw_stream(siws);
    let mut completed = vec![false; siws.len()];
    let mut ready_queue = vec![0]; // Start with SIW 0
    
    while !ready_queue.is_empty() {
        let idx = ready_queue.remove(0);
        
        // Execute SIW
        execute_single_siw(&siws[idx], context);
        completed[idx] = true;
        
        // Add dependents to ready queue
        for i in (idx + 1)..siws.len() {
            if !completed[i] && dep_graph.can_execute(i, &completed) {
                ready_queue.push(i);
            }
        }
    }
}
```

**Note:** In Tier 3 code, the compiler ensures dependencies are structured for maximum parallelism (double-buffer pattern), so this overhead is minimal.

---

## Phext Page Table (PPT) Integration

The S-Pipe needs to translate phext coordinates to physical addresses. This is handled by the **Phext Page Table** (implemented in Phase 2).

For now (Phase 1 PoC), we use a simple hash table:

```rust
pub struct SimplePhextTable {
    /// Map phext coordinate → physical address
    map: std::collections::HashMap<PhextCoord, *mut u8>,
}

impl SimplePhextTable {
    pub fn translate(&self, coord: &PhextCoord) -> *mut u8 {
        *self.map.get(coord).unwrap_or(&std::ptr::null_mut())
    }
}
```

**Phase 2 upgrade:** Replace with hierarchical trie + TLB-like cache.

---

## Next Steps (Wave 3)

1. Implement Strategy 1 (software dispatch) in `/source/vtpu/src/scheduler/mod.rs`
2. Create synthetic benchmark in `/source/vtpu/benchmarks/synthetic.rs`
3. Measure ops/cycle using RDTSC and RDPMC
4. Validate port utilization (confirm D→Port0/1, S→Port4/5, C→Port2/3)
5. Document results in `/source/vtpu/benchmarks/RESULTS.md`

**Success gate:** Achieve ≥2.5 ops/cycle on synthetic workload, proving the approach.

---

## Open Questions

1. **RDPMC access:** Requires root or `perf_event_paranoid=0`. How to handle in CI/testing?
2. **Prefetch strategy:** Should S-Pipe prefetch be explicit (PREFETCHT0) or rely on HW prefetcher?
3. **Register pressure:** With 16 general regs, 8 phext regs, 4 message regs (28 total), do we fit in Zen 4's physical register file?
4. **NUMA effects:** When scaling to cluster (Phase 3), how do we handle cross-node memory latency?

**To be resolved in subsequent waves.**

---

**Wave 2 Status:** Design complete. Ready for implementation in Wave 3.
