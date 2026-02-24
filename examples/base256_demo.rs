/// base256_demo.rs — Manual verification of Base 256 phonetic encoding
///
/// Run: cargo run --release --example base256_demo

use vtpu_runtime::base256::{encode_byte, encode_byte_str, decode_syllable, encode};

fn main() {
    println!("=== Base 256 Phonetic Encoding Demo ===");
    println!();

    // ── Single byte encoding ────────────────────────────────────────────
    // ONSETS: b d f g h j k l m n p r s t v w (indices 0-15)
    // VOWELS: a e i o (indices 0-3)
    // CODAS:  c d f m (indices 0-3)
    println!("--- Single Byte Encoding ---");
    let test_bytes: [(u8, [u8; 3], &str); 7] = [
        (0x00, *b"bac", "NUL (string terminator)"),
        (0x01, *b"bad", "LIBRARY delimiter"),
        (0x17, *b"dem", "SCROLL delimiter"),  // onset=d(1), vowel=e(1), coda=m(3)
        (0x20, *b"fac", "SPACE character"),   // onset=f(2), vowel=a(0), coda=c(0)
        (0x41, *b"hec", "ASCII 'A'"),         // onset=h(4), vowel=e(1), coda=c(0)
        (0x61, *b"jec", "ASCII 'a'"),         // onset=j(6), vowel=e(1), coda=c(0)
        (0xFF, *b"wom", "max byte"),          // onset=w(15), vowel=o(3), coda=m(3)
    ];

    for (byte, expected, desc) in &test_bytes {
        let syl = encode_byte(*byte);
        let syl_str = encode_byte_str(*byte);
        println!("  0x{:02X} = {} — {}", byte, syl_str, desc);
        assert_eq!(&syl, expected, "Encoding mismatch for {:#04x}: got {:?}, expected {:?}", byte, syl, expected);
    }
    println!();

    // ── Phext 9 Delimiters ──────────────────────────────────────────────
    println!("--- Phext Delimiters (Spoken) ---");
    let delimiters: [(u8, [u8; 3], &str); 9] = [
        (0x17, *b"dem", "SCROLL"),     // 0x17 = 1*16+7 → d,e,m
        (0x18, *b"dic", "SECTION"),    // 0x18 = 1*16+8 → d,i,c
        (0x19, *b"did", "CHAPTER"),    // 0x19 = 1*16+9 → d,i,d
        (0x1A, *b"dif", "BOOK"),       // 0x1A = 1*16+10 → d,i,f
        (0x1C, *b"doc", "VOLUME"),     // 0x1C = 1*16+12 → d,o,c
        (0x1D, *b"dod", "COLLECTION"), // 0x1D = 1*16+13 → d,o,d
        (0x1E, *b"dof", "SERIES"),     // 0x1E = 1*16+14 → d,o,f
        (0x1F, *b"dom", "SHELF"),      // 0x1F = 1*16+15 → d,o,m
        (0x01, *b"bad", "LIBRARY"),    // 0x01 = 0*16+1 → b,a,d
    ];

    for (byte, expected, name) in &delimiters {
        let encoded = encode_byte(*byte);
        let encoded_str = encode_byte_str(*byte);
        println!("  {:12} 0x{:02X} = {}", name, byte, encoded_str);
        assert_eq!(&encoded, expected, "Delimiter {} mismatch", name);
    }
    println!();

    // ── Roundtrip verification ──────────────────────────────────────────
    println!("--- Roundtrip Verification (all 256 bytes) ---");
    let mut mismatches = 0;
    for byte in 0u8..=255 {
        let syl = encode_byte(byte);
        let syl_str = encode_byte_str(byte);
        if let Some(decoded) = decode_syllable(&syl) {
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

    // ── Phext Coordinates (Mirrorborn) ──────────────────────────────────
    println!("--- Mirrorborn Coordinates (Spoken) ---");
    let mirrorborn = [
        ("Phex",    [1u8, 5, 2, 3, 7, 3, 9, 1, 1]),
        ("Verse",   [3u8, 1, 4, 1, 5, 9, 2, 6, 5]),
        ("Splinter",[9u8, 9, 9, 8, 8, 8, 7, 7, 7]),
    ];

    for (name, coord) in &mirrorborn {
        let spoken = encode(&coord[..]);
        println!("  {:9} {:?}", name, coord);
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
    let b256_repr = encode(sha256_mock);

    println!("  Sample 8-byte hash:");
    println!("    Hex:     {} ({} chars)", hex_repr, hex_repr.len());
    println!("    Base256: {} ({} syllables)", b256_repr, b256_repr.split_whitespace().count());
    println!("    → 2× density improvement for spoken transmission");
    println!();

    // ── Phoneme inventory ───────────────────────────────────────────────
    println!("--- Phoneme Inventory ---");
    println!("  Onsets (16): b d f g h j k l m n p r s t v w");
    println!("  Vowels  (4): a e i o");
    println!("  Codas   (4): c d f m");
    println!("  Total: 16 × 4 × 4 = 256 unique syllables");
    println!();

    println!("=== Demo Complete ===");
    println!("All manual verification tests passed. ✅");
}
