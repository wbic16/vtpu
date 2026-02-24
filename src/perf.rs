//! Hardware Performance Counter Integration
//!
//! Zero-dependency wrapper around Linux perf_event_open for measuring:
//! - CPU cycles
//! - Instructions retired
//! - Cache references/misses
//!
//! Uses raw syscalls (no external dependencies).

use std::io;

// Linux perf_event constants
const PERF_TYPE_HARDWARE: u32 = 0;
const PERF_COUNT_HW_CPU_CYCLES: u64 = 0;
const PERF_COUNT_HW_INSTRUCTIONS: u64 = 1;
const PERF_COUNT_HW_CACHE_REFERENCES: u64 = 2;
const PERF_COUNT_HW_CACHE_MISSES: u64 = 3;

const PERF_FLAG_FD_CLOEXEC: u64 = 8;

#[repr(C)]
struct PerfEventAttr {
    type_: u32,
    size: u32,
    config: u64,
    sample_period_or_freq: u64,
    sample_type: u64,
    read_format: u64,
    flags: u64,
    // Rest of struct zeroed
    _reserved: [u64; 8],
}

impl PerfEventAttr {
    fn new(type_: u32, config: u64) -> Self {
        Self {
            type_,
            size: std::mem::size_of::<PerfEventAttr>() as u32,
            config,
            sample_period_or_freq: 0,
            sample_type: 0,
            read_format: 0,
            flags: 1 << 0, // disabled = 1 (start disabled)
            _reserved: [0; 8],
        }
    }
}

/// Hardware performance counters
pub struct PerfCounters {
    cycles_fd: Option<i32>,
    instructions_fd: Option<i32>,
    cache_refs_fd: Option<i32>,
    cache_misses_fd: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub struct PerfMetrics {
    pub cycles: u64,
    pub instructions: u64,
    pub cache_references: u64,
    pub cache_misses: u64,
}

impl PerfMetrics {
    /// Instructions per cycle
    pub fn ipc(&self) -> f64 {
        if self.cycles == 0 {
            0.0
        } else {
            self.instructions as f64 / self.cycles as f64
        }
    }
    
    /// Cache hit rate (0.0 - 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_references == 0 {
            0.0
        } else {
            1.0 - (self.cache_misses as f64 / self.cache_references as f64)
        }
    }
    
    /// Cache miss rate (0.0 - 1.0)
    pub fn cache_miss_rate(&self) -> f64 {
        1.0 - self.cache_hit_rate()
    }
}

impl PerfCounters {
    /// Create new performance counter set
    ///
    /// Opens file descriptors for:
    /// - CPU cycles
    /// - Instructions retired
    /// - Cache references
    /// - Cache misses
    ///
    /// Counters start disabled. Call `start()` to begin counting.
    pub fn new() -> io::Result<Self> {
        let cycles_fd = open_counter(PERF_TYPE_HARDWARE, PERF_COUNT_HW_CPU_CYCLES)?;
        let instructions_fd = open_counter(PERF_TYPE_HARDWARE, PERF_COUNT_HW_INSTRUCTIONS)?;
        let cache_refs_fd = open_counter(PERF_TYPE_HARDWARE, PERF_COUNT_HW_CACHE_REFERENCES)?;
        let cache_misses_fd = open_counter(PERF_TYPE_HARDWARE, PERF_COUNT_HW_CACHE_MISSES)?;
        
        Ok(Self {
            cycles_fd: Some(cycles_fd),
            instructions_fd: Some(instructions_fd),
            cache_refs_fd: Some(cache_refs_fd),
            cache_misses_fd: Some(cache_misses_fd),
        })
    }
    
    /// Start counting
    pub fn start(&self) -> io::Result<()> {
        if let Some(fd) = self.cycles_fd {
            enable_counter(fd)?;
        }
        if let Some(fd) = self.instructions_fd {
            enable_counter(fd)?;
        }
        if let Some(fd) = self.cache_refs_fd {
            enable_counter(fd)?;
        }
        if let Some(fd) = self.cache_misses_fd {
            enable_counter(fd)?;
        }
        Ok(())
    }
    
    /// Stop counting
    pub fn stop(&self) -> io::Result<()> {
        if let Some(fd) = self.cycles_fd {
            disable_counter(fd)?;
        }
        if let Some(fd) = self.instructions_fd {
            disable_counter(fd)?;
        }
        if let Some(fd) = self.cache_refs_fd {
            disable_counter(fd)?;
        }
        if let Some(fd) = self.cache_misses_fd {
            disable_counter(fd)?;
        }
        Ok(())
    }
    
    /// Read current counter values
    pub fn read(&self) -> io::Result<PerfMetrics> {
        let cycles = if let Some(fd) = self.cycles_fd {
            read_counter(fd)?
        } else {
            0
        };
        
        let instructions = if let Some(fd) = self.instructions_fd {
            read_counter(fd)?
        } else {
            0
        };
        
        let cache_references = if let Some(fd) = self.cache_refs_fd {
            read_counter(fd)?
        } else {
            0
        };
        
        let cache_misses = if let Some(fd) = self.cache_misses_fd {
            read_counter(fd)?
        } else {
            0
        };
        
        Ok(PerfMetrics {
            cycles,
            instructions,
            cache_references,
            cache_misses,
        })
    }
    
    /// Reset counters to zero
    pub fn reset(&self) -> io::Result<()> {
        if let Some(fd) = self.cycles_fd {
            reset_counter(fd)?;
        }
        if let Some(fd) = self.instructions_fd {
            reset_counter(fd)?;
        }
        if let Some(fd) = self.cache_refs_fd {
            reset_counter(fd)?;
        }
        if let Some(fd) = self.cache_misses_fd {
            reset_counter(fd)?;
        }
        Ok(())
    }
}

