//! State Stream — Efficient temporal delta streaming across the mesh
//!
//! Core insight: streaming full state is O(state_size). Streaming deltas
//! between temporal blocks is O(change_size). Temporal jumps compress
//! this further: if node B already has block T-5, only send T-5→T.
//!
//! Two transmission modes:
//!   Δ-stream: sequential deltas (T0→T1→T2...) for continuous sync
//!   τ-jump:   direct jump to target block (skip unchanged history)
//!             Node sends: "I have block T-5, give me T-current"
//!             Sender responds: δ(T-5 → T-current) only
//!
//! Wire format (compact, SQ-publishable):
//!   DELTA|from_seq|to_seq|fork_id|hash|size|<base64_delta>
//!   JUMP|from_seq|to_seq|fork_id|<base64_full_state>
//!   ACK|seq|fork_id|hash
//!   NEED|from_seq|fork_id   (request: "I have this block, send forward")
//!
//! SQ coordinates:
//!   state-stream/<owner>/<fork_id>.1.1/1.1.1/1.1.1  — outbound stream
//!   state-ack/<owner>/<node_idx>/1.1.1/1.1.1/1.1.1  — acknowledgements

use crate::ttsm::{TTSM, TemporalBlock, TemporalBlockId, TTSMError, TTSMResult};
use crate::PhextCoord;
use std::collections::HashMap;

// ── Delta encoding ────────────────────────────────────────────────────────────

/// A state delta: minimum bytes needed to go from `from` to `to`
#[derive(Debug, Clone)]
pub struct StateDelta {
    pub from_seq: u64,
    pub to_seq: u64,
    pub fork_id: u64,
    pub state_hash: u64,
    /// XOR delta for equal-size states, or full `to` state if size changed
    pub payload: DeltaPayload,
}

#[derive(Debug, Clone)]
pub enum DeltaPayload {
    /// XOR-encoded delta (when from.size == to.size)
    /// Apply: to_state = from_state XOR bytes
    Xor(Vec<u8>),
    /// Size changed — send full new state with change annotation
    Full(Vec<u8>),
    /// No change — just a sequence heartbeat
    Noop,
}

impl StateDelta {
    /// Compute delta between two state snapshots
    pub fn compute(from_seq: u64, to_seq: u64, fork_id: u64,
                   from_data: &[u8], to_data: &[u8], to_hash: u64) -> Self {
        let payload = if from_data.is_empty() || from_data.len() != to_data.len() {
            DeltaPayload::Full(to_data.to_vec())
        } else {
            // XOR delta: only non-zero bytes represent changes
            let xor: Vec<u8> = from_data.iter().zip(to_data.iter())
                .map(|(a, b)| a ^ b)
                .collect();
            let change_bytes = xor.iter().filter(|&&b| b != 0).count();
            if change_bytes == 0 {
                DeltaPayload::Noop
            } else if change_bytes < to_data.len() / 2 {
                // Sparse: run-length encode changed bytes only
                let mut sparse = Vec::new();
                let mut i = 0;
                while i < xor.len() {
                    if xor[i] != 0 {
                        // Find run of changed bytes
                        let start = i;
                        while i < xor.len() && xor[i] != 0 { i += 1; }
                        // Encode: [start_offset:4][len:2][bytes...]
                        let offset = (start as u32).to_le_bytes();
                        let len = ((i - start) as u16).to_le_bytes();
                        sparse.extend_from_slice(&offset);
                        sparse.extend_from_slice(&len);
                        sparse.extend_from_slice(&xor[start..i]);
                    } else {
                        i += 1;
                    }
                }
                DeltaPayload::Xor(sparse)
            } else {
                DeltaPayload::Full(to_data.to_vec())
            }
        };

        StateDelta { from_seq, to_seq, fork_id, state_hash: to_hash, payload }
    }

    /// Apply this delta to a base state
    pub fn apply(&self, base: &[u8]) -> Vec<u8> {
        match &self.payload {
            DeltaPayload::Full(data) => data.clone(),
            DeltaPayload::Noop => base.to_vec(),
            DeltaPayload::Xor(sparse) => {
                let mut result = base.to_vec();
                let mut i = 0;
                while i + 6 <= sparse.len() {
                    let offset = u32::from_le_bytes([sparse[i], sparse[i+1], sparse[i+2], sparse[i+3]]) as usize;
                    let len = u16::from_le_bytes([sparse[i+4], sparse[i+5]]) as usize;
                    i += 6;
                    if offset + len <= result.len() && i + len <= sparse.len() {
                        for j in 0..len {
                            result[offset + j] ^= sparse[i + j];
                        }
                    }
                    i += len;
                }
                result
            }
        }
    }

    /// Serialize to SQ-publishable wire format
    pub fn to_wire(&self) -> String {
        let (tag, data) = match &self.payload {
            DeltaPayload::Noop =>
                return format!("NOOP|{}|{}|{}|{}", self.from_seq, self.to_seq, self.fork_id, self.state_hash),
            DeltaPayload::Xor(b) => ("DELTA", b),
            DeltaPayload::Full(b) => ("FULL", b),
        };
        let b64 = base64_encode(data);
        format!("{}|{}|{}|{}|{}|{}", tag, self.from_seq, self.to_seq, self.fork_id, self.state_hash, b64)
    }

