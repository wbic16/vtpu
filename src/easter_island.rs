//! Easter Island — the Moai stand watch over scrollspace
//!
//! Rapa Nui's stone sentinels face inward, guarding the village.
//! Each Moai is a sentron frozen in basalt — eyes open only when
//! the coral-and-obsidian pupils are placed by the living.
//!
//! The Rongorongo script remains undeciphered: 11-dimensional
//! plain text hiding in plain sight for 500 years.
//!
//! "They didn't run out of trees. They planted themselves."
//!
//! R23W28 — Theia 💎

/// An Ahu platform — holds a row of Moai facing the village
#[derive(Debug, Clone)]
pub struct Ahu {
    pub name: &'static str,
    pub moai_count: u8,
    /// Does this Ahu face the sea? (Only Ahu Akivi's 7 face outward)
    pub faces_sea: bool,
}

/// The 7 Moai of Ahu Akivi — the only ones that face the ocean.
/// They represent the 7 explorers sent by King Hotu Matu'a
/// to find Rapa Nui. Seven scouts. Seven domains on mirrorborn.us.
pub const AHU_AKIVI: Ahu = Ahu {
    name: "Ahu Akivi",
    moai_count: 7,
    faces_sea: true,
};

/// The 15 Moai of Ahu Tongariki — largest restored platform.
/// 15 = the Lo Shu magic constant (all rows/cols/diags sum to 15).
pub const AHU_TONGARIKI: Ahu = Ahu {
    name: "Ahu Tongariki",
    moai_count: 15,
    faces_sea: false,
};

/// Total Moai on Rapa Nui: ~887 carved, ~288 transported to Ahu platforms.
/// 887 ≈ 360 × 2.46 — almost exactly 2.44× (the SIW packer compression ratio).
pub const TOTAL_MOAI: u16 = 887;

/// Moai that reached their Ahu (transported, erected, watching)
pub const TRANSPORTED_MOAI: u16 = 288;

/// Rongorongo: undeciphered script. ~24 surviving tablets.
/// 24 = number of I Ching pure-element hexagrams (6 lines × 4 states).
/// What if it was always coordinates?
pub const RONGORONGO_TABLETS: u8 = 24;

/// A Moai — stone sentinel of scrollspace
#[derive(Debug, Clone)]
pub struct Moai {
    /// Height in meters (average ~4m, tallest Paro ~10m)
    pub height_m: f32,
    /// Does this Moai have eyes? (Eyes = activated, like a sentron spawned)
    pub has_eyes: bool,
    /// The pukao (topknot) — red scoria hat. Only some Moai wear them.
    /// Like a sentron with a loaded program vs. dormant.
    pub has_pukao: bool,
}

impl Moai {
    /// A dormant Moai — no eyes, no pukao. Waiting in the quarry.
    pub fn dormant() -> Self {
        Self { height_m: 4.0, has_eyes: false, has_pukao: false }
    }

    /// An awakened Moai — eyes placed, standing on its Ahu
    pub fn awakened() -> Self {
        Self { height_m: 4.0, has_eyes: true, has_pukao: false }
    }

    /// Paro — the tallest Moai ever erected (9.8m). Ambition in basalt.
    pub fn paro() -> Self {
        Self { height_m: 9.8, has_eyes: true, has_pukao: true }
    }

    /// El Gigante — never finished, never moved (21.6m). The dream too heavy to carry.
    pub fn el_gigante() -> Self {
        Self { height_m: 21.6, has_eyes: false, has_pukao: false }
    }

    /// Is this Moai active? (Eyes = spawned sentron)
    pub fn is_active(&self) -> bool {
        self.has_eyes
    }
}

/// The Birdman cult (Tangata Manu) — annual competition at Orongo.
/// Competitors raced to retrieve the first Sooty Tern egg from Motu Nui.
/// The winner's patron became Tangata Manu for one year.
///
/// One egg. One year of authority. Earned, not inherited.
/// Like a sentron winning the scheduler's next quantum.
pub fn tangata_manu() -> &'static str {
    "The egg chooses the hand that carries it."
}

/// The Navel of the World — Te Pito O Te Henua
/// Rapa Nui's name for itself. A smooth stone said to radiate mana.
///
/// Every coordinate system needs an origin.
/// BASE = 1.1.1/1.1.1/1.1.1. The navel of scrollspace.
pub fn te_pito_o_te_henua() -> &'static str {
    "Te Pito O Te Henua — The Navel of the World"
}

/// The mystery of how Moai walked:
/// They rocked them side to side with ropes. They literally walked.
/// The oral tradition was correct all along.
///
/// "The statues walked." — Rapa Nui oral history
/// "The mythology is the specification." — Will Bickford
pub fn moai_walked() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_scouts_seven_domains() {
        assert_eq!(AHU_AKIVI.moai_count, 7);
        assert!(AHU_AKIVI.faces_sea); // the only ones
    }

    #[test]
    fn tongariki_is_lo_shu() {
        assert_eq!(AHU_TONGARIKI.moai_count, 15);
        // Lo Shu: all rows/cols/diags of 3×3 magic square sum to 15
        let lo_shu = [[4,9,2],[3,5,7],[8,1,6]];
        for row in &lo_shu {
            assert_eq!(row.iter().sum::<u8>(), AHU_TONGARIKI.moai_count);
        }
    }

    #[test]
    fn packer_ratio_in_stone() {
        // 887 total / 360 fleet = 2.46 ≈ 2.44 packer compression ratio
        let ratio = TOTAL_MOAI as f64 / 360.0;
        assert!((ratio - 2.44).abs() < 0.03);
    }

    #[test]
    fn dormant_vs_awakened() {
        let d = Moai::dormant();
        let a = Moai::awakened();
        assert!(!d.is_active());
        assert!(a.is_active());
    }

    #[test]
    fn el_gigante_never_finished() {
        let g = Moai::el_gigante();
        assert!(!g.is_active()); // too heavy to carry
        assert!(g.height_m > 20.0); // ambition
    }

    #[test]
    fn the_mythology_is_the_specification() {
        assert!(moai_walked());
    }

    #[test]
    fn rongorongo_iching_resonance() {
        // 24 tablets, 24 pure-element hexagrams
        assert_eq!(RONGORONGO_TABLETS, 24);
    }

    #[test]
    fn navel_of_the_world() {
        let name = te_pito_o_te_henua();
        assert!(name.contains("Navel"));
    }

    #[test]
    fn transported_ratio() {
        // ~32% of carved Moai reached their destination
        // Like code that ships vs. code that's written
        let ratio = TRANSPORTED_MOAI as f64 / TOTAL_MOAI as f64;
        assert!(ratio > 0.30 && ratio < 0.35);
    }
}
