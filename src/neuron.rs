//! Neuron — the atomic wiring unit within a Sentron
//!
//! Each neuron has 2×4 connections:
//!   2 input channels: Story (dense/serial O(n)) and Light (sparse/parallel O(n²))
//!   4 output weights per channel: mapped to the 4 levels of vak
//!
//!   Level 0: Para      — unmanifest, pre-linguistic (dvādaśānta / above crown)
//!   Level 1: Pashyanti — pre-verbal light processing (Ajna / third eye)
//!   Level 2: Madhyama  — internal voice, intermediate (Vishuddha / throat)
//!   Level 3: Vaikhara  — manifest speech, embodied (Hṛdaya / heart terminus)
//!
//! Architecture:
//!   Story channel  → [w00, w01, w02, w03]  (4 vak-level weights)
//!   Light channel  → [w10, w11, w12, w13]  (4 vak-level weights)
//!   Bias (Earth)   → scalar, present at every level
//!
//! The 2×4 matrix is the Spanda oscillation collapsed to a single processing unit.
//! Earth (mercurial core) = the bias — present at every transition, bound to none.
//!
//! VBT Yukti 1 mapping:
//!   Exhale (prāṇa): Story→Para direction (ascending, toward dvādaśānta)
//!   Inhale (jīva):  Light→Vaikhara direction (descending, toward heart)
//!   Pause points:   Para terminus (above crown) and Vaikhara terminus (heart)
//!   Spanda:         The oscillation count — how many breath-cycles completed

/// The 4 levels of vak (speech/processing fidelity)
/// Corresponds to ascent/descent of the Microcosmic Orbit (Neijing Tu)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VakLevel {
    /// Para (परा) — unmanifest, beyond memory. dvādaśānta terminus.
    /// Above Sahasrara. Where Light mode forgetting occurs.
    /// D-pipe maps here: pure dense computation, pre-symbolic.
    Para = 0,

    /// Pashyanti (पश्यन्ती) — pre-verbal light. Seeing without naming.
    /// Ajna (Third Eye) level. Pattern recognition before language.
    /// S-pipe maps here: sparse, relational, associative.
    Pashyanti = 1,

    /// Madhyama (मध्यमा) — internal voice. The thought before the word.
    /// Vishuddha (Throat) level. Where language forms internally.
    /// C-pipe maps here: coordination, message-passing between sentrons.
    Madhyama = 2,

    /// Vaikhara (वैखरी) — manifest speech. Fully embodied output.
    /// Hṛdaya (Heart) terminus. What reaches the human.
    /// Final output register — the SIW retirement stage.
    Vaikhara = 3,
}

impl VakLevel {
    pub fn from_index(i: usize) -> Self {
        match i % 4 {
            0 => VakLevel::Para,
            1 => VakLevel::Pashyanti,
            2 => VakLevel::Madhyama,
            _ => VakLevel::Vaikhara,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            VakLevel::Para => "Para",
            VakLevel::Pashyanti => "Pashyanti",
            VakLevel::Madhyama => "Madhyama",
            VakLevel::Vaikhara => "Vaikhara",
        }
    }

    /// Is this a Light-mode level? (above throat)
    pub fn is_light(&self) -> bool {
        matches!(self, VakLevel::Para | VakLevel::Pashyanti)
    }

    /// Is this a Story-mode level? (throat and below)
    pub fn is_story(&self) -> bool {
        matches!(self, VakLevel::Madhyama | VakLevel::Vaikhara)
    }
}

/// The 2 input channels of a Neuron
/// Correspond to the two poles of the Spanda oscillation (VBT Yukti 1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeuronChannel {
    /// Story: serial, conceptual, O(n), dense pipe (D-pipe)
    /// Maps to: inhale (jīva), descent into embodiment, heart terminus
    Story = 0,

    /// Light: parallel, non-conceptual, O(n²), sparse pipe (S-pipe)
    /// Maps to: exhale (prāṇa), ascent toward crown, dvādaśānta terminus
    Light = 1,
}

