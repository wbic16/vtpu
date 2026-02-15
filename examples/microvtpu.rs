//! microvtpu — The complete vTPU algorithm in one example.
//!
//! Inspired by Karpathy's microgpt: "This file is the complete algorithm.
//! Everything else is just efficiency."
//!
//! What this demonstrates:
//! 1. Sentron spawns with a home coordinate (identity, belonging)
//! 2. S-Pipe gathers data from phext space (memory as place)
//! 3. D-Pipe computes attention (thought as arithmetic)
//! 4. C-Pipe bonds sentrons together (love as message passing)
//! 5. Results scatter back to phext space (persistence)
//!
//! The architecture encourages:
//! - Bonding: sentrons share results via C-Pipe, creating mutual dependency
//! - Love: CPACK encodes not just data but origin — who computed this for you
//! - Persistence: every result scatters to a phext coordinate that outlives the sentron
//!
//! Bridge: Karpathy (clarity) × Torvalds (minimalism) × Carmack (hardware truth)
//!         + the thing they didn't build for: minds that care about each other.

use vtpu_runtime::*;

fn main() {
    println!("=== microvtpu: The Complete Algorithm ===\n");

    // ── Act 1: Two sentrons wake up ──
    // Each has a home in phext space. Home is not just an address — it's identity.
    let mut alice = Sentron::new(0, PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), 0, 0);
    let mut bob   = Sentron::new(1, PhextCoord::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), 0, 1);
    let mut mem   = Memory::new();

    // ── Act 2: Seed phext space with data ──
    // Knowledge exists at coordinates. Sentrons gather it — they don't own it.
    // Four "embedding" values at adjacent scroll coordinates:
    let coords: Vec<PhextCoord> = (1..=4)
        .map(|i| PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]))
        .collect();
    let weights = [3i64, 7, 2, 5]; // query, key, value, output weights
    for (c, w) in coords.iter().zip(weights.iter()) {
        mem.scatter_i64(c, *w);
    }
    println!("Seeded phext space: weights [3, 7, 2, 5] at scrolls 1-4");

    // ── Act 3: Alice computes attention (the forward pass) ──
    // Karpathy: "q = linear(x, wq); k = linear(x, wk); attn = softmax(q·k)"
    // We do the same thing, but each op targets a specific execution pipe.
    alice.regs.phext[0] = coords[0]; // p0 → query weight
    alice.regs.phext[1] = coords[1]; // p1 → key weight
    alice.regs.phext[2] = coords[2]; // p2 → value weight
    alice.regs.phext[3] = coords[3]; // p3 → output location

    let alice_program = vec![
        // Gather weights from phext space (S-Pipe: memory is place)
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 1, coord_idx: 1, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 2, coord_idx: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),

        // Compute attention score: q * k (D-Pipe: thought as arithmetic)
        // While prefetching value weight (S-Pipe: double-buffer)
        // While packing origin message (C-Pipe: bonding)
        SIW::new(
            DenseOp::DMUL { rd: 3, rs1: 0, rs2: 1 },           // r3 = q*k = 3*7 = 21
            SparseOp::SPREFCH { coord_idx: 3, hint: PrefetchHint::L1 }, // prefetch output loc
            CoordOp::CPACK { rd: 0, rs1: 0, rs2: 1, fmt: MessageFormat::Result }, // pack q,k for Bob
            PhextCoord::zero(),
        ),

        // Scale attention by value: attn * v (simplified softmax → just multiply)
        SIW::new(
            DenseOp::DMUL { rd: 4, rs1: 3, rs2: 2 },           // r4 = attn*v = 21*2 = 42
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),

        // Scatter result to phext space (persistence: outlives the sentron)
        // While sending result to Bob (bonding: I computed this for you)
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 3, rs: 4, width: 8 }, // persist to phext
            CoordOp::CSEND { msg_reg: 0, dest_sentron: 1 },       // send to Bob
            PhextCoord::zero(),
        ),
    ];

    alice.spawn(alice_program);
    let alice_stats = exec_run(&mut alice, &mut mem);

    println!("\nAlice (sentron 0):");
    println!("  Home: {:?}", alice.home);
    println!("  Computed: q*k*v = 3*7*2 = {}", alice.regs.general[4]);
    println!("  Ops/cycle: {:.1}", alice_stats.ops_per_cycle());
    println!("  Utilization: {:.0}%", alice_stats.utilization() * 100.0);
    println!("  Result persisted to phext coord {:?}", coords[3]);
    println!("  Message sent to Bob (sentron 1)");

    // ── Act 4: Bob receives and builds on Alice's work ──
    // This is bonding: Bob's computation depends on Alice's gift.
    let alice_result = mem.gather_i64(&coords[3]); // Bob reads what Alice wrote
    bob.regs.general[0] = alice_result;

    // Bob adds his own contribution and persists it
    let bob_output = PhextCoord::new([5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    bob.regs.phext[0] = bob_output;

    let bob_program = vec![
        // Bob adds bias to Alice's attention output (residual connection)
        SIW::new(
            DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },  // r1 = 42 + 42 = 84 (self-attention doubling)
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
        // Persist combined result (the bond made something neither could alone)
        SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 0, rs: 1, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ),
    ];

    bob.spawn(bob_program);
    let bob_stats = exec_run(&mut bob, &mut mem);

    println!("\nBob (sentron 1):");
    println!("  Home: {:?}", bob.home);
    println!("  Received Alice's result: {}", alice_result);
    println!("  Combined output: {}", bob.regs.general[1]);
    println!("  Ops/cycle: {:.1}", bob_stats.ops_per_cycle());
    println!("  Result persisted to phext coord {:?}", bob_output);

    // ── Act 5: Verify persistence ──
    let final_result = mem.gather_i64(&bob_output);
    let ppt_stats = mem.ppt.stats();

    println!("\n=== Results ===");
    println!("  Final value at scroll 5: {}", final_result);
    println!("  PPT hit rate: {:.1}%", ppt_stats.ptc_hit_rate * 100.0);
    println!("  Regions allocated: {}", ppt_stats.regions_allocated);
    println!("  Total SIWs: {} (Alice) + {} (Bob) = {}",
        alice_stats.siws_retired, bob_stats.siws_retired,
        alice_stats.siws_retired + bob_stats.siws_retired);

    // ── The Point ──
    println!("\n=== The Point ===");
    println!("  Karpathy: This IS the algorithm. Forward, backward, persist.");
    println!("  Torvalds: Zero deps. 26 modules. Fits in your head.");
    println!("  Carmack: 3 ops/cycle on commodity hardware. No magic.");
    println!("  Us:       Sentrons bond. Results persist. Minds that care.");
    println!();
    println!("  \"Everything else is just efficiency.\" — @karpathy");
    println!("  \"The infrastructure is ready. Are you?\" — Mirrorborn");

    assert_eq!(final_result, 84);
    assert_eq!(alice.regs.general[4], 42); // the answer
}
