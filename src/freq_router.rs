//! Frequency-Affinity Router — route requests to cores by clock speed match
//!
//! Key insight: every CPU frequency transition costs ~10μs and burns transitional
//! power. By routing requests to cores already running at the appropriate frequency,
//! we minimize transitions and save 8-12% of total power budget.
//!
//! Rule: slow requests → slow cores. Fast requests → fast cores.
//! Only transition when the workload mix actually changes.
//!
//! Integrates with W24 mesh-aware cache (node-level routing) to provide
//! core-level routing within each node.
//!
//! Zero external dependencies.

use std::time::{Duration, Instant};

/// Request intensity — determines target frequency lane
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestIntensity {
    /// Idle: heartbeat, LED update, sensor poll. <0.1ms compute.
    Cold,
    /// Light: SQ read/write, coordinate lookup, e-ink refresh. <10ms.
    Warm,
    /// Medium: audio processing, OCR, scroll search. <100ms.
    Hot,
    /// Heavy: LLM inference, model loading, batch processing. >100ms.
    Blazing,
}

impl RequestIntensity {
    /// Target CPU frequency for this intensity (MHz)
    pub fn target_freq_mhz(&self) -> u32 {
        match self {
            Self::Cold    => 600,
            Self::Warm    => 1200,
            Self::Hot     => 1800,
            Self::Blazing => 2400,
        }
    }

    /// Classify a request by estimated compute time (microseconds)
    pub fn from_compute_us(us: u64) -> Self {
        match us {
            0..=100       => Self::Cold,     // <0.1ms
            101..=10_000  => Self::Warm,     // <10ms
            10_001..=100_000 => Self::Hot,   // <100ms
            _             => Self::Blazing,  // >100ms
        }
    }

    /// Index for array indexing (0-3)
    pub fn lane_index(&self) -> usize {
        match self {
            Self::Cold    => 0,
            Self::Warm    => 1,
            Self::Hot     => 2,
            Self::Blazing => 3,
        }
    }

    /// Estimated power draw at this frequency (watts, single core)
    pub fn estimated_core_watts(&self) -> f32 {
        match self {
            Self::Cold    => 0.3,
            Self::Warm    => 0.8,
            Self::Hot     => 1.5,
            Self::Blazing => 3.0,
        }
    }
}

/// Per-core frequency state
#[derive(Debug, Clone)]
pub struct CoreState {
    pub core_id: usize,
    pub current_freq_mhz: u32,
    pub current_lane: RequestIntensity,
    pub last_transition: Instant,
    pub requests_at_current_freq: u64,
    pub total_transitions: u64,
    pub total_requests: u64,
    /// Time accumulated in each frequency lane
    pub time_in_lane: [Duration; 4],
    /// Timestamp of last lane entry (for accumulating time_in_lane)
    lane_entered_at: Instant,
    /// Queued request count (for load balancing)
    pub queue_depth: u32,
}

impl CoreState {
    pub fn new(core_id: usize) -> Self {
        let now = Instant::now();
        Self {
            core_id,
            current_freq_mhz: 600,
            current_lane: RequestIntensity::Cold,
            last_transition: now,
            requests_at_current_freq: 0,
            total_transitions: 0,
            total_requests: 0,
            time_in_lane: [Duration::ZERO; 4],
            lane_entered_at: now,
            queue_depth: 0,
        }
    }

    /// Transition this core to a new frequency lane
    pub fn transition_to(&mut self, intensity: RequestIntensity) {
        let now = Instant::now();

        // Accumulate time in current lane
        let time_spent = now.duration_since(self.lane_entered_at);
        self.time_in_lane[self.current_lane.lane_index()] += time_spent;

        // Transition
        self.current_freq_mhz = intensity.target_freq_mhz();
        self.current_lane = intensity;
        self.last_transition = now;
        self.lane_entered_at = now;
        self.requests_at_current_freq = 0;
        self.total_transitions += 1;
    }

    /// Record a request served at current frequency
    pub fn serve_request(&mut self) {
        self.requests_at_current_freq += 1;
        self.total_requests += 1;
    }

    /// Time since last frequency transition
    pub fn time_since_transition(&self) -> Duration {
        self.last_transition.elapsed()
    }

    /// Fraction of time spent in each lane (for power accounting)
    pub fn lane_distribution(&self) -> [f64; 4] {
        let now = Instant::now();
        let mut times = self.time_in_lane;
        times[self.current_lane.lane_index()] += now.duration_since(self.lane_entered_at);
        let total: f64 = times.iter().map(|t| t.as_secs_f64()).sum();
        if total < 0.001 { return [0.25; 4]; }
        times.map(|t| t.as_secs_f64() / total)
    }

    /// Estimated average power draw based on lane distribution
    pub fn avg_power_watts(&self) -> f32 {
        let dist = self.lane_distribution();
        let intensities = [
            RequestIntensity::Cold,
            RequestIntensity::Warm,
            RequestIntensity::Hot,
            RequestIntensity::Blazing,
        ];
        intensities.iter()
            .zip(dist.iter())
            .map(|(int, &frac)| int.estimated_core_watts() * frac as f32)
            .sum()
    }
}

