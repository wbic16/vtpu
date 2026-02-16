# R23W17: CPU Scheduler Coordination
## Working With the OS, Not Against It

**Wave:** R23W17  
**Date:** 2026-02-16  
**Phase:** 2 (Multi-Core Scaling)  
**Focus:** OS scheduler integration for optimal thread placement

---

## Context: W16 Lesson Learned

**W16 discovery:** Threading without CPU affinity gave 0.09× speedup (not 1.8×)

**Why:**
- OS scheduler placed threads on different physical cores
- OR time-sliced threads on same core (sequential, not parallel)
- Threading overhead dominated with no actual parallelism

**Workaround:** SmtPair model (simulated overlap) achieved 1.89× ✅

**W17 goal:** Get real OS scheduler cooperation for actual parallel execution

---

## The Problem

### Current State (No Scheduler Coordination)

```rust
thread::spawn(|| {
    // D-Pipe heavy workload
    // OS decides where this runs (arbitrary core)
});

thread::spawn(|| {
    // S-Pipe heavy workload
    // OS decides where this runs (arbitrary core)
});
```

**OS scheduler behavior (without hints):**
- May place threads on different cores → cache thrashing
- May place threads on same core → sequential execution (no SMT benefit)
- May migrate threads during execution → lost locality
- No awareness of workload complementarity (D-heavy vs S-heavy)

**Result:** Unpredictable performance (W16 showed 0.09× instead of 1.8×)

### Desired State (Scheduler Coordination)

```rust
// Thread A: D-Pipe heavy
pin_to_cpu(0);  // Physical core 0, thread 0
set_thread_name("vtpu_d_pipe_0");
hint_scheduler(ThreadHint::Compute);

// Thread B: S-Pipe heavy
pin_to_cpu(1);  // Physical core 0, SMT sibling (thread 1)
set_thread_name("vtpu_s_pipe_0");
hint_scheduler(ThreadHint::Memory);
```

**OS scheduler benefits:**
- Threads stay on SMT siblings (share L1/L2 cache)
- No migration → stable performance
- Complementary workloads → minimal port conflicts
- Named threads → easier profiling/debugging

---

## CPU Topology (Zen 4 Example)

### Physical Layout

```
8 cores × 2 SMT threads = 16 hardware threads

Core 0: CPU 0 (physical), CPU 1 (SMT sibling)
Core 1: CPU 2 (physical), CPU 3 (SMT sibling)
Core 2: CPU 4 (physical), CPU 5 (SMT sibling)
Core 3: CPU 6 (physical), CPU 7 (SMT sibling)
Core 4: CPU 8 (physical), CPU 9 (SMT sibling)
Core 5: CPU 10 (physical), CPU 11 (SMT sibling)
Core 6: CPU 12 (physical), CPU 13 (SMT sibling)
Core 7: CPU 14 (physical), CPU 15 (SMT sibling)
```

### Cache Hierarchy

```
L1 Data:  32KB per core (shared by SMT siblings)
L1 Inst:  32KB per core (shared by SMT siblings)
L2:       1MB per core (shared by SMT siblings)
L3:       32MB shared across all cores (8-16MB per CCX)
```

**Key insight:** SMT siblings share L1/L2 → zero-copy handoff

### NUMA Layout (if applicable)

```
Node 0: Cores 0-3 (CCX0)
Node 1: Cores 4-7 (CCX1)

Memory attached to each node
Cross-node access = 1.5-2× latency
```

**Strategy:** Pin memory + threads to same NUMA node

---

## W17 Goals

### Goal 1: CPU Affinity (Thread Pinning)
**What:** Bind threads to specific CPUs

**Why:**
- Prevent migration (stable cache state)
- Enable SMT pairing (siblings on same core)
- Predictable performance

**Implementation:**
```rust
// Linux: sched_setaffinity
// macOS: thread_policy_set
// Windows: SetThreadAffinityMask
```

### Goal 2: NUMA Awareness
**What:** Allocate memory + threads on same node

**Why:**
- Local memory = 1× latency
- Remote memory = 1.5-2× latency
- 30-50% performance difference

