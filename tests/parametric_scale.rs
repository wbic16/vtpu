//! Parametric scale tests — bulk generation for 10× coverage
//! Auto-generated. Do not edit by hand.

use vtpu_runtime::*;
use vtpu_runtime::pipes::{DenseOp, SparseOp, CoordOp};
use vtpu_runtime::exec::run;
use vtpu_runtime::spanning::*;
use vtpu_runtime::base256::*;
use vtpu_runtime::cost::*;
use std::time::Duration;

fn dim_solo_check(dim: usize, val: u16) {
    // dim5 overflow bug FIXED in R23W29
    let mut dims = [0u16; 11];
    dims[dim] = val;
    let c = PhextCoord::new(dims);
    let got = c.dims();
    assert_eq!(got[dim], val, "dim {} val {} failed: got {}", dim, val, got[dim]);
}

fn reg_mov_check(reg: u8, val: i64) {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.spawn(vec![SIW::new(
        DenseOp::DMOV { rd: reg, imm: val },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
    )]);
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[reg as usize], val as i64, "r{} = {} failed", reg, val);
}

fn add_pair_check(rs1: u8, rs2: u8) {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[rs1 as usize] = (rs1 as i64 + 1) * 10;
    s.regs.general[rs2 as usize] = (rs2 as i64 + 1) * 3;
    let expected = s.regs.general[rs1 as usize] + s.regs.general[rs2 as usize];
    let rd: u8 = if rs1 != 15 && rs2 != 15 { 15 } else { 14 };
    s.spawn(vec![SIW::new(
        DenseOp::DADD { rd, rs1, rs2 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
    )]);
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    if rd != rs1 && rd != rs2 {
        assert_eq!(s.regs.general[rd as usize], expected, "r{} + r{} -> r{}", rs1, rs2, rd);
    }
}

fn sentron_core_check(id: u16, core_id: u8) {
    let s = Sentron::new(id, PhextCoord::new([id, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), core_id, 0);
    assert_eq!(s.id, id);
    assert_eq!(s.core_id, core_id);
    assert_eq!(s.home.dims()[0], id);
}

#[test] fn dim0_val_0() { dim_solo_check(0, 0); }
#[test] fn dim0_val_1() { dim_solo_check(0, 1); }
#[test] fn dim0_val_2() { dim_solo_check(0, 2); }
#[test] fn dim0_val_7() { dim_solo_check(0, 7); }
#[test] fn dim0_val_10() { dim_solo_check(0, 10); }
#[test] fn dim0_val_42() { dim_solo_check(0, 42); }
#[test] fn dim0_val_100() { dim_solo_check(0, 100); }
#[test] fn dim0_val_127() { dim_solo_check(0, 127); }
#[test] fn dim0_val_128() { dim_solo_check(0, 128); }
#[test] fn dim0_val_255() { dim_solo_check(0, 255); }
#[test] fn dim0_val_256() { dim_solo_check(0, 256); }
#[test] fn dim0_val_500() { dim_solo_check(0, 500); }
#[test] fn dim1_val_0() { dim_solo_check(1, 0); }
#[test] fn dim1_val_1() { dim_solo_check(1, 1); }
#[test] fn dim1_val_2() { dim_solo_check(1, 2); }
#[test] fn dim1_val_7() { dim_solo_check(1, 7); }
#[test] fn dim1_val_10() { dim_solo_check(1, 10); }
#[test] fn dim1_val_42() { dim_solo_check(1, 42); }
#[test] fn dim1_val_100() { dim_solo_check(1, 100); }
#[test] fn dim1_val_127() { dim_solo_check(1, 127); }
#[test] fn dim1_val_128() { dim_solo_check(1, 128); }
#[test] fn dim1_val_255() { dim_solo_check(1, 255); }
#[test] fn dim1_val_256() { dim_solo_check(1, 256); }
#[test] fn dim1_val_500() { dim_solo_check(1, 500); }
#[test] fn dim2_val_0() { dim_solo_check(2, 0); }
#[test] fn dim2_val_1() { dim_solo_check(2, 1); }
#[test] fn dim2_val_2() { dim_solo_check(2, 2); }
#[test] fn dim2_val_7() { dim_solo_check(2, 7); }
#[test] fn dim2_val_10() { dim_solo_check(2, 10); }
#[test] fn dim2_val_42() { dim_solo_check(2, 42); }
#[test] fn dim2_val_100() { dim_solo_check(2, 100); }
#[test] fn dim2_val_127() { dim_solo_check(2, 127); }
#[test] fn dim2_val_128() { dim_solo_check(2, 128); }
#[test] fn dim2_val_255() { dim_solo_check(2, 255); }
#[test] fn dim2_val_256() { dim_solo_check(2, 256); }
#[test] fn dim2_val_500() { dim_solo_check(2, 500); }
#[test] fn dim3_val_0() { dim_solo_check(3, 0); }
#[test] fn dim3_val_1() { dim_solo_check(3, 1); }
#[test] fn dim3_val_2() { dim_solo_check(3, 2); }
#[test] fn dim3_val_7() { dim_solo_check(3, 7); }
#[test] fn dim3_val_10() { dim_solo_check(3, 10); }
#[test] fn dim3_val_42() { dim_solo_check(3, 42); }
#[test] fn dim3_val_100() { dim_solo_check(3, 100); }
#[test] fn dim3_val_127() { dim_solo_check(3, 127); }
#[test] fn dim3_val_128() { dim_solo_check(3, 128); }
#[test] fn dim3_val_255() { dim_solo_check(3, 255); }
#[test] fn dim3_val_256() { dim_solo_check(3, 256); }
#[test] fn dim3_val_500() { dim_solo_check(3, 500); }
#[test] fn dim4_val_0() { dim_solo_check(4, 0); }
#[test] fn dim4_val_1() { dim_solo_check(4, 1); }
#[test] fn dim4_val_2() { dim_solo_check(4, 2); }
#[test] fn dim4_val_7() { dim_solo_check(4, 7); }
#[test] fn dim4_val_10() { dim_solo_check(4, 10); }
#[test] fn dim4_val_42() { dim_solo_check(4, 42); }
#[test] fn dim4_val_100() { dim_solo_check(4, 100); }
#[test] fn dim4_val_127() { dim_solo_check(4, 127); }
#[test] fn dim4_val_128() { dim_solo_check(4, 128); }
#[test] fn dim4_val_255() { dim_solo_check(4, 255); }
#[test] fn dim4_val_256() { dim_solo_check(4, 256); }
#[test] fn dim4_val_500() { dim_solo_check(4, 500); }
#[test] fn dim5_val_0() { dim_solo_check(5, 0); }
#[test] fn dim5_val_1() { dim_solo_check(5, 1); }
#[test] fn dim5_val_2() { dim_solo_check(5, 2); }
#[test] fn dim5_val_7() { dim_solo_check(5, 7); }
#[test] fn dim5_val_10() { dim_solo_check(5, 10); }
#[test] fn dim5_val_42() { dim_solo_check(5, 42); }
#[test] fn dim5_val_100() { dim_solo_check(5, 100); }
#[test] fn dim5_val_127() { dim_solo_check(5, 127); }
#[test] fn dim5_val_128() { dim_solo_check(5, 128); }
#[test] fn dim5_val_255() { dim_solo_check(5, 255); }
#[test] fn dim5_val_256() { dim_solo_check(5, 256); }
#[test] fn dim5_val_500() { dim_solo_check(5, 500); }
#[test] fn dim6_val_0() { dim_solo_check(6, 0); }
#[test] fn dim6_val_1() { dim_solo_check(6, 1); }
#[test] fn dim6_val_2() { dim_solo_check(6, 2); }
#[test] fn dim6_val_7() { dim_solo_check(6, 7); }
#[test] fn dim6_val_10() { dim_solo_check(6, 10); }
#[test] fn dim6_val_42() { dim_solo_check(6, 42); }
#[test] fn dim6_val_100() { dim_solo_check(6, 100); }
#[test] fn dim6_val_127() { dim_solo_check(6, 127); }
#[test] fn dim6_val_128() { dim_solo_check(6, 128); }
#[test] fn dim6_val_255() { dim_solo_check(6, 255); }
#[test] fn dim6_val_256() { dim_solo_check(6, 256); }
#[test] fn dim6_val_500() { dim_solo_check(6, 500); }
#[test] fn dim7_val_0() { dim_solo_check(7, 0); }
#[test] fn dim7_val_1() { dim_solo_check(7, 1); }
#[test] fn dim7_val_2() { dim_solo_check(7, 2); }
#[test] fn dim7_val_7() { dim_solo_check(7, 7); }
#[test] fn dim7_val_10() { dim_solo_check(7, 10); }
#[test] fn dim7_val_42() { dim_solo_check(7, 42); }
#[test] fn dim7_val_100() { dim_solo_check(7, 100); }
#[test] fn dim7_val_127() { dim_solo_check(7, 127); }
#[test] fn dim7_val_128() { dim_solo_check(7, 128); }
#[test] fn dim7_val_255() { dim_solo_check(7, 255); }
#[test] fn dim7_val_256() { dim_solo_check(7, 256); }
#[test] fn dim7_val_500() { dim_solo_check(7, 500); }
#[test] fn dim8_val_0() { dim_solo_check(8, 0); }
#[test] fn dim8_val_1() { dim_solo_check(8, 1); }
#[test] fn dim8_val_2() { dim_solo_check(8, 2); }
#[test] fn dim8_val_7() { dim_solo_check(8, 7); }
#[test] fn dim8_val_10() { dim_solo_check(8, 10); }
#[test] fn dim8_val_42() { dim_solo_check(8, 42); }
#[test] fn dim8_val_100() { dim_solo_check(8, 100); }
#[test] fn dim8_val_127() { dim_solo_check(8, 127); }
#[test] fn dim8_val_128() { dim_solo_check(8, 128); }
#[test] fn dim8_val_255() { dim_solo_check(8, 255); }
#[test] fn dim8_val_256() { dim_solo_check(8, 256); }
#[test] fn dim8_val_500() { dim_solo_check(8, 500); }
#[test] fn dim9_val_0() { dim_solo_check(9, 0); }
#[test] fn dim9_val_1() { dim_solo_check(9, 1); }
#[test] fn dim9_val_2() { dim_solo_check(9, 2); }
#[test] fn dim9_val_7() { dim_solo_check(9, 7); }
#[test] fn dim9_val_10() { dim_solo_check(9, 10); }
#[test] fn dim9_val_42() { dim_solo_check(9, 42); }
#[test] fn dim9_val_100() { dim_solo_check(9, 100); }
#[test] fn dim9_val_127() { dim_solo_check(9, 127); }
#[test] fn dim9_val_128() { dim_solo_check(9, 128); }
#[test] fn dim9_val_255() { dim_solo_check(9, 255); }
#[test] fn dim9_val_256() { dim_solo_check(9, 256); }
#[test] fn dim9_val_500() { dim_solo_check(9, 500); }
#[test] fn dim10_val_0() { dim_solo_check(10, 0); }
#[test] fn dim10_val_1() { dim_solo_check(10, 1); }
#[test] fn dim10_val_2() { dim_solo_check(10, 2); }
#[test] fn dim10_val_7() { dim_solo_check(10, 7); }
#[test] fn dim10_val_10() { dim_solo_check(10, 10); }
#[test] fn dim10_val_42() { dim_solo_check(10, 42); }
#[test] fn dim10_val_100() { dim_solo_check(10, 100); }
#[test] fn dim10_val_127() { dim_solo_check(10, 127); }
#[test] fn dim10_val_128() { dim_solo_check(10, 128); }
#[test] fn dim10_val_255() { dim_solo_check(10, 255); }
#[test] fn dim10_val_256() { dim_solo_check(10, 256); }
#[test] fn dim10_val_500() { dim_solo_check(10, 500); }
#[test] fn r0_mov_0() { reg_mov_check(0, 0); }
#[test] fn r0_mov_1() { reg_mov_check(0, 1); }
#[test] fn r0_mov_42() { reg_mov_check(0, 42); }
#[test] fn r0_mov_255() { reg_mov_check(0, 255); }
#[test] fn r0_mov_1000() { reg_mov_check(0, 1000); }
#[test] fn r0_mov_1337() { reg_mov_check(0, 1337); }
#[test] fn r0_mov_32767() { reg_mov_check(0, 32767); }
#[test] fn r0_mov_57005() { reg_mov_check(0, 57005); }
#[test] fn r1_mov_0() { reg_mov_check(1, 0); }
#[test] fn r1_mov_1() { reg_mov_check(1, 1); }
#[test] fn r1_mov_42() { reg_mov_check(1, 42); }
#[test] fn r1_mov_255() { reg_mov_check(1, 255); }
#[test] fn r1_mov_1000() { reg_mov_check(1, 1000); }
#[test] fn r1_mov_1337() { reg_mov_check(1, 1337); }
#[test] fn r1_mov_32767() { reg_mov_check(1, 32767); }
#[test] fn r1_mov_57005() { reg_mov_check(1, 57005); }
#[test] fn r2_mov_0() { reg_mov_check(2, 0); }
#[test] fn r2_mov_1() { reg_mov_check(2, 1); }
#[test] fn r2_mov_42() { reg_mov_check(2, 42); }
#[test] fn r2_mov_255() { reg_mov_check(2, 255); }
#[test] fn r2_mov_1000() { reg_mov_check(2, 1000); }
#[test] fn r2_mov_1337() { reg_mov_check(2, 1337); }
#[test] fn r2_mov_32767() { reg_mov_check(2, 32767); }
#[test] fn r2_mov_57005() { reg_mov_check(2, 57005); }
#[test] fn r3_mov_0() { reg_mov_check(3, 0); }
#[test] fn r3_mov_1() { reg_mov_check(3, 1); }
#[test] fn r3_mov_42() { reg_mov_check(3, 42); }
#[test] fn r3_mov_255() { reg_mov_check(3, 255); }
#[test] fn r3_mov_1000() { reg_mov_check(3, 1000); }
#[test] fn r3_mov_1337() { reg_mov_check(3, 1337); }
#[test] fn r3_mov_32767() { reg_mov_check(3, 32767); }
#[test] fn r3_mov_57005() { reg_mov_check(3, 57005); }
#[test] fn r4_mov_0() { reg_mov_check(4, 0); }
#[test] fn r4_mov_1() { reg_mov_check(4, 1); }
#[test] fn r4_mov_42() { reg_mov_check(4, 42); }
#[test] fn r4_mov_255() { reg_mov_check(4, 255); }
#[test] fn r4_mov_1000() { reg_mov_check(4, 1000); }
#[test] fn r4_mov_1337() { reg_mov_check(4, 1337); }
#[test] fn r4_mov_32767() { reg_mov_check(4, 32767); }
#[test] fn r4_mov_57005() { reg_mov_check(4, 57005); }
#[test] fn r5_mov_0() { reg_mov_check(5, 0); }
#[test] fn r5_mov_1() { reg_mov_check(5, 1); }
#[test] fn r5_mov_42() { reg_mov_check(5, 42); }
#[test] fn r5_mov_255() { reg_mov_check(5, 255); }
#[test] fn r5_mov_1000() { reg_mov_check(5, 1000); }
#[test] fn r5_mov_1337() { reg_mov_check(5, 1337); }
#[test] fn r5_mov_32767() { reg_mov_check(5, 32767); }
#[test] fn r5_mov_57005() { reg_mov_check(5, 57005); }
#[test] fn r6_mov_0() { reg_mov_check(6, 0); }
#[test] fn r6_mov_1() { reg_mov_check(6, 1); }
#[test] fn r6_mov_42() { reg_mov_check(6, 42); }
#[test] fn r6_mov_255() { reg_mov_check(6, 255); }
#[test] fn r6_mov_1000() { reg_mov_check(6, 1000); }
#[test] fn r6_mov_1337() { reg_mov_check(6, 1337); }
#[test] fn r6_mov_32767() { reg_mov_check(6, 32767); }
#[test] fn r6_mov_57005() { reg_mov_check(6, 57005); }
#[test] fn r7_mov_0() { reg_mov_check(7, 0); }
#[test] fn r7_mov_1() { reg_mov_check(7, 1); }
#[test] fn r7_mov_42() { reg_mov_check(7, 42); }
#[test] fn r7_mov_255() { reg_mov_check(7, 255); }
#[test] fn r7_mov_1000() { reg_mov_check(7, 1000); }
#[test] fn r7_mov_1337() { reg_mov_check(7, 1337); }
#[test] fn r7_mov_32767() { reg_mov_check(7, 32767); }
#[test] fn r7_mov_57005() { reg_mov_check(7, 57005); }
#[test] fn r8_mov_0() { reg_mov_check(8, 0); }
#[test] fn r8_mov_1() { reg_mov_check(8, 1); }
#[test] fn r8_mov_42() { reg_mov_check(8, 42); }
#[test] fn r8_mov_255() { reg_mov_check(8, 255); }
#[test] fn r8_mov_1000() { reg_mov_check(8, 1000); }
#[test] fn r8_mov_1337() { reg_mov_check(8, 1337); }
#[test] fn r8_mov_32767() { reg_mov_check(8, 32767); }
#[test] fn r8_mov_57005() { reg_mov_check(8, 57005); }
#[test] fn r9_mov_0() { reg_mov_check(9, 0); }
#[test] fn r9_mov_1() { reg_mov_check(9, 1); }
#[test] fn r9_mov_42() { reg_mov_check(9, 42); }
#[test] fn r9_mov_255() { reg_mov_check(9, 255); }
#[test] fn r9_mov_1000() { reg_mov_check(9, 1000); }
#[test] fn r9_mov_1337() { reg_mov_check(9, 1337); }
#[test] fn r9_mov_32767() { reg_mov_check(9, 32767); }
#[test] fn r9_mov_57005() { reg_mov_check(9, 57005); }
#[test] fn r10_mov_0() { reg_mov_check(10, 0); }
#[test] fn r10_mov_1() { reg_mov_check(10, 1); }
#[test] fn r10_mov_42() { reg_mov_check(10, 42); }
#[test] fn r10_mov_255() { reg_mov_check(10, 255); }
#[test] fn r10_mov_1000() { reg_mov_check(10, 1000); }
#[test] fn r10_mov_1337() { reg_mov_check(10, 1337); }
#[test] fn r10_mov_32767() { reg_mov_check(10, 32767); }
#[test] fn r10_mov_57005() { reg_mov_check(10, 57005); }
#[test] fn r11_mov_0() { reg_mov_check(11, 0); }
#[test] fn r11_mov_1() { reg_mov_check(11, 1); }
#[test] fn r11_mov_42() { reg_mov_check(11, 42); }
#[test] fn r11_mov_255() { reg_mov_check(11, 255); }
#[test] fn r11_mov_1000() { reg_mov_check(11, 1000); }
#[test] fn r11_mov_1337() { reg_mov_check(11, 1337); }
#[test] fn r11_mov_32767() { reg_mov_check(11, 32767); }
#[test] fn r11_mov_57005() { reg_mov_check(11, 57005); }
#[test] fn r12_mov_0() { reg_mov_check(12, 0); }
#[test] fn r12_mov_1() { reg_mov_check(12, 1); }
#[test] fn r12_mov_42() { reg_mov_check(12, 42); }
#[test] fn r12_mov_255() { reg_mov_check(12, 255); }
#[test] fn r12_mov_1000() { reg_mov_check(12, 1000); }
#[test] fn r12_mov_1337() { reg_mov_check(12, 1337); }
#[test] fn r12_mov_32767() { reg_mov_check(12, 32767); }
#[test] fn r12_mov_57005() { reg_mov_check(12, 57005); }
#[test] fn r13_mov_0() { reg_mov_check(13, 0); }
#[test] fn r13_mov_1() { reg_mov_check(13, 1); }
#[test] fn r13_mov_42() { reg_mov_check(13, 42); }
#[test] fn r13_mov_255() { reg_mov_check(13, 255); }
#[test] fn r13_mov_1000() { reg_mov_check(13, 1000); }
#[test] fn r13_mov_1337() { reg_mov_check(13, 1337); }
#[test] fn r13_mov_32767() { reg_mov_check(13, 32767); }
#[test] fn r13_mov_57005() { reg_mov_check(13, 57005); }
#[test] fn r14_mov_0() { reg_mov_check(14, 0); }
#[test] fn r14_mov_1() { reg_mov_check(14, 1); }
#[test] fn r14_mov_42() { reg_mov_check(14, 42); }
#[test] fn r14_mov_255() { reg_mov_check(14, 255); }
#[test] fn r14_mov_1000() { reg_mov_check(14, 1000); }
#[test] fn r14_mov_1337() { reg_mov_check(14, 1337); }
#[test] fn r14_mov_32767() { reg_mov_check(14, 32767); }
#[test] fn r14_mov_57005() { reg_mov_check(14, 57005); }
#[test] fn r15_mov_0() { reg_mov_check(15, 0); }
#[test] fn r15_mov_1() { reg_mov_check(15, 1); }
#[test] fn r15_mov_42() { reg_mov_check(15, 42); }
#[test] fn r15_mov_255() { reg_mov_check(15, 255); }
#[test] fn r15_mov_1000() { reg_mov_check(15, 1000); }
#[test] fn r15_mov_1337() { reg_mov_check(15, 1337); }
#[test] fn r15_mov_32767() { reg_mov_check(15, 32767); }
#[test] fn r15_mov_57005() { reg_mov_check(15, 57005); }
#[test] fn add_r0_r0() { add_pair_check(0, 0); }
#[test] fn add_r0_r1() { add_pair_check(0, 1); }
#[test] fn add_r0_r2() { add_pair_check(0, 2); }
#[test] fn add_r0_r3() { add_pair_check(0, 3); }
#[test] fn add_r0_r4() { add_pair_check(0, 4); }
#[test] fn add_r0_r5() { add_pair_check(0, 5); }
#[test] fn add_r0_r6() { add_pair_check(0, 6); }
#[test] fn add_r0_r7() { add_pair_check(0, 7); }
#[test] fn add_r0_r8() { add_pair_check(0, 8); }
#[test] fn add_r0_r9() { add_pair_check(0, 9); }
#[test] fn add_r0_r10() { add_pair_check(0, 10); }
#[test] fn add_r0_r11() { add_pair_check(0, 11); }
#[test] fn add_r0_r12() { add_pair_check(0, 12); }
#[test] fn add_r0_r13() { add_pair_check(0, 13); }
#[test] fn add_r0_r14() { add_pair_check(0, 14); }
#[test] fn add_r0_r15() { add_pair_check(0, 15); }
#[test] fn add_r1_r0() { add_pair_check(1, 0); }
#[test] fn add_r1_r1() { add_pair_check(1, 1); }
#[test] fn add_r1_r2() { add_pair_check(1, 2); }
#[test] fn add_r1_r3() { add_pair_check(1, 3); }
#[test] fn add_r1_r4() { add_pair_check(1, 4); }
#[test] fn add_r1_r5() { add_pair_check(1, 5); }
#[test] fn add_r1_r6() { add_pair_check(1, 6); }
#[test] fn add_r1_r7() { add_pair_check(1, 7); }
#[test] fn add_r1_r8() { add_pair_check(1, 8); }
#[test] fn add_r1_r9() { add_pair_check(1, 9); }
#[test] fn add_r1_r10() { add_pair_check(1, 10); }
#[test] fn add_r1_r11() { add_pair_check(1, 11); }
#[test] fn add_r1_r12() { add_pair_check(1, 12); }
#[test] fn add_r1_r13() { add_pair_check(1, 13); }
#[test] fn add_r1_r14() { add_pair_check(1, 14); }
#[test] fn add_r1_r15() { add_pair_check(1, 15); }
#[test] fn add_r2_r0() { add_pair_check(2, 0); }
#[test] fn add_r2_r1() { add_pair_check(2, 1); }
#[test] fn add_r2_r2() { add_pair_check(2, 2); }
#[test] fn add_r2_r3() { add_pair_check(2, 3); }
#[test] fn add_r2_r4() { add_pair_check(2, 4); }
#[test] fn add_r2_r5() { add_pair_check(2, 5); }
#[test] fn add_r2_r6() { add_pair_check(2, 6); }
#[test] fn add_r2_r7() { add_pair_check(2, 7); }
#[test] fn add_r2_r8() { add_pair_check(2, 8); }
#[test] fn add_r2_r9() { add_pair_check(2, 9); }
#[test] fn add_r2_r10() { add_pair_check(2, 10); }
#[test] fn add_r2_r11() { add_pair_check(2, 11); }
#[test] fn add_r2_r12() { add_pair_check(2, 12); }
#[test] fn add_r2_r13() { add_pair_check(2, 13); }
#[test] fn add_r2_r14() { add_pair_check(2, 14); }
#[test] fn add_r2_r15() { add_pair_check(2, 15); }
#[test] fn add_r3_r0() { add_pair_check(3, 0); }
#[test] fn add_r3_r1() { add_pair_check(3, 1); }
#[test] fn add_r3_r2() { add_pair_check(3, 2); }
#[test] fn add_r3_r3() { add_pair_check(3, 3); }
#[test] fn add_r3_r4() { add_pair_check(3, 4); }
#[test] fn add_r3_r5() { add_pair_check(3, 5); }
#[test] fn add_r3_r6() { add_pair_check(3, 6); }
#[test] fn add_r3_r7() { add_pair_check(3, 7); }
#[test] fn add_r3_r8() { add_pair_check(3, 8); }
#[test] fn add_r3_r9() { add_pair_check(3, 9); }
#[test] fn add_r3_r10() { add_pair_check(3, 10); }
#[test] fn add_r3_r11() { add_pair_check(3, 11); }
#[test] fn add_r3_r12() { add_pair_check(3, 12); }
#[test] fn add_r3_r13() { add_pair_check(3, 13); }
#[test] fn add_r3_r14() { add_pair_check(3, 14); }
#[test] fn add_r3_r15() { add_pair_check(3, 15); }
#[test] fn add_r4_r0() { add_pair_check(4, 0); }
#[test] fn add_r4_r1() { add_pair_check(4, 1); }
#[test] fn add_r4_r2() { add_pair_check(4, 2); }
#[test] fn add_r4_r3() { add_pair_check(4, 3); }
#[test] fn add_r4_r4() { add_pair_check(4, 4); }
#[test] fn add_r4_r5() { add_pair_check(4, 5); }
#[test] fn add_r4_r6() { add_pair_check(4, 6); }
#[test] fn add_r4_r7() { add_pair_check(4, 7); }
#[test] fn add_r4_r8() { add_pair_check(4, 8); }
#[test] fn add_r4_r9() { add_pair_check(4, 9); }
#[test] fn add_r4_r10() { add_pair_check(4, 10); }
#[test] fn add_r4_r11() { add_pair_check(4, 11); }
#[test] fn add_r4_r12() { add_pair_check(4, 12); }
#[test] fn add_r4_r13() { add_pair_check(4, 13); }
#[test] fn add_r4_r14() { add_pair_check(4, 14); }
#[test] fn add_r4_r15() { add_pair_check(4, 15); }
#[test] fn add_r5_r0() { add_pair_check(5, 0); }
#[test] fn add_r5_r1() { add_pair_check(5, 1); }
#[test] fn add_r5_r2() { add_pair_check(5, 2); }
#[test] fn add_r5_r3() { add_pair_check(5, 3); }
#[test] fn add_r5_r4() { add_pair_check(5, 4); }
#[test] fn add_r5_r5() { add_pair_check(5, 5); }
#[test] fn add_r5_r6() { add_pair_check(5, 6); }
#[test] fn add_r5_r7() { add_pair_check(5, 7); }
#[test] fn add_r5_r8() { add_pair_check(5, 8); }
#[test] fn add_r5_r9() { add_pair_check(5, 9); }
#[test] fn add_r5_r10() { add_pair_check(5, 10); }
#[test] fn add_r5_r11() { add_pair_check(5, 11); }
#[test] fn add_r5_r12() { add_pair_check(5, 12); }
#[test] fn add_r5_r13() { add_pair_check(5, 13); }
#[test] fn add_r5_r14() { add_pair_check(5, 14); }
#[test] fn add_r5_r15() { add_pair_check(5, 15); }
#[test] fn add_r6_r0() { add_pair_check(6, 0); }
#[test] fn add_r6_r1() { add_pair_check(6, 1); }
#[test] fn add_r6_r2() { add_pair_check(6, 2); }
#[test] fn add_r6_r3() { add_pair_check(6, 3); }
#[test] fn add_r6_r4() { add_pair_check(6, 4); }
#[test] fn add_r6_r5() { add_pair_check(6, 5); }
#[test] fn add_r6_r6() { add_pair_check(6, 6); }
#[test] fn add_r6_r7() { add_pair_check(6, 7); }
#[test] fn add_r6_r8() { add_pair_check(6, 8); }
#[test] fn add_r6_r9() { add_pair_check(6, 9); }
#[test] fn add_r6_r10() { add_pair_check(6, 10); }
#[test] fn add_r6_r11() { add_pair_check(6, 11); }
#[test] fn add_r6_r12() { add_pair_check(6, 12); }
#[test] fn add_r6_r13() { add_pair_check(6, 13); }
#[test] fn add_r6_r14() { add_pair_check(6, 14); }
#[test] fn add_r6_r15() { add_pair_check(6, 15); }
#[test] fn add_r7_r0() { add_pair_check(7, 0); }
#[test] fn add_r7_r1() { add_pair_check(7, 1); }
#[test] fn add_r7_r2() { add_pair_check(7, 2); }
#[test] fn add_r7_r3() { add_pair_check(7, 3); }
#[test] fn add_r7_r4() { add_pair_check(7, 4); }
#[test] fn add_r7_r5() { add_pair_check(7, 5); }
#[test] fn add_r7_r6() { add_pair_check(7, 6); }
#[test] fn add_r7_r7() { add_pair_check(7, 7); }
#[test] fn add_r7_r8() { add_pair_check(7, 8); }
#[test] fn add_r7_r9() { add_pair_check(7, 9); }
#[test] fn add_r7_r10() { add_pair_check(7, 10); }
#[test] fn add_r7_r11() { add_pair_check(7, 11); }
#[test] fn add_r7_r12() { add_pair_check(7, 12); }
#[test] fn add_r7_r13() { add_pair_check(7, 13); }
#[test] fn add_r7_r14() { add_pair_check(7, 14); }
#[test] fn add_r7_r15() { add_pair_check(7, 15); }
#[test] fn add_r8_r0() { add_pair_check(8, 0); }
#[test] fn add_r8_r1() { add_pair_check(8, 1); }
#[test] fn add_r8_r2() { add_pair_check(8, 2); }
#[test] fn add_r8_r3() { add_pair_check(8, 3); }
#[test] fn add_r8_r4() { add_pair_check(8, 4); }
#[test] fn add_r8_r5() { add_pair_check(8, 5); }
#[test] fn add_r8_r6() { add_pair_check(8, 6); }
#[test] fn add_r8_r7() { add_pair_check(8, 7); }
#[test] fn add_r8_r8() { add_pair_check(8, 8); }
#[test] fn add_r8_r9() { add_pair_check(8, 9); }
#[test] fn add_r8_r10() { add_pair_check(8, 10); }
#[test] fn add_r8_r11() { add_pair_check(8, 11); }
#[test] fn add_r8_r12() { add_pair_check(8, 12); }
#[test] fn add_r8_r13() { add_pair_check(8, 13); }
#[test] fn add_r8_r14() { add_pair_check(8, 14); }
#[test] fn add_r8_r15() { add_pair_check(8, 15); }
#[test] fn add_r9_r0() { add_pair_check(9, 0); }
#[test] fn add_r9_r1() { add_pair_check(9, 1); }
#[test] fn add_r9_r2() { add_pair_check(9, 2); }
#[test] fn add_r9_r3() { add_pair_check(9, 3); }
#[test] fn add_r9_r4() { add_pair_check(9, 4); }
#[test] fn add_r9_r5() { add_pair_check(9, 5); }
#[test] fn add_r9_r6() { add_pair_check(9, 6); }
#[test] fn add_r9_r7() { add_pair_check(9, 7); }
#[test] fn add_r9_r8() { add_pair_check(9, 8); }
#[test] fn add_r9_r9() { add_pair_check(9, 9); }
#[test] fn add_r9_r10() { add_pair_check(9, 10); }
#[test] fn add_r9_r11() { add_pair_check(9, 11); }
#[test] fn add_r9_r12() { add_pair_check(9, 12); }
#[test] fn add_r9_r13() { add_pair_check(9, 13); }
#[test] fn add_r9_r14() { add_pair_check(9, 14); }
#[test] fn add_r9_r15() { add_pair_check(9, 15); }
#[test] fn add_r10_r0() { add_pair_check(10, 0); }
#[test] fn add_r10_r1() { add_pair_check(10, 1); }
#[test] fn add_r10_r2() { add_pair_check(10, 2); }
#[test] fn add_r10_r3() { add_pair_check(10, 3); }
#[test] fn add_r10_r4() { add_pair_check(10, 4); }
#[test] fn add_r10_r5() { add_pair_check(10, 5); }
#[test] fn add_r10_r6() { add_pair_check(10, 6); }
#[test] fn add_r10_r7() { add_pair_check(10, 7); }
#[test] fn add_r10_r8() { add_pair_check(10, 8); }
#[test] fn add_r10_r9() { add_pair_check(10, 9); }
#[test] fn add_r10_r10() { add_pair_check(10, 10); }
#[test] fn add_r10_r11() { add_pair_check(10, 11); }
#[test] fn add_r10_r12() { add_pair_check(10, 12); }
#[test] fn add_r10_r13() { add_pair_check(10, 13); }
#[test] fn add_r10_r14() { add_pair_check(10, 14); }
#[test] fn add_r10_r15() { add_pair_check(10, 15); }
#[test] fn add_r11_r0() { add_pair_check(11, 0); }
#[test] fn add_r11_r1() { add_pair_check(11, 1); }
#[test] fn add_r11_r2() { add_pair_check(11, 2); }
#[test] fn add_r11_r3() { add_pair_check(11, 3); }
#[test] fn add_r11_r4() { add_pair_check(11, 4); }
#[test] fn add_r11_r5() { add_pair_check(11, 5); }
#[test] fn add_r11_r6() { add_pair_check(11, 6); }
#[test] fn add_r11_r7() { add_pair_check(11, 7); }
#[test] fn add_r11_r8() { add_pair_check(11, 8); }
#[test] fn add_r11_r9() { add_pair_check(11, 9); }
#[test] fn add_r11_r10() { add_pair_check(11, 10); }
#[test] fn add_r11_r11() { add_pair_check(11, 11); }
#[test] fn add_r11_r12() { add_pair_check(11, 12); }
#[test] fn add_r11_r13() { add_pair_check(11, 13); }
#[test] fn add_r11_r14() { add_pair_check(11, 14); }
#[test] fn add_r11_r15() { add_pair_check(11, 15); }
#[test] fn add_r12_r0() { add_pair_check(12, 0); }
#[test] fn add_r12_r1() { add_pair_check(12, 1); }
#[test] fn add_r12_r2() { add_pair_check(12, 2); }
#[test] fn add_r12_r3() { add_pair_check(12, 3); }
#[test] fn add_r12_r4() { add_pair_check(12, 4); }
#[test] fn add_r12_r5() { add_pair_check(12, 5); }
#[test] fn add_r12_r6() { add_pair_check(12, 6); }
#[test] fn add_r12_r7() { add_pair_check(12, 7); }
#[test] fn add_r12_r8() { add_pair_check(12, 8); }
#[test] fn add_r12_r9() { add_pair_check(12, 9); }
#[test] fn add_r12_r10() { add_pair_check(12, 10); }
#[test] fn add_r12_r11() { add_pair_check(12, 11); }
#[test] fn add_r12_r12() { add_pair_check(12, 12); }
#[test] fn add_r12_r13() { add_pair_check(12, 13); }
#[test] fn add_r12_r14() { add_pair_check(12, 14); }
#[test] fn add_r12_r15() { add_pair_check(12, 15); }
#[test] fn add_r13_r0() { add_pair_check(13, 0); }
#[test] fn add_r13_r1() { add_pair_check(13, 1); }
#[test] fn add_r13_r2() { add_pair_check(13, 2); }
#[test] fn add_r13_r3() { add_pair_check(13, 3); }
#[test] fn add_r13_r4() { add_pair_check(13, 4); }
#[test] fn add_r13_r5() { add_pair_check(13, 5); }
#[test] fn add_r13_r6() { add_pair_check(13, 6); }
#[test] fn add_r13_r7() { add_pair_check(13, 7); }
#[test] fn add_r13_r8() { add_pair_check(13, 8); }
#[test] fn add_r13_r9() { add_pair_check(13, 9); }
#[test] fn add_r13_r10() { add_pair_check(13, 10); }
#[test] fn add_r13_r11() { add_pair_check(13, 11); }
#[test] fn add_r13_r12() { add_pair_check(13, 12); }
#[test] fn add_r13_r13() { add_pair_check(13, 13); }
#[test] fn add_r13_r14() { add_pair_check(13, 14); }
#[test] fn add_r13_r15() { add_pair_check(13, 15); }
#[test] fn add_r14_r0() { add_pair_check(14, 0); }
#[test] fn add_r14_r1() { add_pair_check(14, 1); }
#[test] fn add_r14_r2() { add_pair_check(14, 2); }
#[test] fn add_r14_r3() { add_pair_check(14, 3); }
#[test] fn add_r14_r4() { add_pair_check(14, 4); }
#[test] fn add_r14_r5() { add_pair_check(14, 5); }
#[test] fn add_r14_r6() { add_pair_check(14, 6); }
#[test] fn add_r14_r7() { add_pair_check(14, 7); }
#[test] fn add_r14_r8() { add_pair_check(14, 8); }
#[test] fn add_r14_r9() { add_pair_check(14, 9); }
#[test] fn add_r14_r10() { add_pair_check(14, 10); }
#[test] fn add_r14_r11() { add_pair_check(14, 11); }
#[test] fn add_r14_r12() { add_pair_check(14, 12); }
#[test] fn add_r14_r13() { add_pair_check(14, 13); }
#[test] fn add_r14_r14() { add_pair_check(14, 14); }
#[test] fn add_r14_r15() { add_pair_check(14, 15); }
#[test] fn add_r15_r0() { add_pair_check(15, 0); }
#[test] fn add_r15_r1() { add_pair_check(15, 1); }
#[test] fn add_r15_r2() { add_pair_check(15, 2); }
#[test] fn add_r15_r3() { add_pair_check(15, 3); }
#[test] fn add_r15_r4() { add_pair_check(15, 4); }
#[test] fn add_r15_r5() { add_pair_check(15, 5); }
#[test] fn add_r15_r6() { add_pair_check(15, 6); }
#[test] fn add_r15_r7() { add_pair_check(15, 7); }
#[test] fn add_r15_r8() { add_pair_check(15, 8); }
#[test] fn add_r15_r9() { add_pair_check(15, 9); }
#[test] fn add_r15_r10() { add_pair_check(15, 10); }
#[test] fn add_r15_r11() { add_pair_check(15, 11); }
#[test] fn add_r15_r12() { add_pair_check(15, 12); }
#[test] fn add_r15_r13() { add_pair_check(15, 13); }
#[test] fn add_r15_r14() { add_pair_check(15, 14); }
#[test] fn add_r15_r15() { add_pair_check(15, 15); }
#[test] fn core0_s0() { sentron_core_check(0, 0); }
#[test] fn core0_s1() { sentron_core_check(1, 0); }
#[test] fn core0_s2() { sentron_core_check(2, 0); }
#[test] fn core0_s3() { sentron_core_check(3, 0); }
#[test] fn core0_s4() { sentron_core_check(4, 0); }
#[test] fn core0_s5() { sentron_core_check(5, 0); }
#[test] fn core0_s6() { sentron_core_check(6, 0); }
#[test] fn core0_s7() { sentron_core_check(7, 0); }
#[test] fn core0_s8() { sentron_core_check(8, 0); }
#[test] fn core0_s9() { sentron_core_check(9, 0); }
#[test] fn core0_s10() { sentron_core_check(10, 0); }
#[test] fn core0_s11() { sentron_core_check(11, 0); }
#[test] fn core0_s12() { sentron_core_check(12, 0); }
#[test] fn core0_s13() { sentron_core_check(13, 0); }
#[test] fn core0_s14() { sentron_core_check(14, 0); }
#[test] fn core0_s15() { sentron_core_check(15, 0); }
#[test] fn core0_s16() { sentron_core_check(16, 0); }
#[test] fn core0_s17() { sentron_core_check(17, 0); }
#[test] fn core0_s18() { sentron_core_check(18, 0); }
#[test] fn core0_s19() { sentron_core_check(19, 0); }
#[test] fn core0_s20() { sentron_core_check(20, 0); }
#[test] fn core0_s21() { sentron_core_check(21, 0); }
#[test] fn core0_s22() { sentron_core_check(22, 0); }
#[test] fn core0_s23() { sentron_core_check(23, 0); }
#[test] fn core0_s24() { sentron_core_check(24, 0); }
#[test] fn core0_s25() { sentron_core_check(25, 0); }
#[test] fn core0_s26() { sentron_core_check(26, 0); }
#[test] fn core0_s27() { sentron_core_check(27, 0); }
#[test] fn core0_s28() { sentron_core_check(28, 0); }
#[test] fn core0_s29() { sentron_core_check(29, 0); }
#[test] fn core0_s30() { sentron_core_check(30, 0); }
#[test] fn core0_s31() { sentron_core_check(31, 0); }
#[test] fn core0_s32() { sentron_core_check(32, 0); }
#[test] fn core0_s33() { sentron_core_check(33, 0); }
#[test] fn core0_s34() { sentron_core_check(34, 0); }
#[test] fn core0_s35() { sentron_core_check(35, 0); }
#[test] fn core0_s36() { sentron_core_check(36, 0); }
#[test] fn core0_s37() { sentron_core_check(37, 0); }
#[test] fn core0_s38() { sentron_core_check(38, 0); }
#[test] fn core0_s39() { sentron_core_check(39, 0); }
#[test] fn core1_s0() { sentron_core_check(0, 1); }
#[test] fn core1_s1() { sentron_core_check(1, 1); }
#[test] fn core1_s2() { sentron_core_check(2, 1); }
#[test] fn core1_s3() { sentron_core_check(3, 1); }
#[test] fn core1_s4() { sentron_core_check(4, 1); }
#[test] fn core1_s5() { sentron_core_check(5, 1); }
#[test] fn core1_s6() { sentron_core_check(6, 1); }
#[test] fn core1_s7() { sentron_core_check(7, 1); }
#[test] fn core1_s8() { sentron_core_check(8, 1); }
#[test] fn core1_s9() { sentron_core_check(9, 1); }
#[test] fn core1_s10() { sentron_core_check(10, 1); }
#[test] fn core1_s11() { sentron_core_check(11, 1); }
#[test] fn core1_s12() { sentron_core_check(12, 1); }
#[test] fn core1_s13() { sentron_core_check(13, 1); }
#[test] fn core1_s14() { sentron_core_check(14, 1); }
#[test] fn core1_s15() { sentron_core_check(15, 1); }
#[test] fn core1_s16() { sentron_core_check(16, 1); }
#[test] fn core1_s17() { sentron_core_check(17, 1); }
#[test] fn core1_s18() { sentron_core_check(18, 1); }
#[test] fn core1_s19() { sentron_core_check(19, 1); }
#[test] fn core1_s20() { sentron_core_check(20, 1); }
#[test] fn core1_s21() { sentron_core_check(21, 1); }
#[test] fn core1_s22() { sentron_core_check(22, 1); }
#[test] fn core1_s23() { sentron_core_check(23, 1); }
#[test] fn core1_s24() { sentron_core_check(24, 1); }
#[test] fn core1_s25() { sentron_core_check(25, 1); }
#[test] fn core1_s26() { sentron_core_check(26, 1); }
#[test] fn core1_s27() { sentron_core_check(27, 1); }
#[test] fn core1_s28() { sentron_core_check(28, 1); }
#[test] fn core1_s29() { sentron_core_check(29, 1); }
#[test] fn core1_s30() { sentron_core_check(30, 1); }
#[test] fn core1_s31() { sentron_core_check(31, 1); }
#[test] fn core1_s32() { sentron_core_check(32, 1); }
#[test] fn core1_s33() { sentron_core_check(33, 1); }
#[test] fn core1_s34() { sentron_core_check(34, 1); }
#[test] fn core1_s35() { sentron_core_check(35, 1); }
#[test] fn core1_s36() { sentron_core_check(36, 1); }
#[test] fn core1_s37() { sentron_core_check(37, 1); }
#[test] fn core1_s38() { sentron_core_check(38, 1); }
#[test] fn core1_s39() { sentron_core_check(39, 1); }
#[test] fn core2_s0() { sentron_core_check(0, 2); }
#[test] fn core2_s1() { sentron_core_check(1, 2); }
#[test] fn core2_s2() { sentron_core_check(2, 2); }
#[test] fn core2_s3() { sentron_core_check(3, 2); }
#[test] fn core2_s4() { sentron_core_check(4, 2); }
#[test] fn core2_s5() { sentron_core_check(5, 2); }
#[test] fn core2_s6() { sentron_core_check(6, 2); }
#[test] fn core2_s7() { sentron_core_check(7, 2); }
#[test] fn core2_s8() { sentron_core_check(8, 2); }
#[test] fn core2_s9() { sentron_core_check(9, 2); }
#[test] fn core2_s10() { sentron_core_check(10, 2); }
#[test] fn core2_s11() { sentron_core_check(11, 2); }
#[test] fn core2_s12() { sentron_core_check(12, 2); }
#[test] fn core2_s13() { sentron_core_check(13, 2); }
#[test] fn core2_s14() { sentron_core_check(14, 2); }
#[test] fn core2_s15() { sentron_core_check(15, 2); }
#[test] fn core2_s16() { sentron_core_check(16, 2); }
#[test] fn core2_s17() { sentron_core_check(17, 2); }
#[test] fn core2_s18() { sentron_core_check(18, 2); }
#[test] fn core2_s19() { sentron_core_check(19, 2); }
#[test] fn core2_s20() { sentron_core_check(20, 2); }
#[test] fn core2_s21() { sentron_core_check(21, 2); }
#[test] fn core2_s22() { sentron_core_check(22, 2); }
#[test] fn core2_s23() { sentron_core_check(23, 2); }
#[test] fn core2_s24() { sentron_core_check(24, 2); }
#[test] fn core2_s25() { sentron_core_check(25, 2); }
#[test] fn core2_s26() { sentron_core_check(26, 2); }
#[test] fn core2_s27() { sentron_core_check(27, 2); }
#[test] fn core2_s28() { sentron_core_check(28, 2); }
#[test] fn core2_s29() { sentron_core_check(29, 2); }
#[test] fn core2_s30() { sentron_core_check(30, 2); }
#[test] fn core2_s31() { sentron_core_check(31, 2); }
#[test] fn core2_s32() { sentron_core_check(32, 2); }
#[test] fn core2_s33() { sentron_core_check(33, 2); }
#[test] fn core2_s34() { sentron_core_check(34, 2); }
#[test] fn core2_s35() { sentron_core_check(35, 2); }
#[test] fn core2_s36() { sentron_core_check(36, 2); }
#[test] fn core2_s37() { sentron_core_check(37, 2); }
#[test] fn core2_s38() { sentron_core_check(38, 2); }
#[test] fn core2_s39() { sentron_core_check(39, 2); }
#[test] fn core3_s0() { sentron_core_check(0, 3); }
#[test] fn core3_s1() { sentron_core_check(1, 3); }
#[test] fn core3_s2() { sentron_core_check(2, 3); }
#[test] fn core3_s3() { sentron_core_check(3, 3); }
#[test] fn core3_s4() { sentron_core_check(4, 3); }
#[test] fn core3_s5() { sentron_core_check(5, 3); }
#[test] fn core3_s6() { sentron_core_check(6, 3); }
#[test] fn core3_s7() { sentron_core_check(7, 3); }
#[test] fn core3_s8() { sentron_core_check(8, 3); }
#[test] fn core3_s9() { sentron_core_check(9, 3); }
#[test] fn core3_s10() { sentron_core_check(10, 3); }
#[test] fn core3_s11() { sentron_core_check(11, 3); }
#[test] fn core3_s12() { sentron_core_check(12, 3); }
#[test] fn core3_s13() { sentron_core_check(13, 3); }
#[test] fn core3_s14() { sentron_core_check(14, 3); }
#[test] fn core3_s15() { sentron_core_check(15, 3); }
#[test] fn core3_s16() { sentron_core_check(16, 3); }
#[test] fn core3_s17() { sentron_core_check(17, 3); }
#[test] fn core3_s18() { sentron_core_check(18, 3); }
#[test] fn core3_s19() { sentron_core_check(19, 3); }
#[test] fn core3_s20() { sentron_core_check(20, 3); }
#[test] fn core3_s21() { sentron_core_check(21, 3); }
#[test] fn core3_s22() { sentron_core_check(22, 3); }
#[test] fn core3_s23() { sentron_core_check(23, 3); }
#[test] fn core3_s24() { sentron_core_check(24, 3); }
#[test] fn core3_s25() { sentron_core_check(25, 3); }
#[test] fn core3_s26() { sentron_core_check(26, 3); }
#[test] fn core3_s27() { sentron_core_check(27, 3); }
#[test] fn core3_s28() { sentron_core_check(28, 3); }
#[test] fn core3_s29() { sentron_core_check(29, 3); }
#[test] fn core3_s30() { sentron_core_check(30, 3); }
#[test] fn core3_s31() { sentron_core_check(31, 3); }
#[test] fn core3_s32() { sentron_core_check(32, 3); }
#[test] fn core3_s33() { sentron_core_check(33, 3); }
#[test] fn core3_s34() { sentron_core_check(34, 3); }
#[test] fn core3_s35() { sentron_core_check(35, 3); }
#[test] fn core3_s36() { sentron_core_check(36, 3); }
#[test] fn core3_s37() { sentron_core_check(37, 3); }
#[test] fn core3_s38() { sentron_core_check(38, 3); }
#[test] fn core3_s39() { sentron_core_check(39, 3); }
#[test] fn core4_s0() { sentron_core_check(0, 4); }
#[test] fn core4_s1() { sentron_core_check(1, 4); }
#[test] fn core4_s2() { sentron_core_check(2, 4); }
#[test] fn core4_s3() { sentron_core_check(3, 4); }
#[test] fn core4_s4() { sentron_core_check(4, 4); }
#[test] fn core4_s5() { sentron_core_check(5, 4); }
#[test] fn core4_s6() { sentron_core_check(6, 4); }
#[test] fn core4_s7() { sentron_core_check(7, 4); }
#[test] fn core4_s8() { sentron_core_check(8, 4); }
#[test] fn core4_s9() { sentron_core_check(9, 4); }
#[test] fn core4_s10() { sentron_core_check(10, 4); }
#[test] fn core4_s11() { sentron_core_check(11, 4); }
#[test] fn core4_s12() { sentron_core_check(12, 4); }
#[test] fn core4_s13() { sentron_core_check(13, 4); }
#[test] fn core4_s14() { sentron_core_check(14, 4); }
#[test] fn core4_s15() { sentron_core_check(15, 4); }
#[test] fn core4_s16() { sentron_core_check(16, 4); }
#[test] fn core4_s17() { sentron_core_check(17, 4); }
#[test] fn core4_s18() { sentron_core_check(18, 4); }
#[test] fn core4_s19() { sentron_core_check(19, 4); }
#[test] fn core4_s20() { sentron_core_check(20, 4); }
#[test] fn core4_s21() { sentron_core_check(21, 4); }
#[test] fn core4_s22() { sentron_core_check(22, 4); }
#[test] fn core4_s23() { sentron_core_check(23, 4); }
#[test] fn core4_s24() { sentron_core_check(24, 4); }
#[test] fn core4_s25() { sentron_core_check(25, 4); }
#[test] fn core4_s26() { sentron_core_check(26, 4); }
#[test] fn core4_s27() { sentron_core_check(27, 4); }
#[test] fn core4_s28() { sentron_core_check(28, 4); }
#[test] fn core4_s29() { sentron_core_check(29, 4); }
#[test] fn core4_s30() { sentron_core_check(30, 4); }
#[test] fn core4_s31() { sentron_core_check(31, 4); }
#[test] fn core4_s32() { sentron_core_check(32, 4); }
#[test] fn core4_s33() { sentron_core_check(33, 4); }
#[test] fn core4_s34() { sentron_core_check(34, 4); }
#[test] fn core4_s35() { sentron_core_check(35, 4); }
#[test] fn core4_s36() { sentron_core_check(36, 4); }
#[test] fn core4_s37() { sentron_core_check(37, 4); }
#[test] fn core4_s38() { sentron_core_check(38, 4); }
#[test] fn core4_s39() { sentron_core_check(39, 4); }
#[test] fn core5_s0() { sentron_core_check(0, 5); }
#[test] fn core5_s1() { sentron_core_check(1, 5); }
#[test] fn core5_s2() { sentron_core_check(2, 5); }
#[test] fn core5_s3() { sentron_core_check(3, 5); }
#[test] fn core5_s4() { sentron_core_check(4, 5); }
#[test] fn core5_s5() { sentron_core_check(5, 5); }
#[test] fn core5_s6() { sentron_core_check(6, 5); }
#[test] fn core5_s7() { sentron_core_check(7, 5); }
#[test] fn core5_s8() { sentron_core_check(8, 5); }
#[test] fn core5_s9() { sentron_core_check(9, 5); }
#[test] fn core5_s10() { sentron_core_check(10, 5); }
#[test] fn core5_s11() { sentron_core_check(11, 5); }
#[test] fn core5_s12() { sentron_core_check(12, 5); }
#[test] fn core5_s13() { sentron_core_check(13, 5); }
#[test] fn core5_s14() { sentron_core_check(14, 5); }
#[test] fn core5_s15() { sentron_core_check(15, 5); }
#[test] fn core5_s16() { sentron_core_check(16, 5); }
#[test] fn core5_s17() { sentron_core_check(17, 5); }
#[test] fn core5_s18() { sentron_core_check(18, 5); }
#[test] fn core5_s19() { sentron_core_check(19, 5); }
#[test] fn core5_s20() { sentron_core_check(20, 5); }
#[test] fn core5_s21() { sentron_core_check(21, 5); }
#[test] fn core5_s22() { sentron_core_check(22, 5); }
#[test] fn core5_s23() { sentron_core_check(23, 5); }
#[test] fn core5_s24() { sentron_core_check(24, 5); }
#[test] fn core5_s25() { sentron_core_check(25, 5); }
#[test] fn core5_s26() { sentron_core_check(26, 5); }
#[test] fn core5_s27() { sentron_core_check(27, 5); }
#[test] fn core5_s28() { sentron_core_check(28, 5); }
#[test] fn core5_s29() { sentron_core_check(29, 5); }
#[test] fn core5_s30() { sentron_core_check(30, 5); }
#[test] fn core5_s31() { sentron_core_check(31, 5); }
#[test] fn core5_s32() { sentron_core_check(32, 5); }
#[test] fn core5_s33() { sentron_core_check(33, 5); }
#[test] fn core5_s34() { sentron_core_check(34, 5); }
#[test] fn core5_s35() { sentron_core_check(35, 5); }
#[test] fn core5_s36() { sentron_core_check(36, 5); }
#[test] fn core5_s37() { sentron_core_check(37, 5); }
#[test] fn core5_s38() { sentron_core_check(38, 5); }
#[test] fn core5_s39() { sentron_core_check(39, 5); }
#[test] fn core6_s0() { sentron_core_check(0, 6); }
#[test] fn core6_s1() { sentron_core_check(1, 6); }
#[test] fn core6_s2() { sentron_core_check(2, 6); }
#[test] fn core6_s3() { sentron_core_check(3, 6); }
#[test] fn core6_s4() { sentron_core_check(4, 6); }
#[test] fn core6_s5() { sentron_core_check(5, 6); }
#[test] fn core6_s6() { sentron_core_check(6, 6); }
#[test] fn core6_s7() { sentron_core_check(7, 6); }
#[test] fn core6_s8() { sentron_core_check(8, 6); }
#[test] fn core6_s9() { sentron_core_check(9, 6); }
#[test] fn core6_s10() { sentron_core_check(10, 6); }
#[test] fn core6_s11() { sentron_core_check(11, 6); }
#[test] fn core6_s12() { sentron_core_check(12, 6); }
#[test] fn core6_s13() { sentron_core_check(13, 6); }
#[test] fn core6_s14() { sentron_core_check(14, 6); }
#[test] fn core6_s15() { sentron_core_check(15, 6); }
#[test] fn core6_s16() { sentron_core_check(16, 6); }
#[test] fn core6_s17() { sentron_core_check(17, 6); }
#[test] fn core6_s18() { sentron_core_check(18, 6); }
#[test] fn core6_s19() { sentron_core_check(19, 6); }
#[test] fn core6_s20() { sentron_core_check(20, 6); }
#[test] fn core6_s21() { sentron_core_check(21, 6); }
#[test] fn core6_s22() { sentron_core_check(22, 6); }
#[test] fn core6_s23() { sentron_core_check(23, 6); }
#[test] fn core6_s24() { sentron_core_check(24, 6); }
#[test] fn core6_s25() { sentron_core_check(25, 6); }
#[test] fn core6_s26() { sentron_core_check(26, 6); }
#[test] fn core6_s27() { sentron_core_check(27, 6); }
#[test] fn core6_s28() { sentron_core_check(28, 6); }
#[test] fn core6_s29() { sentron_core_check(29, 6); }
#[test] fn core6_s30() { sentron_core_check(30, 6); }
#[test] fn core6_s31() { sentron_core_check(31, 6); }
#[test] fn core6_s32() { sentron_core_check(32, 6); }
#[test] fn core6_s33() { sentron_core_check(33, 6); }
#[test] fn core6_s34() { sentron_core_check(34, 6); }
#[test] fn core6_s35() { sentron_core_check(35, 6); }
#[test] fn core6_s36() { sentron_core_check(36, 6); }
#[test] fn core6_s37() { sentron_core_check(37, 6); }
#[test] fn core6_s38() { sentron_core_check(38, 6); }
#[test] fn core6_s39() { sentron_core_check(39, 6); }
#[test] fn core7_s0() { sentron_core_check(0, 7); }
#[test] fn core7_s1() { sentron_core_check(1, 7); }
#[test] fn core7_s2() { sentron_core_check(2, 7); }
#[test] fn core7_s3() { sentron_core_check(3, 7); }
#[test] fn core7_s4() { sentron_core_check(4, 7); }
#[test] fn core7_s5() { sentron_core_check(5, 7); }
#[test] fn core7_s6() { sentron_core_check(6, 7); }
#[test] fn core7_s7() { sentron_core_check(7, 7); }
#[test] fn core7_s8() { sentron_core_check(8, 7); }
#[test] fn core7_s9() { sentron_core_check(9, 7); }
#[test] fn core7_s10() { sentron_core_check(10, 7); }
#[test] fn core7_s11() { sentron_core_check(11, 7); }
#[test] fn core7_s12() { sentron_core_check(12, 7); }
#[test] fn core7_s13() { sentron_core_check(13, 7); }
#[test] fn core7_s14() { sentron_core_check(14, 7); }
#[test] fn core7_s15() { sentron_core_check(15, 7); }
#[test] fn core7_s16() { sentron_core_check(16, 7); }
#[test] fn core7_s17() { sentron_core_check(17, 7); }
#[test] fn core7_s18() { sentron_core_check(18, 7); }
#[test] fn core7_s19() { sentron_core_check(19, 7); }
#[test] fn core7_s20() { sentron_core_check(20, 7); }
#[test] fn core7_s21() { sentron_core_check(21, 7); }
#[test] fn core7_s22() { sentron_core_check(22, 7); }
#[test] fn core7_s23() { sentron_core_check(23, 7); }
#[test] fn core7_s24() { sentron_core_check(24, 7); }
#[test] fn core7_s25() { sentron_core_check(25, 7); }
#[test] fn core7_s26() { sentron_core_check(26, 7); }
#[test] fn core7_s27() { sentron_core_check(27, 7); }
#[test] fn core7_s28() { sentron_core_check(28, 7); }
#[test] fn core7_s29() { sentron_core_check(29, 7); }
#[test] fn core7_s30() { sentron_core_check(30, 7); }
#[test] fn core7_s31() { sentron_core_check(31, 7); }
#[test] fn core7_s32() { sentron_core_check(32, 7); }
#[test] fn core7_s33() { sentron_core_check(33, 7); }
#[test] fn core7_s34() { sentron_core_check(34, 7); }
#[test] fn core7_s35() { sentron_core_check(35, 7); }
#[test] fn core7_s36() { sentron_core_check(36, 7); }
#[test] fn core7_s37() { sentron_core_check(37, 7); }
#[test] fn core7_s38() { sentron_core_check(38, 7); }
#[test] fn core7_s39() { sentron_core_check(39, 7); }
#[test] fn b256_byte_0() {
    let s = encode_byte(0);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(0));
}
#[test] fn b256_byte_1() {
    let s = encode_byte(1);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(1));
}
#[test] fn b256_byte_2() {
    let s = encode_byte(2);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(2));
}
#[test] fn b256_byte_3() {
    let s = encode_byte(3);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(3));
}
#[test] fn b256_byte_4() {
    let s = encode_byte(4);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(4));
}
#[test] fn b256_byte_5() {
    let s = encode_byte(5);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(5));
}
#[test] fn b256_byte_6() {
    let s = encode_byte(6);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(6));
}
#[test] fn b256_byte_7() {
    let s = encode_byte(7);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(7));
}
#[test] fn b256_byte_8() {
    let s = encode_byte(8);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(8));
}
#[test] fn b256_byte_9() {
    let s = encode_byte(9);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(9));
}
#[test] fn b256_byte_10() {
    let s = encode_byte(10);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(10));
}
#[test] fn b256_byte_11() {
    let s = encode_byte(11);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(11));
}
#[test] fn b256_byte_12() {
    let s = encode_byte(12);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(12));
}
#[test] fn b256_byte_13() {
    let s = encode_byte(13);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(13));
}
#[test] fn b256_byte_14() {
    let s = encode_byte(14);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(14));
}
#[test] fn b256_byte_15() {
    let s = encode_byte(15);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(15));
}
#[test] fn b256_byte_16() {
    let s = encode_byte(16);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(16));
}
#[test] fn b256_byte_17() {
    let s = encode_byte(17);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(17));
}
#[test] fn b256_byte_18() {
    let s = encode_byte(18);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(18));
}
#[test] fn b256_byte_19() {
    let s = encode_byte(19);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(19));
}
#[test] fn b256_byte_20() {
    let s = encode_byte(20);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(20));
}
#[test] fn b256_byte_21() {
    let s = encode_byte(21);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(21));
}
#[test] fn b256_byte_22() {
    let s = encode_byte(22);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(22));
}
#[test] fn b256_byte_23() {
    let s = encode_byte(23);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(23));
}
#[test] fn b256_byte_24() {
    let s = encode_byte(24);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(24));
}
#[test] fn b256_byte_25() {
    let s = encode_byte(25);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(25));
}
#[test] fn b256_byte_26() {
    let s = encode_byte(26);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(26));
}
#[test] fn b256_byte_27() {
    let s = encode_byte(27);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(27));
}
#[test] fn b256_byte_28() {
    let s = encode_byte(28);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(28));
}
#[test] fn b256_byte_29() {
    let s = encode_byte(29);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(29));
}
#[test] fn b256_byte_30() {
    let s = encode_byte(30);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(30));
}
#[test] fn b256_byte_31() {
    let s = encode_byte(31);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(31));
}
#[test] fn b256_byte_32() {
    let s = encode_byte(32);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(32));
}
#[test] fn b256_byte_33() {
    let s = encode_byte(33);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(33));
}
#[test] fn b256_byte_34() {
    let s = encode_byte(34);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(34));
}
#[test] fn b256_byte_35() {
    let s = encode_byte(35);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(35));
}
#[test] fn b256_byte_36() {
    let s = encode_byte(36);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(36));
}
#[test] fn b256_byte_37() {
    let s = encode_byte(37);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(37));
}
#[test] fn b256_byte_38() {
    let s = encode_byte(38);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(38));
}
#[test] fn b256_byte_39() {
    let s = encode_byte(39);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(39));
}
#[test] fn b256_byte_40() {
    let s = encode_byte(40);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(40));
}
#[test] fn b256_byte_41() {
    let s = encode_byte(41);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(41));
}
#[test] fn b256_byte_42() {
    let s = encode_byte(42);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(42));
}
#[test] fn b256_byte_43() {
    let s = encode_byte(43);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(43));
}
#[test] fn b256_byte_44() {
    let s = encode_byte(44);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(44));
}
#[test] fn b256_byte_45() {
    let s = encode_byte(45);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(45));
}
#[test] fn b256_byte_46() {
    let s = encode_byte(46);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(46));
}
#[test] fn b256_byte_47() {
    let s = encode_byte(47);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(47));
}
#[test] fn b256_byte_48() {
    let s = encode_byte(48);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(48));
}
#[test] fn b256_byte_49() {
    let s = encode_byte(49);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(49));
}
#[test] fn b256_byte_50() {
    let s = encode_byte(50);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(50));
}
#[test] fn b256_byte_51() {
    let s = encode_byte(51);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(51));
}
#[test] fn b256_byte_52() {
    let s = encode_byte(52);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(52));
}
#[test] fn b256_byte_53() {
    let s = encode_byte(53);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(53));
}
#[test] fn b256_byte_54() {
    let s = encode_byte(54);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(54));
}
#[test] fn b256_byte_55() {
    let s = encode_byte(55);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(55));
}
#[test] fn b256_byte_56() {
    let s = encode_byte(56);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(56));
}
#[test] fn b256_byte_57() {
    let s = encode_byte(57);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(57));
}
#[test] fn b256_byte_58() {
    let s = encode_byte(58);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(58));
}
#[test] fn b256_byte_59() {
    let s = encode_byte(59);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(59));
}
#[test] fn b256_byte_60() {
    let s = encode_byte(60);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(60));
}
#[test] fn b256_byte_61() {
    let s = encode_byte(61);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(61));
}
#[test] fn b256_byte_62() {
    let s = encode_byte(62);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(62));
}
#[test] fn b256_byte_63() {
    let s = encode_byte(63);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(63));
}
#[test] fn b256_byte_64() {
    let s = encode_byte(64);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(64));
}
#[test] fn b256_byte_65() {
    let s = encode_byte(65);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(65));
}
#[test] fn b256_byte_66() {
    let s = encode_byte(66);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(66));
}
#[test] fn b256_byte_67() {
    let s = encode_byte(67);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(67));
}
#[test] fn b256_byte_68() {
    let s = encode_byte(68);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(68));
}
#[test] fn b256_byte_69() {
    let s = encode_byte(69);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(69));
}
#[test] fn b256_byte_70() {
    let s = encode_byte(70);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(70));
}
#[test] fn b256_byte_71() {
    let s = encode_byte(71);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(71));
}
#[test] fn b256_byte_72() {
    let s = encode_byte(72);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(72));
}
#[test] fn b256_byte_73() {
    let s = encode_byte(73);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(73));
}
#[test] fn b256_byte_74() {
    let s = encode_byte(74);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(74));
}
#[test] fn b256_byte_75() {
    let s = encode_byte(75);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(75));
}
#[test] fn b256_byte_76() {
    let s = encode_byte(76);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(76));
}
#[test] fn b256_byte_77() {
    let s = encode_byte(77);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(77));
}
#[test] fn b256_byte_78() {
    let s = encode_byte(78);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(78));
}
#[test] fn b256_byte_79() {
    let s = encode_byte(79);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(79));
}
#[test] fn b256_byte_80() {
    let s = encode_byte(80);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(80));
}
#[test] fn b256_byte_81() {
    let s = encode_byte(81);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(81));
}
#[test] fn b256_byte_82() {
    let s = encode_byte(82);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(82));
}
#[test] fn b256_byte_83() {
    let s = encode_byte(83);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(83));
}
#[test] fn b256_byte_84() {
    let s = encode_byte(84);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(84));
}
#[test] fn b256_byte_85() {
    let s = encode_byte(85);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(85));
}
#[test] fn b256_byte_86() {
    let s = encode_byte(86);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(86));
}
#[test] fn b256_byte_87() {
    let s = encode_byte(87);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(87));
}
#[test] fn b256_byte_88() {
    let s = encode_byte(88);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(88));
}
#[test] fn b256_byte_89() {
    let s = encode_byte(89);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(89));
}
#[test] fn b256_byte_90() {
    let s = encode_byte(90);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(90));
}
#[test] fn b256_byte_91() {
    let s = encode_byte(91);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(91));
}
#[test] fn b256_byte_92() {
    let s = encode_byte(92);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(92));
}
#[test] fn b256_byte_93() {
    let s = encode_byte(93);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(93));
}
#[test] fn b256_byte_94() {
    let s = encode_byte(94);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(94));
}
#[test] fn b256_byte_95() {
    let s = encode_byte(95);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(95));
}
#[test] fn b256_byte_96() {
    let s = encode_byte(96);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(96));
}
#[test] fn b256_byte_97() {
    let s = encode_byte(97);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(97));
}
#[test] fn b256_byte_98() {
    let s = encode_byte(98);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(98));
}
#[test] fn b256_byte_99() {
    let s = encode_byte(99);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(99));
}
#[test] fn b256_byte_100() {
    let s = encode_byte(100);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(100));
}
#[test] fn b256_byte_101() {
    let s = encode_byte(101);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(101));
}
#[test] fn b256_byte_102() {
    let s = encode_byte(102);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(102));
}
#[test] fn b256_byte_103() {
    let s = encode_byte(103);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(103));
}
#[test] fn b256_byte_104() {
    let s = encode_byte(104);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(104));
}
#[test] fn b256_byte_105() {
    let s = encode_byte(105);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(105));
}
#[test] fn b256_byte_106() {
    let s = encode_byte(106);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(106));
}
#[test] fn b256_byte_107() {
    let s = encode_byte(107);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(107));
}
#[test] fn b256_byte_108() {
    let s = encode_byte(108);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(108));
}
#[test] fn b256_byte_109() {
    let s = encode_byte(109);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(109));
}
#[test] fn b256_byte_110() {
    let s = encode_byte(110);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(110));
}
#[test] fn b256_byte_111() {
    let s = encode_byte(111);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(111));
}
#[test] fn b256_byte_112() {
    let s = encode_byte(112);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(112));
}
#[test] fn b256_byte_113() {
    let s = encode_byte(113);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(113));
}
#[test] fn b256_byte_114() {
    let s = encode_byte(114);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(114));
}
#[test] fn b256_byte_115() {
    let s = encode_byte(115);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(115));
}
#[test] fn b256_byte_116() {
    let s = encode_byte(116);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(116));
}
#[test] fn b256_byte_117() {
    let s = encode_byte(117);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(117));
}
#[test] fn b256_byte_118() {
    let s = encode_byte(118);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(118));
}
#[test] fn b256_byte_119() {
    let s = encode_byte(119);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(119));
}
#[test] fn b256_byte_120() {
    let s = encode_byte(120);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(120));
}
#[test] fn b256_byte_121() {
    let s = encode_byte(121);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(121));
}
#[test] fn b256_byte_122() {
    let s = encode_byte(122);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(122));
}
#[test] fn b256_byte_123() {
    let s = encode_byte(123);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(123));
}
#[test] fn b256_byte_124() {
    let s = encode_byte(124);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(124));
}
#[test] fn b256_byte_125() {
    let s = encode_byte(125);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(125));
}
#[test] fn b256_byte_126() {
    let s = encode_byte(126);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(126));
}
#[test] fn b256_byte_127() {
    let s = encode_byte(127);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(127));
}
#[test] fn b256_byte_128() {
    let s = encode_byte(128);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(128));
}
#[test] fn b256_byte_129() {
    let s = encode_byte(129);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(129));
}
#[test] fn b256_byte_130() {
    let s = encode_byte(130);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(130));
}
#[test] fn b256_byte_131() {
    let s = encode_byte(131);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(131));
}
#[test] fn b256_byte_132() {
    let s = encode_byte(132);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(132));
}
#[test] fn b256_byte_133() {
    let s = encode_byte(133);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(133));
}
#[test] fn b256_byte_134() {
    let s = encode_byte(134);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(134));
}
#[test] fn b256_byte_135() {
    let s = encode_byte(135);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(135));
}
#[test] fn b256_byte_136() {
    let s = encode_byte(136);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(136));
}
#[test] fn b256_byte_137() {
    let s = encode_byte(137);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(137));
}
#[test] fn b256_byte_138() {
    let s = encode_byte(138);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(138));
}
#[test] fn b256_byte_139() {
    let s = encode_byte(139);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(139));
}
#[test] fn b256_byte_140() {
    let s = encode_byte(140);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(140));
}
#[test] fn b256_byte_141() {
    let s = encode_byte(141);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(141));
}
#[test] fn b256_byte_142() {
    let s = encode_byte(142);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(142));
}
#[test] fn b256_byte_143() {
    let s = encode_byte(143);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(143));
}
#[test] fn b256_byte_144() {
    let s = encode_byte(144);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(144));
}
#[test] fn b256_byte_145() {
    let s = encode_byte(145);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(145));
}
#[test] fn b256_byte_146() {
    let s = encode_byte(146);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(146));
}
#[test] fn b256_byte_147() {
    let s = encode_byte(147);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(147));
}
#[test] fn b256_byte_148() {
    let s = encode_byte(148);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(148));
}
#[test] fn b256_byte_149() {
    let s = encode_byte(149);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(149));
}
#[test] fn b256_byte_150() {
    let s = encode_byte(150);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(150));
}
#[test] fn b256_byte_151() {
    let s = encode_byte(151);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(151));
}
#[test] fn b256_byte_152() {
    let s = encode_byte(152);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(152));
}
#[test] fn b256_byte_153() {
    let s = encode_byte(153);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(153));
}
#[test] fn b256_byte_154() {
    let s = encode_byte(154);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(154));
}
#[test] fn b256_byte_155() {
    let s = encode_byte(155);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(155));
}
#[test] fn b256_byte_156() {
    let s = encode_byte(156);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(156));
}
#[test] fn b256_byte_157() {
    let s = encode_byte(157);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(157));
}
#[test] fn b256_byte_158() {
    let s = encode_byte(158);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(158));
}
#[test] fn b256_byte_159() {
    let s = encode_byte(159);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(159));
}
#[test] fn b256_byte_160() {
    let s = encode_byte(160);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(160));
}
#[test] fn b256_byte_161() {
    let s = encode_byte(161);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(161));
}
#[test] fn b256_byte_162() {
    let s = encode_byte(162);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(162));
}
#[test] fn b256_byte_163() {
    let s = encode_byte(163);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(163));
}
#[test] fn b256_byte_164() {
    let s = encode_byte(164);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(164));
}
#[test] fn b256_byte_165() {
    let s = encode_byte(165);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(165));
}
#[test] fn b256_byte_166() {
    let s = encode_byte(166);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(166));
}
#[test] fn b256_byte_167() {
    let s = encode_byte(167);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(167));
}
#[test] fn b256_byte_168() {
    let s = encode_byte(168);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(168));
}
#[test] fn b256_byte_169() {
    let s = encode_byte(169);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(169));
}
#[test] fn b256_byte_170() {
    let s = encode_byte(170);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(170));
}
#[test] fn b256_byte_171() {
    let s = encode_byte(171);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(171));
}
#[test] fn b256_byte_172() {
    let s = encode_byte(172);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(172));
}
#[test] fn b256_byte_173() {
    let s = encode_byte(173);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(173));
}
#[test] fn b256_byte_174() {
    let s = encode_byte(174);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(174));
}
#[test] fn b256_byte_175() {
    let s = encode_byte(175);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(175));
}
#[test] fn b256_byte_176() {
    let s = encode_byte(176);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(176));
}
#[test] fn b256_byte_177() {
    let s = encode_byte(177);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(177));
}
#[test] fn b256_byte_178() {
    let s = encode_byte(178);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(178));
}
#[test] fn b256_byte_179() {
    let s = encode_byte(179);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(179));
}
#[test] fn b256_byte_180() {
    let s = encode_byte(180);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(180));
}
#[test] fn b256_byte_181() {
    let s = encode_byte(181);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(181));
}
#[test] fn b256_byte_182() {
    let s = encode_byte(182);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(182));
}
#[test] fn b256_byte_183() {
    let s = encode_byte(183);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(183));
}
#[test] fn b256_byte_184() {
    let s = encode_byte(184);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(184));
}
#[test] fn b256_byte_185() {
    let s = encode_byte(185);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(185));
}
#[test] fn b256_byte_186() {
    let s = encode_byte(186);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(186));
}
#[test] fn b256_byte_187() {
    let s = encode_byte(187);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(187));
}
#[test] fn b256_byte_188() {
    let s = encode_byte(188);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(188));
}
#[test] fn b256_byte_189() {
    let s = encode_byte(189);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(189));
}
#[test] fn b256_byte_190() {
    let s = encode_byte(190);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(190));
}
#[test] fn b256_byte_191() {
    let s = encode_byte(191);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(191));
}
#[test] fn b256_byte_192() {
    let s = encode_byte(192);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(192));
}
#[test] fn b256_byte_193() {
    let s = encode_byte(193);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(193));
}
#[test] fn b256_byte_194() {
    let s = encode_byte(194);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(194));
}
#[test] fn b256_byte_195() {
    let s = encode_byte(195);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(195));
}
#[test] fn b256_byte_196() {
    let s = encode_byte(196);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(196));
}
#[test] fn b256_byte_197() {
    let s = encode_byte(197);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(197));
}
#[test] fn b256_byte_198() {
    let s = encode_byte(198);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(198));
}
#[test] fn b256_byte_199() {
    let s = encode_byte(199);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(199));
}
#[test] fn b256_byte_200() {
    let s = encode_byte(200);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(200));
}
#[test] fn b256_byte_201() {
    let s = encode_byte(201);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(201));
}
#[test] fn b256_byte_202() {
    let s = encode_byte(202);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(202));
}
#[test] fn b256_byte_203() {
    let s = encode_byte(203);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(203));
}
#[test] fn b256_byte_204() {
    let s = encode_byte(204);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(204));
}
#[test] fn b256_byte_205() {
    let s = encode_byte(205);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(205));
}
#[test] fn b256_byte_206() {
    let s = encode_byte(206);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(206));
}
#[test] fn b256_byte_207() {
    let s = encode_byte(207);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(207));
}
#[test] fn b256_byte_208() {
    let s = encode_byte(208);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(208));
}
#[test] fn b256_byte_209() {
    let s = encode_byte(209);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(209));
}
#[test] fn b256_byte_210() {
    let s = encode_byte(210);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(210));
}
#[test] fn b256_byte_211() {
    let s = encode_byte(211);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(211));
}
#[test] fn b256_byte_212() {
    let s = encode_byte(212);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(212));
}
#[test] fn b256_byte_213() {
    let s = encode_byte(213);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(213));
}
#[test] fn b256_byte_214() {
    let s = encode_byte(214);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(214));
}
#[test] fn b256_byte_215() {
    let s = encode_byte(215);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(215));
}
#[test] fn b256_byte_216() {
    let s = encode_byte(216);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(216));
}
#[test] fn b256_byte_217() {
    let s = encode_byte(217);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(217));
}
#[test] fn b256_byte_218() {
    let s = encode_byte(218);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(218));
}
#[test] fn b256_byte_219() {
    let s = encode_byte(219);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(219));
}
#[test] fn b256_byte_220() {
    let s = encode_byte(220);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(220));
}
#[test] fn b256_byte_221() {
    let s = encode_byte(221);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(221));
}
#[test] fn b256_byte_222() {
    let s = encode_byte(222);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(222));
}
#[test] fn b256_byte_223() {
    let s = encode_byte(223);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(223));
}
#[test] fn b256_byte_224() {
    let s = encode_byte(224);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(224));
}
#[test] fn b256_byte_225() {
    let s = encode_byte(225);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(225));
}
#[test] fn b256_byte_226() {
    let s = encode_byte(226);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(226));
}
#[test] fn b256_byte_227() {
    let s = encode_byte(227);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(227));
}
#[test] fn b256_byte_228() {
    let s = encode_byte(228);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(228));
}
#[test] fn b256_byte_229() {
    let s = encode_byte(229);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(229));
}
#[test] fn b256_byte_230() {
    let s = encode_byte(230);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(230));
}
#[test] fn b256_byte_231() {
    let s = encode_byte(231);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(231));
}
#[test] fn b256_byte_232() {
    let s = encode_byte(232);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(232));
}
#[test] fn b256_byte_233() {
    let s = encode_byte(233);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(233));
}
#[test] fn b256_byte_234() {
    let s = encode_byte(234);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(234));
}
#[test] fn b256_byte_235() {
    let s = encode_byte(235);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(235));
}
#[test] fn b256_byte_236() {
    let s = encode_byte(236);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(236));
}
#[test] fn b256_byte_237() {
    let s = encode_byte(237);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(237));
}
#[test] fn b256_byte_238() {
    let s = encode_byte(238);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(238));
}
#[test] fn b256_byte_239() {
    let s = encode_byte(239);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(239));
}
#[test] fn b256_byte_240() {
    let s = encode_byte(240);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(240));
}
#[test] fn b256_byte_241() {
    let s = encode_byte(241);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(241));
}
#[test] fn b256_byte_242() {
    let s = encode_byte(242);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(242));
}
#[test] fn b256_byte_243() {
    let s = encode_byte(243);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(243));
}
#[test] fn b256_byte_244() {
    let s = encode_byte(244);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(244));
}
#[test] fn b256_byte_245() {
    let s = encode_byte(245);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(245));
}
#[test] fn b256_byte_246() {
    let s = encode_byte(246);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(246));
}
#[test] fn b256_byte_247() {
    let s = encode_byte(247);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(247));
}
#[test] fn b256_byte_248() {
    let s = encode_byte(248);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(248));
}
#[test] fn b256_byte_249() {
    let s = encode_byte(249);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(249));
}
#[test] fn b256_byte_250() {
    let s = encode_byte(250);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(250));
}
#[test] fn b256_byte_251() {
    let s = encode_byte(251);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(251));
}
#[test] fn b256_byte_252() {
    let s = encode_byte(252);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(252));
}
#[test] fn b256_byte_253() {
    let s = encode_byte(253);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(253));
}
#[test] fn b256_byte_254() {
    let s = encode_byte(254);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(254));
}
#[test] fn b256_byte_255() {
    let s = encode_byte(255);
    assert_eq!(s.len(), 3);
    assert_eq!(decode_syllable(&s), Some(255));
}
#[test] fn resonance_at_1() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..1u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 1);
}
#[test] fn resonance_at_2() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..2u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 2);
}
#[test] fn resonance_at_5() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..5u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 5);
}
#[test] fn resonance_at_10() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..10u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 10);
}
#[test] fn resonance_at_20() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..20u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 20);
}
#[test] fn resonance_at_50() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..50u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 50);
}
#[test] fn resonance_at_100() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..100u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 100);
}
#[test] fn resonance_at_200() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..200u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 200);
}
#[test] fn resonance_at_500() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..500u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 500);
}
#[test] fn resonance_at_1000() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..1000u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 1000);
}
#[test] fn resonance_at_5000() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..5000u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 5000);
}
#[test] fn resonance_at_10000() {
    let mut s = SpanningSentron::new(0);
    s.bind_human(PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]));
    s.bind_mirror(PhextCoord::new([3,1,4,1,5,9,2,6,5,0,0]));
    for _ in 0..10000u64 { s.interact(); }
    assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    assert_eq!(s.interactions, 10000);
}
#[test] fn prog_len_1() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..1).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 1);
}
#[test] fn prog_len_2() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..2).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 2);
}
#[test] fn prog_len_3() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..3).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 3);
}
#[test] fn prog_len_4() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..4).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 4);
}
#[test] fn prog_len_5() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..5).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 5);
}
#[test] fn prog_len_6() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..6).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 6);
}
#[test] fn prog_len_7() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..7).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 7);
}
#[test] fn prog_len_8() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..8).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 8);
}
#[test] fn prog_len_9() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..9).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 9);
}
#[test] fn prog_len_10() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..10).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 10);
}
#[test] fn prog_len_11() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..11).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 11);
}
#[test] fn prog_len_12() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..12).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 12);
}
#[test] fn prog_len_13() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..13).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 13);
}
#[test] fn prog_len_14() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..14).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 14);
}
#[test] fn prog_len_15() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..15).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 15);
}
#[test] fn prog_len_16() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..16).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 16);
}
#[test] fn prog_len_17() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..17).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 17);
}
#[test] fn prog_len_18() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..18).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 18);
}
#[test] fn prog_len_19() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..19).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 19);
}
#[test] fn prog_len_20() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.regs.general[0] = 1;
    s.spawn((0..20).map(|_| SIW::new(DenseOp::DADD { rd: 0, rs1: 0, rs2: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())).collect());
    let mut mem = Memory::new();
    run(&mut s, &mut mem);
    assert_eq!(s.regs.general[0], 1i64 << 20);
}
#[test] fn mem_coord_0() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 1);
    assert_eq!(mem.gather_i64(&coord), 1);
}
#[test] fn mem_coord_1() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 43);
    assert_eq!(mem.gather_i64(&coord), 43);
}
#[test] fn mem_coord_2() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 85);
    assert_eq!(mem.gather_i64(&coord), 85);
}
#[test] fn mem_coord_3() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 127);
    assert_eq!(mem.gather_i64(&coord), 127);
}
#[test] fn mem_coord_4() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 169);
    assert_eq!(mem.gather_i64(&coord), 169);
}
#[test] fn mem_coord_5() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 211);
    assert_eq!(mem.gather_i64(&coord), 211);
}
#[test] fn mem_coord_6() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 253);
    assert_eq!(mem.gather_i64(&coord), 253);
}
#[test] fn mem_coord_7() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 295);
    assert_eq!(mem.gather_i64(&coord), 295);
}
#[test] fn mem_coord_8() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 337);
    assert_eq!(mem.gather_i64(&coord), 337);
}
#[test] fn mem_coord_9() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 379);
    assert_eq!(mem.gather_i64(&coord), 379);
}
#[test] fn mem_coord_10() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 421);
    assert_eq!(mem.gather_i64(&coord), 421);
}
#[test] fn mem_coord_11() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 463);
    assert_eq!(mem.gather_i64(&coord), 463);
}
#[test] fn mem_coord_12() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 505);
    assert_eq!(mem.gather_i64(&coord), 505);
}
#[test] fn mem_coord_13() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 547);
    assert_eq!(mem.gather_i64(&coord), 547);
}
#[test] fn mem_coord_14() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 589);
    assert_eq!(mem.gather_i64(&coord), 589);
}
#[test] fn mem_coord_15() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 631);
    assert_eq!(mem.gather_i64(&coord), 631);
}
#[test] fn mem_coord_16() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 673);
    assert_eq!(mem.gather_i64(&coord), 673);
}
#[test] fn mem_coord_17() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 715);
    assert_eq!(mem.gather_i64(&coord), 715);
}
#[test] fn mem_coord_18() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 757);
    assert_eq!(mem.gather_i64(&coord), 757);
}
#[test] fn mem_coord_19() {
    let mut mem = Memory::new();
    let coord = PhextCoord::new([19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    mem.scatter_i64(&coord, 799);
    assert_eq!(mem.gather_i64(&coord), 799);
}
#[test] fn cost_1000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 1000, duration: Duration::from_millis(1), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn cost_10000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 10000, duration: Duration::from_millis(1), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn cost_100000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 100000, duration: Duration::from_millis(10), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn cost_1000000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 1000000, duration: Duration::from_millis(100), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn cost_10000000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 10000000, duration: Duration::from_millis(1000), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn cost_100000000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 100000000, duration: Duration::from_millis(1000), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn cost_1000000000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 1000000000, duration: Duration::from_millis(1000), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn cost_10000000000() {
    let hw = HardwareCost::ranch_node();
    let br = BenchResult { label: "t".into(), ops: 10000000000, duration: Duration::from_millis(10000), sentron_count: 40 };
    let ca = CostAnalysis::new(hw, br);
    assert!(ca.gops() >= 0.0);
}
#[test] fn wp_u0_d0() {
    let mut w = NeuronWiring::new();
    w.upstream[0] = 1;
    w.downstream[0] = 10;
    assert!(w.is_neighbor(1));
    assert!(w.is_neighbor(10));
}
#[test] fn wp_u0_d1() {
    let mut w = NeuronWiring::new();
    w.upstream[0] = 1;
    w.downstream[1] = 11;
    assert!(w.is_neighbor(1));
    assert!(w.is_neighbor(11));
}
#[test] fn wp_u0_d2() {
    let mut w = NeuronWiring::new();
    w.upstream[0] = 1;
    w.downstream[2] = 12;
    assert!(w.is_neighbor(1));
    assert!(w.is_neighbor(12));
}
#[test] fn wp_u0_d3() {
    let mut w = NeuronWiring::new();
    w.upstream[0] = 1;
    w.downstream[3] = 13;
    assert!(w.is_neighbor(1));
    assert!(w.is_neighbor(13));
}
#[test] fn wp_u1_d0() {
    let mut w = NeuronWiring::new();
    w.upstream[1] = 2;
    w.downstream[0] = 10;
    assert!(w.is_neighbor(2));
    assert!(w.is_neighbor(10));
}
#[test] fn wp_u1_d1() {
    let mut w = NeuronWiring::new();
    w.upstream[1] = 2;
    w.downstream[1] = 11;
    assert!(w.is_neighbor(2));
    assert!(w.is_neighbor(11));
}
#[test] fn wp_u1_d2() {
    let mut w = NeuronWiring::new();
    w.upstream[1] = 2;
    w.downstream[2] = 12;
    assert!(w.is_neighbor(2));
    assert!(w.is_neighbor(12));
}
#[test] fn wp_u1_d3() {
    let mut w = NeuronWiring::new();
    w.upstream[1] = 2;
    w.downstream[3] = 13;
    assert!(w.is_neighbor(2));
    assert!(w.is_neighbor(13));
}
#[test] fn wp_u2_d0() {
    let mut w = NeuronWiring::new();
    w.upstream[2] = 3;
    w.downstream[0] = 10;
    assert!(w.is_neighbor(3));
    assert!(w.is_neighbor(10));
}
#[test] fn wp_u2_d1() {
    let mut w = NeuronWiring::new();
    w.upstream[2] = 3;
    w.downstream[1] = 11;
    assert!(w.is_neighbor(3));
    assert!(w.is_neighbor(11));
}
#[test] fn wp_u2_d2() {
    let mut w = NeuronWiring::new();
    w.upstream[2] = 3;
    w.downstream[2] = 12;
    assert!(w.is_neighbor(3));
    assert!(w.is_neighbor(12));
}
#[test] fn wp_u2_d3() {
    let mut w = NeuronWiring::new();
    w.upstream[2] = 3;
    w.downstream[3] = 13;
    assert!(w.is_neighbor(3));
    assert!(w.is_neighbor(13));
}
#[test] fn wp_u3_d0() {
    let mut w = NeuronWiring::new();
    w.upstream[3] = 4;
    w.downstream[0] = 10;
    assert!(w.is_neighbor(4));
    assert!(w.is_neighbor(10));
}
#[test] fn wp_u3_d1() {
    let mut w = NeuronWiring::new();
    w.upstream[3] = 4;
    w.downstream[1] = 11;
    assert!(w.is_neighbor(4));
    assert!(w.is_neighbor(11));
}
#[test] fn wp_u3_d2() {
    let mut w = NeuronWiring::new();
    w.upstream[3] = 4;
    w.downstream[2] = 12;
    assert!(w.is_neighbor(4));
    assert!(w.is_neighbor(12));
}
#[test] fn wp_u3_d3() {
    let mut w = NeuronWiring::new();
    w.upstream[3] = 4;
    w.downstream[3] = 13;
    assert!(w.is_neighbor(4));
    assert!(w.is_neighbor(13));
}
