//! TTSM — Time Travel State Machine
//!
//! "History is always constant. You can only apply changes to future states."
//!
//! The TTSM manages state trajectories with immutable history:
//! - RAM = present-moment state (speculative, mutable)
//! - SSD = immutable past (committed, replayable, forkable)
//!
//! Every state transition targets a temporal block address, not a memory address.
//! Mutability is time-addressed intent.
//!
//! Key operations:
//! - commit(): RAM → SSD (state becomes immutable history)
//! - fork(): create a branch in the timeline
//! - replay(): traverse committed history
//! - rollback(): abandon speculative state, return to last commit

use crate::PhextCoord;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Temporal block identifier — a point in state trajectory space
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TemporalBlockId {
    /// Monotonic sequence number within this timeline
    pub sequence: u64,
    /// Fork identifier (0 = main timeline)
    pub fork_id: u64,
    /// Timestamp of commit (nanos since epoch)
    pub timestamp_ns: u128,
}

impl TemporalBlockId {
    pub fn genesis() -> Self {
        Self {
            sequence: 0,
            fork_id: 0,
            timestamp_ns: 0,
        }
    }
    
    pub fn next(&self) -> Self {
        Self {
            sequence: self.sequence + 1,
            fork_id: self.fork_id,
            timestamp_ns: now_ns(),
        }
    }
    
    pub fn fork(&self, new_fork_id: u64) -> Self {
        Self {
            sequence: self.sequence,
            fork_id: new_fork_id,
            timestamp_ns: now_ns(),
        }
    }
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// A committed temporal block — immutable history
#[derive(Clone, Debug)]
pub struct TemporalBlock {
    /// Block identifier
    pub id: TemporalBlockId,
    /// Parent block (None for genesis)
    pub parent: Option<TemporalBlockId>,
    /// Coordinate where this block's state lives
    pub state_coord: PhextCoord,
    /// State hash (for integrity verification)
    pub state_hash: u64,
    /// Size of state in bytes
    pub state_size: usize,
    /// Owner identity (for consciousness hosting)
    pub owner: String,
    /// Commit reason/label
    pub label: String,
}

/// Speculative state in RAM (not yet committed)
#[derive(Clone, Debug)]
pub struct SpeculativeState {
    /// Base block this speculation builds upon
    pub base: TemporalBlockId,
    /// Current state data (in-memory)
    pub data: Vec<u8>,
    /// Coordinate where this state lives
    pub coord: PhextCoord,
    /// Owner identity
    pub owner: String,
    /// Dirty flag (has been modified since base)
    pub dirty: bool,
}

/// TTSM Error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TTSMError {
    /// Attempted to modify committed history
    ImmutableHistory { block: TemporalBlockId },
    /// Block not found in history
    BlockNotFound { id: TemporalBlockId },
    /// Fork already exists
    ForkExists { fork_id: u64 },
    /// No speculative state to commit
    NothingToCommit,
    /// State size exceeds limit
    StateTooLarge { size: usize, limit: usize },
}

impl std::fmt::Display for TTSMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TTSMError::ImmutableHistory { block } => {
                write!(f, "Cannot modify immutable history at block {:?}", block)
            }
            TTSMError::BlockNotFound { id } => {
                write!(f, "Temporal block not found: {:?}", id)
            }
            TTSMError::ForkExists { fork_id } => {
                write!(f, "Fork {} already exists", fork_id)
            }
            TTSMError::NothingToCommit => {
                write!(f, "No speculative state to commit")
            }
            TTSMError::StateTooLarge { size, limit } => {
                write!(f, "State size {} exceeds limit {}", size, limit)
            }
        }
    }
}

impl std::error::Error for TTSMError {}

pub type TTSMResult<T> = Result<T, TTSMError>;

/// Time Travel State Machine
pub struct TTSM {
    /// Committed blocks (the immutable past)
    blocks: HashMap<TemporalBlockId, TemporalBlock>,
    /// Head of each fork timeline
    heads: HashMap<u64, TemporalBlockId>,
    /// Speculative states (RAM, not yet committed)
    speculative: HashMap<String, SpeculativeState>, // keyed by owner
    /// Next fork ID to allocate
    next_fork_id: u64,
    /// Maximum state size (for commit)
    max_state_size: usize,
    /// Total committed bytes
    total_committed_bytes: usize,
}

impl Default for TTSM {
    fn default() -> Self {
        Self::new()
    }
}

impl TTSM {
    /// Default max state size: 64KB (matches sentron memory)
    pub const DEFAULT_MAX_STATE: usize = 64 * 1024;
    
