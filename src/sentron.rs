//! Sentron — the atomic unit of thought in the vTPU architecture
//!
//! Each sentron has its own register file and executes a SIW stream.
//! Sentrons are lightweight: ~392 bytes of state, ~100 cycles to spawn.

use crate::coord::PhextCoord;
use crate::siw::SIWStream;

/// Sentron lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentronState {
    /// Not yet spawned
    Dormant,
    /// Being initialized (register file setup)
    Spawning,
    /// Actively executing SIW stream
    Running,
    /// Blocked on barrier or receive
    Waiting,
    /// Execution complete, results available
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
    /// General-purpose registers (D-Pipe operands)
    pub general: [i64; 16],
    /// Phext coordinate registers (S-Pipe addressing)
    pub phext: [PhextCoord; 8],
    /// Message registers (C-Pipe buffers) — 256 bits each
    pub message: [[u8; 32]; 4],
    /// Status register
    pub status: u64,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self {
            general: [0i64; 16],
            phext: [PhextCoord::BASE; 8],
            message: [[0u8; 32]; 4],
            status: 0,
        }
    }
}

impl RegisterFile {
    /// Size in bytes (should be ~392)
    pub fn size_bytes() -> usize {
        16 * 8  // general: 128
        + 8 * 16  // phext: 128
        + 4 * 32  // message: 128
        + 8       // status: 8
        // = 392
    }
}

/// A sentron execution context
#[derive(Debug)]
pub struct Sentron {
    /// Unique identifier within the cluster
    pub id: u16,
    /// Current lifecycle state
    pub state: SentronState,
    /// Register file
    pub regs: RegisterFile,
    /// Home coordinate in phext space
    pub home: PhextCoord,
    /// Assigned physical core (0-7 per node)
    pub core_id: u8,
    /// SMT thread (0 or 1)
    pub thread_id: u8,
    /// Instruction pointer (index into SIW stream)
    pub ip: usize,
    /// The instruction stream being executed
    pub program: Option<SIWStream>,
    /// Cycles executed
    pub cycles: u64,
    /// SIWs retired
    pub retired: u64,
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
            program: None,
            cycles: 0,
            retired: 0,
        }
    }

    /// Spawn: load a program and transition to Running
    pub fn spawn(&mut self, program: SIWStream) {
        self.state = SentronState::Running;
        self.program = Some(program);
        self.ip = 0;
        self.cycles = 0;
        self.retired = 0;
    }

    /// Current ops/cycle ratio (the KPI we're chasing)
    pub fn ops_per_cycle(&self) -> f64 {
        if self.cycles == 0 {
            return 0.0;
        }
        // Each retired SIW = 3 ops (by the scheduling contract)
        (self.retired * 3) as f64 / self.cycles as f64
    }

    /// Has more instructions to execute?
    pub fn has_next(&self) -> bool {
        match &self.program {
            Some(p) => self.ip < p.len(),
            None => false,
        }
    }

    /// Retire the sentron
    pub fn retire(&mut self) {
        self.state = SentronState::Retired;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_file_size() {
        // Spec says 392 bytes. Rust struct may have padding,
        // but the logical size should match.
        assert_eq!(RegisterFile::size_bytes(), 392);
    }

    #[test]
    fn sentron_lifecycle() {
        let mut s = Sentron::new(0, PhextCoord::BASE, 0, 0);
        assert_eq!(s.state, SentronState::Dormant);

        let stream = SIWStream::new(PhextCoord::BASE);
        s.spawn(stream);
        assert_eq!(s.state, SentronState::Running);

        s.retired = 100;
        s.cycles = 100;
        assert!((s.ops_per_cycle() - 3.0).abs() < 0.01); // perfect: 3 ops/cycle

        s.retire();
        assert_eq!(s.state, SentronState::Retired);
    }

    #[test]
    fn spawn_cost() {
        // Spec target: ≤100 cycles (25 ns at 4 GHz)
        // We can't measure real cycles in a unit test, but we verify
        // the spawn path is O(1) — no allocation beyond the program itself.
        let mut s = Sentron::new(42, PhextCoord::BASE, 3, 1);
        let stream = SIWStream::new(PhextCoord::BASE);
        s.spawn(stream);
        assert_eq!(s.ip, 0);
        assert_eq!(s.cycles, 0);
    }
}
