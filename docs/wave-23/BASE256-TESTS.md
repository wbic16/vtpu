# Base256 Test Specification
**Wave:** R23W23  
**Date:** 2026-02-19

Defines all test cases required for Base256 implementation validation.

---

## 1. Correctness Tests

### T1: Encoding Roundtrip
**Requirement:** `∀b ∈ [0x00, 0xFF]: decode(encode(b)) == b`

**Test Cases:**
```rust
#[test]
fn test_roundtrip_all_bytes() {
    for byte in 0u8..=255u8 {
        let syllable = encode(byte);
        let decoded = decode(&syllable).unwrap();
        assert_eq!(decoded, byte, "Roundtrip failed for byte {:#04X} → {}", byte, syllable);
    }
}
```

**Expected Result:** All 256 bytes pass roundtrip without error.

---

### T2: Collision Detection
**Requirement:** `∀i,j ∈ [0, 255], i≠j: encode(i) ≠ encode(j)`

**Test Cases:**
```rust
#[test]
fn test_no_collisions() {
    let mut seen = HashSet::new();
    for byte in 0u8..=255u8 {
        let syllable = encode(byte);
        assert!(
            seen.insert(syllable.clone()),
            "Collision detected: byte {:#04X} and previous byte both map to '{}'",
            byte, syllable
        );
    }
    assert_eq!(seen.len(), 256, "Expected exactly 256 unique syllables");
}
```

**Expected Result:** 256 unique syllables, no duplicates.

---

### T3: Table Completeness
**Requirement:** All 256 byte values have exactly one syllable mapping.

**Test Cases:**
```rust
#[test]
fn test_table_completeness() {
    // Forward map: every byte has a syllable
    for byte in 0u8..=255u8 {
        let syllable = encode(byte);
        assert_eq!(syllable.len(), 3, "Syllable for {:#04X} is not 3 chars: '{}'", byte, syllable);
        assert!(syllable.chars().all(|c| c.is_ascii_lowercase()), 
                "Syllable '{}' contains non-lowercase-ascii", syllable);
    }
    
    // Reverse map: every syllable decodes to exactly one byte
    let mut reverse_map = HashMap::new();
    for byte in 0u8..=255u8 {
        let syllable = encode(byte);
        reverse_map.insert(syllable, byte);
    }
    assert_eq!(reverse_map.len(), 256, "Reverse map should have 256 entries");
}
```

**Expected Result:** All bytes map to 3-char lowercase syllables; reverse map is bijective.

---

## 2. Specification Compliance Tests

### T4: Known Encodings
**Requirement:** Verify examples from BASE256-SPEC.md § 5.1

**Test Vector:**
| Byte   | Expected Syllable |
|--------|-------------------|
| 0x00   | bac               |
| 0x01   | bad               |
| 0x0F   | bom               |
| 0x10   | dac               |
| 0x42   | hef               |
| 0xA5   | red               |
| 0xCA   | tif               |
| 0xFF   | zom               |

**Test Cases:**
```rust
#[test]
fn test_known_encodings() {
    assert_eq!(encode(0x00), "bac");
    assert_eq!(encode(0x01), "bad");
    assert_eq!(encode(0x0F), "bom");
    assert_eq!(encode(0x10), "dac");
    assert_eq!(encode(0x42), "hef");
    assert_eq!(encode(0xA5), "red");
    assert_eq!(encode(0xCA), "tif");
    assert_eq!(encode(0xFF), "zom");
}
```

**Expected Result:** All assertions pass.

---

### T5: Known Decodings
**Requirement:** Verify reverse mapping for known syllables

**Test Cases:**
```rust
#[test]
fn test_known_decodings() {
    assert_eq!(decode("bac").unwrap(), 0x00);
    assert_eq!(decode("bad").unwrap(), 0x01);
    assert_eq!(decode("bom").unwrap(), 0x0F);
    assert_eq!(decode("dac").unwrap(), 0x10);
    assert_eq!(decode("hef").unwrap(), 0x42);
    assert_eq!(decode("red").unwrap(), 0xA5);
    assert_eq!(decode("tif").unwrap(), 0xCA);
    assert_eq!(decode("zom").unwrap(), 0xFF);
}
```

**Expected Result:** All assertions pass.

---

### T6: Phext Coordinate Encoding
**Requirement:** Verify example from BASE256-SPEC.md § 5.2

