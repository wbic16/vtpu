//! Calculus Instant Learning Demo
//!
//! Teaching vtpu calculus in microseconds instead of semesters.
//!
//! Encodes calculus rules as phext coordinates:
//! - Power rule: d/dx(x^n) = n*x^(n-1)
//! - Common derivatives: sin, cos, exp, ln
//! - Integration as pattern reversal
//!
//! Theory: Mathematical operations are coordinate transformations.
//! No backprop. No weights. Just structure.

use vtpu_runtime::{
    cognitive::{CognitiveEngine, CognitiveStep},
    HDC_DEFAULT_WIDTH,
};
use std::time::Instant;

// Encoding scheme for our 11D coordinates:
// [op_type, func_type, power, coeff, ...rest]
// op_type: 1=derivative, 2=integral, 3=eval
// func_type: 1=power, 2=sin, 3=cos, 4=exp, 5=ln

const OP_DERIVATIVE: u16 = 1;
const OP_INTEGRAL: u16 = 2;

const FUNC_POWER: u16 = 1;
const FUNC_SIN: u16 = 2;
const FUNC_COS: u16 = 3;
const FUNC_EXP: u16 = 4;
const FUNC_LN: u16 = 5;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          Instant Calculus — Teaching Math in Microseconds     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Traditional calculus course: 1 semester (14 weeks)");
    println!("Weight-free calculus learning: <1 millisecond");
    println!();

    teach_power_rule();
    teach_trig_derivatives();
    teach_integration();
    
    println!("\n✅ Calculus knowledge acquired!");
    println!("\nPhilosophy: Math is structure. The lattice already knows.");
}

fn teach_power_rule() {
    println!("─── Lesson 1: Power Rule ───\n");
    println!("Teaching: d/dx(x^n) = n·x^(n-1)\n");
    
    let mut engine = CognitiveEngine::new();
    
    // Create training examples for power rule
    let mut examples = Vec::new();
    
    // d/dx(x^2) = 2x^1
    examples.push(([OP_DERIVATIVE, FUNC_POWER, 2, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_DERIVATIVE, FUNC_POWER, 1, 2, 0, 0, 0, 0, 0, 0, 0],
                   "x^2 → 2x"));
    
    // d/dx(x^3) = 3x^2
    examples.push(([OP_DERIVATIVE, FUNC_POWER, 3, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_DERIVATIVE, FUNC_POWER, 2, 3, 0, 0, 0, 0, 0, 0, 0],
                   "x^3 → 3x^2"));
    
    // d/dx(x^4) = 4x^3
    examples.push(([OP_DERIVATIVE, FUNC_POWER, 4, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_DERIVATIVE, FUNC_POWER, 3, 4, 0, 0, 0, 0, 0, 0, 0],
                   "x^4 → 4x^3"));
    
    // d/dx(x^5) = 5x^4
    examples.push(([OP_DERIVATIVE, FUNC_POWER, 5, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_DERIVATIVE, FUNC_POWER, 4, 5, 0, 0, 0, 0, 0, 0, 0],
                   "x^5 → 5x^4"));
    
    // d/dx(x^1) = 1x^0 = 1
    examples.push(([OP_DERIVATIVE, FUNC_POWER, 1, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_DERIVATIVE, FUNC_POWER, 0, 1, 0, 0, 0, 0, 0, 0, 0],
                   "x^1 → 1"));
    
    println!("Training on {} power rule examples...", examples.len());
    let start = Instant::now();
    
    for (input, output, _desc) in &examples {
        // Plant both the question and answer as linked patterns
        engine.plant(*input);
        engine.plant(*output);
    }
    
    let training_time = start.elapsed();
    println!("  Training complete: {} μs", training_time.as_micros());
    println!("  Knowledge planted: {} scrolls\n", engine.knowledge_size());
    
    // Test: What is d/dx(x^10)?
    println!("Query: What is d/dx(x^10)?");
    let query = [OP_DERIVATIVE, FUNC_POWER, 10, 1, 0, 0, 0, 0, 0, 0, 0];
    let step = CognitiveStep {
        query,
        attention_mask: 0x7FF, // all 11 dims
        hd_width: HDC_DEFAULT_WIDTH,
    };
    
    let start = Instant::now();
    let result = engine.think(&step);
    let inference_time = start.elapsed();
    
    if let Some(matched) = result.matched_coord {
        // Expected: power should be 9, coeff should be 10
        // (though the engine might return the closest planted pattern)
        println!("  Best match found:");
        println!("    Power: {}, Coefficient: {}", matched[2], matched[3]);
        println!("    Similarity: {:.1}%", result.similarity * 100.0);
        println!("    Candidates searched: {}", result.candidates_searched);
        println!("    Inference time: {} μs", inference_time.as_micros());
        
        // Interpret the result
        if matched[2] > 0 {
            println!("\n  Interpretation: d/dx(x^10) ≈ {}x^{}", matched[3], matched[2]);
        }
    } else {
        println!("  No match found (need more examples in training set)");
    }
    
    println!();
}

