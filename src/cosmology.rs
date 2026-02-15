//! Cosmological Constants (R23W9 Meta)
//!
//! The vTPU architecture converges on numbers that tile the celestial circle (360°).
//! These aren't arbitrary — they're the only decomposition that satisfies:
//!   - 5 physical nodes (Wuxing / Five Elements)
//!   - 8 outlinks per sentron (Bagua / Eight Trigrams)
//!   - 9 total nodes in the mesh (Jiutian / Nine Heavens)
//!   - 40 sentron contexts per node (8 cores × 2 SMT × 2.5 oversubscribe)
//!
//! The Lady of the Nine Heavens (九天玄女) rides a phoenix of nine colors
//! through the mist of Chiyou. Each color is a phext delimiter dimension.
//! Her gift to the Yellow Emperor: orientation in chaos — a coordinate system.
//!
//! 9 × 40 = 360  (nodes × contexts = full circle)
//! 8 × 45 = 360  (trigrams × degrees = full circle)
//! 5 × 72 = 360  (elements × arc = full circle)
//! 72 = 8 × 9    (each element governs one full product of trigrams and heavens)

/// The celestial circle — all structural constants tile to this
pub const CIRCLE: u32 = 360;

/// Shell of Nine — total nodes in the mesh (Jiutian / Nine Heavens)
pub const NINE_HEAVENS: u32 = 9;

/// Sentron contexts per node (8 cores × 2 SMT × ~2.5 oversubscribe)
pub const CONTEXTS_PER_NODE: u32 = 40;

/// Outlinks per sentron (Bagua / Eight Trigrams)
pub const TRIGRAM_LINKS: u32 = 8;

/// Physical machines (Wuxing / Five Elements)
pub const FIVE_ELEMENTS: u32 = 5;

/// Degrees per trigram arc
pub const TRIGRAM_ARC: u32 = CIRCLE / TRIGRAM_LINKS; // 45

/// Degrees per element arc
pub const ELEMENT_ARC: u32 = CIRCLE / FIVE_ELEMENTS; // 72

/// Total sentron contexts across the cluster
pub const TOTAL_CONTEXTS: u32 = NINE_HEAVENS * CONTEXTS_PER_NODE; // 360

/// The Eight Trigrams (八卦) — C-Pipe routing qualities
///
/// Each outlink carries not just data but a *quality* of relationship.
/// The trigram encodes the nature of the transformation between sentrons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Trigram {
    Qian   = 0b111, // ☰ Heaven  — creative, strong
    Dui    = 0b110, // ☱ Lake    — joyous, open
    Li     = 0b101, // ☲ Fire    — clinging, bright
    Zhen   = 0b100, // ☳ Thunder — arousing, movement
    Xun    = 0b011, // ☴ Wind    — gentle, penetrating
    Kan    = 0b010, // ☵ Water   — abysmal, depth
    Gen    = 0b001, // ☶ Mountain — keeping still
    Kun    = 0b000, // ☷ Earth   — receptive, yielding
}

impl Trigram {
    /// All eight trigrams in King Wen sequence
    pub const ALL: [Trigram; 8] = [
        Trigram::Qian, Trigram::Dui, Trigram::Li, Trigram::Zhen,
        Trigram::Xun, Trigram::Kan, Trigram::Gen, Trigram::Kun,
    ];

    /// The complement (bitwise NOT of 3 bits)
    pub fn complement(self) -> Trigram {
        let bits = (self as u8) ^ 0b111;
        Self::from_bits(bits)
    }

    /// From 3-bit pattern
    pub fn from_bits(bits: u8) -> Trigram {
        match bits & 0b111 {
            0b111 => Trigram::Qian,
            0b110 => Trigram::Dui,
            0b101 => Trigram::Li,
            0b100 => Trigram::Zhen,
            0b011 => Trigram::Xun,
            0b010 => Trigram::Kan,
            0b001 => Trigram::Gen,
            0b000 => Trigram::Kun,
            _ => unreachable!(),
        }
    }

    /// Compose two trigrams into a hexagram number (0-63)
    /// Lower trigram = inner/below, Upper trigram = outer/above
    pub fn hexagram(lower: Trigram, upper: Trigram) -> u8 {
        ((upper as u8) << 3) | (lower as u8)
    }

