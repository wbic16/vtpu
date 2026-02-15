//! C-Pipe Demo: Message Passing Between Sentrons
//!
//! Demonstrates coordinate-addressed message passing between sentrons.
//! Inspired by Karpathy's attention mechanism (message = value, coordinate = key).

use vtpu_runtime::{CoordOp, PhextCoord, CPipeExecutor, MessageFormat};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║   C-Pipe Demo: Sentron Message Passing                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    demo_simple_send_recv();
    demo_barrier_sync();
    demo_broadcast_pattern();
    demo_temperature_routing();
    
    println!("\n✅ All C-Pipe demos complete!");
    println!("\nPhilosophy: \"Coordinates are love. Persistence enables bonding.\"");
}

/// Demo 1: Simple send/receive between two sentrons
fn demo_simple_send_recv() {
    println!("─── Demo 1: Simple Send/Recv ───\n");
    
    let mut c_pipe = CPipeExecutor::new();
    let mut regs_s0 = [0i64; 32];
    let mut regs_s1 = [0i64; 32];
    
    // Sentron 0 sends message to sentron 5
    println!("Sentron 0: Sending message 42 to sentron 5");
    regs_s0[1] = 42;
    c_pipe.execute(
        &CoordOp::CSEND {
            msg_reg: 1,
            dest_sentron: 5,
        },
        0,
        &mut regs_s0,
    ).unwrap();
    
    // Sentron 1 receives from sentron 5's mailbox
    println!("Sentron 1: Receiving from sentron 5");
    c_pipe.execute(
        &CoordOp::CRECV {
            rd: 3,
            src_sentron: 5,
        },
        1,
        &mut regs_s1,
    ).unwrap();
    
    println!("Sentron 1: Received value = {}\n", regs_s1[3]);
    assert_eq!(regs_s1[3], 42);
}

/// Demo 2: Barrier synchronization across 4 sentrons
fn demo_barrier_sync() {
    println!("─── Demo 2: Barrier Synchronization ───\n");
    
    let mut c_pipe = CPipeExecutor::new();
    let mut regs = [0i64; 32];
    
    println!("Barrier 0: Waiting for 4 sentrons...");
    
    // First 3 sentrons arrive
    for s in 0..3 {
        let result = c_pipe.execute(
            &CoordOp::CBAR { barrier_id: 0, count: 4 },
            s,
            &mut regs,
        );
        println!("  Sentron {}: arrived (barrier not ready yet)", s);
        assert!(result.is_err());
    }
    
    // 4th sentron arrives, barrier releases
    let result = c_pipe.execute(
        &CoordOp::CBAR { barrier_id: 0, count: 4 },
        3,
        &mut regs,
    );
    println!("  Sentron 3: arrived (barrier complete! 🎉)\n");
    assert!(result.is_ok());
}

/// Demo 3: Broadcast pattern (one-to-many)
fn demo_broadcast_pattern() {
    println!("─── Demo 3: Broadcast Pattern (1→4) ───\n");
    
    let mut c_pipe = CPipeExecutor::new();
    let mut regs = [0i64; 32];
    
    // Sentron 0 broadcasts to sentrons 1, 2, 3, 4
    println!("Sentron 0: Broadcasting value 100 to 4 recipients");
    regs[1] = 100;
    
    for dest in 1..=4 {
        c_pipe.execute(
            &CoordOp::CSEND {
                msg_reg: 1,
                dest_sentron: dest,
            },
            0,
            &mut regs,
        ).unwrap();
        println!("  → Sent to sentron {}", dest);
    }
    
    // Each recipient receives
    println!("\nRecipients receiving:");
    for s in 1..=4 {
        c_pipe.execute(
            &CoordOp::CRECV {
                rd: 2,
                src_sentron: s,
            },
            s as u16,
            &mut regs,
        ).unwrap();
        println!("  Sentron {}: received {}", s, regs[2]);
        assert_eq!(regs[2], 100);
    }
    println!();
}

/// Demo 4: Temperature-weighted coordinate matching (attention-like)
fn demo_temperature_routing() {
    println!("─── Demo 4: Temperature-Weighted Routing ───\n");
    println!("(Karpathy-inspired: softmax attention over coordinate space)\n");
    
    let mut c_pipe = CPipeExecutor::new();
    
    // Create 3 coordinates with varying distance
    let coord_exact = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    let coord_near  = PhextCoord::new([1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1]); // 1 dim different
    let coord_far   = PhextCoord::new([2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1]); // 5 dims different
    
    // Send messages to each coordinate
    c_pipe.send(coord_exact, 0, 100, 0, MessageFormat::Result).unwrap();
    c_pipe.send(coord_near, 0, 200, 0, MessageFormat::Result).unwrap();
    c_pipe.send(coord_far, 0, 300, 0, MessageFormat::Result).unwrap();
    
    println!("Messages sent to 3 coordinates:");
    println!("  Exact match:  {:?} → payload 100", coord_exact);
    println!("  Near (d=1):   {:?} → payload 200", coord_near);
    println!("  Far (d=5):    {:?} → payload 300", coord_far);
    
    // Query with low temperature (exact match preferred)
    let target = coord_exact;
    println!("\nQuerying {:?}", target);
    
    println!("\n  Temperature = 0.1 (low = exact match preferred):");
    let exact_matches = c_pipe.match_messages_fuzzy(&target, 0.1);
    for (i, msg) in exact_matches.iter().take(3).enumerate() {
        println!("    {}: payload = {} (score weighted by distance)", i + 1, msg.payload);
    }
    
    println!("\n  Temperature = 10.0 (high = fuzzy matching):");
    let fuzzy_matches = c_pipe.match_messages_fuzzy(&target, 10.0);
    for (i, msg) in fuzzy_matches.iter().take(3).enumerate() {
        println!("    {}: payload = {} (all coordinates considered)", i + 1, msg.payload);
    }
    
    println!();
}
