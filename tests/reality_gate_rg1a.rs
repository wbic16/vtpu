//! RG-1A: independent D-pipe semantic model and differential test corpus.
//!
//! This is a *provisional* spec for DRED/HDC: it characterizes the current
//! OctaWire semantics, not an architectural decision by the maintainer.
//! Arithmetic, comparison and selection follow the explicit Reality Gate
//! contract. Once the ambiguous reductions/HDC operations are specified,
//! change this model first and require the runtime to agree.
//!
//! Full RG-1 (100K mixed D/S/C streams, scheduling, packing, memory effects)
//! remains a separate gate. No external random/test crates are needed.

use vtpu_runtime::{
    exec_run, exec_run_batched, exec_siw_octawire, CoordOp, DenseOp, Memory, PhextCoord,
    ReductionOp, Sentron, SparseOp, SIW,
};

const SEED: u64 = 0x13A2_2026_10D0_0001;
const STREAMS: usize = 10_000;
const ACTIVE_D_OPCODES: usize = 16;

struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn reg(&mut self) -> u8 {
        (self.next() % 16) as u8
    }
}

fn reduction(mode: u64) -> ReductionOp {
    match mode % 6 {
        0 => ReductionOp::Sum,
        1 => ReductionOp::Max,
        2 => ReductionOp::Min,
        3 => ReductionOp::And,
        4 => ReductionOp::Or,
        _ => ReductionOp::Xor,
    }
}

fn make_op(rng: &mut XorShift64, kind: usize) -> DenseOp {
    let rd = rng.reg();
    let a = rng.reg();
    let b = rng.reg();
    match kind {
        0 => DenseOp::DFMA {
            rd,
            rs1: a,
            rs2: b,
            rs3: rng.reg(),
        },
        1 => DenseOp::DADD { rd, rs1: a, rs2: b },
        2 => DenseOp::DSUB { rd, rs1: a, rs2: b },
        3 => DenseOp::DMUL { rd, rs1: a, rs2: b },
        4 => DenseOp::DCMP { rd, rs1: a, rs2: b },
        5 => DenseOp::DRED {
            rd,
            rs1: a,
            op: reduction(rng.next()),
        },
        6 => DenseOp::DSEL {
            rd,
            rs1: a,
            rs2: b,
            flags: rng.reg(),
        },
        7 => DenseOp::DMOV {
            rd,
            imm: rng.next() as i64,
        },
        8 => DenseOp::DHDENC {
            rd,
            rs: a,
            width: rng.next() as u16,
        },
        9 => DenseOp::DHDBIND { rd, rs1: a, rs2: b },
        10 => DenseOp::DHDBUND { rd, rs1: a, rs2: b },
        11 => DenseOp::DHDPERM {
            rd,
            rs: a,
            k: rng.next() as u16,
        },
        12 => DenseOp::DHDSIM { rd, rs1: a, rs2: b },
        13 => DenseOp::DTERNARY {
            rd,
            rs1: a,
            trit_reg: b,
        },
        14 => DenseOp::DTPOP { rd, rs: a },
        15 => DenseOp::DTACC {
            rd,
            rs1: a,
            trit_reg: b,
        },
        _ => unreachable!(),
    }
}

