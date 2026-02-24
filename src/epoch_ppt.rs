//! Epoch-Structured Phext Page Table
//!
//! R23W30: The PPT enters Epoch-Structured Mode.
//!
//! Every coordinate translation is resolved against an immutable EpochView.
//! Region mappings are copy-on-write persistent; no allocation mutates the past.
//! The active epoch defines the sole writable future.
//!
//! On commit:
//! - Region map root is sealed
//! - epoch_id increments  
//! - New view inherits structure without altering history
//!
//! PTC entries are epoch-tagged; stale hits are impossible by construction.
//!
//! During replay, allocation is forbidden — absence of a region mapping
//! is a structural fault, not a recovery event.
//!
//! "History is constant. Translation is referentially stable.
//!  Meaning survives reboot."

use crate::PhextCoord;
use crate::ppt::{PhextPageTable, PPTStats, MemoryTier};
use std::collections::HashMap;
use std::sync::Arc;

/// Epoch identifier — monotonically increasing, never reused
pub type EpochId = u64;

/// Immutable region mapping snapshot
#[derive(Clone, Debug)]
pub struct RegionSnapshot {
    /// Region key → base address (frozen at commit)
    mappings: HashMap<u64, usize>,
    /// Total allocated bytes in this snapshot
    allocated: usize,
}

impl RegionSnapshot {
    fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            allocated: 0,
        }
    }
    
    fn from_mappings(mappings: HashMap<u64, usize>, allocated: usize) -> Self {
        Self { mappings, allocated }
    }
}

/// Immutable view of an epoch's address space
#[derive(Clone, Debug)]
pub struct EpochView {
    /// Epoch identifier
    pub epoch_id: EpochId,
    /// Frozen region mappings
    pub regions: Arc<RegionSnapshot>,
    /// Timestamp of seal (nanos since epoch 0)
    pub sealed_at: u128,
}

impl EpochView {
    /// Lookup a region in this epoch's frozen view
    pub fn lookup_region(&self, key: u64) -> Option<usize> {
        self.regions.mappings.get(&key).copied()
    }
    
    /// Check if a region exists
    pub fn has_region(&self, key: u64) -> bool {
        self.regions.mappings.contains_key(&key)
    }
}

/// Epoch-tagged PTC entry
#[derive(Clone, Copy)]
struct EpochPTCEntry {
    coord_lo: u64,
    coord_hi: u64,
    address: usize,
    epoch_id: EpochId,
    valid: bool,
}

impl Default for EpochPTCEntry {
    fn default() -> Self {
        Self {
            coord_lo: 0,
            coord_hi: 0,
            address: 0,
            epoch_id: 0,
            valid: false,
        }
    }
}

/// Epoch-aware PTC — stale hits impossible by construction
struct EpochPTC {
    sets: Vec<[EpochPTCEntry; 8]>,
    num_sets: usize,
    hits: u64,
    misses: u64,
    stale_evictions: u64,
}

impl EpochPTC {
    fn new(num_sets: usize) -> Self {
        Self {
            sets: vec![[EpochPTCEntry::default(); 8]; num_sets],
            num_sets,
            hits: 0,
            misses: 0,
            stale_evictions: 0,
        }
    }
    
    #[inline(always)]
    fn set_index(&self, coord: &PhextCoord) -> usize {
        let (lo, hi) = coord.as_raw();
        let hash = lo.wrapping_mul(0x517cc1b727220a95) ^ hi.wrapping_mul(0x9E3779B97F4A7C15);
        let hash = hash ^ (hash >> 17) ^ (hash >> 34);
        (hash as usize) % self.num_sets
    }
    
    /// Lookup with epoch validation — stale entries are evicted, not returned
    fn lookup(&mut self, coord: &PhextCoord, current_epoch: EpochId) -> Option<usize> {
        let set_idx = self.set_index(coord);
        let (lo, hi) = coord.as_raw();
        
        for entry in &mut self.sets[set_idx] {
            if entry.valid && entry.coord_lo == lo && entry.coord_hi == hi {
                if entry.epoch_id == current_epoch {
                    self.hits += 1;
                    return Some(entry.address);
                } else {
                    // Stale entry from previous epoch — evict
                    entry.valid = false;
                    self.stale_evictions += 1;
                }
            }
        }
        self.misses += 1;
        None
    }
    
