//! R23W16 - SMT Using Existing Infrastructure
//!
//! Uses SmtPair from smt.rs to demonstrate dual-thread speedup.
//! Measures effective ops/cycle accounting for SMT overlap.
//!
//! Usage: cargo run --release --bin w16_smt_using_existing

use vtpu_runtime::*;
use std::time::Instant;

const BASELINE_OPS_PER_SEC: f64 = 137_000_000.0; // From w16_smt_baseline

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║   R23W16: SMT Using SmtPair (Existing Infrastructure)        ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Baseline: {:.0}M ops/sec (single-thread)", BASELINE_OPS_PER_SEC / 1e6);
    println!("Target: 1.8× SMT speedup");
    println!();

    bench_sequential_vs_smt();
}

fn bench_sequential_vs_smt() {
    let mut mem = Memory::new();
    
    // Build forward program (D-Pipe heavy: 90% compute, 10% memory)
    let mut forward_prog = Vec::new();
    for i in 0..15_000 {
        // Compute ops
        forward_prog.push(SIW::new(
            DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
        
        forward_prog.push(SIW::new(
            DenseOp::DADD { rd: 3, rs1: 2, rs2: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
        
        forward_prog.push(SIW::new(
            DenseOp::DFMA { rd: 4, rs1: 2, rs2: 3, rs3: 0 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
        
        // Occasional memory (10%)
        if i % 10 == 0 {
            let coord = PhextCoord::new([(i % 100) as u16 + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            forward_prog.push(SIW::new(
                DenseOp::DNOP,
                SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
                CoordOp::CNOP,
                coord,
            ));
        }
    }
    
    // Build backward program (S-Pipe heavy: 90% memory, 10% compute)
    // Make it LONGER to stress SMT overlap
    let mut backward_prog = Vec::new();
    
    // Pre-populate coordinates
    for i in 1..=1000 {
        let coord = PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        mem.scatter_i64(&coord, i as i64);
    }
    
    for i in 0..25_000 {  // More iterations to match forward load
        let coord = PhextCoord::new([(i % 1000) as u16 + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        // Memory ops
        backward_prog.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 },
            CoordOp::CNOP,
            coord,
        ));
        
        backward_prog.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SSCATTR { coord_idx: 0, rs: 0, width: 8 },
            CoordOp::CNOP,
            coord,
        ));
        
        // Occasional compute (10%)
        if i % 10 == 0 {
            backward_prog.push(SIW::new(
                DenseOp::DADD { rd: 1, rs1: 0, rs2: 0 },
                SparseOp::SNOP,
                CoordOp::CNOP,
                PhextCoord::zero(),
            ));
        }
    }
    
    println!("─── Sequential Execution (Baseline) ───");
    println!();
    
    // Sequential: run forward, then backward
    let mut sentron_fwd = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron_fwd.spawn(forward_prog.clone());
    
    let start_seq = Instant::now();
    let stats_fwd = exec::run(&mut sentron_fwd, &mut mem);
    
    let mut sentron_bwd = Sentron::new(1, PhextCoord::zero(), 0, 0);
    sentron_bwd.spawn(backward_prog.clone());
    let stats_bwd = exec::run(&mut sentron_bwd, &mut mem);
    
    let elapsed_seq = start_seq.elapsed();
    
    let seq_total_ops = stats_fwd.ops_retired + stats_bwd.ops_retired;
    let seq_total_cycles = stats_fwd.cycles + stats_bwd.cycles; // Sequential = sum
    let seq_throughput = seq_total_ops as f64 / elapsed_seq.as_secs_f64();
    
    println!("  Forward:  {} ops, {} cycles", stats_fwd.ops_retired, stats_fwd.cycles);
    println!("  Backward: {} ops, {} cycles", stats_bwd.ops_retired, stats_bwd.cycles);
    println!("  Total ops:     {}", seq_total_ops);
    println!("  Total cycles:  {} (sequential sum)", seq_total_cycles);
    println!("  Wall time:     {:.2} µs", elapsed_seq.as_micros());
    println!("  Throughput:    {:.2}M ops/sec", seq_throughput / 1e6);
    println!();
    
    println!("─── SMT Execution (SmtPair - Overlapped) ───");
    println!();
    
    // SMT: forward + backward overlap on same core
    let mut pair = SmtPair::new(0, 0, PhextCoord::zero());
    
    let start_smt = Instant::now();
    let train_stats = pair.train_step(&mut mem, forward_prog, backward_prog);
    let elapsed_smt = start_smt.elapsed();
    
    let smt_throughput = train_stats.total_ops as f64 / elapsed_smt.as_secs_f64();
    
    println!("  Forward:  {} ops, {} cycles", 
             train_stats.forward_stats.ops_retired, train_stats.forward_stats.cycles);
    println!("  Backward: {} ops, {} cycles",
             train_stats.backward_stats.ops_retired, train_stats.backward_stats.cycles);
    println!("  Total ops:     {}", train_stats.total_ops);
    println!("  Total cycles:  {} (max, not sum — SMT overlap)", train_stats.total_cycles);
    println!("  Wall time:     {:.2} µs", elapsed_smt.as_micros());
    println!("  Throughput:    {:.2}M ops/sec", smt_throughput / 1e6);
    println!("  Effective ops/cycle: {:.2}", train_stats.effective_ops_per_cycle);
    println!();
    
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                         SMT Speedup                           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    let cycle_speedup = seq_total_cycles as f64 / train_stats.total_cycles as f64;
    let wall_speedup = elapsed_seq.as_secs_f64() / elapsed_smt.as_secs_f64();
    
    println!("  Sequential total cycles:  {}", seq_total_cycles);
    println!("  SMT total cycles:         {}", train_stats.total_cycles);
    println!("  Cycle speedup:            {:.2}×", cycle_speedup);
    println!();
    println!("  Sequential wall time:     {:.2} µs", elapsed_seq.as_micros());
    println!("  SMT wall time:            {:.2} µs", elapsed_smt.as_micros());
    println!("  Wall time speedup:        {:.2}×", wall_speedup);
    println!();
    
    if cycle_speedup >= 1.8 {
        println!("  ✅ SMT TARGET ACHIEVED (≥1.8× cycle speedup)");
    } else if cycle_speedup >= 1.5 {
        println!("  🟡 Good speedup ({:.2}×), below 1.8× target", cycle_speedup);
    } else {
        println!("  ⚠️  Below target ({:.2}× vs 1.8×)", cycle_speedup);
    }
    println!();
    
    println!("Note: SmtPair models SMT by taking max(fwd_cycles, bwd_cycles)");
    println!("instead of sum. This simulates concurrent execution on one core.");
}
