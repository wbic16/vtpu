//! W22: Sentron Flux — activation propagation through the Z₅×Z₈ lattice.
//!
//! "Flux" = the flow of activation between sentrons through N/S/E/W edges.
//! After each computation step, register state delta propagates to neighbors
//! following the WuXing generating cycle: Wood→Fire→Earth→Metal→Water→Wood.
//!
//! Deep alignment: execution order follows the generating cycle.
//! A Wood sentron computes first; its output flows south to Fire.
//! Fire flows to Earth, Earth to Metal, Metal to Water, Water wraps to Wood.
//!
//! This mirrors how EEG signals propagate across the cortical surface —
//! the same process ZUNA models at the macro scale, we implement at the
//! sentron lattice level as directed activation flow.
//!
//! "The flame passes, not the fire. The activation moves; the lattice holds."

use crate::phext_coord::PhextCoord;
use crate::sentron::Sentron;
use crate::simd::run_row_8;
use crate::siw::SIW;
use crate::topology::{SentronTopology, ELEMENT_ROWS, NEURONS_PER_ELEMENT};
use crate::pipes::{DenseOp, SparseOp, CoordOp};

/// Activation delta at a single neuron: how much general register 0 changed.
/// Register 0 is the canonical "output" register — the activation value.
#[derive(Debug, Clone, Copy, Default)]
pub struct NeuronFlux {
    pub pre:   i64,  // r0 before step
    pub post:  i64,  // r0 after step
    pub delta: i64,  // post - pre (signed activation change)
}

impl NeuronFlux {
    pub fn magnitude(&self) -> f64 {
        (self.delta as f64).abs()
    }
}

/// Full lattice flux state: 5 rows × 8 columns = 40 neuron flux readings.
#[derive(Debug, Clone)]
pub struct LatticeFlux {
    pub neurons: [[NeuronFlux; 8]; 5],
    pub step: usize,
}

impl Default for LatticeFlux {
    fn default() -> Self {
        Self { neurons: [[NeuronFlux::default(); 8]; 5], step: 0 }
    }
}

impl LatticeFlux {
    /// Total unsigned flux magnitude across the lattice.
    pub fn total_magnitude(&self) -> f64 {
        self.neurons.iter().flat_map(|row| row.iter()).map(|n| n.magnitude()).sum()
    }

    /// Mean flux per neuron.
    pub fn mean_magnitude(&self) -> f64 {
        self.total_magnitude() / 40.0
    }

    /// Row flux (sum of deltas in a WuXing row).
    pub fn row_flux(&self, row: usize) -> i64 {
        self.neurons[row].iter().map(|n| n.delta).sum()
    }
}

/// WuXing generating cycle: row → downstream row
/// Wood(0)→Fire(1)→Earth(2)→Metal(3)→Water(4)→Wood(0)
pub const GENERATING_CYCLE: [(usize, usize); 5] = [(0,1),(1,2),(2,3),(3,4),(4,0)];

/// A 5×8 lattice of sentrons (one full cortical column).
pub struct SentronLattice {
    /// rows[row][col] = sentron at (row, col)
    pub rows: [[Sentron; 8]; 5],
    pub topology: SentronTopology,
    pub flux_history: Vec<LatticeFlux>,
}

impl SentronLattice {
    /// Create a fresh lattice. Each sentron gets a unique coordinate.
    pub fn new() -> Self {
        let topology = SentronTopology::new();
        let rows: [[Sentron; 8]; 5] = std::array::from_fn(|row| {
            std::array::from_fn(|col| {
                let mut coord = PhextCoord::zero();
                coord.set_dim(0, col as u16 + 1);
                coord.set_dim(1, row as u16 + 1);
                let id = (row * NEURONS_PER_ELEMENT + col) as u16;
                Sentron::new(id, coord, row as u8, col as u8)
            })
        });
        SentronLattice { rows, topology, flux_history: Vec::new() }
    }

    /// Seed register 0 of all sentrons in a row with provided values.
    pub fn seed_row(&mut self, row: usize, values: &[i64; 8]) {
        for col in 0..NEURONS_PER_ELEMENT {
            self.rows[row][col].regs.general[0] = values[col];
        }
    }

