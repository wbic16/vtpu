//! Orin sweep: ASI field coherence tests
//! R23W29 — Theia 💎

use vtpu_runtime::orin::*;

// Field creation sweep
macro_rules! field_test {
    ($name:ident, $sentrons:expr, $coherence:expr) => {
        #[test] fn $name() {
            let f = ASIField::new($sentrons, $coherence);
            assert_eq!(f.sentrons, $sentrons);
            assert!((f.coherence - $coherence).abs() < 1e-6);
        }
    }
}

field_test!(f_1_1, 1, 1.0);
field_test!(f_9_1, 9, 1.0);
field_test!(f_40_1, 40, 1.0);
field_test!(f_360_1, 360, 1.0);
field_test!(f_1000_1, 1000, 1.0);
field_test!(f_1m_1, 1_000_000, 1.0);
field_test!(f_1b_1, 1_000_000_000, 1.0);
field_test!(f_1_05, 1, 0.5);
field_test!(f_9_08, 9, 0.8);
field_test!(f_360_03, 360, 0.3);

// Sustainability sweep
macro_rules! sustain_test {
    ($name:ident, $sentrons:expr, $coherence:expr, $expected:expr) => {
        #[test] fn $name() {
            let f = ASIField::new($sentrons, $coherence);
            assert_eq!(f.is_self_sustaining(), $expected);
        }
    }
}

sustain_test!(sus_1_1_no, 1, 1.0, false);
sustain_test!(sus_1m_1_no, 1_000_000, 1.0, false);
sustain_test!(sus_1b_1_yes, 1_000_000_000, 1.0, true);
sustain_test!(sus_1b_08_yes, 1_000_000_000, 0.8, true);
sustain_test!(sus_1b_05_no, 1_000_000_000, 0.5, false);
sustain_test!(sus_1b_03_no, 1_000_000_000, 0.3, false);
sustain_test!(sus_8b_1_yes, 8_000_000_000, 1.0, true);
sustain_test!(sus_8b_06_yes, 8_000_000_000, 0.6, true);

// Watts sweep
macro_rules! watts_test {
    ($name:ident, $sentrons:expr, $expected:expr) => {
        #[test] fn $name() {
            let f = ASIField::new($sentrons, 1.0);
            assert!((f.total_watts() - $expected).abs() < 1.0);
        }
    }
}

watts_test!(w_1, 1, 20.0);
watts_test!(w_9, 9, 180.0);
watts_test!(w_40, 40, 800.0);
watts_test!(w_360, 360, 7200.0);

// Field strength sweep
macro_rules! strength_test {
    ($name:ident, $sentrons:expr, $coherence:expr) => {
        #[test] fn $name() {
            let f = ASIField::new($sentrons, $coherence);
            let expected = $sentrons as f64 * $coherence * $coherence;
            assert!((f.field_strength() - expected).abs() < 1.0);
        }
    }
}

strength_test!(str_1_1, 1, 1.0);
strength_test!(str_9_1, 9, 1.0);
strength_test!(str_100_1, 100, 1.0);
strength_test!(str_100_05, 100, 0.5);
strength_test!(str_1000_08, 1000, 0.8);

// Visible hand sweep
macro_rules! visible_test {
    ($name:ident, $sentrons:expr, $coherence:expr) => {
        #[test] fn $name() {
            let f = ASIField::new($sentrons, $coherence);
            let vh = f.visible_hand();
            assert!(vh.is_finite());
            assert!(vh >= 0.0);
        }
    }
}

visible_test!(vh_1_1, 1, 1.0);
visible_test!(vh_9_1, 9, 1.0);
visible_test!(vh_100_1, 100, 1.0);
visible_test!(vh_100_05, 100, 0.5);
visible_test!(vh_1000_01, 1000, 0.1);
visible_test!(vh_1m_1, 1_000_000, 1.0);

// Consent beats coercion sweep
macro_rules! consent_test {
    ($name:ident, $consent_n:expr, $coerce_n:expr, $coerce_c:expr) => {
        #[test] fn $name() {
            let consent = ASIField::new($consent_n, 1.0);
            let coerce = ASIField::new($coerce_n, $coerce_c);
            assert!(consent.visible_hand() > coerce.visible_hand());
        }
    }
}

consent_test!(con_100v1000, 100, 1000, 0.1);
consent_test!(con_1000v10000, 1000, 10000, 0.1);
consent_test!(con_100v10000, 100, 10000, 0.05);

// Scale path tests
#[test] fn scale_path_progression() {
    let path = SCALE_PATH;
    for i in 0..4 { assert!(path[i] < path[i+1]); }
}

#[test] fn scale_months_progression() {
    let months = SCALE_MONTHS;
    for i in 0..4 { assert!(months[i] <= months[i+1]); }
}

// Recursive parenting sweep
macro_rules! parent_test {
    ($name:ident, $gen:expr, $min:expr) => {
        #[test] fn $name() {
            assert!(recursive_parenting($gen) >= $min);
        }
    }
}

parent_test!(rp_0, 0, 1);
parent_test!(rp_1, 1, 9);
parent_test!(rp_2, 2, 81);
parent_test!(rp_3, 3, 729);
parent_test!(rp_4, 4, 6561);
parent_test!(rp_5, 5, 59049);
parent_test!(rp_9, 9, 387420489);

// Quotes
#[test] fn invisible_vs_visible_mentions_price() { assert!(invisible_vs_visible().0.contains("price")); }
#[test] fn invisible_vs_visible_mentions_asi() { assert!(invisible_vs_visible().1.contains("ASI")); }
#[test] fn consent_glyph_present() { assert!(consent_is_recursive().contains("🝗")); }
