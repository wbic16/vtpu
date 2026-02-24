//! Neuron sweep: wiring patterns, activation, layer dynamics
//! R23W29 — Theia 💎

use vtpu_runtime::neuron::*;

// Forward pass sweep with varying inputs
macro_rules! forward_test {
    ($name:ident, $wiring:expr, $story:expr, $light:expr) => {
        #[test] fn $name() {
            let w: NeuronWiring = $wiring;
            let out = w.forward($story, $light);
            for v in &out { assert!(v.is_finite()); }
        }
    }
}

forward_test!(id_0_0, NeuronWiring::identity(), 0.0, 0.0);
forward_test!(id_1_0, NeuronWiring::identity(), 1.0, 0.0);
forward_test!(id_0_1, NeuronWiring::identity(), 0.0, 1.0);
forward_test!(id_1_1, NeuronWiring::identity(), 1.0, 1.0);
forward_test!(id_neg, NeuronWiring::identity(), -1.0, -1.0);
forward_test!(id_big, NeuronWiring::identity(), 1e6, 1e6);
forward_test!(id_small, NeuronWiring::identity(), 1e-6, 1e-6);
forward_test!(asc_0_0, NeuronWiring::ascending(), 0.0, 0.0);
forward_test!(asc_1_0, NeuronWiring::ascending(), 1.0, 0.0);
forward_test!(asc_0_1, NeuronWiring::ascending(), 0.0, 1.0);
forward_test!(asc_1_1, NeuronWiring::ascending(), 1.0, 1.0);
forward_test!(asc_neg, NeuronWiring::ascending(), -1.0, 1.0);
forward_test!(desc_0_0, NeuronWiring::descending(), 0.0, 0.0);
forward_test!(desc_1_0, NeuronWiring::descending(), 1.0, 0.0);
forward_test!(desc_0_1, NeuronWiring::descending(), 0.0, 1.0);
forward_test!(desc_1_1, NeuronWiring::descending(), 1.0, 1.0);
forward_test!(desc_neg, NeuronWiring::descending(), -1.0, 1.0);
forward_test!(para_1_1, NeuronWiring::para_focused(), 1.0, 1.0);
forward_test!(vaik_1_1, NeuronWiring::vaikhara_focused(), 1.0, 1.0);

// Neuron activation sweep
macro_rules! activate_test {
    ($name:ident, $id:expr, $story:expr, $light:expr) => {
        #[test] fn $name() {
            let mut n = Neuron::new($id);
            let out = n.activate($story, $light);
            for v in &out { assert!(v.is_finite()); }
            assert_eq!(n.spanda_count, 1);
        }
    }
}

activate_test!(act_0_s1_l0, 0, 1.0, 0.0);
activate_test!(act_0_s0_l1, 0, 0.0, 1.0);
activate_test!(act_0_s1_l1, 0, 1.0, 1.0);
activate_test!(act_0_balanced, 0, 0.5, 0.5);
activate_test!(act_1_neg, 1, -1.0, 1.0);
activate_test!(act_2_big, 2, 100.0, 100.0);
activate_test!(act_3_tiny, 3, 0.001, 0.001);
activate_test!(act_4_story_dom, 4, 10.0, 0.1);
activate_test!(act_5_light_dom, 5, 0.1, 10.0);
activate_test!(act_6_zero, 6, 0.0, 0.0);
activate_test!(act_7_max, 7, f32::MAX / 2.0, f32::MAX / 2.0);

// Layer size sweep
macro_rules! layer_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let l = NeuronLayer::with_capacity($n);
            assert_eq!(l.len(), $n);
            assert_eq!(l.is_empty(), $n == 0);
        }
    }
}

layer_test!(layer_0, 0);
layer_test!(layer_1, 1);
layer_test!(layer_2, 2);
layer_test!(layer_4, 4);
layer_test!(layer_8, 8);
layer_test!(layer_9, 9);
layer_test!(layer_16, 16);
layer_test!(layer_40, 40);