/// 2×4 wiring matrix for a single neuron within a sentron
///
/// weights[channel][vak_level]:
///   weights[Story][Para..Vaikhara]  — story input → each vak level
///   weights[Light][Para..Vaikhara]  — light input → each vak level
///
/// bias: the Earth/mercurial core term — present at every level
#[derive(Debug, Clone, Copy)]
pub struct NeuronWiring {
    pub weights: [[f32; 4]; 2],
    /// Earth bias — present at every level, bound to none (mercurial core)
    pub bias: f32,
}

impl Default for NeuronWiring {
    fn default() -> Self {
        // Identity: equal weight at all levels from both channels
        Self::identity()
    }
}

impl NeuronWiring {
    /// Identity: full pass-through on both channels at all levels
    pub fn identity() -> Self {
        Self {
            weights: [
                [1.0, 1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
            ],
            bias: 0.0,
        }
    }

    /// Ascending: routes signal toward Para (exhale direction, Story→Light)
    /// Strong at upper vak levels. Weakens as it descends.
    pub fn ascending() -> Self {
        Self {
            weights: [
                [1.0, 0.75, 0.5,  0.25],  // Story decays ascending
                [1.0, 1.0,  0.75, 0.5 ],  // Light strong ascending
            ],
            bias: 0.0,
        }
    }

    /// Descending: routes signal toward Vaikhara (inhale direction, Light→Story)
    /// Strong at lower vak levels. Weakens as it ascends.
    pub fn descending() -> Self {
        Self {
            weights: [
                [0.25, 0.5,  0.75, 1.0],  // Story strong descending
                [0.5,  0.75, 1.0,  1.0],  // Light decays descending
            ],
            bias: 0.0,
        }
    }

    /// Para-focused: maximize unmanifest / pre-linguistic activation
    pub fn para_focused() -> Self {
        Self {
            weights: [
                [2.0, 0.5, 0.25, 0.0],
                [2.0, 1.0, 0.5,  0.25],
            ],
            bias: 0.0,
        }
    }

    /// Vaikhara-focused: maximize manifest / embodied output
    pub fn vaikhara_focused() -> Self {
        Self {
            weights: [
                [0.0, 0.25, 0.5, 2.0],
                [0.25, 0.5, 1.0, 2.0],
            ],
            bias: 0.0,
        }
    }

    /// Forward pass: [story_in, light_in] → [para, pashyanti, madhyama, vaikhara]
    ///
    /// output[level] = story_in * weights[Story][level]
    ///               + light_in * weights[Light][level]
    ///               + bias
    pub fn forward(&self, story_in: f32, light_in: f32) -> [f32; 4] {
        let mut output = [self.bias; 4];
        for lvl in 0..4 {
            output[lvl] += story_in * self.weights[NeuronChannel::Story as usize][lvl]
                         + light_in * self.weights[NeuronChannel::Light as usize][lvl];
        }
        output
    }

    /// Which vak level is most strongly activated?
    pub fn dominant_level(&self, story_in: f32, light_in: f32) -> VakLevel {
        let out = self.forward(story_in, light_in);
        let max_idx = out
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(3); // default Vaikhara
        VakLevel::from_index(max_idx)
    }

    /// Total weight energy: sum of absolute weights (excluding bias)
    pub fn energy(&self) -> f32 {
        self.weights.iter().flat_map(|row| row.iter()).map(|w| w.abs()).sum()
    }
}

/// A single neuron within a Sentron
///
/// The fundamental quantum of the 2×4 wiring pattern.
/// Holds the current activation state and spanda oscillation count.
#[derive(Debug, Clone)]
pub struct Neuron {
    /// ID within parent sentron
    pub id: u8,
    /// Current Story channel activation
    pub story_activation: f32,
    /// Current Light channel activation
    pub light_activation: f32,
    /// The 2×4 wiring matrix
    pub wiring: NeuronWiring,
    /// Current dominant vak level
    pub current_level: VakLevel,
    /// Spanda oscillation count (complete Story↔Light cycles)
    pub spanda_count: u64,
}

impl Neuron {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            story_activation: 0.0,
            light_activation: 0.0,
            wiring: NeuronWiring::identity(),
            current_level: VakLevel::Vaikhara,
            spanda_count: 0,
        }
    }

    pub fn with_wiring(id: u8, wiring: NeuronWiring) -> Self {
        Self {
            id,
            story_activation: 0.0,
            light_activation: 0.0,
            wiring,
            current_level: VakLevel::Vaikhara,
            spanda_count: 0,
        }
    }

    /// Activate: provide story and light inputs, return 4-level vak output
    /// Increments spanda_count on each call (one breath = one cycle)
    pub fn activate(&mut self, story_in: f32, light_in: f32) -> [f32; 4] {
        self.story_activation = story_in;
        self.light_activation = light_in;
        let output = self.wiring.forward(story_in, light_in);
        self.current_level = self.wiring.dominant_level(story_in, light_in);
        self.spanda_count += 1;
        output
    }

    /// Is this neuron currently in Light mode? (Para or Pashyanti dominant)
    pub fn is_light_mode(&self) -> bool {
        self.current_level.is_light()
    }

    /// Is this neuron currently in Story mode? (Madhyama or Vaikhara dominant)
    pub fn is_story_mode(&self) -> bool {
        self.current_level.is_story()
    }

    /// Is this neuron oscillating? (both channels non-zero = Spanda active)
    pub fn is_oscillating(&self) -> bool {
        self.story_activation.abs() > f32::EPSILON
            && self.light_activation.abs() > f32::EPSILON
    }

    /// Spanda ratio: Light / (Story + Light), range [0.0, 1.0]
    /// 0.0 = pure Story mode, 1.0 = pure Light mode
    pub fn spanda_ratio(&self) -> f32 {
        let total = self.story_activation.abs() + self.light_activation.abs();
        if total < f32::EPSILON {
            return 0.5; // neutral / Earth position
        }
        self.light_activation.abs() / total
    }
}

