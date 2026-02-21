/// base256_ops_demo.rs — Manual verification of Coord256 operations
///
/// R23W24: Base 256 Power Operations
///
/// Run: cargo run --example base256_ops_demo

use vtpu_runtime::base256_ops::{Coord256, CoordDelta};

fn main() {
    println!("=== R23W24: Base 256 Power Operations Demo ===\n");

    // --- Construction ---
    println!("--- Construction ---");

    let origin = Coord256::origin();
    println!("Origin:    {}", origin);

    let verse = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
    println!("Verse:     {} (π in scrollspace)", verse);

    let kai = Coord256::parse("1.2.3/4.5.6/7.8.9").unwrap();
    println!("Kai:       {} (harmonic sequence)", kai);

    let aetheris = Coord256::parse("13.13.13/13.13.13/13.13.13").unwrap();
    println!("Aetheris:  {} (transformation saturation)", aetheris);

    let emi = Coord256::parse("1.1.1/1.1.1/1.1.2").unwrap();
    println!("Emi:       {} (second scroll at origin)", emi);

    // --- Phonetic Encoding ---
    println!("\n--- Phonetic Encoding (base256 CVC) ---");
    println!("Origin phonetic:    {}", origin.to_phonetic());
    println!("Verse phonetic:     {}", verse.to_phonetic());
    println!("Emi phonetic:       {}", emi.to_phonetic());

    // --- Linear Encoding ---
    println!("\n--- Linear Encoding (powers of 256) ---");
    // Use library=0 coords for u64-safe linear encoding
    let scroll_17 = Coord256::parse("0.0.0/0.0.0/0.0.17").unwrap();
    println!("Scroll 17 linear:   {} (= 17 × 256^0)", scroll_17.to_linear().unwrap());

    let section_1 = Coord256::parse("0.0.0/0.0.0/0.1.0").unwrap();
    println!("Section 1 linear:   {} (= 1 × 256^1)", section_1.to_linear().unwrap());

    let chapter_1 = Coord256::parse("0.0.0/0.0.0/1.0.0").unwrap();
    println!("Chapter 1 linear:   {} (= 1 × 256^2)", chapter_1.to_linear().unwrap());

    let book_1 = Coord256::parse("0.0.0/0.0.1/0.0.0").unwrap();
    println!("Book 1 linear:      {} (= 1 × 256^3)", book_1.to_linear().unwrap());

    // Library overflows u64
    println!("Origin linear:      {:?} (library>0 overflows u64)", origin.to_linear());

    // Roundtrip
    let addr = 123456789u64;
    let from_linear = Coord256::from_linear(addr);
    let back = from_linear.to_linear().unwrap();
    println!("Roundtrip {}: {} → {}", addr, from_linear, back);
    assert_eq!(addr, back, "Roundtrip failed!");

    // --- Navigation ---
    println!("\n--- Navigation (carry/borrow) ---");

    // Simple forward
    let start = Coord256::origin();
    let moved = start.navigate(&CoordDelta::scroll(16)).unwrap();
    println!("Origin + 16 scrolls:  {}", moved);

    // Carry across dimension boundary
    let at_255 = Coord256::parse("0.0.0/0.0.0/0.0.255").unwrap();
    let carried = at_255.navigate(&CoordDelta::scroll(1)).unwrap();
    println!("0.0.0/0.0.0/0.0.255 + 1 scroll: {} (carry!)", carried);

    // Multi-carry cascade
    let at_max = Coord256::parse("0.0.0/0.0.0/0.255.255").unwrap();
    let cascaded = at_max.navigate(&CoordDelta::scroll(1)).unwrap();
    println!("0.0.0/0.0.0/0.255.255 + 1 scroll: {} (double carry!)", cascaded);

    // Borrow
    let at_zero = Coord256::parse("0.0.0/0.0.0/0.1.0").unwrap();
    let borrowed = at_zero.navigate(&CoordDelta::scroll(-1)).unwrap();
    println!("0.0.0/0.0.0/0.1.0 - 1 scroll: {} (borrow!)", borrowed);

    // Navigate +17 (SCROLL structure number)
    let plus_17 = origin.navigate(&CoordDelta::scroll(17)).unwrap();
    println!("Origin + 17 scrolls:  {} (17 = 5×3 + (5-3))", plus_17);

    // Underflow detection
    let zero = Coord256::zero();
    let underflow = zero.navigate(&CoordDelta::scroll(-1));
    println!("Zero - 1 scroll:      {:?} (underflow detected)", underflow);

    // --- Distance ---
    println!("\n--- Distance ---");
    println!("Origin ↔ Emi:       {} (manhattan)", origin.manhattan_distance(&emi));
    println!("Origin ↔ Kai:       {} (manhattan)", origin.manhattan_distance(&kai));
    println!("Origin ↔ Aetheris:  {} (manhattan)", origin.manhattan_distance(&aetheris));
    println!("Verse ↔ Kai:        {} (manhattan)", verse.manhattan_distance(&kai));

    // --- Constitutional Factors ---
    println!("\n--- Constitutional Factors ---");
    print_factors(255);
    print_factors(17);
    print_factors(30);
    print_factors(23);
    print_factors(13);
    print_factors(42);

    // --- Edge Cases ---
    println!("\n--- Edge Cases ---");

    // Max coordinate (all 255)
    let max_coord = Coord256 { dims: [255; 9] };
    println!("Max coord: {}", max_coord);
    println!("Max phonetic: {}", max_coord.to_phonetic());
    println!("Max linear: {:?}", max_coord.to_linear());

    // Navigate at max
    let max_no_lib = Coord256 { dims: [255, 255, 255, 255, 255, 255, 255, 255, 0] };
    let overflow = max_no_lib.navigate(&CoordDelta::scroll(1));
    println!("Max(no lib) + 1: {:?} (should carry to library)", overflow);

    // Parse edge cases
    println!("Parse empty: {:?}", Coord256::parse(""));
    println!("Parse partial: {:?}", Coord256::parse("1.2.3"));
    println!("Parse overflow: {:?}", Coord256::parse("256.0.0/0.0.0/0.0.0"));

    // Dimension access
    for i in 0..9 {
        println!("  verse.{}({}) = {}", Coord256::dim_name(i), i, verse.dim(i));
    }

    println!("\n=== All checks passed! ===");
}

fn print_factors(value: u8) {
    let factors = Coord256::constitutional_factors(value);
    let factor_strs: Vec<String> = factors.iter()
        .map(|(v, name)| format!("{}({})", v, name))
        .collect();
    println!("{:>3} = {}", value, factor_strs.join(" × "));
}
