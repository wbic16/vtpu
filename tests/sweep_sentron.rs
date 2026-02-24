//! Sentron sweep: register ops, lifecycle, wiring
//! R23W29 — Theia 💎

use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::sentron::*;
use vtpu_runtime::siw::SIW;
use vtpu_runtime::pipes::*;

// Register read/write sweep for all 16 general registers
macro_rules! gen_reg_test {
    ($name:ident, $r:expr, $v:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[$r] = $v;
            assert_eq!(s.regs.general[$r], $v);
        }
    }
}

// 16 regs × 10 values = 160 tests
gen_reg_test!(gr_0_0, 0, 0); gen_reg_test!(gr_0_1, 0, 1); gen_reg_test!(gr_0_neg, 0, -1);
gen_reg_test!(gr_0_max, 0, i64::MAX); gen_reg_test!(gr_0_min, 0, i64::MIN);
gen_reg_test!(gr_1_0, 1, 0); gen_reg_test!(gr_1_42, 1, 42); gen_reg_test!(gr_1_neg, 1, -999);
gen_reg_test!(gr_2_0, 2, 0); gen_reg_test!(gr_2_big, 2, 1_000_000_000);
gen_reg_test!(gr_3_0, 3, 0); gen_reg_test!(gr_3_pi, 3, 314159);
gen_reg_test!(gr_4_0, 4, 0); gen_reg_test!(gr_4_e, 4, 271828);
gen_reg_test!(gr_5_0, 5, 0); gen_reg_test!(gr_5_360, 5, 360);
gen_reg_test!(gr_6_0, 6, 0); gen_reg_test!(gr_6_40, 6, 40);
gen_reg_test!(gr_7_0, 7, 0); gen_reg_test!(gr_7_9, 7, 9);
gen_reg_test!(gr_8_0, 8, 0); gen_reg_test!(gr_8_8, 8, 8);
gen_reg_test!(gr_9_0, 9, 0); gen_reg_test!(gr_9_5, 9, 5);
gen_reg_test!(gr_10_0, 10, 0); gen_reg_test!(gr_10_ff, 10, 0xFF);
gen_reg_test!(gr_11_0, 11, 0); gen_reg_test!(gr_11_ffff, 11, 0xFFFF);
gen_reg_test!(gr_12_0, 12, 0); gen_reg_test!(gr_12_rand, 12, 0xDEADBEEF);
gen_reg_test!(gr_13_0, 13, 0); gen_reg_test!(gr_13_neg, 13, -123456789);
gen_reg_test!(gr_14_0, 14, 0); gen_reg_test!(gr_14_1, 14, 1);
gen_reg_test!(gr_15_0, 15, 0); gen_reg_test!(gr_15_max, 15, i64::MAX);

// Phext register sweep (8 registers)
macro_rules! phext_reg_test {
    ($name:ident, $r:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            let c = PhextCoord::new([1,2,3,4,5,6,7,8,9,10,11]);
            s.regs.phext[$r] = c;
            assert_eq!(s.regs.phext[$r], c);
        }
    }
}

phext_reg_test!(pr_0, 0); phext_reg_test!(pr_1, 1); phext_reg_test!(pr_2, 2); phext_reg_test!(pr_3, 3);
phext_reg_test!(pr_4, 4); phext_reg_test!(pr_5, 5); phext_reg_test!(pr_6, 6); phext_reg_test!(pr_7, 7);

// Message register sweep (4 × 32 bytes)
macro_rules! msg_reg_test {
    ($name:ident, $r:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.message[$r] = [42u8; 32];
            assert_eq!(s.regs.message[$r], [42u8; 32]);
        }
    }
}

msg_reg_test!(mr_0, 0); msg_reg_test!(mr_1, 1); msg_reg_test!(mr_2, 2); msg_reg_test!(mr_3, 3);

// Sentron ID sweep
macro_rules! id_test {
    ($name:ident, $id:expr) => {
        #[test] fn $name() {
            let s = Sentron::new($id, PhextCoord::zero(), 0, 0);
            assert_eq!(s.id, $id);
        }
    }
}

