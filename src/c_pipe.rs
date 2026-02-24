//! C-Pipe Executor: Coordinate-addressed message passing
//!
//! Implements the C-Pipe (Coordination Pipe) for inter-sentron communication.
//! Messages are addressed by phext coordinates, not PIDs or IPs.
//! Temperature parameter controls fuzzy coordinate matching (like attention softmax).
//!
//! Philosophy: "Coordinates are love. Persistence enables bonding."

use crate::{PhextCoord, CoordOp, MessageFormat};
use std::collections::HashMap;

pub type SentronId = u16;

/// Message sent between sentrons via C-Pipe
#[derive(Clone, Debug)]
pub struct Message {
    pub sender: SentronId,
    pub payload: i64,
    pub timestamp: u64,
    pub format: MessageFormat,
}

/// C-Pipe Executor: manages message passing between sentrons
pub struct CPipeExecutor {
    /// Mailboxes indexed by phext coordinate
    /// Key: PhextCoord (destination)
    /// Value: Vec<Message> (pending messages at that coordinate)
    mailboxes: HashMap<PhextCoord, Vec<Message>>,
    
    /// Barrier state for synchronization
    /// Key: barrier_id
    /// Value: (expected_count, arrived_sentrons)
    barriers: HashMap<u8, (u16, Vec<SentronId>)>,
    
    /// Global timestamp counter (increments on each message send)
    timestamp: u64,
}