**Input:** PhextCoord `1.5.2/3.7.3/9.1.1`  
**As bytes (1-indexed u16, low byte only):** `[0x01, 0x05, 0x02, 0x03, 0x07, 0x03, 0x09, 0x01, 0x01]`  
**Expected syllables:** `["bad", "bed", "baf", "bag", "bem", "bag", "bif", "bad", "bad"]`

**Test Cases:**
```rust
#[test]
fn test_phext_coordinate_encoding() {
    let bytes = [0x01, 0x05, 0x02, 0x03, 0x07, 0x03, 0x09, 0x01, 0x01];
    let syllables: Vec<String> = bytes.iter().map(|&b| encode(b)).collect();
    let expected = vec!["bad", "bed", "baf", "bag", "bem", "bag", "bif", "bad", "bad"];
    assert_eq!(syllables, expected);
}
```

**Expected Result:** Syllable sequence matches expected.

---

## 3. Edge Cases

### T7: Boundary Values
**Requirement:** Verify correct encoding of byte range boundaries

**Test Cases:**
```rust
#[test]
fn test_boundary_values() {
    // Lower bound
    assert_eq!(encode(0x00), "bac");
    assert_eq!(encode(0x01), "bad");
    
    // Upper bound
    assert_eq!(encode(0xFE), "zof");
    assert_eq!(encode(0xFF), "zom");
    
    // Nibble boundaries (every 16th byte)
    assert_eq!(encode(0x0F), "bom");  // End of b- range
    assert_eq!(encode(0x10), "dac");  // Start of d- range
    assert_eq!(encode(0x1F), "dom");  // End of d- range
    assert_eq!(encode(0xF0), "zac");  // Start of z- range
}
```

**Expected Result:** All boundary cases encode correctly.

---

### T8: Invalid Syllables
**Requirement:** Decoder must reject invalid input gracefully

**Test Cases:**
```rust
#[test]
fn test_invalid_syllables() {
    // Too short
    assert!(decode("ba").is_err(), "Should reject 2-char input");
    
    // Too long
    assert!(decode("bacd").is_err(), "Should reject 4-char input");
    
    // Invalid initial consonant
    assert!(decode("xac").is_err(), "Should reject 'x' as initial");
    
    // Invalid vowel
    assert!(decode("buc").is_err(), "Should reject 'u' as vowel");
    
    // Invalid final consonant
    assert!(decode("bag").is_err(), "Should reject 'g' as final");
    
    // Non-alphabetic characters
    assert!(decode("b4c").is_err(), "Should reject digit");
    assert!(decode("b-c").is_err(), "Should reject hyphen");
    
    // Uppercase (if not normalized)
    // Note: spec says "SHOULD accept uppercase but normalize" — decide if this is error or auto-fix
    // For strict test: treat uppercase as error unless decode() has normalization flag
    assert!(decode("BAC").is_err() || decode("BAC").unwrap() == 0x00, 
            "Decide: reject uppercase or auto-normalize?");
}
```

**Expected Result:** All invalid inputs return `Err` or handle gracefully per implementation choice.

---

### T9: Multi-Byte Sequences
**Requirement:** Encode/decode sequences of bytes with optional separators

**Test Cases:**
```rust
#[test]
fn test_multi_byte_encoding() {
    let bytes = vec![0x42, 0xA5, 0xFF];
    let syllables: Vec<String> = bytes.iter().map(|&b| encode(b)).collect();
    assert_eq!(syllables, vec!["hef", "red", "zom"]);
    
    // Joined string (no separators)
    let joined = syllables.join("");
    assert_eq!(joined, "hefredzom");
    
    // Hyphen-separated
    let hyphenated = syllables.join("-");
    assert_eq!(hyphenated, "hef-red-zom");
}

#[test]
fn test_multi_byte_decoding_with_separators() {
    // Decoder should ignore non-alphabetic characters
    let input = "hef-red-zom";
    let decoded = decode_sequence(input).unwrap();
    assert_eq!(decoded, vec![0x42, 0xA5, 0xFF]);
    
    // Also works without separators
    let input2 = "hefredzom";
    let decoded2 = decode_sequence(input2).unwrap();
    assert_eq!(decoded2, vec![0x42, 0xA5, 0xFF]);
}
```

**Expected Result:** Sequences encode/decode correctly with or without separators.

---

## 4. Performance Tests

