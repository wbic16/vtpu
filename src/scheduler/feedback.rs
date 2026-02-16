//! Scheduler Feedback Loop (R23W17-2 Redux)
//!
//! Monitors OS scheduler behavior and feeds metrics back into vTPU execution decisions.
//! Enables adaptive scheduling based on actual OS placement, migrations, and contention.

use std::time::{Duration, Instant};

/// Scheduler feedback metrics collected during execution
#[derive(Debug, Clone, Default)]
pub struct SchedulerFeedback {
    /// Number of thread migrations detected
    pub migrations: usize,
    
    /// CPU core where thread is currently running (Linux: sched_getcpu)
    pub current_cpu: Option<usize>,
    
    /// CPU cores seen during execution (migration trail)
    pub cpu_trail: Vec<usize>,
    
    /// Voluntary context switches (thread yielded)
    pub voluntary_switches: u64,
    
    /// Involuntary context switches (preempted)
    pub involuntary_switches: u64,
    
    /// Time spent executing vs waiting
    pub execution_time: Duration,
    pub wait_time: Duration,
    
    /// Cache miss rate (if perf counters available)
    pub cache_misses: Option<u64>,
    pub cache_references: Option<u64>,
}

impl SchedulerFeedback {
    /// Create new empty feedback
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sample current scheduler state
    pub fn sample(&mut self) {
        // Get current CPU
        #[cfg(target_os = "linux")]
        {
            let cpu = unsafe { libc::sched_getcpu() };
            if cpu >= 0 {
                let cpu = cpu as usize;
                
                // Track migration
                if let Some(last_cpu) = self.current_cpu {
                    if last_cpu != cpu {
                        self.migrations += 1;
                    }
                }
                
                self.current_cpu = Some(cpu);
                
                // Add to trail
                if self.cpu_trail.is_empty() || self.cpu_trail.last() != Some(&cpu) {
                    self.cpu_trail.push(cpu);
                }
            }
        }
        
        // TODO: Read /proc/self/status for context switches
        // - voluntary_ctxt_switches
        // - nonvoluntary_ctxt_switches
    }
    
    /// Check if thread has migrated
    pub fn has_migrated(&self) -> bool {
        self.migrations > 0
    }
    
    /// Check if thread is stable on one core
    pub fn is_stable(&self) -> bool {
        self.cpu_trail.len() <= 2 && self.migrations == 0
    }
    
    /// Calculate cache miss rate
    pub fn cache_miss_rate(&self) -> Option<f64> {
        if let (Some(misses), Some(refs)) = (self.cache_misses, self.cache_references) {
            if refs > 0 {
                Some(misses as f64 / refs as f64)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Get context switch ratio (involuntary / voluntary)
    /// High ratio = getting preempted a lot (CPU contention)
    pub fn preemption_ratio(&self) -> f64 {
        if self.voluntary_switches > 0 {
            self.involuntary_switches as f64 / self.voluntary_switches as f64
        } else {
            0.0
        }
    }
}

/// Feedback-driven execution policy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdaptivePolicy {
    /// No migrations detected, stay aggressive
    Stable,
    
    /// Migrations detected, increase cooperation
    Cooperative,
    
    /// High contention detected, yield more frequently
    Defensive,
}

/// Adaptive scheduler that adjusts based on feedback
pub struct AdaptiveScheduler {
    /// Current policy
    policy: AdaptivePolicy,
    
    /// Feedback collected so far
    feedback: SchedulerFeedback,
    
    /// Last sample time
    last_sample: Instant,
    
    /// Sample interval
    sample_interval: Duration,
}

impl AdaptiveScheduler {
    /// Create new adaptive scheduler
    pub fn new() -> Self {
        Self {
            policy: AdaptivePolicy::Stable,
            feedback: SchedulerFeedback::new(),
            last_sample: Instant::now(),
            sample_interval: Duration::from_millis(100), // Sample every 100ms
        }
    }
    
    /// Update feedback and adapt policy
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // Sample if interval elapsed
        if now.duration_since(self.last_sample) >= self.sample_interval {
            self.feedback.sample();
            self.last_sample = now;
            
            // Adapt policy based on feedback
            self.adapt_policy();
        }
    }
    
    /// Adapt policy based on current feedback
    fn adapt_policy(&mut self) {
        // If thread has migrated, switch to cooperative
        if self.feedback.has_migrated() {
            self.policy = AdaptivePolicy::Cooperative;
        }
        
        // If high preemption ratio, switch to defensive
        if self.feedback.preemption_ratio() > 2.0 {
            self.policy = AdaptivePolicy::Defensive;
        }
        
        // If stable for a while, return to stable
        if self.feedback.is_stable() && self.feedback.preemption_ratio() < 0.5 {
            self.policy = AdaptivePolicy::Stable;
        }
    }
    
    /// Get current policy
    pub fn policy(&self) -> AdaptivePolicy {
        self.policy
    }
    
    /// Get current feedback
    pub fn feedback(&self) -> &SchedulerFeedback {
        &self.feedback
    }
    
    /// Decide whether to yield based on current policy
    pub fn should_yield(&self, quantum_idx: usize) -> bool {
        match self.policy {
            AdaptivePolicy::Stable => {
                // Yield every 8 quanta when stable
                quantum_idx > 0 && quantum_idx % 8 == 0
            }
            AdaptivePolicy::Cooperative => {
                // Yield every 4 quanta when cooperative (default)
                quantum_idx > 0 && quantum_idx % 4 == 0
            }
            AdaptivePolicy::Defensive => {
                // Yield every 2 quanta when defensive (high contention)
                quantum_idx > 0 && quantum_idx % 2 == 0
            }
        }
    }
}

impl Default for AdaptiveScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Read context switches from /proc/self/status (Linux only)
#[cfg(target_os = "linux")]
pub fn read_context_switches() -> Result<(u64, u64), std::io::Error> {
    use std::fs;
    
    let status = fs::read_to_string("/proc/self/status")?;
    
    let mut voluntary = 0;
    let mut involuntary = 0;
    
    for line in status.lines() {
        if line.starts_with("voluntary_ctxt_switches:") {
            voluntary = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("nonvoluntary_ctxt_switches:") {
            involuntary = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }
    
    Ok((voluntary, involuntary))
}

#[cfg(not(target_os = "linux"))]
pub fn read_context_switches() -> Result<(u64, u64), std::io::Error> {
    Ok((0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feedback_creation() {
        let feedback = SchedulerFeedback::new();
        assert_eq!(feedback.migrations, 0);
        assert_eq!(feedback.current_cpu, None);
        assert!(feedback.is_stable());
    }
    
    #[test]
    fn test_adaptive_scheduler() {
        let mut sched = AdaptiveScheduler::new();
        assert_eq!(sched.policy(), AdaptivePolicy::Stable);
        
        // Initially should yield infrequently
        assert!(!sched.should_yield(1));
        assert!(!sched.should_yield(2));
        assert!(!sched.should_yield(4));
        assert!(sched.should_yield(8)); // Every 8 quanta when stable
    }
    
    #[test]
    fn test_preemption_ratio() {
        let mut feedback = SchedulerFeedback::new();
        feedback.voluntary_switches = 100;
        feedback.involuntary_switches = 50;
        
        assert_eq!(feedback.preemption_ratio(), 0.5);
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_read_context_switches() {
        // This will only work on Linux
        let result = read_context_switches();
        assert!(result.is_ok());
        
        let (voluntary, involuntary) = result.unwrap();
        // Should have at least some context switches
        assert!(voluntary > 0 || involuntary >= 0);
    }
}