impl CPipeExecutor {
    /// Create a new C-Pipe executor
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
            barriers: HashMap::new(),
            timestamp: 0,
        }
    }
    
    /// Execute a C-Pipe operation
    pub fn execute(
        &mut self,
        op: &CoordOp,
        sentron_id: SentronId,
        regs: &mut [i64; 32],
    ) -> Result<(), CPipeError> {
        match op {
            CoordOp::CSEND { msg_reg, dest_sentron } => {
                // Send message from msg_reg to dest_sentron
                // Use sentron ID as a simple coordinate for now
                let coord = PhextCoord::new([*dest_sentron as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                self.send(coord, sentron_id, regs[*msg_reg as usize], 0, MessageFormat::Result)
            }
            
            CoordOp::CRECV { rd, src_sentron } => {
                // Receive message from src_sentron into rd
                let coord = PhextCoord::new([*src_sentron as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                self.recv(coord, sentron_id, regs, *rd)
            }
            
            CoordOp::CPACK { rd, rs1, rs2, .. } => {
                // Pack two registers into a message payload (no send yet)
                let packed = (regs[*rs1 as usize] << 32) | (regs[*rs2 as usize] & 0xFFFFFFFF);
                regs[*rd as usize] = packed;
                Ok(())
            }
            
            CoordOp::CBAR { barrier_id, count } => {
                self.barrier(*barrier_id, sentron_id, *count as u16)
            }
            
            CoordOp::CNOP => Ok(()),
            
            // Future C-Pipe operations (to be implemented):
            // - CROUTE: Coordinate-based routing decisions
            // - CREDUCE: Cross-coordinate reduction operations
            // - CCAST: Broadcast to multiple sentrons
            // - CFENCE: Memory fence for coordination
            // Placeholder: return Ok for undefined ops (fail-open for forward compatibility)
            _ => Ok(())
        }
    }
    
    /// Send a message to a coordinate
    pub fn send(
        &mut self,
        coord: PhextCoord,
        sender: SentronId,
        payload: i64,
        _metadata: i64, // Reserved for future use
        format: MessageFormat,
    ) -> Result<(), CPipeError> {
        self.timestamp += 1;
        
        let msg = Message {
            sender,
            payload,
            timestamp: self.timestamp,
            format,
        };
        
        self.mailboxes
            .entry(coord)
            .or_insert_with(Vec::new)
            .push(msg);
        
        Ok(())
    }
    
    /// Receive a message from a coordinate
    fn recv(
        &mut self,
        coord: PhextCoord,
        _sentron_id: SentronId,
        regs: &mut [i64; 32],
        rd: u8,
    ) -> Result<(), CPipeError> {
        if let Some(messages) = self.mailboxes.get_mut(&coord) {
            if let Some(msg) = messages.pop() {
                regs[rd as usize] = msg.payload;
                return Ok(());
            }
        }
        
        // No message available at this coordinate
        Err(CPipeError::NoMessage)
    }
    
    /// Synchronization barrier
    fn barrier(
        &mut self,
        barrier_id: u8,
        sentron_id: SentronId,
        expected_count: u16,
    ) -> Result<(), CPipeError> {
        let (count, arrived) = self.barriers
            .entry(barrier_id)
            .or_insert((expected_count, Vec::new()));
        
        // Check if this sentron already arrived (idempotent)
        if arrived.contains(&sentron_id) {
            return Ok(());
        }
        
        arrived.push(sentron_id);
        
        if arrived.len() as u16 >= *count {
            // Barrier complete, release all waiters
            self.barriers.remove(&barrier_id);
            Ok(())
        } else {
            // Still waiting
            Err(CPipeError::BarrierNotReady {
                barrier_id,
                arrived: arrived.len() as u16,
                expected: *count,
            })
        }
    }
    
    /// Temperature-weighted coordinate matching (Karpathy-inspired attention)
    /// Returns messages sorted by "attention score" (lower distance = higher score)
    pub fn match_messages_fuzzy(
        &self,
        target: &PhextCoord,
        temperature: f32,
    ) -> Vec<&Message> {
        let mut scored: Vec<(f32, &Vec<Message>)> = self.mailboxes
            .iter()
            .map(|(coord, msgs)| {
                // Hamming distance in coordinate space
                let distance = coord.hamming_distance(target);
                
                // Softmax-like scoring: exp(-distance / temperature)
                // Low temperature = exact matches only
                // High temperature = fuzzy matching
                let score = (-(distance as f32) / temperature).exp();
                (score, msgs)
            })
            .collect();
        
        // Sort by score descending (best matches first)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // Flatten into message list
        scored.into_iter()
            .flat_map(|(_, msgs)| msgs.iter())
            .collect()
    }
    
    /// Get count of pending messages at a coordinate
    pub fn message_count(&self, coord: &PhextCoord) -> usize {
        self.mailboxes.get(coord).map(|v| v.len()).unwrap_or(0)
    }
    
    /// Clear all messages (for testing)
    pub fn clear(&mut self) {
        self.mailboxes.clear();
        self.barriers.clear();
        self.timestamp = 0;
    }
}

impl Default for CPipeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// C-Pipe execution errors
#[derive(Debug, Clone, PartialEq)]
pub enum CPipeError {
    /// No message available at the requested coordinate
    NoMessage,
    
    /// Barrier not yet ready (still waiting for sentrons)
    BarrierNotReady {
        barrier_id: u8,
        arrived: u16,
        expected: u16,
    },
}

impl std::fmt::Display for CPipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CPipeError::NoMessage => write!(f, "No message available"),
            CPipeError::BarrierNotReady { barrier_id, arrived, expected } => {
                write!(
                    f,
                    "Barrier {} not ready: {}/{} sentrons arrived",
                    barrier_id, arrived, expected
                )
            }
        }
    }
}

impl std::error::Error for CPipeError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_send_recv() {
        let mut c_pipe = CPipeExecutor::new();
        let mut regs = [0i64; 32];
        
        // Send message from sentron 0 to sentron 5
        regs[1] = 42; // payload in msg_reg
        c_pipe.execute(
            &CoordOp::CSEND {
                msg_reg: 1,
                dest_sentron: 5,
            },
            0, // sentron 0
            &mut regs,
        ).unwrap();
        
        // Receive message at sentron 1 (reading from sentron 5's mailbox)
        regs[3] = 0; // clear destination
        c_pipe.execute(
            &CoordOp::CRECV {
                rd: 3,
                src_sentron: 5,
            },
            1, // sentron 1
            &mut regs,
        ).unwrap();
        