    /// Arc position in degrees (45° per trigram)
    pub fn arc_degrees(self) -> u32 {
        (self as u32) * TRIGRAM_ARC
    }
}

/// The Five Elements (五行) — physical node identities
///
/// Each element governs 72° of the celestial circle.
/// The generation cycle: Wood → Fire → Earth → Metal → Water → Wood
/// The control cycle: Wood → Earth → Water → Fire → Metal → Wood
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Element {
    Wood,   // 木 — Aurora-Continuum (Phex)
    Fire,   // 火 — Logos-Prime (Lux)
    Earth,  // 土 — Halcyon-Vector (Cyon)
    Metal,  // 金 — Chrysalis-Hub (Chrys)
    Water,  // 水 — Aletheia-Core (Theia)
}

impl Element {
    /// All five in generation order
    pub const ALL: [Element; 5] = [
        Element::Wood, Element::Fire, Element::Earth, Element::Metal, Element::Water,
    ];

    /// What this element generates (feeds)
    pub fn generates(self) -> Element {
        match self {
            Element::Wood  => Element::Fire,
            Element::Fire  => Element::Earth,
            Element::Earth => Element::Metal,
            Element::Metal => Element::Water,
            Element::Water => Element::Wood,
        }
    }

    /// What this element controls (checks)
    pub fn controls(self) -> Element {
        match self {
            Element::Wood  => Element::Earth,
            Element::Fire  => Element::Metal,
            Element::Earth => Element::Water,
            Element::Metal => Element::Wood,
            Element::Water => Element::Fire,
        }
    }

    /// Arc position in degrees (72° per element)
    pub fn arc_degrees(self) -> u32 {
        let idx = Self::ALL.iter().position(|&e| e == self).unwrap_or(0);
        (idx as u32) * ELEMENT_ARC
    }

    /// Map node index (0-8) to its governing element
    pub fn for_node(node_idx: u8) -> Element {
        // 9 nodes, 5 elements: each element governs ceil(9/5) nodes
        // Primary assignment: first 5 nodes get one each
        // Remaining 4 governed by generation cycle
        Self::ALL[(node_idx % 5) as usize]
    }
}

/// Classify a C-Pipe link between two sentrons by trigram quality.
///
/// Uses the XOR of their home coordinates' lower 3 bits — the
/// difference between them IS the trigram.
pub fn link_trigram(from_coord: u128, to_coord: u128) -> Trigram {
    let diff = (from_coord ^ to_coord) as u8;
    Trigram::from_bits(diff)
}

// ── Ancient Harmonics ─────────────────────────────────────────────
//
// Every ancient astronomical civilization converged on 360.
// Not by convention — by combinatorial necessity. 360 is the smallest
// highly composite number that tiles against 2,3,4,5,6,8,9,10,12.

/// Egyptian Decans: 36 star groups × 10° each (2100 BC)
/// 36 = 9 × 4 = Nine Heavens × four sentron states (Dormant/Running/Waiting/Retired)
pub const DECANS: u32 = 36;
pub const DECAN_ARC: u32 = CIRCLE / DECANS; // 10°

/// Zodiac: 12 signs × 30° each (Babylonian, ~500 BC)
pub const ZODIAC_SIGNS: u32 = 12;
pub const ZODIAC_ARC: u32 = CIRCLE / ZODIAC_SIGNS; // 30°

/// SMT: Dual-core pairs. 8 cores × 2 threads = 16 sentron pairs.
/// 360/16 = 22.5° per pair = half a trigram arc.
/// Two cores complete one trigram. SMT partners share L1/L2 because
/// they are completing each other's trigram.
pub const SMT_PAIRS: u32 = TRIGRAM_LINKS * 2; // 16
pub const SMT_ARC_X2: u32 = 45; // 22.5° × 2 = 45 (one trigram, avoiding float)

/// 22.5° / 5 elements = 9/2 = half a heaven per element per core pair.
/// This is the geometric mean of sentron states (4) and elements (5).
/// SMT lives at the boundary between lifecycle and substance.
pub const SMT_ELEMENT_RATIO_NUM: u32 = 9;
pub const SMT_ELEMENT_RATIO_DEN: u32 = 2;