**Implementation:**
```rust
// Linux: numa_alloc_onnode, mbind
// macOS: No NUMA (single node)
// Windows: VirtualAllocExNuma
```

### Goal 3: Thread Naming
**What:** Set descriptive thread names

**Why:**
- Easier profiling (`perf top` shows names)
- Debugger clarity
- OS scheduler visibility

**Implementation:**
```rust
// Linux: pthread_setname_np
// macOS: pthread_setname_np
// Windows: SetThreadDescription
```

### Goal 4: Priority Hints (Optional)
**What:** Suggest scheduling priority

**Why:**
- Real-time for critical threads
- Background for logging/telemetry
- Fair for worker threads

**Implementation:**
```rust
// Linux: sched_setscheduler (SCHED_FIFO, SCHED_RR, SCHED_OTHER)
// macOS: thread_policy_set
// Windows: SetThreadPriority
```

---

## Implementation Strategy

### Phase 1: CPU Topology Discovery

**Discover:**
- How many physical cores?
- Which CPUs are SMT siblings?
- NUMA nodes (if any)?
- Cache sizes?

**Libraries:**
- `num_cpus` crate (basic)
- `hwloc` crate (comprehensive)
- OR parse `/proc/cpuinfo`, `/sys/devices/system/cpu/`

**Code:**
```rust
pub struct CpuTopology {
    pub physical_cores: usize,
    pub smt_threads: usize,
    pub siblings: Vec<(usize, usize)>,  // (physical, sibling) pairs
    pub numa_nodes: Vec<NumaNode>,
}

impl CpuTopology {
    pub fn discover() -> Self {
        // Parse /proc/cpuinfo or use hwloc
        // Return topology
    }
    
    pub fn smt_sibling(&self, cpu: usize) -> Option<usize> {
        // Given CPU N, return its SMT sibling
    }
}
```

### Phase 2: Thread Affinity API

**Cross-platform wrapper:**
```rust
#[cfg(target_os = "linux")]
pub fn pin_to_cpu(cpu_id: usize) -> Result<(), String> {
    use libc::{cpu_set_t, CPU_ZERO, CPU_SET, sched_setaffinity};
    
    unsafe {
        let mut cpu_set: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpu_set);
        CPU_SET(cpu_id, &mut cpu_set);
        
        if sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpu_set) != 0 {
            return Err(format!("Failed to pin to CPU {}", cpu_id));
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn pin_to_cpu(cpu_id: usize) -> Result<(), String> {
    // macOS doesn't support hard affinity, use thread_policy_set for hints
    Err("macOS does not support CPU pinning".to_string())
}

#[cfg(target_os = "windows")]
pub fn pin_to_cpu(cpu_id: usize) -> Result<(), String> {
    use windows::Win32::System::Threading::{GetCurrentThread, SetThreadAffinityMask};
    
    unsafe {
        let mask = 1u64 << cpu_id;
        if SetThreadAffinityMask(GetCurrentThread(), mask) == 0 {
            return Err(format!("Failed to pin to CPU {}", cpu_id));
        }
    }
    Ok(())
}
```

### Phase 3: NUMA-Aware Allocation

**Allocate memory on specific node:**
```rust
pub struct NumaMemory {
    ptr: *mut u8,
    size: usize,
    node: usize,
}

impl NumaMemory {
    #[cfg(target_os = "linux")]
    pub fn alloc_on_node(size: usize, node: usize) -> Self {
        use libc::{mmap, PROT_READ, PROT_WRITE, MAP_PRIVATE, MAP_ANONYMOUS};
        
        let ptr = unsafe {
            let ptr = mmap(
                std::ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            );
            
            // Bind to NUMA node
            libc::mbind(ptr, size, libc::MPOL_BIND, &(1u64 << node), 64, 0);
            
            ptr as *mut u8
        };
        
        NumaMemory { ptr, size, node }
    }
}
```

### Phase 4: Scheduler-Aware Executor

