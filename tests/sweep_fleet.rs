//! Fleet sweep: topology, messaging, barrier tests across fleet sizes
//! R23W29 — Theia 💎

use vtpu_runtime::fleet::Fleet;
use vtpu_runtime::pipes::CoordOp;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::siw::SIW;
use vtpu_runtime::pipes::{DenseOp, SparseOp};
use vtpu_runtime::memory::Memory;
use vtpu_runtime::coop_fleet::coop_fleet_execute;

// Fleet size sweep
macro_rules! fleet_size_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let f = Fleet::new($n);
            assert_eq!(f.size(), $n);
            for i in 0..$n as u16 {
                assert!(f.sentron(i).is_some());
                assert!(f.wiring(i).is_some());
            }
            assert!(f.sentron($n as u16).is_none());
        }
    }
}

fleet_size_test!(fleet_4, 4);
fleet_size_test!(fleet_5, 5);
fleet_size_test!(fleet_8, 8);
fleet_size_test!(fleet_9, 9);
fleet_size_test!(fleet_16, 16);
fleet_size_test!(fleet_32, 32);
fleet_size_test!(fleet_40, 40);
fleet_size_test!(fleet_64, 64);
fleet_size_test!(fleet_128, 128);
fleet_size_test!(fleet_256, 256);
fleet_size_test!(fleet_360, 360);

// Ring wiring: every sentron has correct neighbor IDs
macro_rules! ring_wiring_test {
    ($name:ident, $size:expr, $id:expr) => {
        #[test] fn $name() {
            let f = Fleet::new($size);
            let w = f.wiring($id).unwrap();
            let n = $size as u16;
            assert_eq!(w.downstream[0], ($id + 1) % n);
            assert_eq!(w.upstream[0], ($id + n - 1) % n);
        }
    }
}

ring_wiring_test!(ring_10_s0, 10, 0);
ring_wiring_test!(ring_10_s1, 10, 1);
ring_wiring_test!(ring_10_s5, 10, 5);
ring_wiring_test!(ring_10_s9, 10, 9);
ring_wiring_test!(ring_40_s0, 40, 0);
ring_wiring_test!(ring_40_s20, 40, 20);
ring_wiring_test!(ring_40_s39, 40, 39);
ring_wiring_test!(ring_360_s0, 360, 0);
ring_wiring_test!(ring_360_s180, 360, 180);
ring_wiring_test!(ring_360_s359, 360, 359);

// Barrier sweep: groups of varying sizes
macro_rules! barrier_test {
    ($name:ident, $fleet_size:expr, $group:expr) => {
        #[test] fn $name() {
            let mut f = Fleet::new($fleet_size);
            for i in 0..$group as u16 {
                f.dispatch(i, &CoordOp::CBAR{barrier_id:0, count:$group});
            }
            assert_eq!(f.total_barrier_syncs(), $group as u64);
        }
    }
}

barrier_test!(barrier_2, 10, 2);
barrier_test!(barrier_3, 10, 3);
barrier_test!(barrier_4, 10, 4);
barrier_test!(barrier_5, 10, 5);
barrier_test!(barrier_8, 10, 8);
barrier_test!(barrier_9, 10, 9);
barrier_test!(barrier_10, 10, 10);
barrier_test!(barrier_40, 40, 40);

// Message relay chains of various lengths
macro_rules! relay_test {
    ($name:ident, $chain_len:expr) => {
        #[test] fn $name() {
            let n = $chain_len + 1;
            let mut f = Fleet::new(n);
            f.sentron_mut(0).unwrap().regs.general[0] = 42;

            // Build chain: 0→1→2→...→chain_len
            let nop = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
            let send = |dest: u8| SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CSEND{msg_reg:0,dest_sentron:dest}, PhextCoord::zero());
            let recv = |src: u8| SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CRECV{rd:0,src_sentron:src}, PhextCoord::zero());

            // Sentron 0: send to 1
            f.sentron_mut(0).unwrap().spawn(vec![send(1)]);

            // Middle sentrons: wait, recv, send
            for i in 1..$chain_len as u16 {
                let mut prog = vec![nop.clone(); i as usize]; // delay
                prog.push(recv((i - 1) as u8));
                prog.push(send((i + 1) as u8));
                f.sentron_mut(i).unwrap().spawn(prog);
            }

            // Last sentron: wait, recv
            let last = $chain_len as u16;
            let mut prog = vec![nop.clone(); last as usize];
            prog.push(recv((last - 1) as u8));
            f.sentron_mut(last).unwrap().spawn(prog);

            let mut mem = Memory::new();
            coop_fleet_execute(&mut f, &mut mem, 1, 100_000);
            let val = f.sentron(last).unwrap().regs.general[0];
            assert_eq!(val, 42, "chain length {} failed: got {}", $chain_len, val);
        }
    }
}

relay_test!(relay_4, 4);
relay_test!(relay_5, 5);
relay_test!(relay_8, 8);

// Coop fleet with varying quantum sizes
macro_rules! quantum_test {
    ($name:ident, $sentrons:expr, $program_len:expr, $quantum:expr) => {
        #[test] fn $name() {
            let nop = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
            let mut f = Fleet::new($sentrons);
            for i in 0..$sentrons as u16 {
                f.sentron_mut(i).unwrap().spawn(vec![nop.clone(); $program_len]);
            }
            let mut mem = Memory::new();
            let r = coop_fleet_execute(&mut f, &mut mem, $quantum, 1_000_000);
            assert_eq!(r.total_retired, ($sentrons * $program_len) as u64);
        }
    }
}

quantum_test!(q1_s4_p8, 4, 8, 1);
quantum_test!(q2_s4_p8, 4, 8, 2);
quantum_test!(q4_s4_p8, 4, 8, 4);
quantum_test!(q8_s4_p8, 4, 8, 8);
quantum_test!(q1_s9_p9, 9, 9, 1);
quantum_test!(q3_s9_p9, 9, 9, 3);
quantum_test!(q9_s9_p9, 9, 9, 9);
quantum_test!(q1_s40_p1, 40, 1, 1);
quantum_test!(q4_s40_p4, 40, 4, 4);
quantum_test!(q1_s360_p1, 360, 1, 1);

// Fanout: verify broadcast to downstream
#[test]
fn fanout_broadcasts() {
    let mut f = Fleet::new(10);
    f.sentron_mut(0).unwrap().regs.general[0] = 99;
    f.dispatch(0, &CoordOp::CSEND{msg_reg:0, dest_sentron:1});
    f.dispatch(0, &CoordOp::CROUTE{msg_reg:0, dest_node:0});
    f.flush();
    // At least sentron 1 should have the message in inbox
    assert!(!f.sentron(1).unwrap().inbox.is_empty());
}

// Stats consistency
#[test]
fn fleet_stats_consistent() {
    let mut f = Fleet::new(10);
    f.sentron_mut(0).unwrap().regs.general[0] = 1;
    for i in 1..5u8 {
        f.dispatch(0, &CoordOp::CSEND{msg_reg:0, dest_sentron:i});
    }
    assert_eq!(f.total_sends(), 4);
    assert_eq!(f.total_recvs(), 0);
    for i in 1..5u16 {
        f.dispatch(i, &CoordOp::CRECV{rd:0, src_sentron:0});
    }
    assert_eq!(f.total_recvs(), 4);
}
