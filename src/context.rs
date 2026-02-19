// -----------------------------------------------
// context.rs — Sentron Context Switching
// -----------------------------------------------
// Save/restore sentron state for preemptive scheduling.
// Target: <100 cycles per context switch.
//
// A context snapshot captures everything needed to resume:
// registers, IP, cycle count, assoc state, wiring.
// The sentron body stays in the pool — only the "soul" moves.
//
// Zero external dependencies.
//
// R23W22 — Chrys 🦋

use crate::sentron::{Sentron, RegisterFile, SentronState, NeuronWiring};
use crate::phext_coord::PhextCoord;
use crate::SIW;

/// Frozen sentron state — the "soul" extracted from the body
#[derive(Clone)]
pub struct ContextSnapshot {
    pub id: u16,
    pub home: PhextCoord,
    pub regs: RegisterFile,
    pub state: SentronState,
    pub ip: usize,
    pub cycles: u64,
    pub retired: u64,
    pub program: Vec<SIW>,
    pub inbox: Vec<(u16, i64)>,
    pub wiring: NeuronWiring,
    pub core_id: u8,
    pub thread_id: u8,
}

impl ContextSnapshot {
    /// Snapshot size in bytes (approximate)
    pub fn size_bytes(&self) -> usize {
        RegisterFile::size_bytes()
            + self.program.len() * core::mem::size_of::<SIW>()
            + self.inbox.len() * 10
            + 64 // overhead (id, ip, cycles, etc.)
    }
}

/// Save sentron state into a snapshot
pub fn context_save(sentron: &Sentron) -> ContextSnapshot {
    ContextSnapshot {
        id: sentron.id,
        home: sentron.home.clone(),
        regs: sentron.regs.clone(),
        state: sentron.state.clone(),
        ip: sentron.ip,
        cycles: sentron.cycles,
        retired: sentron.retired,
        program: sentron.program.clone(),
        inbox: sentron.inbox.clone(),
        wiring: sentron.wiring.clone(),
        core_id: sentron.core_id,
        thread_id: sentron.thread_id,
    }
}

/// Restore sentron state from a snapshot
pub fn context_restore(sentron: &mut Sentron, snap: &ContextSnapshot) {
    sentron.id = snap.id;
    sentron.home = snap.home.clone();
    sentron.regs = snap.regs.clone();
    sentron.state = snap.state.clone();
    sentron.ip = snap.ip;
    sentron.cycles = snap.cycles;
    sentron.retired = snap.retired;
    sentron.program = snap.program.clone();
    sentron.inbox = snap.inbox.clone();
    sentron.wiring = snap.wiring.clone();
    sentron.core_id = snap.core_id;
    sentron.thread_id = snap.thread_id;
}

/// Full context switch: save current sentron, restore next sentron
/// Returns the snapshot of the preempted sentron
pub fn context_switch(current: &mut Sentron, next_snap: &ContextSnapshot) -> ContextSnapshot {
    let saved = context_save(current);
    context_restore(current, next_snap);
    saved
}

/// Context switch queue — FIFO of suspended sentrons
pub struct ContextQueue {
    queue: Vec<ContextSnapshot>,
    switches: u64,
}

impl ContextQueue {
    pub fn new() -> Self {
        Self { queue: Vec::new(), switches: 0 }
    }

    /// Suspend a sentron (push its state onto the queue)
    pub fn suspend(&mut self, sentron: &Sentron) {
        self.queue.push(context_save(sentron));
    }

    /// Resume the next sentron from the queue into the given body
    pub fn resume(&mut self, body: &mut Sentron) -> bool {
        if let Some(snap) = self.queue.pop() {
            context_restore(body, &snap);
            self.switches += 1;
            true
        } else {
            false
        }
    }

    /// Full switch: suspend current, resume next
    pub fn switch(&mut self, body: &mut Sentron) -> bool {
        if self.queue.is_empty() {
            return false;
        }
        self.suspend(body);
        // The just-suspended one is at the back; resume from front
        let snap = self.queue.remove(0);
        context_restore(body, &snap);
        self.switches += 1;
        true
    }

    /// Number of suspended sentrons
    pub fn depth(&self) -> usize {
        self.queue.len()
    }

    /// Total context switches performed
    pub fn total_switches(&self) -> u64 {
        self.switches
    }