**SmtPair with real OS integration:**
```rust
pub struct SchedulerAwareSmtPair {
    topology: CpuTopology,
    core_id: usize,
    thread_a: Option<thread::JoinHandle<ExecStats>>,
    thread_b: Option<thread::JoinHandle<ExecStats>>,
}

impl SchedulerAwareSmtPair {
    pub fn new(core_id: usize, topology: CpuTopology) -> Self {
        SchedulerAwareSmtPair {
            topology,
            core_id,
            thread_a: None,
            thread_b: None,
        }
    }
    
    pub fn execute(&mut self, fwd_prog: Vec<SIW>, bwd_prog: Vec<SIW>, mem: Arc<Mutex<Memory>>) -> TrainStats {
        let (cpu_a, cpu_b) = self.topology.siblings[self.core_id];
        
        // Thread A: D-Pipe heavy, pinned to physical thread
        let mem_a = Arc::clone(&mem);
        let handle_a = thread::spawn(move || {
            pin_to_cpu(cpu_a).expect("Failed to pin thread A");
            set_thread_name("vtpu_d_pipe");
            
            let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
            sentron.spawn(fwd_prog);
            
            let mut mem = mem_a.lock().unwrap();
            exec::run(&mut sentron, &mut *mem)
        });
        
        // Thread B: S-Pipe heavy, pinned to SMT sibling
        let mem_b = Arc::clone(&mem);
        let handle_b = thread::spawn(move || {
            pin_to_cpu(cpu_b).expect("Failed to pin thread B");
            set_thread_name("vtpu_s_pipe");
            
            let mut sentron = Sentron::new(1, PhextCoord::zero(), 0, 0);
            sentron.spawn(bwd_prog);
            
            let mut mem = mem_b.lock().unwrap();
            exec::run(&mut sentron, &mut *mem)
        });
        
        // Wait for both threads
        let stats_a = handle_a.join().unwrap();
        let stats_b = handle_b.join().unwrap();
        
        // Build TrainStats from both
        TrainStats {
            forward_stats: stats_a,
            backward_stats: stats_b,
            loss: 0,
            total_ops: stats_a.ops_retired + stats_b.ops_retired,
            total_cycles: stats_a.cycles.max(stats_b.cycles),  // SMT overlap
            effective_ops_per_cycle: (stats_a.ops_retired + stats_b.ops_retired) as f64 / stats_a.cycles.max(stats_b.cycles) as f64,
        }
    }
}
```

---

## W17 Deliverables

### 1. `src/scheduler.rs` - OS Scheduler Integration
**Functions:**
- `CpuTopology::discover()` - Detect cores, siblings, NUMA
- `pin_to_cpu(cpu_id)` - Cross-platform affinity
- `set_thread_name(name)` - Cross-platform naming
- `alloc_numa(size, node)` - NUMA-aware allocation

### 2. `src/bin/w17_topology.rs` - Topology Discovery
**Purpose:** Print CPU layout for debugging

**Output:**
```
CPU Topology:
  Physical cores: 8
  SMT threads:    16
  SMT siblings:   [(0,1), (2,3), (4,5), (6,7), (8,9), (10,11), (12,13), (14,15)]
  NUMA nodes:     2
    Node 0: Cores 0-3
    Node 1: Cores 4-7
  Cache:
    L1d: 32 KB per core
    L2:  1 MB per core
    L3:  32 MB shared
```

### 3. `src/bin/w17_smt_pinned.rs` - Real SMT with Affinity
**Purpose:** W16 benchmark but with actual CPU pinning

**Expected:**
- Sequential: 99K cycles
- SMT (pinned): 52.5K cycles
- Speedup: 1.89× (same as W16 model, but real threads)

### 4. `src/bin/w17_numa_benchmark.rs` - NUMA Impact
**Purpose:** Measure local vs remote memory access

**Expected:**
- Local access: 1.0× latency
- Remote access: 1.5-2.0× latency
- Recommendation: Pin threads + memory to same node

---

## Success Criteria

### Must Have (W17 Gate)
1. ✅ CPU topology discovery working
2. ✅ Thread pinning (affinity) working on Linux
3. ✅ Real SMT benchmark ≥1.8× speedup (not simulated)
4. ✅ Thread naming for profiling

### Should Have
5. ✅ NUMA detection
6. ✅ NUMA-aware allocation
7. ✅ Cross-platform (Linux + macOS stubs + Windows stubs)

