/// Base 256 — Human-Pronounceable Byte Encoding (R23W23)
///
/// Every byte (0x00–0xFF) maps to exactly one 3-character syllable:
///   [onset consonant][vowel][coda consonant]
///
/// 16 onsets × 4 vowels × 4 codas = 256 unique syllables.
///
/// Encoding: byte = (onset << 4) | (vowel << 2) | coda

const ONSETS: [u8; 16] = [b'b', b'd', b'f', b'g', b'h', b'j', b'k', b'l',
                           b'm', b'n', b'p', b'r', b's', b't', b'v', b'w'];
const VOWELS: [u8; 4] = [b'a', b'e', b'i', b'o'];
const CODAS: [u8; 4] = [b'c', b'd', b'f', b'm'];

/// Encode a single byte to its 3-character syllable.
pub fn encode_byte(byte: u8) -> [u8; 3] {
    let onset = ONSETS[(byte >> 4) as usize];
    let vowel = VOWELS[((byte >> 2) & 0x3) as usize];
    let coda = CODAS[(byte & 0x3) as usize];
    [onset, vowel, coda]
}

/// Encode a single byte to a String syllable.
pub fn encode_byte_str(byte: u8) -> String {
    let arr = encode_byte(byte);
    String::from_utf8_lossy(&arr).into_owned()
}

/// Encode a byte slice to a space-separated syllable string.
pub fn encode(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len() * 4);
    for (i, &byte) in data.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        let syl = encode_byte(byte);
        result.push(syl[0] as char);
        result.push(syl[1] as char);
        result.push(syl[2] as char);
    }
    result
}

/// Encode a byte slice with no separators (fixed 3-char per byte).
pub fn encode_compact(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len() * 3);
    for &byte in data {
        let syl = encode_byte(byte);
        result.push(syl[0] as char);
        result.push(syl[1] as char);
        result.push(syl[2] as char);
    }
    result
}

/// Decode a 3-character syllable to a byte. Returns None if invalid.
pub fn decode_syllable(syl: &[u8; 3]) -> Option<u8> {
    let onset = ONSETS.iter().position(|&c| c == syl[0])?;
    let vowel = VOWELS.iter().position(|&c| c == syl[1])?;
    let coda = CODAS.iter().position(|&c| c == syl[2])?;
    Some(((onset as u8) << 4) | ((vowel as u8) << 2) | (coda as u8))
}

/// Decode a space-separated syllable string back to bytes.
pub fn decode(encoded: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    for syllable in encoded.split_whitespace() {
        let bytes = syllable.as_bytes();
        if bytes.len() != 3 {
            return None;
        }
        let syl = [bytes[0], bytes[1], bytes[2]];
        result.push(decode_syllable(&syl)?);
    }
    Some(result)
}

