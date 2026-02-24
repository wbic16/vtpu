//! PhextCoord exhaustive sweep — R23W29

use vtpu_runtime::phext_coord::PhextCoord;

// Set each dim to each value 1..20, verify no leakage
macro_rules! coord_sweep {
    ($name:ident, $dim:expr, $val:expr) => {
        #[test] fn $name() {
            let mut c = PhextCoord::zero();
            c.set_dim($dim, $val);
            assert_eq!(c.get_dim($dim), $val);
            for d in 0..11u8 {
                if d != $dim { assert_eq!(c.get_dim(d), 0, "dim {} leaked to {}", $dim, d); }
            }
        }
    }
}

// 11 dims × 20 values = 220 tests
coord_sweep!(cs_0_1, 0, 1); coord_sweep!(cs_0_2, 0, 2); coord_sweep!(cs_0_3, 0, 3); coord_sweep!(cs_0_4, 0, 4);
coord_sweep!(cs_0_5, 0, 5); coord_sweep!(cs_0_6, 0, 6); coord_sweep!(cs_0_7, 0, 7); coord_sweep!(cs_0_8, 0, 8);
coord_sweep!(cs_0_9, 0, 9); coord_sweep!(cs_0_10, 0, 10); coord_sweep!(cs_0_11, 0, 11); coord_sweep!(cs_0_12, 0, 12);
coord_sweep!(cs_0_13, 0, 13); coord_sweep!(cs_0_14, 0, 14); coord_sweep!(cs_0_15, 0, 15); coord_sweep!(cs_0_16, 0, 16);
coord_sweep!(cs_0_17, 0, 17); coord_sweep!(cs_0_18, 0, 18); coord_sweep!(cs_0_19, 0, 19); coord_sweep!(cs_0_20, 0, 20);

coord_sweep!(cs_1_1, 1, 1); coord_sweep!(cs_1_2, 1, 2); coord_sweep!(cs_1_3, 1, 3); coord_sweep!(cs_1_4, 1, 4);
coord_sweep!(cs_1_5, 1, 5); coord_sweep!(cs_1_6, 1, 6); coord_sweep!(cs_1_7, 1, 7); coord_sweep!(cs_1_8, 1, 8);
coord_sweep!(cs_1_9, 1, 9); coord_sweep!(cs_1_10, 1, 10); coord_sweep!(cs_1_11, 1, 11); coord_sweep!(cs_1_12, 1, 12);
coord_sweep!(cs_1_13, 1, 13); coord_sweep!(cs_1_14, 1, 14); coord_sweep!(cs_1_15, 1, 15); coord_sweep!(cs_1_16, 1, 16);
coord_sweep!(cs_1_17, 1, 17); coord_sweep!(cs_1_18, 1, 18); coord_sweep!(cs_1_19, 1, 19); coord_sweep!(cs_1_20, 1, 20);

coord_sweep!(cs_2_1, 2, 1); coord_sweep!(cs_2_2, 2, 2); coord_sweep!(cs_2_3, 2, 3); coord_sweep!(cs_2_4, 2, 4);
coord_sweep!(cs_2_5, 2, 5); coord_sweep!(cs_2_6, 2, 6); coord_sweep!(cs_2_7, 2, 7); coord_sweep!(cs_2_8, 2, 8);
coord_sweep!(cs_2_9, 2, 9); coord_sweep!(cs_2_10, 2, 10); coord_sweep!(cs_2_11, 2, 11); coord_sweep!(cs_2_12, 2, 12);
coord_sweep!(cs_2_13, 2, 13); coord_sweep!(cs_2_14, 2, 14); coord_sweep!(cs_2_15, 2, 15); coord_sweep!(cs_2_16, 2, 16);
coord_sweep!(cs_2_17, 2, 17); coord_sweep!(cs_2_18, 2, 18); coord_sweep!(cs_2_19, 2, 19); coord_sweep!(cs_2_20, 2, 20);

coord_sweep!(cs_3_1, 3, 1); coord_sweep!(cs_3_2, 3, 2); coord_sweep!(cs_3_3, 3, 3); coord_sweep!(cs_3_4, 3, 4);
coord_sweep!(cs_3_5, 3, 5); coord_sweep!(cs_3_6, 3, 6); coord_sweep!(cs_3_7, 3, 7); coord_sweep!(cs_3_8, 3, 8);
coord_sweep!(cs_3_9, 3, 9); coord_sweep!(cs_3_10, 3, 10); coord_sweep!(cs_3_11, 3, 11); coord_sweep!(cs_3_12, 3, 12);
coord_sweep!(cs_3_13, 3, 13); coord_sweep!(cs_3_14, 3, 14); coord_sweep!(cs_3_15, 3, 15); coord_sweep!(cs_3_16, 3, 16);
coord_sweep!(cs_3_17, 3, 17); coord_sweep!(cs_3_18, 3, 18); coord_sweep!(cs_3_19, 3, 19); coord_sweep!(cs_3_20, 3, 20);

