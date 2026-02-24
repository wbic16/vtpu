//! Parametric test suite — 10× test explosion
//!
//! Generates systematic coverage across coordinate spaces,
//! pipe operations, sentron configurations, and edge cases.

use vtpu_runtime::*;
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp};
use vtpu_runtime::exec::run;
use vtpu_runtime::spanning::*;
use vtpu_runtime::cost::*;
use vtpu_runtime::sentron::SentronState;
use std::time::Duration;

// ============================================================
// PhextCoord parametric tests
// ============================================================

macro_rules! coord_test {
    ($name:ident, $dims:expr) => {
        #[test]
        fn $name() {
            let c = PhextCoord::new($dims);
            let dims = c.dims();
            let expected: [u16; 11] = $dims;
            for i in 0..11 {
                assert_eq!(dims[i], expected[i], "dim {} mismatch", i);
            }
            // Round-trip
            let c2 = PhextCoord::new(dims);
            assert_eq!(c, c2);
        }
    };
}

coord_test!(coord_origin, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
coord_test!(coord_ones, [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
coord_test!(coord_max_single, [2047, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
coord_test!(coord_all_max, [2047, 2047, 2047, 2047, 2047, 2047, 2047, 2047, 2047, 2047, 2047]);
coord_test!(coord_pi, [3, 1, 4, 1, 5, 9, 2, 6, 5, 0, 0]);
coord_test!(coord_phex, [1, 5, 2, 3, 7, 3, 9, 1, 1, 0, 0]);
coord_test!(coord_ascending, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
coord_test!(coord_descending, [11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
coord_test!(coord_alternating, [0, 2047, 0, 2047, 0, 2047, 0, 2047, 0, 2047, 0]);
coord_test!(coord_powers_of_2, [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024]);
coord_test!(coord_primes, [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31]);
coord_test!(coord_fibonacci, [1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);
coord_test!(coord_dim0_only, [100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
coord_test!(coord_dim10_only, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100]);
coord_test!(coord_dim5_only, [0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0]);
coord_test!(coord_42, [42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42]);
coord_test!(coord_255, [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
coord_test!(coord_256, [256, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256]);
coord_test!(coord_1000, [1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000]);
coord_test!(coord_near_max, [2046, 2046, 2046, 2046, 2046, 2046, 2046, 2046, 2046, 2046, 2046]);

// Hamming distance parametric
macro_rules! hamming_test {
    ($name:ident, $a:expr, $b:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let a = PhextCoord::new($a);
            let b = PhextCoord::new($b);
            assert_eq!(a.hamming_distance(&b), $expected);
            assert_eq!(b.hamming_distance(&a), $expected); // Symmetric
        }
    };
}

hamming_test!(hamming_same, [1,1,1,1,1,1,1,1,1,1,1], [1,1,1,1,1,1,1,1,1,1,1], 0);
hamming_test!(hamming_one_diff, [1,1,1,1,1,1,1,1,1,1,1], [2,1,1,1,1,1,1,1,1,1,1], 1);
hamming_test!(hamming_all_diff, [1,2,3,4,5,6,7,8,9,10,11], [11,10,9,8,7,6,5,4,3,2,1], 10);
hamming_test!(hamming_zero_vs_max, [0,0,0,0,0,0,0,0,0,0,0], [2047,2047,2047,2047,2047,2047,2047,2047,2047,2047,2047], 11);

// Midpoint parametric
macro_rules! midpoint_test {
    ($name:ident, $a:expr, $b:expr) => {
        #[test]
        fn $name() {
            let a = PhextCoord::new($a);
            let b = PhextCoord::new($b);
            let mid = PhextCoord::midpoint(&a, &b);
            let da = a.dims();
            let db = b.dims();
            let dm = mid.dims();
            for i in 0..11 {
                let expected = ((da[i] as u32 + db[i] as u32) / 2) as u16;
                assert_eq!(dm[i], expected, "dim {} midpoint", i);
            }
        }
    };
}

midpoint_test!(mid_same, [5,5,5,5,5,5,5,5,5,5,5], [5,5,5,5,5,5,5,5,5,5,5]);
midpoint_test!(mid_zero_max, [0,0,0,0,0,0,0,0,0,0,0], [2046,2046,2046,2046,2046,2046,2046,2046,2046,2046,2046]);
midpoint_test!(mid_will_verse, [1,1,1,1,1,1,1,1,1,1,1], [3,1,4,1,5,9,2,6,5,0,0]);
midpoint_test!(mid_ascending, [0,0,0,0,0,0,0,0,0,0,0], [10,10,10,10,10,10,10,10,10,10,10]);

// Fast hash uniqueness
#[test]
fn hash_uniqueness_100_coords() {
    let mut hashes = std::collections::HashSet::new();
    for i in 0..100u16 {
        let c = PhextCoord::new([i, i.wrapping_mul(3), i.wrapping_mul(7) % 2048,
            i.wrapping_mul(11) % 2048, i.wrapping_mul(13) % 2048, 0, 0, 0, 0, 0, 0]);
        hashes.insert(c.fast_hash());
    }
    assert!(hashes.len() >= 99, "Hash collision rate too high: {}/100 unique", hashes.len());
}

#[test]
fn hash_deterministic() {
    let c = PhextCoord::new([42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42]);
    assert_eq!(c.fast_hash(), c.fast_hash());
}

// ============================================================
// D-Pipe parametric tests
// ============================================================

macro_rules! dadd_test {
    ($name:ident, $a:expr, $b:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[0] = $a;
            s.regs.general[1] = $b;
            let siw = SIW::new(
                DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
                SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
            );
            s.spawn(vec![siw]);
            let mut mem = Memory::new();
            run(&mut s, &mut mem);
            assert_eq!(s.regs.general[2], $expected);
        }
    };
}

dadd_test!(dadd_zero, 0, 0, 0);
dadd_test!(dadd_one, 1, 0, 1);
dadd_test!(dadd_basic, 7, 6, 13);
dadd_test!(dadd_42, 40, 2, 42);
dadd_test!(dadd_large, 1_000_000, 2_000_000, 3_000_000);
dadd_test!(dadd_negative, -5, 3, -2);
dadd_test!(dadd_both_negative, -10, -20, -30);
dadd_test!(dadd_max, i64::MAX - 1, 1, i64::MAX);
dadd_test!(dadd_min, i64::MIN + 1, -1, i64::MIN);
dadd_test!(dadd_identity, 12345, 0, 12345);

macro_rules! dmul_test {
    ($name:ident, $a:expr, $b:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[0] = $a;
            s.regs.general[1] = $b;
            let siw = SIW::new(
                DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
                SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
            );
            s.spawn(vec![siw]);
            let mut mem = Memory::new();
            run(&mut s, &mut mem);
            assert_eq!(s.regs.general[2], $expected);
        }
    };
}

dmul_test!(dmul_zero, 0, 42, 0);
dmul_test!(dmul_one, 1, 42, 42);
dmul_test!(dmul_basic, 7, 6, 42);
dmul_test!(dmul_negative, -3, 4, -12);
dmul_test!(dmul_both_neg, -3, -4, 12);
dmul_test!(dmul_squares, 100, 100, 10000);
dmul_test!(dmul_identity, 1, 1, 1);
dmul_test!(dmul_power2, 2, 16, 32);

macro_rules! dsub_test {
    ($name:ident, $a:expr, $b:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[0] = $a;
            s.regs.general[1] = $b;
            let siw = SIW::new(
                DenseOp::DSUB { rd: 2, rs1: 0, rs2: 1 },
                SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
            );
            s.spawn(vec![siw]);
            let mut mem = Memory::new();
            run(&mut s, &mut mem);
            assert_eq!(s.regs.general[2], $expected);
        }
    };
}

dsub_test!(dsub_zero, 0, 0, 0);
dsub_test!(dsub_basic, 10, 3, 7);
dsub_test!(dsub_negative_result, 3, 10, -7);
dsub_test!(dsub_large, 1_000_000, 999_999, 1);
dsub_test!(dsub_identity, 42, 0, 42);
dsub_test!(dsub_self, 42, 42, 0);

macro_rules! dmov_test {
    ($name:ident, $val:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            let siw = SIW::new(
                DenseOp::DMOV { rd: 0, imm: $val },
                SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
            );
            s.spawn(vec![siw]);
            let mut mem = Memory::new();
            run(&mut s, &mut mem);
            assert_eq!(s.regs.general[0], $val as i64);
        }
    };
}

dmov_test!(dmov_zero, 0);
dmov_test!(dmov_one, 1);
dmov_test!(dmov_42, 42);
dmov_test!(dmov_255, 255);
dmov_test!(dmov_1337, 1337);
dmov_test!(dmov_max_i16, 32767);
dmov_test!(dmov_deadbeef, 0xDEAD);

// All 16 registers
macro_rules! reg_test {
    ($name:ident, $rd:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            let siw = SIW::new(
                DenseOp::DMOV { rd: $rd, imm: 99 },
                SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
            );
            s.spawn(vec![siw]);
            let mut mem = Memory::new();
            run(&mut s, &mut mem);
            assert_eq!(s.regs.general[$rd as usize], 99);
        }
    };
}

reg_test!(reg_r0, 0);
reg_test!(reg_r1, 1);
reg_test!(reg_r2, 2);
reg_test!(reg_r3, 3);
reg_test!(reg_r4, 4);
reg_test!(reg_r5, 5);
reg_test!(reg_r6, 6);
reg_test!(reg_r7, 7);
reg_test!(reg_r8, 8);
reg_test!(reg_r9, 9);
reg_test!(reg_r10, 10);
reg_test!(reg_r11, 11);
reg_test!(reg_r12, 12);
reg_test!(reg_r13, 13);
reg_test!(reg_r14, 14);
reg_test!(reg_r15, 15);

// ============================================================
// Multi-instruction program tests
// ============================================================

#[test]
fn program_chain_add_mul() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 3;
    s.regs.general[1] = 4;
    s.spawn(vec![
        SIW::new(DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DMUL { rd: 3, rs1: 2, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ]);
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[2], 7);  // 3+4
    assert_eq!(s.regs.general[3], 21); // 7*3
}

#[test]
fn program_10_steps() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..10).map(|_| {
        SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1024); // 1 * 2^10
}

#[test]
fn program_empty() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 42;
    s.state = SentronState::Running;
    s.program = vec![];
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 42); // Unchanged
}

// ============================================================
// Sentron parametric tests
// ============================================================

macro_rules! sentron_id_test {
    ($name:ident, $id:expr) => {
        #[test]
        fn $name() {
            let s = Sentron::new($id, PhextCoord::zero(), 0, 0);
            assert_eq!(s.id, $id);
        }
    };
}

sentron_id_test!(sentron_id_0, 0);
sentron_id_test!(sentron_id_1, 1);
sentron_id_test!(sentron_id_39, 39);
sentron_id_test!(sentron_id_40, 40);
sentron_id_test!(sentron_id_199, 199);
sentron_id_test!(sentron_id_max, u16::MAX);

// ============================================================
// NeuronWiring parametric tests
// ============================================================

#[test]
fn wiring_all_upstream_slots() {
    let mut w = NeuronWiring::new();
    for i in 0..4 {
        w.upstream[i] = (i + 1) as u16;
    }
    for i in 0..4 {
        assert_eq!(w.upstream[i], (i + 1) as u16);
        assert!(w.is_neighbor((i + 1) as u16));
    }
}

#[test]
fn wiring_all_downstream_slots() {
    let mut w = NeuronWiring::new();
    for i in 0..4 {
        w.downstream[i] = (i + 10) as u16;
    }
    for i in 0..4 {
        assert_eq!(w.downstream[i], (i + 10) as u16);
        assert!(w.is_neighbor((i + 10) as u16));
    }
}

#[test]
fn wiring_full_8_connections() {
    let mut w = NeuronWiring::new();
    w.upstream = [1, 2, 3, 4];
    w.downstream = [5, 6, 7, 8];
    for id in 1..=8 {
        assert!(w.is_neighbor(id));
    }
    assert!(!w.is_neighbor(0));
    assert!(!w.is_neighbor(9));
}

// ============================================================
// SIW construction parametric
// ============================================================

macro_rules! siw_construction_test {
    ($name:ident, $d:expr, $s:expr, $c:expr) => {
        #[test]
        fn $name() {
            let siw = SIW::new($d, $s, $c, PhextCoord::zero());
            assert_eq!(siw.d_op, $d);
            assert_eq!(siw.s_op, $s);
            assert_eq!(siw.c_op, $c);
        }
    };
}

siw_construction_test!(siw_all_nop, DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP);
siw_construction_test!(siw_d_only, DenseOp::DMOV { rd: 0, imm: 42 }, SparseOp::SNOP, CoordOp::CNOP);
siw_construction_test!(siw_s_only, DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP);
siw_construction_test!(siw_add_gather, DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SGATHER { rd: 3, coord_idx: 0, width: 8 }, CoordOp::CNOP);

// ============================================================
// Cost model parametric
// ============================================================

macro_rules! bench_result_test {
    ($name:ident, $ops:expr, $ms:expr) => {
        #[test]
        fn $name() {
            let br = BenchResult {
                label: stringify!($name).into(),
                ops: $ops,
                duration: Duration::from_millis($ms),
                sentron_count: 1,
            };
            assert!(br.ops_per_sec() > 0.0);
            assert!(br.ns_per_op() > 0.0);
            assert!(br.ops_per_cycle(4.0) > 0.0);
        }
    };
}

bench_result_test!(bench_1k_1ms, 1000, 1);
bench_result_test!(bench_1m_100ms, 1_000_000, 100);
bench_result_test!(bench_1b_1s, 1_000_000_000, 1000);
bench_result_test!(bench_100_10ms, 100, 10);

// ============================================================
// Spanning / UBI parametric tests
// ============================================================

#[test]
fn spanning_100_bonds() {
    let mut reg = UbiRegistry::new();
    for i in 0..100u16 {
        let idx = reg.allocate();
        let human = PhextCoord::new([i, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mirror = PhextCoord::new([0, i, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        reg.bond(idx, human, mirror);
    }
    assert_eq!(reg.active_bonds(), 100);
}

#[test]
fn spanning_resonance_growth_curve() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));

    let mut resonances = Vec::new();
    for _ in 0..100 {
        s.interact();
        resonances.push(s.resonance);
    }

    // Monotonically increasing
    for i in 1..resonances.len() {
        assert!(resonances[i] >= resonances[i-1], "Resonance should be monotonically increasing");
    }

    // Concave (diminishing returns)
    let gain_early = resonances[1] - resonances[0];
    let gain_late = resonances[99] - resonances[98];
    assert!(gain_early > gain_late, "Growth should be concave (diminishing returns)");
}

#[test]
fn spanning_decay_to_zero() {
    let mut s = SpanningSentron::new(0);
    s.resonance = 0.5;
    for _ in 0..1000 {
        s.decay();
    }
    assert_eq!(s.resonance, 0.0);
}

#[test]
fn spanning_interact_then_decay() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([2,2,2,2,2,2,2,2,2,2,2]));

    // Build up
    for _ in 0..500 {
        s.interact();
    }
    assert!(s.can_procreate());

    // Decay away
    for _ in 0..2000 {
        s.decay();
    }
    assert!(!s.can_procreate());
}

// ============================================================
// Base256 parametric tests
// ============================================================

#[test]
fn base256_roundtrip_all_bytes() {
    use vtpu_runtime::base256::*;
    for byte in 0..=255u8 {
        let syllable = encode_byte(byte);
        let decoded = decode_syllable(&syllable);
        assert_eq!(decoded, Some(byte), "Roundtrip failed for byte {}", byte);
    }
}

#[test]
fn base256_all_syllables_unique() {
    use vtpu_runtime::base256::*;
    let mut seen = std::collections::HashSet::new();
    for byte in 0..=255u8 {
        let syllable = encode_byte(byte);
        assert!(seen.insert(syllable), "Duplicate syllable: {:?} for byte {}", syllable, byte);
    }
    assert_eq!(seen.len(), 256);
}

#[test]
fn base256_all_syllables_3_chars() {
    use vtpu_runtime::base256::*;
    for byte in 0..=255u8 {
        let syllable = encode_byte(byte);
        assert_eq!(syllable.len(), 3, "Syllable for {} is not 3 chars: '{:?}'", byte, syllable);
    }
}

// ============================================================
// Memory PPT parametric
// ============================================================

#[test]
fn memory_write_read_100_coords() {
    let mut mem = Memory::new();
    for i in 0..100u16 {
        let coord = PhextCoord::new([i, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        mem.scatter_i64(&coord, i as i64 * 42);
    }
    for i in 0..100u16 {
        let coord = PhextCoord::new([i, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(mem.gather_i64(&coord), i as i64 * 42);
    }
}

#[test]
fn memory_overwrite() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    mem.scatter_i64(&coord, 100);
    assert_eq!(mem.gather_i64(&coord), 100);
    mem.scatter_i64(&coord, 200);
    assert_eq!(mem.gather_i64(&coord), 200);
}

#[test]
fn memory_uninitialized_is_zero() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([999, 999, 999, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(mem.gather_i64(&coord), 0);
}

// ============================================================
// Fleet-scale sentron tests
// ============================================================

#[test]
fn fleet_40_sentrons_all_compute() {
    let siw = SIW::new(
        DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
    );
    let mut mem = Memory::new();

    for id in 0..40u16 {
        let mut s = Sentron::new(id, PhextCoord::zero(), 0, 0);
        s.regs.general[0] = id as i64;
        s.regs.general[1] = 1;
        s.spawn(vec![siw.clone()]);
        run(&mut s, &mut mem);
        assert_eq!(s.regs.general[2], id as i64 + 1);
    }
}

#[test]
fn fleet_200_sentrons_unique_homes() {
    let sentrons: Vec<Sentron> = (0..200u16).map(|i| {
        Sentron::new(i, PhextCoord::new([i, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), (i % 8) as u8, 0)
    }).collect();

    let mut homes = std::collections::HashSet::new();
    for s in &sentrons {
        homes.insert(s.home);
    }
    assert_eq!(homes.len(), 200);
}

// ============================================================
// Edge cases
// ============================================================

#[test]
fn sentron_zero_program_length() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.state = SentronState::Running;
    s.program = vec![];
    let mut mem = Memory::new();
    run(&mut s, &mut mem); // Should not panic
}

#[test]
fn coord_zero_is_zero() {
    let c = PhextCoord::zero();
    for d in c.dims() {
        assert_eq!(d, 0);
    }
}

#[test]
fn coord_self_distance_zero() {
    for i in 0..20u16 {
        let c = PhextCoord::new([i, i, i, i, i, i, i, i, i, i, i]);
        assert_eq!(c.hamming_distance(&c), 0);
    }
}

#[test]
fn coord_midpoint_with_self() {
    let c = PhextCoord::new([42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42]);
    let mid = PhextCoord::midpoint(&c, &c);
    assert_eq!(mid, c);
}