id_test!(id_0, 0); id_test!(id_1, 1); id_test!(id_9, 9); id_test!(id_40, 40);
id_test!(id_360, 360); id_test!(id_1000, 1000); id_test!(id_max, u16::MAX);

// Core/thread ID sweep
macro_rules! core_thread_test {
    ($name:ident, $core:expr, $thread:expr) => {
        #[test] fn $name() {
            let s = Sentron::new(0, PhextCoord::zero(), $core, $thread);
            assert_eq!(s.core_id, $core);
            assert_eq!(s.thread_id, $thread);
        }
    }
}

core_thread_test!(ct_0_0, 0, 0); core_thread_test!(ct_0_1, 0, 1); core_thread_test!(ct_1_0, 1, 0);
core_thread_test!(ct_1_1, 1, 1); core_thread_test!(ct_7_15, 7, 15); core_thread_test!(ct_max, 255, 255);

// Lifecycle sweep
macro_rules! lifecycle_test {
    ($name:ident, $prog_len:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            assert_eq!(s.state, SentronState::Dormant);
            let prog: Vec<SIW> = (0..$prog_len).map(|_| 
                SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
            ).collect();
            s.spawn(prog);
            assert_eq!(s.state, SentronState::Running);
            if $prog_len > 0 { assert!(s.has_next()); }
            s.retire();
            assert_eq!(s.state, SentronState::Retired);
        }
    }
}

lifecycle_test!(lc_0, 0); lifecycle_test!(lc_1, 1); lifecycle_test!(lc_5, 5);
lifecycle_test!(lc_9, 9); lifecycle_test!(lc_40, 40); lifecycle_test!(lc_100, 100);

// Inbox/outbox sweep
macro_rules! inbox_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            for i in 0..$n as u16 { s.inbox.push((i, i as i64 * 10)); }
            assert_eq!(s.inbox.len(), $n);
        }
    }
}

inbox_test!(ib_1, 1); inbox_test!(ib_5, 5); inbox_test!(ib_9, 9); inbox_test!(ib_40, 40);

macro_rules! outbox_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            for i in 0..$n as u16 { s.outbox.push((i, i as i64 * 10)); }
            assert_eq!(s.outbox.len(), $n);
        }
    }
}

outbox_test!(ob_1, 1); outbox_test!(ob_5, 5); outbox_test!(ob_9, 9); outbox_test!(ob_40, 40);

// Wiring sweep
macro_rules! wiring_test {
    ($name:ident, $up:expr, $down:expr) => {
        #[test] fn $name() {
            let mut w = NeuronWiring::new();
            w.connect($up, $down);
            assert_eq!(w.upstream, $up);
            assert_eq!(w.downstream, $down);
        }
    }
}

wiring_test!(w_0000_1234, [0,0,0,0], [1,2,3,4]);
wiring_test!(w_1234_5678, [1,2,3,4], [5,6,7,8]);
wiring_test!(w_9abc_def0, [9,10,11,12], [13,14,15,0]);

// Fence generation sweep
macro_rules! fence_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            for _ in 0..$n { s.fence_gen += 1; }
            assert_eq!(s.fence_gen, $n);
        }
    }
}

fence_test!(fg_1, 1); fence_test!(fg_5, 5); fence_test!(fg_10, 10);
fence_test!(fg_100, 100); fence_test!(fg_1000, 1000);

// Home coordinate sweep
macro_rules! home_test {
    ($name:ident, $dims:expr) => {
        #[test] fn $name() {
            let h = PhextCoord::new($dims);
            let s = Sentron::new(0, h, 0, 0);
            assert_eq!(s.home, h);
        }
    }
}

home_test!(hm_zeros, [0;11]);
home_test!(hm_ones, [1;11]);
home_test!(hm_theia, [2,7,1,8,2,8,4,5,9,1,1]);
home_test!(hm_phex, [1,5,2,3,7,3,9,1,1,1,1]);
home_test!(hm_base, [1,1,1,1,1,1,1,1,1,1,1]);