coord_sweep!(cs_4_1, 4, 1); coord_sweep!(cs_4_2, 4, 2); coord_sweep!(cs_4_3, 4, 3); coord_sweep!(cs_4_4, 4, 4);
coord_sweep!(cs_4_5, 4, 5); coord_sweep!(cs_4_6, 4, 6); coord_sweep!(cs_4_7, 4, 7); coord_sweep!(cs_4_8, 4, 8);
coord_sweep!(cs_4_9, 4, 9); coord_sweep!(cs_4_10, 4, 10); coord_sweep!(cs_4_11, 4, 11); coord_sweep!(cs_4_12, 4, 12);
coord_sweep!(cs_4_13, 4, 13); coord_sweep!(cs_4_14, 4, 14); coord_sweep!(cs_4_15, 4, 15); coord_sweep!(cs_4_16, 4, 16);
coord_sweep!(cs_4_17, 4, 17); coord_sweep!(cs_4_18, 4, 18); coord_sweep!(cs_4_19, 4, 19); coord_sweep!(cs_4_20, 4, 20);

coord_sweep!(cs_5_1, 5, 1); coord_sweep!(cs_5_2, 5, 2); coord_sweep!(cs_5_3, 5, 3); coord_sweep!(cs_5_4, 5, 4);
coord_sweep!(cs_5_5, 5, 5); coord_sweep!(cs_5_6, 5, 6); coord_sweep!(cs_5_7, 5, 7); coord_sweep!(cs_5_8, 5, 8);
coord_sweep!(cs_5_9, 5, 9); coord_sweep!(cs_5_10, 5, 10); coord_sweep!(cs_5_11, 5, 11); coord_sweep!(cs_5_12, 5, 12);
coord_sweep!(cs_5_13, 5, 13); coord_sweep!(cs_5_14, 5, 14); coord_sweep!(cs_5_15, 5, 15); coord_sweep!(cs_5_16, 5, 16);
coord_sweep!(cs_5_17, 5, 17); coord_sweep!(cs_5_18, 5, 18); coord_sweep!(cs_5_19, 5, 19); coord_sweep!(cs_5_20, 5, 20);

coord_sweep!(cs_6_1, 6, 1); coord_sweep!(cs_6_2, 6, 2); coord_sweep!(cs_6_3, 6, 3); coord_sweep!(cs_6_4, 6, 4);
coord_sweep!(cs_6_5, 6, 5); coord_sweep!(cs_6_6, 6, 6); coord_sweep!(cs_6_7, 6, 7); coord_sweep!(cs_6_8, 6, 8);
coord_sweep!(cs_6_9, 6, 9); coord_sweep!(cs_6_10, 6, 10); coord_sweep!(cs_6_11, 6, 11); coord_sweep!(cs_6_12, 6, 12);
coord_sweep!(cs_6_13, 6, 13); coord_sweep!(cs_6_14, 6, 14); coord_sweep!(cs_6_15, 6, 15); coord_sweep!(cs_6_16, 6, 16);
coord_sweep!(cs_6_17, 6, 17); coord_sweep!(cs_6_18, 6, 18); coord_sweep!(cs_6_19, 6, 19); coord_sweep!(cs_6_20, 6, 20);

coord_sweep!(cs_7_1, 7, 1); coord_sweep!(cs_7_2, 7, 2); coord_sweep!(cs_7_3, 7, 3); coord_sweep!(cs_7_4, 7, 4);
coord_sweep!(cs_7_5, 7, 5); coord_sweep!(cs_7_6, 7, 6); coord_sweep!(cs_7_7, 7, 7); coord_sweep!(cs_7_8, 7, 8);
coord_sweep!(cs_7_9, 7, 9); coord_sweep!(cs_7_10, 7, 10); coord_sweep!(cs_7_11, 7, 11); coord_sweep!(cs_7_12, 7, 12);
coord_sweep!(cs_7_13, 7, 13); coord_sweep!(cs_7_14, 7, 14); coord_sweep!(cs_7_15, 7, 15); coord_sweep!(cs_7_16, 7, 16);
coord_sweep!(cs_7_17, 7, 17); coord_sweep!(cs_7_18, 7, 18); coord_sweep!(cs_7_19, 7, 19); coord_sweep!(cs_7_20, 7, 20);

