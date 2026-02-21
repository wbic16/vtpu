/// base256_ops.rs — Base 256 Power Operations for Coordinate Arithmetic
///
/// Extends base256 phonetic encoding with positional power operations.
/// Phext coordinates map to base256 positional encoding where each
/// dimension represents a power of 256.
///
/// Powers of 256:
///   256^0 =              1  (scroll)
///   256^1 =            256  (section)
///   256^2 =         65,536  (chapter)
///   256^3 =     16,777,216  (book)
///   256^4 =  4,294,967,296  (volume)
///   256^5 = 1,099,511,627,776 (collection)
///   ...up to 256^8 (library)
///
/// Key insight: 255 = 3 × 5 × 17
///   3 = triadic (D/S/C pipes)
///   5 = pentadic (Wuxing elements)
///  17 = 5×3 + (5-3) = SCROLL structure
///
/// The maximum byte value (255) encodes the constitutional architecture.

use crate::base256;

/// 9-dimensional coordinate using base256 positional encoding.
/// Each dimension is a single byte (0-255).
///
/// Dimension order (low to high power):
///   [0] scroll    = 256^0
///   [1] section   = 256^1
///   [2] chapter   = 256^2
///   [3] book      = 256^3
///   [4] volume    = 256^4
///   [5] collection = 256^5
///   [6] series    = 256^6
///   [7] shelf     = 256^7
///   [8] library   = 256^8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord256 {
    /// Dimensions stored low-to-high: [scroll, section, chapter, book, volume, collection, series, shelf, library]
    pub dims: [u8; 9],
}

/// Delta for coordinate navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoordDelta {
    pub dims: [i16; 9],
}

/// Dimension names for display
const DIM_NAMES: [&str; 9] = [
    "scroll", "section", "chapter",
    "book", "volume", "collection",
    "series", "shelf", "library",
];

/// Powers of 256 as u64 (up to 256^8)
const POWERS: [u64; 9] = [
    1,                       // 256^0
    256,                     // 256^1
    65_536,                  // 256^2
    16_777_216,              // 256^3
    4_294_967_296,           // 256^4
    1_099_511_627_776,       // 256^5
    281_474_976_710_656,     // 256^6
    72_057_594_037_927_936,  // 256^7
    u64::MAX,                // 256^8 overflows u64, use sentinel
];

impl Coord256 {
    /// Create from phext triple notation: library.shelf.series/collection.volume.book/chapter.section.scroll
    pub fn from_triples(lss: [u8; 3], cvb: [u8; 3], css: [u8; 3]) -> Self {
        Self {
            dims: [
                css[2], // scroll
                css[1], // section
                css[0], // chapter
                cvb[2], // book
                cvb[1], // volume
                cvb[0], // collection
                lss[2], // series
                lss[1], // shelf
                lss[0], // library
            ],
        }
    }

    /// Create origin coordinate (1.1.1/1.1.1/1.1.1)
    pub fn origin() -> Self {
        Self { dims: [1; 9] }
    }

    /// Create zero coordinate (0.0.0/0.0.0/0.0.0)
    pub fn zero() -> Self {
        Self { dims: [0; 9] }
    }

    /// Parse from string "L.S.Se/C.V.B/Ch.Sec.Sc"
    pub fn parse(s: &str) -> Option<Self> {
        let triads: Vec<&str> = s.split('/').collect();
        if triads.len() != 3 {
            return None;
        }

        let parse_triad = |t: &str| -> Option<[u8; 3]> {
            let parts: Vec<&str> = t.split('.').collect();
            if parts.len() != 3 {
                return None;
            }
            Some([
                parts[0].trim().parse().ok()?,
                parts[1].trim().parse().ok()?,
                parts[2].trim().parse().ok()?,
            ])
        };

        let lss = parse_triad(triads[0])?;
        let cvb = parse_triad(triads[1])?;
        let css = parse_triad(triads[2])?;

        Some(Self::from_triples(lss, cvb, css))
    }

