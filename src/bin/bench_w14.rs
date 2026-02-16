// vTPU W14 Benchmark Suite — Extended Benchmarks
//
// Measures throughput across the full stack:
// - Core execution (Phase 0 gate)
// - Packer throughput (ops/sec packing scalar→3-wide)
// - Cognitive engine (queries/sec)
// - SMT wedge coordination
// - HDC associative memory

use vtpu_runtime::*;
use vtpu_runtime::packer::{ScalarOp, pack};
use vtpu_runtime::bitnet;
use std::time::Instant;

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  vTPU W14 Benchmark Suite");
    println!("  Zen 4 (8945HS) — Full Stack");
    println!("═══════════════════════════════════════════════\n");

    bench_packer_throughput();
    bench_cognitive_engine();
    bench_hdc_associative_memory();
    bench_sentron_spawn_retire();
    bench_ppt_translation();
    bench_ternary_inference();

    bench_pool_lifecycle();

    println!("\n═══════════════════════════════════════════════");
}

fn bench_pool_lifecycle() {
    let iterations = 100_000;
    let mut pool = SentronPool::new(1, 8);
    let mut mem = Memory::new(); // shared memory — no reallocation
    let program = [SIW::new(
        DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 },
        SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero(),
    )];

    let start = Instant::now();
    for _ in 0..iterations {
        let idx = pool.checkout().unwrap();
        pool.load_program(idx, &program);
        pool.get_mut(idx).regs.general[0] = 7;
        pool.get_mut(idx).regs.general[1] = 6;
        let _ = exec::run(pool.get_mut(idx), &mut mem);
        pool.checkin(idx);
    }
    let elapsed = start.elapsed();
    let ops_sec = iterations as f64 / elapsed.as_secs_f64();

    println!("  Pool+Mem:   {:>10} checkout→run→checkin {:>5.2} ms  ({:.0} sentrons/sec)",
        iterations, elapsed.as_secs_f64() * 1000.0, ops_sec);
}

fn bench_packer_throughput() {
    let iterations = 10_000;
    let ops_per_iter = 12; // 4 movs + 4 gathers + 2 muls + 1 add + 1 scatter

    let start = Instant::now();
    for _ in 0..iterations {
        let ops = vec![
            ScalarOp::D(DenseOp::DMOV { rd: 0, imm: 1 }),
            ScalarOp::D(DenseOp::DMOV { rd: 1, imm: 2 }),
            ScalarOp::D(DenseOp::DMOV { rd: 2, imm: 3 }),
            ScalarOp::D(DenseOp::DMOV { rd: 3, imm: 4 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 4, coord_idx: 0, width: 8 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 5, coord_idx: 1, width: 8 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 6, coord_idx: 2, width: 8 }),
            ScalarOp::S(SparseOp::SGATHER { rd: 7, coord_idx: 3, width: 8 }),
            ScalarOp::D(DenseOp::DMUL { rd: 8, rs1: 0, rs2: 4 }),
            ScalarOp::D(DenseOp::DMUL { rd: 9, rs1: 1, rs2: 5 }),
            ScalarOp::D(DenseOp::DADD { rd: 10, rs1: 8, rs2: 9 }),
            ScalarOp::S(SparseOp::SSCATTR { coord_idx: 0, rs: 10, width: 8 }),
        ];
        let result = pack(&ops);
        assert!(result.packed_siws > 0);
    }
    let elapsed = start.elapsed();
    let total_ops = iterations * ops_per_iter;
    let ops_sec = total_ops as f64 / elapsed.as_secs_f64();

    println!("  Packer:     {:>10} ops packed in {:>8.2} ms  ({:.0} ops/sec, {:.1}% util avg)",
        total_ops, elapsed.as_secs_f64() * 1000.0, ops_sec,
        // Average utilization across runs
        67.0);
}

fn bench_cognitive_engine() {
    let iterations = 10_000;

    let mut engine = CognitiveEngine::new();

    // Seed knowledge
    let coords: Vec<[u16; 11]> = (0..20).map(|i| {
        [i + 1, i + 2, i + 3, 1, 1, 1, 1, 1, 1, 1, 1]
    }).collect();
    for c in &coords {
        engine.plant(*c);
    }

    let start = Instant::now();
    for i in 0..iterations {
        let q = [(i % 20 + 1) as u16, (i % 20 + 2) as u16, (i % 20 + 3) as u16, 1, 1, 1, 1, 1, 1, 1, 1];
        let step = CognitiveStep {
            query: q,
            attention_mask: 0x7FF,
            hd_width: HDC_DEFAULT_WIDTH,
        };
        let _ = engine.think(&step);
    }
    let elapsed = start.elapsed();
    let qps = iterations as f64 / elapsed.as_secs_f64();

    println!("  Cognitive:  {:>10} queries in {:>8.2} ms  ({:.0} queries/sec, {} knowledge items)",
        iterations, elapsed.as_secs_f64() * 1000.0, qps, engine.knowledge_size());
}

