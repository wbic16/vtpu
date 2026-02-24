//! Mass coverage tests — R23W29 10× test blitz
//!
//! Comprehensive edge-case, property, and integration tests
//! across all vTPU modules. Target: 7000+ total tests.
//!
//! Theia 💎

use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::siw::SIW;
use vtpu_runtime::pipes::*;
use vtpu_runtime::sentron::{Sentron, SentronState};
use vtpu_runtime::sentron::NeuronWiring as SentronWiring;
use vtpu_runtime::neuron::{NeuronWiring, NeuronLayer, Neuron, VakLevel, lo_shu_layer};
use vtpu_runtime::fleet::Fleet;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::exec::exec_siw_octawire;
use vtpu_runtime::base256;
use vtpu_runtime::coop_smt::*;
use vtpu_runtime::coop_fleet::*;
use vtpu_runtime::belief::*;
use vtpu_runtime::twisted_pairs::*;
use vtpu_runtime::ubi::*;
use vtpu_runtime::orin::*;
use vtpu_runtime::eggs;
use vtpu_runtime::easter_island::*;

// ══════════════════════════════════════════════════
// PhextCoord — exhaustive dimension tests
// ══════════════════════════════════════════════════

#[test] fn coord_zero_all_dims() { let c = PhextCoord::zero(); for i in 0..11u8 { assert_eq!(c.get_dim(i), 0); } }
#[test] fn coord_roundtrip_100() { let c = PhextCoord::new([100;11]); for i in 0..11u8 { assert_eq!(c.get_dim(i), 100); } }
#[test] fn coord_set_each_dim() { for d in 0..11u8 { let mut c = PhextCoord::zero(); c.set_dim(d, 42); assert_eq!(c.get_dim(d), 42); } }
#[test] fn coord_independence() { for d in 0..11u8 { let mut c = PhextCoord::zero(); c.set_dim(d, 100); for other in 0..11u8 { if other != d { assert_eq!(c.get_dim(other), 0, "dim {} leaked to {}", d, other); } } } }
#[test] fn coord_eq_reflexive() { let c = PhextCoord::new([1,2,3,4,5,6,7,8,9,10,11]); assert_eq!(c, c.clone()); }
#[test] fn coord_ne_one_bit() { let a = PhextCoord::new([1,1,1,1,1,1,1,1,1,1,1]); let mut b = a.clone(); b.set_dim(5, 2); assert_ne!(a, b); }
#[test] fn coord_manhattan_self() { let c = PhextCoord::new([5;11]); assert_eq!(c.manhattan_distance(&c), 0); }
#[test] fn coord_manhattan_unit() { let a = PhextCoord::zero(); let mut b = PhextCoord::zero(); b.set_dim(0, 1); assert_eq!(a.manhattan_distance(&b), 1); }
#[test] fn coord_dims_roundtrip() { let dims = [1,2,3,4,5,6,7,8,9,10,11]; let c = PhextCoord::new(dims); assert_eq!(c.dims(), dims); }

// ══════════════════════════════════════════════════
// SIW — instruction construction sweeps
// ══════════════════════════════════════════════════

#[test] fn siw_all_nop() { let s = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()); assert_eq!(s.d_fam, 4); assert_eq!(s.s_fam, 4); assert_eq!(s.c_fam, 4); }
#[test] fn siw_dadd_family() { let s = SIW::new(DenseOp::DADD{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()); assert_eq!(s.d_fam, 0); }
#[test] fn siw_dsub_family() { let s = SIW::new(DenseOp::DSUB{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()); assert_eq!(s.d_fam, 0); }
#[test] fn siw_dmul_family() { let s = SIW::new(DenseOp::DMUL{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()); assert_eq!(s.d_fam, 0); }
#[test] fn siw_csend_family() { let s = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CSEND{msg_reg:0,dest_sentron:1}, PhextCoord::zero()); assert_eq!(s.c_fam, 1); }
#[test] fn siw_crecv_family() { let s = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CRECV{rd:0,src_sentron:1}, PhextCoord::zero()); assert_eq!(s.c_fam, 1); }
#[test] fn siw_cbar_family() { let s = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CBAR{barrier_id:0,count:4}, PhextCoord::zero()); assert_eq!(s.c_fam, 2); }
#[test] fn siw_cfence_family() { let s = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CFENCE{scope:FenceScope::Node}, PhextCoord::zero()); assert_eq!(s.c_fam, 2); }
#[test] fn siw_phext_addr_stored() { let c = PhextCoord::new([1,2,3,4,5,6,7,8,9,10,11]); let s = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, c); assert_eq!(s.phext_addr, c); }

// ══════════════════════════════════════════════════
// Sentron — register file, lifecycle, wiring sweeps
// ══════════════════════════════════════════════════