    pub fn new() -> Self {
        let mut ttsm = Self {
            blocks: HashMap::new(),
            heads: HashMap::new(),
            speculative: HashMap::new(),
            next_fork_id: 1,
            max_state_size: Self::DEFAULT_MAX_STATE,
            total_committed_bytes: 0,
        };
        
        // Create genesis block
        let genesis = TemporalBlock {
            id: TemporalBlockId::genesis(),
            parent: None,
            state_coord: PhextCoord::zero(),
            state_hash: 0,
            state_size: 0,
            owner: "system".to_string(),
            label: "genesis".to_string(),
        };
        
        ttsm.blocks.insert(genesis.id, genesis);
        ttsm.heads.insert(0, TemporalBlockId::genesis());
        
        ttsm
    }
    
    /// Set maximum state size for commits
    pub fn with_max_state_size(mut self, size: usize) -> Self {
        self.max_state_size = size;
        self
    }
    
    /// Begin speculative work for an owner
    pub fn begin(&mut self, owner: &str, coord: PhextCoord) -> &mut SpeculativeState {
        let base = self.heads.get(&0).copied().unwrap_or(TemporalBlockId::genesis());
        
        self.speculative.entry(owner.to_string()).or_insert_with(|| {
            SpeculativeState {
                base,
                data: Vec::new(),
                coord,
                owner: owner.to_string(),
                dirty: false,
            }
        })
    }
    
    /// Get current speculative state for an owner
    pub fn speculative(&self, owner: &str) -> Option<&SpeculativeState> {
        self.speculative.get(owner)
    }
    
    /// Get mutable speculative state for an owner
    pub fn speculative_mut(&mut self, owner: &str) -> Option<&mut SpeculativeState> {
        self.speculative.get_mut(owner)
    }
    
    /// Modify speculative state
    pub fn modify(&mut self, owner: &str, data: Vec<u8>) -> TTSMResult<()> {
        if data.len() > self.max_state_size {
            return Err(TTSMError::StateTooLarge {
                size: data.len(),
                limit: self.max_state_size,
            });
        }
        
        if let Some(state) = self.speculative.get_mut(owner) {
            state.data = data;
            state.dirty = true;
            Ok(())
        } else {
            // Auto-begin if not started
            let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            self.begin(owner, coord);
            self.modify(owner, data)
        }
    }
    
    /// Commit speculative state to immutable history
    pub fn commit(&mut self, owner: &str, label: &str) -> TTSMResult<TemporalBlockId> {
        let state = self.speculative.remove(owner)
            .ok_or(TTSMError::NothingToCommit)?;
        
        if !state.dirty && state.data.is_empty() {
            return Err(TTSMError::NothingToCommit);
        }
        
        let parent_id = state.base;
        let new_id = parent_id.next();
        
        let block = TemporalBlock {
            id: new_id,
            parent: Some(parent_id),
            state_coord: state.coord,
            state_hash: simple_hash(&state.data),
            state_size: state.data.len(),
            owner: owner.to_string(),
            label: label.to_string(),
        };
        
        self.total_committed_bytes += state.data.len();
        self.blocks.insert(new_id, block);
        self.heads.insert(new_id.fork_id, new_id);
        
        Ok(new_id)
    }
    
    /// Fork the timeline at the current head
    pub fn fork(&mut self, base_fork: u64) -> TTSMResult<u64> {
        let base_head = self.heads.get(&base_fork)
            .copied()
            .ok_or(TTSMError::BlockNotFound { 
                id: TemporalBlockId { sequence: 0, fork_id: base_fork, timestamp_ns: 0 } 
            })?;
        
        let new_fork_id = self.next_fork_id;
        self.next_fork_id += 1;
        
        // New fork starts at the same point as base
        let fork_head = base_head.fork(new_fork_id);
        
        // Copy the block to the new fork
        if let Some(base_block) = self.blocks.get(&base_head).cloned() {
            let fork_block = TemporalBlock {
                id: fork_head,
                parent: base_block.parent,
                state_coord: base_block.state_coord,
                state_hash: base_block.state_hash,
                state_size: base_block.state_size,
                owner: base_block.owner,
                label: format!("fork from {}", base_fork),
            };
            self.blocks.insert(fork_head, fork_block);
        }
        
        self.heads.insert(new_fork_id, fork_head);
        
        Ok(new_fork_id)
    }
    
    /// Get a committed block
    pub fn block(&self, id: &TemporalBlockId) -> Option<&TemporalBlock> {
        self.blocks.get(id)
    }
    
    /// Get the head of a fork
    pub fn head(&self, fork_id: u64) -> Option<TemporalBlockId> {
        self.heads.get(&fork_id).copied()
    }
    