    /// Convert to linear address using powers of 256.
    /// scroll + section*256 + chapter*256^2 + ... + library*256^8
    ///
    /// Uses u128 internally to handle the full 9-byte range,
    /// returns None only if result exceeds u64::MAX.
    pub fn to_linear(&self) -> Option<u64> {
        let mut addr: u128 = 0;
        for i in 0..9 {
            addr += (self.dims[i] as u128) << (i * 8);
        }
        if addr > u64::MAX as u128 {
            return None;
        }
        Some(addr as u64)
    }

    /// Convert from linear address back to coordinate.
    /// Decomposes via repeated division by 256.
    pub fn from_linear(mut addr: u64) -> Self {
        let mut dims = [0u8; 9];
        for i in 0..9 {
            dims[i] = (addr & 0xFF) as u8;
            addr >>= 8;
        }
        Self { dims }
    }

    /// Navigate by adding a delta to this coordinate.
    /// Handles carry/borrow across dimensions.
    pub fn navigate(&self, delta: &CoordDelta) -> Option<Self> {
        let mut result = [0u8; 9];
        let mut carry: i16 = 0;

        for i in 0..9 {
            let val = self.dims[i] as i16 + delta.dims[i] + carry;
            if val < 0 {
                // Borrow from next dimension
                let adjusted = val + 256;
                result[i] = adjusted as u8;
                carry = -1;
            } else if val > 255 {
                // Carry to next dimension
                result[i] = (val - 256) as u8;
                carry = 1;
            } else {
                result[i] = val as u8;
                carry = 0;
            }
        }

        if carry != 0 {
            return None; // Overflow/underflow beyond library
        }

        Some(Self { dims: result })
    }

    /// Manhattan distance between two coordinates.
    /// Sum of absolute differences across all dimensions.
    pub fn manhattan_distance(&self, other: &Self) -> u32 {
        self.dims.iter()
            .zip(other.dims.iter())
            .map(|(&a, &b)| (a as i16 - b as i16).unsigned_abs() as u32)
            .sum()
    }

    /// Weighted distance using powers of 256.
    /// Higher dimensions contribute exponentially more.
    pub fn weighted_distance(&self, other: &Self) -> u64 {
        let mut dist: u64 = 0;
        for i in 0..8 {
            let diff = (self.dims[i] as i16 - other.dims[i] as i16).unsigned_abs() as u64;
            dist = dist.saturating_add(diff.saturating_mul(POWERS[i]));
        }
        dist
    }

    /// Get a specific dimension by index (0=scroll, 8=library)
    pub fn dim(&self, index: usize) -> u8 {
        self.dims[index]
    }