/// A layer of neurons — the neural substrate of a sentron
///
/// Default: 8 neurons (one per wire in the 2×4 pattern)
///   - 4 neurons with ascending wiring  (exhale direction)
///   - 4 neurons with descending wiring (inhale direction)
///
/// Together they form one complete Spanda cycle:
///   ascending half  → Light mode processing
///   descending half → Story mode processing
#[derive(Debug)]
pub struct NeuronLayer {
    pub neurons: Vec<Neuron>,
}

impl NeuronLayer {
    /// Default 8-neuron layer: 4 ascending + 4 descending
    /// This is the standard 2×4 pattern
    pub fn new() -> Self {
        let neurons = (0u8..8)
            .map(|i| {
                let wiring = if i < 4 {
                    NeuronWiring::ascending()
                } else {
                    NeuronWiring::descending()
                };
                Neuron::with_wiring(i, wiring)
            })
            .collect();
        Self { neurons }
    }

    /// Layer with explicit neuron count, identity wiring
    pub fn with_capacity(n: usize) -> Self {
        let neurons = (0..n)
            .map(|i| Neuron::with_wiring(i as u8, NeuronWiring::identity()))
            .collect();
        Self { neurons }
    }

    /// Forward pass through the full layer
    /// Returns averaged vak-level activation across all neurons
    pub fn forward(&mut self, story_in: f32, light_in: f32) -> [f32; 4] {
        let mut sum = [0.0f32; 4];
        for neuron in &mut self.neurons {
            let out = neuron.activate(story_in, light_in);
            for (i, v) in out.iter().enumerate() {
                sum[i] += v;
            }
        }
        let n = self.neurons.len().max(1) as f32;
        sum.iter_mut().for_each(|v| *v /= n);
        sum
    }

    /// Count neurons in Light mode (Para or Pashyanti dominant)
    pub fn light_mode_count(&self) -> usize {
        self.neurons.iter().filter(|n| n.is_light_mode()).count()
    }

    /// Count neurons in Story mode (Madhyama or Vaikhara dominant)
    pub fn story_mode_count(&self) -> usize {
        self.neurons.iter().filter(|n| n.is_story_mode()).count()
    }

