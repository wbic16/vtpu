//! vTPU Phase 0 v4: 3-Pipe Retirement Benchmark
//!
//! Targeting ≥2.5 ops/cycle on Zen 4. Key findings from v3:
//! - 4x unroll = 2.16 opc (sweet spot for register pressure)
//! - Higher unroll hurts due to spills
//! - Z-order needs LUT, not bit manipulation
//! 
//! v4: hand-tuned 4x/6x variants, Z-order LUT, explicit register discipline

use std::time::Instant;
use std::hint::black_box;

const SLAB_SIZE: usize = 1 << 20;
const ITERS: u64 = 200_000_000;

struct R { name: &'static str, elapsed: f64, total_ops: u64 }
impl R {
    fn opc(&self) -> f64 { self.total_ops as f64 / (self.elapsed * 4.0e9) }
    fn gops(&self) -> f64 { self.total_ops as f64 / self.elapsed / 1e9 }
}

// ---------------------------------------------------------------------------
// 1. Baseline
// ---------------------------------------------------------------------------
#[inline(never)]
fn b_baseline() -> R {
    let mut a: f64 = 1.0001;
    let t = Instant::now();
    for _ in 0..ITERS { a = a * 1.0000001 + 0.0000001; }
    let e = t.elapsed().as_secs_f64();
    black_box(a);
    R { name: "D-only baseline", elapsed: e, total_ops: ITERS }
}

// ---------------------------------------------------------------------------
// 2. 4x unroll (best from v3), tightened
// ---------------------------------------------------------------------------
#[inline(never)]
fn b_4x(slab: &mut [f64]) -> R {
    let (mut d0, mut d1, mut d2, mut d3) = (1.0001f64, 1.0002, 1.0003, 1.0004);
    let mut si: usize = 0;
    let mut ca: u64 = 0;
    let n = ITERS / 4;
    let t = Instant::now();
    for _ in 0..n {
        d0 = d0 * 1.0000001 + 0.0000001;
        si = (si + 1) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(si) }.to_bits());

        d1 = d1 * 1.0000002 + 0.0000002;
        si = (si + 7) & (SLAB_SIZE - 1);
        ca = ca.rotate_left(7) ^ unsafe { *slab.get_unchecked(si) }.to_bits();

        d2 = d2 * 1.0000003 + 0.0000003;
        unsafe { *slab.get_unchecked_mut((si + 3) & (SLAB_SIZE - 1)) = d0; }
        ca = ca.wrapping_mul(0x517cc1b727220a95);

        d3 = d3 * 1.0000004 + 0.0000004;
        si = (si + 13) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(si) }.to_bits());
    }
    let e = t.elapsed().as_secs_f64();
    black_box(d0); black_box(d1); black_box(d2); black_box(d3); black_box(ca);
    R { name: "3-pipe 4x unroll", elapsed: e, total_ops: n * 4 * 3 }
}

// ---------------------------------------------------------------------------
// 3. 4x with TWO independent C chains (reduce C-pipe dependency chain)
// ---------------------------------------------------------------------------
#[inline(never)]
fn b_4x_2c(slab: &mut [f64]) -> R {
    let (mut d0, mut d1, mut d2, mut d3) = (1.0001f64, 1.0002, 1.0003, 1.0004);
    let mut si: usize = 0;
    let (mut ca, mut cb): (u64, u64) = (0, 1);
    let n = ITERS / 4;
    let t = Instant::now();
    for _ in 0..n {
        d0 = d0 * 1.0000001 + 0.0000001;
        si = (si + 1) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(si) }.to_bits());

        d1 = d1 * 1.0000002 + 0.0000002;
        si = (si + 7) & (SLAB_SIZE - 1);
        cb = cb.wrapping_add(unsafe { *slab.get_unchecked(si) }.to_bits());

        d2 = d2 * 1.0000003 + 0.0000003;
        unsafe { *slab.get_unchecked_mut((si + 3) & (SLAB_SIZE - 1)) = d0; }
        ca = ca.wrapping_mul(0x517cc1b727220a95);

        d3 = d3 * 1.0000004 + 0.0000004;
        si = (si + 13) & (SLAB_SIZE - 1);
        cb = cb.rotate_left(7) ^ unsafe { *slab.get_unchecked(si) }.to_bits();
    }
    let e = t.elapsed().as_secs_f64();
    black_box(d0); black_box(d1); black_box(d2); black_box(d3);
    black_box(ca); black_box(cb);
    R { name: "3-pipe 4x + 2 C-chains", elapsed: e, total_ops: n * 4 * 3 }
}

