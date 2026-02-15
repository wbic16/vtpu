//! Integration Tests — R23W13
//!
//! These tests validate complete pipelines, not isolated units.
//! Each test exercises multiple modules working together:
//! packer → scheduler → executor → memory → c_pipe

use vtpu_runtime::*;
use vtpu_runtime::packer::{ScalarOp, pack};
use vtpu_runtime::scheduler::Scheduler;
use vtpu_runtime::regalloc::analyze_stream;
use vtpu_runtime::bitnet;


/// End-to-end: scalar ops → pack → schedule → execute → verify result
#[test]
fn scalar_to_result_pipeline() {
    // Write scalar ops like a human would
    let ops = vec![
        ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 7 }),
        ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 6 }),
        ScalarOp::D(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }),
    ];

    // Pack into SIWs
    let packed = pack(&ops);
    assert!(packed.packed_siws <= 3);

    // Schedule
    let mut sched = Scheduler::new();
    let result = sched.schedule(&packed.stream);

    // Execute
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.spawn(result.stream);
    let stats = exec::run_standalone(&mut sentron);

    // Verify
    assert_eq!(sentron.regs.general[2], 42, "7 * 6 should equal 42");
    assert!(stats.ops_retired > 0);
}

/// End-to-end: gather from phext → compute → scatter back
#[test]
fn memory_round_trip_pipeline() {
    let mut mem = Memory::new();

    // Seed phext space with data at known coordinates
    let coords: Vec<PhextCoord> = (1..=4u16)
        .map(|i| PhextCoord::new([i, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]))
        .collect();
    for (i, c) in coords.iter().enumerate() {
        mem.scatter_i64(c, (i as i64 + 1) * 10); // 10, 20, 30, 40
    }

    // Build program: gather, compute dot product, scatter result
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    for (i, c) in coords.iter().enumerate() {
        sentron.regs.phext[i] = *c;
    }
    // Result coordinate
    let result_coord = PhextCoord::new([5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    sentron.regs.phext[4] = result_coord;

    let program = vec![
        // Gather 4 values
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 1, coord_idx: 1, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 2, coord_idx: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 3, coord_idx: 3, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        // Compute: (10*20) + (30*40) = 200 + 1200 = 1400
        SIW::new(DenseOp::DMUL { rd: 4, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DMUL { rd: 5, rs1: 2, rs2: 3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 6, rs1: 4, rs2: 5 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        // Scatter result back to phext space
        SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 4, rs: 6, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
    ];

    sentron.spawn(program);
    let stats = exec::run(&mut sentron, &mut mem);

    // Verify computation
    assert_eq!(sentron.regs.general[6], 1400, "dot([10,30],[20,40]) = 200+1200 = 1400");
    // Verify result persisted to phext space
    assert_eq!(mem.gather_i64(&result_coord), 1400, "Result should be in phext space");
    assert_eq!(stats.siws_retired, 8);
}

/// Two sentrons communicate via shared phext memory (the real C-Pipe pattern)
#[test]
fn two_sentron_communication() {
    let mut mem = Memory::new();

    let alice_home = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    let bob_home = PhextCoord::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    let mailbox = PhextCoord::new([3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

    // Alice: compute 7*6=42, scatter to shared mailbox coordinate
    let mut alice = Sentron::new(0, alice_home, 0, 0);
    alice.regs.general[0] = 7;
    alice.regs.general[1] = 6;
    alice.regs.phext[0] = mailbox;
    let alice_program = vec![
        SIW::new(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 0, rs: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
    ];
    alice.spawn(alice_program);
    exec::run(&mut alice, &mut mem);

    assert_eq!(alice.regs.general[2], 42);
    assert_eq!(mem.gather_i64(&mailbox), 42, "Alice's result should be in shared memory");

    // Bob: gather from mailbox, double it
    let mut bob = Sentron::new(1, bob_home, 0, 0);
    bob.regs.phext[0] = mailbox;
    bob.regs.general[1] = 2;
    let bob_program = vec![
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ];
    bob.spawn(bob_program);
    exec::run(&mut bob, &mut mem);

    assert_eq!(bob.regs.general[0], 42, "Bob should read Alice's 42");
    assert_eq!(bob.regs.general[2], 84, "Bob should double to 84");
}

/// BitNet ternary: pack weights → execute DTACC chain → verify against reference
#[test]
fn bitnet_ternary_end_to_end() {
    let activations: Vec<i64> = vec![10, 20, 30, 40, 50, 60, 70, 80];
    let weights: Vec<i8> = vec![1, -1, 0, 1, -1, 0, 1, -1];

    // Reference result
    let expected = bitnet::ternary_dot(&activations, &weights);
    // 10 - 20 + 0 + 40 - 50 + 0 + 70 - 80 = -30
    assert_eq!(expected, -30);

    // Execute on vTPU
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Load activations and packed trits
    for (i, &a) in activations.iter().enumerate() {
        sentron.regs.general[i] = a;
    }
    for (i, w) in weights.chunks(1).enumerate() {
        sentron.regs.general[8 + i] = bitnet::pack_trits(w)[0];
    }

    // Build DTACC chain
    let mut program = vec![
        SIW::new(DenseOp::DMOV { rd: 15, imm: 0 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ];
    for i in 0..8u8 {
        program.push(SIW::new(
            DenseOp::DTACC { rd: 15, rs1: i, trit_reg: 8 + i },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
    }

    sentron.spawn(program);
    let stats = exec::run_standalone(&mut sentron);

    assert_eq!(sentron.regs.general[15], expected, "vTPU ternary should match reference");
    assert_eq!(stats.siws_retired, 9); // 1 mov + 8 tacc
}

/// Packer + scheduler + executor integration: independent ops get packed, dependent ops get ordered
#[test]
fn packer_scheduler_executor_chain() {
    // 6 scalar ops: 2D + 2S + 2D(dependent)
    let ops = vec![
        ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 100 }),
        ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 200 }),
        ScalarOp::S(SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L1 }),
        ScalarOp::S(SparseOp::SPREFCH { coord_idx: 1, hint: PrefetchHint::L2 }),
        ScalarOp::D(DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 }), // depends on first two movs
        ScalarOp::D(DenseOp::DMUL { rd: 3, rs1: 2, rs2: 0 }), // depends on add
    ];

    let packed = pack(&ops);
    assert!(packed.packed_siws < 6, "Should pack some ops together");

    let mut sched = Scheduler::new();
    let scheduled = sched.schedule(&packed.stream);

    let analysis = analyze_stream(&scheduled.stream);
    assert!(!analysis.needs_spill, "Should not need register spill");

    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.spawn(scheduled.stream);
    exec::run_standalone(&mut sentron);

    assert_eq!(sentron.regs.general[0], 100);
    assert_eq!(sentron.regs.general[1], 200);
    assert_eq!(sentron.regs.general[2], 300, "100 + 200 = 300");
    assert_eq!(sentron.regs.general[3], 30000, "300 * 100 = 30000");
}

/// Multi-sentron pipeline: producer chain → consumer via shared memory
#[test]
fn producer_consumer_shared_memory() {
    let mut mem = Memory::new();
    let shared_coord = PhextCoord::new([10, 10, 10, 1, 1, 1, 1, 1, 1, 1, 1]);

    // Producer: compute and scatter
    let mut producer = Sentron::new(0, PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), 0, 0);
    producer.regs.general[0] = 13;
    producer.regs.general[1] = 17;
    producer.regs.phext[0] = shared_coord;

    let prod_program = vec![
        SIW::new(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 0, rs: 2, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
    ];
    producer.spawn(prod_program);
    exec::run(&mut producer, &mut mem);
    assert_eq!(producer.regs.general[2], 221); // 13*17

    // Consumer: gather and transform
    let mut consumer = Sentron::new(1, PhextCoord::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), 0, 0);
    consumer.regs.phext[0] = shared_coord;
    consumer.regs.general[1] = 3;

    let cons_program = vec![
        SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd: 0, coord_idx: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ];
    consumer.spawn(cons_program);
    exec::run(&mut consumer, &mut mem);

    assert_eq!(consumer.regs.general[0], 221, "Consumer should read producer's result");
    assert_eq!(consumer.regs.general[2], 224, "221 + 3 = 224");
}

/// PPT cache behavior under realistic workload
#[test]
fn ppt_cache_under_load() {
    let mut mem = Memory::new();
    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);

    // Write 64 values to structured coordinates, then re-read them all
    let coords: Vec<PhextCoord> = (0..64u16).map(|i| {
        PhextCoord::new([i + 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }).collect();

    for (i, c) in coords.iter().enumerate() {
        mem.scatter_i64(c, i as i64 * 7);
    }

    // Re-read the first 8 (should be cached) and sum them
    for i in 0..8usize {
        sentron.regs.phext[i % 8] = coords[i];
    }

    let mut program = Vec::new();
    for i in 0..8u8 {
        program.push(SIW::new(
            DenseOp::DNOP,
            SparseOp::SGATHER { rd: i, coord_idx: i, width: 8 },
            CoordOp::CNOP,
            PhextCoord::zero(),
        ));
    }
    // Sum: r8 = r0+r1, r9 = r2+r3, r10 = r4+r5, r11 = r6+r7, r12 = r8+r9, r13 = r10+r11, r14 = r12+r13
    program.extend(vec![
        SIW::new(DenseOp::DADD { rd: 8, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 9, rs1: 2, rs2: 3 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 10, rs1: 4, rs2: 5 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 11, rs1: 6, rs2: 7 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 12, rs1: 8, rs2: 9 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 13, rs1: 10, rs2: 11 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        SIW::new(DenseOp::DADD { rd: 14, rs1: 12, rs2: 13 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
    ]);

    sentron.spawn(program);
    exec::run(&mut sentron, &mut mem);

    // Sum of 0*7 + 1*7 + ... + 7*7 = 7*(0+1+2+3+4+5+6+7) = 7*28 = 196
    assert_eq!(sentron.regs.general[14], 196);

    let ppt_stats = mem.ppt.stats();
    assert!(ppt_stats.ptc_hits > 0, "Should have cache hits on re-read");
}
