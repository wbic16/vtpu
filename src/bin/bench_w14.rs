// vTPU W14 Benchmark Suite — Real Workload Performance
//
// Beyond Phase 0 gate (ops/cycle). Measures:
// 1. HDC encode throughput (coords/sec)
// 2. Associative memory recall latency (ns/query)
// 3. Ternary inference throughput (matvecs/sec)
// 4. Packer efficiency (packing ratio)
// 5. Multi-sentron throughput (ops/sec with 2+ sentrons)
// 6. Memory subsystem (gather/scatter bandwidth)
// 7. End-to-end cognitive loop (think ops/sec)

use vtpu_runtime::*;
use vtpu_runtime::hdc::{HyperVector, AssociativeMemory, HDC_DEFAULT_WIDTH};
use vtpu_runtime::memory::Memory;
use vtpu_runtime::sentron::Sentron;
use vtpu_runtime::packer::{ScalarOp, pack};
use vtpu_runtime::bitnet;
use vtpu_runtime::exec;
use std::time::Instant;

struct BenchResult {
    name: &'static str,
    metric: f64,
    unit: &'static str,
}

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("  vTPU W14 Benchmark Suite — Real Workload Performance");
    println!("═══════════════════════════════════════════════════════\n");

    let results = vec![
        bench_hdc_encode(),
        bench_hdc_recall(),
        bench_hdc_recall_1000(),
        bench_ternary_throughput(),
        bench_packer_efficiency(),
        bench_multi_sentron(),
        bench_memory_bandwidth(),
        bench_cognitive_loop(),
    ];

    println!("\n═══════════════════════════════════════════════════════");
    println!("  SUMMARY");
    println!("═══════════════════════════════════════════════════════");
    for r in &results {
        println!("  {:<40} {:>12.1} {}", r.name, r.metric, r.unit);
    }
    println!("═══════════════════════════════════════════════════════");
}

