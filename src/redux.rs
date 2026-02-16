//! Redux — Real-Time Scheduler Feedback Loop (R23W17-2)
//!
//! Feeds execution metrics back into scheduling decisions in real-time.
//! The vTPU observes itself and adapts:
//!   - Pipe utilization → rebalance workload assignment
//!   - Cache behavior → adjust quantum size
//!   - Throughput trend → detect stalls and switch strategy
//!
//! "Redux" = reduce + feedback. State flows one direction:
//!   Execute → Observe → Decide → Adjust → Execute

use crate::exec::ExecStats;

/// Observation window: how many quanta of history to keep
const WINDOW_SIZE: usize = 16;

/// Feedback signals from execution
#[derive(Debug, Clone, Default)]
pub struct Signal {
    pub ops_per_cycle: f64,
    pub d_utilization: f64,
    pub s_utilization: f64,
    pub c_utilization: f64,
    pub siws_retired: u64,
}

impl Signal {
    pub fn from_stats(stats: &ExecStats) -> Self {
        Self {
            ops_per_cycle: stats.ops_per_cycle(),
            d_utilization: stats.d_utilization(),
            s_utilization: stats.s_utilization(),
            c_utilization: stats.c_utilization(),
            siws_retired: stats.siws_retired,
        }
    }

    /// Overall pipe balance: 0.0 = completely unbalanced, 1.0 = perfect 3-way balance
    pub fn balance(&self) -> f64 {
        let utils = [self.d_utilization, self.s_utilization, self.c_utilization];
        let max = utils.iter().cloned().fold(0.0_f64, f64::max);
        let min = utils.iter().cloned().fold(1.0_f64, f64::min);
        if max < 0.001 { return 0.0; }
        min / max
    }
}

/// Scheduling adjustment decisions
#[derive(Debug, Clone, PartialEq)]
pub enum Adjustment {
    /// Stay the course — performance is good
    Hold,
    /// Increase quantum size (reduce yield overhead)
    IncreaseQuantum { new_quantum: usize },
    /// Decrease quantum size (improve responsiveness)
    DecreaseQuantum { new_quantum: usize },
    /// Switch to complementary pairing (D+S imbalance detected)
    PairComplementary,
    /// Workload is stalling — try a different sentron order
    Reorder,
    /// Performance is degrading — flush and restart scheduling
    Reset,
}

/// The Redux feedback controller
pub struct Redux {
    /// Rolling window of recent signals
    history: Vec<Signal>,
    /// Current quantum size
    quantum: usize,
    /// Number of adjustments made
    adjustments: u64,
    /// Consecutive stall count
    stall_count: u32,
    /// Best observed ops/cycle
    peak_opc: f64,
}

impl Redux {
    pub fn new(initial_quantum: usize) -> Self {
        Self {
            history: Vec::with_capacity(WINDOW_SIZE),
            quantum: initial_quantum,
            adjustments: 0,
            stall_count: 0,
            peak_opc: 0.0,
        }
    }

    /// Feed a new execution signal into the controller.
    /// Returns the recommended adjustment.
    pub fn observe(&mut self, signal: Signal) -> Adjustment {
        // Track peak
        if signal.ops_per_cycle > self.peak_opc {
            self.peak_opc = signal.ops_per_cycle;
        }

        // Add to history (ring buffer behavior)
        if self.history.len() >= WINDOW_SIZE {
            self.history.remove(0);
        }
        self.history.push(signal);

        // Need at least 3 samples to make decisions
        if self.history.len() < 3 {
            return Adjustment::Hold;
        }

        let decision = self.decide();
        if decision != Adjustment::Hold {
            self.adjustments += 1;
        }
        decision
    }

    fn decide(&mut self) -> Adjustment {
        let recent = &self.history[self.history.len() - 1];
        let prev = &self.history[self.history.len() - 2];

        // Detect stall: ops/cycle dropped significantly
        if recent.ops_per_cycle < prev.ops_per_cycle * 0.7 && recent.ops_per_cycle > 0.0 {
            self.stall_count += 1;
            if self.stall_count >= 3 {
                self.stall_count = 0;
                return Adjustment::Reset;
            }
            return Adjustment::Reorder;
        } else {
            self.stall_count = 0;
        }

        // Detect pipe imbalance: one pipe doing most work
        let balance = recent.balance();
        if balance < 0.3 {
            // Severe imbalance — try complementary pairing
            return Adjustment::PairComplementary;
        }

        // Throughput optimization: adjust quantum
        let avg_opc = self.avg_ops_per_cycle();
        if avg_opc > 0.0 {
            // If we're achieving >90% of peak, increase quantum for efficiency
            if avg_opc > self.peak_opc * 0.9 && self.quantum < 256 {
                let new_q = (self.quantum * 2).min(256);
                if new_q != self.quantum {
                    self.quantum = new_q;
                    return Adjustment::IncreaseQuantum { new_quantum: new_q };
                }
            }
            // If below 60% of peak, decrease quantum for responsiveness
            if avg_opc < self.peak_opc * 0.6 && self.quantum > 8 {
                let new_q = (self.quantum / 2).max(8);
                if new_q != self.quantum {
                    self.quantum = new_q;
                    return Adjustment::DecreaseQuantum { new_quantum: new_q };
                }
            }
        }

        Adjustment::Hold
    }

