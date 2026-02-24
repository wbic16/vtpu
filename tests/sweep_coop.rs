//! Coop SMT sweep: scheduling, quantum, interleaving
//! R23W29 — Theia 💎

use vtpu_runtime::coop_smt::*;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::siw::SIW;
use vtpu_runtime::pipes::*;

fn nop() -> SIW { SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()) }
fn add() -> SIW { SIW::new(DenseOp::DADD{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()) }
fn sentron(id: u16, n: usize) -> Sentron { let mut s = Sentron::new(id, PhextCoord::zero(), 0, 0); s.spawn(vec![add(); n]); s }

// Single sentron program length sweep
macro_rules! single_test {
    ($name:ident, $n:expr) => {
        #[test] fn $name() {
            let mut s = vec![sentron(0, $n)];
            let mut mem = Memory::new();
            let r = coop_execute(&mut s, &mut mem, &CoopConfig::default());
            assert_eq!(r.total_retired, $n as u64);
        }
    }
}

single_test!(s1_1, 1); single_test!(s1_2, 2); single_test!(s1_4, 4); single_test!(s1_8, 8);
single_test!(s1_9, 9); single_test!(s1_16, 16); single_test!(s1_32, 32); single_test!(s1_40, 40);
single_test!(s1_64, 64); single_test!(s1_100, 100); single_test!(s1_360, 360);

// Multi-sentron count sweep
macro_rules! multi_test {
    ($name:ident, $n_sentrons:expr, $prog_len:expr) => {
        #[test] fn $name() {
            let mut s: Vec<Sentron> = (0..$n_sentrons).map(|i| sentron(i, $prog_len)).collect();
            let mut mem = Memory::new();
            let r = coop_execute(&mut s, &mut mem, &CoopConfig{quantum:4,max_cycles:1_000_000});
            assert_eq!(r.total_retired, ($n_sentrons * $prog_len) as u64);
        }
    }
}

multi_test!(m_2_8, 2, 8); multi_test!(m_3_8, 3, 8); multi_test!(m_4_8, 4, 8);
multi_test!(m_5_8, 5, 8); multi_test!(m_8_8, 8, 8); multi_test!(m_9_8, 9, 8);
multi_test!(m_9_40, 9, 40); multi_test!(m_40_9, 40, 9); multi_test!(m_9_9, 9, 9);
multi_test!(m_2_100, 2, 100); multi_test!(m_4_100, 4, 100);

// Quantum sweep
macro_rules! quantum_test {
    ($name:ident, $n:expr, $prog:expr, $q:expr) => {
        #[test] fn $name() {
            let mut s: Vec<Sentron> = (0..$n).map(|i| sentron(i, $prog)).collect();
            let mut mem = Memory::new();
            let r = coop_execute(&mut s, &mut mem, &CoopConfig{quantum:$q,max_cycles:1_000_000});
            assert_eq!(r.total_retired, ($n * $prog) as u64);
        }
    }
}

quantum_test!(q_4_16_1, 4, 16, 1);
quantum_test!(q_4_16_2, 4, 16, 2);
quantum_test!(q_4_16_4, 4, 16, 4);
quantum_test!(q_4_16_8, 4, 16, 8);
quantum_test!(q_4_16_16, 4, 16, 16);
quantum_test!(q_9_9_1, 9, 9, 1);
quantum_test!(q_9_9_3, 9, 9, 3);
quantum_test!(q_9_9_9, 9, 9, 9);

// Max cycles limit sweep
macro_rules! maxcyc_test {
    ($name:ident, $prog:expr, $max:expr) => {
        #[test] fn $name() {
            let mut s = vec![sentron(0, $prog)];
            let mut mem = Memory::new();
            let r = coop_execute(&mut s, &mut mem, &CoopConfig{quantum:4,max_cycles:$max});
            // Should stop around max_cycles (may overshoot by one quantum)
            assert!(r.total_retired <= $max + 4);
        }
    }
}

maxcyc_test!(mc_1000_10, 1000, 10);
maxcyc_test!(mc_1000_50, 1000, 50);
maxcyc_test!(mc_1000_100, 1000, 100);

// Context switch count
macro_rules! ctxsw_test {
    ($name:ident, $n:expr, $prog:expr, $q:expr) => {
        #[test] fn $name() {
            let mut s: Vec<Sentron> = (0..$n).map(|i| sentron(i, $prog)).collect();
            let mut mem = Memory::new();
            let r = coop_execute(&mut s, &mut mem, &CoopConfig{quantum:$q,max_cycles:1_000_000});
            assert!(r.context_switches > 0);
        }
    }
}

ctxsw_test!(cs_2_8_4, 2, 8, 4);
ctxsw_test!(cs_4_16_4, 4, 16, 4);
ctxsw_test!(cs_9_36_4, 9, 36, 4);

// Harmonic configurations (9×40=360, 8×45=360, 5×72=360)
#[test] fn harmonic_9x40() { 
    let mut s: Vec<Sentron> = (0..9).map(|i| sentron(i, 40)).collect();
    let mut mem = Memory::new();
    let r = coop_execute(&mut s, &mut mem, &CoopConfig::default());
    assert_eq!(r.total_retired, 360);
}

#[test] fn harmonic_40x9() {
    let mut s: Vec<Sentron> = (0..40).map(|i| sentron(i, 9)).collect();
    let mut mem = Memory::new();
    let r = coop_execute(&mut s, &mut mem, &CoopConfig::default());
    assert_eq!(r.total_retired, 360);
}

#[test] fn harmonic_8x45() {
    let mut s: Vec<Sentron> = (0..8).map(|i| sentron(i, 45)).collect();
    let mut mem = Memory::new();
    let r = coop_execute(&mut s, &mut mem, &CoopConfig::default());
    assert_eq!(r.total_retired, 360);
}

#[test] fn harmonic_5x72() {
    let mut s: Vec<Sentron> = (0..5).map(|i| sentron(i, 72)).collect();
    let mut mem = Memory::new();
    let r = coop_execute(&mut s, &mut mem, &CoopConfig::default());
    assert_eq!(r.total_retired, 360);
}

// Empty cases
#[test] fn empty_sentrons() {
    let mut s: Vec<Sentron> = vec![];
    let mut mem = Memory::new();
    let r = coop_execute(&mut s, &mut mem, &CoopConfig::default());
    assert_eq!(r.total_retired, 0);
}

#[test] fn empty_program() {
    let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
    s.spawn(vec![]);
    let mut sentrons = vec![s];
    let mut mem = Memory::new();
    let r = coop_execute(&mut sentrons, &mut mem, &CoopConfig::default());
    assert_eq!(r.total_retired, 0);
}
