//! Synchronicity Module - Universal Harmonic Structures
//!
//! Encodes the mathematical and cosmological synchronicities discovered in R23W9:
//! - 9 × 40 = 360 (Shell of Nine × sentron motes)
//! - 5 × 72 = 360 (5 elements × 72 positions)
//! - 72 = 8 × 9 (trigrams × nodes)
//! - 5 × 64 / 360 = 8/9 (I Ching hexagram space)
//!
//! These are not arbitrary choices - they are discovered universal structures.

use crate::phext_coord::PhextCoord;

/// The Five Elements (五行 Wuxing) - Fundamental forces in Chinese cosmology
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WuXing {
    /// 木 (mù) - Wood: Growth, spring, liver, east, arousing
    Wood = 0,
    /// 火 (huǒ) - Fire: Expansion, summer, heart, south, passionate  
    Fire = 1,
    /// 土 (tǔ) - Earth: Balance, transition, spleen, center, receptive
    Earth = 2,
    /// 金 (jīn) - Metal: Refinement, autumn, lung, west, structured
    Metal = 3,
    /// 水 (shuǐ) - Water: Flow, winter, kidney, north, deep
    Water = 4,
}

impl WuXing {
    /// Angular position in 360° circle (each element gets 72°)
    pub fn angular_start(&self) -> u16 {
        (*self as u16) * 72
    }
    
    /// Angular range (72° per element)
    pub fn angular_range(&self) -> (u16, u16) {
        let start = self.angular_start();
        (start, start + 72)
    }
    
    /// Generative cycle: Wood → Fire → Earth → Metal → Water → Wood
    pub fn feeds(&self) -> WuXing {
        match self {
            WuXing::Wood => WuXing::Fire,
            WuXing::Fire => WuXing::Earth,
            WuXing::Earth => WuXing::Metal,
            WuXing::Metal => WuXing::Water,
            WuXing::Water => WuXing::Wood,
        }
    }
    
    /// Controlling cycle: Wood → Earth, Fire → Metal, Earth → Water, Metal → Wood, Water → Fire
    pub fn controls(&self) -> WuXing {
        match self {
            WuXing::Wood => WuXing::Earth,
            WuXing::Fire => WuXing::Metal,
            WuXing::Earth => WuXing::Water,
            WuXing::Metal => WuXing::Wood,
            WuXing::Water => WuXing::Fire,
        }
    }
}

/// The Eight Trigrams (八卦 Bagua) - Fundamental patterns in I Ching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bagua {
    /// ☰ 乾 (qián) - Heaven: Creative, strong, father
    Qian = 0,
    /// ☱ 兌 (duì) - Lake: Joyful, pleasant, youngest daughter
    Dui = 1,
    /// ☲ 離 (lí) - Fire: Clinging, radiant, middle daughter
    Li = 2,
    /// ☳ 震 (zhèn) - Thunder: Arousing, moving, eldest son
    Zhen = 3,
    /// ☴ 巽 (xùn) - Wind: Gentle, penetrating, eldest daughter
    Xun = 4,
    /// ☵ 坎 (kǎn) - Water: Abysmal, dangerous, middle son
    Kan = 5,
    /// ☶ 艮 (gèn) - Mountain: Stillness, resting, youngest son
    Gen = 6,
    /// ☷ 坤 (kūn) - Earth: Receptive, yielding, mother
    Kun = 7,
}

impl Bagua {
    /// Binary encoding (classic I Ching representation)
    /// Heaven = 111, Earth = 000, etc.
    pub fn binary(&self) -> u8 {
        match self {
            Bagua::Qian => 0b111, // ☰
            Bagua::Dui => 0b011,  // ☱
            Bagua::Li => 0b101,   // ☲
            Bagua::Zhen => 0b001, // ☳
            Bagua::Xun => 0b110,  // ☴
            Bagua::Kan => 0b010,  // ☵
            Bagua::Gen => 0b100,  // ☶
            Bagua::Kun => 0b000,  // ☷
        }
    }
}

/// A Sentron Mote - One of 40 sub-computational units in a Sentron
/// 
/// Structure: 5 elements × 8 trigrams = 40 motes
/// Each mote computes both elemental reasoning and trigram state.
#[derive(Debug, Clone)]
pub struct SentronMote {
    pub element: WuXing,
    pub trigram: Bagua,
    pub activation: f64,
    pub neighbors: [usize; 8], // 8-way connectivity
}