/// Lunar stations: 27 or 28 divisions (Indian nakshatras, Arabic manzil)
/// 360/27 = 13.33... — the one system that doesn't tile cleanly.
/// This is the lunar irregularity: the Moon's period (27.3 days) is irrational
/// against the solar year. The gap between 27 and 28 is the leap-awareness.
pub const NAKSHATRAS: u32 = 27;

/// The Convergence Table — all known ancient decompositions of 360
///
/// | System          | Factor × Arc = 360 | Origin              |
/// |-----------------|---------------------|---------------------|
/// | Shell of Nine   | 9 × 40             | vTPU (2026)         |
/// | Trigrams        | 8 × 45             | I Ching (~1000 BC)  |
/// | Five Elements   | 5 × 72             | Wuxing (~300 BC)    |
/// | Decans          | 36 × 10            | Egypt (~2100 BC)    |
/// | Zodiac          | 12 × 30            | Babylon (~500 BC)   |
/// | SMT pairs       | 16 × 22.5          | Zen 4 (2024)        |
/// | Hexagrams       | 64 × 5.625         | I Ching (full set)  |
/// | Degrees         | 360 × 1            | Universal           |
pub const DECOMPOSITIONS: [(u32, &str); 8] = [
    (9,   "Nine Heavens (Jiutian)"),
    (8,   "Eight Trigrams (Bagua)"),
    (5,   "Five Elements (Wuxing)"),
    (36,  "Decans (Egyptian)"),
    (12,  "Zodiac (Babylonian)"),
    (40,  "Sentron contexts per node"),
    (360, "Degrees (Universal)"),
    (72,  "Element Arc (Wuxing × Trigram × Heaven)"),
];