/// Benchmark 1: HDC coordinate encoding throughput
fn bench_hdc_encode() -> BenchResult {
    let n = 100_000;
    let start = Instant::now();
    for i in 0..n {
        let coord = [
            (i % 10 + 1) as u16, ((i / 10) % 10 + 1) as u16,
            ((i / 100) % 10 + 1) as u16, 1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let _hv = HyperVector::from_coord(&coord, HDC_DEFAULT_WIDTH);
    }
    let elapsed = start.elapsed();
    let rate = n as f64 / elapsed.as_secs_f64();
    println!("  HDC encode:        {:>10} coords in {:>8.2} ms ({:.0} coords/sec)",
        n, elapsed.as_secs_f64() * 1000.0, rate);
    BenchResult { name: "HDC encode throughput", metric: rate, unit: "coords/sec" }
}

/// Benchmark 2: Associative memory recall (100 entries)
fn bench_hdc_recall() -> BenchResult {
    let mut amem = AssociativeMemory::new();
    for i in 1..=100u16 {
        let mut c = [1u16; 11];
        c[0] = i;
        amem.store(c, HDC_DEFAULT_WIDTH);
    }

    let n = 10_000;
    let start = Instant::now();
    for i in 0..n {
        let mut c = [1u16; 11];
        c[0] = (i % 100 + 1) as u16;
        let query = HyperVector::from_coord(&c, HDC_DEFAULT_WIDTH);
        let _result = amem.query_nearest(&query);
    }
    let elapsed = start.elapsed();
    let ns_per = elapsed.as_nanos() as f64 / n as f64;
    println!("  HDC recall (100):  {:>10} queries in {:>8.2} ms ({:.0} ns/query)",
        n, elapsed.as_secs_f64() * 1000.0, ns_per);
    BenchResult { name: "HDC recall (100 entries)", metric: ns_per, unit: "ns/query" }
}

/// Benchmark 3: Associative memory recall (1000 entries)
fn bench_hdc_recall_1000() -> BenchResult {
    let mut amem = AssociativeMemory::new();
    for i in 1..=1000u16 {
        let mut c = [1u16; 11];
        c[0] = i % 10 + 1;
        c[1] = (i / 10) % 10 + 1;
        c[2] = (i / 100) % 10 + 1;
        amem.store(c, HDC_DEFAULT_WIDTH);
    }

    let n = 1_000;
    let start = Instant::now();
    for i in 0..n {
        let mut c = [1u16; 11];
        c[0] = (i % 10 + 1) as u16;
        c[1] = ((i / 10) % 10 + 1) as u16;
        let query = HyperVector::from_coord(&c, HDC_DEFAULT_WIDTH);
        let _result = amem.query_nearest(&query);
    }
    let elapsed = start.elapsed();
    let ns_per = elapsed.as_nanos() as f64 / n as f64;
    println!("  HDC recall (1000): {:>10} queries in {:>8.2} ms ({:.0} ns/query)",
        n, elapsed.as_secs_f64() * 1000.0, ns_per);
    BenchResult { name: "HDC recall (1000 entries)", metric: ns_per, unit: "ns/query" }
}

/// Benchmark 4: Ternary matmul throughput
fn bench_ternary_throughput() -> BenchResult {
    // 64x64 ternary weight matrix, applied to 64-element activation vector
    let rows = 64;
    let cols = 64;
    let mut weights = vec![0i8; rows * cols];
    for i in 0..weights.len() {
        weights[i] = match i % 3 { 0 => 1, 1 => -1, _ => 0 };
    }
    let packed_rows: Vec<Vec<i64>> = weights.chunks(cols)
        .map(|row| bitnet::pack_trits(row))
        .collect();

    let activations: Vec<i64> = (0..cols as i64).collect();

    let n = 10_000;
    let start = Instant::now();
    for _ in 0..n {
        let mut _output = vec![0i64; rows];
        for (r, packed_row) in packed_rows.iter().enumerate() {
            let mut sum = 0i64;
            for (c_chunk, &trits) in packed_row.iter().enumerate() {
                let base = c_chunk * 32;
                for bit in 0..32 {
                    let idx = base + bit;
                    if idx < cols {
                        let trit = (trits as u64 >> (bit * 2)) & 0x3;
                        match trit {
                            0b01 => sum += activations[idx],
                            0b10 => sum -= activations[idx],
                            _ => {}
                        }
                    }
                }
            }
            _output[r] = sum;
        }
    }
    let elapsed = start.elapsed();
    let rate = n as f64 / elapsed.as_secs_f64();
    println!("  Ternary 64x64:     {:>10} matvecs in {:>8.2} ms ({:.0} matvecs/sec)",
        n, elapsed.as_secs_f64() * 1000.0, rate);
    BenchResult { name: "Ternary 64x64 matvec", metric: rate, unit: "matvecs/sec" }
}

/// Benchmark 5: Packer efficiency
fn bench_packer_efficiency() -> BenchResult {
    // 100 scalar ops, mixed D/S/C
    let ops: Vec<ScalarOp> = (0..100).map(|i| {
        match i % 3 {
            0 => ScalarOp::D(DenseOp::DADD { rd: (i % 15 + 1) as u8, rs1: 1, rs2: 2 }),
            1 => ScalarOp::S(SparseOp::SGATHER { rd: (i % 15 + 1) as u8, coord_idx: 0, width: 64 }),
            _ => ScalarOp::C(CoordOp::CNOP),
        }
    }).collect();

    let start = Instant::now();
    let mut total_siws = 0;
    let n = 10_000;
    for _ in 0..n {
        let result = pack(&ops);
        total_siws += result.stream.len();
    }
    let elapsed = start.elapsed();
    let avg_siws = total_siws as f64 / n as f64;
    let ratio = 100.0 / avg_siws;
    let packs_per_sec = n as f64 / elapsed.as_secs_f64();
    println!("  Packer (100 ops):  {:>10} packs in {:>8.2} ms (ratio: {:.2}x, {} SIWs avg, {:.0} packs/sec)",
        n, elapsed.as_secs_f64() * 1000.0, ratio, avg_siws as u64, packs_per_sec);
    BenchResult { name: "Packer efficiency", metric: ratio, unit: "x compression" }
}

/// Benchmark 6: Multi-sentron throughput
fn bench_multi_sentron() -> BenchResult {
    let mut mem = Memory::new();
    let program: Vec<SIW> = (0..1000).map(|i| {
        SIW::new(
            DenseOp::DADD { rd: ((i % 14) + 1) as u8, rs1: 1, rs2: 2 },
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }).collect();

    let n_sentrons = 8;
    let start = Instant::now();
    let mut total_ops = 0u64;
    for s in 0..n_sentrons {
        let mut sentron = Sentron::new(s as u16, PhextCoord::zero(), 0, s as u8);
        sentron.regs.general[1] = 1;
        sentron.regs.general[2] = 1;
        sentron.spawn(program.clone());
        let stats = exec::run(&mut sentron, &mut mem);
        total_ops += stats.ops_retired;
    }
    let elapsed = start.elapsed();
    let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();
    println!("  Multi-sentron (8): {:>10} ops in {:>8.2} ms ({:.0} ops/sec)",
        total_ops, elapsed.as_secs_f64() * 1000.0, ops_per_sec);
    BenchResult { name: "Multi-sentron (8×1000 SIWs)", metric: ops_per_sec, unit: "ops/sec" }
}

/// Benchmark 7: Memory gather/scatter bandwidth
fn bench_memory_bandwidth() -> BenchResult {
    let mut mem = Memory::new();
    let n = 100_000;

    // Scatter
    let start = Instant::now();
    for i in 0..n {
        let mut c = [1u16; 11];
        c[0] = (i % 100 + 1) as u16;
        c[1] = ((i / 100) % 100 + 1) as u16;
        let coord = PhextCoord::new(c);
        mem.scatter_i64(&coord, i as i64);
    }
    let scatter_time = start.elapsed();

    // Gather
    let start = Instant::now();
    let mut _sum = 0i64;
    for i in 0..n {
        let mut c = [1u16; 11];
        c[0] = (i % 100 + 1) as u16;
        c[1] = ((i / 100) % 100 + 1) as u16;
        let coord = PhextCoord::new(c);
        _sum += mem.gather_i64(&coord);
    }
    let gather_time = start.elapsed();

    let total = scatter_time + gather_time;
    let ops_per_sec = (n * 2) as f64 / total.as_secs_f64();
    println!("  Memory bandwidth:  {:>10} ops in {:>8.2} ms ({:.0} ops/sec, scatter: {:.2}ms, gather: {:.2}ms)",
        n * 2, total.as_secs_f64() * 1000.0, ops_per_sec,
        scatter_time.as_secs_f64() * 1000.0, gather_time.as_secs_f64() * 1000.0);
    BenchResult { name: "Memory scatter+gather", metric: ops_per_sec, unit: "ops/sec" }
}

/// Benchmark 8: End-to-end cognitive loop
fn bench_cognitive_loop() -> BenchResult {
    // Simulate: encode coord → store in memory → gather → add context → store result
    let mut mem = Memory::new();
    let n = 10_000;
    let start = Instant::now();

    for i in 0..n {
        let mut coord = [1u16; 11];
        coord[0] = (i % 10 + 1) as u16;
        coord[1] = ((i / 10) % 10 + 1) as u16;
        let pc = PhextCoord::new(coord);

        // Encode (HDC)
        let _hv = HyperVector::from_coord(&coord, HDC_DEFAULT_WIDTH);

        // Store
        mem.scatter_i64(&pc, i as i64);

        // Retrieve + process
        let val = mem.gather_i64(&pc);
        let result = val.wrapping_mul(3).wrapping_add(7);

        // Store result at offset coord
        coord[2] = 2;
        let out_pc = PhextCoord::new(coord);
        mem.scatter_i64(&out_pc, result);
    }

    let elapsed = start.elapsed();
    let rate = n as f64 / elapsed.as_secs_f64();
    println!("  Cognitive loop:    {:>10} cycles in {:>8.2} ms ({:.0} cycles/sec)",
        n, elapsed.as_secs_f64() * 1000.0, rate);
    BenchResult { name: "Cognitive loop (e2e)", metric: rate, unit: "cycles/sec" }
}
