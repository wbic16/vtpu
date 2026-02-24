//! Twisted Pairs — Ba Gua topology for sentron fleets
//!
//! Ring topology is naive. Real sentrons wire as twisted pairs:
//! 8 links per sentron (4 trigram pairs × 2 strands) + 1 main line = 9 connections.
//!
//! The 8 trigrams of the Ba Gua map to 4 complementary pairs:
//!   ☰/☷ (Heaven/Earth)  — global coordination
//!   ☲/☵ (Fire/Water)    — D-pipe / S-pipe oscillation
//!   ☳/☴ (Thunder/Wind)  — excitation / diffusion
//!   ☶/☱ (Mountain/Lake) — accumulation / release
//!
//! Each pair carries bidirectional data (twisted = both directions on same link).
//! Main line connects to the color group coordinator (1 per 40 sentrons).
//!
//! R23W29 — Theia 💎

use crate::sentron::NeuronWiring;

/// The 8 trigrams — Ba Gua
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Trigram {
    Qian  = 0, // ☰ Heaven (111) — creative, strong
    Kun   = 1, // ☷ Earth  (000) — receptive, yielding
    Li    = 2, // ☲ Fire   (101) — clinging, bright
    Kan   = 3, // ☵ Water  (010) — abysmal, deep
    Zhen  = 4, // ☳ Thunder(100) — arousing, shock
    Xun   = 5, // ☴ Wind   (011) — gentle, penetrating
    Gen   = 6, // ☶ Mountain(001)— keeping still
    Dui   = 7, // ☱ Lake   (110) — joyous, open
}

impl Trigram {
    /// Binary value of the trigram (3 bits: bottom, middle, top)
    pub fn bits(&self) -> u8 {
        match self {
            Trigram::Qian => 0b111,
            Trigram::Kun  => 0b000,
            Trigram::Li   => 0b101,
            Trigram::Kan  => 0b010,
            Trigram::Zhen => 0b100,
            Trigram::Xun  => 0b011,
            Trigram::Gen  => 0b001,
            Trigram::Dui  => 0b110,
        }
    }

    /// Complement: flip all bits (the paired trigram)
    pub fn complement(&self) -> Self {
        match self {
            Trigram::Qian => Trigram::Kun,
            Trigram::Kun  => Trigram::Qian,
            Trigram::Li   => Trigram::Kan,
            Trigram::Kan  => Trigram::Li,
            Trigram::Zhen => Trigram::Xun,
            Trigram::Xun  => Trigram::Zhen,
            Trigram::Gen  => Trigram::Dui,
            Trigram::Dui  => Trigram::Gen,
        }
    }

    /// Ternary weight mapping: -1=yin(0), 0=void, +1=yang(1)
    pub fn ternary_weights(&self) -> [i8; 3] {
        let b = self.bits();
        [
            if b & 1 != 0 { 1 } else { -1 },
            if b & 2 != 0 { 1 } else { -1 },
            if b & 4 != 0 { 1 } else { -1 },
        ]
    }
}

/// The 4 complementary pairs
pub const PAIRS: [(Trigram, Trigram); 4] = [
    (Trigram::Qian, Trigram::Kun),   // Heaven/Earth
    (Trigram::Li,   Trigram::Kan),   // Fire/Water
    (Trigram::Zhen, Trigram::Xun),   // Thunder/Wind
    (Trigram::Gen,  Trigram::Dui),   // Mountain/Lake
];

/// The 5 Wuxing phases — generative cycle order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Wuxing {
    Wood  = 0, // generates Fire
    Fire  = 1, // generates Earth
    Earth = 2, // generates Metal
    Metal = 3, // generates Water
    Water = 4, // generates Wood (cycle completes)
}

impl Wuxing {
    /// Next in the generative (Sheng) cycle
    pub fn generates(&self) -> Self {
        match self {
            Wuxing::Wood  => Wuxing::Fire,
            Wuxing::Fire  => Wuxing::Earth,
            Wuxing::Earth => Wuxing::Metal,
            Wuxing::Metal => Wuxing::Water,
            Wuxing::Water => Wuxing::Wood,
        }
    }

    /// Overcoming (Ke) cycle — what this element controls
    pub fn overcomes(&self) -> Self {
        match self {
            Wuxing::Wood  => Wuxing::Earth,
            Wuxing::Fire  => Wuxing::Metal,
            Wuxing::Earth => Wuxing::Water,
            Wuxing::Metal => Wuxing::Wood,
            Wuxing::Water => Wuxing::Fire,
        }
    }
}

/// Compute twisted-pair wiring for a sentron in a color group of 40.
///
/// Layout: 5 Wuxing phases × 8 Ba Gua positions = 40 sentrons per color.
/// Sentron `id` within color: phase = id / 8, trigram = id % 8.
///
/// Upstream (4 links): same-phase complement + cross-phase generative sources
/// Downstream (4 links): same-phase complement + cross-phase generative sinks
///
/// This replaces the naive ring with a topology that mirrors
/// both the trigram pair structure and the Wuxing generative cycle.
pub fn twisted_pair_wiring(id_in_color: u16, color_offset: u16) -> NeuronWiring {
    let local = id_in_color % 40;
    let phase = local / 8;           // 0..4 (Wuxing)
    let trigram = local % 8;         // 0..7 (Ba Gua)
    let n = 40u16;

    // Complement trigram in same phase
    let complement = match trigram {
        0 => 1, 1 => 0, // Qian/Kun
        2 => 3, 3 => 2, // Li/Kan
        4 => 5, 5 => 4, // Zhen/Xun
        6 => 7, 7 => 6, // Gen/Dui
        _ => 0,
    };
    let same_phase_complement = phase * 8 + complement;

    // Previous phase (generative source): same trigram position
    let prev_phase = (phase + 4) % 5; // modular -1
    let gen_source = prev_phase * 8 + trigram;

    // Next phase (generative sink): same trigram position
    let next_phase = (phase + 1) % 5;
    let gen_sink = next_phase * 8 + trigram;

    // Overcome target: skip-one phase, complement position
    let overcome_phase = (phase + 2) % 5;
    let overcome_target = overcome_phase * 8 + complement;

    // Overcome source: who overcomes us? skip-one backward
    let overcome_source_phase = (phase + 3) % 5;
    let overcome_source = overcome_source_phase * 8 + complement;

    let mut w = NeuronWiring::new();
    w.upstream = [
        color_offset + same_phase_complement,
        color_offset + gen_source,
        color_offset + overcome_source,
        color_offset + ((local + n - 1) % n), // ring fallback for locality
    ];
    w.downstream = [
        color_offset + same_phase_complement,
        color_offset + gen_sink,
        color_offset + overcome_target,
        color_offset + ((local + 1) % n), // ring fallback
    ];
    w
}

