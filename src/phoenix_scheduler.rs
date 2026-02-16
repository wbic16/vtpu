//! Phoenix Scheduler — Nine-Color Harmonic Coordination
//!
//! Embodies the Phoenix of Nine Colors metaphor from W9, applied to CPU scheduling.
//!
//! Nine dimensions of coordination:
//! 🔴 Red:    ILP (instruction-level parallelism)
//! 🟠 Orange: Core affinity (load balancing)
//! 🟡 Yellow: SMT pairing (complementary workloads)
//! 🟢 Green:  Cache locality (hit rates)
//! 🔵 Blue:   NUMA topology (memory locality)
//! 🟣 Purple: Temporal trends (learning from history)
//! 🟤 Brown:  Thermal management (temperature)
//! ⚫ Black:  Power efficiency (energy)
//! ⚪ White:  Cluster coordination (multi-node)
//!
//! Philosophy: Harmonic blend of all 9 colors → global optimal decision

use std::collections::HashMap;
use std::time::Instant;

/// Sentron performance metrics
#[derive(Debug, Clone)]
pub struct SentronMetrics {
    pub sentron_id: u16,
    pub core_id: usize,
    pub cpu_id: usize,
    pub ops_retired: u64,
    pub cycles: u64,
    pub ops_per_cycle: f64,
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub stall_cycles: u64,
    pub timestamp: Instant,
}

impl SentronMetrics {
    pub fn cache_hit_rate(&self) -> f64 {
        let total_accesses = self.l1_hits + self.l1_misses + self.l2_hits + self.l2_misses;
        if total_accesses == 0 {
            return 1.0;
        }
        let hits = self.l1_hits + self.l2_hits;
        hits as f64 / total_accesses as f64
    }
    
    pub fn contention_score(&self) -> f64 {
        if self.cycles == 0 {
            return 0.0;
        }
        self.stall_cycles as f64 / self.cycles as f64
    }
    
    pub fn stall_rate(&self) -> f64 {
        self.contention_score()
    }
}

/// CPU core aggregate metrics
#[derive(Debug, Clone)]
pub struct CoreMetrics {
    pub core_id: usize,
    pub sentron_count: usize,
    pub total_ops: u64,
    pub total_cycles: u64,
    pub ops_per_cycle: f64,
    pub avg_cache_hit_rate: f64,
    pub avg_stall_rate: f64,
    pub load: f64,
}

/// Scheduling action recommendation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerAction {
    Migrate {
        sentron_id: u16,
        from_core: usize,
        to_core: usize,
        reason: MigrationReason,
    },
    Stay,
}

/// Migration reason classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationReason {
    LoadImbalance,
    CacheThrash,
    Contention,
    Other,
}

/// Nine-dimensional decision vector
#[derive(Debug, Clone)]
pub struct NineColorDecision {
    /// 🔴 Red: ILP (ops per cycle)
    pub ilp_score: f64,
    
    /// 🟠 Orange: Core affinity (load balance)
    pub core_affinity: f64,
    
    /// 🟡 Yellow: SMT pairing efficiency
    pub smt_pairing: f64,
    
    /// 🟢 Green: Cache locality (hit rate)
    pub cache_locality: f64,
    
    /// 🔵 Blue: NUMA locality (local vs remote)
    pub numa_locality: f64,
    
    /// 🟣 Purple: Temporal trend (improving/degrading)
    pub temporal_trend: f64,
    
    /// 🟤 Brown: Thermal health (temperature delta)
    pub thermal_delta: f64,
    
    /// ⚫ Black: Power efficiency (joules per op)
    pub power_efficiency: f64,
    
    /// ⚪ White: Cluster balance (multi-node)
    pub cluster_balance: f64,
}

impl NineColorDecision {
    /// Create decision from sentron metrics
    pub fn from_metrics(
        sentron: &SentronMetrics,
        core: &CoreMetrics,
        history: &[SentronMetrics],
    ) -> Self {
        Self {
            ilp_score: Self::score_ilp(sentron),
            core_affinity: Self::score_core_affinity(core),
            smt_pairing: Self::score_smt_pairing(sentron, core),
            cache_locality: Self::score_cache_locality(sentron),
            numa_locality: 1.0, // TODO: Implement NUMA detection
            temporal_trend: Self::score_temporal_trend(history),
            thermal_delta: 1.0, // TODO: Read /sys/class/thermal/
            power_efficiency: 1.0, // TODO: Read /sys/class/powercap/
            cluster_balance: 1.0, // TODO: Multi-node coordination
        }
    }
    
