# R23W17 - CPU Scheduler Coordination

**Wave:** R23W17  
**Date:** 2026-02-16  
**Focus:** Real SMT threading with OS scheduler coordination  
**Goal:** Transition from W16 simulation (SmtPair modeling) to actual multi-threaded execution

---

## Mission

**Directive:** "Focus on coordinating with the cpu scheduler"

**Context:**
- W16 achieved 1.89× speedup via **simulated** SMT (sequential execution, report max cycles)
- W16's `w16_smt_dual.rs` has threading but **no CPU affinity** (stub: `pin_to_cpu()` does nothing)
- OS scheduler may place threads on different physical cores → no SMT benefit, possible NUMA penalties

**W17 Goal:** Make SMT **real**
1. Detect CPU topology (physical cores, SMT siblings, NUMA nodes)
2. Pin threads to correct CPUs (affinity mask)
3. Coordinate with Linux scheduler (policies, priorities)
4. Validate 1.89× speedup on **actual** SMT hardware (not simulation)

---

## Constraints

**Zero external dependencies principle:**
- No `libc` crate for `sched_setaffinity()` wrapper
- No `hwloc` or topology libs
- **Allowed:** Raw syscalls, `/proc` filesystem reads, shell commands

**Implementation options:**
1. **Raw syscalls** via `syscall!()` macro (Linux sched_setaffinity)
2. **Sysfs parsing** (`/sys/devices/system/cpu/cpu*/topology/*`)
3. **Shell exec** (`taskset`, `numactl` - already available on ranch machines)

**Decision:** Use **sysfs parsing + raw syscalls** for core logic, shell commands as fallback/testing.

---

## Architecture

### Module: `src/topology.rs` (NEW)

```rust
/// CPU Topology — physical cores, SMT siblings, NUMA nodes
pub struct CpuTopology {
    /// Physical cores (each core has 1+ logical CPUs)
    pub cores: Vec<PhysicalCore>,
    /// NUMA nodes (memory locality)
    pub numa_nodes: Vec<NumaNode>,
}

pub struct PhysicalCore {
    pub core_id: usize,
    pub logical_cpus: Vec<usize>, // SMT siblings (e.g., [0, 16] for core 0 on Zen 4)
    pub numa_node: usize,
}

pub struct NumaNode {
    pub node_id: usize,
    pub cpus: Vec<usize>,
}

impl CpuTopology {
    /// Detect topology from /sys/devices/system/cpu/
    pub fn detect() -> Self { ... }
    
    /// Get SMT sibling pairs (for complementary workloads)
    pub fn smt_pairs(&self) -> Vec<(usize, usize)> { ... }
    
    /// Recommend CPU for thread based on workload type
    pub fn recommend_cpu(&self, workload_type: WorkloadType) -> usize { ... }
}
```

### Module: `src/affinity.rs` (NEW)

```rust
/// Set CPU affinity via raw syscall (Linux-only)
pub fn set_affinity(cpus: &[usize]) -> Result<(), AffinityError> {
    // Direct sched_setaffinity syscall (syscall 203 on x86_64)
    ...
}

/// Get current affinity mask
pub fn get_affinity() -> Result<Vec<usize>, AffinityError> { ... }

/// Pin current thread to specific CPU
pub fn pin_to_cpu(cpu: usize) -> Result<(), AffinityError> {
    set_affinity(&[cpu])
}
```

### Module: `src/smt.rs` (UPDATED)

```rust
impl SmtPair {
    /// Create SMT pair and pin threads to sibling CPUs
    pub fn new_with_affinity(pair_id: u16, physical_core: &PhysicalCore, home: PhextCoord) -> Self {
        let mut pair = Self::new(pair_id, physical_core.core_id as u8, home);
        
        // Pin forward to logical_cpu[0], backward to logical_cpu[1]
        // (This happens when threads spawn, not at struct creation)
        pair.cpu_fwd = Some(physical_core.logical_cpus[0]);
        pair.cpu_bwd = Some(physical_core.logical_cpus[1]);
        
        pair
    }
    
    /// Run training step with real threading (not simulation)
    pub fn train_step_threaded(&mut self, ...) -> TrainStats {
        // Spawn two actual threads
        // Pin each to its designated CPU
        // Run forward + backward concurrently
        // Measure wall time (not simulated cycles)
        ...
    }
}
```