### Nice to Have
8. 🟡 Priority hints (SCHED_FIFO for real-time)
9. 🟡 CPU isolation (isolcpus for dedicated cores)
10. 🟡 IRQ affinity (pin interrupts away from vTPU cores)

---

## Timeline (3 days)

### Day 1: Topology + Affinity
- Implement `CpuTopology::discover()`
- Implement `pin_to_cpu()` for Linux
- Create `w17_topology.rs` to validate
- Test on logos-prime (8-core Zen 4)

### Day 2: Real SMT Benchmark
- Port W16 benchmark to use real pinning
- Compare simulated (1.89×) vs real (target: 1.8-2.0×)
- Debug if real < simulated (check topology detection)

### Day 3: NUMA + Documentation
- Implement NUMA detection
- Benchmark local vs remote memory
- Document findings in R23W17-COMPLETE.md
- Update Phase 2 roadmap

---

## Dependencies

### Crates to Add (Cargo.toml)
```toml
[dependencies]
libc = "0.2"           # For sched_setaffinity, mbind
num_cpus = "1.16"      # Basic CPU count
hwloc2 = { version = "2.0", optional = true }  # Comprehensive topology (optional)

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = ["Win32_System_Threading"] }
```

### Platform Support
- **Linux:** Full support (affinity, NUMA, naming)
- **macOS:** Partial (naming works, affinity = best effort)
- **Windows:** Full support (different APIs)

---

## Testing Strategy

### Unit Tests
```rust
#[test]
fn topology_has_smt_siblings() {
    let topo = CpuTopology::discover();
    assert!(topo.smt_threads >= topo.physical_cores);
    assert_eq!(topo.siblings.len(), topo.physical_cores);
}

#[test]
fn pin_to_cpu_succeeds() {
    let result = pin_to_cpu(0);
    #[cfg(target_os = "linux")]
    assert!(result.is_ok());
    
    #[cfg(target_os = "macos")]
    assert!(result.is_err());  // macOS doesn't support hard pinning
}
```

### Integration Tests
- Run `w17_smt_pinned.rs` on 1-core, 2-core, 4-core, 8-core
- Validate speedup scales linearly
- Check `perf top` shows threads on correct CPUs

---

## Risks & Mitigations

### Risk 1: Insufficient Permissions
**Problem:** CPU pinning requires CAP_SYS_NICE on Linux

**Mitigation:**
- Document: "Run with sudo or add CAP_SYS_NICE capability"
- Graceful fallback: warn if pinning fails, continue without

### Risk 2: Platform Differences
**Problem:** macOS doesn't support hard CPU affinity

**Mitigation:**
- Detect platform at compile time
- Provide best-effort hints on macOS
- Full support on Linux (primary target)

### Risk 3: NUMA Complexity
**Problem:** Not all systems have NUMA

**Mitigation:**
- Make NUMA optional (detect, use if available)
- Single-node systems = no-op NUMA calls
- Document NUMA benefits for multi-socket systems

---

## Expected Results

**W16 (simulated SMT):** 1.89× speedup via model  
**W17 (real SMT with pinning):** 1.8-2.0× speedup measured

**If W17 < W16:**
- Debug: Are threads actually on SMT siblings?
- Check: `cat /proc/<pid>/task/*/status | grep Cpus_allowed_list`
- Profile: `perf stat -e cpu-migrations`

**If W17 > W16:**
- Great! Real threading + cache benefits exceed model
- Document why (shared L1/L2, zero-copy handoff)

---

## Documentation Updates

After W17:
- Update MEMORY.md with scheduler integration
- Add scheduler.rs module docs
- Document platform support matrix
- Add troubleshooting guide (permissions, NUMA, etc.)

---

## Bottom Line

**W16:** Proved SMT concept via simulation (1.89×)  
**W17:** Make it real via OS scheduler coordination

**Goal:** ≥1.8× speedup with actual threads on pinned SMT siblings

**Benefit:** Production-ready multi-threading (not just models)

---

**R23W17:** CPU Scheduler Coordination  
**Timeline:** 3 days  
**Target:** Real SMT ≥1.8× speedup via OS integration

Let's coordinate. 🔄⚙️