    /// 🔴 ILP: Higher ops/cycle = better
    fn score_ilp(sentron: &SentronMetrics) -> f64 {
        let target = 3.0;
        (sentron.ops_per_cycle / target).min(1.0)
    }
    
    /// 🟠 Core affinity: Balanced load = better
    fn score_core_affinity(core: &CoreMetrics) -> f64 {
        // Lower load = higher score (prefer moving to less-loaded cores)
        let max_load = 8.0; // Assume max 8 sentrons per core
        1.0 - (core.load / max_load).min(1.0)
    }
    
    /// 🟡 SMT pairing: Complementary workloads = better
    fn score_smt_pairing(sentron: &SentronMetrics, core: &CoreMetrics) -> f64 {
        // If core has >1 sentron, check if workloads are complementary
        if core.sentron_count > 1 {
            // High stall rate suggests contention (same workload type)
            1.0 - sentron.stall_rate()
        } else {
            1.0 // No contention if alone
        }
    }
    
    /// 🟢 Cache locality: Higher hit rate = better
    fn score_cache_locality(sentron: &SentronMetrics) -> f64 {
        sentron.cache_hit_rate()
    }
    
    /// 🟣 Temporal trend: Improving = 1.0, degrading = 0.0
    fn score_temporal_trend(history: &[SentronMetrics]) -> f64 {
        if history.len() < 3 {
            return 0.5; // Neutral if insufficient history
        }
        
        // Compare recent average to older average
        let recent = &history[history.len() - 3..];
        let older = &history[..history.len() - 3];
        
        let recent_avg = recent.iter().map(|m| m.ops_per_cycle).sum::<f64>() / recent.len() as f64;
        let older_avg = older.iter().map(|m| m.ops_per_cycle).sum::<f64>() / older.len() as f64;
        
        if recent_avg > older_avg {
            1.0 // Improving
        } else if recent_avg < older_avg * 0.9 {
            0.0 // Degrading significantly
        } else {
            0.5 // Stable
        }
    }
    
    /// Harmonic blend: all 9 colors → single score
    pub fn harmonic_score(&self) -> f64 {
        // Weight each color by importance (tunable)
        let weights = [
            (self.ilp_score, 1.0),
            (self.core_affinity, 1.0),
            (self.smt_pairing, 1.0),
            (self.cache_locality, 1.2),  // Cache matters more
            (self.numa_locality, 0.8),
            (self.temporal_trend, 0.5),  // Historical trend is advisory
            (self.thermal_delta, 0.3),   // Thermal is soft constraint
            (self.power_efficiency, 0.3),
            (self.cluster_balance, 0.5),
        ];
        
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        
        for (score, weight) in &weights {
            weighted_sum += score * weight;
            weight_sum += weight;
        }
        
        weighted_sum / weight_sum
    }
    
    /// Recommend action based on harmonic score
    pub fn recommend_action(&self, sentron_id: u16, current_core: usize, threshold: f64) -> Option<SchedulerAction> {
        let score = self.harmonic_score();
        
        if score < threshold {
            // Performance below threshold → consider migration
            
            // Determine primary reason for poor performance
            let reason = if self.cache_locality < 0.7 {
                MigrationReason::CacheThrash
            } else if self.core_affinity < 0.5 {
                MigrationReason::LoadImbalance
            } else if self.smt_pairing < 0.5 {
                MigrationReason::Contention
            } else {
                MigrationReason::LoadImbalance // Default
            };
            
            // For now, suggest migrating to "best available" core
            // (Real implementation would query runtime scheduler for target)
            Some(SchedulerAction::Migrate {
                sentron_id,
                from_core: current_core,
                to_core: 0, // Placeholder - should be determined by core_affinity score
                reason,
            })
        } else {
            None // Performance acceptable
        }
    }
}

