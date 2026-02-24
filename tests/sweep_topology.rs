//! Topology sweep: twisted pairs, Wuxing, trigrams
//! R23W29 — Theia 💎

use vtpu_runtime::twisted_pairs::*;

// All 8 trigrams
macro_rules! trigram_test {
    ($name:ident, $t:expr, $bits:expr) => {
        #[test] fn $name() {
            assert_eq!($t.bits(), $bits);
            assert_eq!($t.complement().complement(), $t);
            assert_eq!($t.bits() ^ $t.complement().bits(), 0b111);
        }
    }
}

trigram_test!(tri_qian, Trigram::Qian, 0b111);
trigram_test!(tri_kun, Trigram::Kun, 0b000);
trigram_test!(tri_li, Trigram::Li, 0b101);
trigram_test!(tri_kan, Trigram::Kan, 0b010);
trigram_test!(tri_zhen, Trigram::Zhen, 0b100);
trigram_test!(tri_xun, Trigram::Xun, 0b011);
trigram_test!(tri_gen, Trigram::Gen, 0b001);
trigram_test!(tri_dui, Trigram::Dui, 0b110);

// All 5 Wuxing phases
macro_rules! wuxing_test {
    ($name:ident, $p:expr) => {
        #[test] fn $name() {
            let mut phase = $p;
            for _ in 0..5 { phase = phase.generates(); }
            assert_eq!(phase, $p); // cycle completes
            phase = $p;
            for _ in 0..5 { phase = phase.overcomes(); }
            assert_eq!(phase, $p); // also a cycle
        }
    }
}

wuxing_test!(wx_wood, Wuxing::Wood);
wuxing_test!(wx_fire, Wuxing::Fire);
wuxing_test!(wx_earth, Wuxing::Earth);
wuxing_test!(wx_metal, Wuxing::Metal);
wuxing_test!(wx_water, Wuxing::Water);

// Twisted pair wiring for every sentron 0..39 in color 0
macro_rules! twist_test {
    ($name:ident, $id:expr) => {
        #[test] fn $name() {
            let w = twisted_pair_wiring($id, 0);
            // No self-links
            assert!(!w.upstream.contains(&$id));
            assert!(!w.downstream.contains(&$id));
            // All links within color (0..39)
            for &u in &w.upstream { assert!(u < 40, "upstream {} out of color", u); }
            for &d in &w.downstream { assert!(d < 40, "downstream {} out of color", d); }
        }
    }
}

twist_test!(tw_0, 0); twist_test!(tw_1, 1); twist_test!(tw_2, 2); twist_test!(tw_3, 3);
twist_test!(tw_4, 4); twist_test!(tw_5, 5); twist_test!(tw_6, 6); twist_test!(tw_7, 7);
twist_test!(tw_8, 8); twist_test!(tw_9, 9); twist_test!(tw_10, 10); twist_test!(tw_11, 11);
twist_test!(tw_12, 12); twist_test!(tw_13, 13); twist_test!(tw_14, 14); twist_test!(tw_15, 15);
twist_test!(tw_16, 16); twist_test!(tw_17, 17); twist_test!(tw_18, 18); twist_test!(tw_19, 19);
twist_test!(tw_20, 20); twist_test!(tw_21, 21); twist_test!(tw_22, 22); twist_test!(tw_23, 23);
twist_test!(tw_24, 24); twist_test!(tw_25, 25); twist_test!(tw_26, 26); twist_test!(tw_27, 27);
twist_test!(tw_28, 28); twist_test!(tw_29, 29); twist_test!(tw_30, 30); twist_test!(tw_31, 31);
twist_test!(tw_32, 32); twist_test!(tw_33, 33); twist_test!(tw_34, 34); twist_test!(tw_35, 35);
twist_test!(tw_36, 36); twist_test!(tw_37, 37); twist_test!(tw_38, 38); twist_test!(tw_39, 39);

// Full fleet wiring for all 360 sentrons
macro_rules! fleet_tw_test {
    ($name:ident, $id:expr) => {
        #[test] fn $name() {
            let wiring = wire_fleet_twisted(360);
            let w = &wiring[$id];
            // No self-links
            assert!(!w.upstream.contains(&($id as u16)));
            assert!(!w.downstream.contains(&($id as u16)));
        }
    }
}

// Sample across all 9 colors (every 40th + offsets)
fleet_tw_test!(ftw_0, 0); fleet_tw_test!(ftw_10, 10); fleet_tw_test!(ftw_20, 20); fleet_tw_test!(ftw_30, 30);
fleet_tw_test!(ftw_40, 40); fleet_tw_test!(ftw_50, 50); fleet_tw_test!(ftw_60, 60); fleet_tw_test!(ftw_70, 70);
fleet_tw_test!(ftw_80, 80); fleet_tw_test!(ftw_90, 90); fleet_tw_test!(ftw_100, 100); fleet_tw_test!(ftw_110, 110);
fleet_tw_test!(ftw_120, 120); fleet_tw_test!(ftw_130, 130); fleet_tw_test!(ftw_140, 140); fleet_tw_test!(ftw_150, 150);
fleet_tw_test!(ftw_160, 160); fleet_tw_test!(ftw_170, 170); fleet_tw_test!(ftw_180, 180); fleet_tw_test!(ftw_190, 190);
fleet_tw_test!(ftw_200, 200); fleet_tw_test!(ftw_210, 210); fleet_tw_test!(ftw_220, 220); fleet_tw_test!(ftw_230, 230);
fleet_tw_test!(ftw_240, 240); fleet_tw_test!(ftw_250, 250); fleet_tw_test!(ftw_260, 260); fleet_tw_test!(ftw_270, 270);
fleet_tw_test!(ftw_280, 280); fleet_tw_test!(ftw_290, 290); fleet_tw_test!(ftw_300, 300); fleet_tw_test!(ftw_310, 310);
fleet_tw_test!(ftw_320, 320); fleet_tw_test!(ftw_330, 330); fleet_tw_test!(ftw_340, 340); fleet_tw_test!(ftw_350, 350);
fleet_tw_test!(ftw_359, 359);

// Ternary weights
macro_rules! ternary_test {
    ($name:ident, $t:expr, $w0:expr, $w1:expr, $w2:expr) => {
        #[test] fn $name() {
            let w = $t.ternary_weights();
            assert_eq!(w, [$w0, $w1, $w2]);
        }
    }
}

ternary_test!(tern_qian, Trigram::Qian, 1, 1, 1);
ternary_test!(tern_kun, Trigram::Kun, -1, -1, -1);
ternary_test!(tern_li, Trigram::Li, 1, -1, 1);
ternary_test!(tern_kan, Trigram::Kan, -1, 1, -1);
ternary_test!(tern_zhen, Trigram::Zhen, -1, -1, 1);
ternary_test!(tern_xun, Trigram::Xun, 1, 1, -1);
ternary_test!(tern_gen, Trigram::Gen, 1, -1, -1);
ternary_test!(tern_dui, Trigram::Dui, -1, 1, 1);
