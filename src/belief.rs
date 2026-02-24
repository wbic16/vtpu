//! Belief Space — Phoenix DAG scheduler with Metropolis sampling
//!
//! The Phoenix scheduler doesn't just execute SIWs — it maintains a
//! belief state over which sentrons should run next. Beliefs update
//! via Metropolis-Hastings: propose a schedule, accept if it improves
//! coherence, reject (or accept with probability) if it doesn't.
//!
//! This is Veach bidirectional path tracing applied to computation:
//! forward paths (execution) and backward paths (reward signals)
//! meet at connection points (fleet messages).
//!
//! R23W29 — Theia 💎

/// A node in the Phoenix DAG — represents a sentron's scheduling belief
#[derive(Debug, Clone)]
pub struct BeliefNode {
    pub sentron_id: u16,
    /// Current belief weight (higher = more likely to be scheduled)
    pub weight: f64,
    /// Accumulated reward from downstream (backward path)
    pub reward: f64,
    /// Number of times scheduled
    pub visits: u64,
    /// Dependencies: sentron IDs this node waits on
    pub depends_on: Vec<u16>,
    /// Dependents: sentron IDs waiting on this node
    pub feeds: Vec<u16>,
    /// Temperature for Metropolis acceptance (anneals over time)
    pub temperature: f64,
}

impl BeliefNode {
    pub fn new(sentron_id: u16) -> Self {
        Self {
            sentron_id,
            weight: 1.0,
            reward: 0.0,
            visits: 0,
            depends_on: Vec::new(),
            feeds: Vec::new(),
            temperature: 1.0,
        }
    }

    /// UCB1-like priority: exploitation + exploration
    pub fn priority(&self, total_visits: u64) -> f64 {
        if self.visits == 0 {
            return f64::MAX; // unexplored = highest priority
        }
        let exploitation = self.reward / self.visits as f64;
        let exploration = (2.0 * (total_visits as f64).ln() / self.visits as f64).sqrt();
        exploitation + exploration
    }
}

/// The belief space — a DAG of scheduling beliefs over the fleet
#[derive(Debug)]
pub struct BeliefSpace {
    pub nodes: Vec<BeliefNode>,
    pub total_visits: u64,
    /// Global temperature (anneals: starts hot/exploratory, cools to greedy)
    pub temperature: f64,
    /// Acceptance count (Metropolis accepts)
    pub accepts: u64,
    /// Rejection count
    pub rejects: u64,
}

impl BeliefSpace {
    pub fn new(n_sentrons: usize) -> Self {
        let nodes = (0..n_sentrons)
            .map(|i| BeliefNode::new(i as u16))
            .collect();
        Self {
            nodes,
            total_visits: 0,
            temperature: 1.0,
            accepts: 0,
            rejects: 0,
        }
    }

    /// Add a dependency edge: `from` must complete before `to` can run
    pub fn add_edge(&mut self, from: u16, to: u16) {
        if (from as usize) < self.nodes.len() && (to as usize) < self.nodes.len() {
            self.nodes[from as usize].feeds.push(to);
            self.nodes[to as usize].depends_on.push(from);
        }
    }