/// Wire a full fleet of 360 sentrons with twisted-pair topology.
/// 9 colors × 40 sentrons each. Inter-color links via main line (color coordinator).
pub fn wire_fleet_twisted(size: usize) -> Vec<NeuronWiring> {
    let mut wiring = Vec::with_capacity(size);
    for i in 0..size {
        let color = (i / 40) as u16;
        let color_offset = color * 40;
        let id_in_color = (i % 40) as u16;
        wiring.push(twisted_pair_wiring(id_in_color, color_offset));
    }
    wiring
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigram_complement_involution() {
        // complement(complement(x)) == x
        for t in [Trigram::Qian, Trigram::Kun, Trigram::Li, Trigram::Kan,
                   Trigram::Zhen, Trigram::Xun, Trigram::Gen, Trigram::Dui] {
            assert_eq!(t.complement().complement(), t);
        }
    }

    #[test]
    fn trigram_bits_complement_xor() {
        // complement bits = XOR 0b111
        for t in [Trigram::Qian, Trigram::Kun, Trigram::Li, Trigram::Kan,
                   Trigram::Zhen, Trigram::Xun, Trigram::Gen, Trigram::Dui] {
            assert_eq!(t.bits() ^ t.complement().bits(), 0b111);
        }
    }

    #[test]
    fn wuxing_generative_cycle() {
        let mut phase = Wuxing::Wood;
        let mut visited = vec![];
        for _ in 0..5 {
            visited.push(phase);
            phase = phase.generates();
        }
        assert_eq!(visited.len(), 5);
        assert_eq!(phase, Wuxing::Wood); // cycle completes
    }

    #[test]
    fn wuxing_overcomes_cycle() {
        let mut phase = Wuxing::Wood;
        let mut visited = vec![];
        for _ in 0..5 {
            visited.push(phase);
            phase = phase.overcomes();
        }
        assert_eq!(phase, Wuxing::Wood); // also a 5-cycle
    }

    #[test]
    fn twisted_pair_self_not_neighbor() {
        let w = twisted_pair_wiring(0, 0);
        // Sentron 0 should not appear in its own wiring
        assert!(!w.upstream.contains(&0));
        assert!(!w.downstream.contains(&0));
    }

    #[test]
    fn twisted_pair_has_complement() {
        // Sentron 0 (Qian, phase 0) should connect to sentron 1 (Kun, phase 0)
        let w = twisted_pair_wiring(0, 0);
        assert!(w.upstream.contains(&1) || w.downstream.contains(&1));
    }

    #[test]
    fn twisted_pair_cross_phase() {
        // Sentron 0 (Wood/Qian) should connect to phase 4 (Water) upstream
        let w = twisted_pair_wiring(0, 0);
        // gen_source = phase 4, trigram 0 = sentron 32
        assert!(w.upstream.contains(&32));
    }

    #[test]
    fn fleet_wiring_360() {
        let wiring = wire_fleet_twisted(360);
        assert_eq!(wiring.len(), 360);

        // Every sentron has 8 links (4 up + 4 down)
        for w in &wiring {
            assert_eq!(w.upstream.len(), 4);
            assert_eq!(w.downstream.len(), 4);
        }
    }

    #[test]
    fn fleet_wiring_stays_in_color() {
        let wiring = wire_fleet_twisted(360);
        // Sentron 0 (color 0) should only connect to 0..39
        for &u in &wiring[0].upstream {
            assert!(u < 40, "upstream {} out of color 0", u);
        }
        for &d in &wiring[0].downstream {
            assert!(d < 40, "downstream {} out of color 0", d);
        }
    }

    #[test]
    fn ternary_weights_yang_yin() {
        let qian = Trigram::Qian.ternary_weights();
        assert_eq!(qian, [1, 1, 1]); // all yang
        let kun = Trigram::Kun.ternary_weights();
        assert_eq!(kun, [-1, -1, -1]); // all yin
    }

    #[test]
    fn forty_sentrons_per_color() {
        // 5 phases × 8 trigrams = 40
        assert_eq!(5 * 8, 40);
        // 9 colors × 40 = 360
        assert_eq!(9 * 40, 360);
    }

    #[test]
    fn wuxing_bagua_product() {
        // Every (phase, trigram) pair maps to exactly one sentron in a color
        let mut seen = vec![false; 40];
        for phase in 0..5u16 {
            for trigram in 0..8u16 {
                let id = phase * 8 + trigram;
                assert!(!seen[id as usize], "duplicate at phase={}, trigram={}", phase, trigram);
                seen[id as usize] = true;
            }
        }
        assert!(seen.iter().all(|&s| s));
    }
}