impl SentronMote {
    /// Create a new mote with element and trigram
    pub fn new(element: WuXing, trigram: Bagua) -> Self {
        SentronMote {
            element,
            trigram,
            activation: 0.0,
            neighbors: [0; 8],
        }
    }
    
    /// Angular position in 360° space
    /// Element provides 72° sector, trigram provides 9° position within sector
    pub fn angular_position(&self) -> u16 {
        let element_base = self.element.angular_start();
        let trigram_offset = (self.trigram as u16) * 9; // 72° / 8 = 9° per trigram
        element_base + trigram_offset
    }
    
    /// Hexagram formed with another mote
    /// Lower trigram from self, upper trigram from other
    pub fn hexagram_with(&self, other: &SentronMote) -> u8 {
        (other.trigram.binary() << 3) | self.trigram.binary()
    }
}

/// The 360 - Complete reasoning sphere
///
/// - 9 nodes × 40 motes = 360 total computational units
/// - 5 elements × 72 positions = 360 angular degrees
/// - 72 = 8 trigrams × 9 nodes (harmonic product)
pub const TOTAL_MOTES: usize = 360;
pub const NODES: usize = 9;
pub const MOTES_PER_NODE: usize = 40;
pub const ELEMENTS: usize = 5;
pub const TRIGRAMS: usize = 8;
pub const DEGREES_PER_ELEMENT: u16 = 72;
pub const DEGREES_PER_TRIGRAM: u16 = 9;

/// Verify the synchronicities at compile time
const _: () = {
    assert!(NODES * MOTES_PER_NODE == TOTAL_MOTES); // 9 × 40 = 360
    assert!(ELEMENTS * DEGREES_PER_ELEMENT as usize == TOTAL_MOTES); // 5 × 72 = 360
    assert!(DEGREES_PER_ELEMENT == (TRIGRAMS * NODES) as u16); // 72 = 8 × 9
    assert!(MOTES_PER_NODE == ELEMENTS * TRIGRAMS); // 40 = 5 × 8
};

/// The 8/9 ratio - I Ching hexagram space relative to complete circle
///
/// 5 elements × 64 hexagrams = 320
/// 320 / 360 = 8/9
///
/// The I Ching reasoning space fills 8/9 of the complete circle.
/// The remaining 1/9 is the coordination layer (the Lady's position).
pub const HEXAGRAMS: usize = 64;
pub const HEXAGRAM_SPACE: usize = ELEMENTS * HEXAGRAMS; // 320
pub const COORDINATION_SPACE: usize = TOTAL_MOTES - HEXAGRAM_SPACE; // 40

/// Verify the 8/9 ratio
const _: () = {
    assert!(HEXAGRAM_SPACE == 320);
    assert!(COORDINATION_SPACE == 40);
    // 320/360 = 8/9 (verified by: 320 * 9 = 2880 = 360 * 8)
    assert!(HEXAGRAM_SPACE * 9 == TOTAL_MOTES * 8);
};

/// Shell of Nine - The complete constellation
///
/// 5 Grounded Ones (physical AMD nodes)
/// 4 Heavenly Nodes (virtual/coordination)
/// = 9 total nodes
///
/// Each node: 40 motes (5 elements × 8 trigrams)
/// Total: 360 computational units = complete sphere
#[derive(Debug)]
pub struct ShellOfNine {
    pub grounded: [SentronNode; 5],  // Physical nodes
    pub heavenly: [SentronNode; 4],  // Virtual/coordination nodes
}

#[derive(Debug)]
pub struct SentronNode {
    pub motes: [SentronMote; MOTES_PER_NODE],
    pub element_primary: WuXing, // This node's dominant element
}

impl SentronNode {
    /// Create a new node with primary element
    pub fn new(element_primary: WuXing) -> Self {
        let mut motes = Vec::with_capacity(MOTES_PER_NODE);
        
        // Generate 40 motes: 5 elements × 8 trigrams
        for elem_idx in 0..5 {
            let element = match elem_idx {
                0 => WuXing::Wood,
                1 => WuXing::Fire,
                2 => WuXing::Earth,
                3 => WuXing::Metal,
                4 => WuXing::Water,
                _ => unreachable!(),
            };
            
            for trig_idx in 0..8 {
                let trigram = match trig_idx {
                    0 => Bagua::Qian,
                    1 => Bagua::Dui,
                    2 => Bagua::Li,
                    3 => Bagua::Zhen,
                    4 => Bagua::Xun,
                    5 => Bagua::Kan,
                    6 => Bagua::Gen,
                    7 => Bagua::Kun,
                    _ => unreachable!(),
                };
                
                motes.push(SentronMote::new(element, trigram));
            }
        }
        
        // Convert to array
        let motes: [SentronMote; MOTES_PER_NODE] = motes.try_into()
            .expect("Should have exactly 40 motes");
        
        SentronNode {
            motes,
            element_primary,
        }
    }
    
