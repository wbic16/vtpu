//! Phoenix Scheduler — Nine-Color Harmonic Coordination
//!
//! Embodies the Phoenix of Nine Colors: 9 scheduling dimensions
//! blend into one coordinated decision via harmonic scoring.
//!
//! 🔴 Red:    ILP (ops per cycle)
//! 🟠 Orange: Core affinity (load balance)
//! 🟡 Yellow: SMT pairing (complementary workloads)
//! 🟢 Green:  Cache locality (PPT hit rate)
//! 🔵 Blue:   NUMA topology (memory locality)
//! 🟣 Purple: Temporal trends (learning from history)
//! 🟤 Brown:  Thermal management (placeholder)
//! ⚫ Black:  Power efficiency (placeholder)
//! ⚪ White:  Cluster coordination (placeholder)

use crate::exec::ExecStats;
use crate::ppt::PPTStats;

/// Performance snapshot for a sentron execution.
#[derive(Debug, Clone)]
pub struct SentronMetrics {
    pub sentron_id: u16,
    pub core_id: usize,
    pub ops_retired: u64,
    pub cycles: u64,
    pub ops_per_cycle: f64,
    pub d_util: f64,
    pub s_util: f64,
    pub c_util: f64,
    pub ptc_hit_rate: f64,
}

impl SentronMetrics {
    pub fn from_exec(sentron_id: u16, core_id: usize, stats: &ExecStats, ppt: &PPTStats) -> Self {
        SentronMetrics {
            sentron_id,
            core_id,
            ops_retired: stats.ops_retired,
            cycles: stats.cycles,
            ops_per_cycle: stats.ops_per_cycle(),
            d_util: if stats.siws_retired > 0 { stats.d_ops as f64 / stats.siws_retired as f64 } else { 0.0 },
            s_util: if stats.siws_retired > 0 { stats.s_ops as f64 / stats.siws_retired as f64 } else { 0.0 },
            c_util: if stats.siws_retired > 0 { stats.c_ops as f64 / stats.siws_retired as f64 } else { 0.0 },
            ptc_hit_rate: ppt.ptc_hit_rate,
        }
    }

    /// Simple constructor for testing/benchmarks without PPT.
    pub fn from_exec_simple(sentron_id: u16, core_id: usize, stats: &ExecStats) -> Self {
        SentronMetrics {
            sentron_id,
            core_id,
            ops_retired: stats.ops_retired,
            cycles: stats.cycles,
            ops_per_cycle: stats.ops_per_cycle(),
            d_util: if stats.siws_retired > 0 { stats.d_ops as f64 / stats.siws_retired as f64 } else { 0.0 },
            s_util: if stats.siws_retired > 0 { stats.s_ops as f64 / stats.siws_retired as f64 } else { 0.0 },
            c_util: if stats.siws_retired > 0 { stats.c_ops as f64 / stats.siws_retired as f64 } else { 0.0 },
            ptc_hit_rate: 1.0,
        }
    }

    /// Are D and S pipes complementary? (one high, one low = good SMT pair)
    pub fn is_complementary(&self) -> bool {
        (self.d_util - self.s_util).abs() > 0.3
    }
}

/// Aggregate metrics for a physical core.
#[derive(Debug, Clone)]
pub struct CoreMetrics {
    pub core_id: usize,
    pub sentron_count: usize,
    pub total_ops: u64,
    pub avg_ops_per_cycle: f64,
    pub avg_ptc_hit_rate: f64,
}

impl CoreMetrics {
    pub fn from_sentrons(core_id: usize, metrics: &[SentronMetrics]) -> Self {
        let count = metrics.len();
        if count == 0 {
            return CoreMetrics {
                core_id, sentron_count: 0, total_ops: 0,
                avg_ops_per_cycle: 0.0, avg_ptc_hit_rate: 0.0,
            };
        }
        let total_ops: u64 = metrics.iter().map(|m| m.ops_retired).sum();
        let avg_opc = metrics.iter().map(|m| m.ops_per_cycle).sum::<f64>() / count as f64;
        let avg_ptc = metrics.iter().map(|m| m.ptc_hit_rate).sum::<f64>() / count as f64;
        CoreMetrics {
            core_id, sentron_count: count, total_ops,
            avg_ops_per_cycle: avg_opc, avg_ptc_hit_rate: avg_ptc,
        }
    }
}

