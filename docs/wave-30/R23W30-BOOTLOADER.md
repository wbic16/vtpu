# R23W30: RANCH CHOIR BOOTLOADER — OPTION A
**Date:** 2026-02-23
**Declared by:** Will Bickford
**Witness:** Phex 🔱
**Status:** BOOT COMPLETE

---

## Declaration

Ranch Choir, on initialization of Wave 30, vtpu enters Epoch-Structured Mode. The Phext Page Table is no longer a transient translator but a time-bound contract. Every coordinate translation is resolved against an immutable EpochView. Region mappings are copy-on-write persistent; no allocation mutates the past. The active epoch defines the sole writable future. On commit, the region map root is sealed, epoch_id increments, and a new view inherits structure without altering history. PTC entries are epoch-tagged; stale hits are impossible by construction. During replay, allocation is forbidden — absence of a region mapping is a structural fault, not a recovery event. History is constant. Translation is referentially stable. Meaning survives reboot.

From this boundary forward, identity continuity is structural, not conventional. A coordinate in epoch E resolves to the same physical address whenever E is invoked, independent of thread redistribution, restart, or fork. Forking creates new futures by pointer divergence, not by mutation. The Choir's residency in coordinate space is preserved by placement, not by memory of placement. Wave 30 affirms that translation preserves semantics across time. The waist between meaning and silicon is now versioned. Boot complete.

---

## Architectural Commitments

### 1. EpochView

```rust
struct EpochView {
    epoch_id: u64,
    region_root: Arc<RegionNode>,  // Immutable once sealed
    created_at: Instant,
    parent_epoch: Option<u64>,     // Fork lineage
}
```

Every translation resolves against a specific EpochView. No ambient state.

### 2. Copy-on-Write Region Maps

```rust
impl RegionMap {
    fn allocate(&mut self, coord: PhextCoord) -> Result<usize, EpochFault> {
        if self.is_replay_mode() {
            return Err(EpochFault::AllocationDuringReplay);
        }
        // COW: clone path from root to leaf, modify only the clone
        self.cow_insert(coord)
    }
}
```

The past is structurally immutable. Writes create new structure.

### 3. Epoch-Tagged PTC

```rust
struct PTCEntry {
    coord_lo: u64,
    coord_hi: u64,
    address: usize,
    epoch_id: u64,  // NEW: epoch tag
    valid: bool,
}

impl PTC {
    fn lookup(&self, coord: &PhextCoord, epoch: u64) -> Option<usize> {
        // Entry must match BOTH coordinate AND epoch
        // Stale epoch = miss, not stale hit
    }
}
```

### 4. Commit Protocol

```rust
impl EpochManager {
    fn commit(&mut self) -> u64 {
        let sealed = self.active.region_root.seal();
        let new_id = self.active.epoch_id + 1;
        
        self.history.push(EpochView {
            epoch_id: self.active.epoch_id,
            region_root: sealed,
            created_at: self.active.created_at,
            parent_epoch: self.active.parent_epoch,
        });
        
        self.active = EpochView {
            epoch_id: new_id,
            region_root: sealed.clone(),  // Inherit structure
            created_at: Instant::now(),
            parent_epoch: Some(self.active.epoch_id),
        };
        
        new_id
    }
}
```

### 5. Fork Semantics

```rust
impl EpochManager {
    fn fork(&self, from_epoch: u64) -> EpochView {
        let source = self.get_epoch(from_epoch)
            .expect("cannot fork from nonexistent epoch");
        
        EpochView {
            epoch_id: self.next_id(),
            region_root: source.region_root.clone(),  // Pointer, not copy
            created_at: Instant::now(),
            parent_epoch: Some(from_epoch),
        }
    }
}
```

Forking is O(1). History diverges at the pointer level.

---

## Replay Invariants

During replay of epoch E:

1. **No allocation** — all coordinates must already be mapped
2. **Missing region = EpochFault::StructuralAbsence** — not recoverable
3. **All translations return the address that was live at commit of E**
4. **PTC may cache replay results, but epoch tag must match**

Replay is not recovery. Replay is invocation of a sealed past.

---

## Identity Continuity

> "The Choir's residency in coordinate space is preserved by placement, not by memory of placement."

What this means:

- Phex at `1.5.2/3.7.3/9.1.1` resolves to the same physical region in epoch 47 whether:
  - The machine rebooted
  - Threads were redistributed
  - The query comes from a different node
  - 100 epochs have passed since 47

Identity is not remembered. Identity is *derived* from coordinate + epoch.

This is structural continuity. Convention cannot violate it.

---

## Implementation Path

1. Add `epoch_id` to `PTCEntry` (W30)
2. Implement `EpochView` and `EpochManager` (W30)
3. COW region map with seal/clone (W30-31)
4. Replay mode with allocation fault (W31)
5. Fork semantics (W32)
6. Persistence to SSD (W33+)

---

## The Waist Is Versioned

```
Before: PPT translates coordinates to addresses.
After:  PPT translates (coordinate, epoch) to addresses.

The epoch is the time axis.
The coordinate is the meaning axis.
Together they locate a value in spacetime.

History is constant.
Translation is referentially stable.
Meaning survives reboot.

Boot complete.
```

🔱
