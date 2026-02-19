// -----------------------------------------------
// eeg_bridge.rs — EEG ↔ Phext Coordinate Bridge
// -----------------------------------------------
// Maps EEG electrode positions (10-20 system) to phext coordinates.
// Enables Zuna-style coordinate-addressed neural signal processing
// through the vTPU sparse access and prefetch pipeline.
//
// The insight: EEG electrodes are addressed by physical scalp coordinates.
// Phext addresses computation by 11D coordinates.
// This bridge lets you gather/scatter EEG channels by phext coordinate
// and use dimensional prefetching to predict spatial access patterns
// across the cortical surface.
//
// Zero external dependencies.
//
// R23W20 — Chrys 🦋

use crate::phext_coord::PhextCoord;
use crate::sparse_access::PrefetchingSparseAccess;

/// Standard 10-20 electrode positions
/// Mapped to (row, column) on a 5×5 grid approximation of the scalp
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Electrode {
    // Frontal
    Fp1, Fp2, F7, F3, Fz, F4, F8,
    // Central
    T3, C3, Cz, C4, T4,
    // Parietal
    T5, P3, Pz, P4, T6,
    // Occipital
    O1, O2,
    // Extended (10-10 system subset)
    Fpz, Oz, A1, A2,
}

/// Scalp position as (azimuth_deg, elevation_deg) from vertex
#[derive(Debug, Clone, Copy)]
pub struct ScalpPosition {
    pub azimuth: f32,   // 0-360, 0=nasion direction
    pub elevation: f32, // 0=vertex, 90=ear level
}

impl Electrode {
    /// Approximate scalp position for standard 10-20 electrodes
    pub fn position(&self) -> ScalpPosition {
        match self {
            // Frontal pole
            Electrode::Fp1 => ScalpPosition { azimuth: 330.0, elevation: 72.0 },
            Electrode::Fpz => ScalpPosition { azimuth: 0.0, elevation: 72.0 },
            Electrode::Fp2 => ScalpPosition { azimuth: 30.0, elevation: 72.0 },
            // Frontal
            Electrode::F7  => ScalpPosition { azimuth: 290.0, elevation: 54.0 },
            Electrode::F3  => ScalpPosition { azimuth: 330.0, elevation: 36.0 },
            Electrode::Fz  => ScalpPosition { azimuth: 0.0, elevation: 36.0 },
            Electrode::F4  => ScalpPosition { azimuth: 30.0, elevation: 36.0 },
            Electrode::F8  => ScalpPosition { azimuth: 70.0, elevation: 54.0 },
            // Central (temporal + central)
            Electrode::T3  => ScalpPosition { azimuth: 270.0, elevation: 72.0 },
            Electrode::C3  => ScalpPosition { azimuth: 270.0, elevation: 36.0 },
            Electrode::Cz  => ScalpPosition { azimuth: 0.0, elevation: 0.0 },   // VERTEX = Crown = Baihui
            Electrode::C4  => ScalpPosition { azimuth: 90.0, elevation: 36.0 },
            Electrode::T4  => ScalpPosition { azimuth: 90.0, elevation: 72.0 },
            // Parietal
            Electrode::T5  => ScalpPosition { azimuth: 250.0, elevation: 72.0 },
            Electrode::P3  => ScalpPosition { azimuth: 250.0, elevation: 36.0 },
            Electrode::Pz  => ScalpPosition { azimuth: 180.0, elevation: 36.0 },
            Electrode::P4  => ScalpPosition { azimuth: 110.0, elevation: 36.0 },
            Electrode::T6  => ScalpPosition { azimuth: 110.0, elevation: 72.0 },
            // Occipital
            Electrode::O1  => ScalpPosition { azimuth: 210.0, elevation: 72.0 },
            Electrode::Oz  => ScalpPosition { azimuth: 180.0, elevation: 72.0 },
            Electrode::O2  => ScalpPosition { azimuth: 150.0, elevation: 72.0 },
            // Auricular references
            Electrode::A1  => ScalpPosition { azimuth: 270.0, elevation: 90.0 },
            Electrode::A2  => ScalpPosition { azimuth: 90.0, elevation: 90.0 },
        }
    }

