//! W25 — Efficient State Streaming with Temporal Jumps
//!
//! Demonstrates:
//!   1. Δ-streaming: send only XOR delta between consecutive states
//!   2. τ-jump: "I have block T-N, send me T-current" — skip unchanged history
//!   3. Compression ratio vs full-state transmission
//!   4. Mesh state sync: simulate 8 nodes converging on shared state
//!
//! Gate: δ-streaming uses ≥50% less bandwidth than full-state transmission.

use vtpu_runtime::state_stream::{StateDelta, StateStreamManager, TemporalJumpRequest, fnv1a_64};
use vtpu_runtime::ttsm::TTSM;
use vtpu_runtime::PhextCoord;
use std::collections::HashMap;
use std::time::Instant;

const N_NODES: usize = 8;
const N_COMMITS: usize = 200;
const STATE_SIZE: usize = 4096;  // 4KB state per commit
const CHANGE_RATE: f64 = 0.05;   // 5% of bytes change per commit
const TARGET_SAVINGS: f64 = 0.50; // 50% bandwidth reduction

fn main() {
    println!("\nvTPU W25 — Efficient State Streaming with Temporal Jumps");
    println!("=========================================================");
    println!("State size : {}B per commit", STATE_SIZE);
    println!("Change rate: {:.0}% per commit", CHANGE_RATE * 100.0);
    println!("Commits    : {}", N_COMMITS);
    println!("Nodes      : {} (simulating mesh sync)", N_NODES);
    println!();

    // ── Phase 1: Generate a state trajectory ─────────────────────────────────
    println!("── Phase 1: State Trajectory Generation ──");
    let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    let mut ttsm = TTSM::new();
    let mut state_history: HashMap<u64, Vec<u8>> = HashMap::new();
    let mut current_state: Vec<u8> = vec![0u8; STATE_SIZE];

    // Seed with initial data
    for i in 0..STATE_SIZE {
        current_state[i] = (i * 7 + 13) as u8;
    }

    ttsm.begin("mesh", coord);
    ttsm.modify("mesh", current_state.clone()).unwrap();
    ttsm.commit("mesh", "genesis").unwrap();
    state_history.insert(0, current_state.clone());

    for commit_idx in 1..=N_COMMITS {
        // Simulate partial state change
        let n_changes = (STATE_SIZE as f64 * CHANGE_RATE) as usize;
        for j in 0..n_changes {
            let pos = (commit_idx * 47 + j * 31) % STATE_SIZE;
            current_state[pos] = (commit_idx * 13 + j) as u8;
        }
        ttsm.begin("mesh", coord);
        ttsm.modify("mesh", current_state.clone()).unwrap();
        ttsm.commit("mesh", &format!("T{}", commit_idx)).unwrap();
        state_history.insert(commit_idx as u64, current_state.clone());
    }

    println!("  Generated {} commits ({} bytes each = {}KB total)",
        N_COMMITS, STATE_SIZE, N_COMMITS * STATE_SIZE / 1024);

    // ── Phase 2: Full-state transmission (baseline) ───────────────────────────
    println!();
    println!("── Phase 2: Full-state Transmission (baseline) ──");
    let full_bytes = N_COMMITS * STATE_SIZE * N_NODES;
    println!("  Bytes to sync {} nodes × {} commits × {}B = {}KB",
        N_NODES, N_COMMITS, STATE_SIZE, full_bytes / 1024);

    // ── Phase 3: Δ-streaming (sequential deltas) ─────────────────────────────
    println!();
    println!("── Phase 3: Δ-Streaming (sequential deltas) ──");
    let t0 = Instant::now();
    let mut stream_mgr = StateStreamManager::new();
    let mut delta_wires: Vec<String> = Vec::new();
    let mut prev_state = state_history[&0].clone();

    for seq in 1..=N_COMMITS as u64 {
        let state = &state_history[&seq];
        let hash = fnv1a_64(state);
        let wire = stream_mgr.stream_commit("mesh", seq, 0, state, hash);
        delta_wires.push(wire);
        prev_state = state.clone();
    }

    let delta_time = t0.elapsed();
    let delta_bytes: usize = delta_wires.iter().map(|w| w.len()).sum();
    let delta_savings = (full_bytes - delta_bytes * N_NODES) as f64 / full_bytes as f64;

    println!("  Delta bytes    : {} ({:.1}KB)", delta_bytes, delta_bytes as f64 / 1024.0);
    println!("  Full equivalent: {} ({:.1}KB)", full_bytes / N_NODES, STATE_SIZE as f64 * N_COMMITS as f64 / 1024.0);
    println!("  Savings/node   : {:.1}%", delta_savings * 100.0);
    println!("  Encode time    : {:.2}ms", delta_time.as_secs_f64() * 1000.0);

    // Verify roundtrip
    let mut reconstructed = state_history[&0].clone();
    let mut correct = 0;
    for (i, wire) in delta_wires.iter().enumerate() {
        if let Some(delta) = StateDelta::from_wire(wire) {
            reconstructed = delta.apply(&reconstructed);
            let expected = &state_history[&(i as u64 + 1)];
            if &reconstructed == expected { correct += 1; }
        }
    }
    println!("  Roundtrip verify: {}/{} commits correct", correct, N_COMMITS);

    // ── Phase 4: τ-jumps (temporal jumps) ─────────────────────────────────────
    println!();
    println!("── Phase 4: τ-Jump Streaming (temporal jumps) ──");

    // Simulate 8 nodes at different sync points
    let node_positions: Vec<u64> = (0..N_NODES as u64)
        .map(|i| i * N_COMMITS as u64 / N_NODES as u64)
        .collect();

    let current_seq = N_COMMITS as u64;
    let mut jump_bytes = 0usize;
    let mut jump_count = 0;

    for (node_idx, &from_seq) in node_positions.iter().enumerate() {
        let req = TemporalJumpRequest {
            owner: "mesh".to_string(),
            from_seq,
            from_fork: 0,
            to_seq: current_seq,
            to_fork: 0,
            requester_node: node_idx as u16,
        };

        if let Some(wire) = stream_mgr.serve_jump(&req, &ttsm, &state_history) {
            let gap = current_seq - from_seq;
            let gap_full_bytes = gap as usize * STATE_SIZE;
            let savings = (gap_full_bytes.saturating_sub(wire.len())) as f64
                / gap_full_bytes.max(1) as f64;
            println!("  Node {}: T{} → T{} (gap {}): {}B wire vs {}B full = {:.1}% saved",
                node_idx, from_seq, current_seq, gap,
                wire.len(), gap_full_bytes, savings * 100.0);
            jump_bytes += wire.len();
            jump_count += 1;
        }
    }

    // ── Phase 5: Mesh convergence simulation ──────────────────────────────────
    println!();
    println!("── Phase 5: Mesh Convergence ──");
    println!("  {} nodes converging to T{} via τ-jumps", N_NODES, current_seq);
    println!("  Total jump bytes: {} ({:.1}KB)", jump_bytes, jump_bytes as f64 / 1024.0);
    let full_sync_bytes = N_NODES * N_COMMITS * STATE_SIZE;
    let mesh_savings = (full_sync_bytes.saturating_sub(jump_bytes)) as f64 / full_sync_bytes as f64;
    println!("  Full sync would need: {} ({:.1}KB)", full_sync_bytes, full_sync_bytes as f64 / 1024.0);
    println!("  Mesh sync savings  : {:.1}%", mesh_savings * 100.0);

    // ── Phase 6: Compression breakdown by change type ─────────────────────────
    println!();
    println!("── Phase 6: Delta Compression Breakdown ──");
    let mut noop_count = 0; let mut xor_count = 0; let mut full_count = 0;
    for wire in &delta_wires {
        if wire.starts_with("NOOP") { noop_count += 1; }
        else if wire.starts_with("DELTA") { xor_count += 1; }
        else { full_count += 1; }
    }
    println!("  NOOP (no change)   : {}", noop_count);
    println!("  Sparse XOR delta   : {}", xor_count);
    println!("  Full state         : {}", full_count);
    println!("  {}", stream_mgr.stats_summary());

    // ── Gate check ───────────────────────────────────────────────────────────
    println!();
    println!("── W25 Gate ──");
    let actual_savings = delta_savings;

    if actual_savings >= TARGET_SAVINGS {
        println!("✅ W25 GATE PASSED: {:.1}% bandwidth savings ≥ {:.0}% target",
            actual_savings * 100.0, TARGET_SAVINGS * 100.0);
        println!("   Δ-streaming + τ-jumps reduce mesh sync bandwidth by {:.1}×",
            1.0 / (1.0 - actual_savings));
    } else {
        println!("⚠️  W25: {:.1}% savings (target {:.0}%)",
            actual_savings * 100.0, TARGET_SAVINGS * 100.0);
        println!("   Increase state size or reduce change rate to see stronger compression.");
    }

    println!();
    println!("Key insight: Temporal jumps collapse N×M transmission to N×1.");
    println!("Each node sends one NEED request; receives one δ(T_base→T_now).");
    println!("Phext coordinates address state blocks by time, not memory location.");
    println!("The lattice knows when things changed. We only send the difference.");
}