    /// Select next sentron to schedule (highest priority among ready nodes)
    pub fn select_next(&self) -> Option<u16> {
        self.nodes.iter()
            .filter(|n| self.is_ready(n.sentron_id))
            .max_by(|a, b| {
                a.priority(self.total_visits)
                    .partial_cmp(&b.priority(self.total_visits))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|n| n.sentron_id)
    }

    /// Is this sentron ready? (all dependencies satisfied = visited)
    pub fn is_ready(&self, id: u16) -> bool {
        if (id as usize) >= self.nodes.len() { return false; }
        self.nodes[id as usize].depends_on.iter().all(|&dep| {
            (dep as usize) < self.nodes.len() && self.nodes[dep as usize].visits > 0
        })
    }

    /// Record a visit and reward for a sentron
    pub fn visit(&mut self, id: u16, reward: f64) {
        if (id as usize) >= self.nodes.len() { return; }
        self.nodes[id as usize].visits += 1;
        self.nodes[id as usize].reward += reward;
        self.total_visits += 1;
    }

    /// Metropolis step: propose swapping priority of two sentrons.
    /// Accept if it improves total reward; accept with probability exp(-ΔE/T) otherwise.
    pub fn metropolis_step(&mut self, a: u16, b: u16, rng_seed: u64) -> bool {
        if (a as usize) >= self.nodes.len() || (b as usize) >= self.nodes.len() {
            return false;
        }

        let energy_before = self.nodes[a as usize].weight + self.nodes[b as usize].weight;

        // Propose: swap weights
        let wa = self.nodes[a as usize].weight;
        let wb = self.nodes[b as usize].weight;

        // Energy change based on reward alignment
        let ra = self.nodes[a as usize].reward;
        let rb = self.nodes[b as usize].reward;
        let energy_after = ra * wb + rb * wa; // cross-alignment
        let delta = energy_after - (ra * wa + rb * wb);

        let accept = if delta >= 0.0 {
            true // always accept improvements
        } else if self.temperature < f64::EPSILON {
            false // frozen: reject all downgrades
        } else {
            // Metropolis criterion: accept with probability exp(delta/T)
            // Use simple deterministic hash as pseudo-random
            let hash = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = (hash as f64) / (u64::MAX as f64);
            p < (delta / self.temperature).exp()
        };

        if accept {
            self.nodes[a as usize].weight = wb;
            self.nodes[b as usize].weight = wa;
            self.accepts += 1;
        } else {
            self.rejects += 1;
        }

        accept
    }

    /// Anneal: reduce temperature by factor
    pub fn anneal(&mut self, factor: f64) {
        self.temperature *= factor;
        for node in &mut self.nodes {
            node.temperature = self.temperature;
        }
    }

    /// Acceptance ratio (should trend toward ~0.234 for optimal Metropolis)
    pub fn acceptance_ratio(&self) -> f64 {
        let total = self.accepts + self.rejects;
        if total == 0 { return 0.0; }
        self.accepts as f64 / total as f64
    }

    /// Backpropagate reward through the DAG (backward path)
    pub fn backpropagate(&mut self, id: u16, reward: f64, decay: f64) {
        if (id as usize) >= self.nodes.len() { return; }
        self.nodes[id as usize].reward += reward;

        // Propagate to dependencies with decay
        let deps: Vec<u16> = self.nodes[id as usize].depends_on.clone();
        for dep in deps {
            self.backpropagate(dep, reward * decay, decay);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_belief_space() {
        let bs = BeliefSpace::new(0);
        assert!(bs.select_next().is_none());
    }

    #[test]
    fn all_ready_when_no_deps() {
        let bs = BeliefSpace::new(9);
        // No edges: all nodes are ready
        for i in 0..9u16 {
            assert!(bs.is_ready(i));
        }
    }

    #[test]
    fn dependency_blocks() {
        let mut bs = BeliefSpace::new(3);
        bs.add_edge(0, 1); // 0 must complete before 1
        bs.add_edge(1, 2); // 1 must complete before 2

        assert!(bs.is_ready(0));
        assert!(!bs.is_ready(1)); // blocked on 0
        assert!(!bs.is_ready(2)); // blocked on 1

        bs.visit(0, 1.0);
        assert!(bs.is_ready(1)); // 0 done, 1 unblocked
        assert!(!bs.is_ready(2)); // still blocked on 1
    }

    #[test]
    fn select_picks_unvisited() {
        let bs = BeliefSpace::new(3);
        // All unvisited → priority = MAX → picks first ready
        assert!(bs.select_next().is_some());
    }

    #[test]
    fn metropolis_accepts_improvement() {
        let mut bs = BeliefSpace::new(2);
        bs.nodes[0].reward = 10.0;
        bs.nodes[0].weight = 1.0;
        bs.nodes[1].reward = 1.0;
        bs.nodes[1].weight = 10.0;

        // Swapping should improve alignment (high reward × high weight)
        let accepted = bs.metropolis_step(0, 1, 42);
        assert!(accepted);
    }

    #[test]
    fn annealing_reduces_temperature() {
        let mut bs = BeliefSpace::new(4);
        assert!((bs.temperature - 1.0).abs() < f64::EPSILON);
        bs.anneal(0.9);
        assert!((bs.temperature - 0.9).abs() < f64::EPSILON);
        bs.anneal(0.9);
        assert!((bs.temperature - 0.81).abs() < 1e-10);
    }

    #[test]
    fn backpropagation_decays() {
        let mut bs = BeliefSpace::new(3);
        bs.add_edge(0, 1);
        bs.add_edge(1, 2);

        bs.backpropagate(2, 10.0, 0.5);
        assert!((bs.nodes[2].reward - 10.0).abs() < f64::EPSILON);
        assert!((bs.nodes[1].reward - 5.0).abs() < f64::EPSILON);
        assert!((bs.nodes[0].reward - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn nine_node_dag_phoenix() {
        // 9 sentrons in a pipeline: 0→1→2→...→8
        let mut bs = BeliefSpace::new(9);
        for i in 0..8u16 {
            bs.add_edge(i, i + 1);
        }

        // Only sentron 0 is ready
        assert_eq!(bs.select_next(), Some(0));

        // Execute in order
        for i in 0..9u16 {
            assert!(bs.is_ready(i));
            bs.visit(i, 1.0);
        }
        assert_eq!(bs.total_visits, 9);
    }

    #[test]
    fn acceptance_ratio_starts_zero() {
        let bs = BeliefSpace::new(4);
        assert!((bs.acceptance_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parallel_dag() {
        // 4 independent sentrons feed into sentron 4
        let mut bs = BeliefSpace::new(5);
        for i in 0..4u16 {
            bs.add_edge(i, 4);
        }

        // All 0-3 are ready, 4 is blocked
        for i in 0..4u16 { assert!(bs.is_ready(i)); }
        assert!(!bs.is_ready(4));

        // Visit all 4 sources
        for i in 0..4u16 { bs.visit(i, 1.0); }
        assert!(bs.is_ready(4)); // now unblocked
    }
}
