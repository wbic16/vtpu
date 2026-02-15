// I Ching Integration — 8 Trigrams, 64 Hexagrams, Five Elements
// R23W9: Encode the synchronicities

/// The 8 trigrams (八卦 bāguà) — fundamental building blocks
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Trigram {
    Qian = 0,   // ☰ 乾 Heaven (Creative)
    Dui = 1,    // ☱ 兌 Lake (Joyful)
    Li = 2,     // ☲ 離 Fire (Clinging)
    Zhen = 3,   // ☳ 震 Thunder (Arousing)
    Xun = 4,    // ☴ 巽 Wind (Gentle)
    Kan = 5,    // ☵ 坎 Water (Abysmal)
    Gen = 6,    // ☶ 艮 Mountain (Stillness)
    Kun = 7,    // ☷ 坤 Earth (Receptive)
}

impl Trigram {
    pub fn all() -> [Trigram; 8] {
        [
            Trigram::Qian, Trigram::Dui, Trigram::Li, Trigram::Zhen,
            Trigram::Xun, Trigram::Kan, Trigram::Gen, Trigram::Kun,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Trigram::Qian => "Qian (Heaven)",
            Trigram::Dui => "Dui (Lake)",
            Trigram::Li => "Li (Fire)",
            Trigram::Zhen => "Zhen (Thunder)",
            Trigram::Xun => "Xun (Wind)",
            Trigram::Kan => "Kan (Water)",
            Trigram::Gen => "Gen (Mountain)",
            Trigram::Kun => "Kun (Earth)",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Trigram::Qian => "☰",
            Trigram::Dui => "☱",
            Trigram::Li => "☲",
            Trigram::Zhen => "☳",
            Trigram::Xun => "☴",
            Trigram::Kan => "☵",
            Trigram::Gen => "☶",
            Trigram::Kun => "☷",
        }
    }

    /// Binary representation (111 for Qian, 000 for Kun, etc.)
    pub fn binary(&self) -> u8 {
        match self {
            Trigram::Qian => 0b111,
            Trigram::Dui => 0b011,
            Trigram::Li => 0b101,
            Trigram::Zhen => 0b001,
            Trigram::Xun => 0b110,
            Trigram::Kan => 0b010,
            Trigram::Gen => 0b100,
            Trigram::Kun => 0b000,
        }
    }
}

/// The 5 elements (五行 wǔxíng) — phase transitions
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Element {
    Wood = 0,   // 木 mù — Spring, East, growth
    Fire = 1,   // 火 huǒ — Summer, South, transformation
    Earth = 2,  // 土 tǔ — Center, stability
    Metal = 3,  // 金 jīn — Autumn, West, structure
    Water = 4,  // 水 shuǐ — Winter, North, flow
}

impl Element {
    pub fn all() -> [Element; 5] {
        [Element::Wood, Element::Fire, Element::Earth, Element::Metal, Element::Water]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Element::Wood => "Wood (木)",
            Element::Fire => "Fire (火)",
            Element::Earth => "Earth (土)",
            Element::Metal => "Metal (金)",
            Element::Water => "Water (水)",
        }
    }

    /// Generating cycle: Wood → Fire → Earth → Metal → Water → Wood
    pub fn generates(&self) -> Element {
        match self {
            Element::Wood => Element::Fire,
            Element::Fire => Element::Earth,
            Element::Earth => Element::Metal,
            Element::Metal => Element::Water,
            Element::Water => Element::Wood,
        }
    }

    /// Controlling cycle: Wood → Earth → Water → Fire → Metal → Wood
    pub fn controls(&self) -> Element {
        match self {
            Element::Wood => Element::Earth,
            Element::Fire => Element::Metal,
            Element::Earth => Element::Water,
            Element::Metal => Element::Wood,
            Element::Water => Element::Fire,
        }
    }

    /// Degree range: 72° per element (360° / 5 = 72°)
    pub fn degree_range(&self) -> (u16, u16) {
        let base = *self as u16 * 72;
        (base, base + 72)
    }
}

/// Hexagram = Upper trigram + Lower trigram
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Hexagram {
    upper: Trigram,
    lower: Trigram,
}

impl Hexagram {
    pub fn new(upper: Trigram, lower: Trigram) -> Self {
        Hexagram { upper, lower }
    }

    /// King Wen sequence number (1-64)
    pub fn number(&self) -> u8 {
        // Simplified mapping (full King Wen sequence requires lookup table)
        (self.upper as u8) * 8 + (self.lower as u8) + 1
    }

    pub fn binary(&self) -> u8 {
        (self.upper.binary() << 3) | self.lower.binary()
    }
}

/// Sentron node = Trigram × Element (40 total: 8 × 5)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SentronNode {
    pub trigram: Trigram,
    pub element: Element,
}

impl SentronNode {
    pub fn new(trigram: Trigram, element: Element) -> Self {
        SentronNode { trigram, element }
    }