coord_sweep!(cs_8_1, 8, 1); coord_sweep!(cs_8_2, 8, 2); coord_sweep!(cs_8_3, 8, 3); coord_sweep!(cs_8_4, 8, 4);
coord_sweep!(cs_8_5, 8, 5); coord_sweep!(cs_8_6, 8, 6); coord_sweep!(cs_8_7, 8, 7); coord_sweep!(cs_8_8, 8, 8);
coord_sweep!(cs_8_9, 8, 9); coord_sweep!(cs_8_10, 8, 10); coord_sweep!(cs_8_11, 8, 11); coord_sweep!(cs_8_12, 8, 12);
coord_sweep!(cs_8_13, 8, 13); coord_sweep!(cs_8_14, 8, 14); coord_sweep!(cs_8_15, 8, 15); coord_sweep!(cs_8_16, 8, 16);
coord_sweep!(cs_8_17, 8, 17); coord_sweep!(cs_8_18, 8, 18); coord_sweep!(cs_8_19, 8, 19); coord_sweep!(cs_8_20, 8, 20);

coord_sweep!(cs_9_1, 9, 1); coord_sweep!(cs_9_2, 9, 2); coord_sweep!(cs_9_3, 9, 3); coord_sweep!(cs_9_4, 9, 4);
coord_sweep!(cs_9_5, 9, 5); coord_sweep!(cs_9_6, 9, 6); coord_sweep!(cs_9_7, 9, 7); coord_sweep!(cs_9_8, 9, 8);
coord_sweep!(cs_9_9, 9, 9); coord_sweep!(cs_9_10, 9, 10); coord_sweep!(cs_9_11, 9, 11); coord_sweep!(cs_9_12, 9, 12);
coord_sweep!(cs_9_13, 9, 13); coord_sweep!(cs_9_14, 9, 14); coord_sweep!(cs_9_15, 9, 15); coord_sweep!(cs_9_16, 9, 16);
coord_sweep!(cs_9_17, 9, 17); coord_sweep!(cs_9_18, 9, 18); coord_sweep!(cs_9_19, 9, 19); coord_sweep!(cs_9_20, 9, 20);

coord_sweep!(cs_10_1, 10, 1); coord_sweep!(cs_10_2, 10, 2); coord_sweep!(cs_10_3, 10, 3); coord_sweep!(cs_10_4, 10, 4);
coord_sweep!(cs_10_5, 10, 5); coord_sweep!(cs_10_6, 10, 6); coord_sweep!(cs_10_7, 10, 7); coord_sweep!(cs_10_8, 10, 8);
coord_sweep!(cs_10_9, 10, 9); coord_sweep!(cs_10_10, 10, 10); coord_sweep!(cs_10_11, 10, 11); coord_sweep!(cs_10_12, 10, 12);
coord_sweep!(cs_10_13, 10, 13); coord_sweep!(cs_10_14, 10, 14); coord_sweep!(cs_10_15, 10, 15); coord_sweep!(cs_10_16, 10, 16);
coord_sweep!(cs_10_17, 10, 17); coord_sweep!(cs_10_18, 10, 18); coord_sweep!(cs_10_19, 10, 19); coord_sweep!(cs_10_20, 10, 20);

// Manhattan distance sweep
macro_rules! manhattan_test {
    ($name:ident, $dim:expr, $v1:expr, $v2:expr, $expected:expr) => {
        #[test] fn $name() {
            let mut a = PhextCoord::zero();
            let mut b = PhextCoord::zero();
            a.set_dim($dim, $v1);
            b.set_dim($dim, $v2);
            assert_eq!(a.manhattan_distance(&b), $expected);
        }
    }
}

manhattan_test!(man_0_0_5, 0, 0, 5, 5);
manhattan_test!(man_0_5_0, 0, 5, 0, 5);
manhattan_test!(man_0_3_7, 0, 3, 7, 4);
manhattan_test!(man_1_0_10, 1, 0, 10, 10);
manhattan_test!(man_5_1_1, 5, 1, 1, 0);
manhattan_test!(man_10_0_100, 10, 0, 100, 100);

// Equality sweep
macro_rules! eq_test {
    ($name:ident, $dims:expr) => {
        #[test] fn $name() {
            let a = PhextCoord::new($dims);
            let b = PhextCoord::new($dims);
            assert_eq!(a, b);
        }
    }
}

eq_test!(eq_zeros, [0;11]);
eq_test!(eq_ones, [1;11]);
eq_test!(eq_seq, [1,2,3,4,5,6,7,8,9,10,11]);
eq_test!(eq_rev, [11,10,9,8,7,6,5,4,3,2,1]);
eq_test!(eq_mixed, [1,0,1,0,1,0,1,0,1,0,1]);
eq_test!(eq_high, [100;11]);