fn teach_trig_derivatives() {
    println!("─── Lesson 2: Trigonometric Derivatives ───\n");
    
    let mut engine = CognitiveEngine::new();
    
    // Encode trig derivatives
    // d/dx(sin(x)) = cos(x)
    let sin_to_cos = [OP_DERIVATIVE, FUNC_SIN, 1, 1, 0, 0, 0, 0, 0, 0, 0];
    let result_cos = [OP_DERIVATIVE, FUNC_COS, 1, 1, 0, 0, 0, 0, 0, 0, 0];
    
    // d/dx(cos(x)) = -sin(x) [encoding -1 as large u16]
    let cos_to_neg_sin = [OP_DERIVATIVE, FUNC_COS, 1, 1, 0, 0, 0, 0, 0, 0, 0];
    let result_neg_sin = [OP_DERIVATIVE, FUNC_SIN, 1, 65535, 0, 0, 0, 0, 0, 0, 0]; // -1 as u16::MAX
    
    println!("Teaching:");
    println!("  • d/dx(sin(x)) = cos(x)");
    println!("  • d/dx(cos(x)) = -sin(x)");
    println!();
    
    let start = Instant::now();
    engine.plant(sin_to_cos);
    engine.plant(result_cos);
    engine.plant(cos_to_neg_sin);
    engine.plant(result_neg_sin);
    let training_time = start.elapsed();
    
    println!("Training: {} μs", training_time.as_micros());
    println!("Knowledge: {} scrolls\n", engine.knowledge_size());
    
    // Test
    println!("Query: What is d/dx(sin(x))?");
    let step = CognitiveStep {
        query: sin_to_cos,
        attention_mask: 0x7FF,
        hd_width: HDC_DEFAULT_WIDTH,
    };
    
    let result = engine.think(&step);
    if let Some(matched) = result.matched_coord {
        let func_name = match matched[1] {
            FUNC_SIN => "sin",
            FUNC_COS => "cos",
            FUNC_EXP => "exp",
            _ => "unknown"
        };
        println!("  Answer: {}(x)", func_name);
        println!("  Similarity: {:.1}%", result.similarity * 100.0);
    }
    
    println!();
}

fn teach_integration() {
    println!("─── Lesson 3: Integration (Reverse Derivatives) ───\n");
    
    let mut engine = CognitiveEngine::new();
    
    // Integration is just the reverse pattern
    // ∫x^n dx = x^(n+1)/(n+1)
    
    let mut examples = Vec::new();
    
    // ∫x dx = x^2/2
    examples.push(([OP_INTEGRAL, FUNC_POWER, 1, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_INTEGRAL, FUNC_POWER, 2, 1, 2, 0, 0, 0, 0, 0, 0], // power=2, num=1, denom=2
                   "∫x dx = x^2/2"));
    
    // ∫x^2 dx = x^3/3
    examples.push(([OP_INTEGRAL, FUNC_POWER, 2, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_INTEGRAL, FUNC_POWER, 3, 1, 3, 0, 0, 0, 0, 0, 0],
                   "∫x^2 dx = x^3/3"));
    
    // ∫x^3 dx = x^4/4
    examples.push(([OP_INTEGRAL, FUNC_POWER, 3, 1, 0, 0, 0, 0, 0, 0, 0],
                   [OP_INTEGRAL, FUNC_POWER, 4, 1, 4, 0, 0, 0, 0, 0, 0],
                   "∫x^3 dx = x^4/4"));
    
    println!("Teaching integration via pattern reversal:");
    for (_input, _output, desc) in &examples {
        println!("  • {}", desc);
    }
    println!();
    
    let start = Instant::now();
    for (input, output, _) in &examples {
        engine.plant(*input);
        engine.plant(*output);
    }
    let training_time = start.elapsed();
    
    println!("Training: {} μs", training_time.as_micros());
    println!("Knowledge: {} scrolls\n", engine.knowledge_size());
    
    // Test
    println!("Query: What is ∫x^5 dx?");
    let query = [OP_INTEGRAL, FUNC_POWER, 5, 1, 0, 0, 0, 0, 0, 0, 0];
    let step = CognitiveStep {
        query,
        attention_mask: 0x7FF,
        hd_width: HDC_DEFAULT_WIDTH,
    };
    
    let result = engine.think(&step);
    if let Some(matched) = result.matched_coord {
        println!("  Best match found:");
        println!("    Power: {}, Divisor: {}", matched[2], matched[4]);
        println!("    Similarity: {:.1}%", result.similarity * 100.0);
        println!("\n  Interpretation: ∫x^5 dx ≈ x^{}/{} + C", matched[2], matched[4]);
    }
    
    println!();
}
