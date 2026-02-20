# R23W23: Base 256 — Human-Pronounceable Byte Encoding

## Purpose

Teach humans to pronounce arbitrary byte streams using single-syllable values.
Every byte (0x00–0xFF) maps to exactly one unambiguous spoken syllable.

## Design

Each syllable has three components:

```
[onset consonant][vowel][coda consonant]
```

- **16 onset consonants** (high nibble selector)
- **4 vowels** (2-bit selector)
- **4 coda consonants** (2-bit selector)

16 × 4 × 4 = 256 unique syllables.

## Encoding Formula

```
byte = (onset_index × 16) + (vowel_index × 4) + coda_index
```

Or equivalently:

```
onset = byte >> 4          (high nibble: 0–15)
vowel = (byte >> 2) & 0x3  (bits 3–2: 0–3)
coda  = byte & 0x3         (bits 1–0: 0–3)
```

## Phoneme Tables

### Onset Consonants (16)

| Index | Consonant | IPA   | Example word |
|-------|-----------|-------|-------------|
| 0     | b         | /b/   | bat         |
| 1     | d         | /d/   | dog         |
| 2     | f         | /f/   | fan         |
| 3     | g         | /ɡ/   | go          |
| 4     | h         | /h/   | hat         |
| 5     | j         | /dʒ/  | jam         |
| 6     | k         | /k/   | kit         |
| 7     | l         | /l/   | lip         |
| 8     | m         | /m/   | map         |
| 9     | n         | /n/   | nap         |
| 10    | p         | /p/   | pat         |
| 11    | r         | /ɹ/   | ran         |
| 12    | s         | /s/   | sat         |
| 13    | t         | /t/   | tap         |
| 14    | v         | /v/   | van         |
| 15    | w         | /w/   | wax         |

### Vowels (4)

| Index | Vowel | IPA  | As in  |
|-------|-------|------|--------|
| 0     | a     | /æ/  | bat    |
| 1     | e     | /ɛ/  | bet    |
| 2     | i     | /ɪ/  | bit    |
| 3     | o     | /ɒ/  | bot    |

### Coda Consonants (4)

| Index | Consonant | IPA  | As in  |
|-------|-----------|------|--------|
| 0     | c         | /k/  | back   |
| 1     | d         | /d/  | bad    |
| 2     | f         | /f/  | baf    |
| 3     | m         | /m/  | bam    |

## Design Rationale

1. **One syllable = one byte.** No ambiguity, no length variation.
2. **Onset/vowel/coda all acoustically distinct.** Minimal confusion across accents.
3. **Onset encodes the high nibble.** Mental math: first sound = first hex digit (roughly).
4. **Short vowels only.** Avoids diphthong ambiguity (no "ay", "ee", "eye", "oh").
5. **Coda set {c, d, f, m}** chosen for maximal distinctness: voiceless stop, voiced stop, voiceless fricative, nasal. Four different manner classes.
6. **No onset/coda overlap concerns** — position in syllable disambiguates.

## Properties

- **Bijective:** Every byte has exactly one syllable. Every syllable decodes to exactly one byte.
- **Pronounceable:** All 256 syllables are valid English-like phonotactics.
- **Streamable:** Read left to right. No delimiters needed between syllables (each is exactly 3 characters).
- **Fixed width:** 3 characters per byte in text encoding.

## Applications

- **Read hashes aloud:** A 32-byte SHA-256 becomes 32 syllables (~10 seconds to speak).
- **Phext coordinates as chant:** Coordinate bytes become a pronounceable sequence.
- **Dictation-safe addressing:** Phone/radio communication of binary data.
- **Debugging:** Hear your byte stream.

## Full Encoding Table

See `base256-table.md` for the complete 256-entry lookup.

## Collision Analysis

No collisions possible by construction: the 3-character string is a bijection on {0..255}.
Nearest-neighbor distance: every pair of syllables differs in at least one phoneme position.