/// Decode a compact (no separator) string. Length must be multiple of 3.
pub fn decode_compact(encoded: &str) -> Option<Vec<u8>> {
    let bytes = encoded.as_bytes();
    if bytes.len() % 3 != 0 {
        return None;
    }
    let mut result = Vec::with_capacity(bytes.len() / 3);
    for chunk in bytes.chunks(3) {
        let syl = [chunk[0], chunk[1], chunk[2]];
        result.push(decode_syllable(&syl)?);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Boundary tests ===

    #[test]
    fn byte_zero() {
        assert_eq!(encode_byte_str(0x00), "bac");
    }

    #[test]
    fn byte_max() {
        assert_eq!(encode_byte_str(0xFF), "wom");
    }

    #[test]
    fn byte_one() {
        assert_eq!(encode_byte_str(0x01), "bad");
    }

    #[test]
    fn byte_two() {
        assert_eq!(encode_byte_str(0x02), "baf");
    }

    #[test]
    fn byte_three() {
        assert_eq!(encode_byte_str(0x03), "bam");
    }

    // === Onset boundary tests (first byte of each onset) ===

    #[test]
    fn onset_boundaries() {
        assert_eq!(encode_byte_str(0x00), "bac"); // b
        assert_eq!(encode_byte_str(0x10), "dac"); // d
        assert_eq!(encode_byte_str(0x20), "fac"); // f
        assert_eq!(encode_byte_str(0x30), "gac"); // g
        assert_eq!(encode_byte_str(0x40), "hac"); // h
        assert_eq!(encode_byte_str(0x50), "jac"); // j
        assert_eq!(encode_byte_str(0x60), "kac"); // k
        assert_eq!(encode_byte_str(0x70), "lac"); // l
        assert_eq!(encode_byte_str(0x80), "mac"); // m
        assert_eq!(encode_byte_str(0x90), "nac"); // n
        assert_eq!(encode_byte_str(0xA0), "pac"); // p
        assert_eq!(encode_byte_str(0xB0), "rac"); // r
        assert_eq!(encode_byte_str(0xC0), "sac"); // s
        assert_eq!(encode_byte_str(0xD0), "tac"); // t
        assert_eq!(encode_byte_str(0xE0), "vac"); // v
        assert_eq!(encode_byte_str(0xF0), "wac"); // w
    }

    // === ASCII landmark tests ===

    #[test]
    fn ascii_newline() {
        assert_eq!(encode_byte_str(0x0A), "bif"); // LF
    }

    #[test]
    fn ascii_space() {
        assert_eq!(encode_byte_str(0x20), "fac"); // space
    }

    #[test]
    fn ascii_upper_a() {
        assert_eq!(encode_byte_str(0x41), "had"); // 'A'
    }

    #[test]
    fn ascii_lower_a() {
        assert_eq!(encode_byte_str(0x61), "kad"); // 'a'
    }

    #[test]
    fn ascii_zero() {
        assert_eq!(encode_byte_str(0x30), "gac"); // '0'
    }

    // === Bijection: every byte round-trips ===

    #[test]
    fn roundtrip_all_256() {
        for byte in 0u8..=255 {
            let syl = encode_byte(byte);
            let decoded = decode_syllable(&syl);
            assert_eq!(decoded, Some(byte), "failed roundtrip for byte {}", byte);
        }
    }

    // === Uniqueness: no two bytes produce the same syllable ===

    #[test]
    fn all_syllables_unique() {
        let mut seen = std::collections::HashSet::new();
        for byte in 0u8..=255 {
            let syl = encode_byte(byte);
            assert!(seen.insert(syl), "duplicate syllable for byte {}", byte);
        }
        assert_eq!(seen.len(), 256);
    }

    // === Encode/decode stream tests ===

    #[test]
    fn encode_empty() {
        assert_eq!(encode(&[]), "");
    }

    #[test]
    fn encode_single() {
        assert_eq!(encode(&[0x00]), "bac");
    }

    #[test]
    fn encode_multiple() {
        assert_eq!(encode(&[0x00, 0xFF]), "bac wom");
    }

    #[test]
    fn decode_empty() {
        assert_eq!(decode(""), Some(vec![]));
    }

    #[test]
    fn decode_single() {
        assert_eq!(decode("bac"), Some(vec![0x00]));
    }

    #[test]
    fn decode_multiple() {
        assert_eq!(decode("bac wom"), Some(vec![0x00, 0xFF]));
    }

    #[test]
    fn decode_invalid_onset() {
        assert_eq!(decode("xac"), None);
    }

    #[test]
    fn decode_invalid_vowel() {
        assert_eq!(decode("buc"), None);
    }

    #[test]
    fn decode_invalid_coda() {
        assert_eq!(decode("bax"), None);
    }

    #[test]
    fn decode_wrong_length() {
        assert_eq!(decode("ba"), None);
    }

    // === Compact mode ===

    #[test]
    fn compact_roundtrip() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode_compact(&data);
        assert_eq!(encoded.len(), 768); // 256 * 3
        let decoded = decode_compact(&encoded);
        assert_eq!(decoded, Some(data));
    }

    #[test]
    fn compact_invalid_length() {
        assert_eq!(decode_compact("ba"), None);
    }

    // === Phext delimiter tests ===

    #[test]
    fn phext_scroll_delimiter() {
        assert_eq!(encode_byte_str(0x17), "dem"); // SCROLL
    }

    #[test]
    fn phext_section_delimiter() {
        assert_eq!(encode_byte_str(0x18), "dic"); // SECTION
    }

    #[test]
    fn phext_chapter_delimiter() {
        assert_eq!(encode_byte_str(0x19), "did"); // CHAPTER
    }

    #[test]
    fn phext_book_delimiter() {
        assert_eq!(encode_byte_str(0x1A), "dif"); // BOOK
    }

    #[test]
    fn phext_volume_delimiter() {
        assert_eq!(encode_byte_str(0x1C), "doc"); // VOLUME
    }

    #[test]
    fn phext_collection_delimiter() {
        assert_eq!(encode_byte_str(0x1D), "dod"); // COLLECTION
    }

    #[test]
    fn phext_series_delimiter() {
        assert_eq!(encode_byte_str(0x1E), "dof"); // SERIES
    }

    #[test]
    fn phext_shelf_delimiter() {
        assert_eq!(encode_byte_str(0x1F), "dom"); // SHELF
    }

    #[test]
    fn phext_library_delimiter() {
        assert_eq!(encode_byte_str(0x01), "bad"); // LIBRARY
    }

    // === "Hello" as syllables ===

    #[test]
    fn hello_world() {
        let hello = b"Hello";
        let encoded = encode(hello);
        // H=0x48, e=0x65, l=0x6C, l=0x6C, o=0x6F
        assert_eq!(encoded, "hic ked koc koc kom");
    }

    // === Bit-level encoding verification ===

    #[test]
    fn bit_structure() {
        // 0xA5 = 1010_0101 = onset:10(p) vowel:01(e) coda:01(d)
        assert_eq!(encode_byte_str(0xA5), "ped");
        // 0x5A = 0101_1010 = onset:5(j) vowel:10(i) coda:10(f)
        assert_eq!(encode_byte_str(0x5A), "jif");
    }
}