/// Nine-dimensional decision vector — one score per color.
#[derive(Debug, Clone)]
pub struct NineColorDecision {
    pub ilp: f64,           // 🔴 Red
    pub core_affinity: f64, // 🟠 Orange
    pub smt_pairing: f64,   // 🟡 Yellow
    pub cache_locality: f64,// 🟢 Green
    pub numa_locality: f64, // 🔵 Blue
    pub temporal_trend: f64,// 🟣 Purple
    pub thermal: f64,       // 🟤 Brown
    pub power: f64,         // ⚫ Black
    pub cluster: f64,       // ⚪ White
}

impl NineColorDecision {
    /// Score from a sentron + core snapshot.
    pub fn from_metrics(sentron: &SentronMetrics, core: &CoreMetrics, history: &[f64]) -> Self {
        NineColorDecision {
            ilp: (sentron.ops_per_cycle / 3.0).min(1.0),
            core_affinity: if core.sentron_count > 0 {
                1.0 - (core.sentron_count as f64 / 360.0).min(1.0)
            } else { 1.0 },
            smt_pairing: if sentron.is_complementary() { 1.0 } else { 0.5 },
            cache_locality: sentron.ptc_hit_rate,
            numa_locality: 1.0,  // single-node for now
            temporal_trend: Self::score_trend(history),
            thermal: 1.0,       // placeholder — future: /sys/class/thermal/
            power: 1.0,         // placeholder — future: /sys/class/powercap/
            cluster: 1.0,       // placeholder — future: multi-node
        }
    }

    fn score_trend(history: &[f64]) -> f64 {
        if history.len() < 2 { return 0.5; }
        let recent = history[history.len() - 1];
        let prior = history[history.len() - 2];
        if prior == 0.0 { return 0.5; }
        let ratio = recent / prior;
        // >1.0 = improving, <1.0 = degrading
        (ratio / 2.0).min(1.0)
    }

    /// Harmonic blend — weighted sum of all 9 colors.
    /// Weights reflect W17 findings: cache and ILP matter most,
    /// thermal/power/cluster are placeholders until measured.
    pub fn harmonic_score(&self) -> f64 {
        let weights = [
            1.0,  // 🔴 ILP
            0.8,  // 🟠 Core affinity
            0.9,  // 🟡 SMT pairing
            1.2,  // 🟢 Cache locality (highest — W17 proved it matters)
            0.7,  // 🔵 NUMA
            0.6,  // 🟣 Temporal trend
            0.3,  // 🟤 Thermal (placeholder)
            0.3,  // ⚫ Power (placeholder)
            0.5,  // ⚪ Cluster (placeholder)
        ];
        let scores = [
            self.ilp, self.core_affinity, self.smt_pairing,
            self.cache_locality, self.numa_locality, self.temporal_trend,
            self.thermal, self.power, self.cluster,
        ];
        let weighted_sum: f64 = scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum();
        let total_weight: f64 = weights.iter().sum();
        weighted_sum / total_weight
    }

    /// All nine scores as an array.
    pub fn as_array(&self) -> [f64; 9] {
        [self.ilp, self.core_affinity, self.smt_pairing,
         self.cache_locality, self.numa_locality, self.temporal_trend,
         self.thermal, self.power, self.cluster]
    }
}

/// Recommended action from the Phoenix scheduler.
#[derive(Debug, Clone, PartialEq)]
pub enum PhoenixAction {
    /// Stay put — everything is resonating.
    Hold,
    /// Migrate sentron to a different core.
    Migrate { sentron_id: u16, to_core: usize, reason: String },
    /// Rebalance the fleet across cores.
    Rebalance,
}

/// The Phoenix Scheduler — coordinates via harmonic resonance.
pub struct PhoenixScheduler {
    /// Decision history per sentron.
    history: std::collections::HashMap<u16, Vec<f64>>,
    /// Minimum harmonic score before recommending migration.
    migration_threshold: f64,
}

impl PhoenixScheduler {
    pub fn new() -> Self {
        PhoenixScheduler {
            history: std::collections::HashMap::new(),
            migration_threshold: 0.5,
        }
    }