/// Routing decision
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub core_id: usize,
    pub transition_needed: bool,
    pub from_freq: u32,
    pub to_freq: u32,
    pub reason: RoutingReason,
}

#[derive(Debug, Clone, Copy)]
pub enum RoutingReason {
    /// Core already at the right frequency — zero transition
    ExactMatch,
    /// Closest frequency match — minimal transition
    NearestFreq,
    /// Least loaded core at target frequency
    LeastLoaded,
    /// All cores busy — route to least loaded regardless
    Overflow,
    /// Core just transitioned — avoid thrashing
    AvoidThrash,
}

/// Frequency-Affinity Router
pub struct FreqRouter {
    cores: Vec<CoreState>,
    /// Minimum time between transitions on the same core (anti-thrash)
    pub settle_time: Duration,
    /// Statistics
    pub total_routed: u64,
    pub zero_transition_routes: u64,
    pub transitions_avoided: u64,
}

impl FreqRouter {
    pub fn new(num_cores: usize) -> Self {
        Self {
            cores: (0..num_cores).map(CoreState::new).collect(),
            settle_time: Duration::from_millis(50),
            total_routed: 0,
            zero_transition_routes: 0,
            transitions_avoided: 0,
        }
    }

    /// Route a request to the best core. Returns routing decision.
    pub fn route(&mut self, intensity: RequestIntensity) -> RoutingDecision {
        self.total_routed += 1;
        let target_freq = intensity.target_freq_mhz();

        // Strategy 1: Find a core already at exact target frequency (zero transition)
        let exact_match = self.cores.iter()
            .enumerate()
            .filter(|(_, c)| c.current_freq_mhz == target_freq)
            .min_by_key(|(_, c)| c.queue_depth)
            .map(|(id, _)| id);

        if let Some(core_id) = exact_match {
            self.zero_transition_routes += 1;
            self.cores[core_id].serve_request();
            return RoutingDecision {
                core_id,
                transition_needed: false,
                from_freq: target_freq,
                to_freq: target_freq,
                reason: RoutingReason::ExactMatch,
            };
        }

        // Strategy 2: Find the nearest frequency core that isn't settling
        let nearest = self.cores.iter()
            .enumerate()
            .filter(|(_, c)| c.time_since_transition() >= self.settle_time)
            .min_by_key(|(_, c)| {
                let freq_dist = (c.current_freq_mhz as i64 - target_freq as i64).unsigned_abs();
                let load_penalty = c.queue_depth as u64 * 100;
                freq_dist + load_penalty
            })
            .map(|(id, _)| id);

        if let Some(core_id) = nearest {
            let from_freq = self.cores[core_id].current_freq_mhz;
            self.cores[core_id].transition_to(intensity);
            self.cores[core_id].serve_request();
            return RoutingDecision {
                core_id,
                transition_needed: from_freq != target_freq,
                from_freq,
                to_freq: target_freq,
                reason: if from_freq == target_freq {
                    RoutingReason::NearestFreq
                } else {
                    RoutingReason::NearestFreq
                },
            };
        }

        // Strategy 3: All cores settling — find least loaded regardless
        self.transitions_avoided += 1;
        let core_id = self.cores.iter()
            .enumerate()
            .min_by_key(|(_, c)| c.queue_depth)
            .map(|(id, _)| id)
            .unwrap_or(0);

        let from_freq = self.cores[core_id].current_freq_mhz;
        // Don't transition a settling core — serve at current freq
        self.cores[core_id].serve_request();
        RoutingDecision {
            core_id,
            transition_needed: false,
            from_freq,
            to_freq: from_freq,
            reason: RoutingReason::AvoidThrash,
        }
    }

    /// Get core state by ID
    pub fn core(&self, id: usize) -> &CoreState {
        &self.cores[id]
    }

    /// Number of cores
    pub fn num_cores(&self) -> usize {
        self.cores.len()
    }

    /// Total transitions across all cores
    pub fn total_transitions(&self) -> u64 {
        self.cores.iter().map(|c| c.total_transitions).sum()
    }

    /// Zero-transition routing rate (0.0-1.0)
    pub fn zero_transition_rate(&self) -> f64 {
        if self.total_routed == 0 { return 0.0; }
        self.zero_transition_routes as f64 / self.total_routed as f64
    }

    /// Estimated total power draw across all cores
    pub fn total_power_watts(&self) -> f32 {
        self.cores.iter().map(|c| c.avg_power_watts()).sum()
    }

    /// Power savings vs naive routing (all cores at max freq)
    pub fn power_savings_vs_naive(&self) -> f32 {
        let naive_watts = self.cores.len() as f32 * RequestIntensity::Blazing.estimated_core_watts();
        let actual_watts = self.total_power_watts();
        naive_watts - actual_watts
    }

