// -----------------------------------------------
// fleet.rs — Fleet Coordination Engine
// -----------------------------------------------
// Manages 360 sentrons, dispatches C-Pipe operations
// (CSEND/CRECV/CROUTE/CBARRIER) between sentrons via
// NeuronWiring topology.
//
// The fleet is the full harmonic circle:
// 9 colors × 5 phases × 8 links = 360 sentrons.
//
// Zero external dependencies.
//
// R23W21 — Chrys 🦋

use crate::sentron::{Sentron, NeuronWiring};
use crate::pipes::CoordOp;
use crate::phext_coord::PhextCoord;

/// Message in flight between sentrons
#[derive(Debug, Clone)]
pub struct FleetMessage {
    pub sender_id: u16,
    pub receiver_id: u16,
    pub value: i64,
    pub coord: PhextCoord,
    pub delivered: bool,
}

/// Barrier state for synchronized fleet operations
#[derive(Debug)]
pub struct Barrier {
    pub id: u16,
    pub expected: usize,
    pub arrived: usize,
}

impl Barrier {
    pub fn new(id: u16, expected: usize) -> Self {
        Self { id, expected, arrived: 0 }
    }

    pub fn arrive(&mut self) -> bool {
        self.arrived += 1;
        self.arrived >= self.expected
    }

    pub fn is_complete(&self) -> bool {
        self.arrived >= self.expected
    }
}

/// Fleet coordination engine
pub struct Fleet {
    /// All sentrons in the fleet
    sentrons: Vec<Sentron>,
    /// Message queue (pending deliveries)
    messages: Vec<FleetMessage>,
    /// Active barriers
    barriers: Vec<Barrier>,
    /// Wiring topology (indexed by sentron ID)
    wiring: Vec<NeuronWiring>,
    /// Fleet size
    size: usize,
    /// Stats
    sends: u64,
    recvs: u64,
    routes: u64,
    barrier_syncs: u64,
}

impl Fleet {
    /// Create a fleet of given size with default wiring
    pub fn new(size: usize) -> Self {
        let sentrons: Vec<Sentron> = (0..size).map(|i| {
            Sentron::new(i as u16, PhextCoord::zero(), 0, 0)
        }).collect();

        // Default wiring: ring topology (each sentron wired to ±1, ±2)
        let wiring: Vec<NeuronWiring> = (0..size).map(|i| {
            let mut w = NeuronWiring::new();
            let n = size as u16;
            w.upstream = [
                ((i as u16 + n - 1) % n),
                ((i as u16 + n - 2) % n),
                ((i as u16 + n - 3) % n),
                ((i as u16 + n - 4) % n),
            ];
            w.downstream = [
                ((i as u16 + 1) % n),
                ((i as u16 + 2) % n),
                ((i as u16 + 3) % n),
                ((i as u16 + 4) % n),
            ];
            w
        }).collect();

        Self {
            sentrons,
            messages: Vec::new(),
            barriers: Vec::new(),
            wiring,
            size,
            sends: 0,
            recvs: 0,
            routes: 0,
            barrier_syncs: 0,
        }
    }

    /// Standard fleet: 360 sentrons
    pub fn standard() -> Self {
        Self::new(360)
    }

    /// Execute a C-Pipe operation
    pub fn dispatch(&mut self, sender_id: u16, op: &CoordOp) {
        match op {
            CoordOp::CSEND { msg_reg, dest_sentron } => {
                self.send(sender_id, *dest_sentron as u16, *msg_reg);
            }
            CoordOp::CRECV { rd, src_sentron } => {
                self.recv(sender_id, *src_sentron as u16, *rd);
            }
            CoordOp::CROUTE { msg_reg, dest_node } => {
                self.route(sender_id, *dest_node, *msg_reg);
            }
            CoordOp::CBAR { barrier_id: _, count } => {
                self.barrier(sender_id, *count as usize);
            }
            _ => {} // CNOP or unknown
        }
    }

    /// Send a value from one sentron to another
    fn send(&mut self, sender_id: u16, receiver_id: u16, msg_reg: u8) {
        if (sender_id as usize) >= self.size || (receiver_id as usize) >= self.size {
            return;
        }

        // Validate: receiver must be in sender's downstream (topology constraint)
        let is_neighbor = self.wiring[sender_id as usize].downstream
            .iter().any(|&d| d == receiver_id);

        let value = if is_neighbor {
            // Read from sender's general register (using msg_reg as index into general regs)
            let s = &self.sentrons[sender_id as usize];
            s.regs.general[(msg_reg as usize) % 16]
        } else {
            0 // Non-neighbor sends are dropped (topology violation)
        };

        self.messages.push(FleetMessage {
            sender_id,
            receiver_id,
            value,
            coord: PhextCoord::zero(),
            delivered: false,
        });
        self.sends += 1;
    }

    /// Receive a pending message
    fn recv(&mut self, receiver_id: u16, src_sentron: u16, rd: u8) {
        if (receiver_id as usize) >= self.size {
            return;
        }

        // Find the first undelivered message from src to receiver
        let msg_value = self.messages.iter_mut()
            .find(|m| m.sender_id == src_sentron && m.receiver_id == receiver_id && !m.delivered)
            .map(|m| {
                m.delivered = true;
                m.value
            });

        if let Some(value) = msg_value {
            self.sentrons[receiver_id as usize].regs.general[(rd as usize) % 16] = value;
        }
        self.recvs += 1;
    }