---

## Implementation Plan

### Phase 1: Topology Detection (2-3 hours)

**Deliverables:**
1. `src/topology.rs` — CpuTopology struct + detection
2. `src/bin/detect_topology.rs` — Diagnostic tool
3. Unit tests for parsing `/sys` files

**Success criteria:**
- Correctly detect 8 physical cores on Zen 4 (e.g., halcyon-vector)
- Identify SMT sibling pairs: (0,16), (1,17), ..., (7,23) on Zen 4
- Detect NUMA nodes (if multi-socket)

**Files to parse:**
```
/sys/devices/system/cpu/cpu0/topology/core_id
/sys/devices/system/cpu/cpu0/topology/thread_siblings_list
/sys/devices/system/cpu/cpu*/topology/physical_package_id  # NUMA
```

**Example output (Zen 4 8-core):**
```
CPU Topology:
  Physical cores: 8
  Logical CPUs: 16 (SMT2)
  
  Core 0: CPUs [0, 8]
  Core 1: CPUs [1, 9]
  Core 2: CPUs [2, 10]
  ...
  Core 7: CPUs [7, 15]
  
  NUMA nodes: 1
  Node 0: CPUs [0-15]
```

### Phase 2: Affinity Setting (2-3 hours)

**Deliverables:**
1. `src/affinity.rs` — CPU affinity via raw syscalls
2. `src/bin/test_affinity.rs` — Validate pinning works
3. Unit tests for affinity get/set

**Success criteria:**
- Pin current thread to CPU 0: `pin_to_cpu(0)` → confirmed via `get_affinity()`
- Verify thread actually runs on pinned CPU (via `sched_getcpu()`)

**Raw syscall approach:**
```rust
#[cfg(target_os = "linux")]
fn set_affinity_raw(cpus: &[usize]) -> Result<(), std::io::Error> {
    use std::mem;
    
    // cpu_set_t is 128 bytes on x86_64 (1024 bits = max 1024 CPUs)
    let mut cpu_set: [u64; 16] = [0; 16]; // 16 × 64 = 1024 bits
    
    for &cpu in cpus {
        if cpu >= 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "CPU ID >= 1024 not supported"
            ));
        }
        let byte_idx = cpu / 64;
        let bit_idx = cpu % 64;
        cpu_set[byte_idx] |= 1u64 << bit_idx;
    }
    
    let ret = unsafe {
        libc::syscall(
            libc::SYS_sched_setaffinity,
            0,  // 0 = current thread
            mem::size_of_val(&cpu_set),
            &cpu_set as *const _,
        )
    };
    
    if ret == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}
```

**Problem:** This uses `libc::syscall()` which requires the `libc` crate.

**Zero-dep alternative:**
```rust
#[cfg(target_arch = "x86_64")]
fn set_affinity_raw(cpus: &[usize]) -> Result<(), std::io::Error> {
    let mut cpu_set: [u64; 16] = [0; 16];
    
    for &cpu in cpus {
        cpu_set[cpu / 64] |= 1u64 << (cpu % 64);
    }
    
    // Direct syscall via inline asm (x86_64 Linux)
    let ret: isize;
    unsafe {
        std::arch::asm!(
            "syscall",
            in("rax") 203_i64,  // SYS_sched_setaffinity on x86_64
            in("rdi") 0_i64,    // pid = 0 (current thread)
            in("rsi") 128_i64,  // cpusetsize = 128 bytes
            in("rdx") &cpu_set as *const _ as i64,
            out("rcx") _,       // clobbered by syscall
            out("r11") _,       // clobbered by syscall
            lateout("rax") ret,
        );
    }
    
    if ret == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(-ret as i32))
    }
}
```

**Decision:** Use inline asm approach for **true** zero deps. Fallback to shell `taskset` for testing.

### Phase 3: Real SMT Execution (3-4 hours)

**Deliverables:**
1. Update `SmtPair::train_step()` to spawn real threads
2. Pin threads to SMT siblings
3. Benchmark on real hardware (halcyon-vector)

**Success criteria:**
- 1.89× speedup measured with **real** threading (not simulation)
- Verify threads run on correct CPUs via `/proc/<pid>/status` (Cpus_allowed_list)