fn dense_siw(op: DenseOp) -> SIW {
    SIW::new(op, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
}

fn rotate(x: i64, shift: u32) -> i64 {
    let shift = shift % 64;
    if shift == 0 {
        return x;
    }
    let word = x as u64;
    ((word << shift) | (word >> (64 - shift))) as i64
}

/// Interpret packed trits with unbounded intermediate precision, then reduce
/// modulo 2^64. This is independent from the runtime's loop with i64 additions.
fn dot_trits(activation: i64, packed: i64) -> i64 {
    let mut coefficient = 0i128;
    for i in 0..32 {
        match ((packed as u64) >> (i * 2)) & 3 {
            1 => coefficient += 1,
            2 => coefficient -= 1,
            _ => {} // 00 or reserved 11 has weight 0 in the current runtime
        }
    }
    (coefficient * (activation as i128)) as i64
}

/// Independent reference. Every op reads a pre-instruction register snapshot;
/// aliases of rd/rs*/flags must therefore behave as a single ordered step.
fn model_step(regs: &mut [i64; 16], op: DenseOp) {
    let before = *regs;
    let at = |r: u8| before[r as usize];
    let (dst, result) = match op {
        DenseOp::DNOP => return,
        DenseOp::DFMA { rd, rs1, rs2, rs3 } => (
            rd,
            ((at(rs1) as i128 * at(rs2) as i128) + at(rs3) as i128) as i64,
        ),
        DenseOp::DADD { rd, rs1, rs2 } => (rd, (at(rs1) as i128 + at(rs2) as i128) as i64),
        DenseOp::DSUB { rd, rs1, rs2 } => (rd, (at(rs1) as i128 - at(rs2) as i128) as i64),
        DenseOp::DMUL { rd, rs1, rs2 } => (rd, (at(rs1) as i128 * at(rs2) as i128) as i64),
        DenseOp::DCMP { rd, rs1, rs2 } => (
            rd,
            if at(rs1) < at(rs2) {
                -1
            } else if at(rs1) > at(rs2) {
                1
            } else {
                0
            },
        ),
        DenseOp::DSEL {
            rd,
            rs1,
            rs2,
            flags,
        } => (rd, if at(flags) != 0 { at(rs1) } else { at(rs2) }),
        DenseOp::DMOV { rd, imm } => (rd, imm),
        DenseOp::DRED { rd, rs1, op } => {
            let val = at(rs1);
            let out = match op {
                ReductionOp::Sum | ReductionOp::Max | ReductionOp::Min => val,
                ReductionOp::And => val & 255,
                ReductionOp::Or => val | 1,
                ReductionOp::Xor => val ^ rotate(val, 64 - 16),
            };
            (rd, out)
        }
        DenseOp::DHDENC { rd, rs, width } => (rd, rotate(at(rs), width as u32)),
        DenseOp::DHDBIND { rd, rs1, rs2 } => (rd, at(rs1) ^ at(rs2)),
        DenseOp::DHDBUND { rd, rs1, rs2 } => (rd, (at(rs1) as i128 + at(rs2) as i128) as i64),
        DenseOp::DHDPERM { rd, rs, k } => (rd, rotate(at(rs), k as u32)),
        DenseOp::DHDSIM { rd, rs1, rs2 } => {
            let matching = (!(at(rs1) ^ at(rs2))).count_ones();
            (rd, matching as i64)
        }
        DenseOp::DTERNARY { rd, rs1, trit_reg } => (rd, dot_trits(at(rs1), at(trit_reg))),
        DenseOp::DTPOP { rd, rs } => {
            let mut n = 0;
            for i in 0..32 {
                if ((at(rs) as u64) >> (2 * i)) & 3 != 0 {
                    n += 1;
                }
            }
            (rd, n)
        }
        DenseOp::DTACC { rd, rs1, trit_reg } => (
            rd,
            (at(rd) as i128 + dot_trits(at(rs1), at(trit_reg)) as i128) as i64,
        ),
    };
    regs[dst as usize] = result;
}

fn assert_registers_equal(got: &[i64; 16], expected: &[i64; 16], seed: u64, prefix: usize) {
    assert_eq!(
        got, expected,
        "RG-1A divergence: seed={seed:#x}, SIW prefix={prefix}"
    );
}

#[test]
fn all_d_pipe_opcodes_differential_10k_seeded_streams() {
    let mut coverage = [0u64; ACTIVE_D_OPCODES];
    let mut reduce_coverage = [0u64; 6];
    let mut mem = Memory::with_capacity(8); // D-only; avoid 16 MiB allocation per stream

    for stream_id in 0..STREAMS {
        let seed = SEED ^ (stream_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut rng = XorShift64::new(seed);
        let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
        for reg in &mut sentron.regs.general {
            *reg = match rng.next() % 12 {
                0 => i64::MIN,
                1 => i64::MAX,
                2 => 0,
                3 => -1,
                _ => (rng.next() as i64) >> 40,
            };
        }
        let mut reference = sentron.regs.general;
        let len = 1 + (rng.next() as usize % 64);

        for idx in 0..len {
            let kind = (stream_id + idx) % ACTIVE_D_OPCODES;
            let op = make_op(&mut rng, kind);
            coverage[kind] += 1;
            if let DenseOp::DRED { op, .. } = op {
                let bucket = match op {
                    ReductionOp::Sum => 0,
                    ReductionOp::Max => 1,
                    ReductionOp::Min => 2,
                    ReductionOp::And => 3,
                    ReductionOp::Or => 4,
                    ReductionOp::Xor => 5,
                };
                reduce_coverage[bucket] += 1;
            }
            model_step(&mut reference, op);
            let retired = exec_siw_octawire(&mut sentron, &dense_siw(op), &mut mem);
            assert_eq!(retired, 1, "seed={seed:#x}, prefix={idx}");
            assert_registers_equal(&sentron.regs.general, &reference, seed, idx + 1);
        }
        assert_eq!(sentron.regs.status, 0);
        assert_eq!(sentron.regs.phext, [PhextCoord::zero(); 8]);
        assert_eq!(sentron.regs.message, [[0u8; 32]; 4]);
        assert!(sentron.inbox.is_empty());
        assert!(sentron.outbox.is_empty());
        assert_eq!(sentron.fence_gen, 0);
    }

    assert!(
        coverage.iter().all(|&n| n > 0),
        "missing D opcode: {coverage:?}"
    );
    assert!(
        reduce_coverage.iter().all(|&n| n > 0),
        "missing DRED variant: {reduce_coverage:?}"
    );
}

#[test]
fn dsel_uses_register_value_and_dcmp_avoids_overflow() {
    let mut mem = Memory::with_capacity(8);
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.regs.general[1] = 99;
    s.regs.general[2] = -99;
    s.regs.general[3] = 0;
    exec_siw_octawire(
        &mut s,
        &dense_siw(DenseOp::DSEL {
            rd: 4,
            rs1: 1,
            rs2: 2,
            flags: 3,
        }),
        &mut mem,
    );
    assert_eq!(s.regs.general[4], -99, "nonzero flags index is not truthy");
    exec_siw_octawire(
        &mut s,
        &dense_siw(DenseOp::DSEL {
            rd: 4,
            rs1: 1,
            rs2: 2,
            flags: 0,
        }),
        &mut mem,
    );
    assert_eq!(
        s.regs.general[4], 99,
        "register zero can hold a true condition"
    );

    for &(a, b, expected) in &[
        (i64::MIN, i64::MAX, -1),
        (i64::MAX, i64::MIN, 1),
        (i64::MIN, i64::MIN, 0),
        (i64::MAX, i64::MAX, 0),
    ] {
        s.regs.general[1] = a;
        s.regs.general[2] = b;
        exec_siw_octawire(
            &mut s,
            &dense_siw(DenseOp::DCMP {
                rd: 4,
                rs1: 1,
                rs2: 2,
            }),
            &mut mem,
        );
        assert_eq!(s.regs.general[4], expected);
    }
}

#[test]
fn comparator_detects_a_deliberate_one_bit_mutation() {
    let mut reference = [0i64; 16];
    reference[1] = 7;
    reference[2] = 6;
    model_step(
        &mut reference,
        DenseOp::DADD {
            rd: 3,
            rs1: 1,
            rs2: 2,
        },
    );
    let mut mutated = reference;
    mutated[3] ^= 1;
    let rejected =
        std::panic::catch_unwind(|| assert_registers_equal(&mutated, &reference, SEED, 1));
    assert!(
        rejected.is_err(),
        "self-test: an incorrect output must be rejected"
    );
}

#[test]
fn run_and_batched_agree_on_small_ordered_d_stream() {
    let program = vec![
        dense_siw(DenseOp::DMOV { rd: 1, imm: 7 }),
        dense_siw(DenseOp::DMOV { rd: 2, imm: 6 }),
        dense_siw(DenseOp::DADD {
            rd: 3,
            rs1: 1,
            rs2: 2,
        }),
        dense_siw(DenseOp::DSEL {
            rd: 4,
            rs1: 3,
            rs2: 2,
            flags: 1,
        }),
        dense_siw(DenseOp::DCMP {
            rd: 5,
            rs1: 4,
            rs2: 2,
        }),
    ];
    let mut a = Sentron::new(0, PhextCoord::zero(), 0, 0);
    let mut b = Sentron::new(0, PhextCoord::zero(), 0, 0);
    a.spawn(program.clone());
    b.spawn(program);
    let mut am = Memory::with_capacity(8);
    let mut bm = Memory::with_capacity(8);
    let as_ = exec_run(&mut a, &mut am);
    let bs = exec_run_batched(&mut b, &mut bm);
    assert_eq!(as_.siws_retired, 5);
    assert_eq!(bs.siws_retired, 5);
    assert_eq!(as_.ops_retired, 5);
    assert_eq!(bs.ops_retired, 5);
    assert_eq!(a.regs.general, b.regs.general);
    assert_eq!(a.regs.phext, b.regs.phext);
    assert_eq!(a.regs.message, b.regs.message);
    assert_eq!(a.regs.status, b.regs.status);
    assert_eq!(a.outbox, b.outbox);
    assert_eq!(a.inbox, b.inbox);
    assert_eq!(a.regs.general[3..6], [13, 13, 1]);
}
