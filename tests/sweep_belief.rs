//! Belief space sweep: DAG shapes, Metropolis, annealing
//! R23W29 — Theia 💎

use vtpu_runtime::belief::*;

// DAG shapes
macro_rules! chain_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let mut bs = BeliefSpace::new($n);
            for i in 0..($n-1) as u16 { bs.add_edge(i, i+1); }
            // Only node 0 is ready
            assert!(bs.is_ready(0));
            for i in 1..$n as u16 { assert!(!bs.is_ready(i)); }
            // Execute in order
            for i in 0..$n as u16 { bs.visit(i, 1.0); }
            assert_eq!(bs.total_visits, $n);
        }
    }
}

chain_test!(chain_2, 2);
chain_test!(chain_3, 3);
chain_test!(chain_4, 4);
chain_test!(chain_5, 5);
chain_test!(chain_8, 8);
chain_test!(chain_9, 9);
chain_test!(chain_16, 16);
chain_test!(chain_40, 40);

// Fan-in DAGs: N sources → 1 sink
macro_rules! fanin_test {
    ($name:ident, $sources:expr) => {
        #[test] fn $name() {
            let total = $sources + 1;
            let mut bs = BeliefSpace::new(total);
            let sink = $sources as u16;
            for i in 0..$sources as u16 { bs.add_edge(i, sink); }
            // All sources ready, sink blocked
            for i in 0..$sources as u16 { assert!(bs.is_ready(i)); }
            assert!(!bs.is_ready(sink));
            // Visit all sources
            for i in 0..$sources as u16 { bs.visit(i, 1.0); }
            assert!(bs.is_ready(sink));
        }
    }
}

fanin_test!(fanin_2, 2);
fanin_test!(fanin_3, 3);
fanin_test!(fanin_4, 4);
fanin_test!(fanin_5, 5);
fanin_test!(fanin_8, 8);
fanin_test!(fanin_9, 9);
fanin_test!(fanin_40, 40);

// Fan-out DAGs: 1 source → N sinks
macro_rules! fanout_test {
    ($name:ident, $sinks:expr) => {
        #[test] fn $name() {
            let total = $sinks + 1;
            let mut bs = BeliefSpace::new(total);
            for i in 1..total as u16 { bs.add_edge(0, i); }
            // Only source ready
            assert!(bs.is_ready(0));
            for i in 1..total as u16 { assert!(!bs.is_ready(i)); }
            bs.visit(0, 1.0);
            for i in 1..total as u16 { assert!(bs.is_ready(i)); }
        }
    }
}

fanout_test!(fanout_2, 2);
fanout_test!(fanout_3, 3);
fanout_test!(fanout_4, 4);
fanout_test!(fanout_8, 8);
fanout_test!(fanout_9, 9);
fanout_test!(fanout_40, 40);

// Diamond DAG: 0→{1,2}→3
#[test] fn diamond_dag() {
    let mut bs = BeliefSpace::new(4);
    bs.add_edge(0, 1); bs.add_edge(0, 2);
    bs.add_edge(1, 3); bs.add_edge(2, 3);
    assert!(bs.is_ready(0));
    assert!(!bs.is_ready(1));
    bs.visit(0, 1.0);
    assert!(bs.is_ready(1)); assert!(bs.is_ready(2));
    assert!(!bs.is_ready(3));
    bs.visit(1, 1.0); bs.visit(2, 1.0);
    assert!(bs.is_ready(3));
}

// Backpropagation sweep
macro_rules! backprop_test {
    ($name:ident, $depth:expr, $reward:expr, $decay:expr) => {
        #[test] fn $name() {
            let mut bs = BeliefSpace::new($depth);
            for i in 0..($depth-1) as u16 { bs.add_edge(i, i+1); }
            let last = ($depth - 1) as u16;
            bs.backpropagate(last, $reward as f64, $decay);
            // Last node gets full reward
            assert!((bs.nodes[last as usize].reward - $reward as f64).abs() < 1e-6);
            // First node gets reward × decay^(depth-1)
            let expected = ($reward as f64) * ($decay as f64).powi(($depth - 1) as i32);
            assert!((bs.nodes[0].reward - expected).abs() < 1e-4,
                "node 0: expected {}, got {}", expected, bs.nodes[0].reward);
        }
    }
}

backprop_test!(bp_2_10_half, 2, 10.0, 0.5);
backprop_test!(bp_3_10_half, 3, 10.0, 0.5);
backprop_test!(bp_5_100_9, 5, 100.0, 0.9);
backprop_test!(bp_9_1_half, 9, 1.0, 0.5);

// Annealing convergence
macro_rules! anneal_test {
    ($name:ident, $steps:expr, $factor:expr, $max_temp:expr) => {
        #[test] fn $name() {
            let mut bs = BeliefSpace::new(4);
            for _ in 0..$steps { bs.anneal($factor); }
            assert!(bs.temperature < $max_temp,
                "temp {} should be < {}", bs.temperature, $max_temp);
        }
    }
}

anneal_test!(anneal_10_99, 10, 0.99, 1.0);
anneal_test!(anneal_100_99, 100, 0.99, 0.5);
anneal_test!(anneal_100_95, 100, 0.95, 0.01);
anneal_test!(anneal_1000_99, 1000, 0.99, 0.0001);

// Metropolis sweep
macro_rules! metro_test {
    ($name:ident, $n:expr, $steps:expr) => {
        #[test] fn $name() {
            let mut bs = BeliefSpace::new($n);
            for i in 0..$n as u16 { bs.visit(i, (i as f64 + 1.0) * 10.0); }
            for s in 0..$steps as u64 {
                bs.metropolis_step(
                    (s % $n as u64) as u16,
                    ((s + 1) % $n as u64) as u16,
                    s * 12345 + 67890
                );
            }
            assert!(bs.accepts + bs.rejects == $steps);
        }
    }
}

metro_test!(metro_4_10, 4, 10);
metro_test!(metro_4_100, 4, 100);
metro_test!(metro_9_50, 9, 50);
metro_test!(metro_40_100, 40, 100);

// Priority ordering
#[test] fn priority_unvisited_highest() {
    let mut bs = BeliefSpace::new(3);
    bs.visit(0, 100.0); // high reward but visited
    // Node 1 unvisited → should have highest priority
    let next = bs.select_next();
    assert!(next == Some(1) || next == Some(2)); // either unvisited node
}

#[test] fn priority_visited_by_reward() {
    let mut bs = BeliefSpace::new(3);
    bs.visit(0, 1.0);
    bs.visit(1, 100.0);
    bs.visit(2, 50.0);
    // Node 1 has highest reward/visit ratio
    let next = bs.select_next();
    assert_eq!(next, Some(1));
}