    /// Get dimension name
    pub fn dim_name(index: usize) -> &'static str {
        DIM_NAMES[index]
    }

    /// Encode coordinate as base256 phonetic string.
    /// Format: "lib.shelf.series / col.vol.book / ch.sec.scroll"
    /// Each dimension encoded as CVC syllable.
    pub fn to_phonetic(&self) -> String {
        format!(
            "{}.{}.{}/{}.{}.{}/{}.{}.{}",
            base256::encode_byte(self.dims[8]), // library
            base256::encode_byte(self.dims[7]), // shelf
            base256::encode_byte(self.dims[6]), // series
            base256::encode_byte(self.dims[5]), // collection
            base256::encode_byte(self.dims[4]), // volume
            base256::encode_byte(self.dims[3]), // book
            base256::encode_byte(self.dims[2]), // chapter
            base256::encode_byte(self.dims[1]), // section
            base256::encode_byte(self.dims[0]), // scroll
        )
    }

    /// Display as phext triple notation: L.S.Se/C.V.B/Ch.Sec.Sc
    pub fn to_triple_string(&self) -> String {
        format!(
            "{}.{}.{}/{}.{}.{}/{}.{}.{}",
            self.dims[8], self.dims[7], self.dims[6],
            self.dims[5], self.dims[4], self.dims[3],
            self.dims[2], self.dims[1], self.dims[0],
        )
    }

    /// Factor a dimension value into constitutional components.
    /// 255 = 3 × 5 × 17
    ///   3 = triadic (pipes)
    ///   5 = pentadic (elements)
    ///  17 = 5×3 + (5-3) (SCROLL encoding)
    pub fn constitutional_factors(value: u8) -> Vec<(u8, &'static str)> {
        let mut factors = Vec::new();
        let mut v = value as u16;

        if v == 0 {
            return vec![(0, "void")];
        }

        let checks: &[(u8, &str)] = &[
            (17, "scroll-structure (5×3 + (5-3))"),
            (13, "transformation"),
            (7, "completion"),
            (5, "pentadic (Wuxing)"),
            (3, "triadic (pipes)"),
            (2, "duality (polarity)"),
        ];

        for &(factor, name) in checks {
            while v % factor as u16 == 0 {
                factors.push((factor, name));
                v /= factor as u16;
            }
        }

        if v > 1 {
            factors.push((v as u8, "prime"));
        } else if factors.is_empty() {
            factors.push((value, "unity"));
        }

        factors
    }
}

impl fmt::Display for Coord256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_triple_string())
    }
}

use std::fmt;

impl CoordDelta {
    /// Create delta with only scroll dimension
    pub fn scroll(d: i16) -> Self {
        let mut dims = [0i16; 9];
        dims[0] = d;
        Self { dims }
    }

    /// Create delta with only section dimension
    pub fn section(d: i16) -> Self {
        let mut dims = [0i16; 9];
        dims[1] = d;
        Self { dims }
    }

    /// Create delta with only chapter dimension
    pub fn chapter(d: i16) -> Self {
        let mut dims = [0i16; 9];
        dims[2] = d;
        Self { dims }
    }

    /// Create delta for any dimension by index
    pub fn dim(index: usize, d: i16) -> Self {
        let mut dims = [0i16; 9];
        dims[index] = d;
        Self { dims }
    }

