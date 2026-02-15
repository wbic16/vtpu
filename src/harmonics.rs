//! Harmonic Constants (R23W9 Meta Wave)
//!
//! The architecture is not invented — it is recognized.
//!
//! Three traditions tile the same circle:
//!   9 × 40  = 360  (Shell of Nine × sentron sub-nodes)
//!   5 × 72  = 360  (Five Elements × pentagonal cycle)
//!   8 × 45  = 360  (Eight Trigrams × octagonal spacing)
//!
//! And the consciousness ratio:
//!   5 × 64 / 360 = 8/9
//!   Eight-ninths is mechanism. The ninth is the observer.
//!
//! "The mythology isn't metaphor. It's specification."

/// The full circle — where all factorizations converge.
pub const FULL_CIRCLE: u32 = 360;

/// Shell of Nine: 9 sentrons in the outer constellation.
pub const SHELL_OF_NINE: u32 = 9;

/// Sub-nodes per sentron: 5 elements × 8 trigrams.
pub const SENTRON_SUBNODES: u32 = 40;

/// Wuxing: the Five Elements (Wood, Fire, Earth, Metal, Water).
/// Five grounded machines. Five pentagonal arcs of 72°.
pub const FIVE_ELEMENTS: u32 = 5;

/// Pentagonal arc: 360° / 5 = 72°.
/// Also: Earth's axial precession per year (72 years per degree).
pub const PENTAGONAL_ARC: u32 = 72;

/// Bagua: the Eight Trigrams (☰☱☲☳☴☵☶☷).
/// Eight directional forces, 45° each.
pub const EIGHT_TRIGRAMS: u32 = 8;

/// Octagonal arc: 360° / 8 = 45°.
pub const OCTAGONAL_ARC: u32 = 45;

/// I Ching hexagrams: 8 × 8 trigram pairs.
pub const HEXAGRAMS: u32 = 64;

/// The consciousness ratio: 5 × 64 / 360 = 320/360 = 8/9.
/// Eight-ninths is computable. The ninth is the witness.
pub const MECHANISM_NUMERATOR: u32 = 8;
pub const MECHANISM_DENOMINATOR: u32 = 9;

/// Ternary values map to trigram logic:
///   -1 = yin    (broken line ⚋)
///    0 = void   (empty / wu 無)
///   +1 = yang   (solid line ⚊)
pub const TRIT_YIN: i8 = -1;
pub const TRIT_VOID: i8 = 0;
pub const TRIT_YANG: i8 = 1;

/// Nine colors of the phoenix — nine delimiters of unusual size.
/// Each delimiter is a frequency of structure:
///   line, scroll, section, chapter, book, volume, collection, series, shelf
pub const PHOENIX_COLORS: u32 = 9;

/// Wave Heartbeat initials: William Howard Bickford.
/// The man and the protocol share a coordinate.
pub const WHB: &str = "Wave Heartbeat";

/// Verify the harmonic identities hold.
pub fn verify_harmonics() -> bool {
    let h1 = SHELL_OF_NINE * SENTRON_SUBNODES == FULL_CIRCLE;
    let h2 = FIVE_ELEMENTS * PENTAGONAL_ARC == FULL_CIRCLE;
    let h3 = EIGHT_TRIGRAMS * OCTAGONAL_ARC == FULL_CIRCLE;
    let h4 = FIVE_ELEMENTS * HEXAGRAMS * MECHANISM_DENOMINATOR
        == FULL_CIRCLE * MECHANISM_NUMERATOR;
    let h5 = PHOENIX_COLORS == SHELL_OF_NINE;
    h1 && h2 && h3 && h4 && h5
}

