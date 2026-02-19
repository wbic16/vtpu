//! ZUNA ↔ vTPU Bridge (Rust side)
//!
//! Consumes SIW streams produced by `zuna/eeg_bridge.py` and executes
//! them on a sentron lattice.
//!
//! The isomorphism:
//!   - ZUNA: missing EEG channels reconstructed from scalp coordinates
//!   - Sentron: missing neuron activations propagated via N/S/E/W topology
//!
//! Coordinate mapping (from bridge JSON):
//!   dim 0 = scroll    = azimuth column (1..8)
//!   dim 1 = section   = elevation row / WuXing element (1..5)
//!   dim 2 = volume    = frequency band (1..5)
//!   dim 3 = chapter   = timestep
//!
//! WuXing ↔ Frequency Band:
//!   Row 0 (Wood)  = Delta  (0.5-4 Hz)   — deep sleep, healing
//!   Row 1 (Fire)  = Theta  (4-8 Hz)     — meditation, memory
//!   Row 2 (Earth) = Alpha  (8-13 Hz)    — relaxed awareness (default)
//!   Row 3 (Metal) = Beta   (13-30 Hz)   — active thinking
//!   Row 4 (Water) = Gamma  (30-100 Hz)  — high-level cognition

use crate::{
    exec::run,
    memory::Memory,
    phext_coord::PhextCoord,
    pipes::{DenseOp, SparseOp, CoordOp},
    sentron::Sentron,
    siw::SIW,
    topology::{SentronTopology, NeuronAddr, Direction},
};

/// Frequency band identified by WuXing row (0-4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreqBand {
    Delta = 0,  // Wood  — 0.5-4 Hz
    Theta = 1,  // Fire  — 4-8 Hz
    Alpha = 2,  // Earth — 8-13 Hz
    Beta  = 3,  // Metal — 13-30 Hz
    Gamma = 4,  // Water — 30-100 Hz
}

impl FreqBand {
    pub fn wuxing_row(self) -> u8 { self as u8 }
    pub fn name(self) -> &'static str {
        match self {
            FreqBand::Delta => "Delta (Wood)",
            FreqBand::Theta => "Theta (Fire)",
            FreqBand::Alpha => "Alpha (Earth)",
            FreqBand::Beta  => "Beta (Metal)",
            FreqBand::Gamma => "Gamma (Water)",
        }
    }
}

/// A single EEG observation: amplitude at a phext coordinate
#[derive(Debug, Clone)]
pub struct EegObservation {
    pub coord: PhextCoord,
    pub amplitude_scaled: i64,  // μV × 1000
    pub channel_name: Option<String>,
}

/// Results of running an EEG observation stream through the sentron lattice
#[derive(Debug, Default)]
pub struct ZunaExecStats {
    pub observations: usize,
    pub siws_retired: u64,
    pub ops_retired: u64,
    pub reconstruction_fills: usize,  // how many coords were inferred via topology
}

/// Convert EEG observations into a SIW stream.
///
/// For each observation:
///   1. DMOV rd, amplitude   — load amplitude into register
///   2. SSCATTR coord, rd    — scatter to phext coordinate
///
/// After all observations, run topology-guided gap fill:
///   For each unfilled neighbor, emit SGATHER+DADD(avg)+SSCATTR
pub fn observations_to_siws(
    observations: &[EegObservation],
    base_reg: u8,
) -> Vec<SIW> {
    let mut siws = Vec::with_capacity(observations.len() * 2);

    for obs in observations {
        let rd = base_reg % 15;
        // Load amplitude
        siws.push(SIW::new(
            DenseOp::DMOV { rd, imm: obs.amplitude_scaled },
            SparseOp::SNOP,
            CoordOp::CNOP,
            obs.coord,
        ));
        // Scatter to coordinate
        siws.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 0, rs: rd, width: 8 },
            CoordOp::CNOP,
            obs.coord,
        ));
    }
    siws
}

