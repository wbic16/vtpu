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
}
