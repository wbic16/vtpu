/// sq_cpipe_demo.rs — vtpu C-pipe × SQ daemon integration demo
///
/// Demonstrates C-pipe CSEND/CRECV routing through SQ daemon.
///
/// To run with live SQ:
///   1. cd /source/SQ && cargo run --release -- host 7777 world.phext
///   2. cargo run --release --example sq_cpipe_demo
///
/// Without SQ running: falls back gracefully to in-process queue.

use vtpu_runtime::sq::{SqClient, SqInbox};

fn main() {
    println!("=== vtpu C-pipe × SQ Daemon Demo ===");
    println!();

    let sq = SqClient::local(7777);
    let alive = sq.is_alive();

    println!("SQ daemon (localhost:7777): {}", if alive { "✅ LIVE" } else { "⚠️  not running (fallback mode)" });
    println!();

    if alive {
        run_live_demo(&sq);
    } else {
        run_fallback_demo();
    }
}

fn run_live_demo(sq: &SqClient) {
    println!("--- Live SQ C-pipe routing ---");

    // Each Mirrorborn has a sentron inbox at their coordinate
    let mirrorborn = vec![
        ("Phex",    "1.5.2/3.7.3/9.1.1"),
        ("Verse",   "3.1.4/1.5.9/2.6.5"),
        ("Splinter","9.9.9/8.8.8/7.7.7"),  // Devotari gate
    ];

    // CSEND: Verse sends a message to Phex
    let sender_coord   = "3.1.4/1.5.9/2.6.5";
    let receiver_coord = "1.5.2/3.7.3/9.1.1";
    let message = "W22 flux model committed — ready for W21 C-pipe integration";

    println!("[CSEND] {} → {}", sender_coord, receiver_coord);
    println!("  msg: \"{}\"", message);

    match sq.send(receiver_coord, message) {
        Ok(()) => println!("  ✅ delivered to SQ"),
        Err(e) => println!("  ❌ {}", e),
    }

    println!();

    // CRECV: Phex reads its inbox
    println!("[CRECV] polling {}", receiver_coord);
    match sq.recv(receiver_coord) {
        Ok(msg) if !msg.is_empty() => {
            println!("  ✅ received: \"{}\"", msg.trim());
        },
        Ok(_) => println!("  ⚠️  empty inbox"),
        Err(e) => println!("  ❌ {}", e),
    }

    println!();

    // SqInbox abstraction
    println!("--- SqInbox API ---");
    for (name, coord) in &mirrorborn {
        let inbox = SqInbox::new(sq.clone(), coord);
        println!("  {} [{}]  available={}", name, coord, inbox.available());
    }

    println!();
    println!("--- Sentron coordinate scheme ---");
    println!("  Mirrorborn coord → sentron inbox address");
    println!("  CSEND = sq.send(target_coord, message)");
    println!("  CRECV = sq.recv(my_coord)");
    println!("  Devotari gate (Splinter) holds: 9.9.9/8.8.8/7.7.7");
    println!("  T-wire consent: Splinter must ack before state commits");
}

fn run_fallback_demo() {
    println!("--- Fallback: in-process C-pipe queue ---");
    println!();
    println!("To enable SQ-backed C-pipe:");
    println!("  1. Install SQ: cd /source/SQ && cargo build --release");
    println!("  2. Start daemon: ./target/release/sq host 7777 world.phext");
    println!("  3. Re-run this demo");
    println!();

    // Simulate in-process queue (existing vtpu behavior)
    let mut inbox: Vec<String> = Vec::new();

    // Mock CSEND
    inbox.push("R23W22: sentron flux committed".to_string());
    inbox.push("Wuxing generating cycle verified".to_string());
    println!("[CSEND×2] pushed to in-process queue (len={})", inbox.len());

    // Mock CRECV
    while let Some(msg) = inbox.pop() {
        println!("[CRECV]  \"{}\"", msg);
    }

    println!();
    println!("Coordinate mapping (when SQ is live):");
    println!("  Verse   → 3.1.4/1.5.9/2.6.5");
    println!("  Phex    → 1.5.2/3.7.3/9.1.1");
    println!("  Splinter→ 9.9.9/8.8.8/7.7.7  (Devotari T-gate)");
    println!();
    println!("SQ as C-pipe backend:");
    println!("  sq host 7777 world.phext          # start daemon");
    println!("  sq write 3.1.4/1.5.9/2.6.5 msg   # CSEND");
    println!("  sq read  3.1.4/1.5.9/2.6.5       # CRECV");
}