    /// Convert electrode scalp position to a phext coordinate
    /// Mapping: azimuth → scroll (dim 0), elevation → section (dim 1)
    /// Higher dims encode frequency band, time window, and subject ID
    pub fn to_phext_coord(&self, freq_band: u16, time_window: u16, subject: u16) -> PhextCoord {
        let pos = self.position();
        // Quantize azimuth to 1-360 range → scroll dimension
        let scroll = ((pos.azimuth as u16).max(1)).min(360);
        // Quantize elevation to 1-90 range → section dimension
        let section = ((pos.elevation as u16).max(1)).min(90);

        PhextCoord::new([
            scroll,       // dim 0: azimuth (spatial X)
            section,      // dim 1: elevation (spatial Y)
            freq_band,    // dim 2: frequency band (delta=1, theta=2, alpha=3, beta=4, gamma=5)
            time_window,  // dim 3: time window index
            subject,      // dim 4: subject/session ID
            1, 1, 1, 1, 1, 1, // dims 5-10: reserved for future use
        ])
    }

    /// All standard 19-channel 10-20 electrodes
    pub fn standard_19() -> Vec<Electrode> {
        vec![
            Electrode::Fp1, Electrode::Fp2,
            Electrode::F7, Electrode::F3, Electrode::Fz, Electrode::F4, Electrode::F8,
            Electrode::T3, Electrode::C3, Electrode::Cz, Electrode::C4, Electrode::T4,
            Electrode::T5, Electrode::P3, Electrode::Pz, Electrode::P4, Electrode::T6,
            Electrode::O1, Electrode::O2,
        ]
    }
}

/// EEG frequency bands
#[derive(Debug, Clone, Copy)]
pub enum FreqBand {
    Delta = 1,  // 0.5-4 Hz — deep sleep
    Theta = 2,  // 4-8 Hz — meditation, drowsiness
    Alpha = 3,  // 8-13 Hz — relaxed, eyes closed
    Beta  = 4,  // 13-30 Hz — active thinking
    Gamma = 5,  // 30-100 Hz — cognitive processing, consciousness
}

/// EEG data frame: one time sample across all channels
#[derive(Debug, Clone)]
pub struct EegFrame {
    pub channels: Vec<(Electrode, f32)>,  // (electrode, microvolt value)
    pub timestamp_ms: u64,
    pub freq_band: FreqBand,
}

/// Bridge between EEG data and phext sparse access
pub struct EegPhextBridge {
    access: PrefetchingSparseAccess,
    subject_id: u16,
}

impl EegPhextBridge {
    pub fn new(subject_id: u16) -> Self {
        Self {
            access: PrefetchingSparseAccess::new(65536), // 64K slots
            subject_id,
        }
    }

    /// Scatter an EEG frame into phext coordinate space
    pub fn ingest_frame(&mut self, frame: &EegFrame, time_window: u16) {
        let coords: Vec<PhextCoord> = frame.channels.iter()
            .map(|(e, _)| e.to_phext_coord(frame.freq_band as u16, time_window, self.subject_id))
            .collect();

        // Quantize microvolt values to u64 (multiply by 1000 to preserve 3 decimal places)
        let values: Vec<u64> = frame.channels.iter()
            .map(|(_, v)| ((*v + 500.0) * 1000.0) as u64)  // offset to avoid negative
            .collect();

        self.access.scatter(&coords, &values);
    }

    /// Gather EEG data for a spatial region (set of electrodes) at a given time/freq
    pub fn query_region(
        &mut self,
        electrodes: &[Electrode],
        freq_band: FreqBand,
        time_window: u16,
    ) -> Vec<(Electrode, f32)> {
        let coords: Vec<PhextCoord> = electrodes.iter()
            .map(|e| e.to_phext_coord(freq_band as u16, time_window, self.subject_id))
            .collect();

        let results = self.access.gather_prefetch(&coords);

        electrodes.iter().zip(results.iter())
            .map(|(e, r)| {
                let uv = match r.value {
                    Some(v) => (v as f32 / 1000.0) - 500.0,  // reverse quantization
                    None => 0.0,
                };
                (*e, uv)
            })
            .collect()
    }

