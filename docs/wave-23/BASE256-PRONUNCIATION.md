# Base256 Pronunciation Guide
**Wave:** R23W23  
**Date:** 2026-02-19

Practical pronunciation guide for all 256 Base256 syllables, organized by phonetic similarity and with cross-language examples.

---

## Quick Reference: Sound Each Component

### Initial Consonants (16)
Pronounce as in standard English onset position:

| Letter | Sound | Example Word | Notes |
|--------|-------|--------------|-------|
| **b**  | /b/   | **b**at      | Voiced bilabial stop |
| **d**  | /d/   | **d**og      | Voiced alveolar stop |
| **f**  | /f/   | **f**at      | Voiceless labiodental fricative |
| **g**  | /g/   | **g**o       | Voiced velar stop |
| **h**  | /h/   | **h**at      | Voiceless glottal fricative |
| **k**  | /k/   | **k**ite     | Voiceless velar stop |
| **l**  | /l/   | **l**et      | Alveolar lateral approximant |
| **m**  | /m/   | **m**at      | Bilabial nasal |
| **n**  | /n/   | **n**et      | Alveolar nasal |
| **p**  | /p/   | **p**at      | Voiceless bilabial stop |
| **r**  | /r/   | **r**ed      | Alveolar approximant (English /ɹ/) |
| **s**  | /s/   | **s**at      | Voiceless alveolar fricative |
| **t**  | /t/   | **t**op      | Voiceless alveolar stop |
| **v**  | /v/   | **v**at      | Voiced labiodental fricative |
| **w**  | /w/   | **w**et      | Labial-velar approximant |
| **z**  | /z/   | **z**oo      | Voiced alveolar fricative |

### Vowels (4)
Short, pure vowels (no diphthongs):

| Letter | IPA  | Example Word | Cross-Language |
|--------|------|--------------|----------------|
| **a**  | /æ/  | c**a**t      | Spanish "p**a**pa" (more open) |
| **e**  | /ɛ/  | b**e**d      | French "b**è**te" |
| **i**  | /ɪ/  | b**i**t      | Spanish "s**i**" (but shorter) |
| **o**  | /oʊ/ | n**o**te     | Spanish "n**o**" (pure /o/, no glide) |

**Important:** Keep vowels short. Do not glide into diphthongs.

### Final Consonants (4)
Pronounce clearly in coda position:

| Letter | Sound | Example Word | Notes |
|--------|-------|--------------|-------|
| **c**  | /k/   | ba**ck**     | Voiceless velar stop (unreleased or lightly released) |
| **d**  | /d/   | ba**d**      | Voiced alveolar stop (may be unreleased) |
| **f**  | /f/   | pu**ff**     | Voiceless labiodental fricative |
| **m**  | /m/   | ha**m**      | Bilabial nasal |

**Important:** Final consonants are **not silent**. Articulate them clearly but don't add extra vowels (avoid "bac-**uh**").

---

## Common Syllables by Similarity Group

### Group 1: /æ/ vowel (bac, bad, baf, bam, ...)
These rhyme with English words ending in -ack, -ad, -aff, -am:

| Syllable | Sounds Like | Byte |
|----------|-------------|------|
| bac      | "back"      | 0x00 |
| bad      | "bad"       | 0x01 |
| baf      | "baff"      | 0x02 |
| bam      | "bam"       | 0x03 |
| dac      | "dak" (rhymes with "back") | 0x10 |
| fac      | "fak" (rhymes with "back") | 0x20 |
| gac      | "gak" (rhymes with "back") | 0x30 |
| ...      | ...         | ...  |

### Group 2: /ɛ/ vowel (bec, bed, bef, bem, ...)
These rhyme with English words ending in -eck, -ed, -eff, -em:

| Syllable | Sounds Like | Byte |
|----------|-------------|------|
| bec      | "beck"      | 0x04 |
| bed      | "bed"       | 0x05 |
| bef      | "beff"      | 0x06 |
| bem      | "bem"       | 0x07 |
| dec      | "deck"      | 0x14 |
| hef      | "heff"      | 0x46 |
| red      | "red"       | 0xA5 |
| ...      | ...         | ...  |

### Group 3: /ɪ/ vowel (bic, bid, bif, bim, ...)
These rhyme with English words ending in -ick, -id, -iff, -im:

| Syllable | Sounds Like | Byte |
|----------|-------------|------|
| bic      | "bick" (rhymes with "sick") | 0x08 |
| bid      | "bid"       | 0x09 |
| bif      | "biff"      | 0x0A |
| bim      | "bim"       | 0x0B |
| kic      | "kick"      | 0x58 |
| tif      | "tiff"      | 0xCA |
| ...      | ...         | ...  |

### Group 4: /oʊ/ vowel (boc, bod, bof, bom, ...)
These rhyme with English words ending in -oke, -ode, -off (but with "o" sound), -ome:

| Syllable | Sounds Like | Byte |
|----------|-------------|------|
| boc      | "boke" (rhymes with "joke") | 0x0C |
| bod      | "bode"      | 0x0D |
| bof      | "boff"      | 0x0E |
| bom      | "bome" (rhymes with "home") | 0x0F |
| hom      | "home"      | 0x4F |
| zom      | "zome" (rhymes with "home") | 0xFF |
| ...      | ...         | ...  |

---

## Practice Sequences

### Sequence 1: All Four Finals (Same Initial + Vowel)
Say these in order to practice final consonant distinction:
```
bac - bad - baf - bam
(back, bad, baff, bam)
```

### Sequence 2: All Four Vowels (Same Initial + Final)
Say these in order to practice vowel distinction:
```
bac - bec - bic - boc
(back, beck, bick, boke)
```