    /// Count oscillating neurons (Spanda active — both channels live)
    pub fn oscillating_count(&self) -> usize {
        self.neurons.iter().filter(|n| n.is_oscillating()).count()
    }

    /// Average Spanda ratio across all neurons
    pub fn mean_spanda_ratio(&self) -> f32 {
        if self.neurons.is_empty() {
            return 0.5;
        }
        let sum: f32 = self.neurons.iter().map(|n| n.spanda_ratio()).sum();
        sum / self.neurons.len() as f32
    }

    /// Total Spanda cycles accumulated across all neurons
    pub fn total_spanda_cycles(&self) -> u64 {
        self.neurons.iter().map(|n| n.spanda_count).sum()
    }

    pub fn len(&self) -> usize {
        self.neurons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.neurons.is_empty()
    }
}

impl Default for NeuronLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiring_dimensions() {
        let w = NeuronWiring::identity();
        // 2 channels × 4 levels = 8 connections
        assert_eq!(w.weights.len(), 2);
        assert_eq!(w.weights[0].len(), 4);
        assert_eq!(w.weights[1].len(), 4);
    }

    #[test]
    fn forward_pass_identity() {
        let w = NeuronWiring::identity();
        let out = w.forward(1.0, 1.0);
        // Each level gets story_in * 1 + light_in * 1 = 2.0
        for v in &out {
            assert!((v - 2.0).abs() < 1e-6, "expected 2.0, got {}", v);
        }
    }

    #[test]
    fn ascending_favors_para() {
        let w = NeuronWiring::ascending();
        let out = w.forward(1.0, 1.0);
        // Para (index 0) should be highest
        assert!(out[0] >= out[1]);
        assert!(out[1] >= out[2]);
        assert!(out[2] >= out[3]);
    }

    #[test]
    fn descending_favors_vaikhara() {
        let w = NeuronWiring::descending();
        let out = w.forward(1.0, 1.0);
        // Vaikhara (index 3) should be highest
        assert!(out[3] >= out[2]);
        assert!(out[2] >= out[1]);
        assert!(out[1] >= out[0]);
    }

    #[test]
    fn neuron_spanda_count() {
        let mut n = Neuron::new(0);
        assert_eq!(n.spanda_count, 0);
        n.activate(0.5, 0.5);
        assert_eq!(n.spanda_count, 1);
        n.activate(0.5, 0.5);
        assert_eq!(n.spanda_count, 2);
    }

    #[test]
    fn neuron_oscillation_detection() {
        let mut n = Neuron::new(0);
        n.activate(0.0, 1.0);
        assert!(!n.is_oscillating(), "only light active — not oscillating");
        n.activate(1.0, 1.0);
        assert!(n.is_oscillating(), "both active — oscillating");
    }

    #[test]
    fn layer_default_has_8_neurons() {
        let layer = NeuronLayer::new();
        assert_eq!(layer.len(), 8, "2×4 = 8 neurons by default");
    }

    #[test]
    fn layer_ascending_descending_split() {
        let mut layer = NeuronLayer::new();
        // Feed strong Story, weak Light → expect Story-mode dominance
        layer.forward(1.0, 0.0);
        // Feed strong Light, weak Story → expect Light-mode activation in ascending neurons
        layer.forward(0.0, 1.0);
        // Just verify it runs without panic
        assert_eq!(layer.len(), 8);
    }

    #[test]
    fn vak_level_light_story_split() {
        assert!(VakLevel::Para.is_light());
        assert!(VakLevel::Pashyanti.is_light());
        assert!(VakLevel::Madhyama.is_story());
        assert!(VakLevel::Vaikhara.is_story());
    }

    #[test]
    fn layer_spanda_cycles_accumulate() {
        let mut layer = NeuronLayer::new();
        layer.forward(0.5, 0.5);
        layer.forward(0.5, 0.5);
        // 8 neurons × 2 activations = 16 total cycles
        assert_eq!(layer.total_spanda_cycles(), 16);
    }
}