    /// Average ops/cycle over the observation window
    pub fn avg_ops_per_cycle(&self) -> f64 {
        if self.history.is_empty() { return 0.0; }
        let sum: f64 = self.history.iter().map(|s| s.ops_per_cycle).sum();
        sum / self.history.len() as f64
    }

    /// Average pipe balance over the window
    pub fn avg_balance(&self) -> f64 {
        if self.history.is_empty() { return 0.0; }
        let sum: f64 = self.history.iter().map(|s| s.balance()).sum();
        sum / self.history.len() as f64
    }

    pub fn quantum(&self) -> usize { self.quantum }
    pub fn adjustments(&self) -> u64 { self.adjustments }
    pub fn peak_opc(&self) -> f64 { self.peak_opc }
    pub fn window_size(&self) -> usize { self.history.len() }
}

impl Default for Redux {
    fn default() -> Self { Self::new(64) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(opc: f64, d: f64, s: f64, c: f64) -> Signal {
        Signal {
            ops_per_cycle: opc,
            d_utilization: d,
            s_utilization: s,
            c_utilization: c,
            siws_retired: 100,
        }
    }

    #[test]
    fn hold_with_insufficient_data() {
        let mut redux = Redux::new(64);
        assert_eq!(redux.observe(make_signal(3.0, 1.0, 1.0, 1.0)), Adjustment::Hold);
        assert_eq!(redux.observe(make_signal(3.0, 1.0, 1.0, 1.0)), Adjustment::Hold);
    }

    #[test]
    fn hold_when_stable() {
        // At max quantum (256), no further increases possible → Hold
        let mut redux = Redux::new(256);
        for _ in 0..5 {
            assert_eq!(redux.observe(make_signal(2.5, 0.8, 0.7, 0.6)), Adjustment::Hold);
        }
    }

    #[test]
    fn detect_stall() {
        let mut redux = Redux::new(64);
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        // Sudden drop
        let adj = redux.observe(make_signal(1.0, 0.3, 0.3, 0.3));
        assert_eq!(adj, Adjustment::Reorder);
    }

    #[test]
    fn detect_pipe_imbalance() {
        let mut redux = Redux::new(64);
        redux.observe(make_signal(1.0, 1.0, 0.1, 0.0));
        redux.observe(make_signal(1.0, 1.0, 0.1, 0.0));
        let adj = redux.observe(make_signal(1.0, 1.0, 0.1, 0.0));
        assert_eq!(adj, Adjustment::PairComplementary);
    }

    #[test]
    fn increase_quantum_at_peak() {
        let mut redux = Redux::new(32);
        // Sustain high performance — first 3 establish history, then triggers
        redux.observe(make_signal(3.0, 0.9, 0.8, 0.7));
        redux.observe(make_signal(3.0, 0.9, 0.8, 0.7));
        redux.observe(make_signal(3.0, 0.9, 0.8, 0.7));
        let adj = redux.observe(make_signal(3.0, 0.9, 0.8, 0.7));
        // Should increase quantum since avg ≈ peak and quantum < 256
        assert!(matches!(adj, Adjustment::IncreaseQuantum { .. }), "got {:?}", adj);
    }

    #[test]
    fn decrease_quantum_on_degradation() {
        let mut redux = Redux::new(128);
        // Establish peak at 3.0
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        // Gradual sustained decline: each step >70% of prev (no stall), but avg drops below 60% of peak
        for opc in [2.5, 2.0, 1.7, 1.5, 1.3, 1.1, 1.0, 1.0, 1.0, 1.0] {
            redux.observe(make_signal(opc, 0.5, 0.5, 0.5));
        }
        // avg now well below 1.8 (60% of peak 3.0)
        let adj = redux.observe(make_signal(1.0, 0.5, 0.5, 0.5));
        assert!(matches!(adj, Adjustment::DecreaseQuantum { .. }), "got {:?}, avg={:.2}", adj, redux.avg_ops_per_cycle());
    }

    #[test]
    fn reset_after_consecutive_stalls() {
        let mut redux = Redux::new(64);
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        // 3 consecutive stalls
        redux.observe(make_signal(1.0, 0.3, 0.3, 0.3)); // Reorder
        redux.observe(make_signal(0.5, 0.2, 0.2, 0.2)); // Reorder
        let adj = redux.observe(make_signal(0.3, 0.1, 0.1, 0.1)); // Reset
        assert_eq!(adj, Adjustment::Reset);
    }

    #[test]
    fn balance_metric() {
        let balanced = make_signal(3.0, 0.9, 0.9, 0.9);
        assert!(balanced.balance() > 0.9);

        let imbalanced = make_signal(1.0, 1.0, 0.1, 0.0);
        assert!(imbalanced.balance() < 0.1);
    }

    #[test]
    fn peak_tracking() {
        let mut redux = Redux::new(64);
        redux.observe(make_signal(2.0, 0.7, 0.7, 0.7));
        redux.observe(make_signal(3.0, 1.0, 1.0, 1.0));
        redux.observe(make_signal(2.5, 0.8, 0.8, 0.8));
        assert!((redux.peak_opc() - 3.0).abs() < f64::EPSILON);
    }
}