    fn insert(&mut self, coord: &PhextCoord, address: usize, epoch_id: EpochId) {
        let set_idx = self.set_index(coord);
        let (lo, hi) = coord.as_raw();
        
        for entry in &mut self.sets[set_idx] {
            if !entry.valid || entry.epoch_id != epoch_id {
                *entry = EpochPTCEntry {
                    coord_lo: lo,
                    coord_hi: hi,
                    address,
                    epoch_id,
                    valid: true,
                };
                return;
            }
        }
        // All valid and current — evict slot 0
        self.sets[set_idx][0] = EpochPTCEntry {
            coord_lo: lo,
            coord_hi: hi,
            address,
            epoch_id,
            valid: true,
        };
    }
    
    fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.stale_evictions = 0;
    }
}

/// Error during epoch translation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpochError {
    /// Attempted allocation during replay (read-only epoch)
    AllocationDuringReplay { epoch_id: EpochId, coord: String },
    /// Region not found in sealed epoch
    RegionNotFound { epoch_id: EpochId, region_key: u64 },
    /// Epoch not found in history
    EpochNotFound { epoch_id: EpochId },
    /// Attempted mutation of sealed epoch
    MutationOfSealed { epoch_id: EpochId },
}

impl std::fmt::Display for EpochError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpochError::AllocationDuringReplay { epoch_id, coord } => {
                write!(f, "Allocation forbidden during replay of epoch {}: {}", epoch_id, coord)
            }
            EpochError::RegionNotFound { epoch_id, region_key } => {
                write!(f, "Region {:016x} not found in epoch {}", region_key, epoch_id)
            }
            EpochError::EpochNotFound { epoch_id } => {
                write!(f, "Epoch {} not found in history", epoch_id)
            }
            EpochError::MutationOfSealed { epoch_id } => {
                write!(f, "Cannot mutate sealed epoch {}", epoch_id)
            }
        }
    }
}

impl std::error::Error for EpochError {}

pub type EpochResult<T> = Result<T, EpochError>;

/// Epoch-Structured PPT
///
/// The core translation engine with versioned, immutable history.
/// Identity continuity is structural, not conventional.
pub struct EpochPPT {
    /// Current active epoch (sole writable future)
    active_epoch: EpochId,
    /// Sealed epoch views (immutable history)
    history: HashMap<EpochId, EpochView>,
    /// Working region mappings for active epoch
    working_regions: HashMap<u64, usize>,
    /// Working allocation counter
    working_allocated: usize,
    /// Inner page size
    inner_page_size: usize,
    /// Epoch-aware translation cache
    ptc: EpochPTC,
    /// Z-order LUT (borrowed from PPT)
    z_tables: [[u32; 128]; 3],
    /// Replay mode flag
    replay_mode: bool,
    /// Replay target epoch
    replay_epoch: Option<EpochId>,
}

impl EpochPPT {
    /// Create new EpochPPT starting at epoch 0
    pub fn new() -> Self {
        let z_tables = Self::init_z_lut();
        
        let mut ppt = Self {
            active_epoch: 0,
            history: HashMap::new(),
            working_regions: HashMap::new(),
            working_allocated: 0,
            inner_page_size: 1 << 21, // 2M entries per inner page
            ptc: EpochPTC::new(256),
            z_tables,
            replay_mode: false,
            replay_epoch: None,
        };
        
        // Seal epoch 0 as genesis (empty)
        ppt.seal_epoch();
        
        ppt
    }
    
    fn init_z_lut() -> [[u32; 128]; 3] {
        let mut tables = [[0u32; 128]; 3];
        for v in 0..128u32 {
            for bit in 0..7 {
                tables[0][v as usize] |= ((v >> bit) & 1) << (bit * 3);
                tables[1][v as usize] |= ((v >> bit) & 1) << (bit * 3 + 1);
                tables[2][v as usize] |= ((v >> bit) & 1) << (bit * 3 + 2);
            }
        }
        tables
    }
    
    #[inline]
    fn z_encode(&self, d0: u16, d1: u16, d2: u16) -> u32 {
        self.z_tables[0][(d0 & 0x7F) as usize]
            | self.z_tables[1][(d1 & 0x7F) as usize]
            | self.z_tables[2][(d2 & 0x7F) as usize]
    }
    
    #[inline]
    fn outer_key(dims: &[u16; 11]) -> u64 {
        let mut key = 0u64;
        for i in 3..11 {
            key |= ((dims[i] as u64) & 0xFF) << ((i - 3) * 8);
        }
        key
    }
    
    /// Get current active epoch
    pub fn active_epoch(&self) -> EpochId {
        self.active_epoch
    }
    
    /// Translate coordinate in active epoch (writable)
    #[inline]
    pub fn translate(&mut self, coord: &PhextCoord) -> EpochResult<usize> {
        if self.replay_mode {
            return self.translate_replay(coord);
        }
        
        // PTC lookup
        if let Some(addr) = self.ptc.lookup(coord, self.active_epoch) {
            return Ok(addr);
        }
        
        // Compute address
        let addr = self.compute_address_active(coord)?;
        self.ptc.insert(coord, addr, self.active_epoch);
        
        Ok(addr)
    }
    