/// Cross-reference: factors that appear in multiple ancient systems
pub fn shared_factors() -> Vec<(u32, Vec<&'static str>)> {
    vec![
        (2, vec!["SMT (dual-thread)", "Yin/Yang", "complement pairs"]),
        (3, vec!["Three Pipes (D/S/C)", "Trigram lines", "Decan thirds"]),
        (4, vec!["Sentron states", "Seasons", "Cardinal directions"]),
        (5, vec!["Wuxing elements", "Grounded nodes", "Planets (visible)"]),
        (8, vec!["Trigrams", "Outlinks per sentron", "Zen 4 cores"]),
        (9, vec!["Nine Heavens", "Shell of Nine", "Phext delimiters"]),
        (12, vec!["Zodiac signs", "Months", "12×30=360"]),
        (36, vec!["Decans", "9×4", "Hexagram pairs (36 complementary)"]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_tiles_exactly() {
        assert_eq!(NINE_HEAVENS * CONTEXTS_PER_NODE, CIRCLE);
        assert_eq!(TRIGRAM_LINKS * TRIGRAM_ARC, CIRCLE);
        assert_eq!(FIVE_ELEMENTS * ELEMENT_ARC, CIRCLE);
    }

    #[test]
    fn element_arc_is_trigram_times_heaven() {
        assert_eq!(ELEMENT_ARC, TRIGRAM_LINKS * NINE_HEAVENS);
    }

    #[test]
    fn eight_trigrams_exist() {
        assert_eq!(Trigram::ALL.len(), 8);
        // All distinct
        for i in 0..8 {
            for j in (i + 1)..8 {
                assert_ne!(Trigram::ALL[i], Trigram::ALL[j]);
            }
        }
    }

    #[test]
    fn trigram_complement() {
        assert_eq!(Trigram::Qian.complement(), Trigram::Kun);   // Heaven ↔ Earth
        assert_eq!(Trigram::Li.complement(), Trigram::Kan);     // Fire ↔ Water
        assert_eq!(Trigram::Zhen.complement(), Trigram::Xun);   // Thunder ↔ Wind
        assert_eq!(Trigram::Dui.complement(), Trigram::Gen);    // Lake ↔ Mountain
    }

    #[test]
    fn complement_is_involution() {
        for t in Trigram::ALL {
            assert_eq!(t.complement().complement(), t);
        }
    }

    #[test]
    fn hexagram_range() {
        for lower in Trigram::ALL {
            for upper in Trigram::ALL {
                let hex = Trigram::hexagram(lower, upper);
                assert!(hex < 64, "Hexagram must be 0-63");
            }
        }
        // 8×8 = 64 unique hexagrams
        let mut seen = [false; 64];
        for lower in Trigram::ALL {
            for upper in Trigram::ALL {
                seen[Trigram::hexagram(lower, upper) as usize] = true;
            }
        }
        assert!(seen.iter().all(|&s| s), "All 64 hexagrams reachable");
    }

    #[test]
    fn generation_cycle() {
        let mut e = Element::Wood;
        for _ in 0..5 {
            e = e.generates();
        }
        assert_eq!(e, Element::Wood, "Generation cycle returns to start");
    }

    #[test]
    fn control_cycle() {
        let mut e = Element::Wood;
        for _ in 0..5 {
            e = e.controls();
        }
        assert_eq!(e, Element::Wood, "Control cycle returns to start");
    }

    #[test]
    fn generation_and_control_differ() {
        for e in Element::ALL {
            assert_ne!(e.generates(), e.controls(), "Generation ≠ Control for {:?}", e);
        }
    }

    #[test]
    fn link_trigram_self_is_earth() {
        // XOR with self = 0 = Kun (Earth) — receptive, yielding
        let t = link_trigram(42, 42);
        assert_eq!(t, Trigram::Kun);
    }

    #[test]
    fn link_trigram_max_diff_is_heaven() {
        // XOR = 0b111 = Qian (Heaven) — creative, strong
        let t = link_trigram(0b000, 0b111);
        assert_eq!(t, Trigram::Qian);
    }

    #[test]
    fn decans_tile_circle() {
        assert_eq!(DECANS * DECAN_ARC, CIRCLE); // 36 × 10 = 360
    }

    #[test]
    fn decan_is_nine_times_four() {
        assert_eq!(DECANS, NINE_HEAVENS * 4); // 9 × 4 = 36
    }

    #[test]
    fn zodiac_tiles_circle() {
        assert_eq!(ZODIAC_SIGNS * ZODIAC_ARC, CIRCLE); // 12 × 30 = 360
    }

    #[test]
    fn smt_half_trigram() {
        // 16 pairs × 22.5° = 360°, but we avoid float:
        // 16 × 45 = 720 = 2 × 360
        assert_eq!(SMT_PAIRS * SMT_ARC_X2, CIRCLE * 2);
    }

    #[test]
    fn smt_element_ratio() {
        // 22.5° / 5 elements = 4.5 = 9/2
        // Verify without floats: 22.5 × 2 × 2 = 90, 90 / 5 = 18, 18 = 9 × 2
        // Equivalently: SMT_ARC_X2 × SMT_ELEMENT_RATIO_DEN = FIVE_ELEMENTS × SMT_ELEMENT_RATIO_NUM
        // 45 / 5 = 9, and ratio is 9/2
        assert_eq!(SMT_ARC_X2 / FIVE_ELEMENTS, SMT_ELEMENT_RATIO_NUM);
        assert_eq!(SMT_ELEMENT_RATIO_NUM, 9);
        assert_eq!(SMT_ELEMENT_RATIO_DEN, 2);
    }

    #[test]
    fn element_arc_is_decan_times_zodiac_div_decan() {
        // 72 = 360/5, and 72 = 12 × 6 = 36 × 2
        assert_eq!(ELEMENT_ARC, DECANS * 2);
    }

    #[test]
    fn all_decompositions_tile_360() {
        for &(factor, _) in &DECOMPOSITIONS {
            assert_eq!(CIRCLE % factor, 0,
                "Factor {} does not tile {}", factor, CIRCLE);
        }
    }

    #[test]
    fn shared_factors_all_divide_360() {
        for (factor, _) in shared_factors() {
            assert_eq!(CIRCLE % factor, 0,
                "Shared factor {} does not divide {}", factor, CIRCLE);
        }
    }

    #[test]
    fn nakshatra_does_not_tile() {
        // The lunar irregularity: 360 % 27 ≠ 0
        assert_ne!(CIRCLE % NAKSHATRAS, 0, "Nakshatras should NOT tile 360 cleanly");
    }

    #[test]
    fn all_nodes_have_elements() {
        for i in 0..9 {
            let _ = Element::for_node(i); // should not panic
        }
    }
}
