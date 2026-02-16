//! CPU Affinity — Zero-dep OS scheduler coordination
//!
//! Pin sentron pools to specific physical cores for deterministic SMT pairing.
//! Uses raw Linux syscalls (no libc dependency).
//!
//! W17: The vTPU doesn't fight the OS scheduler — it cooperates.
//! By pinning threads to cores, we guarantee SMT siblings share L1/L2
//! and complementary pipe workloads actually hit the same physical core.

/// CPU affinity mask — which cores a thread may run on.
#[derive(Debug, Clone)]
pub struct CpuSet {
    mask: u64, // supports up to 64 cores (enough for Zen 4)
}

impl CpuSet {
    /// Empty set — no cores allowed.
    pub fn empty() -> Self {
        CpuSet { mask: 0 }
    }

    /// All cores allowed.
    pub fn all() -> Self {
        CpuSet { mask: !0 }
    }

    /// Single core.
    pub fn single(core: usize) -> Self {
        assert!(core < 64, "Core ID must be < 64");
        CpuSet { mask: 1u64 << core }
    }

    /// SMT pair — two logical cores on the same physical core.
    /// On Zen 4: core N and core N+num_physical_cores are SMT siblings.
    pub fn smt_pair(physical_core: usize, num_physical_cores: usize) -> Self {
        let thread_0 = physical_core;
        let thread_1 = physical_core + num_physical_cores;
        assert!(thread_1 < 64);
        CpuSet { mask: (1u64 << thread_0) | (1u64 << thread_1) }
    }

    /// Add a core to the set.
    pub fn add(&mut self, core: usize) {
        assert!(core < 64);
        self.mask |= 1u64 << core;
    }

    /// Remove a core from the set.
    pub fn remove(&mut self, core: usize) {
        assert!(core < 64);
        self.mask &= !(1u64 << core);
    }

    /// Check if a core is in the set.
    pub fn contains(&self, core: usize) -> bool {
        core < 64 && (self.mask >> core) & 1 == 1
    }

    /// Number of cores in the set.
    pub fn count(&self) -> u32 {
        self.mask.count_ones()
    }

    /// Raw mask for syscall.
    pub fn as_raw(&self) -> u64 {
        self.mask
    }

    /// Range of physical cores (0..n) for an n-core machine.
    pub fn physical_range(cores: usize) -> Self {
        assert!(cores <= 64);
        CpuSet { mask: (1u64 << cores) - 1 }
    }
}

/// Pin current thread to the given CPU set.
///
/// Uses raw `sched_setaffinity` syscall (203 on x86_64 Linux).
/// Returns Ok(()) on success, Err with errno on failure.
///
/// # Safety
/// This is a raw syscall. Only works on Linux x86_64.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn pin_thread(set: &CpuSet) -> Result<(), i64> {
    let mask = set.as_raw();
    let ret: i64;
    unsafe {
        // sched_setaffinity(pid=0 means current thread, cpusetsize=8, mask_ptr)
        core::arch::asm!(
            "syscall",
            in("rax") 203u64,      // __NR_sched_setaffinity
            in("rdi") 0u64,        // pid = 0 (current thread)
            in("rsi") 8u64,        // cpusetsize = 8 bytes
            in("rdx") &mask as *const u64,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    if ret < 0 { Err(-ret) } else { Ok(()) }
}

/// Get current thread's CPU affinity.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn get_affinity() -> Result<CpuSet, i64> {
    let mut mask: u64 = 0;
    let ret: i64;
    unsafe {
        // sched_getaffinity(pid=0, cpusetsize=8, mask_ptr)
        core::arch::asm!(
            "syscall",
            in("rax") 204u64,      // __NR_sched_getaffinity
            in("rdi") 0u64,
            in("rsi") 8u64,
            in("rdx") &mut mask as *mut u64,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    if ret < 0 { Err(-ret) } else { Ok(CpuSet { mask }) }
}

/// Fallback for non-Linux: no-op.
#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub fn pin_thread(_set: &CpuSet) -> Result<(), i64> {
    Ok(()) // no-op on non-Linux
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub fn get_affinity() -> Result<CpuSet, i64> {
    Ok(CpuSet::all())
}

/// Detect number of logical CPUs via /proc/cpuinfo or sysconf.
/// Falls back to 1 if detection fails.
pub fn num_cpus() -> usize {
    // Try reading from /proc/cpuinfo (Linux)
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        let count = content.lines().filter(|l| l.starts_with("processor")).count();
        if count > 0 { return count; }
    }
    1
}

/// Detect number of physical cores (logical / 2 for SMT).
pub fn num_physical_cores() -> usize {
    let logical = num_cpus();
    // Check if SMT is enabled
    if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/smt/active") {
        if content.trim() == "1" {
            return logical / 2;
        }
    }
    logical
}

/// Zen 4 topology: map logical core → (physical_core, smt_thread)
pub fn zen4_topology(logical_core: usize) -> (usize, usize) {
    let physical = num_physical_cores();
    if logical_core < physical {
        (logical_core, 0) // first SMT thread
    } else {
        (logical_core - physical, 1) // second SMT thread
    }
}

/// Thread yield hint — cooperate with the OS scheduler.
/// On x86: PAUSE instruction (saves power, signals spinwait).
pub fn yield_hint() {
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::x86_64::_mm_pause(); }

    #[cfg(not(target_arch = "x86_64"))]
    std::thread::yield_now();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpuset_single() {
        let s = CpuSet::single(3);
        assert!(s.contains(3));
        assert!(!s.contains(0));
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn cpuset_smt_pair() {
        let s = CpuSet::smt_pair(2, 8); // physical core 2 on 8-core
        assert!(s.contains(2));   // thread 0
        assert!(s.contains(10));  // thread 1
        assert_eq!(s.count(), 2);
    }

    #[test]
    fn cpuset_range() {
        let s = CpuSet::physical_range(8);
        for i in 0..8 { assert!(s.contains(i)); }
        assert!(!s.contains(8));
        assert_eq!(s.count(), 8);
    }

    #[test]
    fn cpuset_add_remove() {
        let mut s = CpuSet::empty();
        s.add(5);
        assert!(s.contains(5));
        s.remove(5);
        assert!(!s.contains(5));
    }

    #[test]
    fn detect_cpus() {
        let n = num_cpus();
        assert!(n >= 1);
        let p = num_physical_cores();
        assert!(p >= 1);
        assert!(p <= n);
    }

    #[test]
    fn get_current_affinity() {
        let aff = get_affinity().unwrap();
        assert!(aff.count() >= 1);
    }

    #[test]
    fn pin_and_restore() {
        let original = get_affinity().unwrap();
        // Pin to core 0
        let _ = pin_thread(&CpuSet::single(0));
        let pinned = get_affinity().unwrap();
        assert!(pinned.contains(0));
        // Restore
        let _ = pin_thread(&original);
    }

    #[test]
    fn zen4_topo() {
        let (phys, thread) = zen4_topology(0);
        assert_eq!(phys, 0);
        assert_eq!(thread, 0);
    }

    #[test]
    fn yield_doesnt_crash() {
        yield_hint();
    }
}