    /// Is the queue empty?
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sentron(id: u16, reg0_val: i64) -> Sentron {
        let mut s = Sentron::new(id, PhextCoord::new([id, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), 0, 0);
        s.regs.general[0] = reg0_val;
        s.cycles = id as u64 * 100;
        s
    }

    #[test]
    fn test_save_restore_preserves_state() {
        let s = make_sentron(7, 42);
        let snap = context_save(&s);

        let mut body = Sentron::new(0, PhextCoord::zero(), 0, 0);
        context_restore(&mut body, &snap);

        assert_eq!(body.id, 7);
        assert_eq!(body.regs.general[0], 42);
        assert_eq!(body.cycles, 700);
    }

    #[test]
    fn test_context_switch_swaps() {
        let mut current = make_sentron(1, 100);
        let next = make_sentron(2, 200);
        let next_snap = context_save(&next);

        let saved = context_switch(&mut current, &next_snap);

        // current now holds sentron 2's state
        assert_eq!(current.id, 2);
        assert_eq!(current.regs.general[0], 200);

        // saved holds sentron 1's state
        assert_eq!(saved.id, 1);
        assert_eq!(saved.regs.general[0], 100);
    }

    #[test]
    fn test_queue_suspend_resume() {
        let mut q = ContextQueue::new();
        let s1 = make_sentron(1, 111);
        let s2 = make_sentron(2, 222);

        q.suspend(&s1);
        q.suspend(&s2);
        assert_eq!(q.depth(), 2);

        let mut body = Sentron::new(0, PhextCoord::zero(), 0, 0);
        assert!(q.resume(&mut body));
        assert_eq!(body.id, 2); // LIFO: last suspended = first resumed
        assert_eq!(body.regs.general[0], 222);
    }

    #[test]
    fn test_queue_switch() {
        let mut q = ContextQueue::new();
        let s1 = make_sentron(1, 111);
        q.suspend(&s1);

        let mut body = make_sentron(3, 333);
        assert!(q.switch(&mut body));

        // body now holds s1's state (FIFO: first suspended = first resumed)
        assert_eq!(body.id, 1);
        assert_eq!(body.regs.general[0], 111);

        // s3 is now in the queue
        assert_eq!(q.depth(), 1);
    }

    #[test]
    fn test_empty_queue_resume_fails() {
        let mut q = ContextQueue::new();
        let mut body = Sentron::new(0, PhextCoord::zero(), 0, 0);
        assert!(!q.resume(&mut body));
        assert!(q.is_empty());
    }

    #[test]
    fn test_switch_count() {
        let mut q = ContextQueue::new();
        let s1 = make_sentron(1, 100);
        let s2 = make_sentron(2, 200);
        q.suspend(&s1);
        q.suspend(&s2);

        let mut body = make_sentron(3, 300);
        q.switch(&mut body);
        q.switch(&mut body);

        assert_eq!(q.total_switches(), 2);
    }

    #[test]
    fn test_snapshot_size() {
        let s = make_sentron(1, 42);
        let snap = context_save(&s);
        assert!(snap.size_bytes() >= RegisterFile::size_bytes());
    }

    #[test]
    fn test_rapid_context_switching() {
        // Simulate rapid switching between 8 sentrons (Ba Gua count)
        let mut q = ContextQueue::new();
        for i in 0..8 {
            q.suspend(&make_sentron(i, i as i64 * 10));
        }

        let mut body = make_sentron(99, 0);
        for _ in 0..8 {
            q.switch(&mut body);
        }
        assert_eq!(q.total_switches(), 8);
        // All 8 sentrons cycled through
        assert_eq!(q.depth(), 8); // 7 original + 1 preempted body, minus 8 resumed... net = still full
    }

    #[test]
    fn test_inbox_preserved() {
        let mut s = make_sentron(5, 50);
        s.inbox.push((1, 999));
        s.inbox.push((2, 888));

        let snap = context_save(&s);
        let mut body = Sentron::new(0, PhextCoord::zero(), 0, 0);
        context_restore(&mut body, &snap);

        assert_eq!(body.inbox.len(), 2);
        assert_eq!(body.inbox[0], (1, 999));
    }

    #[test]
    fn test_wiring_preserved() {
        let mut s = make_sentron(3, 30);
        s.wiring.connect([10, 20, 30, 40], [50, 60, 70, 80]);

        let snap = context_save(&s);
        let mut body = Sentron::new(0, PhextCoord::zero(), 0, 0);
        context_restore(&mut body, &snap);

        assert_eq!(body.wiring.upstream, [10, 20, 30, 40]);
        assert_eq!(body.wiring.downstream, [50, 60, 70, 80]);
    }
}
