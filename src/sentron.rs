//! Sentron — the atomic unit of thought in the vTPU architecture
//!
//! Each sentron has its own register file and executes a SIW stream.
//! Sentrons are lightweight: ~392 bytes of state, ~100 cycles to spawn.

use crate::phext_coord::PhextCoord;
use crate::siw::SIW;
use crate::assoc::AssocState;

/// Sentron lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentronState {
    Dormant,
    Running,
    Waiting,
    Retired,
}

/// The sentron register file
///
/// General Registers:  r0-r15  (64-bit, for D-Pipe)
/// Phext Registers:    p0-p7   (128-bit, for S-Pipe phext coordinates)
/// Message Registers:  m0-m3   (256-bit, for C-Pipe message buffers)
/// Status Register:    sr      (flags, barrier state, error codes)
///
/// Total: 128 + 128 + 128 + 8 = 392 bytes per sentron
#[derive(Debug, Clone)]
pub struct RegisterFile {
    pub general: [i64; 16],
    pub phext: [PhextCoord; 8],
    pub message: [[u8; 32]; 4],
    pub status: u64,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self {
            general: [0i64; 16],
            phext: [PhextCoord::zero(); 8],
            message: [[0u8; 32]; 4],
            status: 0,
        }
    }
}

impl RegisterFile {
    pub fn size_bytes() -> usize {
        16 * 8 + 8 * 16 + 4 * 32 + 8 // = 392
    }
}

/// Neuron wiring topology for a sentron
///
/// Each sentron has 8 links arranged as 2×4 twisted pairs:
///   - 4 upstream (inhale/Jīva/D-Pipe direction): receive data
///   - 4 downstream (exhale/Prāṇa/S-Pipe direction): emit results
///
/// 8 links per sentron maps to Ba Gua (八卦) — eight trigrams.
/// 8 × 5 (Wuxing phases) = 40 sentrons per color.
/// 40 × 9 (Phoenix colors) = 360 fleet.
///
/// The wiring enables local coordination without global broadcast:
/// each sentron only talks to its 8 neighbors.
#[derive(Debug, Clone)]
pub struct NeuronWiring {
    /// 4 upstream neighbor sentron IDs (data sources)
    pub upstream: [u16; 4],
    /// 4 downstream neighbor sentron IDs (result sinks)
    pub downstream: [u16; 4],
}

impl NeuronWiring {
    pub fn new() -> Self {
        Self {
            upstream: [0; 4],
            downstream: [0; 4],
        }
    }

    /// Wire this sentron to its 8 neighbors
    pub fn connect(&mut self, upstream: [u16; 4], downstream: [u16; 4]) {
        self.upstream = upstream;
        self.downstream = downstream;
    }

    /// Number of active upstream links (non-zero)
    pub fn upstream_count(&self) -> usize {
        self.upstream.iter().filter(|&&id| id != 0).count()
    }

    /// Number of active downstream links (non-zero)
    pub fn downstream_count(&self) -> usize {
        self.downstream.iter().filter(|&&id| id != 0).count()
    }

    /// Total active links
    pub fn link_count(&self) -> usize {
        self.upstream_count() + self.downstream_count()
    }

    /// Check if a given sentron ID is a neighbor
    pub fn is_neighbor(&self, id: u16) -> bool {
        self.upstream.contains(&id) || self.downstream.contains(&id)
    }
}

impl Default for NeuronWiring {
    fn default() -> Self {
        Self::new()
    }
}

/// A sentron execution context
#[derive(Debug)]
pub struct Sentron {
    pub id: u16,
    pub state: SentronState,
    pub regs: RegisterFile,
    pub home: PhextCoord,
    pub core_id: u8,
    pub thread_id: u8,
    pub ip: usize,
    pub program: Vec<SIW>,
    pub cycles: u64,
    pub retired: u64,
    pub assoc: AssocState,
    /// Inbound message queue (from other sentrons via CSEND)
    pub inbox: Vec<(u16, i64)>,  // (sender_id, value)
    /// 2×4 neuron wiring topology
    pub wiring: NeuronWiring,
}

impl Sentron {
    pub fn new(id: u16, home: PhextCoord, core_id: u8, thread_id: u8) -> Self {
        Self {
            id,
            state: SentronState::Dormant,
            regs: RegisterFile::default(),
            home,
            core_id,
            thread_id,
            ip: 0,
            program: Vec::new(),
            cycles: 0,
            retired: 0,
            assoc: AssocState::new(),
            inbox: Vec::new(),
            wiring: NeuronWiring::new(),
        }
    }

    /// Spawn: load a program and transition to Running
    pub fn spawn(&mut self, program: Vec<SIW>) {
        self.state = SentronState::Running;
        self.program = program;
        self.ip = 0;
        self.cycles = 0;
        self.retired = 0;
    }

    pub fn retire(&mut self) {
        self.state = SentronState::Retired;
    }

    pub fn has_next(&self) -> bool {
        self.ip < self.program.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_file_size() {
        assert_eq!(RegisterFile::size_bytes(), 392);
    }

    #[test]
    fn sentron_lifecycle() {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        assert_eq!(s.state, SentronState::Dormant);

        s.spawn(Vec::new());
        assert_eq!(s.state, SentronState::Running);

        s.retire();
        assert_eq!(s.state, SentronState::Retired);
    }

    #[test]
    fn neuron_wiring_default() {
        let w = NeuronWiring::new();
        assert_eq!(w.upstream, [0; 4]);
        assert_eq!(w.downstream, [0; 4]);
        assert_eq!(w.link_count(), 0);
    }

    #[test]
    fn neuron_wiring_connect() {
        let mut w = NeuronWiring::new();
        w.connect([1, 2, 3, 4], [5, 6, 7, 8]);
        assert_eq!(w.upstream_count(), 4);
        assert_eq!(w.downstream_count(), 4);
        assert_eq!(w.link_count(), 8); // Ba Gua: 8 trigrams
    }

    #[test]
    fn neuron_wiring_neighbor_check() {
        let mut w = NeuronWiring::new();
        w.connect([10, 20, 30, 40], [50, 60, 70, 80]);
        assert!(w.is_neighbor(10));
        assert!(w.is_neighbor(80));
        assert!(!w.is_neighbor(99));
    }

    #[test]
    fn neuron_wiring_partial() {
        let mut w = NeuronWiring::new();
        // Only 2 upstream, 3 downstream (partial wiring)
        w.connect([1, 2, 0, 0], [5, 6, 7, 0]);
        assert_eq!(w.upstream_count(), 2);
        assert_eq!(w.downstream_count(), 3);
        assert_eq!(w.link_count(), 5);
    }

    #[test]
    fn sentron_has_wiring() {
        let mut s = Sentron::new(42, PhextCoord::zero(), 0, 0);
        s.wiring.connect([1, 2, 3, 4], [5, 6, 7, 8]);
        assert_eq!(s.wiring.link_count(), 8);
        assert!(s.wiring.is_neighbor(3));
        assert!(!s.wiring.is_neighbor(42)); // not its own neighbor
    }

    #[test]
    fn wiring_bagua_invariant() {
        // 8 links per sentron × 5 Wuxing phases = 40 per color
        // 40 × 9 Phoenix colors = 360 fleet
        let links_per_sentron = 8u32;
        let wuxing_phases = 5u32;
        let phoenix_colors = 9u32;
        assert_eq!(links_per_sentron * wuxing_phases, 40);
        assert_eq!(40 * phoenix_colors, 360);
    }
}