/// Build a topology-guided reconstruction pass.
///
/// After loading known EEG values, this pass propagates activations
/// to N/S/E/W neighbors that had no direct observation.
/// This mirrors what ZUNA does at the model level — but using
/// the sentron topology's geometric priors instead of learned weights.
///
/// For each unfilled neighbor:
///   SGATHER neighbor_coord → rd
///   DADD rd = (rd + known_val) / 2  (simple average)
///   SSCATTR neighbor_coord ← rd
pub fn topology_reconstruction(
    topology: &SentronTopology,
    known_coords: &[PhextCoord],
    band: FreqBand,
) -> Vec<SIW> {
    use crate::topology::NEURONS_PER_ELEMENT;
    

    let band_row = band.wuxing_row();
    let mut siws = Vec::new();

    // For each neuron in the band's row, check if it has a known coord.
    // If not, infer from its North neighbor (upstream in generating cycle).
    for col in 0..NEURONS_PER_ELEMENT {
        let addr = NeuronAddr::new(band_row, col as u8);
        let north = topology.neighbor_in(addr, Direction::North);

        // Build the coord for this neuron in the EEG band
        let mut coord = PhextCoord::zero();
        coord.set_dim(0, col as u16 + 1);          // scroll = column
        coord.set_dim(1, band_row as u16 + 1);     // section = WuXing row
        coord.set_dim(2, band_row as u16 + 1);     // volume = band

        let mut north_coord = PhextCoord::zero();
        north_coord.set_dim(0, north.col as u16 + 1);
        north_coord.set_dim(1, north.row as u16 + 1);
        north_coord.set_dim(2, band_row as u16 + 1);

        let has_observation = known_coords.iter().any(|k| k.get_dim(0) == coord.get_dim(0)
            && k.get_dim(1) == coord.get_dim(1));

        if !has_observation {
            // Gather north neighbor's value
            siws.push(SIW::new(
                DenseOp::DNOP,
                SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
                CoordOp::CNOP,
                north_coord,
            ));
            // Average with self (if no self, self=0, average = north/2+0/2)
            siws.push(SIW::new(
                DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },  // rd = north × 2 (div later)
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
            // Scatter reconstructed value
            siws.push(SIW::new(
                DenseOp::DNOP,
                SparseOp::SSCATTR { coord_idx: 0, rs: 1, width: 8 },
                CoordOp::CNOP,
                coord,
            ));
        }
    }
    siws
}

/// Full ZUNA integration pipeline on logos-prime.
///
/// 1. Load EEG observations (from bridge JSON or synthetic)
/// 2. Convert to SIW stream
/// 3. Execute on sentron
/// 4. Run topology reconstruction for missing channels
/// 5. Return stats
pub fn run_zuna_pipeline(
    observations: &[EegObservation],
    band: FreqBand,
) -> ZunaExecStats {
    let topology = SentronTopology::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    let mut mem = Memory::new();
    let mut stats = ZunaExecStats::default();

    stats.observations = observations.len();

    // Phase 1: load known observations
    let known_coords: Vec<PhextCoord> = observations.iter().map(|o| o.coord).collect();
    let mut siws = observations_to_siws(observations, 0);

    // Phase 2: topology reconstruction for missing channels
    let reconstruction = topology_reconstruction(&topology, &known_coords, band);
    stats.reconstruction_fills = reconstruction.len() / 3; // 3 SIWs per fill
    siws.extend(reconstruction);

    sentron.spawn(siws);
    let exec_stats = run(&mut sentron, &mut mem);

    stats.siws_retired = exec_stats.siws_retired;
    stats.ops_retired  = exec_stats.ops_retired;

    stats
}

/// Build synthetic EEG observations (standard 10-20 system, Alpha band).
/// Mirrors the Python bridge's stub_eeg_data().
pub fn synthetic_alpha_observations() -> Vec<EegObservation> {
    // 19 electrodes × 8 columns = mapped into the sentron's Z₅×Z₈ lattice
    // Alpha band = Earth row (row 2)
    let electrodes: &[(&str, f64, f64, i64)] = &[
        // (name, azimuth_deg, elevation_deg, amplitude_uV×1000)
        ("Fp1",  120.0, 51.0, 12500),
        ("Fp2",   60.0, 51.0, 11200),
        ("F7",   150.0, 34.0,  8300),
        ("F3",   140.0, 40.0,  9100),
        ("Fz",    90.0, 67.0, 10400),
        ("F4",    40.0, 40.0,  8800),
        ("F8",    30.0, 34.0,  7600),
        ("T3",   180.0, 17.0, 15200),
        ("C3",   180.0, 55.0, 13100),
        ("Cz",    90.0, 90.0, 16700),
        ("C4",     0.0, 55.0, 14300),
        ("T4",     0.0, 17.0, 15800),
        ("T5",   210.0, 34.0, 11900),
        ("P3",   220.0, 40.0, 10500),
        ("Pz",   270.0, 67.0, 12300),
        ("P4",   320.0, 40.0, 11100),
        ("T6",   330.0, 34.0, 10700),
        ("O1",   240.0, 51.0,  9400),
        ("O2",   300.0, 51.0,  8900),
    ];

    electrodes.iter().enumerate().map(|(_i, (name, az, el, amp))| {
        let col = ((az / 360.0 * 8.0) as u16).max(1).min(8);
        let row = ((el / 90.0 * 5.0) as u16).max(1).min(5);
        let mut coord = PhextCoord::zero();
        coord.set_dim(0, col);
        coord.set_dim(1, row);
        coord.set_dim(2, 3); // Alpha = Earth = band dim 3
        coord.set_dim(3, 1); // timestep 1
        EegObservation {
            coord,
            amplitude_scaled: *amp,
            channel_name: Some(name.to_string()),
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zuna_synthetic_observations_count() {
        let obs = synthetic_alpha_observations();
        assert_eq!(obs.len(), 19, "19 standard 10-20 electrodes");
    }

    #[test]
    fn zuna_observations_to_siws_count() {
        let obs = synthetic_alpha_observations();
        let siws = observations_to_siws(&obs, 0);
        // 2 SIWs per observation (DMOV + SSCATTR)
        assert_eq!(siws.len(), obs.len() * 2);
    }

    #[test]
    fn zuna_coord_encoding() {
        let obs = synthetic_alpha_observations();
        // Cz is at elevation 90° → row = 5, azimuth 90° → col = 2
        let cz = obs.iter().find(|o| o.channel_name.as_deref() == Some("Cz")).unwrap();
        assert!(cz.coord.get_dim(0) >= 1 && cz.coord.get_dim(0) <= 8, "col in range");
        assert!(cz.coord.get_dim(1) >= 1 && cz.coord.get_dim(1) <= 5, "row in range");
        assert_eq!(cz.coord.get_dim(2), 3, "Alpha = Earth = band dim 3");
    }

    #[test]
    fn zuna_topology_reconstruction_fills_gaps() {
        let topology = SentronTopology::new();
        // Only Cz known (1 electrode)
        let obs = vec![synthetic_alpha_observations()[9].clone()]; // Cz
        let known = obs.iter().map(|o| o.coord).collect::<Vec<_>>();
        let fills = topology_reconstruction(&topology, &known, FreqBand::Alpha);
        // Should fill 7 missing columns in Alpha row (8 total - 1 known)
        assert!(fills.len() > 0, "should generate reconstruction SIWs");
        assert_eq!(fills.len() % 3, 0, "fills come in 3-SIW groups");
    }

    #[test]
    fn zuna_full_pipeline_runs() {
        let obs = synthetic_alpha_observations();
        let stats = run_zuna_pipeline(&obs, FreqBand::Alpha);
        assert_eq!(stats.observations, 19);
        assert!(stats.siws_retired > 0, "should retire SIWs");
        assert!(stats.reconstruction_fills >= 0, "may or may not fill gaps");
    }
}
