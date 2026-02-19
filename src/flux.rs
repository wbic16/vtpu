// -----------------------------------------------
// flux.rs — Sentron Flux Analysis
// -----------------------------------------------
// Measures the flow of sentrons through lifecycle states.
// Flux = the rate at which sentrons transition between
// Dormant → Running → Waiting → Retired → (recycle).
//
// Deep alignment: the flux pattern reveals whether the
// system is breathing correctly. Balanced flux means
// equal inhale (activation) and exhale (retirement).
// Bottlenecks appear as state accumulation.
//
// Zero external dependencies.
//
// R23W22 — Chrys 🦋

use crate::sentron::SentronState;

/// Flux measurement for a single state transition
#[derive(Debug, Clone, Copy)]
pub struct StateTransition {
    pub from: SentronState,
    pub to: SentronState,
    pub count: u64,
    pub total_cycles: u64,
}

impl StateTransition {
    pub fn avg_cycles(&self) -> f64 {
        if self.count == 0 { return 0.0; }
        self.total_cycles as f64 / self.count as f64
    }
}

/// Flux state for the entire fleet
pub struct FluxAnalyzer {
    /// Transition counts: [from][to] indexed by state ordinal
    transitions: [[u64; 4]; 4],
    /// Cycle costs: [from][to]
    cycle_costs: [[u64; 4]; 4],
    /// Current population per state
    population: [u64; 4],
    /// Total sentrons tracked
    total_sentrons: u64,
    /// Total transitions observed
    total_transitions: u64,
    /// Imbalance score history (running average)
    imbalance_sum: f64,
    imbalance_samples: u64,
}

fn state_idx(s: &SentronState) -> usize {
    match s {
        SentronState::Dormant => 0,
        SentronState::Running => 1,
        SentronState::Waiting => 2,
        SentronState::Retired => 3,
    }
}

impl FluxAnalyzer {
    pub fn new() -> Self {
        Self {
            transitions: [[0; 4]; 4],
            cycle_costs: [[0; 4]; 4],
            population: [0; 4],
            total_sentrons: 0,
            total_transitions: 0,
            imbalance_sum: 0.0,
            imbalance_samples: 0,
        }
    }

    /// Register a sentron entering the system
    pub fn enter(&mut self, state: &SentronState) {
        self.population[state_idx(state)] += 1;
        self.total_sentrons += 1;
    }

    /// Record a state transition with cycle cost
    pub fn transition(&mut self, from: &SentronState, to: &SentronState, cycles: u64) {
        let fi = state_idx(from);
        let ti = state_idx(to);
        self.transitions[fi][ti] += 1;
        self.cycle_costs[fi][ti] += cycles;
        // Update population
        if self.population[fi] > 0 {
            self.population[fi] -= 1;
        }
        self.population[ti] += 1;
        self.total_transitions += 1;

        // Update imbalance score
        self.imbalance_sum += self.instantaneous_imbalance();
        self.imbalance_samples += 1;
    }

    /// Flux rate for a specific transition (transitions per observation)
    pub fn flux_rate(&self, from: &SentronState, to: &SentronState) -> u64 {
        self.transitions[state_idx(from)][state_idx(to)]
    }

    /// Average cycle cost for a specific transition
    pub fn avg_transition_cost(&self, from: &SentronState, to: &SentronState) -> f64 {
        let fi = state_idx(from);
        let ti = state_idx(to);
        if self.transitions[fi][ti] == 0 { return 0.0; }
        self.cycle_costs[fi][ti] as f64 / self.transitions[fi][ti] as f64
    }

    /// Current population per state
    pub fn population(&self, state: &SentronState) -> u64 {
        self.population[state_idx(state)]
    }

    /// Instantaneous imbalance: how far from even distribution
    /// 0.0 = perfectly balanced, 1.0 = all sentrons in one state
    fn instantaneous_imbalance(&self) -> f64 {
        let total = self.population.iter().sum::<u64>() as f64;
        if total == 0.0 { return 0.0; }
        let ideal = total / 4.0;
        let deviation: f64 = self.population.iter()
            .map(|&p| (p as f64 - ideal).abs())
            .sum();
        deviation / (2.0 * total) // Normalized to 0..1
    }

    /// Average imbalance over time
    pub fn avg_imbalance(&self) -> f64 {
        if self.imbalance_samples == 0 { return 0.0; }
        self.imbalance_sum / self.imbalance_samples as f64
    }

    /// Breathing ratio: activations / retirements
    /// 1.0 = perfectly balanced breathing. >1.0 = more inhale. <1.0 = more exhale.
    pub fn breathing_ratio(&self) -> f64 {
        let activations = self.transitions[0][1]; // Dormant → Running
        let retirements = self.transitions[1][3] + self.transitions[2][3]; // Running/Waiting → Retired
        if retirements == 0 { return f64::INFINITY; }
        activations as f64 / retirements as f64
    }

    /// Recycle rate: Retired → Dormant transitions
    pub fn recycle_rate(&self) -> u64 {
        self.transitions[3][0] // Retired → Dormant
    }

    /// Bottleneck detection: which state has the most accumulation?
    pub fn bottleneck(&self) -> SentronState {
        let max_idx = self.population.iter()
            .enumerate()
            .max_by_key(|(_, &v)| v)
            .map(|(i, _)| i)
            .unwrap_or(0);
        match max_idx {
            0 => SentronState::Dormant,
            1 => SentronState::Running,
            2 => SentronState::Waiting,
            _ => SentronState::Retired,
        }
    }

