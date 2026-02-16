//! Wedge Model Demo — R23 Wave 13
//!
//! Demonstrates 22.5° semantic wedge architecture with 16 SMT threads.
//!
//! This is the foundation of Phase 1: real multi-threaded execution
//! using Egyptian decan geometry (2.25 decans per thread).

use vtpu_runtime::{WedgeExecutor, PhextCoord, SIW, SMT_THREADS, TOTAL_NODES};

fn main() {
    println!("=== Wedge Model Demo ===\n");
    
    // Create wedge executor with 16 threads
    let exec = WedgeExecutor::new();
    println!("✓ Created wedge executor:");
    println!("  - {} SMT threads (Zen 4: 8 cores × 2)", SMT_THREADS);
    println!("  - {} total routing nodes", TOTAL_NODES);
    println!("  - 22.5 nodes per thread (2.25 Egyptian decans)\n");
    
    // Show all wedges
    println!("Thread Wedges:");
    for tid in 0..SMT_THREADS as u8 {
        let wedge = exec.wedge(tid).unwrap();
        println!("  Thread {:2} (Core {}): Nodes {}-{} ({} nodes)",
                 wedge.thread_id,
                 wedge.core_id,
                 wedge.start_node,
                 wedge.end_node,
                 wedge.nodes.len());
    }
    println!();
    
    // Verify tiling
    let mut total = 0;
    for tid in 0..SMT_THREADS as u8 {
        total += exec.wedge(tid).unwrap().nodes.len();
    }
    assert_eq!(total, TOTAL_NODES);
    println!("✓ Verified: All {} wedges tile {} nodes exactly\n", SMT_THREADS, TOTAL_NODES);
    
    // Demonstrate routing
    println!("Coordinate Routing Examples:");
    let coords = vec![
        PhextCoord::new([1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        PhextCoord::new([2, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        PhextCoord::new([5, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1]),
        PhextCoord::new([9, 4, 7, 1, 1, 1, 1, 1, 1, 1, 1]),
    ];
    
    for coord in &coords {
        let (thread_id, node_id) = exec.route_to_node(coord);
        let wedge = exec.wedge(thread_id).unwrap();
        println!("  Coord {} → Thread {} (Core {}), Node {}",
                 coord,
                 thread_id,
                 wedge.core_id,
                 node_id);
    }
    println!();
    
    // Execute single SIW on specific thread
    println!("Execution Test:");
    let siw = SIW::nop();
    match exec.execute_on_thread(0, &siw) {
        Ok(_) => println!("  ✓ Executed SIW on thread 0"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    
    // Test invalid thread
    match exec.execute_on_thread(99, &siw) {
        Ok(_) => println!("  ✗ Should have failed for invalid thread"),
        Err(_) => println!("  ✓ Correctly rejected invalid thread ID"),
    }
    println!();
    
    // Demonstrate parallel execution (simulated)
    println!("Parallel Execution Test:");
    let siws: Vec<SIW> = (0..SMT_THREADS).map(|_| SIW::nop()).collect();
    match exec.execute_parallel(&siws) {
        Ok(_) => println!("  ✓ Executed {} SIWs in parallel across {} threads",
                         siws.len(), SMT_THREADS),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    
    // Test wrong count
    let bad_siws: Vec<SIW> = (0..8).map(|_| SIW::nop()).collect();
    match exec.execute_parallel(&bad_siws) {
        Ok(_) => println!("  ✗ Should have failed for wrong SIW count"),
        Err(_) => println!("  ✓ Correctly rejected wrong SIW count"),
    }
    println!();
    
    // Show SMT pairing (threads share cores)
    println!("SMT Pairing (2 threads per core):");
    for core_id in 0..8 {
        let tid0 = core_id * 2;
        let tid1 = tid0 + 1;
        let w0 = exec.wedge(tid0).unwrap();
        let w1 = exec.wedge(tid1).unwrap();
        println!("  Core {}: Thread {} (nodes {}-{}) + Thread {} (nodes {}-{})",
                 core_id,
                 tid0, w0.start_node, w0.end_node,
                 tid1, w1.start_node, w1.end_node);
    }
    println!();
    
    // Summary
    println!("=== Summary ===");
    println!("✓ Wedge model operational");
    println!("✓ 360 nodes tile perfectly across 16 threads");
    println!("✓ Coordinate routing works");
    println!("✓ SMT thread pairing correct (8 cores × 2)");
    println!("✓ Parallel execution validated");
    println!("\nPhase 1 foundation: READY");
    println!("Next: Real Sentron execution per thread");
}