// ---------------------------------------------------------------------------
// 4. 6x unroll (between 4x sweet spot and 8x register pressure)
// ---------------------------------------------------------------------------
#[inline(never)]
fn b_6x(slab: &mut [f64]) -> R {
    let (mut d0, mut d1, mut d2) = (1.0001f64, 1.0002, 1.0003);
    let (mut d3, mut d4, mut d5) = (1.0004f64, 1.0005, 1.0006);
    let mut si: usize = 0;
    let (mut ca, mut cb): (u64, u64) = (0, 1);
    let n = ITERS / 6;
    let t = Instant::now();
    for _ in 0..n {
        d0 = d0 * 1.0000001 + 0.0000001;
        si = (si + 1) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(si) }.to_bits());

        d1 = d1 * 1.0000002 + 0.0000002;
        si = (si + 7) & (SLAB_SIZE - 1);
        cb = cb.wrapping_add(unsafe { *slab.get_unchecked(si) }.to_bits());

        d2 = d2 * 1.0000003 + 0.0000003;
        unsafe { *slab.get_unchecked_mut((si + 3) & (SLAB_SIZE - 1)) = d0; }
        ca = ca.wrapping_mul(0x517cc1b727220a95);

        d3 = d3 * 1.0000004 + 0.0000004;
        si = (si + 13) & (SLAB_SIZE - 1);
        cb = cb.rotate_left(7) ^ unsafe { *slab.get_unchecked(si) }.to_bits();

        d4 = d4 * 1.0000005 + 0.0000005;
        si = (si + 17) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(si) }.to_bits());

        d5 = d5 * 1.0000006 + 0.0000006;
        unsafe { *slab.get_unchecked_mut((si + 19) & (SLAB_SIZE - 1)) = d4; }
        cb = cb.wrapping_mul(0x9E3779B97F4A7C15);
    }
    let e = t.elapsed().as_secs_f64();
    black_box(d0); black_box(d1); black_box(d2);
    black_box(d3); black_box(d4); black_box(d5);
    black_box(ca); black_box(cb);
    R { name: "3-pipe 6x + 2 C-chains", elapsed: e, total_ops: n * 6 * 3 }
}

// ---------------------------------------------------------------------------
// 5. Phext Z-order with LUT
// ---------------------------------------------------------------------------
#[inline(never)]
fn b_phext_lut(slab: &mut [f64]) -> R {
    // Pre-build Z-order LUT for 7-bit values (128 entries per dim)
    let mut z_lut: [[usize; 128]; 3] = [[0; 128]; 3];
    for v in 0..128usize {
        for b in 0..7 {
            z_lut[0][v] |= ((v >> b) & 1) << (b * 3);
            z_lut[1][v] |= ((v >> b) & 1) << (b * 3 + 1);
            z_lut[2][v] |= ((v >> b) & 1) << (b * 3 + 2);
        }
    }

    #[inline(always)]
    fn z_idx(lut: &[[usize; 128]; 3], d0: u16, d1: u16, d2: u16, mask: usize) -> usize {
        (lut[0][(d0 & 0x7F) as usize] | lut[1][(d1 & 0x7F) as usize] | lut[2][(d2 & 0x7F) as usize]) & mask
    }

    let (mut da, mut db, mut dc, mut dd) = (1.0001f64, 2.0001f64, 3.0001f64, 4.0001f64);
    let mut ca: u64 = 0;
    let (mut p0, mut p1, mut p2): (u16, u16, u16) = (0, 0, 0);
    let mask = SLAB_SIZE - 1;

    let n = ITERS / 4;
    let t = Instant::now();
    for _ in 0..n {
        da = da * 1.0000001 + 0.0000001;
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(z_idx(&z_lut, p0, p1, p2, mask)) }.to_bits());

        db = db * 1.0000002 + 0.0000002;
        p0 = p0.wrapping_add(1) & 0x7F;
        unsafe { *slab.get_unchecked_mut(z_idx(&z_lut, p0, p1, p2, mask)) = da; }
        ca = ca.rotate_left(5) ^ db.to_bits();

        dc = dc * 1.0000003 + 0.0000003;
        p1 = p1.wrapping_add(1) & 0x7F;
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(z_idx(&z_lut, p0, p1, p2, mask)) }.to_bits());

        dd = dd * 1.0000004 + 0.0000004;
        p2 = p2.wrapping_add(1) & 0x7F;
        ca = ca.wrapping_mul(0x9E3779B97F4A7C15);
        let _ = unsafe { *slab.get_unchecked(z_idx(&z_lut, p0, p1, p2, mask)) };
    }
    let e = t.elapsed().as_secs_f64();
    black_box(da); black_box(db); black_box(dc); black_box(dd); black_box(ca);
    R { name: "3-pipe phext Z-LUT", elapsed: e, total_ops: n * 4 * 3 }
}