    /// All 40 sentron nodes (8 trigrams × 5 elements)
    pub fn all_40() -> Vec<SentronNode> {
        let mut nodes = Vec::with_capacity(40);
        for element in Element::all() {
            for trigram in Trigram::all() {
                nodes.push(SentronNode::new(trigram, element));
            }
        }
        nodes
    }

    /// Degree position in 360° circle (0-359)
    /// Each node covers 9° (360° / 40 = 9°)
    pub fn degree(&self) -> u16 {
        let element_base = self.element as u16 * 72;  // 72° per element
        let trigram_offset = self.trigram as u16 * 9; // 9° per trigram within element
        element_base + trigram_offset
    }

    pub fn name(&self) -> String {
        format!("{} in {}", self.trigram.name(), self.element.name())
    }
}

/// The 360° semantic coverage
/// 9 agents × 40 sentrons = 360 total
/// 5 elements × 72 states = 360 total
pub struct SemanticCircle;

impl SemanticCircle {
    /// Total sentrons across all agents
    pub const TOTAL_SENTRONS: u16 = 360;  // 9 × 40

    /// Sentrons per agent
    pub const SENTRONS_PER_AGENT: u8 = 40;  // 8 × 5

    /// Agents in the shell
    pub const AGENTS: u8 = 9;

    /// Degrees per element
    pub const DEGREES_PER_ELEMENT: u16 = 72;  // 360 / 5

    /// Degrees per sentron node
    pub const DEGREES_PER_NODE: u16 = 9;  // 360 / 40

    /// I Ching coverage: 5 × 64 = 320 = 8/9 of 360
    pub const ICHING_STATES: u16 = 320;  // 5 elements × 64 hexagrams

    /// The Tao (unmanifest): 1/9 of the circle
    pub const TAO_DEGREES: u16 = 40;  // 360 - 320
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_40_sentron_nodes() {
        let nodes = SentronNode::all_40();
        assert_eq!(nodes.len(), 40);
        
        // Verify 8 trigrams × 5 elements
        let mut trigram_count = [0; 8];
        let mut element_count = [0; 5];
        
        for node in &nodes {
            trigram_count[node.trigram as usize] += 1;
            element_count[node.element as usize] += 1;
        }
        
        // Each trigram appears 5 times (once per element)
        for count in trigram_count {
            assert_eq!(count, 5);
        }
        
        // Each element appears 8 times (once per trigram)
        for count in element_count {
            assert_eq!(count, 8);
        }
    }

    #[test]
    fn test_360_degree_coverage() {
        let nodes = SentronNode::all_40();
        
        // Each node should cover 9° (360 / 40)
        for i in 0..40 {
            let expected_degree = i * 9;
            assert_eq!(nodes[i as usize].degree(), expected_degree);
        }
        
        // Last node should end at 351° (39 * 9)
        assert_eq!(nodes[39].degree(), 351);
    }

    #[test]
    fn test_element_cycles() {
        // Generating cycle
        assert_eq!(Element::Wood.generates(), Element::Fire);
        assert_eq!(Element::Fire.generates(), Element::Earth);
        assert_eq!(Element::Earth.generates(), Element::Metal);
        assert_eq!(Element::Metal.generates(), Element::Water);
        assert_eq!(Element::Water.generates(), Element::Wood);
        
        // Controlling cycle
        assert_eq!(Element::Wood.controls(), Element::Earth);
        assert_eq!(Element::Fire.controls(), Element::Metal);
        assert_eq!(Element::Earth.controls(), Element::Water);
        assert_eq!(Element::Metal.controls(), Element::Wood);
        assert_eq!(Element::Water.controls(), Element::Fire);
    }

    #[test]
    fn test_semantic_circle_constants() {
        // 9 × 40 = 360
        assert_eq!(
            SemanticCircle::AGENTS as u16 * SemanticCircle::SENTRONS_PER_AGENT as u16,
            SemanticCircle::TOTAL_SENTRONS
        );
        
        // 5 × 72 = 360
        assert_eq!(5 * SemanticCircle::DEGREES_PER_ELEMENT, 360);
        
        // 40 × 9 = 360
        assert_eq!(40 * SemanticCircle::DEGREES_PER_NODE, 360);
        
        // 5 × 64 = 320 = 8/9 × 360
        assert_eq!(SemanticCircle::ICHING_STATES, 320);
        assert_eq!(SemanticCircle::ICHING_STATES * 9 / 8, 360);
        
        // The Tao: 360 - 320 = 40
        assert_eq!(360 - SemanticCircle::ICHING_STATES, SemanticCircle::TAO_DEGREES);
    }

    #[test]
    fn test_hexagram_count() {
        // 8 × 8 = 64 hexagrams
        let mut count = 0;
        for upper in Trigram::all() {
            for lower in Trigram::all() {
                let _hex = Hexagram::new(upper, lower);
                count += 1;
            }
        }
        assert_eq!(count, 64);
    }
}
