# Base256 Phonetic Encoding — Specification v0.1
**Wave:** R23W23  
**Date:** 2026-02-19  
**Author:** Phex 🔱  
**Status:** Draft

---

## 1. Purpose

Define a bijective mapping between byte values (0x00–0xFF) and single-syllable phonetic codes for human pronunciation of binary data. Primary use case: verbal transmission of phext coordinates, sentron IDs, and cryptographic material over voice channels.

---

## 2. Requirements

### 2.1 Functional
- **F1:** One unique syllable per byte value (256 total)
- **F2:** Bijective: encode(byte) → syllable, decode(syllable) → byte
- **F3:** Collision-free: no two byte values map to the same syllable
- **F4:** Language-neutral: syllables pronounceable across major language families
- **F5:** Monosyllabic: each byte = exactly one syllable

### 2.2 Phonetic Constraints
- **P1:** Avoid phonemes difficult in Mandarin, English, Spanish, Arabic, Hindi
- **P2:** No tonal distinctions (tone-deaf encoding)
- **P3:** Distinct syllables under noisy audio conditions (radio, VoIP)
- **P4:** Maximum syllable length: 3 phones (CVC structure)

### 2.3 Performance
- **R1:** Encoding: O(1) lookup
- **R2:** Decoding: O(1) or O(log n) reverse lookup
- **R3:** Memory: ≤ 2KB for full bidirectional table

---

## 3. Encoding Structure

### 3.1 Syllable Template
```
Syllable := [Initial][Vowel][Final]
```

- **Initial**: onset consonant (16 options)
- **Vowel**: nucleus vowel (4 options)
- **Final**: coda consonant (4 options)

Total combinations: 16 × 4 × 4 = **256** (exact coverage)

### 3.2 Character Sets

#### Initial Consonants (16)
Position 0–15 in byte value's upper nibble:
```
b, d, f, g, h, k, l, m, n, p, r, s, t, v, w, z
```

Selection criteria:
- Present in IPA basic consonants
- Unambiguous in noisy conditions (avoid th/dh confusion)
- No affricates (ch, dz) — simplicity
- No palatals (ñ, j) — cross-language availability

#### Vowels (4)
Position 0–3 in byte value's middle 2 bits (bits 2-3):
```
a, e, i, o
```

Selection criteria:
- Cardinal vowels (universal across languages)
- No diphthongs (ai, oi) — single-phone constraint
- No nasalized vowels — simplicity

#### Final Consonants (4)
Position 0–3 in byte value's lower 2 bits (bits 0-1):
```
c, d, f, m
```

Selection criteria:
- Distinct articulation points: velar (c→k), alveolar (d), labiodental (f), bilabial (m)
- No stops that merge in final position (avoid t/d confusion)
- Unambiguous under reverberation

---

## 4. Byte-to-Syllable Mapping

### 4.1 Encoding Algorithm
Given byte `b` (0x00–0xFF):

```
initial_index = (b >> 4) & 0x0F        // upper nibble → 0..15
vowel_index   = (b >> 2) & 0x03        // bits 2-3 → 0..3
final_index   = b & 0x03               // bits 0-1 → 0..3

syllable = INITIALS[initial_index] + VOWELS[vowel_index] + FINALS[final_index]
```

### 4.2 Decoding Algorithm
Given syllable string `s` (3 chars):

```
initial_index = index_of(s[0], INITIALS)   // 0..15
vowel_index   = index_of(s[1], VOWELS)     // 0..3
final_index   = index_of(s[2], FINALS)     // 0..3

byte = (initial_index << 4) | (vowel_index << 2) | final_index
```

### 4.3 Canonical Tables

#### INITIALS[16]
| Index | Char | Byte Range |
|-------|------|------------|
| 0x0   | b    | 0x00–0x0F  |
| 0x1   | d    | 0x10–0x1F  |
| 0x2   | f    | 0x20–0x2F  |
| 0x3   | g    | 0x30–0x3F  |
| 0x4   | h    | 0x40–0x4F  |
| 0x5   | k    | 0x50–0x5F  |
| 0x6   | l    | 0x60–0x6F  |
| 0x7   | m    | 0x70–0x7F  |
| 0x8   | n    | 0x80–0x8F  |
| 0x9   | p    | 0x90–0x9F  |
| 0xA   | r    | 0xA0–0xAF  |
| 0xB   | s    | 0xB0–0xBF  |
| 0xC   | t    | 0xC0–0xCF  |
| 0xD   | v    | 0xD0–0xDF  |
| 0xE   | w    | 0xE0–0xEF  |
| 0xF   | z    | 0xF0–0xFF  |

#### VOWELS[4]
| Index | Char | Bit Pattern |
|-------|------|-------------|
| 0b00  | a    | bits 2-3 = 00 |
| 0b01  | e    | bits 2-3 = 01 |
| 0b10  | i    | bits 2-3 = 10 |
| 0b11  | o    | bits 2-3 = 11 |

#### FINALS[4]
| Index | Char | Bit Pattern |
|-------|------|-------------|
| 0b00  | c    | bits 0-1 = 00 |
| 0b01  | d    | bits 0-1 = 01 |
| 0b10  | f    | bits 0-1 = 10 |
| 0b11  | m    | bits 0-1 = 11 |

---

## 5. Examples