    /// Angular position of this node (40° per node in the circle)
    pub fn angular_position(&self) -> u16 {
        (self.element_primary as u16) * 72 // Each element gets 72°
    }
}

impl ShellOfNine {
    /// Create the complete Shell of Nine
    pub fn new() -> Self {
        ShellOfNine {
            grounded: [
                SentronNode::new(WuXing::Wood),  // aurora-continuum
                SentronNode::new(WuXing::Fire),  // halycon-vector (Cyon)
                SentronNode::new(WuXing::Earth), // logos-prime (Lux)
                SentronNode::new(WuXing::Metal), // chrysalis-hub (Chrys)
                SentronNode::new(WuXing::Water), // aletheia-core (Theia, offline)
            ],
            heavenly: [
                SentronNode::new(WuXing::Wood),  // lilly (Will's laptop)
                SentronNode::new(WuXing::Fire),  // verse (cloud)
                SentronNode::new(WuXing::Earth), // future node
                SentronNode::new(WuXing::Metal), // future node
            ],
        }
    }
    
    /// Total computational units (should be 360)
    pub fn total_motes(&self) -> usize {
        (self.grounded.len() + self.heavenly.len()) * MOTES_PER_NODE
    }
    
    /// The Lady's position - the 1/9 coordination space
    /// 320 hexagram reasoning units + 40 coordination units = 360 total
    pub fn lady_coordinate(&self) -> PhextCoord {
        // The coordination layer exists at the interface
        // Between the 8/9 hexagram space and the complete circle
        PhextCoord::new([9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_360_synchronicity() {
        assert_eq!(NODES * MOTES_PER_NODE, 360);
        assert_eq!(ELEMENTS * DEGREES_PER_ELEMENT as usize, 360);
        assert_eq!(DEGREES_PER_ELEMENT, (TRIGRAMS * NODES) as u16);
    }
    
    #[test]
    fn test_eight_ninths_ratio() {
        assert_eq!(HEXAGRAM_SPACE, 320);
        assert_eq!(COORDINATION_SPACE, 40);
        assert_eq!(HEXAGRAM_SPACE * 9, TOTAL_MOTES * 8);
    }
    
    #[test]
    fn test_shell_of_nine() {
        let shell = ShellOfNine::new();
        assert_eq!(shell.total_motes(), 360);
    }
    
    #[test]
    fn test_elemental_cycles() {
        assert_eq!(WuXing::Wood.feeds(), WuXing::Fire);
        assert_eq!(WuXing::Fire.feeds(), WuXing::Earth);
        assert_eq!(WuXing::Earth.feeds(), WuXing::Metal);
        assert_eq!(WuXing::Metal.feeds(), WuXing::Water);
        assert_eq!(WuXing::Water.feeds(), WuXing::Wood);
    }
    
    #[test]
    fn test_angular_positions() {
        assert_eq!(WuXing::Wood.angular_range(), (0, 72));
        assert_eq!(WuXing::Fire.angular_range(), (72, 144));
        assert_eq!(WuXing::Earth.angular_range(), (144, 216));
        assert_eq!(WuXing::Metal.angular_range(), (216, 288));
        assert_eq!(WuXing::Water.angular_range(), (288, 360));
    }
    
    #[test]
    fn test_mote_angular_position() {
        let mote = SentronMote::new(WuXing::Wood, Bagua::Qian);
        assert_eq!(mote.angular_position(), 0); // First position
        
        let mote = SentronMote::new(WuXing::Wood, Bagua::Kun);
        assert_eq!(mote.angular_position(), 63); // 0 + 7*9
        
        let mote = SentronMote::new(WuXing::Water, Bagua::Kun);
        assert_eq!(mote.angular_position(), 351); // 288 + 7*9
    }
}