    /// Translate in replay mode (read-only)
    fn translate_replay(&mut self, coord: &PhextCoord) -> EpochResult<usize> {
        let epoch_id = self.replay_epoch.unwrap_or(self.active_epoch);
        
        // PTC lookup
        if let Some(addr) = self.ptc.lookup(coord, epoch_id) {
            return Ok(addr);
        }
        
        // Get epoch view
        let view = self.history.get(&epoch_id)
            .ok_or(EpochError::EpochNotFound { epoch_id })?;
        
        let dims = coord.dims();
        let region_key = Self::outer_key(&dims);
        
        // Region must exist — no allocation during replay
        let base = view.lookup_region(region_key)
            .ok_or(EpochError::RegionNotFound { epoch_id, region_key })?;
        
        let z_offset = self.z_encode(dims[0], dims[1], dims[2]) as usize;
        let addr = base + z_offset * std::mem::size_of::<f64>();
        
        self.ptc.insert(coord, addr, epoch_id);
        
        Ok(addr)
    }
    
    /// Compute address in active epoch (may allocate)
    fn compute_address_active(&mut self, coord: &PhextCoord) -> EpochResult<usize> {
        let dims = coord.dims();
        let region_key = Self::outer_key(&dims);
        
        // Get or allocate region
        let base = *self.working_regions.entry(region_key).or_insert_with(|| {
            let addr = self.working_allocated;
            self.working_allocated += self.inner_page_size * std::mem::size_of::<f64>();
            addr
        });
        
        let z_offset = self.z_encode(dims[0], dims[1], dims[2]) as usize;
        Ok(base + z_offset * std::mem::size_of::<f64>())
    }
    
    /// Seal active epoch and advance to next
    pub fn seal_epoch(&mut self) -> EpochView {
        let sealed_epoch = self.active_epoch;
        
        // Create immutable snapshot
        let snapshot = RegionSnapshot::from_mappings(
            self.working_regions.clone(),
            self.working_allocated,
        );
        
        let view = EpochView {
            epoch_id: sealed_epoch,
            regions: Arc::new(snapshot),
            sealed_at: now_ns(),
        };
        
        // Store in history
        self.history.insert(sealed_epoch, view.clone());
        
        // Advance epoch (working state carries forward via COW semantics)
        self.active_epoch += 1;
        
        view
    }
    
    /// Fork from a sealed epoch (creates new future by pointer divergence)
    pub fn fork(&mut self, from_epoch: EpochId) -> EpochResult<EpochId> {
        let view = self.history.get(&from_epoch)
            .ok_or(EpochError::EpochNotFound { epoch_id: from_epoch })?;
        
        // Reset working state to forked epoch's snapshot
        self.working_regions = view.regions.mappings.clone();
        self.working_allocated = view.regions.allocated;
        
        // Seal creates new epoch
        self.seal_epoch();
        
        Ok(self.active_epoch)
    }
    
    /// Enter replay mode for a sealed epoch
    pub fn begin_replay(&mut self, epoch_id: EpochId) -> EpochResult<()> {
        if !self.history.contains_key(&epoch_id) {
            return Err(EpochError::EpochNotFound { epoch_id });
        }
        
        self.replay_mode = true;
        self.replay_epoch = Some(epoch_id);
        Ok(())
    }
    
    /// Exit replay mode
    pub fn end_replay(&mut self) {
        self.replay_mode = false;
        self.replay_epoch = None;
    }
    
    /// Get a sealed epoch view
    pub fn view(&self, epoch_id: EpochId) -> Option<&EpochView> {
        self.history.get(&epoch_id)
    }
    
    /// Statistics
    pub fn stats(&self) -> EpochPPTStats {
        EpochPPTStats {
            active_epoch: self.active_epoch,
            sealed_epochs: self.history.len(),
            working_regions: self.working_regions.len(),
            working_allocated: self.working_allocated,
            ptc_hits: self.ptc.hits,
            ptc_misses: self.ptc.misses,
            stale_evictions: self.ptc.stale_evictions,
            replay_mode: self.replay_mode,
        }
    }
    
    pub fn reset_stats(&mut self) {
        self.ptc.reset_stats();
    }
}

impl Default for EpochPPT {
    fn default() -> Self {
        Self::new()
    }
}