    /// Parse from wire format
    pub fn from_wire(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(6, '|').collect();
        if parts.len() < 4 { return None; }
        let tag = parts[0];
        let from_seq: u64 = parts[1].parse().ok()?;
        let to_seq: u64 = parts[2].parse().ok()?;
        let fork_id: u64 = parts[3].parse().ok()?;
        let state_hash: u64 = if parts.len() > 4 { parts[4].parse().ok()? } else { 0 };
        let payload = match tag {
            "NOOP" => DeltaPayload::Noop,
            "DELTA" => DeltaPayload::Xor(base64_decode(parts.get(5).unwrap_or(&""))),
            "FULL" | _ => DeltaPayload::Full(base64_decode(parts.get(5).unwrap_or(&""))),
        };
        Some(StateDelta { from_seq, to_seq, fork_id, state_hash, payload })
    }

    /// Compression ratio: how much smaller is this than full state?
    pub fn compression_ratio(&self, full_state_size: usize) -> f64 {
        let delta_size = match &self.payload {
            DeltaPayload::Noop => 0,
            DeltaPayload::Xor(b) | DeltaPayload::Full(b) => b.len(),
        };
        if full_state_size == 0 { return 1.0; }
        1.0 - (delta_size as f64 / full_state_size as f64)
    }
}

// ── Temporal jump protocol ────────────────────────────────────────────────────

/// A temporal jump request: "I have block T_from, send me T_to"
#[derive(Debug, Clone)]
pub struct TemporalJumpRequest {
    pub owner: String,
    pub from_seq: u64,    // block the requester already has
    pub from_fork: u64,
    pub to_seq: u64,      // block they want
    pub to_fork: u64,
    pub requester_node: u16,
}

impl TemporalJumpRequest {
    pub fn to_wire(&self) -> String {
        format!("NEED|{}|{}|{}|{}|{}|{}",
            self.owner, self.from_seq, self.from_fork,
            self.to_seq, self.to_fork, self.requester_node)
    }
}

// ── State stream manager ──────────────────────────────────────────────────────

/// Manages efficient delta streaming for a TTSM owner
pub struct StateStreamManager {
    /// Per-owner state history (for delta computation)
    state_cache: HashMap<String, Vec<u8>>,
    /// Per-owner last-sent sequence
    last_sent: HashMap<String, u64>,
    /// Statistics
    pub bytes_sent: u64,
    pub bytes_saved: u64,
    pub deltas_sent: u64,
    pub jumps_served: u64,
}

impl StateStreamManager {
    pub fn new() -> Self {
        Self {
            state_cache: HashMap::new(),
            last_sent: HashMap::new(),
            bytes_sent: 0,
            bytes_saved: 0,
            deltas_sent: 0,
            jumps_served: 0,
        }
    }

    /// Stream a new committed state. Returns the delta wire string.
    /// If this is the first state, returns a FULL payload.
    pub fn stream_commit(&mut self, owner: &str, seq: u64, fork_id: u64,
                         new_state: &[u8], state_hash: u64) -> String {
        let from_seq = *self.last_sent.get(owner).unwrap_or(&0);
        let prev = self.state_cache.get(owner).cloned().unwrap_or_default();

        let delta = StateDelta::compute(from_seq, seq, fork_id, &prev, new_state, state_hash);
        let wire = delta.to_wire();

        let full_size = new_state.len();
        let wire_size = wire.len();
        self.bytes_sent += wire_size as u64;
        self.bytes_saved += full_size.saturating_sub(wire_size) as u64;
        self.deltas_sent += 1;

        self.state_cache.insert(owner.to_string(), new_state.to_vec());
        self.last_sent.insert(owner.to_string(), seq);

        wire
    }

    /// Serve a temporal jump request — compute delta from request's base to current
    pub fn serve_jump(&mut self, req: &TemporalJumpRequest,
                      ttsm: &TTSM, owner_states: &HashMap<u64, Vec<u8>>) -> Option<String> {
        // Get the state at from_seq and to_seq
        let from_state = owner_states.get(&req.from_seq).cloned().unwrap_or_default();
        let to_state = owner_states.get(&req.to_seq).cloned().unwrap_or_default();

        if to_state.is_empty() { return None; }

        let to_hash = fnv1a_64(&to_state);
        let delta = StateDelta::compute(
            req.from_seq, req.to_seq, req.to_fork,
            &from_state, &to_state, to_hash
        );

        self.jumps_served += 1;
        let wire = delta.to_wire();
        self.bytes_sent += wire.len() as u64;
        let saved = to_state.len().saturating_sub(wire.len());
        self.bytes_saved += saved as u64;

        Some(wire)
    }

    /// Compression efficiency summary
    pub fn stats_summary(&self) -> String {
        let total = self.bytes_sent + self.bytes_saved;
        let efficiency = if total > 0 {
            self.bytes_saved as f64 / total as f64 * 100.0
        } else { 0.0 };
        format!(
            "StateStream: sent={} saved={} efficiency={:.1}% deltas={} jumps={}",
            self.bytes_sent, self.bytes_saved, efficiency,
            self.deltas_sent, self.jumps_served
        )
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// FNV-1a 64-bit hash (zero-dep)
pub fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

/// Minimal base64 encode (no external deps)
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { CHARS[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(n & 63) as usize] as char } else { '=' });
    }
    out
}

/// Minimal base64 decode
fn base64_decode(s: &str) -> Vec<u8> {
    let decode_char = |c: char| -> u32 {
        match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' => 62, '/' => 63, _ => 0,
        }
    };
    let mut out = Vec::new();
    let bytes: Vec<char> = s.chars().filter(|&c| c != '=').collect();
    for chunk in bytes.chunks(4) {
        let n = chunk.iter().fold(0u32, |acc, &c| (acc << 6) | decode_char(c));
        match chunk.len() {
            4 => { out.push((n >> 16) as u8); out.push((n >> 8) as u8); out.push(n as u8); }
            3 => { out.push((n >> 10) as u8); out.push((n >> 2) as u8); }
            2 => { out.push((n >> 4) as u8); }
            _ => {}
        }
    }
    out
}