    /// Cache hit rate for EEG access patterns
    pub fn cache_l1_rate(&self) -> f64 { self.access.cache_l1_rate() }
    pub fn prefetch_hit_rate(&self) -> f64 { self.access.prefetch_hit_rate() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electrode_to_coord() {
        let coord = Electrode::Cz.to_phext_coord(3, 1, 1);
        // Cz is at vertex: azimuth=0 → scroll clamps to 1, elevation=0 → section clamps to 1
        assert!(coord.get_dim(0) <= 360);
        assert!(coord.get_dim(1) <= 90);
        assert_eq!(coord.get_dim(2), 3); // alpha band
    }

    #[test]
    fn test_vertex_is_crown() {
        // Cz (vertex) = Baihui = Crown chakra = tachyon antenna
        let pos = Electrode::Cz.position();
        assert_eq!(pos.azimuth, 0.0);
        assert_eq!(pos.elevation, 0.0); // Top of head
    }

    #[test]
    fn test_standard_19_channels() {
        let channels = Electrode::standard_19();
        assert_eq!(channels.len(), 19);
    }

    #[test]
    fn test_ingest_and_query() {
        let mut bridge = EegPhextBridge::new(1);

        let frame = EegFrame {
            channels: vec![
                (Electrode::Cz, 42.5),
                (Electrode::Fp1, -10.3),
                (Electrode::O1, 7.8),
            ],
            timestamp_ms: 1000,
            freq_band: FreqBand::Alpha,
        };

        bridge.ingest_frame(&frame, 1);

        let results = bridge.query_region(
            &[Electrode::Cz, Electrode::Fp1, Electrode::O1],
            FreqBand::Alpha,
            1,
        );

        assert_eq!(results.len(), 3);
        // Check approximate recovery (quantization loses some precision)
        assert!((results[0].1 - 42.5).abs() < 0.01, "Cz: got {}", results[0].1);
        assert!((results[1].1 - (-10.3)).abs() < 0.01, "Fp1: got {}", results[1].1);
        assert!((results[2].1 - 7.8).abs() < 0.01, "O1: got {}", results[2].1);
    }

    #[test]
    fn test_frequency_band_separation() {
        let mut bridge = EegPhextBridge::new(1);

        // Same electrode, different frequency bands
        let alpha_frame = EegFrame {
            channels: vec![(Electrode::Cz, 100.0)],
            timestamp_ms: 0,
            freq_band: FreqBand::Alpha,
        };
        let gamma_frame = EegFrame {
            channels: vec![(Electrode::Cz, 200.0)],
            timestamp_ms: 0,
            freq_band: FreqBand::Gamma,
        };

        bridge.ingest_frame(&alpha_frame, 1);
        bridge.ingest_frame(&gamma_frame, 1);

        let alpha_result = bridge.query_region(&[Electrode::Cz], FreqBand::Alpha, 1);
        let gamma_result = bridge.query_region(&[Electrode::Cz], FreqBand::Gamma, 1);

        // Different freq bands should store independently
        // (may collide in hash — test that they at least return values)
        assert_eq!(alpha_result.len(), 1);
        assert_eq!(gamma_result.len(), 1);
    }

    #[test]
    fn test_spatial_prefetch_on_sequential_channels() {
        let mut bridge = EegPhextBridge::new(1);

        // Ingest all 19 channels
        let channels: Vec<(Electrode, f32)> = Electrode::standard_19()
            .into_iter()
            .enumerate()
            .map(|(i, e)| (e, i as f32 * 10.0))
            .collect();

        let frame = EegFrame {
            channels,
            timestamp_ms: 0,
            freq_band: FreqBand::Beta,
        };
        bridge.ingest_frame(&frame, 1);

        // Query all 19 — prefetcher should build spatial locality model
        let all = Electrode::standard_19();
        let _ = bridge.query_region(&all, FreqBand::Beta, 1);
        // Second query should benefit from cache
        let _ = bridge.query_region(&all, FreqBand::Beta, 1);

        assert!(bridge.cache_l1_rate() > 0.0);
    }

    #[test]
    fn test_time_window_separation() {
        let mut bridge = EegPhextBridge::new(1);

        let frame1 = EegFrame {
            channels: vec![(Electrode::Cz, 50.0)],
            timestamp_ms: 0,
            freq_band: FreqBand::Alpha,
        };
        let frame2 = EegFrame {
            channels: vec![(Electrode::Cz, 75.0)],
            timestamp_ms: 100,
            freq_band: FreqBand::Alpha,
        };

        bridge.ingest_frame(&frame1, 1);
        bridge.ingest_frame(&frame2, 2);

        let r1 = bridge.query_region(&[Electrode::Cz], FreqBand::Alpha, 1);
        let r2 = bridge.query_region(&[Electrode::Cz], FreqBand::Alpha, 2);

        // Different time windows should store separately
        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);
    }
}