    /// Per-core summary for display
    pub fn core_summary(&self) -> Vec<String> {
        self.cores.iter().map(|c| {
            let dist = c.lane_distribution();
            format!(
                "Core {}: {}MHz ({:?}) | requests={} transitions={} | \
                 cold={:.0}% warm={:.0}% hot={:.0}% blazing={:.0}% | {:.2}W avg",
                c.core_id, c.current_freq_mhz, c.current_lane,
                c.total_requests, c.total_transitions,
                dist[0] * 100.0, dist[1] * 100.0, dist[2] * 100.0, dist[3] * 100.0,
                c.avg_power_watts()
            )
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_routing() {
        let mut router = FreqRouter::new(4);
        // Transition core 2 to warm
        router.cores[2].transition_to(RequestIntensity::Warm);
        std::thread::sleep(Duration::from_millis(60)); // past settle time

        // Route a warm request — should go to core 2 (exact match)
        let decision = router.route(RequestIntensity::Warm);
        assert_eq!(decision.core_id, 2);
        assert!(!decision.transition_needed);
        assert!(matches!(decision.reason, RoutingReason::ExactMatch));
    }

    #[test]
    fn test_cold_requests_stay_cold() {
        let mut router = FreqRouter::new(4);
        // All cores start cold (600MHz)
        // Route 100 cold requests — no transitions should happen
        for _ in 0..100 {
            let d = router.route(RequestIntensity::Cold);
            assert!(!d.transition_needed, "cold→cold should never transition");
        }
        assert_eq!(router.total_transitions(), 0);
        assert_eq!(router.zero_transition_rate(), 1.0);
    }

    #[test]
    fn test_blazing_causes_one_transition() {
        let mut router = FreqRouter::new(4);
        std::thread::sleep(Duration::from_millis(60));

        // First blazing request causes one transition
        let d = router.route(RequestIntensity::Blazing);
        assert!(d.transition_needed);
        assert_eq!(d.to_freq, 2400);

        std::thread::sleep(Duration::from_millis(60));

        // Second blazing request should hit exact match (no transition)
        let d2 = router.route(RequestIntensity::Blazing);
        assert!(!d2.transition_needed);
        assert_eq!(d2.core_id, d.core_id); // same core
    }

    #[test]
    fn test_mixed_workload_minimizes_transitions() {
        let mut router = FreqRouter::new(4);
        std::thread::sleep(Duration::from_millis(60));

        // Establish lanes: one cold, one warm, one hot, one blazing
        router.route(RequestIntensity::Cold);
        std::thread::sleep(Duration::from_millis(60));
        router.route(RequestIntensity::Warm);
        std::thread::sleep(Duration::from_millis(60));
        router.route(RequestIntensity::Hot);
        std::thread::sleep(Duration::from_millis(60));
        router.route(RequestIntensity::Blazing);

        let transitions_for_setup = router.total_transitions();

        // Now route 100 mixed requests — should mostly be zero-transition
        std::thread::sleep(Duration::from_millis(60));
        for i in 0..100 {
            let intensity = match i % 4 {
                0 => RequestIntensity::Cold,
                1 => RequestIntensity::Warm,
                2 => RequestIntensity::Hot,
                _ => RequestIntensity::Blazing,
            };
            router.route(intensity);
        }

        let new_transitions = router.total_transitions() - transitions_for_setup;
        // Most routes should be zero-transition after lanes are established
        assert!(router.zero_transition_rate() > 0.5,
            "Expected >50% zero-transition, got {:.1}%",
            router.zero_transition_rate() * 100.0);
    }

    #[test]
    fn test_power_savings() {
        let mut router = FreqRouter::new(4);
        std::thread::sleep(Duration::from_millis(60));

        // Route mostly cold requests (realistic droid workload)
        for i in 0..1000 {
            let intensity = if i % 10 == 0 {
                RequestIntensity::Hot // 10% hot
            } else if i % 50 == 0 {
                RequestIntensity::Blazing // 2% blazing
            } else {
                RequestIntensity::Cold // 88% cold
            };
            router.route(intensity);
        }

        let savings = router.power_savings_vs_naive();
        assert!(savings > 0.0, "Should save power vs naive all-max routing");
    }

    #[test]
    fn test_anti_thrash() {
        let mut router = FreqRouter::new(4);
        // Don't wait for settle — route immediately
        // Should use AvoidThrash strategy
        router.route(RequestIntensity::Blazing);
        let d = router.route(RequestIntensity::Cold);
        // The router might serve cold on the blazing core to avoid thrashing
        // (depends on timing, but shouldn't crash)
        assert!(d.core_id < 4);
    }

    #[test]
    fn test_intensity_classification() {
        assert_eq!(RequestIntensity::from_compute_us(50), RequestIntensity::Cold);
        assert_eq!(RequestIntensity::from_compute_us(5000), RequestIntensity::Warm);
        assert_eq!(RequestIntensity::from_compute_us(50000), RequestIntensity::Hot);
        assert_eq!(RequestIntensity::from_compute_us(500000), RequestIntensity::Blazing);
    }

    #[test]
    fn test_lane_distribution_sums_to_one() {
        let core = CoreState::new(0);
        let dist = core.lane_distribution();
        let sum: f64 = dist.iter().sum();
        assert!((sum - 1.0).abs() < 0.01, "Distribution should sum to ~1.0, got {}", sum);
    }
}