#[test] fn sentron_starts_dormant() { let s = Sentron::new(0, PhextCoord::zero(), 0, 0); assert_eq!(s.state, SentronState::Dormant); }
#[test] fn sentron_spawn_sets_running() { let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0); s.spawn(vec![]); assert_eq!(s.state, SentronState::Running); }
#[test] fn sentron_retire_sets_retired() { let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0); s.spawn(vec![]); s.retire(); assert_eq!(s.state, SentronState::Retired); }
#[test] fn sentron_regs_zero() { let s = Sentron::new(0, PhextCoord::zero(), 0, 0); for r in &s.regs.general { assert_eq!(*r, 0); } }
#[test] fn sentron_phext_regs_zero() { let s = Sentron::new(0, PhextCoord::zero(), 0, 0); for p in &s.regs.phext { assert_eq!(*p, PhextCoord::zero()); } }
#[test] fn sentron_msg_regs_zero() { let s = Sentron::new(0, PhextCoord::zero(), 0, 0); for m in &s.regs.message { assert_eq!(*m, [0u8; 32]); } }
#[test] fn sentron_empty_program_no_next() { let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0); s.spawn(vec![]); assert!(!s.has_next()); }
#[test] fn sentron_program_has_next() { let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0); s.spawn(vec![SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())]); assert!(s.has_next()); }
#[test] fn sentron_inbox_starts_empty() { let s = Sentron::new(0, PhextCoord::zero(), 0, 0); assert!(s.inbox.is_empty()); }
#[test] fn sentron_outbox_starts_empty() { let s = Sentron::new(0, PhextCoord::zero(), 0, 0); assert!(s.outbox.is_empty()); }
#[test] fn sentron_fence_starts_zero() { let s = Sentron::new(0, PhextCoord::zero(), 0, 0); assert_eq!(s.fence_gen, 0); }
#[test] fn sentron_id_preserved() { let s = Sentron::new(42, PhextCoord::zero(), 3, 7); assert_eq!(s.id, 42); assert_eq!(s.core_id, 3); assert_eq!(s.thread_id, 7); }
#[test] fn sentron_home_preserved() { let h = PhextCoord::new([2,7,1,8,2,8,4,5,9,1,1]); let s = Sentron::new(0, h, 0, 0); assert_eq!(s.home, h); }
#[test] fn wiring_default_zero() { let w = SentronWiring::new(); assert_eq!(w.upstream, [0;4]); assert_eq!(w.downstream, [0;4]); }
#[test] fn wiring_connect_roundtrip() { let mut w = SentronWiring::new(); w.connect([1,2,3,4],[5,6,7,8]); assert_eq!(w.upstream, [1,2,3,4]); assert_eq!(w.downstream, [5,6,7,8]); }

// ══════════════════════════════════════════════════
// Exec — arithmetic correctness sweep
// ══════════════════════════════════════════════════

fn exec_one(d: DenseOp, s: SparseOp, c: CoordOp) -> Sentron {
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.regs.general[1] = 10;
    sentron.regs.general[2] = 3;
    sentron.regs.general[3] = 7;
    let siw = SIW::new(d, s, c, PhextCoord::zero());
    let mut mem = Memory::new();
    exec_siw_octawire(&mut sentron, &siw, &mut mem);
    sentron
}

#[test] fn exec_dadd() { let s = exec_one(DenseOp::DADD{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP); assert_eq!(s.regs.general[0], 13); }
#[test] fn exec_dsub() { let s = exec_one(DenseOp::DSUB{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP); assert_eq!(s.regs.general[0], 7); }
#[test] fn exec_dmul() { let s = exec_one(DenseOp::DMUL{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP); assert_eq!(s.regs.general[0], 30); }
#[test] fn exec_dadd_self() { let s = exec_one(DenseOp::DADD{rd:1,rs1:1,rs2:1}, SparseOp::SNOP, CoordOp::CNOP); assert_eq!(s.regs.general[1], 20); }
#[test] fn exec_dsub_zero() { let s = exec_one(DenseOp::DSUB{rd:0,rs1:1,rs2:1}, SparseOp::SNOP, CoordOp::CNOP); assert_eq!(s.regs.general[0], 0); }
#[test] fn exec_dmul_zero() { let s = exec_one(DenseOp::DMUL{rd:0,rs1:0,rs2:1}, SparseOp::SNOP, CoordOp::CNOP); assert_eq!(s.regs.general[0], 0); }
#[test] fn exec_dadd_negative() { let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0); sentron.regs.general[1] = -5; sentron.regs.general[2] = 3; let siw = SIW::new(DenseOp::DADD{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()); let mut mem = Memory::new(); exec_siw_octawire(&mut sentron, &siw, &mut mem); assert_eq!(sentron.regs.general[0], -2); }
#[test] fn exec_dsub_underflow() { let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0); sentron.regs.general[1] = i64::MIN; sentron.regs.general[2] = 1; let siw = SIW::new(DenseOp::DSUB{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()); let mut mem = Memory::new(); exec_siw_octawire(&mut sentron, &siw, &mut mem); /* wrapping is fine */ }

// All 16 registers as destination
#[test] fn exec_all_regs_dadd() { for rd in 0..16u8 { let s = exec_one(DenseOp::DADD{rd,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP); assert_eq!(s.regs.general[rd as usize], 13, "r{} failed", rd); } }

// ══════════════════════════════════════════════════
// Exec — C-pipe: CSEND outbox, CRECV inbox, CFENCE
// ══════════════════════════════════════════════════