    /// Route a message to a specific node (for inter-node fleet communication)
    fn route(&mut self, sender_id: u16, dest_node: u8, msg_reg: u8) {
        // For now, route is a broadcast to all downstream neighbors
        if (sender_id as usize) >= self.size {
            return;
        }

        let downstream = self.wiring[sender_id as usize].downstream;
        let value = self.sentrons[sender_id as usize].regs.general[(msg_reg as usize) % 16];

        for &dest in &downstream {
            self.messages.push(FleetMessage {
                sender_id,
                receiver_id: dest,
                value,
                coord: PhextCoord::new([dest_node as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                delivered: false,
            });
        }
        self.routes += 1;
    }

    /// Barrier synchronization
    fn barrier(&mut self, _sentron_id: u16, group_size: usize) {
        // Find or create barrier
        let barrier = self.barriers.iter_mut()
            .find(|b| !b.is_complete() && b.expected == group_size);

        if let Some(b) = barrier {
            b.arrive();
        } else {
            let mut b = Barrier::new(self.barriers.len() as u16, group_size);
            b.arrive();
            self.barriers.push(b);
        }
        self.barrier_syncs += 1;
    }

    /// Deliver all pending messages
    pub fn flush(&mut self) {
        // Auto-deliver all pending messages
        for msg in &mut self.messages {
            if !msg.delivered && (msg.receiver_id as usize) < self.size {
                self.sentrons[msg.receiver_id as usize]
                    .inbox.push((msg.sender_id, msg.value));
                msg.delivered = true;
            }
        }
        // Clear delivered messages
        self.messages.retain(|m| !m.delivered);
    }

    /// Fleet size
    pub fn size(&self) -> usize { self.size }

    /// Pending messages
    pub fn pending_messages(&self) -> usize {
        self.messages.iter().filter(|m| !m.delivered).count()
    }

    /// Stats
    pub fn total_sends(&self) -> u64 { self.sends }
    pub fn total_recvs(&self) -> u64 { self.recvs }
    pub fn total_routes(&self) -> u64 { self.routes }
    pub fn total_barrier_syncs(&self) -> u64 { self.barrier_syncs }

    /// Get a sentron by ID
    pub fn sentron(&self, id: u16) -> Option<&Sentron> {
        self.sentrons.get(id as usize)
    }

    /// Get mutable sentron
    pub fn sentron_mut(&mut self, id: u16) -> Option<&mut Sentron> {
        self.sentrons.get_mut(id as usize)
    }

    /// Get wiring for a sentron
    pub fn wiring(&self, id: u16) -> Option<&NeuronWiring> {
        self.wiring.get(id as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_creation() {
        let fleet = Fleet::new(40);
        assert_eq!(fleet.size(), 40);
    }

    #[test]
    fn test_standard_fleet_360() {
        let fleet = Fleet::standard();
        assert_eq!(fleet.size(), 360);
    }

    #[test]
    fn test_ring_wiring() {
        let fleet = Fleet::new(10);
        let w = fleet.wiring(0).unwrap();
        // Sentron 0: upstream = [9, 8, 7, 6], downstream = [1, 2, 3, 4]
        assert_eq!(w.downstream[0], 1);
        assert_eq!(w.downstream[1], 2);
        assert_eq!(w.upstream[0], 9);
    }

    #[test]
    fn test_send_recv_neighbor() {
        let mut fleet = Fleet::new(10);
        // Sentron 0's downstream includes sentron 1
        // Set general register 0 on sentron 0
        fleet.sentron_mut(0).unwrap().regs.general[0] = 42;

        // Send from 0 to 1
        fleet.dispatch(0, &CoordOp::CSEND { msg_reg: 0, dest_sentron: 1 });
        assert_eq!(fleet.total_sends(), 1);

        // Recv on sentron 1
        fleet.dispatch(1, &CoordOp::CRECV { rd: 0, src_sentron: 0 });
        let val = fleet.sentron(1).unwrap().regs.general[0];
        assert_eq!(val, 42);
    }

    #[test]
    fn test_route_broadcasts_downstream() {
        let mut fleet = Fleet::new(10);
        fleet.sentron_mut(0).unwrap().regs.general[0] = 99;

        fleet.dispatch(0, &CoordOp::CROUTE { msg_reg: 0, dest_node: 1 });
        assert_eq!(fleet.total_routes(), 1);
        // Should create 4 messages (one per downstream neighbor)
        assert_eq!(fleet.pending_messages(), 4);
    }

    #[test]
    fn test_barrier_sync() {
        let mut fleet = Fleet::new(10);
        // 3 sentrons arrive at barrier of size 3
        fleet.dispatch(0, &CoordOp::CBAR { barrier_id: 0, count: 3 });
        fleet.dispatch(1, &CoordOp::CBAR { barrier_id: 0, count: 3 });
        assert!(!fleet.barriers[0].is_complete());
        fleet.dispatch(2, &CoordOp::CBAR { barrier_id: 0, count: 3 });
        assert!(fleet.barriers[0].is_complete());
        assert_eq!(fleet.total_barrier_syncs(), 3);
    }

    #[test]
    fn test_flush_delivers_all() {
        let mut fleet = Fleet::new(10);
        fleet.sentron_mut(0).unwrap().regs.general[0] = 77;

        // Route creates 4 pending messages
        fleet.dispatch(0, &CoordOp::CROUTE { msg_reg: 0, dest_node: 1 });
        assert_eq!(fleet.pending_messages(), 4);

        fleet.flush();
        assert_eq!(fleet.pending_messages(), 0);
    }

    #[test]
    fn test_bagua_invariant() {
        let fleet = Fleet::new(360);
        // Each sentron has 8 links (4 up + 4 down)
        for i in 0..360u16 {
            let w = fleet.wiring(i).unwrap();
            assert_eq!(w.upstream.len(), 4);
            assert_eq!(w.downstream.len(), 4);
        }
        // 360 = 8 × 5 × 9
        assert_eq!(360, 8 * 5 * 9);
    }
}