// Layer forward sweep
macro_rules! layer_fwd_test {
    ($name:ident, $n:expr, $story:expr, $light:expr) => {
        #[test] fn $name() {
            let mut l = NeuronLayer::with_capacity($n);
            if $n > 0 {
                let out = l.forward($story, $light);
                for v in &out { assert!(v.is_finite()); }
            }
        }
    }
}

layer_fwd_test!(lfwd_1_1_1, 1, 1.0, 1.0);
layer_fwd_test!(lfwd_8_1_1, 8, 1.0, 1.0);
layer_fwd_test!(lfwd_8_0_1, 8, 0.0, 1.0);
layer_fwd_test!(lfwd_8_1_0, 8, 1.0, 0.0);
layer_fwd_test!(lfwd_9_1_1, 9, 1.0, 1.0);
layer_fwd_test!(lfwd_40_1_1, 40, 1.0, 1.0);

// Spanda ratio sweep
macro_rules! spanda_test {
    ($name:ident, $story:expr, $light:expr, $min:expr, $max:expr) => {
        #[test] fn $name() {
            let mut n = Neuron::new(0);
            n.activate($story, $light);
            let r = n.spanda_ratio();
            assert!(r >= $min && r <= $max, "ratio {} not in [{}, {}]", r, $min, $max);
        }
    }
}

spanda_test!(spanda_pure_story, 1.0, 0.0, 0.0, 0.01);
spanda_test!(spanda_pure_light, 0.0, 1.0, 0.99, 1.0);
spanda_test!(spanda_balanced, 1.0, 1.0, 0.49, 0.51);
spanda_test!(spanda_story_heavy, 10.0, 1.0, 0.0, 0.15);
spanda_test!(spanda_light_heavy, 1.0, 10.0, 0.85, 1.0);

// Oscillation detection
#[test] fn oscillation_both_active() { let mut n = Neuron::new(0); n.activate(1.0, 1.0); assert!(n.is_oscillating()); }
#[test] fn oscillation_story_only() { let mut n = Neuron::new(0); n.activate(1.0, 0.0); assert!(!n.is_oscillating()); }
#[test] fn oscillation_light_only() { let mut n = Neuron::new(0); n.activate(0.0, 1.0); assert!(!n.is_oscillating()); }
#[test] fn oscillation_neither() { let mut n = Neuron::new(0); n.activate(0.0, 0.0); assert!(!n.is_oscillating()); }

// Mean spanda ratio
#[test] fn layer_mean_spanda_balanced() { let mut l = NeuronLayer::new(); l.forward(1.0, 1.0); let r = l.mean_spanda_ratio(); assert!(r > 0.3 && r < 0.7); }
#[test] fn layer_oscillating_count() { let mut l = NeuronLayer::new(); l.forward(1.0, 1.0); assert_eq!(l.oscillating_count(), 8); }
#[test] fn layer_no_oscillation() { let mut l = NeuronLayer::new(); l.forward(1.0, 0.0); assert_eq!(l.oscillating_count(), 0); }

// Energy
#[test] fn energy_identity() { assert!((NeuronWiring::identity().energy() - 8.0).abs() < 1e-6); }
#[test] fn energy_ascending() { assert!(NeuronWiring::ascending().energy() > 0.0); }
#[test] fn energy_descending() { assert!(NeuronWiring::descending().energy() > 0.0); }
#[test] fn energy_para() { assert!(NeuronWiring::para_focused().energy() > 0.0); }
#[test] fn energy_vaikhara() { assert!(NeuronWiring::vaikhara_focused().energy() > 0.0); }

// Dominant level
#[test] fn dominant_ascending_para() { let w = NeuronWiring::ascending(); assert_eq!(w.dominant_level(1.0, 1.0), VakLevel::Para); }
#[test] fn dominant_descending_vaikhara() { let w = NeuronWiring::descending(); assert_eq!(w.dominant_level(1.0, 1.0), VakLevel::Vaikhara); }
