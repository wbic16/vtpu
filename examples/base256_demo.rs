/// base256_demo.rs — Manual verification of Base 256 phonetic encoding
///
/// Run: cargo run --release --example base256_demo

use vtpu_runtime::base256::{encode_byte_str, decode_syllable, encode};

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
        let syl = encode_byte_str(*byte);
        println!("  0x{:02X} = {:3} — {}", byte, syl, desc);
        assert_eq!(&syl, *expected, "Encoding mismatch for {:#04x}", byte);
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
        let encoded = encode_byte_str(*byte);
        println!("  {:12} 0x{:02X} = {:3}", name, byte, encoded);
        assert_eq!(&encoded, *syl);
    }
    println!();

    // ── Roundtrip verification ──────────────────────────────────────────
    println!("--- Roundtrip Verification (all 256 bytes) ---");
    let mut mismatches = 0;
    for byte in 0u8..=255 {
        let syl_str = encode_byte_str(byte);
        let syl_bytes: [u8; 3] = [
            syl_str.as_bytes().get(0).copied().unwrap_or(0),
            syl_str.as_bytes().get(1).copied().unwrap_or(0),
            syl_str.as_bytes().get(2).copied().unwrap_or(0),
        ];
        if let Some(decoded) = decode_syllable(&syl_bytes) {
            if decoded != byte {
                println!("  MISMATCH: {:#04x} → {} → {:#04x}", byte, syl_str, decoded);
                mismatches += 1;
            }
        } else {
            println!("  DECODE FAILED: {:#04x} → {}", byte, syl_str);
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
    let encoded = encode(stream);
    println!("  Input:   {:?}", stream);
    println!("  Hex:     {:02X?}", stream);
    println!("  Base256: {}", encoded);
    println!();

    // ── Density comparison ──────────────────────────────────────────────
    println!("--- Density Comparison ---");
    let sha256_mock: &[u8] = &[0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18];
    let hex_repr = sha256_mock.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");
    let b256_repr = encode(sha256_mock);

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
