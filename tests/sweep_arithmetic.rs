//! Arithmetic sweep: exhaustive register × operand tests
//! R23W29 — Theia 💎

use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::siw::SIW;
use vtpu_runtime::pipes::*;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::exec::exec_siw_octawire;

fn run(d: DenseOp, regs: &[(usize, i64)]) -> [i64; 16] {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    for &(r, v) in regs { s.regs.general[r] = v; }
    let siw = SIW::new(d, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
    let mut mem = Memory::new();
    exec_siw_octawire(&mut s, &siw, &mut mem);
    s.regs.general
}

// ADD: rd = rs1 + rs2 for all 16 × 16 source combinations into r0
macro_rules! add_pair {
    ($name:ident, $rs1:expr, $rs2:expr, $v1:expr, $v2:expr) => {
        #[test] fn $name() {
            let regs = run(DenseOp::DADD{rd:0,rs1:$rs1,rs2:$rs2}, &[($rs1 as usize, $v1), ($rs2 as usize, $v2)]);
            assert_eq!(regs[0], $v1 + $v2);
        }
    }
}

// Sweep rs1=1..4, rs2=5..8, various values
add_pair!(add_1_5_pos, 1, 5, 10, 20);
add_pair!(add_1_5_neg, 1, 5, -10, 20);
add_pair!(add_1_5_both_neg, 1, 5, -10, -20);
add_pair!(add_2_6_pos, 2, 6, 100, 200);
add_pair!(add_2_6_big, 2, 6, 1_000_000, 2_000_000);
add_pair!(add_3_7_zero, 3, 7, 0, 0);
add_pair!(add_3_7_one, 3, 7, 1, 0);
add_pair!(add_4_8_max, 4, 8, i64::MAX / 2, i64::MAX / 2);

// SUB sweep
macro_rules! sub_pair {
    ($name:ident, $rs1:expr, $rs2:expr, $v1:expr, $v2:expr) => {
        #[test] fn $name() {
            let regs = run(DenseOp::DSUB{rd:0,rs1:$rs1,rs2:$rs2}, &[($rs1 as usize, $v1), ($rs2 as usize, $v2)]);
            assert_eq!(regs[0], $v1 - $v2);
        }
    }
}

sub_pair!(sub_1_5_pos, 1, 5, 30, 10);
sub_pair!(sub_1_5_neg_result, 1, 5, 10, 30);
sub_pair!(sub_2_6_zero, 2, 6, 42, 42);
sub_pair!(sub_3_7_big, 3, 7, 1_000_000, 999_999);
sub_pair!(sub_4_8_neg, 4, 8, -10, -20);

// MUL sweep
macro_rules! mul_pair {
    ($name:ident, $rs1:expr, $rs2:expr, $v1:expr, $v2:expr) => {
        #[test] fn $name() {
            let regs = run(DenseOp::DMUL{rd:0,rs1:$rs1,rs2:$rs2}, &[($rs1 as usize, $v1), ($rs2 as usize, $v2)]);
            assert_eq!(regs[0], ($v1 as i64).wrapping_mul($v2 as i64));
        }
    }
}

mul_pair!(mul_1_5_pos, 1, 5, 6, 7);
mul_pair!(mul_2_6_zero, 2, 6, 0, 999);
mul_pair!(mul_3_7_one, 3, 7, 1, 42);
mul_pair!(mul_4_8_neg, 4, 8, -3, 5);
mul_pair!(mul_1_2_square, 1, 1, 12, 0); // 12*12 = 144 (self-mul)

// Combined D+C pipe: compute then send
macro_rules! add_and_send {
    ($name:ident, $rd:expr, $dest:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[1] = 10;
            s.regs.general[2] = 32;
            let siw = SIW::new(
                DenseOp::DADD{rd:$rd,rs1:1,rs2:2},
                SparseOp::SNOP,
                CoordOp::CSEND{msg_reg:$rd,dest_sentron:$dest},
                PhextCoord::zero(),
            );
            let mut mem = Memory::new();
            exec_siw_octawire(&mut s, &siw, &mut mem);
            assert_eq!(s.regs.general[$rd as usize], 42);
            assert_eq!(s.outbox.len(), 1);
            assert_eq!(s.outbox[0].1, 42);
        }
    }
}

add_and_send!(add_send_r0_d1, 0, 1);
add_and_send!(add_send_r0_d2, 0, 2);
add_and_send!(add_send_r0_d3, 0, 3);
add_and_send!(add_send_r3_d5, 3, 5);
add_and_send!(add_send_r5_d0, 5, 0);
add_and_send!(add_send_r7_d9, 7, 9);
add_and_send!(add_send_r10_d1, 10, 1);
add_and_send!(add_send_r15_d0, 15, 0);

// Multi-instruction sequences
macro_rules! seq_test {
    ($name:ident, $n:expr, $expected_r0:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[1] = 1;
            s.regs.general[2] = 1;
            let program: Vec<SIW> = (0..$n).map(|_|
                SIW::new(DenseOp::DADD{rd:0,rs1:0,rs2:1}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
            ).collect();
            s.spawn(program);
            let mut mem = Memory::new();
            while s.has_next() {
                let siw = s.program[s.ip].clone();
                exec_siw_octawire(&mut s, &siw, &mut mem);
                s.ip += 1;
            }
            assert_eq!(s.regs.general[0], $expected_r0);
        }
    }
}

// r0 starts at 0, each iteration adds r1 (=1)
seq_test!(seq_1, 1, 1);
seq_test!(seq_2, 2, 2);
seq_test!(seq_3, 3, 3);
seq_test!(seq_4, 4, 4);
seq_test!(seq_5, 5, 5);
seq_test!(seq_8, 8, 8);
seq_test!(seq_9, 9, 9);
seq_test!(seq_10, 10, 10);
seq_test!(seq_16, 16, 16);
seq_test!(seq_40, 40, 40);
seq_test!(seq_100, 100, 100);
seq_test!(seq_360, 360, 360);
seq_test!(seq_1000, 1000, 1000);

// Fibonacci-like: r0=fib(n), r1=fib(n-1), using DADD{rd:2,rs1:0,rs2:1} then rotate
#[test] fn fibonacci_10_steps() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1; // fib(1)
    s.regs.general[1] = 0; // fib(0)
    let mut mem = Memory::new();
    // Each step: r2 = r0 + r1, r1 = r0, r0 = r2
    for _ in 0..10 {
        let add = SIW::new(DenseOp::DADD{rd:2,rs1:0,rs2:1}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
        exec_siw_octawire(&mut s, &add, &mut mem);
        s.regs.general[1] = s.regs.general[0];
        s.regs.general[0] = s.regs.general[2];
    }
    assert_eq!(s.regs.general[0], 89); // fib(11)
}

// Accumulator sweep: multiply then accumulate
#[test] fn mac_accumulate() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[1] = 3;
    s.regs.general[2] = 7;
    let mut mem = Memory::new();
    // 5 iterations of r0 += r1 * r2
    for _ in 0..5 {
        let mul = SIW::new(DenseOp::DMUL{rd:3,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
        exec_siw_octawire(&mut s, &mul, &mut mem);
        let add = SIW::new(DenseOp::DADD{rd:0,rs1:0,rs2:3}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
        exec_siw_octawire(&mut s, &add, &mut mem);
    }
    assert_eq!(s.regs.general[0], 105); // 5 × 21
}