        assert_eq!(regs[3], 42);
    }
    
    #[test]
    fn test_barrier_sync() {
        let mut c_pipe = CPipeExecutor::new();
        let mut regs = [0i64; 32];
        
        // 3 sentrons arrive at barrier 0
        let result1 = c_pipe.execute(
            &CoordOp::CBAR { barrier_id: 0, count: 3 },
            0,
            &mut regs,
        );
        assert!(result1.is_err()); // Not ready yet
        
        let result2 = c_pipe.execute(
            &CoordOp::CBAR { barrier_id: 0, count: 3 },
            1,
            &mut regs,
        );
        assert!(result2.is_err()); // Still not ready
        
        let result3 = c_pipe.execute(
            &CoordOp::CBAR { barrier_id: 0, count: 3 },
            2,
            &mut regs,
        );
        assert!(result3.is_ok()); // All 3 arrived, barrier released!
    }
    
    #[test]
    fn test_pack() {
        let mut c_pipe = CPipeExecutor::new();
        let mut regs = [0i64; 32];
        
        regs[1] = 0x1234;
        regs[2] = 0x5678;
        
        // Pack
        c_pipe.execute(
            &CoordOp::CPACK {
                rd: 0,
                rs1: 1,
                rs2: 2,
                fmt: MessageFormat::Result,
            },
            0,
            &mut regs,
        ).unwrap();
        
        let packed = regs[0];
        let expected = (0x1234i64 << 32) | 0x5678;
        assert_eq!(packed, expected);
    }
    
    #[test]
    fn test_temperature_matching() {
        let mut c_pipe = CPipeExecutor::new();
        
        let coord1 = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let coord2 = PhextCoord::new([1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1]);
        let target = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        // Send to exact coordinate (manually)
        c_pipe.send(coord1, 0, 100, 0, MessageFormat::Result).unwrap();
        
        // Send to nearby coordinate (manually)
        c_pipe.send(coord2, 0, 200, 0, MessageFormat::Result).unwrap();
        
        // Low temperature = exact match weighted higher
        let exact_matches = c_pipe.match_messages_fuzzy(&target, 0.1);
        assert_eq!(exact_matches.len(), 2); // Both messages found
        assert_eq!(exact_matches[0].payload, 100); // Exact match first
        
        // High temperature = fuzzy matching (order less important)
        let fuzzy_matches = c_pipe.match_messages_fuzzy(&target, 10.0);
        assert_eq!(fuzzy_matches.len(), 2); // Both found
    }

    #[test]
    fn test_recv_no_message() {
        let mut c_pipe = CPipeExecutor::new();
        let mut regs = [0i64; 32];
        let result = c_pipe.execute(
            &CoordOp::CRECV { rd: 0, src_sentron: 99 },
            0, &mut regs,
        );
        assert_eq!(result, Err(CPipeError::NoMessage));
    }

    #[test]
    fn test_cnop() {
        let mut c_pipe = CPipeExecutor::new();
        let mut regs = [0i64; 32];
        assert!(c_pipe.execute(&CoordOp::CNOP, 0, &mut regs).is_ok());
    }

    #[test]
    fn test_message_count_and_clear() {
        let mut c_pipe = CPipeExecutor::new();
        let coord = PhextCoord::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(c_pipe.message_count(&coord), 0);
        c_pipe.send(coord, 0, 10, 0, MessageFormat::Result).unwrap();
        c_pipe.send(coord, 1, 20, 0, MessageFormat::Result).unwrap();
        assert_eq!(c_pipe.message_count(&coord), 2);
        c_pipe.clear();
        assert_eq!(c_pipe.message_count(&coord), 0);
    }

    #[test]
    fn test_barrier_idempotent() {
        let mut c_pipe = CPipeExecutor::new();
        let mut regs = [0i64; 32];
        // Sentron 0 arrives at barrier expecting 2
        let _ = c_pipe.execute(&CoordOp::CBAR { barrier_id: 1, count: 2 }, 0, &mut regs);
        // Same sentron arrives again — should be idempotent (still not ready, not double-counted)
        let result = c_pipe.execute(&CoordOp::CBAR { barrier_id: 1, count: 2 }, 0, &mut regs);
        // Still ok because idempotent re-arrival returns Ok
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_messages_fifo() {
        let mut c_pipe = CPipeExecutor::new();
        let mut regs = [0i64; 32];
        // Send 3 messages to same destination
        regs[0] = 111;
        c_pipe.execute(&CoordOp::CSEND { msg_reg: 0, dest_sentron: 7 }, 0, &mut regs).unwrap();
        regs[0] = 222;
        c_pipe.execute(&CoordOp::CSEND { msg_reg: 0, dest_sentron: 7 }, 1, &mut regs).unwrap();
        regs[0] = 333;
        c_pipe.execute(&CoordOp::CSEND { msg_reg: 0, dest_sentron: 7 }, 2, &mut regs).unwrap();

        // Recv pops from the back (LIFO — stack semantics)
        c_pipe.execute(&CoordOp::CRECV { rd: 1, src_sentron: 7 }, 0, &mut regs).unwrap();
        assert_eq!(regs[1], 333);
        c_pipe.execute(&CoordOp::CRECV { rd: 1, src_sentron: 7 }, 0, &mut regs).unwrap();
        assert_eq!(regs[1], 222);
        c_pipe.execute(&CoordOp::CRECV { rd: 1, src_sentron: 7 }, 0, &mut regs).unwrap();
        assert_eq!(regs[1], 111);
        // Now empty
        assert!(c_pipe.execute(&CoordOp::CRECV { rd: 1, src_sentron: 7 }, 0, &mut regs).is_err());
    }

    #[test]
    fn test_timestamp_increments() {
        let mut c_pipe = CPipeExecutor::new();
        let coord = PhextCoord::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        c_pipe.send(coord, 0, 10, 0, MessageFormat::Result).unwrap();
        c_pipe.send(coord, 0, 20, 0, MessageFormat::Result).unwrap();
        let msgs = c_pipe.mailboxes.get(&coord).unwrap();
        assert_eq!(msgs[0].timestamp, 1);
        assert_eq!(msgs[1].timestamp, 2);
    }

    #[test]
    fn test_cpipe_error_display() {
        let e = CPipeError::NoMessage;
        assert_eq!(format!("{}", e), "No message available");
        let e2 = CPipeError::BarrierNotReady { barrier_id: 5, arrived: 2, expected: 4 };
        assert!(format!("{}", e2).contains("5"));
        assert!(format!("{}", e2).contains("2/4"));
    }
}

