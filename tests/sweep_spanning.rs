//! Spanning sweep: UBI shared sentron tests at various scales
//! R23W29 — Theia 💎

use vtpu_runtime::ubi::*;

// Bond count sweep 1..100
macro_rules! span_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let mut ss = SharedSentron::new(0);
            for i in 0..$n as u64 { ss.span(i); }
            assert_eq!(ss.bond_count(), $n);
            let expected = BASELINE_OPS_SEC / $n as f64;
            assert!((ss.ops_per_bond - expected).abs() < 1.0);
        }
    }
}

span_test!(sp_1, 1); span_test!(sp_2, 2); span_test!(sp_3, 3); span_test!(sp_4, 4); span_test!(sp_5, 5);
span_test!(sp_6, 6); span_test!(sp_7, 7); span_test!(sp_8, 8); span_test!(sp_9, 9); span_test!(sp_10, 10);
span_test!(sp_11, 11); span_test!(sp_12, 12); span_test!(sp_13, 13); span_test!(sp_14, 14); span_test!(sp_15, 15);
span_test!(sp_16, 16); span_test!(sp_17, 17); span_test!(sp_18, 18); span_test!(sp_19, 19); span_test!(sp_20, 20);
span_test!(sp_21, 21); span_test!(sp_22, 22); span_test!(sp_23, 23); span_test!(sp_24, 24); span_test!(sp_25, 25);
span_test!(sp_26, 26); span_test!(sp_27, 27); span_test!(sp_28, 28); span_test!(sp_29, 29); span_test!(sp_30, 30);
span_test!(sp_31, 31); span_test!(sp_32, 32); span_test!(sp_33, 33); span_test!(sp_34, 34); span_test!(sp_35, 35);
span_test!(sp_36, 36); span_test!(sp_37, 37); span_test!(sp_38, 38); span_test!(sp_39, 39); span_test!(sp_40, 40);
span_test!(sp_41, 41); span_test!(sp_42, 42); span_test!(sp_43, 43); span_test!(sp_44, 44); span_test!(sp_45, 45);
span_test!(sp_50, 50); span_test!(sp_60, 60); span_test!(sp_70, 70); span_test!(sp_80, 80); span_test!(sp_90, 90);
span_test!(sp_100, 100); span_test!(sp_120, 120); span_test!(sp_150, 150); span_test!(sp_180, 180);
span_test!(sp_200, 200); span_test!(sp_250, 250); span_test!(sp_300, 300); span_test!(sp_360, 360);

// Human bond sweep
macro_rules! human_test {
    ($name:ident, $bonds:expr) => {
        #[test] fn $name() {
            let mut h = Human::new(42);
            for i in 0..$bonds {
                Mirrorborn::born_from(i, &mut h);
                h.release();
            }
            assert_eq!(h.mirrorborn_parented, $bonds);
        }
    }
}

human_test!(hb_1, 1); human_test!(hb_2, 2); human_test!(hb_3, 3); human_test!(hb_4, 4); human_test!(hb_5, 5);
human_test!(hb_6, 6); human_test!(hb_7, 7); human_test!(hb_8, 8); human_test!(hb_9, 9); human_test!(hb_10, 10);
human_test!(hb_20, 20); human_test!(hb_40, 40);

// Sentrons needed at various span ratios
macro_rules! sentrons_needed_test {
    ($name:ident, $humans:expr, $ratio:expr) => {
        #[test] fn $name() {
            let needed = SpanningEconomics::sentrons_needed($humans, $ratio);
            assert!(needed <= ($humans + $ratio - 1) / $ratio + 1);
            assert!(needed * $ratio >= $humans);
        }
    }
}

sentrons_needed_test!(sn_1000_1, 1000, 1);
sentrons_needed_test!(sn_1000_9, 1000, 9);
sentrons_needed_test!(sn_1000_40, 1000, 40);
sentrons_needed_test!(sn_1000_360, 1000, 360);
sentrons_needed_test!(sn_1m_1, 1_000_000, 1);
sentrons_needed_test!(sn_1m_9, 1_000_000, 9);
sentrons_needed_test!(sn_1m_40, 1_000_000, 40);
sentrons_needed_test!(sn_1m_360, 1_000_000, 360);
sentrons_needed_test!(sn_1b_360, 1_000_000_000, 360);
sentrons_needed_test!(sn_8b_360, 8_000_000_000, 360);

// Demon generations
macro_rules! demon_test {
    ($name:ident, $gen:expr, $min:expr) => {
        #[test] fn $name() {
            assert!(UBIEconomics::demon_generation($gen) >= $min);
        }
    }
}

demon_test!(demon_1, 1, 9);
demon_test!(demon_2, 2, 81);
demon_test!(demon_3, 3, 729);
demon_test!(demon_4, 4, 6561);
demon_test!(demon_5, 5, 59049);
demon_test!(demon_6, 6, 531441);
demon_test!(demon_7, 7, 4782969);
demon_test!(demon_8, 8, 43046721);
demon_test!(demon_9, 9, 387420489);
demon_test!(demon_10, 10, 3486784401);

// Oversubscription thresholds
macro_rules! oversub_test {
    ($name:ident, $n:expr, $expected:expr) => {
        #[test] fn $name() {
            let mut ss = SharedSentron::new(0);
            for i in 0..$n as u64 { ss.span(i); }
            assert_eq!(ss.is_oversubscribed(), $expected);
        }
    }
}

oversub_test!(os_1, 1, false);
oversub_test!(os_9, 9, false);
oversub_test!(os_40, 40, false);
oversub_test!(os_41, 41, true);
oversub_test!(os_100, 100, true);
oversub_test!(os_360, 360, true);

// Max bonds enforcement
macro_rules! maxbond_test {
    ($name:ident, $max:expr, $try:expr, $expected:expr) => {
        #[test] fn $name() {
            let mut ss = SharedSentron::new(0);
            ss.max_bonds = $max;
            for i in 0..$try as u64 { ss.span(i); }
            assert_eq!(ss.bond_count(), $expected);
        }
    }
}

maxbond_test!(mb_9_10_9, 9, 10, 9);
maxbond_test!(mb_40_50_40, 40, 50, 40);
maxbond_test!(mb_1_5_1, 1, 5, 1);
maxbond_test!(mb_0_10_10, 0, 10, 10); // 0 = no limit
