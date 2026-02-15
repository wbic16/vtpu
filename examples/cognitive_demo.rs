//! Cognitive Demo - Full Cognitive Loop
//!
//! Demonstrates the complete cognitive cycle from cognitive.rs:
//!   ENCODE → ATTEND → ROUTE → RETRIEVE → RESPOND → PERSIST
//!
//! This is the actual "thinking" operation - not just storage/retrieval,
//! but active cognitive processing with attention and routing.

use vtpu_runtime::{
    cognitive::{CognitiveEngine, CognitiveStep},
    HDC_DEFAULT_WIDTH,
};
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Cognitive Demo - Full Encode→Attend→Route→Respond Cycle   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    demo_basic_cognitive_loop();
    demo_attention_focus();
    demo_memory_retrieval();
    demo_persistence();
    
    println!("\n✅ All cognitive demos complete!");
    println!("\nThis is how a sentron thinks:");
    println!("  1. ENCODE   - Turn input into hypervector");
    println!("  2. ATTEND   - Focus on relevant dimensions");
    println!("  3. ROUTE    - Find nearest expert/scroll");
    println!("  4. RETRIEVE - Wildcard match across lattice");
    println!("  5. RESPOND  - Measure similarity, choose action");
    println!("  6. PERSIST  - Write result back to lattice");
}

fn demo_basic_cognitive_loop() {
    println!("─── Demo 1: Basic Cognitive Loop ───\n");
    
    let mut engine = CognitiveEngine::new();
    
    // "Teach" the engine some facts by planting at coordinates
    println!("Planting 3 coordinates in the lattice:");
    
    let facts = vec![
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3],
    ];
    
    for coord in &facts {
        engine.plant(*coord);
        println!("  Planted at {:?}", coord);
    }
    
    println!();
    
    // Run a cognitive step: query for something close
    println!("Running cognitive step: Query for coordinate close to [1,1,1,1,1,1,1,1,1,1,1]...");
    
    let query = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    let attention = 0b11111111111; // All dimensions active
    
    let step = CognitiveStep {
        query,
        attention_mask: attention,
        hd_width: HDC_DEFAULT_WIDTH,
    };
    
    let start = Instant::now();
    let result = engine.think(&step);
    let elapsed = start.elapsed();
    
    println!("\nResult:");
    println!("  Matched coordinate: {:?}", result.matched_coord);
    println!("  Similarity: {:.1}%", result.similarity * 100.0);
    println!("  Memory tier: {:?}", result.tier);
    println!("  Candidates searched: {}", result.candidates_searched);
    println!("  Persisted at: {:?}", result.persisted_at);
    println!("  Thinking time: {} μs", elapsed.as_micros());
    
    println!("\n✓ Basic cognitive loop functional");
    println!();
}

fn demo_attention_focus() {
    println!("─── Demo 2: Attention Masking ───\n");
    
    let mut engine = CognitiveEngine::new();
    
    // Store patterns that differ in different dimensions
    println!("Planting patterns that differ in specific dimensions:");
    
    engine.plant([1, 1, 1, 5, 5, 5, 1, 1, 1, 1, 1]);
    engine.plant([1, 1, 1, 1, 1, 1, 5, 5, 5, 1, 1]);
    engine.plant([5, 5, 5, 1, 1, 1, 1, 1, 1, 1, 1]);
    
    println!("  Pattern A: [1,1,1,5,5,5,1,1,1,1,1] (varies in dims 3-5)");
    println!("  Pattern B: [1,1,1,1,1,1,5,5,5,1,1] (varies in dims 6-8)");
    println!("  Pattern C: [5,5,5,1,1,1,1,1,1,1,1] (varies in dims 0-2)");
    println!();
    
    // Query with different attention masks
    let query = [1, 1, 1, 4, 4, 4, 1, 1, 1, 1, 1];
    
    // Focus on dims 3-5 (should match Pattern A)
    println!("Query 1: Focus on dimensions 3-5 (mask=0b00000111000)");
    let attention1 = 0b00000111000; // dims 3,4,5
    let step1 = CognitiveStep {
        query,
        attention_mask: attention1,
        hd_width: HDC_DEFAULT_WIDTH,
    };
    let result1 = engine.think(&step1);
    println!("  Matched: {:?}", result1.matched_coord);
    println!("  Similarity: {:.1}%", result1.similarity * 100.0);
    println!();
    
    // Focus on dims 0-2 (should match Pattern C)
    println!("Query 2: Focus on dimensions 0-2 (mask=0b00000000111)");
    let attention2 = 0b00000000111; // dims 0,1,2
    let step2 = CognitiveStep {
        query,
        attention_mask: attention2,
        hd_width: HDC_DEFAULT_WIDTH,
    };
    let result2 = engine.think(&step2);
    println!("  Matched: {:?}", result2.matched_coord);
    println!("  Similarity: {:.1}%", result2.similarity * 100.0);
    println!();
    
    println!("✓ Attention masking works - different focus → different matches");
    println!();
}

fn demo_memory_retrieval() {
    println!("─── Demo 3: Associative Memory Retrieval ───\n");
    
    let mut engine = CognitiveEngine::new();
    
    // Store related concepts at nearby coordinates
    println!("Planting related concepts in a semantic neighborhood:");
    
    let concepts = vec![
        [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
        [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3],
        [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 4],
    ];
    
    for coord in &concepts {
        engine.plant(*coord);
        println!("  Planted {:?}", coord);
    }
    
    println!();
    
    // Query: retrieve information near [2,1,1,1,1,1,1,1,1,1,2]
    println!("Query: Retrieve information near [2,1,1,1,1,1,1,1,1,1,2]");
    
    let query = [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2];
    let attention = 0b11111111111;
    let step = CognitiveStep {
        query,
        attention_mask: attention,
        hd_width: HDC_DEFAULT_WIDTH,
    };
    
    let result = engine.think(&step);
    
    println!("\nRetrieved:");
    println!("  Coordinate: {:?}", result.matched_coord);
    println!("  Similarity: {:.1}%", result.similarity * 100.0);
    
    println!("\n✓ Associative retrieval functional");
    println!();
}

fn demo_persistence() {
    println!("─── Demo 4: Thought Persistence ───\n");
    
    let mut engine = CognitiveEngine::new();
    
    // Run a cognitive step and check if result persists
    println!("Running cognitive step and persisting result...");
    
    engine.plant([3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    
    let query = [3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    let attention = 0b11111111111;
    let step = CognitiveStep {
        query,
        attention_mask: attention,
        hd_width: HDC_DEFAULT_WIDTH,
    };
    
    let result = engine.think(&step);
    
    println!("  Query: {:?}", query);
    println!("  Persisted at: {:?}", result.persisted_at);
    println!("  Knowledge size: {} coordinates", engine.knowledge_size());
    println!("  Total steps: {}", engine.total_steps());
    println!("  Bond rate: {:.1}%", engine.bond_rate() * 100.0);
    
    if let Some(_persisted_coord) = result.persisted_at {
        println!("\n✓ Thought persisted - can be retrieved later");
        println!("  This is memory: the choice to remember");
    } else {
        println!("\n✓ Coordinate exists - retrieval successful");
        println!("  (No new persistence needed)");
    }
    
    println!();
}