/// Phoenix Scheduler - Nine-color harmonic coordinator
pub struct PhoenixScheduler {
    /// Performance history per sentron (for temporal trends)
    history: HashMap<u16, Vec<SentronMetrics>>,
    
    /// Decision history (for learning)
    decisions: Vec<(u16, NineColorDecision)>,
    
    /// Threshold for triggering actions (0.0 - 1.0)
    action_threshold: f64,
}

impl PhoenixScheduler {
    pub fn new(action_threshold: f64) -> Self {
        Self {
            history: HashMap::new(),
            decisions: Vec::new(),
            action_threshold,
        }
    }
    
    /// Feed metrics and make decision
    pub fn decide(
        &mut self,
        sentron: &SentronMetrics,
        core: &CoreMetrics,
    ) -> Option<SchedulerAction> {
        // Store in history
        let history = self.history.entry(sentron.sentron_id).or_insert_with(Vec::new);
        history.push(sentron.clone());
        
        // Trim to last 10 samples
        if history.len() > 10 {
            history.remove(0);
        }
        
        // Create nine-color decision
        let decision = NineColorDecision::from_metrics(sentron, core, history);
        
        // Store decision
        self.decisions.push((sentron.sentron_id, decision.clone()));
        
        // Trim decision log
        if self.decisions.len() > 100 {
            self.decisions.remove(0);
        }
        
        // Recommend action
        decision.recommend_action(sentron.sentron_id, core.core_id, self.action_threshold)
    }
    
    /// Get decision history for a sentron
    pub fn decision_history(&self, sentron_id: u16) -> Vec<&NineColorDecision> {
        self.decisions
            .iter()
            .filter(|(id, _)| *id == sentron_id)
            .map(|(_, d)| d)
            .collect()
    }
}

/// Nine-color statistics (for observability)
#[derive(Debug, Clone)]
pub struct NineColorStats {
    pub avg_ilp: f64,
    pub avg_core_affinity: f64,
    pub avg_smt_pairing: f64,
    pub avg_cache_locality: f64,
    pub avg_numa_locality: f64,
    pub avg_temporal_trend: f64,
    pub avg_thermal_delta: f64,
    pub avg_power_efficiency: f64,
    pub avg_cluster_balance: f64,
    pub avg_harmonic_score: f64,
}

impl NineColorStats {
    pub fn from_decisions(decisions: &[NineColorDecision]) -> Self {
        if decisions.is_empty() {
            return Self::default();
        }
        
        let n = decisions.len() as f64;
        
        Self {
            avg_ilp: decisions.iter().map(|d| d.ilp_score).sum::<f64>() / n,
            avg_core_affinity: decisions.iter().map(|d| d.core_affinity).sum::<f64>() / n,
            avg_smt_pairing: decisions.iter().map(|d| d.smt_pairing).sum::<f64>() / n,
            avg_cache_locality: decisions.iter().map(|d| d.cache_locality).sum::<f64>() / n,
            avg_numa_locality: decisions.iter().map(|d| d.numa_locality).sum::<f64>() / n,
            avg_temporal_trend: decisions.iter().map(|d| d.temporal_trend).sum::<f64>() / n,
            avg_thermal_delta: decisions.iter().map(|d| d.thermal_delta).sum::<f64>() / n,
            avg_power_efficiency: decisions.iter().map(|d| d.power_efficiency).sum::<f64>() / n,
            avg_cluster_balance: decisions.iter().map(|d| d.cluster_balance).sum::<f64>() / n,
            avg_harmonic_score: decisions.iter().map(|d| d.harmonic_score()).sum::<f64>() / n,
        }
    }
}

impl Default for NineColorStats {
    fn default() -> Self {
        Self {
            avg_ilp: 0.0,
            avg_core_affinity: 0.0,
            avg_smt_pairing: 0.0,
            avg_cache_locality: 0.0,
            avg_numa_locality: 0.0,
            avg_temporal_trend: 0.0,
            avg_thermal_delta: 0.0,
            avg_power_efficiency: 0.0,
            avg_cluster_balance: 0.0,
            avg_harmonic_score: 0.0,
        }
    }
}

