//! vTPU Performance Telemetry
//!
//! Tracks real-time performance metrics:
//! - Ops/cycle (target: 3.0)
//! - Cache hit rates (L1/L2/L3)
//! - Energy efficiency (Mops/W)
//! - Uptime and error counts

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// vTPU Performance Telemetry
///
/// Thread-safe performance counters exposed via HTTP API.
pub struct VtpuTelemetry {
    /// Total operations retired (all 3 pipes)
    pub retired_ops: AtomicU64,
    
    /// Total CPU cycles executed
    pub cycles: AtomicU64,
    
    /// Cache hits [L1, L2, L3]
    pub cache_hits: [AtomicU64; 3],
    
    /// Cache misses [L1, L2, L3]
    pub cache_misses: [AtomicU64; 3],
    
    /// Energy consumed (microjoules, via RAPL counters)
    pub energy_microjoules: AtomicU64,
    
    /// Start time (for uptime calculation)
    start_time: Instant,
    
    /// Error count
    pub errors: AtomicU64,
}

impl VtpuTelemetry {
    /// Create new telemetry tracker
    pub fn new() -> Self {
        Self {
            retired_ops: AtomicU64::new(0),
            cycles: AtomicU64::new(0),
            cache_hits: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            cache_misses: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            energy_microjoules: AtomicU64::new(0),
            start_time: Instant::now(),
            errors: AtomicU64::new(0),
        }
    }
    
    /// Record operation retirement (called after each SIW execution)
    #[inline]
    pub fn record_ops(&self, count: u64) {
        self.retired_ops.fetch_add(count, Ordering::Relaxed);
    }
    
    /// Record CPU cycles
    #[inline]
    pub fn record_cycles(&self, count: u64) {
        self.cycles.fetch_add(count, Ordering::Relaxed);
    }
    
    /// Record cache hit
    #[inline]
    pub fn record_cache_hit(&self, level: usize) {
        if level < 3 {
            self.cache_hits[level].fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Record cache miss
    #[inline]
    pub fn record_cache_miss(&self, level: usize) {
        if level < 3 {
            self.cache_misses[level].fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Record energy consumption (microjoules)
    #[inline]
    pub fn record_energy(&self, microjoules: u64) {
        self.energy_microjoules.fetch_add(microjoules, Ordering::Relaxed);
    }
    
    /// Record error
    #[inline]
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get total retired operations
    pub fn total_ops(&self) -> u64 {
        self.retired_ops.load(Ordering::Relaxed)
    }
    
    /// Get total cycles
    pub fn total_cycles(&self) -> u64 {
        self.cycles.load(Ordering::Relaxed)
    }
    
    /// Calculate operations per cycle
    pub fn ops_per_cycle(&self) -> f64 {
        let ops = self.total_ops() as f64;
        let cycles = self.total_cycles() as f64;
        
        if cycles > 0.0 {
            ops / cycles
        } else {
            0.0
        }
    }
    
    /// Calculate cache hit rate for each level
    pub fn cache_hit_rates(&self) -> [f64; 3] {
        let mut rates = [0.0; 3];
        
        for i in 0..3 {
            let hits = self.cache_hits[i].load(Ordering::Relaxed) as f64;
            let misses = self.cache_misses[i].load(Ordering::Relaxed) as f64;
            let total = hits + misses;
            
            rates[i] = if total > 0.0 {
                hits / total
            } else {
                0.0
            };
        }
        
        rates
    }
    
    /// Calculate combined cache hit rate (L1+L2+L3)
    pub fn combined_cache_hit_rate(&self) -> f64 {
        let total_hits: u64 = self.cache_hits.iter()
            .map(|h| h.load(Ordering::Relaxed))
            .sum();
        let total_misses: u64 = self.cache_misses.iter()
            .map(|m| m.load(Ordering::Relaxed))
            .sum();
        let total = (total_hits + total_misses) as f64;
        
        if total > 0.0 {
            total_hits as f64 / total
        } else {
            0.0
        }
    }
    
    /// Calculate throughput (Gops/sec)
    pub fn throughput_gops(&self) -> f64 {
        let ops = self.retired_ops.load(Ordering::Relaxed) as f64;
        let uptime_secs = self.start_time.elapsed().as_secs_f64();
        
        if uptime_secs > 0.0 {
            (ops / 1e9) / uptime_secs
        } else {
            0.0
        }
    }
    
    /// Calculate current power draw (watts)
    pub fn power_watts(&self) -> f64 {
        let energy_joules = self.energy_microjoules.load(Ordering::Relaxed) as f64 / 1e6;
        let uptime_secs = self.start_time.elapsed().as_secs_f64();
        
        if uptime_secs > 0.0 {
            energy_joules / uptime_secs
        } else {
            0.0
        }
    }
    
    /// Calculate energy efficiency (Mops/W)
    pub fn mops_per_watt(&self) -> f64 {
        let mops = self.retired_ops.load(Ordering::Relaxed) as f64 / 1e6;
        let power = self.power_watts();
        
        if power > 0.0 {
            mops / power
        } else {
            0.0
        }
    }
    
    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
    
    /// Generate JSON telemetry report
    pub fn to_json(&self) -> String {
        let ops_per_cycle = self.ops_per_cycle();
        let hit_rates = self.cache_hit_rates();
        let combined_hit_rate = self.combined_cache_hit_rate();
        let throughput = self.throughput_gops();
        let power = self.power_watts();
        let efficiency = self.mops_per_watt();
        let uptime = self.uptime_secs();
        let errors = self.error_count();
        
        format!(r#"{{
  "ops_per_cycle": {:.2},
  "cache_hit_rate": {{
    "L1": {:.3},
    "L2": {:.3},
    "L3": {:.3},
    "combined": {:.3}
  }},
  "throughput_gops": {:.1},
  "power_watts": {:.1},
  "mops_per_watt": {:.1},
  "uptime_seconds": {},
  "errors": {}
}}"#,
            ops_per_cycle,
            hit_rates[0], hit_rates[1], hit_rates[2], combined_hit_rate,
            throughput,
            power,
            efficiency,
            uptime,
            errors
        )
    }
}

impl Default for VtpuTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ops_per_cycle() {
        let telemetry = VtpuTelemetry::new();
        
        telemetry.record_ops(300);
        telemetry.record_cycles(100);
        
        assert_eq!(telemetry.ops_per_cycle(), 3.0);
    }
    
    #[test]
    fn test_cache_hit_rate() {
        let telemetry = VtpuTelemetry::new();
        
        // L1: 90 hits, 10 misses = 90% hit rate
        for _ in 0..90 {
            telemetry.record_cache_hit(0);
        }
        for _ in 0..10 {
            telemetry.record_cache_miss(0);
        }
        
        let rates = telemetry.cache_hit_rates();
        assert!((rates[0] - 0.9).abs() < 0.01);
    }
    
    #[test]
    fn test_json_output() {
        let telemetry = VtpuTelemetry::new();
        
        telemetry.record_ops(300);
        telemetry.record_cycles(100);
        
        let json = telemetry.to_json();
        assert!(json.contains("\"ops_per_cycle\": 3.00"));
    }
}
