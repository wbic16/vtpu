//! W18 Benchmark: 2×4 Sentron Topology — Topology-Guided Parallel Dispatch
//!
//! "2×4 wiring per neuron within a sentron" — Will Bickford, 2026-02-18
//!
//! Key insight from first run: thread-spawn overhead is wrong.
//! The 2×4 topology maps to:
//!   - N/S connections (5 element rows) → SMT pairs on same physical core
//!   - E/W connections (8 neurons/element) → SIMD lanes (8 × u64 in AVX2)
//!
//! This benchmark measures:
//!   1. Topology struct correctness and navigation speed
//!   2. Pre-allocated thread pool fanout (persistent threads, channel-fed)
//!   3. SIMD-width simulation: 8-wide batch per element row (no AVX intrinsics,
//!      but structured data layout that the compiler can auto-vectorize)
//!
//! "Elevation, not aspiration": report what we measure, not what we hope for.

use std::time::{Instant, Duration};
use std::sync::mpsc::{self, SyncSender, Receiver};
use std::thread;

use vtpu_runtime::{
    SentronTopology, NeuronAddr, DispatchPlan,
    NEURONS_PER_SENTRON, CONNECTIONS_PER_NEURON,
};

// ── Work unit ──────────────────────────────────────────────────────────────

/// Minimal work unit: one vTPU SIW (Dense/Sparse/Coord op)
#[derive(Clone, Copy)]
struct WorkUnit {
    coord: [u64; 4],
    op:    u8,
}

impl WorkUnit {
    fn new(i: usize) -> Self {
        WorkUnit {
            coord: [i as u64, (i >> 8) as u64, (i >> 16) as u64, (i >> 24) as u64],
            op: (i % 3) as u8,
        }
    }

    #[inline(always)]
    fn execute(&self) -> u64 {
        match self.op {
            0 => self.coord[0].wrapping_add(self.coord[1]),
            1 => self.coord[0].wrapping_mul(self.coord[1] | 1),
            _ => self.coord[0] ^ self.coord[2],
        }
    }
}

// ── SIMD-style batch: 8 units wide (one element row) ──────────────────────

/// Process 8 work units simultaneously (one element row = one SIMD vector).
/// Data layout enables auto-vectorization by Rust/LLVM.
#[inline(always)]
fn execute_element_row(units: &[WorkUnit; 8]) -> u64 {
    // 8-wide parallel execution; LLVM can vectorize this loop
    let mut results = [0u64; 8];
    for i in 0..8 {
        results[i] = units[i].execute();
    }
    // Reduction: XOR-fold (preserves information, enables scalar writeback)
    results.iter().fold(0u64, |a, &b| a ^ b)
}

// ── Benchmark 1: Linear (baseline) ───────────────────────────────────────

fn bench_linear(work: &[WorkUnit]) -> (Duration, u64) {
    let t = Instant::now();
    let acc = work.iter().fold(0u64, |a, w| a.wrapping_add(w.execute()));
    (t.elapsed(), acc)
}

// ── Benchmark 2: Element-row batching (SIMD-width aligned) ───────────────

fn bench_element_rows(work: &[WorkUnit]) -> (Duration, u64) {
    let t = Instant::now();
    let mut acc = 0u64;
    let mut i = 0;
    while i + 8 <= work.len() {
        let row: [WorkUnit; 8] = work[i..i+8].try_into().unwrap();
        acc ^= execute_element_row(&row);
        i += 8;
    }
    // Tail
    for j in i..work.len() {
        acc = acc.wrapping_add(work[j].execute());
    }
    (t.elapsed(), acc)
}

// ── Benchmark 3: Pre-allocated thread pool (persistent threads) ───────────

struct WorkerPool {
    senders:  Vec<SyncSender<Option<Vec<WorkUnit>>>>,
    results:  Vec<Receiver<u64>>,
    _handles: Vec<thread::JoinHandle<()>>,
}

impl WorkerPool {
    fn new(n_workers: usize) -> Self {
        let mut senders  = Vec::new();
        let mut results  = Vec::new();
        let mut handles  = Vec::new();

        for _ in 0..n_workers {
            let (tx, rx) = mpsc::sync_channel::<Option<Vec<WorkUnit>>>(1);
            let (rtx, rrx) = mpsc::sync_channel::<u64>(1);

            let h = thread::spawn(move || {
                while let Ok(Some(work)) = rx.recv() {
                    let acc = work.iter().fold(0u64, |a, w| a.wrapping_add(w.execute()));
                    let _ = rtx.send(acc);
                }
            });

            senders.push(tx);
            results.push(rrx);
            handles.push(h);
        }

        WorkerPool { senders, results, _handles: handles }
    }

    /// Dispatch work to all workers, wait for results
    fn dispatch(&self, chunks: Vec<Vec<WorkUnit>>) -> u64 {
        for (tx, chunk) in self.senders.iter().zip(chunks.into_iter()) {
            tx.send(Some(chunk)).unwrap();
        }
        self.results.iter().fold(0u64, |a, rx| a.wrapping_add(rx.recv().unwrap()))
    }
}