    /// Total transitions observed
    pub fn total_transitions(&self) -> u64 { self.total_transitions }

    /// Alignment score: composite metric
    /// 1.0 = perfect (balanced breathing, low imbalance, fast transitions)
    /// Lower = worse alignment
    pub fn alignment_score(&self) -> f64 {
        let breathing = self.breathing_ratio();
        let breathing_score = if breathing.is_infinite() { 0.0 }
            else { 1.0 - (breathing - 1.0).abs().min(1.0) };

        let imbalance_score = 1.0 - self.avg_imbalance().min(1.0);

        // Recycle health: are retired sentrons being recycled?
        let retired_pop = self.population[3] as f64;
        let recycles = self.transitions[3][0] as f64;
        let recycle_score = if retired_pop + recycles == 0.0 { 1.0 }
            else { recycles / (retired_pop + recycles) };

        (breathing_score + imbalance_score + recycle_score) / 3.0
    }

    /// Reset all stats
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enter_and_population() {
        let mut flux = FluxAnalyzer::new();
        flux.enter(&SentronState::Dormant);
        flux.enter(&SentronState::Dormant);
        flux.enter(&SentronState::Running);

        assert_eq!(flux.population(&SentronState::Dormant), 2);
        assert_eq!(flux.population(&SentronState::Running), 1);
    }

    #[test]
    fn test_transition_tracking() {
        let mut flux = FluxAnalyzer::new();
        flux.enter(&SentronState::Dormant);
        flux.transition(&SentronState::Dormant, &SentronState::Running, 10);

        assert_eq!(flux.flux_rate(&SentronState::Dormant, &SentronState::Running), 1);
        assert_eq!(flux.population(&SentronState::Dormant), 0);
        assert_eq!(flux.population(&SentronState::Running), 1);
    }

    #[test]
    fn test_avg_transition_cost() {
        let mut flux = FluxAnalyzer::new();
        for _ in 0..3 { flux.enter(&SentronState::Dormant); }
        flux.transition(&SentronState::Dormant, &SentronState::Running, 10);
        flux.transition(&SentronState::Dormant, &SentronState::Running, 20);
        flux.transition(&SentronState::Dormant, &SentronState::Running, 30);

        let avg = flux.avg_transition_cost(&SentronState::Dormant, &SentronState::Running);
        assert!((avg - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_breathing_ratio_balanced() {
        let mut flux = FluxAnalyzer::new();
        for _ in 0..10 { flux.enter(&SentronState::Dormant); }

        // 5 activations
        for _ in 0..5 {
            flux.transition(&SentronState::Dormant, &SentronState::Running, 5);
        }
        // 5 retirements
        for _ in 0..5 {
            flux.transition(&SentronState::Running, &SentronState::Retired, 5);
        }

        assert!((flux.breathing_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut flux = FluxAnalyzer::new();
        for _ in 0..10 { flux.enter(&SentronState::Waiting); }
        for _ in 0..2 { flux.enter(&SentronState::Running); }

        assert_eq!(flux.bottleneck(), SentronState::Waiting);
    }

    #[test]
    fn test_full_lifecycle_flux() {
        let mut flux = FluxAnalyzer::new();
        // Simulate 100 sentrons through full lifecycle
        for _ in 0..100 {
            flux.enter(&SentronState::Dormant);
            flux.transition(&SentronState::Dormant, &SentronState::Running, 5);
            flux.transition(&SentronState::Running, &SentronState::Waiting, 50);
            flux.transition(&SentronState::Waiting, &SentronState::Retired, 10);
            flux.transition(&SentronState::Retired, &SentronState::Dormant, 2);
        }

        assert_eq!(flux.total_transitions(), 400);
        assert_eq!(flux.recycle_rate(), 100);
        assert!(flux.alignment_score() > 0.5);
    }

    #[test]
    fn test_alignment_score_perfect() {
        let mut flux = FluxAnalyzer::new();
        // Balanced: equal population, 1:1 breathing, full recycling
        for _ in 0..40 {
            flux.enter(&SentronState::Dormant);
            flux.transition(&SentronState::Dormant, &SentronState::Running, 5);
            flux.transition(&SentronState::Running, &SentronState::Retired, 5);
            flux.transition(&SentronState::Retired, &SentronState::Dormant, 2);
        }

        let score = flux.alignment_score();
        assert!(score > 0.6, "Alignment score {} should be > 0.6 for balanced flow", score);
    }

    #[test]
    fn test_empty_flux() {
        let flux = FluxAnalyzer::new();
        assert_eq!(flux.total_transitions(), 0);
        assert_eq!(flux.avg_imbalance(), 0.0);
    }

    #[test]
    fn test_imbalanced_system() {
        let mut flux = FluxAnalyzer::new();
        // All sentrons stuck in Running — no exhale
        for _ in 0..100 {
            flux.enter(&SentronState::Dormant);
            flux.transition(&SentronState::Dormant, &SentronState::Running, 5);
        }
        // No retirements → breathing ratio = infinity
        assert!(flux.breathing_ratio().is_infinite());
        // Bottleneck should be Running
        assert_eq!(flux.bottleneck(), SentronState::Running);
    }
}
