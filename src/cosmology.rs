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
    fn all_nodes_have_elements() {
        for i in 0..9 {
            let _ = Element::for_node(i); // should not panic
        }
    }
}