fn bench_pool(pool: &WorkerPool, work: &[WorkUnit], n_workers: usize) -> (Duration, u64) {
    let chunk_size = work.len() / n_workers;
    let chunks: Vec<Vec<WorkUnit>> = (0..n_workers)
        .map(|i| {
            let start = i * chunk_size;
            let end = if i == n_workers - 1 { work.len() } else { start + chunk_size };
            work[start..end].to_vec()
        })
        .collect();

    let t = Instant::now();
    let acc = pool.dispatch(chunks);
    (t.elapsed(), acc)
}

// ── Benchmark 4: Topology-guided placement ────────────────────────────────
// Uses the topology to determine which neurons should execute concurrently.
// N/S pairs → same "logical worker" (simulates SMT threads sharing L1 cache)
// This measures topology traversal overhead in the hot path.

fn bench_topology_guided(topo: &SentronTopology, work: &[WorkUnit]) -> (Duration, u64) {
    let origin = NeuronAddr::new(2, 4); // Earth center
    let plan   = DispatchPlan::build(topo, origin, work.len());
    let chunk  = plan.items_per_target.max(1);

    let t = Instant::now();
    let mut acc = 0u64;

    // Sequential simulation of 4-way fanout — measures topology overhead
    for (i, &_target) in plan.targets.iter().enumerate() {
        let start = (i * chunk).min(work.len());
        let end   = ((i + 1) * chunk).min(work.len());
        for j in start..end {
            acc = acc.wrapping_add(work[j].execute());
        }
    }
    // Remainder on origin
    let rem_start = (4 * chunk).min(work.len());
    for j in rem_start..work.len() {
        acc = acc.wrapping_add(work[j].execute());
    }

    (t.elapsed(), acc)
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn ops_per_cycle(ops: usize, d: Duration) -> f64 {
    const GHZ: f64 = 5.0e9;
    let cycles = d.as_secs_f64() * GHZ;
    if cycles < 1.0 { return 0.0; }
    ops as f64 / cycles
}

fn speedup(base: Duration, vs: Duration) -> f64 {
    if vs.is_zero() { return 0.0; }
    base.as_secs_f64() / vs.as_secs_f64()
}

fn fmt_dur(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000        { format!("{} ns",    ns) }
    else if ns < 1_000_000 { format!("{:.1} µs", ns as f64 / 1e3) }
    else                   { format!("{:.1} ms", ns as f64 / 1e6) }
}

// ── Main ──────────────────────────────────────────────────────────────────

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  W18: Sentron 2×4 Topology — Dispatch Benchmark");
    println!("  {} neurons × {} connections = {} directed edges",
        NEURONS_PER_SENTRON, CONNECTIONS_PER_NEURON,
        NEURONS_PER_SENTRON * CONNECTIONS_PER_NEURON);
    println!("═══════════════════════════════════════════════════════════════\n");

    // ── Topology verification ──────────────────────────────────────────
    let topo = SentronTopology::new();
    match topo.verify() {
        Ok(()) => println!("✓ Topology: symmetric, no self-loops\n"),
        Err(e)  => { eprintln!("✗ Topology error: {}", e); return; }
    }

    // ── Reachability ───────────────────────────────────────────────────
    let origin = NeuronAddr::new(2, 4);
    print!("Reachability from Earth[4]:");
    for hops in [1u8, 2, 3, 4, 5] {
        let r = topo.reachable(origin, hops);
        print!("  {}→{}", hops, r.len());
    }
    println!("  (of 40)");

    // 5 hops reaches 95% of sentron → coordination diameter ≈ 5 cycles
    println!("  Coordination diameter: ~5 cycles to propagate across full sentron\n");

    // ── Timing note ───────────────────────────────────────────────────
    // We measure throughput at N=1M ops. Thread pool has 4 persistent workers.
    // N/worker = 250K. Measures real parallel throughput vs. sequential.

    const N: usize = 1_000_000;
    const ITERS: u32 = 20;

    println!("Generating {} work units...", N);
    let work: Vec<WorkUnit> = (0..N).map(WorkUnit::new).collect();

    // Pre-allocate thread pool ONCE (simulates persistent sentron workers)
    let pool4 = WorkerPool::new(4);
    let pool8 = WorkerPool::new(8);

    // ── Warmup ────────────────────────────────────────────────────────
    for _ in 0..5 {
        let _ = bench_linear(&work[..10_000]);
        let _ = bench_element_rows(&work[..10_000]);
    }
    // Warm up pools
    for _ in 0..3 {
        let _ = bench_pool(&pool4, &work[..10_000], 4);
        let _ = bench_pool(&pool8, &work[..10_000], 8);
    }

    // ── Benchmark runs ────────────────────────────────────────────────
    let mut linear_total = Duration::ZERO;
    let mut rows_total   = Duration::ZERO;
    let mut pool4_total  = Duration::ZERO;
    let mut pool8_total  = Duration::ZERO;
    let mut topo_total   = Duration::ZERO;
    let mut checksum = 0u64;

    for _ in 0..ITERS {
        let (d, a) = bench_linear(&work);           linear_total += d; checksum ^= a;
        let (d, a) = bench_element_rows(&work);     rows_total   += d; checksum ^= a;
        let (d, a) = bench_pool(&pool4, &work, 4);  pool4_total  += d; checksum ^= a;
        let (d, a) = bench_pool(&pool8, &work, 8);  pool8_total  += d; checksum ^= a;
        let (d, a) = bench_topology_guided(&topo, &work); topo_total += d; checksum ^= a;
    }

    let linear_avg = linear_total / ITERS;
    let rows_avg   = rows_total   / ITERS;
    let pool4_avg  = pool4_total  / ITERS;
    let pool8_avg  = pool8_total  / ITERS;
    let topo_avg   = topo_total   / ITERS;

    println!("\n{:<30} {:>10} {:>12} {:>8}",
        "Method", "Avg time", "ops/cycle", "Speedup");
    println!("{}", "─".repeat(64));

    let lin_opc = ops_per_cycle(N, linear_avg);
    println!("{:<30} {:>10} {:>12.2} {:>8}",
        "Linear (baseline)", fmt_dur(linear_avg), lin_opc, "1.00×");

    let rows_opc = ops_per_cycle(N, rows_avg);
    println!("{:<30} {:>10} {:>12.2} {:>8.2}×  {}",
        "Element-row batches (8-wide)", fmt_dur(rows_avg), rows_opc,
        speedup(linear_avg, rows_avg),
        if rows_opc > lin_opc { "✓ faster (SIMD layout)" } else { "○ similar" });

    let topo_opc = ops_per_cycle(N, topo_avg);
    println!("{:<30} {:>10} {:>12.2} {:>8.2}×  {}",
        "Topology-guided (sequential)", fmt_dur(topo_avg), topo_opc,
        speedup(linear_avg, topo_avg),
        if topo_opc > lin_opc * 0.9 { "✓ low overhead" } else { "○ overhead visible" });

    let pool4_opc = ops_per_cycle(N, pool4_avg);
    println!("{:<30} {:>10} {:>12.2} {:>8.2}×  {}",
        "Thread pool × 4 (persistent)", fmt_dur(pool4_avg), pool4_opc,
        speedup(linear_avg, pool4_avg),
        if pool4_opc > lin_opc { "✓ parallel gain" } else { "○ overhead > gain" });

    let pool8_opc = ops_per_cycle(N, pool8_avg);
    println!("{:<30} {:>10} {:>12.2} {:>8.2}×  {}",
        "Thread pool × 8 (persistent)", fmt_dur(pool8_avg), pool8_opc,
        speedup(linear_avg, pool8_avg),
        if pool8_opc > lin_opc { "✓ parallel gain" } else { "○ overhead > gain" });

    println!("\nchecksum = {:#018x} (non-zero = compiler didn't elide work)", checksum);

    // ── Topology navigation speed ─────────────────────────────────────
    println!("\n── Topology navigation overhead ──────────────────────────────");
    const NAV_OPS: usize = 40_000_000;
    let t = Instant::now();
    let mut addr = NeuronAddr::new(0, 0);
    for i in 0..NAV_OPS {
        // Walk the topology in a pseudo-random pattern
        let dir = match i % 4 {
            0 => vtpu_runtime::Direction::North,
            1 => vtpu_runtime::Direction::East,
            2 => vtpu_runtime::Direction::South,
            _ => vtpu_runtime::Direction::West,
        };
        addr = topo.neighbor_in(addr, dir);
    }
    let nav_elapsed = t.elapsed();
    let nav_opc = ops_per_cycle(NAV_OPS, nav_elapsed);
    println!("  {} topology hops in {}", NAV_OPS, fmt_dur(nav_elapsed));
    println!("  {:.1} ns/hop = {:.1} ops/cycle", 
        nav_elapsed.as_secs_f64() / NAV_OPS as f64 * 1e9, nav_opc);
    println!("  Final addr: ({},{}) — confirms no dead-code elimination", addr.row, addr.col);

    // ── Summary ───────────────────────────────────────────────────────
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Sentron topology: {} neurons, {} connections, {} edges verified",
        NEURONS_PER_SENTRON, CONNECTIONS_PER_NEURON,
        NEURONS_PER_SENTRON * CONNECTIONS_PER_NEURON);
    println!("  Phase 0 baseline:     3.0 ops/cycle");
    println!("  Linear (this run):  {:.1} ops/cycle", lin_opc);
    println!("  Best parallel:      {:.1} ops/cycle ({:.1}×)", 
        pool4_opc.max(pool8_opc),
        speedup(linear_avg, if pool4_avg < pool8_avg { pool4_avg } else { pool8_avg }));
    println!("  Navigation speed:   {:.1} ops/cycle (topology lookup cost)", nav_opc);
    println!("═══════════════════════════════════════════════════════════════");
}