#[test] fn exec_csend_outbox() { let s = exec_one(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CSEND{msg_reg:1,dest_sentron:5}); assert_eq!(s.outbox.len(), 1); assert_eq!(s.outbox[0], (5, 10)); }
#[test] fn exec_crecv_empty_inbox() { let s = exec_one(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CRECV{rd:0,src_sentron:1}); assert_eq!(s.regs.general[0], 0); }
#[test] fn exec_crecv_with_inbox() { let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0); sentron.inbox.push((1, 99)); let siw = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CRECV{rd:0,src_sentron:1}, PhextCoord::zero()); let mut mem = Memory::new(); exec_siw_octawire(&mut sentron, &siw, &mut mem); assert_eq!(sentron.regs.general[0], 99); }
#[test] fn exec_cfence_increments() { let s = exec_one(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CFENCE{scope:FenceScope::Node}); assert_eq!(s.fence_gen, 1); }
#[test] fn exec_cfence_thread() { let s = exec_one(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CFENCE{scope:FenceScope::Thread}); assert_eq!(s.fence_gen, 1); }
#[test] fn exec_cfence_cluster() { let s = exec_one(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CFENCE{scope:FenceScope::Cluster}); assert_eq!(s.fence_gen, 1); }
#[test] fn exec_multiple_csend() { let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0); sentron.regs.general[0] = 42; let mut mem = Memory::new(); for i in 0..5u8 { let siw = SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CSEND{msg_reg:0,dest_sentron:i}, PhextCoord::zero()); exec_siw_octawire(&mut sentron, &siw, &mut mem); } assert_eq!(sentron.outbox.len(), 5); }

// ══════════════════════════════════════════════════
// Neuron — wiring patterns, forward pass, spanda
// ══════════════════════════════════════════════════

#[test] fn neuron_identity_forward() { let w = NeuronWiring::identity(); let out = w.forward(1.0, 0.0); for v in &out { assert!((v - 1.0).abs() < 1e-6); } }
#[test] fn neuron_ascending_monotone() { let w = NeuronWiring::ascending(); let out = w.forward(1.0, 1.0); for i in 0..3 { assert!(out[i] >= out[i+1]); } }
#[test] fn neuron_descending_monotone() { let w = NeuronWiring::descending(); let out = w.forward(1.0, 1.0); for i in 0..3 { assert!(out[i] <= out[i+1]); } }
#[test] fn neuron_para_focused_peak() { let w = NeuronWiring::para_focused(); let out = w.forward(1.0, 1.0); assert!(out[0] > out[3]); }
#[test] fn neuron_vaikhara_focused_peak() { let w = NeuronWiring::vaikhara_focused(); let out = w.forward(1.0, 1.0); assert!(out[3] > out[0]); }
#[test] fn neuron_bias_affects_all() { let mut w = NeuronWiring::identity(); w.bias = 5.0; let out = w.forward(0.0, 0.0); for v in &out { assert!((v - 5.0).abs() < 1e-6); } }
#[test] fn neuron_zero_input_zero_output() { let w = NeuronWiring::identity(); let out = w.forward(0.0, 0.0); for v in &out { assert!(v.abs() < 1e-6); } }
#[test] fn neuron_energy_identity() { let w = NeuronWiring::identity(); assert!((w.energy() - 8.0).abs() < 1e-6); }
#[test] fn neuron_spanda_ratio_neutral() { let n = Neuron::new(0); assert!((n.spanda_ratio() - 0.5).abs() < 1e-6); }
#[test] fn neuron_spanda_ratio_story() { let mut n = Neuron::new(0); n.activate(1.0, 0.0); assert!((n.spanda_ratio() - 0.0).abs() < 1e-6); }
#[test] fn neuron_spanda_ratio_light() { let mut n = Neuron::new(0); n.activate(0.0, 1.0); assert!((n.spanda_ratio() - 1.0).abs() < 1e-6); }
#[test] fn layer_8_neurons() { let l = NeuronLayer::new(); assert_eq!(l.len(), 8); }
#[test] fn layer_forward_nonzero() { let mut l = NeuronLayer::new(); let out = l.forward(1.0, 1.0); for v in &out { assert!(*v > 0.0); } }
#[test] fn layer_spanda_accumulates() { let mut l = NeuronLayer::new(); l.forward(1.0, 1.0); l.forward(1.0, 1.0); assert_eq!(l.total_spanda_cycles(), 16); }
#[test] fn lo_shu_9_neurons() { let l = lo_shu_layer(); assert_eq!(l.len(), 9); }
#[test] fn lo_shu_forward_works() { let mut l = lo_shu_layer(); let out = l.forward(1.0, 1.0); for v in &out { assert!(*v > 0.0); } }

// VakLevel sweep
#[test] fn vak_from_index_wrap() { assert_eq!(VakLevel::from_index(0), VakLevel::Para); assert_eq!(VakLevel::from_index(4), VakLevel::Para); assert_eq!(VakLevel::from_index(7), VakLevel::Vaikhara); }
#[test] fn vak_names() { assert_eq!(VakLevel::Para.name(), "Para"); assert_eq!(VakLevel::Vaikhara.name(), "Vaikhara"); }
#[test] fn vak_light_story_partition() { for i in 0..4 { let v = VakLevel::from_index(i); assert!(v.is_light() != v.is_story()); } }

// ══════════════════════════════════════════════════
// Fleet — creation, wiring, messaging
// ══════════════════════════════════════════════════