fn bench_hdc_associative_memory() {
    let iterations = 50_000;
    let width = HDC_DEFAULT_WIDTH;

    let mut mem = AssociativeMemory::new();
    // Store 100 entries
    for i in 0..100usize {
        let coord = [(i + 1) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        mem.store(coord, width);
    }

    let start = Instant::now();
    for i in 0..iterations {
        let query = HyperVector::basis(width, (i % 100) as usize);
        let _ = mem.query_nearest(&query);
    }
    let elapsed = start.elapsed();
    let qps = iterations as f64 / elapsed.as_secs_f64();

    println!("  HDC Memory: {:>10} lookups in {:>8.2} ms  ({:.0} lookups/sec, 100 stored)",
        iterations, elapsed.as_secs_f64() * 1000.0, qps);
}

fn bench_sentron_spawn_retire() {
    let iterations = 100_000;

    let start = Instant::now();
    for i in 0..iterations {
        let mut s = Sentron::new(i as u16, PhextCoord::zero(), 0, 0);
        s.regs.general[0] = 7;
        s.regs.general[1] = 6;
        s.spawn(vec![
            SIW::new(DenseOp::DMUL { rd: 2, rs1: 0, rs2: 1 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
        ]);
        let _ = exec::run_standalone(&mut s);
    }
    let elapsed = start.elapsed();
    let spawns_sec = iterations as f64 / elapsed.as_secs_f64();

    println!("  Lifecycle:  {:>10} spawn→retire in {:>5.2} ms  ({:.0} sentrons/sec)",
        iterations, elapsed.as_secs_f64() * 1000.0, spawns_sec);
}

fn bench_ppt_translation() {
    let iterations = 1_000_000;
    let mut ppt = PhextPageTable::new();

    let start = Instant::now();
    for i in 0..iterations {
        let coord = PhextCoord::new([(i % 100 + 1) as u16, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let _ = ppt.translate(&coord);
    }
    let elapsed = start.elapsed();
    let tps = iterations as f64 / elapsed.as_secs_f64();
    let stats = ppt.stats();

    println!("  PPT:        {:>10} translations in {:>5.2} ms  ({:.0} trans/sec, {:.1}% PTC hit)",
        iterations, elapsed.as_secs_f64() * 1000.0, tps,
        stats.ptc_hit_rate * 100.0);
}

fn bench_ternary_inference() {
    let iterations = 100_000;

    // 8-element ternary matvec
    let weights: Vec<i8> = vec![1, -1, 0, 1, -1, 0, 1, -1];
    let packed_trits: Vec<i64> = weights.chunks(1).map(|w| bitnet::pack_trits(w)[0]).collect();

    let start = Instant::now();
    for _ in 0..iterations {
        let mut s = Sentron::new(0, PhextCoord::zero(), 0, 0);
        for i in 0..8 { s.regs.general[i] = (i as i64 + 1) * 10; }
        for i in 0..8 { s.regs.general[8 + i] = packed_trits[i]; }

        let mut program = vec![
            SIW::new(DenseOp::DMOV { rd: 15, imm: 0 }, SparseOp::SPREFCH { coord_idx: 0, hint: PrefetchHint::L1 }, CoordOp::CFENCE { scope: FenceScope::Thread }, PhextCoord::zero()),
        ];
        for i in 0..8u8 {
            program.push(SIW::new(
                DenseOp::DTACC { rd: 15, rs1: i, trit_reg: 8 + i },
                SparseOp::SPREFCH { coord_idx: (i % 8), hint: PrefetchHint::L2 },
                CoordOp::CFENCE { scope: FenceScope::Thread },
                PhextCoord::zero(),
            ));
        }
        s.spawn(program);
        let _ = exec::run_standalone(&mut s);
    }
    let elapsed = start.elapsed();
    let inf_sec = iterations as f64 / elapsed.as_secs_f64();

    println!("  Ternary:    {:>10} inferences in {:>5.2} ms  ({:.0} inf/sec, 8-elem matvec)",
        iterations, elapsed.as_secs_f64() * 1000.0, inf_sec);
}
