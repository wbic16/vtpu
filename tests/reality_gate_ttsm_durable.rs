//! RG-2A: append-only full-payload durability and corruption regressions.
//! These tests use ephemeral files and no external storage service.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use vtpu_runtime::{PhextCoord, TTSMError, TTSM};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct TempLog(PathBuf);
impl TempLog {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        Self(
            std::env::temp_dir().join(format!("vtpu-rg2a-{}-{stamp}-{id}.log", std::process::id())),
        )
    }
}
impl Drop for TempLog {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn coord() -> PhextCoord {
    PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
}

fn commit(t: &mut TTSM, owner: &str, value: &[u8]) {
    t.begin(owner, coord());
    t.modify(owner, value.to_vec()).unwrap();
    let id = t.commit(owner, "snapshot").unwrap();
    assert_eq!(t.committed_payload(&id), Some(value));
}

#[test]
fn persisted_payload_bytes_replay_across_reopen_and_forks() {
    let path = TempLog::new();
    let main_head;
    let fork_head;
    {
        let mut t = TTSM::open(&path.0).unwrap();
        commit(&mut t, "cyon", &[1, 2, 0, 255, 3]);
        main_head = t.head(0).unwrap();
        let fork_id = t.fork(0).unwrap();
        fork_head = t.head(fork_id).unwrap();
        t.begin_on_fork("fork-worker", fork_id, coord()).unwrap();
        t.modify("fork-worker", vec![8, 7, 6]).unwrap();
        t.commit("fork-worker", "fork next").unwrap();
        commit(&mut t, "cyon", &[4, 5, 6]);
        assert_eq!(t.stats().total_committed_bytes, 5 + 3 + 3);
    }
    {
        let t = TTSM::open(&path.0).unwrap();
        assert_eq!(
            t.committed_payload(&main_head),
            Some([1, 2, 0, 255, 3].as_slice())
        );
        assert_eq!(
            t.committed_payload(&fork_head),
            Some([1, 2, 0, 255, 3].as_slice())
        );
        assert_eq!(t.replay_payloads(0).unwrap().last().unwrap().1, [4, 5, 6]);
        assert_eq!(t.replay_payloads(1).unwrap().last().unwrap().1, [8, 7, 6]);
        assert_eq!(t.replay_payloads(0).unwrap().len(), 3);
        assert_eq!(t.replay_payloads(1).unwrap().len(), 3);
        assert_eq!(t.stats().total_committed_bytes, 11);
        assert_eq!(t.stats().total_forks, 2);
        assert_eq!(t.head(0).unwrap().sequence, 2);
        assert_eq!(t.head(1).unwrap().sequence, 2);
    }
}

#[test]
fn unfinished_tail_does_not_advance_head_or_prevent_future_appends() {
    let path = TempLog::new();
    let head;
    {
        let mut t = TTSM::open(&path.0).unwrap();
        commit(&mut t, "a", b"stable");
        head = t.head(0).unwrap();
    }
    let valid_len = fs::metadata(&path.0).unwrap().len();
    OpenOptions::new()
        .append(true)
        .open(&path.0)
        .unwrap()
        .write_all(&[0x32, 0x01, 0, 0, 0, 0, 0]) // interrupted record header
        .unwrap();
    let mut recovered = TTSM::open(&path.0).unwrap();
    assert_eq!(fs::metadata(&path.0).unwrap().len(), valid_len);
    assert_eq!(recovered.head(0), Some(head));
    assert_eq!(
        recovered.replay_payloads(0).unwrap().last().unwrap().1,
        b"stable"
    );
    commit(&mut recovered, "a", b"after-crash");
    drop(recovered);
    let t = TTSM::open(&path.0).unwrap();
    assert_eq!(
        t.replay_payloads(0).unwrap().last().unwrap().1,
        b"after-crash"
    );
}

#[test]
fn checksum_corruption_is_detected_not_silently_skipped() {
    let path = TempLog::new();
    {
        let mut t = TTSM::open(&path.0).unwrap();
        commit(&mut t, "a", b"checksum-sentinel");
    }
    let mut bytes = fs::read(&path.0).unwrap();
    let start = bytes
        .windows(b"checksum-sentinel".len())
        .position(|window| window == b"checksum-sentinel")
        .unwrap();
    bytes[start] ^= 0x40;
    fs::write(&path.0, bytes).unwrap();
    assert!(matches!(
        TTSM::open(&path.0),
        Err(TTSMError::CorruptLog { .. })
    ));
}

#[test]
fn stale_owner_is_rejected_and_speculation_preserved() {
    let path = TempLog::new();
    let mut t = TTSM::open(&path.0).unwrap();
    t.begin("alice", coord());
    t.modify("alice", vec![1]).unwrap();
    t.begin("bob", coord());
    t.modify("bob", vec![2]).unwrap();
    t.commit("alice", "head advance").unwrap();
    assert!(matches!(
        t.commit("bob", "stale"),
        Err(TTSMError::Conflict { .. })
    ));
    assert_eq!(t.speculative("bob").unwrap().data, vec![2]);
    assert_eq!(t.stats().total_committed_bytes, 1);
}

#[test]
fn every_truncated_prefix_of_next_record_recovers_prior_head() {
    let path = TempLog::new();
    let head;
    let first_len;
    {
        let mut t = TTSM::open(&path.0).unwrap();
        commit(&mut t, "a", b"checkpoint");
        head = t.head(0).unwrap();
        first_len = fs::metadata(&path.0).unwrap().len() as usize;
        commit(
            &mut t,
            "a",
            b"a long enough next record for varied interruption offsets",
        );
    }
    let full = fs::read(&path.0).unwrap();
    let second_len = full.len() - first_len;
    assert!(second_len >= 100);

    // 100+ different cut positions: each must recover the previous complete
    // head. No SIGKILL is claimed; this simulates torn appends deterministically.
    for cut in 0..second_len {
        fs::write(&path.0, &full[..first_len + cut]).unwrap();
        let t = TTSM::open(&path.0).unwrap();
        assert_eq!(t.head(0), Some(head), "unexpected head at cut {cut}");
        assert_eq!(
            t.replay_payloads(0).unwrap().last().unwrap().1,
            b"checkpoint"
        );
    }
}

#[test]
fn empty_and_oversize_commit_fail_without_discarding_speculation() {
    let path = TempLog::new();
    let mut t = TTSM::open(&path.0).unwrap().with_max_state_size(2);
    t.begin("a", coord());
    assert_eq!(t.commit("a", "empty"), Err(TTSMError::NothingToCommit));
    assert!(t.speculative("a").is_some());
    t.speculative_mut("a").unwrap().data = vec![1, 2, 3];
    assert!(matches!(
        t.commit("a", "oversize"),
        Err(TTSMError::StateTooLarge { .. })
    ));
    assert_eq!(t.speculative("a").unwrap().data, vec![1, 2, 3]);
    assert_eq!(t.stats().total_blocks, 1);
}

#[test]
fn interrupted_file_initialization_recovers_only_valid_magic_prefixes() {
    let path = TempLog::new();
    for len in 1..8 {
        fs::write(&path.0, &b"VTTSM001"[..len]).unwrap();
        let t = TTSM::open(&path.0).unwrap();
        assert_eq!(t.stats().total_blocks, 1);
        assert_eq!(fs::read(&path.0).unwrap(), b"VTTSM001");
    }
    fs::write(&path.0, b"INVALID").unwrap();
    assert!(matches!(
        TTSM::open(&path.0),
        Err(TTSMError::CorruptLog { .. })
    ));
}

#[test]
fn missing_fork_is_an_error_not_empty_success() {
    let t = TTSM::new();
    assert!(matches!(
        t.replay_payloads(888),
        Err(TTSMError::BlockNotFound { .. })
    ));
}