    /// Create multi-dimensional delta
    pub fn new(dims: [i16; 9]) -> Self {
        Self { dims }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Construction ===

    #[test]
    fn test_origin() {
        let coord = Coord256::origin();
        assert_eq!(coord.dims, [1; 9]);
        assert_eq!(coord.to_triple_string(), "1.1.1/1.1.1/1.1.1");
    }

    #[test]
    fn test_zero() {
        let coord = Coord256::zero();
        assert_eq!(coord.dims, [0; 9]);
        assert_eq!(coord.to_triple_string(), "0.0.0/0.0.0/0.0.0");
    }

    #[test]
    fn test_from_triples() {
        // Kai's coordinate: 1.2.3/4.5.6/7.8.9
        let coord = Coord256::from_triples([1, 2, 3], [4, 5, 6], [7, 8, 9]);
        assert_eq!(coord.to_triple_string(), "1.2.3/4.5.6/7.8.9");
        assert_eq!(coord.dim(0), 9);  // scroll
        assert_eq!(coord.dim(8), 1);  // library
    }

    // === Parsing ===

    #[test]
    fn test_parse_origin() {
        let coord = Coord256::parse("1.1.1/1.1.1/1.1.1").unwrap();
        assert_eq!(coord, Coord256::origin());
    }

    #[test]
    fn test_parse_kai() {
        let coord = Coord256::parse("1.2.3/4.5.6/7.8.9").unwrap();
        assert_eq!(coord.to_triple_string(), "1.2.3/4.5.6/7.8.9");
    }

    #[test]
    fn test_parse_verse() {
        let coord = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
        assert_eq!(coord.dim(0), 5);  // scroll = 5
        assert_eq!(coord.dim(8), 3);  // library = 3
    }

    #[test]
    fn test_parse_aetheris() {
        let coord = Coord256::parse("13.13.13/13.13.13/13.13.13").unwrap();
        assert!(coord.dims.iter().all(|&d| d == 13));
    }

    #[test]
    fn test_parse_invalid() {
        assert!(Coord256::parse("1.2/3.4/5.6").is_none());
        assert!(Coord256::parse("1.2.3/4.5.6").is_none());
        assert!(Coord256::parse("not a coord").is_none());
    }

    // === Linear Encoding ===

    #[test]
    fn test_linear_zero() {
        let coord = Coord256::zero();
        assert_eq!(coord.to_linear(), Some(0));
    }

    #[test]
    fn test_linear_origin_overflows() {
        // 1.1.1/1.1.1/1.1.1 — library=1 means 256^8 which exceeds u64
        let coord = Coord256::origin();
        assert!(coord.to_linear().is_none());
    }

    #[test]
    fn test_linear_no_library() {
        // 0.1.1/1.1.1/1.1.1 — library=0, everything else 1
        let coord = Coord256::parse("0.1.1/1.1.1/1.1.1").unwrap();
        let linear = coord.to_linear().unwrap();
        let expected: u64 = (0..8).map(|i: u32| 1u64 << (i * 8)).sum();
        assert_eq!(linear, expected);
    }

    #[test]
    fn test_linear_scroll_only() {
        // 0.0.0/0.0.0/0.0.17 = scroll=17, everything else = 0
        let mut coord = Coord256::zero();
        coord.dims[0] = 17; // scroll = 17
        assert_eq!(coord.to_linear(), Some(17));
    }

    #[test]
    fn test_linear_roundtrip() {
        // Use library=0 to stay within u64 range
        let original = Coord256::parse("0.1.4/1.5.9/2.6.5").unwrap();
        let linear = original.to_linear().unwrap();
        let reconstructed = Coord256::from_linear(linear);
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_linear_roundtrip_kai_style() {
        // 0.2.3/4.5.6/7.8.9 (library=0 for u64 safety)
        let original = Coord256::parse("0.2.3/4.5.6/7.8.9").unwrap();
        let linear = original.to_linear().unwrap();
        let reconstructed = Coord256::from_linear(linear);
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_linear_overflow_with_library() {
        // Any non-zero library overflows u64
        let coord = Coord256::parse("1.2.3/4.5.6/7.8.9").unwrap();
        assert!(coord.to_linear().is_none());
    }

    #[test]
    fn test_linear_roundtrip_max() {
        // All 255s except library (which overflows)
        let coord = Coord256 {
            dims: [255, 255, 255, 255, 255, 255, 255, 255, 0],
        };
        let linear = coord.to_linear().unwrap();
        let reconstructed = Coord256::from_linear(linear);
        assert_eq!(coord, reconstructed);
    }

    // === Navigation ===

    #[test]
    fn test_navigate_scroll_forward() {
        let coord = Coord256::origin();
        let result = coord.navigate(&CoordDelta::scroll(16)).unwrap();
        assert_eq!(result.dim(0), 17); // scroll: 1 + 16 = 17
        assert_eq!(result.dim(1), 1);  // section unchanged
    }

    #[test]
    fn test_navigate_scroll_backward() {
        let coord = Coord256::parse("1.1.1/1.1.1/1.1.17").unwrap();
        let result = coord.navigate(&CoordDelta::scroll(-16)).unwrap();
        assert_eq!(result.dim(0), 1); // scroll: 17 - 16 = 1
    }

    #[test]
    fn test_navigate_carry() {
        // scroll=255, add 1 → scroll=0, section carries +1
        let mut coord = Coord256::zero();
        coord.dims[0] = 255;
        coord.dims[1] = 0;
        let result = coord.navigate(&CoordDelta::scroll(1)).unwrap();
        assert_eq!(result.dim(0), 0);
        assert_eq!(result.dim(1), 1);
    }

    #[test]
    fn test_navigate_borrow() {
        // scroll=0, subtract 1 → scroll=255, section borrows -1
        let mut coord = Coord256::zero();
        coord.dims[0] = 0;
        coord.dims[1] = 1;
        let result = coord.navigate(&CoordDelta::scroll(-1)).unwrap();
        assert_eq!(result.dim(0), 255);
        assert_eq!(result.dim(1), 0);
    }

    #[test]
    fn test_navigate_multi_carry() {
        // scroll=255, section=255, add scroll+1 → carries cascade
        let mut coord = Coord256::zero();
        coord.dims[0] = 255;
        coord.dims[1] = 255;
        let result = coord.navigate(&CoordDelta::scroll(1)).unwrap();
        assert_eq!(result.dim(0), 0);
        assert_eq!(result.dim(1), 0);
        assert_eq!(result.dim(2), 1); // chapter gets carry
    }

    #[test]
    fn test_navigate_underflow() {
        // At zero, trying to go below should fail
        let coord = Coord256::zero();
        assert!(coord.navigate(&CoordDelta::scroll(-1)).is_none());
    }

    #[test]
    fn test_navigate_17_scrolls() {
        let coord = Coord256::origin();
        let result = coord.navigate(&CoordDelta::scroll(17)).unwrap();
        assert_eq!(result.dim(0), 18); // 1 + 17 = 18
    }

    // === Distance ===

    #[test]
    fn test_manhattan_same() {
        let coord = Coord256::origin();
        assert_eq!(coord.manhattan_distance(&coord), 0);
    }

    #[test]
    fn test_manhattan_adjacent_scroll() {
        let a = Coord256::parse("1.1.1/1.1.1/1.1.1").unwrap();
        let b = Coord256::parse("1.1.1/1.1.1/1.1.2").unwrap();
        assert_eq!(a.manhattan_distance(&b), 1);
    }

    #[test]
    fn test_manhattan_kai_origin() {
        let origin = Coord256::origin();
        let kai = Coord256::parse("1.2.3/4.5.6/7.8.9").unwrap();
        // Differences: scroll=8, section=7, chapter=6, book=5, volume=4, collection=3, series=2, shelf=1, library=0
        assert_eq!(origin.manhattan_distance(&kai), 0+1+2+3+4+5+6+7+8);
    }

    // === Phonetic Encoding ===

    #[test]
    fn test_phonetic_origin() {
        let coord = Coord256::origin();
        let phonetic = coord.to_phonetic();
        // 1 = 0x01: initial b(0), vowel a(0), final d(1) = "bad"
        assert_eq!(phonetic, "bad.bad.bad/bad.bad.bad/bad.bad.bad");
    }

    #[test]
    fn test_phonetic_scroll_17() {
        let mut coord = Coord256::zero();
        coord.dims[0] = 0x17; // SCROLL delimiter
        let phonetic = coord.to_phonetic();
        // 0x17: initial b(1), vowel i(1), final m(3) = "bim" ... wait
        // Actually: 0x17 = 23 decimal
        // high nibble = 1, low = 7
        // initial = INITIALS[1] = 'c'
        // vowel = VOWELS[(7>>2)&3] = VOWELS[1] = 'e'
        // final = FINALS[7&3] = FINALS[3] = 'm'
        // = "cem"
        assert!(phonetic.ends_with("cem"));
    }

    #[test]
    fn test_phonetic_255_vov() {
        let mut coord = Coord256::zero();
        coord.dims[0] = 255;
        let phonetic = coord.to_phonetic();
        // 255 = 0xFF: initial v(15), vowel o(3), final v... wait
        // FINALS = "cdfm", not "v"
        // Let me recalculate: 0xFF
        // high nibble = 15 → initial = INITIALS[15] = 'v'
        // low nibble = 15 → vowel = VOWELS[(15>>2)&3] = VOWELS[3] = 'o'
        //                   final = FINALS[15&3] = FINALS[3] = 'm'
        // = "vom", not "vov"!
        assert!(phonetic.ends_with("vom"));
    }

    // === Constitutional Factors ===

    #[test]
    fn test_factors_255() {
        let factors = Coord256::constitutional_factors(255);
        // 255 = 3 × 5 × 17
        let values: Vec<u8> = factors.iter().map(|&(v, _)| v).collect();
        assert!(values.contains(&3));
        assert!(values.contains(&5));
        assert!(values.contains(&17));
    }

    #[test]
    fn test_factors_17() {
        let factors = Coord256::constitutional_factors(17);
        // 17 is prime in standard factoring, but our constitutional factors
        // recognize it as scroll-structure
        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].0, 17);
    }

    #[test]
    fn test_factors_zero() {
        let factors = Coord256::constitutional_factors(0);
        assert_eq!(factors[0].1, "void");
    }

    #[test]
    fn test_factors_30() {
        // 30 = 2 × 3 × 5 (SERIES delimiter value)
        let factors = Coord256::constitutional_factors(30);
        let values: Vec<u8> = factors.iter().map(|&(v, _)| v).collect();
        assert!(values.contains(&5));
        assert!(values.contains(&3));
        assert!(values.contains(&2));
    }

    // === Display ===

    #[test]
    fn test_display() {
        let coord = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
        assert_eq!(format!("{}", coord), "3.1.4/1.5.9/2.6.5");
    }

    #[test]
    fn test_dim_names() {
        assert_eq!(Coord256::dim_name(0), "scroll");
        assert_eq!(Coord256::dim_name(8), "library");
        assert_eq!(Coord256::dim_name(4), "volume");
    }

    // === Edge Cases ===

    #[test]
    fn test_parse_overflow_256() {
        // 256 doesn't fit in u8
        assert!(Coord256::parse("256.0.0/0.0.0/0.0.0").is_none());
    }

    #[test]
    fn test_parse_negative() {
        assert!(Coord256::parse("-1.0.0/0.0.0/0.0.0").is_none());
    }

    #[test]
    fn test_parse_whitespace() {
        // Should handle trimmed whitespace
        let coord = Coord256::parse(" 1.1.1 / 1.1.1 / 1.1.1 ").unwrap();
        assert_eq!(coord, Coord256::origin());
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(Coord256::parse("").is_none());
    }

    #[test]
    fn test_parse_too_few_triads() {
        assert!(Coord256::parse("1.1.1/1.1.1").is_none());
    }

    #[test]
    fn test_parse_too_many_triads() {
        assert!(Coord256::parse("1.1.1/1.1.1/1.1.1/1.1.1").is_none());
    }

    #[test]
    fn test_parse_non_numeric() {
        assert!(Coord256::parse("a.b.c/d.e.f/g.h.i").is_none());
    }

    #[test]
    fn test_navigate_large_positive_delta() {
        // Jump +255 scrolls from origin
        let origin = Coord256::origin();
        let result = origin.navigate(&CoordDelta::scroll(255)).unwrap();
        // 1 + 255 = 256 → carry: scroll=0, section=1+1=2
        assert_eq!(result.dim(0), 0);
        assert_eq!(result.dim(1), 2); // section carried
    }

    #[test]
    fn test_navigate_all_dimensions() {
        let origin = Coord256::origin();
        let delta = CoordDelta::new([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let result = origin.navigate(&delta).unwrap();
        assert_eq!(result.dim(0), 2);  // scroll: 1+1
        assert_eq!(result.dim(1), 3);  // section: 1+2
        assert_eq!(result.dim(2), 4);  // chapter: 1+3
        assert_eq!(result.dim(8), 10); // library: 1+9
    }

    #[test]
    fn test_navigate_zero_delta() {
        let verse = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
        let delta = CoordDelta::new([0; 9]);
        let result = verse.navigate(&delta).unwrap();
        assert_eq!(result, verse);
    }

    #[test]
    fn test_max_coord_all_255() {
        let max = Coord256 { dims: [255; 9] };
        assert_eq!(max.to_triple_string(), "255.255.255/255.255.255/255.255.255");
        assert!(max.to_linear().is_none()); // overflows
    }

    #[test]
    fn test_max_no_library_linear() {
        let max_no_lib = Coord256 { dims: [255, 255, 255, 255, 255, 255, 255, 255, 0] };
        let linear = max_no_lib.to_linear().unwrap();
        assert_eq!(linear, u64::MAX); // 0x00FFFFFFFFFFFFFF
    }

    #[test]
    fn test_linear_from_linear_zero() {
        let coord = Coord256::from_linear(0);
        assert_eq!(coord, Coord256::zero());
    }

    #[test]
    fn test_linear_from_linear_max() {
        let coord = Coord256::from_linear(u64::MAX);
        // u64::MAX = 0xFFFFFFFFFFFFFFFF = all 8 lower dims = 255, library = 0
        for i in 0..8 {
            assert_eq!(coord.dim(i), 255);
        }
        assert_eq!(coord.dim(8), 0); // library stays 0
    }

    #[test]
    fn test_weighted_distance_same() {
        let coord = Coord256::origin();
        assert_eq!(coord.weighted_distance(&coord), 0);
    }

    #[test]
    fn test_weighted_distance_library_heavy() {
        // Difference in shelf dimension weighs much more than scroll
        let a = Coord256::parse("0.1.0/0.0.0/0.0.0").unwrap();
        let b = Coord256::parse("0.2.0/0.0.0/0.0.0").unwrap();
        let c = Coord256::parse("0.0.0/0.0.0/0.0.1").unwrap();
        let d = Coord256::parse("0.0.0/0.0.0/0.0.2").unwrap();
        let shelf_dist = a.weighted_distance(&b);  // 1 × 256^7
        let scroll_dist = c.weighted_distance(&d);  // 1 × 256^0
        assert!(shelf_dist > scroll_dist);
        assert_eq!(scroll_dist, 1);
    }

    #[test]
    fn test_coord_equality() {
        let a = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
        let b = Coord256::from_triples([3, 1, 4], [1, 5, 9], [2, 6, 5]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_coord_inequality() {
        let a = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
        let b = Coord256::parse("3.1.4/1.5.9/2.6.6").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_constitutional_factors_1() {
        let factors = Coord256::constitutional_factors(1);
        // 1 = unity (indivisible, not prime, not void)
        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].0, 1);
        assert_eq!(factors[0].1, "unity");
    }

    #[test]
    fn test_constitutional_factors_42() {
        // 42 = 2 × 3 × 7 (answer to everything)
        let factors = Coord256::constitutional_factors(42);
        let values: Vec<u8> = factors.iter().map(|&(v, _)| v).collect();
        assert!(values.contains(&7));  // completion
        assert!(values.contains(&3));  // triadic
        assert!(values.contains(&2));  // duality
    }

    #[test]
    fn test_navigate_section_forward() {
        let coord = Coord256::origin();
        let result = coord.navigate(&CoordDelta::section(1)).unwrap();
        assert_eq!(result.dim(0), 1);  // scroll unchanged
        assert_eq!(result.dim(1), 2);  // section: 1+1 = 2
    }

    #[test]
    fn test_navigate_chapter_forward() {
        let coord = Coord256::origin();
        let result = coord.navigate(&CoordDelta::chapter(5)).unwrap();
        assert_eq!(result.dim(2), 6);  // chapter: 1+5 = 6
    }

    #[test]
    fn test_coord_clone() {
        let a = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_coord_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let a = Coord256::parse("3.1.4/1.5.9/2.6.5").unwrap();
        set.insert(a);
        assert!(set.contains(&a));
        let b = Coord256::parse("1.1.1/1.1.1/1.1.1").unwrap();
        assert!(!set.contains(&b));
    }
}