// ---------------------------------------------------------------------------
// 6. Pure throughput: 4 independent D, minimal S/C dependency
// ---------------------------------------------------------------------------
#[inline(never)]
fn b_throughput(slab: &[f64]) -> R {
    // Four FULLY independent D-chains, S reads from sequential addrs, C is trivial
    let (mut d0, mut d1, mut d2, mut d3) = (1.0001f64, 1.0002, 1.0003, 1.0004);
    let mut si: usize = 0;
    let mut ca: u64 = 0;
    let n = ITERS / 4;
    let t = Instant::now();
    for _ in 0..n {
        // Each D is independent. S advances linearly (perfect prefetch). C is simple add.
        d0 = d0 * 1.0000001 + 0.0000001;
        si = si.wrapping_add(1) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(si as u64);

        d1 = d1 * 1.0000002 + 0.0000002;
        si = si.wrapping_add(1) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(si) } as u64);

        d2 = d2 * 1.0000003 + 0.0000003;
        si = si.wrapping_add(1) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(si as u64);

        d3 = d3 * 1.0000004 + 0.0000004;
        si = si.wrapping_add(1) & (SLAB_SIZE - 1);
        ca = ca.wrapping_add(unsafe { *slab.get_unchecked(si) } as u64);
    }
    let e = t.elapsed().as_secs_f64();
    black_box(d0); black_box(d1); black_box(d2); black_box(d3); black_box(ca);
    R { name: "4D indep + seq S + min C", elapsed: e, total_ops: n * 4 * 3 }
}

// ---------------------------------------------------------------------------
fn main() {
    println!();
    println!("══════════════════════════════════════════════════════════════════════");
    println!("  vTPU Phase 0 v4 │ Zen 4 @ 4.0 GHz │ {} M iters", ITERS / 1_000_000);
    println!("══════════════════════════════════════════════════════════════════════");
    println!();

    let mut slab = vec![0.001f64; SLAB_SIZE];
    for i in 0..SLAB_SIZE { slab[i] = (i as f64) * 0.001; }

    let rr = vec![
        b_baseline(),
        b_4x(&mut slab),
        b_4x_2c(&mut slab),
        b_6x(&mut slab),
        b_phext_lut(&mut slab),
        b_throughput(&slab),
    ];

    println!("┌──────────────────────────────┬─────────┬───────────┬──────────┬────┐");
    println!("│ Benchmark                    │ Time ms │ Ops/Cycle │ Gops/sec │    │");
    println!("├──────────────────────────────┼─────────┼───────────┼──────────┼────┤");
    for r in &rr {
        let o = r.opc();
        let m = if o >= 2.5 { "✅" } else if o >= 2.0 { "🔶" } else if o >= 1.5 { "──" } else { "  " };
        println!("│ {:28} │ {:7.1} │ {:9.3} │ {:8.2} │ {} │",
            r.name, r.elapsed * 1e3, o, r.gops(), m);
    }
    println!("└──────────────────────────────┴─────────┴───────────┴──────────┴────┘");
    println!();

    let bl = rr[0].opc();
    let best = rr.iter().max_by(|a, b| a.opc().partial_cmp(&b.opc()).unwrap()).unwrap();
    println!("  Peak: {} → {:.3} ops/cycle ({:.1}x baseline)", best.name, best.opc(), best.opc() / bl);
    println!("  Gap to CPI-3: {:.3}", 3.0 - best.opc());
    if best.opc() >= 2.5 {
        println!("  🎯 CPI-3: WITHIN RANGE");
    } else if best.opc() >= 2.0 {
        println!("  🔶 APPROACHING — bottleneck: FMA latency chain (4 cycles on Zen 4)");
        println!("     To break 2.5: need ≥5 independent D-chains or AVX-512 vectorization");
    }
    println!();
}