    /// Evaluate a sentron and recommend an action.
    pub fn evaluate(&mut self, sentron: &SentronMetrics, core: &CoreMetrics) -> (NineColorDecision, PhoenixAction) {
        let hist = self.history.entry(sentron.sentron_id).or_default();
        let decision = NineColorDecision::from_metrics(sentron, core, hist);
        let score = decision.harmonic_score();

        hist.push(score);
        if hist.len() > 16 { hist.remove(0); }

        let action = if score < self.migration_threshold {
            PhoenixAction::Migrate {
                sentron_id: sentron.sentron_id,
                to_core: 0, // simplistic — pick least loaded
                reason: format!("harmonic score {:.3} < threshold {:.3}", score, self.migration_threshold),
            }
        } else {
            PhoenixAction::Hold
        };

        (decision, action)
    }

    /// Evaluate an entire fleet and return aggregate recommendation.
    pub fn evaluate_fleet(&mut self, fleet: &[(SentronMetrics, CoreMetrics)]) -> Vec<(NineColorDecision, PhoenixAction)> {
        fleet.iter().map(|(s, c)| self.evaluate(s, c)).collect()
    }

    pub fn history_len(&self, sentron_id: u16) -> usize {
        self.history.get(&sentron_id).map_or(0, |h| h.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_sentron(id: u16, opc: f64, d: f64, s: f64) -> SentronMetrics {
        SentronMetrics {
            sentron_id: id, core_id: 0,
            ops_retired: 1000, cycles: (1000.0 / opc) as u64,
            ops_per_cycle: opc, d_util: d, s_util: s, c_util: 0.1,
            ptc_hit_rate: 0.95,
        }
    }

    fn mock_core(id: usize, count: usize) -> CoreMetrics {
        CoreMetrics {
            core_id: id, sentron_count: count, total_ops: 1000,
            avg_ops_per_cycle: 2.5, avg_ptc_hit_rate: 0.95,
        }
    }

    #[test]
    fn nine_color_decision() {
        let s = mock_sentron(0, 3.0, 0.9, 0.1);
        let c = mock_core(0, 10);
        let d = NineColorDecision::from_metrics(&s, &c, &[]);
        assert!((d.ilp - 1.0).abs() < 0.01);
        assert!(d.smt_pairing == 1.0); // complementary (0.9 vs 0.1)
        assert!(d.harmonic_score() > 0.5);
    }

    #[test]
    fn harmonic_blend() {
        let s = mock_sentron(0, 2.0, 0.5, 0.5);
        let c = mock_core(0, 180);
        let d = NineColorDecision::from_metrics(&s, &c, &[0.6, 0.7]);
        let score = d.harmonic_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn phoenix_hold() {
        let mut phoenix = PhoenixScheduler::new();
        let s = mock_sentron(1, 3.0, 0.8, 0.2);
        let c = mock_core(0, 5);
        let (_, action) = phoenix.evaluate(&s, &c);
        assert_eq!(action, PhoenixAction::Hold);
    }

    #[test]
    fn phoenix_migrate_on_low_score() {
        let mut phoenix = PhoenixScheduler::new();
        let s = SentronMetrics {
            sentron_id: 2, core_id: 0,
            ops_retired: 10, cycles: 100,
            ops_per_cycle: 0.1, d_util: 0.5, s_util: 0.5, c_util: 0.0,
            ptc_hit_rate: 0.1,
        };
        let c = mock_core(0, 350); // overloaded
        let (_, action) = phoenix.evaluate(&s, &c);
        assert!(matches!(action, PhoenixAction::Migrate { .. }));
    }

    #[test]
    fn complementary_detection() {
        let comp = mock_sentron(0, 2.5, 0.9, 0.1);
        assert!(comp.is_complementary());
        let even = mock_sentron(1, 2.5, 0.5, 0.5);
        assert!(!even.is_complementary());
    }

    #[test]
    fn fleet_evaluation() {
        let mut phoenix = PhoenixScheduler::new();
        let fleet: Vec<(SentronMetrics, CoreMetrics)> = (0..9).map(|i| {
            (mock_sentron(i, 2.5 + (i as f64 * 0.05), 0.8, 0.2), mock_core(i as usize, 40))
        }).collect();
        let results = phoenix.evaluate_fleet(&fleet);
        assert_eq!(results.len(), 9);
        // All healthy — should hold
        for (_, action) in &results {
            assert_eq!(*action, PhoenixAction::Hold);
        }
    }

    #[test]
    fn history_tracking() {
        let mut phoenix = PhoenixScheduler::new();
        let s = mock_sentron(5, 2.8, 0.7, 0.3);
        let c = mock_core(0, 20);
        for _ in 0..5 {
            phoenix.evaluate(&s, &c);
        }
        assert_eq!(phoenix.history_len(5), 5);
    }
}