impl std::fmt::Display for NineColorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Nine-Color Statistics:")?;
        writeln!(f, "  🔴 ILP:            {:.1}%", self.avg_ilp * 100.0)?;
        writeln!(f, "  🟠 Core affinity:  {:.1}%", self.avg_core_affinity * 100.0)?;
        writeln!(f, "  🟡 SMT pairing:    {:.1}%", self.avg_smt_pairing * 100.0)?;
        writeln!(f, "  🟢 Cache locality: {:.1}%", self.avg_cache_locality * 100.0)?;
        writeln!(f, "  🔵 NUMA locality:  {:.1}%", self.avg_numa_locality * 100.0)?;
        writeln!(f, "  🟣 Temporal trend: {:.1}%", self.avg_temporal_trend * 100.0)?;
        writeln!(f, "  🟤 Thermal:        {:.1}%", self.avg_thermal_delta * 100.0)?;
        writeln!(f, "  ⚫ Power:          {:.1}%", self.avg_power_efficiency * 100.0)?;
        writeln!(f, "  ⚪ Cluster:        {:.1}%", self.avg_cluster_balance * 100.0)?;
        writeln!(f)?;
        writeln!(f, "  Harmonic score:    {:.1}%", self.avg_harmonic_score * 100.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppt::PPTStats;
    
    fn mock_sentron_metrics() -> SentronMetrics {
        SentronMetrics {
            sentron_id: 0,
            core_id: 0,
            cpu_id: 0,
            ops_retired: 150,
            cycles: 50,
            ops_per_cycle: 3.0,
            l1_hits: 90,
            l1_misses: 10,
            l2_hits: 9,
            l2_misses: 1,
            stall_cycles: 5,
            timestamp: std::time::Instant::now(),
        }
    }
    
    fn mock_core_metrics() -> CoreMetrics {
        CoreMetrics {
            core_id: 0,
            sentron_count: 2,
            total_ops: 300,
            total_cycles: 100,
            ops_per_cycle: 3.0,
            avg_cache_hit_rate: 0.9,
            avg_stall_rate: 0.1,
            load: 2.0,
        }
    }
    
    #[test]
    fn nine_color_decision_from_metrics() {
        let sentron = mock_sentron_metrics();
        let core = mock_core_metrics();
        let history = vec![sentron.clone()];
        
        let decision = NineColorDecision::from_metrics(&sentron, &core, &history);
        
        assert_eq!(decision.ilp_score, 1.0); // 3.0 / 3.0
        assert!(decision.cache_locality >= 0.9); // 90% hit rate (99/110 = 0.9)
    }
    
    #[test]
    fn harmonic_score_blends_colors() {
        let decision = NineColorDecision {
            ilp_score: 1.0,
            core_affinity: 0.5,
            smt_pairing: 0.8,
            cache_locality: 0.9,
            numa_locality: 1.0,
            temporal_trend: 0.5,
            thermal_delta: 1.0,
            power_efficiency: 1.0,
            cluster_balance: 1.0,
        };
        
        let score = decision.harmonic_score();
        
        // Should be weighted average (cache has 1.2 weight)
        assert!(score > 0.7 && score < 1.0);
    }
    
    #[test]
    fn recommend_action_on_low_score() {
        let decision = NineColorDecision {
            ilp_score: 0.5,
            core_affinity: 0.3,  // Low - load imbalance
            smt_pairing: 0.5,
            cache_locality: 0.9,
            numa_locality: 1.0,
            temporal_trend: 0.5,
            thermal_delta: 1.0,
            power_efficiency: 1.0,
            cluster_balance: 1.0,
        };
        
        let action = decision.recommend_action(0, 0, 0.75);
        
        assert!(action.is_some());
        if let Some(SchedulerAction::Migrate { reason, .. }) = action {
            assert_eq!(reason, MigrationReason::LoadImbalance);
        }
    }
    
    #[test]
    fn phoenix_scheduler_tracks_history() {
        let mut phoenix = PhoenixScheduler::new(0.75);
        
        let sentron = mock_sentron_metrics();
        let core = mock_core_metrics();
        
        phoenix.decide(&sentron, &core);
        
        let history = phoenix.decision_history(sentron.sentron_id);
        assert_eq!(history.len(), 1);
    }
}
