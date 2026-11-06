# RG-2A: append-only full-payload TTSM snapshots (experimental)

**Base:** `exo` at merge `26b1c7982068fec1fc2329f17567ab6dc64ee171`

**Scope:** `src/ttsm.rs`, private `src/ttsm_log.rs`, and `tests/reality_gate_ttsm_durable.rs`.

**Status:** First executable persistence slice, **not** the complete RG-2 crash/durability gate.

## API and storage contract

- `TTSM::new()` remains an in-memory implementation; unlike the old implementation it retains immutable committed *payload bytes* for `committed_payload()` and `replay_payloads()`.
- `TTSM::open(path)` creates or replays a versioned append-only file. Every `commit` stores a **full snapshot** and returns successfully only after `File::sync_data()` completes. The file also records `fork` operations, so reopening reconstructs independent heads and fork payloads.
- `replay()` remains the existing metadata-only API to avoid breaking callers. `replay_payloads()` returns block/byte pairs and errors for missing fork or payload; it does not silently drop an unavailable snapshot.
- On Unix, initial file creation synchronizes the file and its parent directory. On other targets `std` cannot guarantee a durable directory entry; this is explicitly outside this first implementation's portable guarantee.
- No SQ daemon, external crates, native hot-swapping, cluster contact, or real user data is required.

## Wire format version 1

| Layer | Encoding |
| --- | --- |
| File header | 8 ASCII bytes `VTTSM001` |
| Record frame | little-endian `u32 body_len`, `u64 FNV-1a checksum(body)`, then `body_len` bytes |
| Commit body, tag `1` | `TemporalBlockId` (u64 sequence, u64 fork, u128 timestamp), parent id, 11 little-endian u16 coordinates, u8 flags, length-prefixed UTF-8 owner/label, u32 payload length and full payload |
| Fork body, tag `2` | source-head block id, newly allocated fork-head block id; replay copies the immutable source metadata/payload and preserves fork lineage |

Record size is capped at **2 MiB**; the existing default state limit is 64 KiB. Checksums detect accidental corruption but are **not cryptographic authentication**. Format changes need a new magic/version and migration path; never silently reinterpret an old record.

There is **no secondary head manifest**: the head of each fork is derived from the last complete validated log records, avoiding a data-versus-pointer update ordering conflict. Append and `sync_data()` are the durable write boundary; speculative state is removed from RAM only after that boundary succeeds. An unsuccessful append poisons the writer; reopen/recovery is required before another write.

## Recovery and conflict behavior

- A truncated last record (partial frame header or body) is discarded; the valid prefix is retained, truncated and synchronized before further writes.
- A partial file magic that is a valid prefix is reinitialized as empty genesis. An incorrect magic/version, bad checksum on a complete record, unknown record type or impossible parent/fork chain fails closed as `TTSMError::CorruptLog`.
- Two speculative owners based on the same old head cannot both commit silently: the second gets `TTSMError::Conflict`; its speculative bytes remain available to rebase, fork or roll back.
- Fork creation is journaled; an owner can begin work on a particular fork through `begin_on_fork`.

## Tests and current evidence boundary

Targeted suite: `cargo test --test reality_gate_ttsm_durable --quiet`. It covers byte-identical reopen, metadata and payload lineage on main/fork, invalid checksums, stale speculative-owner conflicts, state-size guard, interrupted file header, torn append recovery and **every truncation prefix** of a second record (over 100 distinct cut positions). These are deterministic fault injections, **not** SIGKILL/power-loss tests.

**Still blocking full RG-2:**

1. No cross-process writer lock / compare-and-swap transaction. Two processes must not concurrently open the same path for writing; the caller currently supplies single-writer discipline. This is not yet a general concurrent-owner storage protocol.
2. No real random-kill soak or block-device fault injection; successful `sync_data` is assumed to have the OS/filesystem's documented durability semantics.
3. The underlying `PhextCoord` dim10/flags overlap and PPT truncation from the reviewed base must be repaired by the separate coordinate-fidelity PR before asserting **full 11D coordinate replay fidelity**. The wire format stores all 11 dimension fields independently, but reconstruction on this base can still be lossy for affected coordinates.
4. No snapshot compaction, storage quota policy, streaming oversized payloads, encryption or authenticated checksums. Full snapshots can grow linearly with every commit; memory currently caches them too.
5. File-header and frame corruption tests are illustrative; replay of arbitrary malicious log files has not been formally verified. Keep the importer scoped to trusted local artifacts until a security review.

**Merge guidance:** This is a reviewable first durability slice, not evidence that RG-2's full 100+ crash/recovery and concurrent-writer exit gates pass. Keep the PR draft until the single-writer boundary and coordinate dependency are explicitly accepted.
