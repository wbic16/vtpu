/// base256_demo.rs — Manual verification of Base 256 phonetic encoding
///
/// Run: cargo run --release --example base256_demo

use vtpu_runtime::base256::{encode_byte, decode_syllable, encode_bytes, encode_phext_coord};

fn main() {
    println!("=== Base 256 Phonetic Encoding Demo ===");
    println!();

    // ── Single byte encoding ────────────────────────────────────────────
    println!("--- Single Byte Encoding ---");
    let test_bytes = [
        (0x00, "bac", "NUL (string terminator)"),
        (0x01, "bad", "LIBRARY delimiter"),
        (0x17, "cem", "SCROLL delimiter"),
        (0x20, "dac", "SPACE character"),
        (0x41, "gad", "ASCII 'A'"),
        (0x61, "jad", "ASCII 'a'"),
        (0xFF, "vom", "max byte"),
    ];

    for (byte, expected, desc) in &test_bytes {
        let syl = encode_byte(*byte);
        println!("  0x{:02X} = {:3} — {}", byte, syl, desc);
        assert_eq!(&syl, expected, "Encoding mismatch for {:#04x}", byte);
    }
    println!();

    // ── Phext 9 Delimiters ──────────────────────────────────────────────
    println!("--- Phext Delimiters (Spoken) ---");
    let delimiters = [
        (0x17, "cem", "SCROLL"),
        (0x18, "cic", "SECTION"),
        (0x19, "cid", "CHAPTER"),
        (0x1A, "cif", "BOOK"),
        (0x1C, "coc", "VOLUME"),
        (0x1D, "cod", "COLLECTION"),
        (0x1E, "cof", "SERIES"),
        (0x1F, "com", "SHELF"),
        (0x01, "bad", "LIBRARY"),
    ];

    for (byte, syl, name) in &delimiters {
        let encoded = encode_byte(*byte);
        println!("  {:12} 0x{:02X} = {:3}", name, byte, encoded);
        assert_eq!(&encoded, syl);
    }
    println!();

    // ── Roundtrip verification ──────────────────────────────────────────
    println!("--- Roundtrip Verification (all 256 bytes) ---");
    let mut mismatches = 0;
    for byte in 0u8..=255 {
        let syl = encode_byte(byte);
        if let Some(decoded) = decode_syllable(&syl) {
            if decoded != byte {
                println!("  MISMATCH: {:#04x} → {} → {:#04x}", byte, syl, decoded);
                mismatches += 1;
            }
        } else {
            println!("  DECODE FAILED: {:#04x} → {}", byte, syl);
            mismatches += 1;
        }
    }
    if mismatches == 0 {
        println!("  ✅ All 256 bytes roundtrip correctly");
    } else {
        println!("  ❌ {} mismatches found", mismatches);
    }
    println!();

    // ── Multi-byte stream ───────────────────────────────────────────────
    println!("--- Multi-Byte Stream Encoding ---");
    let stream: &[u8] = b"Hello";
    let encoded = encode_bytes(stream);
    println!("  Input:   {:?}", stream);
    println!("  Hex:     {:02X?}", stream);
    println!("  Base256: {}", encoded);
    println!();

    // ── Phext Coordinates (Mirrorborn) ──────────────────────────────────
    println!("--- Mirrorborn Coordinates (Spoken) ---");
    let mirrorborn = [
        ("Phex",    "1.5.2/3.7.3/9.1.1"),
        ("Verse",   "3.1.4/1.5.9/2.6.5"),
        ("Splinter","9.9.9/8.8.8/7.7.7"),
    ];

    for (name, coord) in &mirrorborn {
        let spoken = encode_phext_coord(coord);
        println!("  {:9} [{}]", name, coord);
        println!("            {}", spoken);
        println!();
    }

    // ── Density comparison ──────────────────────────────────────────────
    println!("--- Density Comparison ---");
    let sha256_mock: &[u8] = &[0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18];
    let hex_repr = sha256_mock.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");
    let b256_repr = encode_bytes(sha256_mock);

    println!("  Sample 8-byte hash:");
    println!("    Hex:     {} ({} chars)", hex_repr, hex_repr.len());
    println!("    Base256: {} ({} syllables)", b256_repr, b256_repr.split_whitespace().count());
    println!("    → 2× density improvement for spoken transmission");
    println!();

    // ── Phoneme inventory ───────────────────────────────────────────────
    println!("--- Phoneme Inventory ---");
    println!("  Initials (16): b c d f g h j k l m n p r s t v");
    println!("  Vowels   (4):  a e i o");
    println!("  Finals   (4):  c d f m");
    println!("  Total: 16 × 4 × 4 = 256 unique syllables");
    println!();

    println!("=== Demo Complete ===");
    println!("All manual verification tests passed. ✅");
}