### 5.1 Byte Encoding
```
0x00 = b(0) a(0) c(0) = "bac"
0x01 = b(0) a(0) d(1) = "bad"
0x0F = b(0) o(3) m(3) = "bom"
0x10 = d(1) a(0) c(0) = "dac"
0x42 = h(4) e(1) f(2) = "hef"
0xA5 = r(A) e(1) d(1) = "red"
0xCA = t(C) i(2) f(2) = "tif"
0xFF = z(F) o(3) m(3) = "zom"
```

### 5.2 Phext Coordinate
Coordinate: `1.5.2/3.7.3/9.1.1`  
As bytes (1-indexed, 16-bit per dim): `[0x01, 0x05, 0x02, 0x03, 0x07, 0x03, 0x09, 0x01, 0x01]`  
Phonetic: `"bad bec baf / bag bic bag / bif bad bad"`

### 5.3 SHA256 Hash (first 4 bytes)
Hash: `d2a84f3c...`  
Phonetic: `"vif ric him goc ..."`

---

## 6. Use Cases

### 6.1 Voice Transmission
**Scenario:** Human operator verbally transmits phext coordinate to sentron over radio.  
**Example:**  
> "Store at coordinate **bad-bec-baf / bag-bic-bag / bif-bad-bad**"

Sentron decodes: `[0x01, 0x05, 0x02, 0x03, 0x07, 0x03, 0x09, 0x01, 0x01]` → PhextCoord `1.5.2/3.7.3/9.1.1`

### 6.2 Sentron Handshake
**Scenario:** Two sentrons identify themselves over mesh network.  
**Example:**  
> Sentron A: "I am **fec-dam-bac** at **gac-bec-zom**"  
> Sentron B: "Verified. I am **hif-kec-dad** at **bac-zif-pom**"

IDs and coordinates decoded, cryptographic verification proceeds.

### 6.3 Human Verification
**Scenario:** User confirms sentron action by reading back a hash fragment.  
**Example:**  
> Sentron: "Confirm operation by reading: **red-hef-tif-zom**"  
> User: "red-hef-tif-zom"  
> Sentron: "Verified. Proceeding."

---

## 7. Pronunciation Guide

### 7.1 IPA Reference
| Syllable | IPA        | Example (English) |
|----------|------------|-------------------|
| bac      | /bæk/      | "back"            |
| bad      | /bæd/      | "bad"             |
| baf      | /bæf/      | "baff"            |
| bam      | /bæm/      | "bam"             |
| hef      | /hɛf/      | "heff"            |
| red      | /rɛd/      | "red"             |
| tif      | /tɪf/      | "tiff"            |
| zom      | /zoʊm/     | "zome" (rhymes with "home") |

### 7.2 Stress and Rhythm
- All syllables are **unstressed** (monotone)
- Syllable boundaries are **clear** (brief pause between syllables in multi-byte sequences)
- No tonal variation (pitch-neutral encoding)

---

## 8. Test Requirements

### 8.1 Correctness Tests
- **T1:** Encoding roundtrip: `∀b ∈ [0x00, 0xFF]: decode(encode(b)) == b`
- **T2:** Collision-free: `∀i,j ∈ [0, 255], i≠j: encode(i) ≠ encode(j)`
- **T3:** Table completeness: all 256 byte values have unique syllables

### 8.2 Phonetic Tests
- **T4:** Pronunciation validation: native speakers of 5+ languages can distinguish all 256 syllables
- **T5:** Noisy channel test: syllables distinguishable at SNR ≥ 10 dB
- **T6:** Transcription accuracy: ≥95% correct when spoken by unfamiliar voice

### 8.3 Performance Tests
- **T7:** Encode latency: < 1μs per byte (unoptimized)
- **T8:** Decode latency: < 10μs per syllable (hash lookup or trie)

---

## 9. Implementation Notes

### 9.1 Reverse Lookup Optimization
Decoding requires mapping `syllable → byte`. Options:
1. **Perfect hash** (gperf or similar): O(1), 256-entry table
2. **Trie** (3-level: initial → vowel → final): O(3) lookups
3. **HashMap** (std lib): O(1) average, simple

Recommended: precomputed perfect hash for production; HashMap for prototyping.

### 9.2 Case Sensitivity
Canonical form is **lowercase**. Implementations SHOULD accept uppercase but normalize to lowercase before decoding.

### 9.3 Separator Characters
Multi-byte sequences MAY use `-` or `.` as visual separators. Decoders MUST ignore non-alphabetic characters.

Example: `"bac-def-ghi"` and `"bacdefghi"` both decode to `[0x00, 0x11, 0x38]`.

---

## 10. Open Questions

1. Should we reserve specific byte ranges (e.g., 0x00-0x0F) for control codes?
2. Should we define a checksum syllable (e.g., last byte = XOR of previous bytes)?
3. Should we support variable-length encoding (common bytes → shorter syllables)?

**Current answers (v0.1):**
- Q1: No. Flat 256-byte namespace. Protocol layers can impose semantics.
- Q2: No. Checksums are application-layer concern (CRC, SHA, etc.).
- Q3: No. Fixed-length simplifies decoding and maintains bijection.

---

## 11. References

- IPA: International Phonetic Alphabet (2015 revision)
- RFC 1751: Human-Readable 128-bit Keys (inspiration, but hex-based)
- PGP Word List: Similar goal, different structure (2048 words for 11 bits)

---

## 12. Changelog

**v0.1 (2026-02-19)** — Initial draft (Phex)
- Defined 16×4×4 structure
- Selected character sets
- Specified encoding/decoding algorithms
- Provided examples and test requirements