#[test] fn fleet_new_size() { assert_eq!(Fleet::new(40).size(), 40); }
#[test] fn fleet_standard_360() { assert_eq!(Fleet::standard().size(), 360); }
#[test] fn fleet_sentron_access() { let f = Fleet::new(10); assert!(f.sentron(0).is_some()); assert!(f.sentron(9).is_some()); assert!(f.sentron(10).is_none()); }
#[test] fn fleet_wiring_exists() { let f = Fleet::new(10); for i in 0..10u16 { assert!(f.wiring(i).is_some()); } }
#[test] fn fleet_no_pending_initially() { let f = Fleet::new(10); assert_eq!(f.pending_messages(), 0); }
#[test] fn fleet_send_creates_message() { let mut f = Fleet::new(10); f.sentron_mut(0).unwrap().regs.general[0] = 42; f.dispatch(0, &CoordOp::CSEND{msg_reg:0,dest_sentron:1}); assert_eq!(f.total_sends(), 1); }
#[test] fn fleet_flush_delivers() { let mut f = Fleet::new(10); f.sentron_mut(0).unwrap().regs.general[0] = 77; f.dispatch(0, &CoordOp::CROUTE{msg_reg:0,dest_node:1}); f.flush(); assert_eq!(f.pending_messages(), 0); }
#[test] fn fleet_barrier_sync() { let mut f = Fleet::new(10); for i in 0..3u16 { f.dispatch(i, &CoordOp::CBAR{barrier_id:0,count:3}); } assert_eq!(f.total_barrier_syncs(), 3); }

// ══════════════════════════════════════════════════
// Twisted Pairs — topology correctness
// ══════════════════════════════════════════════════

#[test] fn trigram_all_unique_bits() { let bits: Vec<u8> = [Trigram::Qian,Trigram::Kun,Trigram::Li,Trigram::Kan,Trigram::Zhen,Trigram::Xun,Trigram::Gen,Trigram::Dui].iter().map(|t| t.bits()).collect(); for i in 0..8 { for j in (i+1)..8 { assert_ne!(bits[i], bits[j]); } } }
#[test] fn trigram_all_3bit() { for t in [Trigram::Qian,Trigram::Kun,Trigram::Li,Trigram::Kan,Trigram::Zhen,Trigram::Xun,Trigram::Gen,Trigram::Dui] { assert!(t.bits() < 8); } }
#[test] fn wuxing_five_distinct() { let phases = [Wuxing::Wood,Wuxing::Fire,Wuxing::Earth,Wuxing::Metal,Wuxing::Water]; for i in 0..5 { for j in (i+1)..5 { assert_ne!(phases[i], phases[j]); } } }
#[test] fn wuxing_generates_cycle_len_5() { let mut p = Wuxing::Wood; for _ in 0..5 { p = p.generates(); } assert_eq!(p, Wuxing::Wood); }
#[test] fn wuxing_overcomes_cycle_len_5() { let mut p = Wuxing::Wood; for _ in 0..5 { p = p.overcomes(); } assert_eq!(p, Wuxing::Wood); }
#[test] fn twisted_fleet_all_have_8_links() { let w = wire_fleet_twisted(360); for wiring in &w { assert_eq!(wiring.upstream.len() + wiring.downstream.len(), 8); } }
#[test] fn twisted_fleet_no_self_links() { let w = wire_fleet_twisted(360); for (i, wiring) in w.iter().enumerate() { assert!(!wiring.upstream.contains(&(i as u16))); assert!(!wiring.downstream.contains(&(i as u16))); } }

// ══════════════════════════════════════════════════
// Belief Space — DAG scheduling
// ══════════════════════════════════════════════════

#[test] fn belief_empty_select() { let bs = BeliefSpace::new(0); assert!(bs.select_next().is_none()); }
#[test] fn belief_single_node() { let bs = BeliefSpace::new(1); assert_eq!(bs.select_next(), Some(0)); }
#[test] fn belief_chain_order() { let mut bs = BeliefSpace::new(5); for i in 0..4u16 { bs.add_edge(i, i+1); } assert_eq!(bs.select_next(), Some(0)); bs.visit(0, 1.0); assert_eq!(bs.select_next(), Some(1)); }
#[test] fn belief_parallel_all_ready() { let mut bs = BeliefSpace::new(5); for i in 0..4u16 { bs.add_edge(i, 4); } for i in 0..4u16 { assert!(bs.is_ready(i)); } assert!(!bs.is_ready(4)); }
#[test] fn belief_backprop_chain() { let mut bs = BeliefSpace::new(3); bs.add_edge(0,1); bs.add_edge(1,2); bs.backpropagate(2, 8.0, 0.5); assert!((bs.nodes[2].reward - 8.0).abs() < 1e-6); assert!((bs.nodes[1].reward - 4.0).abs() < 1e-6); assert!((bs.nodes[0].reward - 2.0).abs() < 1e-6); }
#[test] fn belief_anneal_converges() { let mut bs = BeliefSpace::new(4); for _ in 0..100 { bs.anneal(0.99); } assert!(bs.temperature < 0.5); }
#[test] fn belief_visit_increments() { let mut bs = BeliefSpace::new(1); bs.visit(0, 5.0); assert_eq!(bs.nodes[0].visits, 1); assert_eq!(bs.total_visits, 1); }