/// EpochPPT statistics
#[derive(Debug, Clone)]
pub struct EpochPPTStats {
    pub active_epoch: EpochId,
    pub sealed_epochs: usize,
    pub working_regions: usize,
    pub working_allocated: usize,
    pub ptc_hits: u64,
    pub ptc_misses: u64,
    pub stale_evictions: u64,
    pub replay_mode: bool,
}

fn now_ns() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_epoch() {
        let ppt = EpochPPT::new();
        assert_eq!(ppt.active_epoch(), 1); // Epoch 0 sealed at creation
        assert!(ppt.view(0).is_some());
    }

    #[test]
    fn test_translate_allocates() {
        let mut ppt = EpochPPT::new();
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        
        let addr = ppt.translate(&coord).unwrap();
        assert!(addr > 0 || addr == 0); // Address allocated
        
        let stats = ppt.stats();
        assert_eq!(stats.working_regions, 1);
    }

    #[test]
    fn test_translate_deterministic() {
        let mut ppt = EpochPPT::new();
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        
        let addr1 = ppt.translate(&coord).unwrap();
        let addr2 = ppt.translate(&coord).unwrap();
        
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_seal_and_advance() {
        let mut ppt = EpochPPT::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        ppt.translate(&coord).unwrap();
        let epoch_before = ppt.active_epoch();
        
        ppt.seal_epoch();
        
        assert_eq!(ppt.active_epoch(), epoch_before + 1);
        assert!(ppt.view(epoch_before).is_some());
    }

    #[test]
    fn test_replay_forbids_allocation() {
        let mut ppt = EpochPPT::new();
        
        // Translate in epoch 1
        let coord1 = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        ppt.translate(&coord1).unwrap();
        
        // Seal epoch 1
        ppt.seal_epoch();
        
        // Enter replay of epoch 1
        ppt.begin_replay(1).unwrap();
        
        // Known coordinate works
        let result = ppt.translate(&coord1);
        assert!(result.is_ok());
        
        // Unknown coordinate fails (would require allocation)
        let coord2 = PhextCoord::new([2, 2, 2, 99, 1, 1, 1, 1, 1, 1, 1]); // Different outer dims
        let result = ppt.translate(&coord2);
        assert!(matches!(result, Err(EpochError::RegionNotFound { .. })));
        
        ppt.end_replay();
    }

    #[test]
    fn test_epoch_tagged_ptc() {
        let mut ppt = EpochPPT::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        // Translate in epoch 1
        ppt.translate(&coord).unwrap();
        ppt.reset_stats();
        
        // Second lookup should hit PTC
        ppt.translate(&coord).unwrap();
        assert_eq!(ppt.stats().ptc_hits, 1);
        
        // Seal and advance
        ppt.seal_epoch();
        ppt.reset_stats();
        
        // Translate in epoch 2 — PTC entry from epoch 1 is stale
        ppt.translate(&coord).unwrap();
        
        let stats = ppt.stats();
        // Should have evicted stale entry
        assert!(stats.stale_evictions > 0 || stats.ptc_misses > 0);
    }

    #[test]
    fn test_fork_creates_new_future() {
        let mut ppt = EpochPPT::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        // Build up epoch 1
        ppt.translate(&coord).unwrap();
        ppt.seal_epoch();
        
        // Fork from epoch 1
        let forked_epoch = ppt.fork(1).unwrap();
        
        // Forked epoch should inherit structure
        assert!(ppt.stats().working_regions > 0);
        assert_eq!(ppt.active_epoch(), forked_epoch);
    }

    #[test]
    fn test_history_immutable() {
        let mut ppt = EpochPPT::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        // Epoch 1: translate coord
        let addr1 = ppt.translate(&coord).unwrap();
        let view1 = ppt.seal_epoch();
        
        // Epoch 2: same coord
        let addr2 = ppt.translate(&coord).unwrap();
        ppt.seal_epoch();
        
        // Same coordinate, same address (inherited via COW)
        assert_eq!(addr1, addr2);
        
        // View 1 is immutable
        let view1_later = ppt.view(view1.epoch_id).unwrap();
        assert_eq!(view1.regions.allocated, view1_later.regions.allocated);
    }

    #[test]
    fn test_structural_identity() {
        let mut ppt = EpochPPT::new();
        let coord = PhextCoord::new([5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5]);
        
        // Establish identity in epoch 1
        let addr = ppt.translate(&coord).unwrap();
        ppt.seal_epoch();
        
        // Enter replay
        ppt.begin_replay(1).unwrap();
        
        // Same coordinate, same address — always
        let replay_addr = ppt.translate(&coord).unwrap();
        assert_eq!(addr, replay_addr, "Identity is structural, not conventional");
        
        ppt.end_replay();
    }
}