### Sequence 3: All Sixteen Initials (Same Vowel + Final)
Say these in order to practice initial consonant distinction:
```
bac - dac - fac - gac - hac - kac - lac - mac - nac - pac - rac - sac - tac - vac - wac - zac
```

### Sequence 4: Real-World Example (Phext Coordinate)
Coordinate `1.5.2/3.7.3/9.1.1` → bytes `[0x01, 0x05, 0x02, 0x03, 0x07, 0x03, 0x09, 0x01, 0x01]`:
```
bad - bed - baf / bag - bem - bag / bif - bad - bad
```
Pronunciation rhythm: brief pause at `/`, steady tempo within each group.

---

## Cross-Language Notes

### For Mandarin Speakers
- Final **-c** is like Pinyin final **-k** (e.g., "de" in 德 /tɤ/ + /k/)
- Final **-m** is like Pinyin final **-m** (e.g., "三" sān → "sam" if final were /m/)
- Vowel **a** is closer to Pinyin **a** (as in "ba" 八)
- Vowel **e** is closer to Pinyin **e** (as in "de" 的, but shorter)

### For Spanish Speakers
- Vowel **a** = Spanish "a" but slightly more open (English "cat")
- Vowel **e** = Spanish "e" (as in "ver")
- Vowel **i** = Spanish "i" but **shorter** (do not lengthen)
- Vowel **o** = Spanish "o" (pure, no diphthong)
- Final **c** = hard "c" as in "pac-to"
- Final **d** = "d" as in "ciu-dad" (may be softer in Spanish dialects, keep it clear)

### For Arabic Speakers
- All consonants are equivalent to their Arabic cognates except:
  - **p** (use /b/ with lip closure, no voicing during closure)
  - **v** (use /f/ with lower lip on upper teeth, but **voiced**)
- Vowels are **short**: /a/ = فتحة, /i/ = كسرة, /u/ is not used (use /o/ ≈ ضمة but slightly lower)

### For Hindi Speakers
- Consonants map to Devanagari as follows:
  - b=ब, d=द, f=फ़ (with nuqta), g=ग, h=ह, k=क, l=ल, m=म, n=न, p=प, r=र, s=स, t=त, v=व, w=व (same as v), z=ज़
- Vowels: a=अ, e=ए (short), i=इ (short), o=ओ
- Keep vowels **short** — do not lengthen as in Hindi long vowels (आ, ई, ऊ, ए, ओ)

---

## Difficult Pairs (Avoid Confusion)

These syllable pairs sound similar in noisy conditions. Practice distinguishing them:

| Pair      | Difference           | Mnemonic |
|-----------|----------------------|----------|
| **bad** / **pad** | Voiced vs voiceless initial | "**b**ad" has buzz, "**p**ad" is puff |
| **bac** / **pac** | Same as above | Ditto |
| **baf** / **bam** | Final fricative vs nasal | "ba**ff**" (air), "ba**m**" (hum) |
| **bec** / **bic** | Vowel /ɛ/ vs /ɪ/ | "b**e**ck" (bed), "b**i**ck" (bit) |
| **boc** / **bof** | Final stop vs fricative | "bo**ke**" (hard stop), "bo**ff**" (air) |

---

## Stress and Intonation

### Monotone Delivery
- **Do not** use rising/falling intonation to distinguish syllables
- **Do not** stress any syllable more than others in a sequence
- Keep pitch **flat** and **neutral** (approx. 100-120 Hz for male voices, 180-220 Hz for female)

### Rhythm
- Each syllable takes **equal time** (~200-300 ms per syllable at normal speaking rate)
- **Brief pause** (~100-150 ms) between syllables in multi-byte sequences
- **Longer pause** (~300-500 ms) at coordinate separators (`/`)

### Example Timing
```
"bad bed baf / bag bem bag / bif bad bad"
 200  200 200   400  200 200  400  200 200 200 (ms)
 [syllable]  [pause] [separator pause] [syllable] ...
```

---

## Verification Protocol

When transmitting critical data (cryptographic keys, phext coordinates), use **readback verification**:

1. **Sender** encodes and speaks the syllable sequence
2. **Receiver** repeats the sequence back verbatim
3. **Sender** confirms: "Verified" (match) or "Negative, repeat" (mismatch)

Example:
> Sender: "Store at **bad-bec-baf**"  
> Receiver: "Confirm **bad-bec-baf**"  
> Sender: "Verified"

---

## Training Exercises

### Exercise 1: Initial Consonant Ladder (3 min)
Practice all 16 initials with vowel "a" and final "c":
```
bac, dac, fac, gac, hac, kac, lac, mac, nac, pac, rac, sac, tac, vac, wac, zac
```
Repeat 3 times, increasing speed each time.

### Exercise 2: Vowel Shift (2 min)
Practice vowel transitions with initial "b" and final "d":
```
bad, bed, bid, bod
```
Repeat 5 times, ensuring clear vowel distinction.

### Exercise 3: Final Consonant Precision (2 min)
Practice final consonants with initial "t" and vowel "i":
```
tic, tid, tif, tim
```
Repeat 5 times, emphasizing final consonant clarity.

### Exercise 4: Random Byte Stream (5 min)
Use a random number generator to produce 20 bytes. Encode to Base256 and pronounce without looking at the table. Check accuracy afterward.

---

## Audio Examples (Future)

*Planned for v0.2: MP3 files with native speaker pronunciations of:*
- All 256 syllables (alphabetical order)
- Common sequences (coordinates, hashes)
- Difficult pairs (minimal pair discrimination)

---

## Changelog

**v0.1 (2026-02-19)** — Initial pronunciation guide (Phex)
- Phonetic reference for all components
- Cross-language notes (Mandarin, Spanish, Arabic, Hindi)
- Training exercises
- Difficult pair warnings
