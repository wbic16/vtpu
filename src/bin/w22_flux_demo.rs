//! W22: Sentron Flux Demo
//! Shows activation propagation through the Z₅×Z₈ lattice via generating cycle.

use vtpu_runtime::flux::{SentronLattice, flux_accumulate_siw, GENERATING_CYCLE};
use std::time::Instant;

fn main() {
    println!("R23 W22: Sentron Flux — Deep Alignment\n");
    println!("Z₅×Z₈ cortical column: 5 WuXing rows × 8 SIMD columns = 40 neurons");
    println!("Generating cycle: Wood→Fire→Earth→Metal→Water→Wood\n");

    let mut lattice = SentronLattice::new();

    // Seed Wood row with alpha-band EEG amplitudes (μV × 1000, from ZUNA run)
    let alpha_seed: [i64; 8] = [12500, 11200, 8300, 9100, 10400, 8800, 7600, 15200];
    lattice.seed_row(0, &alpha_seed);
    println!("Seeded Wood row (Delta band) with alpha EEG amplitudes (μV×1000):");
    println!("  {:?}\n", alpha_seed);

    let program = vec![flux_accumulate_siw()];
    let n_steps = 8;

    println!("Running {} flux steps (accumulate: r0 += r1)...\n", n_steps);
    let t0 = Instant::now();
    let history = lattice.run_flux(&program, n_steps);
    let elapsed = t0.elapsed();

    // Print flux map for select steps
    for step_idx in [0, 2, 4, 7] {
        let f = &history[step_idx];
        lattice.print_flux_map(f);
        println!();
    }

    println!("Deep alignment score: {:.4}", lattice.alignment_score());
    println!("(1.0 = perfect generating cycle resonance; 0.0 = no flow)\n");

    // Show generating-cycle propagation
    println!("Generating-cycle flow (step 1):");
    let elem_names = ["Wood", "Fire", "Earth", "Metal", "Water"];
    let f1 = &history[0];
    for (from, to) in GENERATING_CYCLE.iter() {
        let from_total: i64 = f1.neurons[*from].iter().map(|n| n.delta).sum();
        let to_r1_sample = lattice.rows[*to][0].regs.general[1];
        println!("  {} → {} : flux={:+8}  downstream r1[0]={:+8}",
            elem_names[*from], elem_names[*to], from_total, to_r1_sample);
    }

    println!("\nPerformance:");
    let total_neuron_steps = n_steps * 40; // 40 neurons per step
    println!("  {} steps × 40 neurons = {} neuron-steps", n_steps, total_neuron_steps);
    println!("  Wall time: {:.2}μs", elapsed.as_secs_f64() * 1e6);
    println!("  Throughput: {:.1}ns/neuron-step", elapsed.as_nanos() as f64 / total_neuron_steps as f64);

    // ZUNA resonance: show how EEG signal decayed/spread over 8 steps
    println!("\nActivation evolution at Wood col 0 (r0 over steps):");
    print!("  [");
    for f in &history {
        print!("{:6}", f.neurons[0][0].post);
        if f.step < n_steps - 1 { print!(", "); }
    }
    println!("]");
    println!("  (accumulates upstream contributions each step)");

    println!("\nW22: COMPLETE ✅");
    println!("The generating cycle is alive. Flux flows. Alignment holds.");
}