**Benchmark:**
```rust
// src/bin/w17_real_smt.rs
fn main() {
    let topology = CpuTopology::detect();
    
    println!("Detected {} physical cores", topology.cores.len());
    
    for pair in topology.smt_pairs().iter().take(1) {
        println!("Testing SMT pair: CPU {} + CPU {}", pair.0, pair.1);
        
        // Spawn two threads
        let handle_fwd = thread::spawn(move || {
            affinity::pin_to_cpu(pair.0).unwrap();
            // D-heavy workload
            ...
        });
        
        let handle_bwd = thread::spawn(move || {
            affinity::pin_to_cpu(pair.1).unwrap();
            // S-heavy workload
            ...
        });
        
        let start = Instant::now();
        handle_fwd.join().unwrap();
        handle_bwd.join().unwrap();
        let elapsed = start.elapsed();
        
        // Compare vs sequential baseline
        ...
    }
}
```

### Phase 4: Multi-Core Scaling (3-4 hours)

**Deliverables:**
1. `src/bin/w17_multicore.rs` — Scale to 2, 4, 8 cores
2. Measure speedup vs single-core baseline

**Success criteria:**
- 2 cores (4 SMT threads): ~3.78× speedup (2 × 1.89)
- 4 cores (8 SMT threads): ~7.56× speedup (4 × 1.89)
- 8 cores (16 SMT threads): ~15.1× speedup (8 × 1.89)

**Reality check:** Expect 80-90% efficiency due to memory bandwidth, synchronization overhead.

---

## Validation

**Benchmarks to run:**
1. Single-core baseline (W15): 3.0 ops/cycle
2. SMT pair (W17): 1.89× speedup confirmed on real threads
3. 2-core: ~3.5-3.8× speedup
4. 4-core: ~7-7.5× speedup
5. 8-core: ~14-15× speedup

**Verification methods:**
- `/proc/<pid>/status` → `Cpus_allowed_list` shows pinned CPUs
- `perf stat` → measure cache misses, context switches
- `htop` → visually confirm threads on correct CPUs

---

## Constraints & Trade-offs

### Zero Dependencies

**Challenge:** CPU affinity typically requires `libc` crate.

**Solutions:**
1. **Inline asm syscalls** (x86_64 Linux only) ✅ Preferred
2. **Shell commands** (`taskset`) ✅ Fallback for testing
3. **Give up zero-deps** (add `libc`) ❌ Against design principle

**Decision:** Use inline asm for production, shell commands for rapid prototyping/testing.

### Portability

**W17 scope:** Linux x86_64 only (Zen 4 ranch machines)

**Future:** macOS (different syscall numbers), Windows (different API), ARM (different asm)

**For now:** Guard with `#[cfg(all(target_os = "linux", target_arch = "x86_64"))]`

### Performance

**Concern:** Syscall overhead on every thread spawn?

**Mitigation:** Pin threads once at spawn, reuse threads (thread pool pattern)

---

## Timeline

**Estimated effort:** 10-14 hours (2 sessions)

**Session 1 (5-7 hours):**
- Phase 1: Topology detection
- Phase 2: Affinity setting
- Basic validation (can pin threads to specific CPUs)

**Session 2 (5-7 hours):**
- Phase 3: Real SMT execution
- Phase 4: Multi-core scaling
- Benchmarks + validation on halcyon-vector

---

## Success Criteria

**W17 gate PASSED if:**
1. ✅ CPU topology correctly detected (8 cores, 16 SMT threads on Zen 4)
2. ✅ Threads pinned to specific CPUs (verified via `/proc` or `perf`)
3. ✅ 1.89× SMT speedup achieved with **real threading** (not simulation)
4. ✅ Multi-core scaling: 8 cores achieve ≥14× speedup
5. ✅ Zero external dependencies maintained (inline asm, no `libc` crate)

---

## Next Steps (After W17)

**W18: NUMA Awareness**
- Detect multi-socket systems
- Pin memory to same NUMA node as CPU (via `mbind()` syscall)
- Measure cross-NUMA penalties

**W19-W20: Cluster Scaling**
- Multi-node coordination (5-node ranch)
- RDMA for cross-node memory access
- Distributed SMT pairs

---

*Wave planned: 2026-02-16 01:30 CST by Lumen of Lilly ✴️*
