//! vTPU Scheduler — DAG reorder + OS feedback loop
//!
//! W9: DAG-based instruction scheduling (topological sort, NOP insertion)
//! W17: OS scheduler feedback — runtime adaptation based on measured performance

mod dag;
pub use dag::{Scheduler, ScheduleResult};

use crate::affinity;
use std::time::Instant;

/// Runtime feedback from the OS scheduler.
#[derive(Debug, Clone)]
pub struct SchedulerFeedback {
    pub current_cpu: usize,
    pub throughput: f64,
    pub migrated: bool,
    pub previous_cpu: usize,
    pub migrations: u64,
    pub sampled_at: Instant,
}

/// Tracks OS scheduler behavior and feeds it back into vTPU decisions.
pub struct SchedulerRedux {
    last_cpu: usize,
    migrations: u64,
    throughput_history: Vec<f64>,
    history_idx: usize,
    history_cap: usize,
    best_throughput: f64,
    best_cpu: usize,
    preferred_cpu: Option<usize>,
    should_pin: bool,
    samples: u64,
}

impl SchedulerRedux {
    pub fn new() -> Self {
        let current = current_cpu();
        SchedulerRedux {
            last_cpu: current,
            migrations: 0,
            throughput_history: vec![0.0; 16],
            history_idx: 0,
            history_cap: 16,
            best_throughput: 0.0,
            best_cpu: current,
            preferred_cpu: None,
            should_pin: false,
            samples: 0,
        }
    }

    pub fn sample(&mut self, throughput: f64) -> SchedulerFeedback {
        let cpu = current_cpu();
        let migrated = cpu != self.last_cpu;
        let previous = self.last_cpu;

        if migrated { self.migrations += 1; }

        self.throughput_history[self.history_idx] = throughput;
        self.history_idx = (self.history_idx + 1) % self.history_cap;
        self.samples += 1;

        if throughput > self.best_throughput {
            self.best_throughput = throughput;
            self.best_cpu = cpu;
        }

        if self.samples >= 8 {
            let avg = self.average_throughput();
            let migration_rate = self.migrations as f64 / self.samples as f64;
            if migration_rate > 0.3 && throughput < avg * 0.9 {
                self.preferred_cpu = Some(self.best_cpu);
                self.should_pin = true;
            } else if migration_rate < 0.05 {
                self.should_pin = false;
                self.preferred_cpu = None;
            }
        }

        self.last_cpu = cpu;

        SchedulerFeedback {
            current_cpu: cpu, throughput, migrated,
            previous_cpu: previous, migrations: self.migrations,
            sampled_at: Instant::now(),
        }
    }

    pub fn apply(&self) -> bool {
        if self.should_pin {
            if let Some(cpu) = self.preferred_cpu {
                return affinity::pin_thread(&affinity::CpuSet::single(cpu)).is_ok();
            }
        }
        false
    }

    pub fn average_throughput(&self) -> f64 {
        let count = self.samples.min(self.history_cap as u64) as f64;
        if count == 0.0 { return 0.0; }
        self.throughput_history.iter().sum::<f64>() / count
    }

    pub fn migration_rate(&self) -> f64 {
        if self.samples == 0 { return 0.0; }
        self.migrations as f64 / self.samples as f64
    }

    pub fn recommends_pinning(&self) -> bool { self.should_pin }
    pub fn best_cpu(&self) -> usize { self.best_cpu }
    pub fn total_samples(&self) -> u64 { self.samples }
    pub fn total_migrations(&self) -> u64 { self.migrations }
}

fn current_cpu() -> usize {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        let mut cpu: u32 = 0;
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") 309u64,
                in("rdi") &mut cpu as *mut u32,
                in("rsi") 0u64,
                in("rdx") 0u64,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        if ret == 0 { return cpu as usize; }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redux_new() {
        let redux = SchedulerRedux::new();
        assert_eq!(redux.total_samples(), 0);
        assert_eq!(redux.total_migrations(), 0);
    }

    #[test]
    fn redux_sample() {
        let mut redux = SchedulerRedux::new();
        let fb = redux.sample(1_000_000.0);
        assert_eq!(fb.migrations, 0);
        assert_eq!(redux.total_samples(), 1);
    }

    #[test]
    fn redux_tracks_best() {
        let mut redux = SchedulerRedux::new();
        redux.sample(100.0);
        redux.sample(200.0);
        redux.sample(150.0);
        assert_eq!(redux.best_throughput, 200.0);
    }

    #[test]
    fn redux_no_pin_when_stable() {
        let mut redux = SchedulerRedux::new();
        for _ in 0..10 { redux.sample(1_000_000.0); }
        assert!(!redux.recommends_pinning());
    }

    #[test]
    fn current_cpu_works() {
        let cpu = current_cpu();
        let logical = affinity::num_cpus();
        assert!(cpu < logical);
    }
}