### T10: Encode Latency
**Requirement:** < 1μs per byte (unoptimized)

**Test Cases:**
```rust
#[test]
#[ignore] // Run only in --release with --ignored
fn bench_encode_latency() {
    let iterations = 1_000_000;
    let start = Instant::now();
    for _ in 0..iterations {
        for byte in 0u8..=255u8 {
            let _ = encode(byte);
        }
    }
    let elapsed = start.elapsed();
    let ns_per_byte = elapsed.as_nanos() / (iterations * 256);
    assert!(ns_per_byte < 1000, "Encode took {}ns per byte (expected <1000ns)", ns_per_byte);
    println!("Encode latency: {}ns/byte", ns_per_byte);
}
```

**Expected Result:** ≤ 1000 ns/byte in release mode.

---

### T11: Decode Latency
**Requirement:** < 10μs per syllable (hash lookup or trie)

**Test Cases:**
```rust
#[test]
#[ignore]
fn bench_decode_latency() {
    let syllables: Vec<String> = (0u8..=255u8).map(|b| encode(b)).collect();
    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        for syl in &syllables {
            let _ = decode(syl).unwrap();
        }
    }
    let elapsed = start.elapsed();
    let ns_per_syllable = elapsed.as_nanos() / (iterations * 256);
    assert!(ns_per_syllable < 10_000, "Decode took {}ns per syllable (expected <10000ns)", ns_per_syllable);
    println!("Decode latency: {}ns/syllable", ns_per_syllable);
}
```

**Expected Result:** ≤ 10,000 ns/syllable in release mode.

---

## 5. Interoperability Tests

### T12: Cross-Language Consistency
**Requirement:** Rust, Python, and JS implementations produce identical output

**Test Vector File:** `test-vectors.json`
```json
{
  "version": "0.1",
  "test_cases": [
    {"byte": 0, "syllable": "bac"},
    {"byte": 1, "syllable": "bad"},
    {"byte": 15, "syllable": "bom"},
    {"byte": 66, "syllable": "hef"},
    {"byte": 165, "syllable": "red"},
    {"byte": 202, "syllable": "tif"},
    {"byte": 255, "syllable": "zom"}
  ]
}
```

**Test Cases:**
```rust
#[test]
fn test_cross_language_vectors() {
    let json = include_str!("../test-vectors.json");
    let vectors: serde_json::Value = serde_json::from_str(json).unwrap();
    for case in vectors["test_cases"].as_array().unwrap() {
        let byte = case["byte"].as_u64().unwrap() as u8;
        let expected = case["syllable"].as_str().unwrap();
        assert_eq!(encode(byte), expected);
        assert_eq!(decode(expected).unwrap(), byte);
    }
}
```

**Expected Result:** All test vectors pass in all language implementations.

---

## 6. Regression Tests

### T13: Fixed Bugs Tracker
**Requirement:** Track and prevent regression of previously discovered bugs

*(Populate as bugs are found and fixed)*

**Example:**
```rust
// Bug #1: decode("zom") returned 0xEF instead of 0xFF (2026-02-19)
// Root cause: final index calculation used wrong bit mask
#[test]
fn test_bug_1_zom_decoding() {
    assert_eq!(decode("zom").unwrap(), 0xFF, "Bug #1 regression: zom decoding");
}
```

---

## 7. Test Execution Plan

### Development Phase
```bash
cargo test                    # Run all non-ignored tests
cargo test -- --ignored       # Run benchmarks (release mode only)
cargo test --doc              # Run documentation examples
```

### CI/CD Phase
```bash
cargo test --all-features     # Full test suite
cargo test --release          # Performance validation
```

### Manual Verification
1. **Audio Test:** Record native speaker reading all 256 syllables, verify no pronunciation ambiguity
2. **Noisy Channel Test:** Play syllables over degraded audio (SNR 10 dB), measure transcription accuracy (target ≥95%)

---

## 8. Success Criteria

All tests must pass before implementation is considered complete:
- ✅ T1-T6: Correctness (100% pass rate)
- ✅ T7-T9: Edge cases (all handled gracefully)
- ✅ T10-T11: Performance (meet latency targets in --release)
- ✅ T12: Cross-language (identical output across Rust/Python/JS)
- ✅ T13: No regressions

---

## Changelog

**v0.1 (2026-02-19)** — Initial test specification (Phex)
- Defined 13 test categories
- Created test vector format
- Specified performance targets