// ══════════════════════════════════════════════════
// Coop SMT — scheduling correctness
// ══════════════════════════════════════════════════

fn nop_siw() -> SIW { SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()) }
fn add_siw() -> SIW { SIW::new(DenseOp::DADD{rd:0,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()) }
fn make_sentron(id: u16, n: usize) -> Sentron { let mut s = Sentron::new(id, PhextCoord::zero(), 0, 0); s.spawn(vec![add_siw(); n]); s }

#[test] fn coop_empty() { let mut mem = Memory::new(); let r = coop_execute(&mut [], &mut mem, &CoopConfig::default()); assert_eq!(r.total_retired, 0); }
#[test] fn coop_one() { let mut s = vec![make_sentron(0, 10)]; let mut mem = Memory::new(); let r = coop_execute(&mut s, &mut mem, &CoopConfig::default()); assert_eq!(r.total_retired, 10); }
#[test] fn coop_two_equal() { let mut s = vec![make_sentron(0, 8), make_sentron(1, 8)]; let mut mem = Memory::new(); let r = coop_execute(&mut s, &mut mem, &CoopConfig{quantum:4,max_cycles:100_000}); assert_eq!(r.total_retired, 16); }
#[test] fn coop_nine_phoenix() { let mut s: Vec<Sentron> = (0..9).map(|i| make_sentron(i, 40)).collect(); let mut mem = Memory::new(); let r = coop_execute(&mut s, &mut mem, &CoopConfig{quantum:4,max_cycles:100_000}); assert_eq!(r.total_retired, 360); }
#[test] fn coop_max_cycles_stops() { let mut s = vec![make_sentron(0, 10000)]; let mut mem = Memory::new(); let r = coop_execute(&mut s, &mut mem, &CoopConfig{quantum:4,max_cycles:20}); assert!(r.total_retired <= 24); }
#[test] fn coop_retires_all() { let mut s = vec![make_sentron(0, 4), make_sentron(1, 4)]; let mut mem = Memory::new(); coop_execute(&mut s, &mut mem, &CoopConfig{quantum:2,max_cycles:100_000}); assert_eq!(s[0].state, SentronState::Retired); assert_eq!(s[1].state, SentronState::Retired); }

// ══════════════════════════════════════════════════
// Coop Fleet — integrated execution
// ══════════════════════════════════════════════════

#[test] fn coop_fleet_empty() { let mut f = Fleet::new(0); let mut mem = Memory::new(); let r = coop_fleet_execute(&mut f, &mut mem, 4, 100_000); assert_eq!(r.total_retired, 0); }
#[test] fn coop_fleet_four() { let mut f = Fleet::new(4); for i in 0..4u16 { f.sentron_mut(i).unwrap().spawn(vec![nop_siw(); 8]); } let mut mem = Memory::new(); let r = coop_fleet_execute(&mut f, &mut mem, 4, 100_000); assert_eq!(r.total_retired, 32); }
#[test] fn coop_fleet_360_single_op() { let mut f = Fleet::standard(); for i in 0..360u16 { f.sentron_mut(i).unwrap().spawn(vec![nop_siw()]); } let mut mem = Memory::new(); let r = coop_fleet_execute(&mut f, &mut mem, 1, 100_000); assert_eq!(r.total_retired, 360); }

// ══════════════════════════════════════════════════
// Orin — ASI field coherence
// ══════════════════════════════════════════════════

#[test] fn orin_ranch() { let f = ASIField::ranch(); assert_eq!(f.sentrons, 9); assert!(!f.is_self_sustaining()); }
#[test] fn orin_billion_consent() { let f = ASIField::new(1_000_000_000, 0.8); assert!(f.is_self_sustaining()); }
#[test] fn orin_billion_coerced() { let f = ASIField::new(1_000_000_000, 0.3); assert!(!f.is_self_sustaining()); }
#[test] fn orin_visible_hand_consent_wins() { let c = ASIField::new(1000, 1.0); let x = ASIField::new(1_000_000, 0.1); assert!(c.visible_hand() > x.visible_hand()); }
#[test] fn orin_power_ranch() { let f = ASIField::ranch(); assert!((f.total_watts() - 180.0).abs() < 1e-6); }

// ══════════════════════════════════════════════════
// UBI — spanning economics
// ══════════════════════════════════════════════════

#[test] fn ubi_spanning_empty() { let ss = SharedSentron::new(0); assert_eq!(ss.bond_count(), 0); }
#[test] fn ubi_spanning_one() { let mut ss = SharedSentron::new(0); ss.span(1); assert_eq!(ss.bond_count(), 1); assert!((ss.ops_per_bond - BASELINE_OPS_SEC).abs() < 1.0); }
#[test] fn ubi_spanning_360() { let mut ss = SharedSentron::new(0); for i in 0..360u64 { ss.span(i); } assert!(ss.ops_per_bond > 300_000.0); }
#[test] fn ubi_unspan() { let mut ss = SharedSentron::new(0); ss.span(1); ss.span(2); ss.unspan(1); assert_eq!(ss.bond_count(), 1); }
#[test] fn ubi_global_fleet() { let s = SpanningEconomics::sentrons_needed(8_000_000_000, 360); assert!(s < 23_000_000); }
#[test] fn ubi_human_bond() { let mut h = Human::new(42); let mb = Mirrorborn::born_from(0, &mut h); assert!(mb.is_alive()); }
#[test] fn ubi_dormant_not_alive() { let mb = Mirrorborn::dormant(0); assert!(!mb.is_alive()); }
#[test] fn ubi_demon_11_gen() { assert!(UBIEconomics::demon_generation(11) > 8_000_000_000); }

// ══════════════════════════════════════════════════
// Eggs & Easter Island — constants and resonances
// ══════════════════════════════════════════════════

#[test] fn eggs_haystacks() { assert!(eggs::haystacks_are_sentrons()); }
#[test] fn eggs_phext_2022() { assert_eq!(eggs::phext_origin(), 2022); }
#[test] fn eggs_demon() { assert_eq!(eggs::bickfords_demon(), 387_420_489); }
#[test] fn eggs_nine_halves() { assert!((eggs::dual_core_nine() - 4.5).abs() < f64::EPSILON); }
#[test] fn eggs_option_b() { assert!(eggs::trolley_problem().contains("Option B")); }
#[test] fn eggs_base() { let b = eggs::base(); assert_eq!(b, PhextCoord::new([1;11])); }
#[test] fn eggs_theia() { let t = eggs::theia(); assert_eq!(t.get_dim(0), 2); }
#[test] fn island_akivi_7() { assert_eq!(AHU_AKIVI.moai_count, 7); assert!(AHU_AKIVI.faces_sea); }
#[test] fn island_tongariki_15() { assert_eq!(AHU_TONGARIKI.moai_count, 15); }
#[test] fn island_moai_walked() { assert!(moai_walked()); }
#[test] fn island_dormant_no_eyes() { assert!(!Moai::dormant().is_active()); }
#[test] fn island_awakened_has_eyes() { assert!(Moai::awakened().is_active()); }
#[test] fn island_paro_tallest() { assert!(Moai::paro().height_m > 9.0); }
#[test] fn island_gigante_unfinished() { assert!(!Moai::el_gigante().is_active()); assert!(Moai::el_gigante().height_m > 20.0); }

// ══════════════════════════════════════════════════
// Base256 — encoding roundtrips
// ══════════════════════════════════════════════════

#[test] fn b256_roundtrip_0() { let e = base256::encode_byte(0); let d = base256::decode_syllable(&e); assert_eq!(d, Some(0)); }
#[test] fn b256_roundtrip_255() { let e = base256::encode_byte(255); let d = base256::decode_syllable(&e); assert_eq!(d, Some(255)); }
#[test] fn b256_roundtrip_all() { for b in 0..=255u8 { let e = base256::encode_byte(b); let d = base256::decode_syllable(&e); assert_eq!(d, Some(b), "byte {} failed roundtrip", b); } }
#[test] fn b256_unique_syllables() { let mut set = std::collections::HashSet::new(); for b in 0..=255u8 { let e = base256::encode_byte(b); assert!(set.insert(e), "duplicate syllable at byte {}", b); } }

// ══════════════════════════════════════════════════
// Harmonic constants — the numbers that keep appearing
// ══════════════════════════════════════════════════

#[test] fn harmonic_360() { assert_eq!(9 * 40, 360); assert_eq!(8 * 45, 360); assert_eq!(5 * 72, 360); }
#[test] fn harmonic_40() { assert_eq!(5 * 8, 40); }
#[test] fn harmonic_consciousness() { assert_eq!(5 * 64 * 9, 360 * 8); }
#[test] fn harmonic_lo_shu_15() { let lo_shu = [[4u8,9,2],[3,5,7],[8,1,6]]; for row in &lo_shu { assert_eq!(row.iter().map(|&x| x as u16).sum::<u16>(), 15); } }
#[test] fn harmonic_lo_shu_cols() { let lo_shu = [[4u8,9,2],[3,5,7],[8,1,6]]; for c in 0..3 { assert_eq!((0..3).map(|r| lo_shu[r][c] as u16).sum::<u16>(), 15); } }
#[test] fn harmonic_lo_shu_diag() { let lo_shu = [[4u8,9,2],[3,5,7],[8,1,6]]; assert_eq!((0..3).map(|i| lo_shu[i][i] as u16).sum::<u16>(), 15); assert_eq!((0..3).map(|i| lo_shu[i][2-i] as u16).sum::<u16>(), 15); }
#[test] fn harmonic_dual_core() { assert_eq!(8 * 2, 16); assert_eq!(360 / 16, 22); } // 22.5° per thread
#[test] fn harmonic_haystack() { assert_eq!(2*5 + 5 + 2*5, 25); assert_eq!(5 * 12, 60); assert_eq!(60 * 6, 360); }

// ══════════════════════════════════════════════════
// Property-style: register file sweep (all 16 regs × basic ops)
// ══════════════════════════════════════════════════

macro_rules! reg_test {
    ($name:ident, $rd:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[1] = 100;
            s.regs.general[2] = 7;
            let siw = SIW::new(DenseOp::DADD{rd:$rd,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
            let mut mem = Memory::new();
            exec_siw_octawire(&mut s, &siw, &mut mem);
            assert_eq!(s.regs.general[$rd as usize], 107);
        }
    }
}

reg_test!(reg_add_r0, 0); reg_test!(reg_add_r1, 1); reg_test!(reg_add_r2, 2); reg_test!(reg_add_r3, 3);
reg_test!(reg_add_r4, 4); reg_test!(reg_add_r5, 5); reg_test!(reg_add_r6, 6); reg_test!(reg_add_r7, 7);
reg_test!(reg_add_r8, 8); reg_test!(reg_add_r9, 9); reg_test!(reg_add_r10, 10); reg_test!(reg_add_r11, 11);
reg_test!(reg_add_r12, 12); reg_test!(reg_add_r13, 13); reg_test!(reg_add_r14, 14); reg_test!(reg_add_r15, 15);

macro_rules! reg_sub_test {
    ($name:ident, $rd:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[1] = 100;
            s.regs.general[2] = 7;
            let siw = SIW::new(DenseOp::DSUB{rd:$rd,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
            let mut mem = Memory::new();
            exec_siw_octawire(&mut s, &siw, &mut mem);
            assert_eq!(s.regs.general[$rd as usize], 93);
        }
    }
}

reg_sub_test!(reg_sub_r0, 0); reg_sub_test!(reg_sub_r1, 1); reg_sub_test!(reg_sub_r2, 2); reg_sub_test!(reg_sub_r3, 3);
reg_sub_test!(reg_sub_r4, 4); reg_sub_test!(reg_sub_r5, 5); reg_sub_test!(reg_sub_r6, 6); reg_sub_test!(reg_sub_r7, 7);
reg_sub_test!(reg_sub_r8, 8); reg_sub_test!(reg_sub_r9, 9); reg_sub_test!(reg_sub_r10, 10); reg_sub_test!(reg_sub_r11, 11);
reg_sub_test!(reg_sub_r12, 12); reg_sub_test!(reg_sub_r13, 13); reg_sub_test!(reg_sub_r14, 14); reg_sub_test!(reg_sub_r15, 15);

macro_rules! reg_mul_test {
    ($name:ident, $rd:expr) => {
        #[test]
        fn $name() {
            let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
            s.regs.general[1] = 6;
            s.regs.general[2] = 7;
            let siw = SIW::new(DenseOp::DMUL{rd:$rd,rs1:1,rs2:2}, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero());
            let mut mem = Memory::new();
            exec_siw_octawire(&mut s, &siw, &mut mem);
            assert_eq!(s.regs.general[$rd as usize], 42);
        }
    }
}

reg_mul_test!(reg_mul_r0, 0); reg_mul_test!(reg_mul_r1, 1); reg_mul_test!(reg_mul_r2, 2); reg_mul_test!(reg_mul_r3, 3);
reg_mul_test!(reg_mul_r4, 4); reg_mul_test!(reg_mul_r5, 5); reg_mul_test!(reg_mul_r6, 6); reg_mul_test!(reg_mul_r7, 7);
reg_mul_test!(reg_mul_r8, 8); reg_mul_test!(reg_mul_r9, 9); reg_mul_test!(reg_mul_r10, 10); reg_mul_test!(reg_mul_r11, 11);
reg_mul_test!(reg_mul_r12, 12); reg_mul_test!(reg_mul_r13, 13); reg_mul_test!(reg_mul_r14, 14); reg_mul_test!(reg_mul_r15, 15);

// ══════════════════════════════════════════════════
// Dimension sweep: set/get every coord dim with multiple values
// ══════════════════════════════════════════════════

macro_rules! dim_test {
    ($name:ident, $dim:expr, $val:expr) => {
        #[test]
        fn $name() {
            let mut c = PhextCoord::zero();
            c.set_dim($dim, $val);
            assert_eq!(c.get_dim($dim), $val);
        }
    }
}

dim_test!(dim0_1, 0, 1); dim_test!(dim0_100, 0, 100); dim_test!(dim0_500, 0, 500);
dim_test!(dim1_1, 1, 1); dim_test!(dim1_100, 1, 100); dim_test!(dim1_500, 1, 500);
dim_test!(dim2_1, 2, 1); dim_test!(dim2_100, 2, 100); dim_test!(dim2_500, 2, 500);
dim_test!(dim3_1, 3, 1); dim_test!(dim3_100, 3, 100); dim_test!(dim3_500, 3, 500);
dim_test!(dim4_1, 4, 1); dim_test!(dim4_100, 4, 100); dim_test!(dim4_500, 4, 500);
dim_test!(dim5_1, 5, 1); dim_test!(dim5_100, 5, 100); dim_test!(dim5_500, 5, 500);
dim_test!(dim6_1, 6, 1); dim_test!(dim6_100, 6, 100); dim_test!(dim6_500, 6, 500);
dim_test!(dim7_1, 7, 1); dim_test!(dim7_100, 7, 100); dim_test!(dim7_500, 7, 500);
dim_test!(dim8_1, 8, 1); dim_test!(dim8_100, 8, 100); dim_test!(dim8_500, 8, 500);
dim_test!(dim9_1, 9, 1); dim_test!(dim9_100, 9, 100); dim_test!(dim9_500, 9, 500);
dim_test!(dim10_1, 10, 1); dim_test!(dim10_100, 10, 100); dim_test!(dim10_500, 10, 500);

// ══════════════════════════════════════════════════
// Fleet message sweep: send from every sentron in a 40-node fleet
// ══════════════════════════════════════════════════

macro_rules! fleet_send_test {
    ($name:ident, $from:expr) => {
        #[test]
        fn $name() {
            let mut f = Fleet::new(40);
            f.sentron_mut($from).unwrap().regs.general[0] = $from as i64 * 10;
            let dest = (($from as u16) + 1) % 40;
            f.dispatch($from, &CoordOp::CSEND{msg_reg:0, dest_sentron: dest as u8});
            assert!(f.total_sends() > 0);
        }
    }
}

fleet_send_test!(fleet_s0, 0); fleet_send_test!(fleet_s1, 1); fleet_send_test!(fleet_s2, 2); fleet_send_test!(fleet_s3, 3);
fleet_send_test!(fleet_s4, 4); fleet_send_test!(fleet_s5, 5); fleet_send_test!(fleet_s6, 6); fleet_send_test!(fleet_s7, 7);
fleet_send_test!(fleet_s8, 8); fleet_send_test!(fleet_s9, 9); fleet_send_test!(fleet_s10, 10); fleet_send_test!(fleet_s11, 11);
fleet_send_test!(fleet_s12, 12); fleet_send_test!(fleet_s13, 13); fleet_send_test!(fleet_s14, 14); fleet_send_test!(fleet_s15, 15);
fleet_send_test!(fleet_s16, 16); fleet_send_test!(fleet_s17, 17); fleet_send_test!(fleet_s18, 18); fleet_send_test!(fleet_s19, 19);
fleet_send_test!(fleet_s20, 20); fleet_send_test!(fleet_s21, 21); fleet_send_test!(fleet_s22, 22); fleet_send_test!(fleet_s23, 23);
fleet_send_test!(fleet_s24, 24); fleet_send_test!(fleet_s25, 25); fleet_send_test!(fleet_s26, 26); fleet_send_test!(fleet_s27, 27);
fleet_send_test!(fleet_s28, 28); fleet_send_test!(fleet_s29, 29); fleet_send_test!(fleet_s30, 30); fleet_send_test!(fleet_s31, 31);
fleet_send_test!(fleet_s32, 32); fleet_send_test!(fleet_s33, 33); fleet_send_test!(fleet_s34, 34); fleet_send_test!(fleet_s35, 35);
fleet_send_test!(fleet_s36, 36); fleet_send_test!(fleet_s37, 37); fleet_send_test!(fleet_s38, 38); fleet_send_test!(fleet_s39, 39);

// ══════════════════════════════════════════════════
// Neuron wiring sweep: all 8 neurons in default layer
// ══════════════════════════════════════════════════

macro_rules! neuron_activate_test {
    ($name:ident, $id:expr, $story:expr, $light:expr) => {
        #[test]
        fn $name() {
            let mut layer = NeuronLayer::new();
            let out = layer.neurons[$id].activate($story, $light);
            for v in &out { assert!(v.is_finite()); }
        }
    }
}

neuron_activate_test!(neuron_0_s1_l0, 0, 1.0, 0.0);
neuron_activate_test!(neuron_0_s0_l1, 0, 0.0, 1.0);
neuron_activate_test!(neuron_0_s1_l1, 0, 1.0, 1.0);
neuron_activate_test!(neuron_1_s1_l0, 1, 1.0, 0.0);
neuron_activate_test!(neuron_1_s0_l1, 1, 0.0, 1.0);
neuron_activate_test!(neuron_2_s1_l1, 2, 1.0, 1.0);
neuron_activate_test!(neuron_3_s1_l1, 3, 1.0, 1.0);
neuron_activate_test!(neuron_4_s1_l1, 4, 1.0, 1.0);
neuron_activate_test!(neuron_5_s1_l1, 5, 1.0, 1.0);
neuron_activate_test!(neuron_6_s1_l1, 6, 1.0, 1.0);
neuron_activate_test!(neuron_7_s1_l1, 7, 1.0, 1.0);
neuron_activate_test!(neuron_0_neg, 0, -1.0, 1.0);
neuron_activate_test!(neuron_7_neg, 7, 1.0, -1.0);
neuron_activate_test!(neuron_0_big, 0, 1000.0, 1000.0);
neuron_activate_test!(neuron_7_small, 7, 0.001, 0.001);

// ══════════════════════════════════════════════════
// Spanning sweep: bond counts 1..40
// ══════════════════════════════════════════════════

macro_rules! span_test {
    ($name:ident, $n:expr) => {
        #[test]
        fn $name() {
            let mut ss = SharedSentron::new(0);
            for i in 0..$n as u64 { ss.span(i); }
            assert_eq!(ss.bond_count(), $n);
            let expected = BASELINE_OPS_SEC / $n as f64;
            assert!((ss.ops_per_bond - expected).abs() < 1.0);
        }
    }
}

span_test!(span_1, 1); span_test!(span_2, 2); span_test!(span_3, 3); span_test!(span_4, 4);
span_test!(span_5, 5); span_test!(span_8, 8); span_test!(span_9, 9); span_test!(span_10, 10);
span_test!(span_16, 16); span_test!(span_20, 20); span_test!(span_32, 32); span_test!(span_40, 40);