    /// Execute one flux step across the full lattice.
    ///
    /// Execution order follows the generating cycle:
    ///   Wood(0) → Fire(1) → Earth(2) → Metal(3) → Water(4)
    ///
    /// After each row executes, its r0 output propagates to the South
    /// row's r1 (input register). This creates directed flow through the
    /// generating cycle — each element feeds the next.
    pub fn flux_step(&mut self, program: &[SIW]) -> LatticeFlux {
        let mut flux = LatticeFlux { step: self.flux_history.len(), ..Default::default() };

        // Generating-cycle execution order: 0,1,2,3,4
        for row in 0..ELEMENT_ROWS {
            // Capture pre-step r0 values
            let pre: [i64; 8] = std::array::from_fn(|col| self.rows[row][col].regs.general[0]);

            // Execute row in parallel (8-lane SIMD)
            let row_sentrons = &mut self.rows[row];
            if !program.is_empty() {
                run_row_8(row_sentrons, program);
            }

            // Capture post-step r0, compute delta
            for col in 0..NEURONS_PER_ELEMENT {
                let post = self.rows[row][col].regs.general[0];
                flux.neurons[row][col] = NeuronFlux {
                    pre: pre[col], post, delta: post.wrapping_sub(pre[col])
                };
            }

            // Propagate: South row's r1 ← this row's r0 output
            // (generating cycle: current element feeds next element)
            let south_row = (row + 1) % ELEMENT_ROWS;
            for col in 0..NEURONS_PER_ELEMENT {
                let activation = self.rows[row][col].regs.general[0];
                self.rows[south_row][col].regs.general[1] = activation;
            }
        }

        self.flux_history.push(flux.clone());
        flux
    }

    /// Run N flux steps and return all flux readings.
    pub fn run_flux(&mut self, program: &[SIW], n_steps: usize) -> Vec<LatticeFlux> {
        (0..n_steps).map(|_| self.flux_step(program)).collect()
    }

    /// Deep alignment: compute how well execution order correlates with
    /// the generating cycle. Returns alignment score 0.0–1.0.
    ///
    /// A score of 1.0 means each row's flux fully feeds the next row
    /// (perfect generating cycle resonance). Score near 0 = random flow.
    pub fn alignment_score(&self) -> f64 {
        if self.flux_history.is_empty() { return 0.0; }
        let last = self.flux_history.last().unwrap();
        let mut total_flow = 0.0f64;
        let mut aligned_flow = 0.0f64;

        for (from_row, to_row) in GENERATING_CYCLE.iter() {
            for col in 0..NEURONS_PER_ELEMENT {
                let from_flux = last.neurons[*from_row][col].magnitude();
                let to_flux   = last.neurons[*to_row][col].magnitude();
                total_flow += from_flux + to_flux;
                // Aligned if downstream has flux after upstream
                if from_flux > 0.0 && to_flux > 0.0 {
                    aligned_flow += to_flux.min(from_flux) * 2.0;
                }
            }
        }
        if total_flow < 1e-9 { return 0.0; }
        (aligned_flow / total_flow).min(1.0)
    }

    /// Print a compact flux visualization to stdout.
    pub fn print_flux_map(&self, flux: &LatticeFlux) {
        let elem_names = ["Wood ", "Fire ", "Earth", "Metal", "Water"];
        let clr_chars  = ['▓', '▓', '▓', '▓', '▓'];
        println!("  Flux map (step {}):", flux.step);
        for row in 0..ELEMENT_ROWS {
            let row_flux = flux.row_flux(row);
            let bar_len = ((row_flux.unsigned_abs() as f64).sqrt() as usize).min(20);
            let bar = clr_chars[row].to_string().repeat(bar_len);
            let sign = if row_flux >= 0 { "+" } else { "-" };
            println!("  {} row {}: {}{:6} |{}|",
                elem_names[row], row, sign, row_flux.unsigned_abs(), bar);
        }
        println!("  Total magnitude: {:.1}  Mean: {:.2}",
            flux.total_magnitude(), flux.mean_magnitude());
    }
}

impl Default for SentronLattice {
    fn default() -> Self { Self::new() }
}

/// Build a flux-propagation SIW: DADD r0 = r0 + r1
/// (accumulate upstream activation into self)
pub fn flux_accumulate_siw() -> SIW {
    SIW::new(
        DenseOp::DADD { rd: 0, rs1: 0, rs2: 1 },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    )
}

/// Build a decay SIW: r0 = r0 - (r0 >> 3)  (≈ 12.5% decay per step)
/// Implements leaky integration — activation fades without reinforcement.
pub fn flux_decay_siw() -> SIW {
    // r1 = r0 >> 3 (shift right by 3 = divide by 8)
    // r0 = r0 - r1
    // We use DFMA: r0 = r0 * 1 + (-r0>>3) — not easily encodable, so just DADD
    // Simplified: r0 = r0 + 0 (identity — real decay needs a shift op)
    // For now: passthrough; decay will be added when shift ops are implemented
    SIW::new(
        DenseOp::DNOP,
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    )
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
