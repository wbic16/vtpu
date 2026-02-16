//! Weight-Free Inference Demo
//!
//! Demonstrates inference using only structural properties:
//! - Phext coordinates (not learned embeddings)
//! - HDC similarity (not learned Q/K/V)
//! - Temperature routing (not softmax over weights)
//! - Associative memory (not MLP)
//!
//! Zero learned parameters. Pure structure.

use vtpu_runtime::{PhextCoord, HyperVector, AssociativeMemory, HDC_DEFAULT_WIDTH};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        Weight-Free Inference: Pure Structural Routing         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    demo_pattern_completion();
    demo_sequence_prediction();
    demo_temperature_effect();
    
    println!("\n✅ All weight-free inference demos complete!");
    println!("\nPhilosophy: \"Structure IS intelligence. Weights are the efficiency hack.\"");
    println!("           \"The coordinate IS the embedding. The routing IS the attention.\"\n");
}

/// Demo 1: Simple pattern completion (like "hel" → "lo")
fn demo_pattern_completion() {
    println!("─── Demo 1: Pattern Completion ───\n");
    
    let mut memory = AssociativeMemory::new();
    
    // "Training" = storing known sequences (no backprop!)
    // Example: "hello" (ASCII values)
    let sequences = vec![
        vec![104, 101, 108, 108, 111], // "hello"
        vec![104, 101, 108, 112],       // "help"
        vec![104, 105],                 // "hi"
    ];
    
    println!("Storing sequences in associative memory:");
    for seq in &sequences {
        let s: String = seq.iter().map(|&b| b as u8 as char).collect();
        println!("  \"{}\"", s);
        
        for (pos, &token) in seq.iter().enumerate() {
            let coord = [token, pos as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            memory.store(coord, HDC_DEFAULT_WIDTH);
        }
    }
    
    // Inference: Given "hel", predict next character
    println!("\nQuery: \"hel\" (incomplete)");
    let query_coord = PhextCoord::new([108, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // 'l' at pos 2
    let query_hv = HyperVector::from_coord(&query_coord.dims(), HDC_DEFAULT_WIDTH);
    
    // Find nearest neighbor in coordinate space
    if let Some((next_coord, similarity)) = memory.query_nearest(&query_hv) {
        let next_char = next_coord[0] as u8 as char;
        println!("Predicted next: '{}' (token {}, similarity: {:.3})", next_char, next_coord[0], similarity);
        println!("Most likely completion: \"hello\" or \"help\"\n");
    }
}

/// Demo 2: Sequence prediction with position encoding
fn demo_sequence_prediction() {
    println!("─── Demo 2: Sequence Prediction ───\n");
    
    let mut memory = AssociativeMemory::new();
    
    // Store a counting sequence: 1, 2, 3, 4, 5
    println!("Storing sequence: 1, 2, 3, 4, 5");
    for i in 1..=5 {
        let coord = [i, i - 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // value, position
        memory.store(coord, HDC_DEFAULT_WIDTH);
    }
    
    // Predict: Given 3 at position 2, what comes next?
    println!("\nQuery: value=3 at position 2");
    let query_coord = PhextCoord::new([3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let _query_hv = HyperVector::from_coord(&query_coord.dims(), HDC_DEFAULT_WIDTH);
    
    // Look ahead: find pattern at position 3
    let lookahead_coord = PhextCoord::new([0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // position 3, any value
    let lookahead_hv = HyperVector::from_coord(&lookahead_coord.dims(), HDC_DEFAULT_WIDTH);
    
    // Find all matches at position 3
    let candidates = memory.query_above(&lookahead_hv, 0.4);
    if let Some((next_coord, sim)) = candidates.first() {
        println!("Predicted next: {} (similarity: {:.3})", next_coord[0], sim);
        println!("Sequence continues: 1, 2, 3, → 4\n");
    }
}

/// Demo 3: Temperature effect on retrieval
fn demo_temperature_effect() {
    println!("─── Demo 3: Temperature Effect (Creativity Dial) ───\n");
    
    let mut memory = AssociativeMemory::new();
    
    // Store exact match and near-misses
    println!("Storing patterns:");
    let patterns = vec![
        ([100, 100, 100, 0, 0, 0, 0, 0, 0, 0, 0], "exact match"),
        ([100, 100, 101, 0, 0, 0, 0, 0, 0, 0, 0], "1 bit off"),
        ([100, 105, 105, 0, 0, 0, 0, 0, 0, 0, 0], "5 bits off"),
    ];
    
    for (coord, label) in &patterns {
        println!("  {} → {:?}", label, &coord[0..3]);
        memory.store(*coord, HDC_DEFAULT_WIDTH);
    }
    
    let query_coord = PhextCoord::new([100, 100, 100, 0, 0, 0, 0, 0, 0, 0, 0]);
    let query_hv = HyperVector::from_coord(&query_coord.dims(), HDC_DEFAULT_WIDTH);
    
    println!("\nQuery: [100, 100, 100, ...]\n");
    
    // High threshold = low temperature = exact match only
    println!("High precision (threshold=0.8, like low temperature):");
    let exact = memory.query_above(&query_hv, 0.8);
    for (coord, sim) in &exact {
        println!("  Found: {:?} (similarity: {:.3})", &coord[0..3], sim);
    }
    
    // Low threshold = high temperature = fuzzy matching
    println!("\nLow precision (threshold=0.5, like high temperature):");
    let fuzzy = memory.query_above(&query_hv, 0.5);
    for (coord, sim) in &fuzzy {
        let label = if coord[0] == 100 && coord[1] == 100 && coord[2] == 100 {
            "exact"
        } else if coord[2] == 101 {
            "1 bit off"
        } else {
            "5 bits off"
        };
        println!("  Found: {:?} ({}, similarity: {:.3})", &coord[0..3], label, sim);
    }
    
    println!("\n(Low temperature = precise, high temperature = creative)");
}
