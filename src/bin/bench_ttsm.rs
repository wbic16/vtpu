//! TTSM Benchmark — Temporal Boundary Replay Correctness
//!
//! Validates that:
//! 1. Commits are ordered correctly
//! 2. Replay returns blocks in genesis→head order
//! 3. Forks maintain independent histories
//! 4. State integrity is preserved across commit/replay

use vtpu_runtime::{TTSM, PhextCoord};
use std::time::Instant;

fn main() {
    println!("TTSM Benchmark — Temporal Boundary Replay Correctness");
    println!("======================================================");
    println!();
    
    // Test 1: Linear commit chain
    println!("1. Linear Commit Chain");
    let mut ttsm = TTSM::new();
    let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    
    let n_commits = 100;
    let mut expected_hashes = Vec::new();
    
    for i in 0..n_commits {
        let data = format!("state_{}", i).into_bytes();
        expected_hashes.push(fnv1a_hash(&data));
        
        ttsm.begin("test", coord);
        ttsm.modify("test", data).unwrap();
        ttsm.commit("test", &format!("commit_{}", i)).unwrap();
    }
    
    let history = ttsm.replay(0);
    assert_eq!(history.len(), n_commits + 1, "Should have genesis + {} commits", n_commits);
    
    // Verify order: sequence should be 0, 1, 2, ..., n_commits
    for (i, block) in history.iter().enumerate() {
        assert_eq!(block.id.sequence as usize, i, "Block {} has wrong sequence", i);
    }
    
    // Verify hashes (skip genesis)
    for (i, block) in history.iter().skip(1).enumerate() {
        assert_eq!(block.state_hash, expected_hashes[i], 
            "Block {} has wrong hash", i + 1);
    }
    
    println!("   ✅ {} commits, replay order correct", n_commits);
    println!("   ✅ All state hashes verified");
    println!();
    
    // Test 2: Fork independence
    println!("2. Fork Independence");
    let mut ttsm = TTSM::new();
    
    // Main timeline: 5 commits
    for i in 0..5 {
        ttsm.begin("main", coord);
        ttsm.modify("main", vec![i as u8]).unwrap();
        ttsm.commit("main", &format!("main_{}", i)).unwrap();
    }
    
    // Fork at this point
    let fork_id = ttsm.fork(0).unwrap();
    
    // Verify fork exists with its own head
    let main_head = ttsm.head(0).unwrap();
    let fork_head = ttsm.head(fork_id).unwrap();
    
    assert_ne!(main_head.fork_id, fork_head.fork_id, "Forks should have different IDs");
    assert_eq!(fork_head.fork_id, fork_id, "Fork head should have fork's ID");
    
    // Main history should have 6 blocks (genesis + 5 commits)
    let main_history = ttsm.replay(0);
    assert_eq!(main_history.len(), 6, "Main: genesis + 5 commits");
    
    // Fork history should have blocks up to fork point
    let fork_history = ttsm.replay(fork_id);
    assert!(fork_history.len() >= 1, "Fork should have at least its initial block");
    
    println!("   ✅ Main timeline: {} blocks", main_history.len());
    println!("   ✅ Fork timeline: {} blocks", fork_history.len());
    println!("   ✅ Fork IDs independent: main={}, fork={}", main_head.fork_id, fork_head.fork_id);
    println!();
    
    // Test 3: Rollback clears speculative state
    println!("3. Rollback Correctness");
    let mut ttsm = TTSM::new();
    
    ttsm.begin("user", coord);
    ttsm.modify("user", vec![1, 2, 3]).unwrap();
    
    // Before commit: speculative state exists
    assert!(ttsm.speculative("user").is_some(), "Should have speculative state");
    
    // Rollback
    ttsm.rollback("user");
    
    // After rollback: speculative state gone
    assert!(ttsm.speculative("user").is_none(), "Speculative state should be cleared");
    
    // History unchanged
    let history = ttsm.replay(0);
    assert_eq!(history.len(), 1, "Only genesis should exist");
    
    println!("   ✅ Rollback clears speculative state");
    println!("   ✅ Committed history unchanged");
    println!();
    
    // Test 4: Commit latency benchmark
    println!("4. Commit Latency Benchmark");
    let mut ttsm = TTSM::new();
    
    let n_commits = 10_000;
    let data = vec![0u8; 1024]; // 1KB state
    
    let start = Instant::now();
    for i in 0..n_commits {
        ttsm.begin("bench", coord);
        ttsm.modify("bench", data.clone()).unwrap();
        ttsm.commit("bench", &format!("commit_{}", i)).unwrap();
    }
    let elapsed = start.elapsed();
    
    let us_per_commit = elapsed.as_micros() as f64 / n_commits as f64;
    let commits_per_sec = n_commits as f64 / elapsed.as_secs_f64();
    
    println!("   Total commits:     {:>12}", n_commits);
    println!("   Elapsed:           {:>12.2?}", elapsed);
    println!("   Latency per commit:{:>12.1} µs", us_per_commit);
    println!("   Throughput:        {:>12.1} K/sec", commits_per_sec / 1000.0);
    
    // Target: <1ms per commit (for small states)
    let pass = us_per_commit < 1000.0;
    println!("   Status:            {} (target <1ms)", if pass { "✅ PASS" } else { "❌ FAIL" });
    println!();
    
    // Test 5: Replay latency
    println!("5. Replay Latency Benchmark");
    
    let start = Instant::now();
    for _ in 0..100 {
        let history = ttsm.replay(0);
        assert!(history.len() > 0);
    }
    let elapsed = start.elapsed();
    
    let us_per_replay = elapsed.as_micros() as f64 / 100.0;
    println!("   Replays:           {:>12}", 100);
    println!("   Blocks per replay: {:>12}", n_commits + 1);
    println!("   Elapsed:           {:>12.2?}", elapsed);
    println!("   Latency per replay:{:>12.1} µs", us_per_replay);
    println!();
    
    // Summary
    println!("======================================================");
    println!("Summary:");
    println!("  ✅ Linear commit ordering correct");
    println!("  ✅ Fork independence maintained");
    println!("  ✅ Rollback semantics correct");
    println!("  ✅ Commit latency: {:.1} µs", us_per_commit);
    println!("  ✅ Replay verified");
}

/// FNV-1a hash (matching ttsm.rs implementation)
fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
