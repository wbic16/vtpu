//! Sentron Pool — pre-allocated sentron recycling
//!
//! The gap: creating a new Sentron + Vec<SIW> per invocation costs ~4,500/sec.
//! The fix: pre-allocate sentrons and recycle them. Zero heap allocation in the hot path.

use crate::sentron::{Sentron, SentronState};
use crate::siw::SIW;
use crate::phext_coord::PhextCoord;

/// A pool of pre-allocated sentrons for high-throughput scheduling.
///
/// Instead of `Sentron::new()` + `spawn(vec![...])` each time,
/// check out a sentron, load its program in-place, run it, return it.
pub struct SentronPool {
    pool: Vec<Sentron>,
    available: Vec<usize>, // indices of available sentrons
    capacity: usize,
}

impl SentronPool {
    /// Create a pool with `capacity` pre-allocated sentrons.
    /// Each sentron gets a program buffer pre-sized to `max_program_len`.
    pub fn new(capacity: usize, max_program_len: usize) -> Self {
        let mut pool = Vec::with_capacity(capacity);
        let mut available = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let mut s = Sentron::new(i as u16, PhextCoord::zero(), 0, 0);
            s.program = Vec::with_capacity(max_program_len);
            pool.push(s);
            available.push(i);
        }
        SentronPool { pool, available, capacity }
    }

    /// Check out a sentron from the pool. Returns its index, or None if exhausted.
    pub fn checkout(&mut self) -> Option<usize> {
        self.available.pop()
    }

    /// Get a mutable reference to a checked-out sentron.
    pub fn get_mut(&mut self, idx: usize) -> &mut Sentron {
        &mut self.pool[idx]
    }

    /// Get an immutable reference.
    pub fn get(&self, idx: usize) -> &Sentron {
        &self.pool[idx]
    }

    /// Load a program into a checked-out sentron without allocating.
    /// Uses the pre-allocated buffer. Panics if program exceeds max_program_len.
    pub fn load_program(&mut self, idx: usize, program: &[SIW]) {
        let s = &mut self.pool[idx];
        s.program.clear();
        s.program.extend_from_slice(program);
        s.state = SentronState::Running;
        s.ip = 0;
        s.cycles = 0;
        s.retired = 0;
        // Reset registers
        s.regs.general = [0i64; 16];
        s.regs.status = 0;
    }

    /// Return a sentron to the pool after execution.
    pub fn checkin(&mut self, idx: usize) {
        self.pool[idx].state = SentronState::Dormant;
        self.pool[idx].program.clear(); // keep capacity
        self.available.push(idx);
    }

    /// How many sentrons are available.
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Total pool capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// How many are currently checked out.
    pub fn active_count(&self) -> usize {
        self.capacity - self.available.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};
    use crate::exec;

    #[test]
    fn pool_checkout_checkin() {
        let mut pool = SentronPool::new(4, 16);
        assert_eq!(pool.available_count(), 4);

        let idx = pool.checkout().unwrap();
        assert_eq!(pool.available_count(), 3);
        assert_eq!(pool.active_count(), 1);

        pool.checkin(idx);
        assert_eq!(pool.available_count(), 4);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn pool_exhaustion() {
        let mut pool = SentronPool::new(2, 8);
        let _a = pool.checkout().unwrap();
        let _b = pool.checkout().unwrap();
        assert!(pool.checkout().is_none());
    }

    #[test]
    fn pool_execute() {
        let mut pool = SentronPool::new(1, 8);
        let idx = pool.checkout().unwrap();

        // Load program first (resets registers), then set inputs
        let program = [
            SIW::new(
                DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ),
        ];
        pool.load_program(idx, &program);
        pool.get_mut(idx).regs.general[0] = 7;
        pool.get_mut(idx).regs.general[1] = 6;

        let _stats = exec::run_standalone(pool.get_mut(idx));
        assert_eq!(pool.get(idx).regs.general[2], 42);

        pool.checkin(idx);
        assert_eq!(pool.available_count(), 1);
    }

    #[test]
    fn pool_recycle_no_alloc() {
        let mut pool = SentronPool::new(1, 16);

        // Run twice — second run reuses the same buffer
        for expected in [42i64, 100] {
            let idx = pool.checkout().unwrap();
            let (a, b) = if expected == 42 { (7, 6) } else { (10, 10) };
            let program = [SIW::new(
                DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
                SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
            )];
            pool.load_program(idx, &program);
            pool.get_mut(idx).regs.general[0] = a;
            pool.get_mut(idx).regs.general[1] = b;
            exec::run_standalone(pool.get_mut(idx));
            assert_eq!(pool.get(idx).regs.general[2], expected);
            pool.checkin(idx);
        }
    }

    #[test]
    fn pool_capacity() {
        let pool = SentronPool::new(360, 32);
        assert_eq!(pool.capacity(), 360);
        assert_eq!(pool.available_count(), 360);
    }
}