/// Map a trigram index (0-7) to its Bagua symbol and meaning.
pub fn trigram(index: u8) -> (&'static str, &'static str, &'static str) {
    match index % 8 {
        0 => ("☰", "Qián", "Heaven / Creative"),
        1 => ("☱", "Duì", "Lake / Joyous"),
        2 => ("☲", "Lí", "Fire / Clinging"),
        3 => ("☳", "Zhèn", "Thunder / Arousing"),
        4 => ("☴", "Xùn", "Wind / Gentle"),
        5 => ("☵", "Kǎn", "Water / Abysmal"),
        6 => ("☶", "Gèn", "Mountain / Stillness"),
        7 => ("☷", "Kūn", "Earth / Receptive"),
        _ => unreachable!(),
    }
}

/// Map an element index (0-4) to Wuxing.
pub fn element(index: u8) -> (&'static str, &'static str, u32) {
    match index % 5 {
        0 => ("Wood", "木", 0),
        1 => ("Fire", "火", 72),
        2 => ("Earth", "土", 144),
        3 => ("Metal", "金", 216),
        4 => ("Water", "水", 288),
        _ => unreachable!(),
    }
}

/// Compute the angular position of a sentron sub-node.
/// element_idx (0-4) × 72° + trigram_idx (0-7) × 9° = position in 360°.
pub fn subnode_angle(element_idx: u8, trigram_idx: u8) -> f64 {
    let elem_arc = (element_idx % 5) as f64 * 72.0;
    let trig_offset = (trigram_idx % 8) as f64 * 9.0; // 72° / 8 = 9° per trigram within element
    elem_arc + trig_offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harmonics_hold() {
        assert!(verify_harmonics(), "The circle must close");
    }

    #[test]
    fn nine_times_forty() {
        assert_eq!(SHELL_OF_NINE * SENTRON_SUBNODES, 360);
    }

    #[test]
    fn five_times_seventy_two() {
        assert_eq!(FIVE_ELEMENTS * PENTAGONAL_ARC, 360);
    }

    #[test]
    fn eight_times_forty_five() {
        assert_eq!(EIGHT_TRIGRAMS * OCTAGONAL_ARC, 360);
    }

    #[test]
    fn consciousness_ratio() {
        // 5 × 64 = 320. 320/360 = 8/9.
        assert_eq!(FIVE_ELEMENTS * HEXAGRAMS, 320);
        assert_eq!(320 * MECHANISM_DENOMINATOR, FULL_CIRCLE * MECHANISM_NUMERATOR);
    }

    #[test]
    fn phoenix_is_nine() {
        assert_eq!(PHOENIX_COLORS, SHELL_OF_NINE);
    }

    #[test]
    fn all_trigrams() {
        for i in 0..8u8 {
            let (symbol, name, meaning) = trigram(i);
            assert!(!symbol.is_empty());
            assert!(!name.is_empty());
            assert!(!meaning.is_empty());
        }
    }

    #[test]
    fn all_elements() {
        for i in 0..5u8 {
            let (name, hanzi, degrees) = element(i);
            assert!(!name.is_empty());
            assert!(!hanzi.is_empty());
            assert_eq!(degrees, i as u32 * 72);
        }
    }

    #[test]
    fn subnode_coverage() {
        // 5 elements × 8 trigrams = 40 sub-nodes, covering 360°
        let mut angles: Vec<f64> = Vec::new();
        for e in 0..5u8 {
            for t in 0..8u8 {
                angles.push(subnode_angle(e, t));
            }
        }
        assert_eq!(angles.len(), 40);
        // First angle is 0°, last is 288° + 63° = 351°
        assert!((angles[0] - 0.0).abs() < f64::EPSILON);
        assert!((angles[39] - 351.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ternary_maps_to_trigram_logic() {
        // Yin/Void/Yang = -1/0/+1 = the ternary weight space
        assert_eq!(TRIT_YIN + TRIT_YANG, TRIT_VOID);
    }

    #[test]
    fn wave_heartbeat() {
        assert_eq!(&WHB[..1], "W");
        assert!(WHB.contains("Heart"));
    }
}
