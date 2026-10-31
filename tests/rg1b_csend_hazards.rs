//! Regression tests for R23 CSEND's General-register dependencies.
//!
//! The executor uses r[msg_reg % 16], so both the register allocator and
//! scalar packer must preserve RAW/WAR ordering for that exact register.

use vtpu_runtime::packer::{pack, ScalarOp};
use vtpu_runtime::regalloc::{detect_hazards, extract_accesses, Hazard, RegClass};
use vtpu_runtime::Scheduler;
use vtpu_runtime::{exec_run, CoordOp, DenseOp, Memory, PhextCoord, Sentron, SparseOp, SIW};

fn mov(rd: u8, imm: i64) -> SIW {
    SIW::new(
        DenseOp::DMOV { rd, imm },
        SparseOp::SNOP,
        CoordOp::CNOP,
        PhextCoord::zero(),
    )
}

fn send(msg_reg: u8) -> SIW {
    SIW::new(
        DenseOp::DNOP,
        SparseOp::SNOP,
        CoordOp::CSEND {
            msg_reg,
            dest_sentron: 7,
        },
        PhextCoord::zero(),
    )
}

#[test]
fn csend_reads_general_register_even_when_encoded_index_wraps() {
    let accesses = extract_accesses(&send(18));
    assert!(
        accesses.reads.contains(&(RegClass::General, 2)),
        "CSEND(18) reads General r2 because the executor uses msg_reg % 16"
    );
    assert!(
        !accesses
            .reads
            .iter()
            .any(|&(class, _)| class == RegClass::Message),
        "CSEND does not read a Message register in either executor"
    );
}

#[test]
fn csend_raw_and_war_edges_follow_general_registers() {
    let producer_first = detect_hazards(&[mov(2, 99), send(18)]);
    assert!(producer_first.contains(&Hazard::RAW {
        reg: 2,
        producer: 0,
        consumer: 1
    }));

    let consumer_first = detect_hazards(&[send(18), mov(2, 99)]);
    assert!(consumer_first.contains(&Hazard::WAR {
        reg: 2,
        reader: 0,
        writer: 1
    }));

    let independent = detect_hazards(&[mov(3, 99), send(18)]);
    assert!(
        !independent
            .iter()
            .any(|h| matches!(h, Hazard::RAW { .. } | Hazard::WAR { .. })),
        "Unrelated General register r3 must remain independent of CSEND(r2)"
    );
}

#[test]
fn packing_csend_before_write_preserves_old_outbox_value() {
    // Serial meaning: send old r2=41, then overwrite r2 with 99.
    // If they are packed into one SIW, D executes before C and the outbox
    // incorrectly contains 99. WAR must force separate SIWs.
    let packed = pack(&[
        ScalarOp::C(CoordOp::CSEND {
            msg_reg: 2,
            dest_sentron: 7,
        }),
        ScalarOp::D(DenseOp::DMOV { rd: 2, imm: 99 }),
    ]);
    assert_eq!(packed.packed_siws, 2, "WAR must prevent same-SIW packing");

    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    sentron.regs.general[2] = 41;
    sentron.spawn(packed.stream);
    let stats = exec_run(&mut sentron, &mut Memory::new());
    assert_eq!(stats.siws_retired, 2);
    assert_eq!(sentron.outbox, vec![(7, 41)], "CSEND must use pre-write r2");
    assert_eq!(sentron.regs.general[2], 99);
}

#[test]
fn scheduler_exposes_csend_raw_and_inserts_pipeline_gap() {
    let mut scheduler = Scheduler::new();
    let result = scheduler.schedule(&[mov(2, 99), send(2)]);
    assert_eq!(
        result.before.raw_hazards, 1,
        "D-Pipe producer and C-Pipe CSEND must have a RAW edge"
    );
    assert!(
        result.nops_inserted >= 1,
        "adjacent producer/CSEND requires the scheduler's current RAW gap"
    );
}
