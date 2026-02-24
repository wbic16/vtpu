//! Cooperative Fleet — interleaved multi-sentron execution WITH message passing
//!
//! Combines coop_smt (interleaved scheduling) with fleet (message routing).
//! After each quantum, outboxes drain → fleet delivers → inboxes fill.
//! This is the actual execution model: sentrons compute AND communicate.
//!
//! R23W28 — Theia 💎

use crate::exec::exec_siw_octawire;
use crate::fleet::Fleet;
use crate::memory::Memory;
use crate::sentron::SentronState;

/// Result of a cooperative fleet run
#[derive(Debug, Default)]
pub struct CoopFleetResult {
    pub total_retired: u64,
    pub total_cycles: u64,
    pub context_switches: u64,
    pub messages_delivered: u64,
    pub flushes: u64,
}

/// Run cooperative fleet execution: interleave + message pass
///
/// Each sentron gets `quantum` SIWs, then the fleet flushes messages.
/// This models real vTPU execution: compute/communicate/compute/communicate.
pub fn coop_fleet_execute(
    fleet: &mut Fleet,
    mem: &mut Memory,
    quantum: usize,
    max_cycles: u64,
) -> CoopFleetResult {
    let n = fleet.size();
    if n == 0 {
        return CoopFleetResult::default();
    }

    let mut result = CoopFleetResult::default();

    // Activate dormant sentrons
    for i in 0..n {
        if let Some(s) = fleet.sentron_mut(i as u16) {
            if s.state == SentronState::Dormant && !s.program.is_empty() {
                s.state = SentronState::Running;
            }
        }
    }

    let mut current = 0usize;

    loop {
        // Count active
        let active: usize = (0..n)
            .filter(|&i| {
                fleet.sentron(i as u16)
                    .map(|s| s.state == SentronState::Running && s.has_next())
                    .unwrap_or(false)
            })
            .count();

        if active == 0 || result.total_cycles >= max_cycles {
            break;
        }

        // Find next running sentron
        let mut found = false;
        for _ in 0..n {
            if let Some(s) = fleet.sentron(current as u16) {
                if s.state == SentronState::Running && s.has_next() {
                    found = true;
                    break;
                }
            }
            current = (current + 1) % n;
        }
        if !found { break; }

        // Execute quantum SIWs
        let s = fleet.sentron_mut(current as u16).unwrap();
        for _ in 0..quantum {
            if !s.has_next() { break; }
            let siw = s.program[s.ip].clone();
            exec_siw_octawire(s, &siw, mem);
            s.ip += 1;
            s.cycles += 1;
            s.retired += 1;
            result.total_retired += 1;
            result.total_cycles += 1;
        }

        // Retire if done
        if !fleet.sentron(current as u16).unwrap().has_next() {
            fleet.sentron_mut(current as u16).unwrap().state = SentronState::Retired;
        }

        // Flush messages after each quantum (the communication phase)
        let pending_before = fleet.pending_messages();
        fleet.flush();
        let delivered = pending_before.saturating_sub(fleet.pending_messages());
        result.messages_delivered += delivered as u64;
        result.flushes += 1;

        result.context_switches += 1;
        current = (current + 1) % n;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::{DenseOp, SparseOp, CoordOp};
    use crate::siw::SIW;
    use crate::phext_coord::PhextCoord;

    fn nop() -> SIW {
        SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }

    fn add(rd: u8, rs1: u8, rs2: u8) -> SIW {
        SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }

    fn send(msg_reg: u8, dest: u8) -> SIW {
        SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CSEND { msg_reg, dest_sentron: dest }, PhextCoord::zero())
    }

    fn recv(rd: u8, src: u8) -> SIW {
        SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CRECV { rd, src_sentron: src }, PhextCoord::zero())
    }

    #[test]
    fn basic_fleet_execution() {
        let mut fleet = Fleet::new(4);
        for i in 0..4u16 {
            fleet.sentron_mut(i).unwrap().spawn(vec![nop(); 8]);
        }
        let mut mem = Memory::new();
        let result = coop_fleet_execute(&mut fleet, &mut mem, 4, 100_000);
        assert_eq!(result.total_retired, 32); // 4 × 8
    }

    #[test]
    fn send_recv_across_fleet() {
        let mut fleet = Fleet::new(4);
        // Sentron 0: set r0=42, send to sentron 1
        fleet.sentron_mut(0).unwrap().regs.general[0] = 42;
        fleet.sentron_mut(0).unwrap().spawn(vec![send(0, 1)]);
        // Sentron 1: recv from sentron 0 into r0
        fleet.sentron_mut(1).unwrap().spawn(vec![nop(), recv(0, 0)]);
        // Sentrons 2,3: idle
        fleet.sentron_mut(2).unwrap().spawn(vec![nop()]);
        fleet.sentron_mut(3).unwrap().spawn(vec![nop()]);

        let mut mem = Memory::new();
        let result = coop_fleet_execute(&mut fleet, &mut mem, 1, 100_000);

        // Sentron 1 should have received 42 in r0
        let val = fleet.sentron(1).unwrap().regs.general[0];
        assert_eq!(val, 42, "message should propagate: got {}", val);
    }

    #[test]
    fn nine_sentron_ring() {
        // One Phoenix color group: 9 sentrons, each does 40 ops
        let mut fleet = Fleet::new(9);
        for i in 0..9u16 {
            fleet.sentron_mut(i).unwrap().spawn(vec![add(0, 1, 2); 40]);
        }
        let mut mem = Memory::new();
        let result = coop_fleet_execute(&mut fleet, &mut mem, 4, 100_000);
        assert_eq!(result.total_retired, 360); // 9 × 40 = 360!
    }

    #[test]
    fn forty_sentron_wuxing() {
        // Full color complement: 40 sentrons × 9 ops = 360
        let mut fleet = Fleet::new(40);
        for i in 0..40u16 {
            fleet.sentron_mut(i).unwrap().spawn(vec![add(0, 1, 2); 9]);
        }
        let mut mem = Memory::new();
        let result = coop_fleet_execute(&mut fleet, &mut mem, 3, 100_000);
        assert_eq!(result.total_retired, 360);
    }

    #[test]
    fn full_fleet_360() {
        let mut fleet = Fleet::standard(); // 360 sentrons
        for i in 0..360u16 {
            fleet.sentron_mut(i).unwrap().spawn(vec![nop(); 1]);
        }
        let mut mem = Memory::new();
        let result = coop_fleet_execute(&mut fleet, &mut mem, 1, 100_000);
        assert_eq!(result.total_retired, 360);
        assert_eq!(result.context_switches, 360);
    }

    #[test]
    fn message_chain() {
        // 0 → 1 → 2: relay a value through the ring
        let mut fleet = Fleet::new(4);
        fleet.sentron_mut(0).unwrap().regs.general[0] = 99;
        fleet.sentron_mut(0).unwrap().spawn(vec![send(0, 1)]);
        fleet.sentron_mut(1).unwrap().spawn(vec![nop(), recv(0, 0), send(0, 2)]);
        fleet.sentron_mut(2).unwrap().spawn(vec![nop(), nop(), recv(0, 1)]);
        fleet.sentron_mut(3).unwrap().spawn(vec![nop()]);

        let mut mem = Memory::new();
        coop_fleet_execute(&mut fleet, &mut mem, 1, 100_000);

        let val = fleet.sentron(2).unwrap().regs.general[0];
        assert_eq!(val, 99, "value should relay through chain: got {}", val);
    }
}