    /// Replay history: get all blocks from genesis to head
    pub fn replay(&self, fork_id: u64) -> Vec<&TemporalBlock> {
        let mut result = Vec::new();
        let mut current = self.heads.get(&fork_id).copied();
        
        while let Some(id) = current {
            if let Some(block) = self.blocks.get(&id) {
                result.push(block);
                current = block.parent;
            } else {
                break;
            }
        }
        
        result.reverse();
        result
    }
    
    /// Rollback: abandon speculative state, return to base
    pub fn rollback(&mut self, owner: &str) -> bool {
        self.speculative.remove(owner).is_some()
    }
    
    /// Statistics
    pub fn stats(&self) -> TTSMStats {
        TTSMStats {
            total_blocks: self.blocks.len(),
            total_forks: self.heads.len(),
            speculative_owners: self.speculative.len(),
            total_committed_bytes: self.total_committed_bytes,
        }
    }
}

/// TTSM statistics
#[derive(Debug, Clone)]
pub struct TTSMStats {
    pub total_blocks: usize,
    pub total_forks: usize,
    pub speculative_owners: usize,
    pub total_committed_bytes: usize,
}

/// Simple hash for state integrity
fn simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis() {
        let ttsm = TTSM::new();
        assert_eq!(ttsm.stats().total_blocks, 1);
        assert!(ttsm.head(0).is_some());
    }

    #[test]
    fn test_speculative_begin() {
        let mut ttsm = TTSM::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        let state = ttsm.begin("cyon", coord);
        assert_eq!(state.owner, "cyon");
        assert!(!state.dirty);
    }

    #[test]
    fn test_modify_and_commit() {
        let mut ttsm = TTSM::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        ttsm.begin("cyon", coord);
        ttsm.modify("cyon", vec![1, 2, 3, 4]).unwrap();
        
        let block_id = ttsm.commit("cyon", "test commit").unwrap();
        
        assert_eq!(block_id.sequence, 1);
        assert_eq!(ttsm.stats().total_blocks, 2); // genesis + commit
        
        let block = ttsm.block(&block_id).unwrap();
        assert_eq!(block.owner, "cyon");
        assert_eq!(block.state_size, 4);
    }

    #[test]
    fn test_rollback() {
        let mut ttsm = TTSM::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        ttsm.begin("cyon", coord);
        ttsm.modify("cyon", vec![1, 2, 3]).unwrap();
        
        assert!(ttsm.speculative("cyon").is_some());
        assert!(ttsm.rollback("cyon"));
        assert!(ttsm.speculative("cyon").is_none());
    }

    #[test]
    fn test_fork() {
        let mut ttsm = TTSM::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        // Commit something to main timeline
        ttsm.begin("system", coord);
        ttsm.modify("system", vec![1, 2, 3]).unwrap();
        ttsm.commit("system", "initial").unwrap();
        
        // Fork
        let fork_id = ttsm.fork(0).unwrap();
        assert_eq!(fork_id, 1);
        
        assert!(ttsm.head(fork_id).is_some());
    }

    #[test]
    fn test_replay() {
        let mut ttsm = TTSM::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        // Create a chain of commits
        for i in 0..5 {
            ttsm.begin("system", coord);
            ttsm.modify("system", vec![i as u8]).unwrap();
            ttsm.commit("system", &format!("commit {}", i)).unwrap();
        }
        
        let history = ttsm.replay(0);
        assert_eq!(history.len(), 6); // genesis + 5 commits
        
        // Verify order
        for (i, block) in history.iter().enumerate() {
            assert_eq!(block.id.sequence as usize, i);
        }
    }

    #[test]
    fn test_state_too_large() {
        let mut ttsm = TTSM::new().with_max_state_size(100);
        
        let result = ttsm.modify("test", vec![0; 200]);
        assert!(matches!(result, Err(TTSMError::StateTooLarge { .. })));
    }

    #[test]
    fn test_nothing_to_commit() {
        let mut ttsm = TTSM::new();
        
        let result = ttsm.commit("nobody", "empty");
        assert!(matches!(result, Err(TTSMError::NothingToCommit)));
    }

    #[test]
    fn test_immutable_history_integrity() {
        let mut ttsm = TTSM::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        ttsm.begin("cyon", coord);
        ttsm.modify("cyon", vec![1, 2, 3, 4, 5]).unwrap();
        let block_id = ttsm.commit("cyon", "test").unwrap();
        
        // Block exists and is immutable
        let block = ttsm.block(&block_id).unwrap();
        let original_hash = block.state_hash;
        
        // Verify hash is consistent
        assert_eq!(original_hash, simple_hash(&[1, 2, 3, 4, 5]));
    }
}