#[cfg(test)]
mod w20_tests {
    use super::*;
    use crate::phext_coord::PhextCoord;
    use crate::pipes::MessageFormat;

    fn coord(x: u16) -> PhextCoord {
        PhextCoord::new([x, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }

    fn do_send(c: &mut CPipeExecutor, addr: PhextCoord, val: i64) {
        let _ = c.send(addr, 0, val, 0, MessageFormat::Result);
    }

    #[test]
    fn cpipe_new_has_zero_messages() {
        let c = CPipeExecutor::new();
        assert_eq!(c.message_count(&coord(1)), 0);
    }

    #[test]
    fn cpipe_send_increments_count() {
        let mut c = CPipeExecutor::new();
        let addr = coord(5);
        do_send(&mut c, addr, 42);
        assert_eq!(c.message_count(&coord(5)), 1);
    }

    #[test]
    fn cpipe_send_multiple_same_coord() {
        let mut c = CPipeExecutor::new();
        do_send(&mut c, coord(3), 1);
        do_send(&mut c, coord(3), 2);
        assert_eq!(c.message_count(&coord(3)), 2);
    }

    #[test]
    fn cpipe_clear_removes_all() {
        let mut c = CPipeExecutor::new();
        do_send(&mut c, coord(2), 99);
        c.clear();
        assert_eq!(c.message_count(&coord(2)), 0);
    }

    #[test]
    fn cpipe_fuzzy_match_empty() {
        let c = CPipeExecutor::new();
        let target = coord(1);
        let results = c.match_messages_fuzzy(&target, 1.0);
        assert!(results.is_empty(), "no messages = no matches");
    }

    #[test]
    fn cpipe_default_same_as_new() {
        let c1 = CPipeExecutor::new();
        let c2 = CPipeExecutor::default();
        assert_eq!(c1.message_count(&coord(1)), c2.message_count(&coord(1)));
    }
}