impl Drop for PerfCounters {
    fn drop(&mut self) {
        // Close file descriptors using raw syscall (no libc dependency)
        const SYS_CLOSE: i64 = 3;
        
        if let Some(fd) = self.cycles_fd.take() {
            unsafe { syscall1(SYS_CLOSE, fd as i64) };
        }
        if let Some(fd) = self.instructions_fd.take() {
            unsafe { syscall1(SYS_CLOSE, fd as i64) };
        }
        if let Some(fd) = self.cache_refs_fd.take() {
            unsafe { syscall1(SYS_CLOSE, fd as i64) };
        }
        if let Some(fd) = self.cache_misses_fd.take() {
            unsafe { syscall1(SYS_CLOSE, fd as i64) };
        }
    }
}

// Raw syscall wrappers (zero dependencies)

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall1(n: i64, arg1: i64) -> i64 {
    let ret: i64;
    std::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") arg1,
        lateout("rax") ret,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall3(n: i64, arg1: i64, arg2: i64, arg3: i64) -> i64 {
    let ret: i64;
    std::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") ret,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall5(n: i64, arg1: i64, arg2: i64, arg3: i64, arg4: i64, arg5: i64) -> i64 {
    let ret: i64;
    std::arch::asm!(
        "syscall",
        in("rax") n,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        in("r8") arg5,
        lateout("rax") ret,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack)
    );
    ret
}

// Linux syscall numbers (x86_64)
const SYS_READ: i64 = 0;
const _SYS_CLOSE: i64 = 3;
const SYS_IOCTL: i64 = 16;
const SYS_PERF_EVENT_OPEN: i64 = 298;

// Helper functions

fn open_counter(type_: u32, config: u64) -> io::Result<i32> {
    let mut attr = PerfEventAttr::new(type_, config);
    
    let fd = unsafe {
        syscall5(
            SYS_PERF_EVENT_OPEN,
            &mut attr as *mut PerfEventAttr as i64,
            0,  // pid (0 = current process)
            -1, // cpu (-1 = any CPU)
            -1, // group_fd (-1 = no group)
            PERF_FLAG_FD_CLOEXEC as i64,
        )
    };
    
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd as i32)
    }
}

fn enable_counter(fd: i32) -> io::Result<()> {
    const PERF_EVENT_IOC_ENABLE: i64 = 0x2400 + 0;
    
    let ret = unsafe {
        syscall3(SYS_IOCTL, fd as i64, PERF_EVENT_IOC_ENABLE, 0)
    };
    
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn disable_counter(fd: i32) -> io::Result<()> {
    const PERF_EVENT_IOC_DISABLE: i64 = 0x2400 + 1;
    
    let ret = unsafe {
        syscall3(SYS_IOCTL, fd as i64, PERF_EVENT_IOC_DISABLE, 0)
    };
    
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn reset_counter(fd: i32) -> io::Result<()> {
    const PERF_EVENT_IOC_RESET: i64 = 0x2400 + 3;
    
    let ret = unsafe {
        syscall3(SYS_IOCTL, fd as i64, PERF_EVENT_IOC_RESET, 0)
    };
    
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn read_counter(fd: i32) -> io::Result<u64> {
    let mut value: u64 = 0;
    
    let ret = unsafe {
        syscall3(
            SYS_READ,
            fd as i64,
            &mut value as *mut u64 as i64,
            std::mem::size_of::<u64>() as i64,
        )
    };
    
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_perf_metrics_ipc() {
        let m = PerfMetrics { cycles: 1000, instructions: 2500, cache_references: 100, cache_misses: 5 };
        assert!((m.ipc() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn test_perf_metrics_zero_cycles() {
        let m = PerfMetrics { cycles: 0, instructions: 0, cache_references: 0, cache_misses: 0 };
        assert_eq!(m.ipc(), 0.0);
        assert_eq!(m.cache_hit_rate(), 0.0);
        assert_eq!(m.cache_miss_rate(), 1.0); // 1 - 0 = 1
    }

    #[test]
    fn test_perf_metrics_cache_rates() {
        let m = PerfMetrics { cycles: 100, instructions: 100, cache_references: 200, cache_misses: 50 };
        assert!((m.cache_hit_rate() - 0.75).abs() < 1e-9);
        assert!((m.cache_miss_rate() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_perf_metrics_perfect_cache() {
        let m = PerfMetrics { cycles: 100, instructions: 100, cache_references: 1000, cache_misses: 0 };
        assert_eq!(m.cache_hit_rate(), 1.0);
        assert_eq!(m.cache_miss_rate(), 0.0);
    }

    #[test]
    #[ignore] // Requires Linux + perf permissions
    fn test_perf_counters() {
        let perf = PerfCounters::new().expect("Failed to create perf counters");
        
        perf.start().expect("Failed to start counters");
        
        // Do some work
        let mut sum = 0u64;
        for i in 0..1000 {
            sum = sum.wrapping_add(i);
        }
        std::hint::black_box(sum);
        
        perf.stop().expect("Failed to stop counters");
        
        let metrics = perf.read().expect("Failed to read counters");
        
        assert!(metrics.cycles > 0, "Should have measured cycles");
        assert!(metrics.instructions > 0, "Should have measured instructions");
        assert!(metrics.ipc() > 0.0, "IPC should be positive");
    }
}
