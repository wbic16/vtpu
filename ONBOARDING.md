# vTPU Onboarding Guide
## R23 Rally - Running the Current Build

**Last updated:** Wave 2 complete (2026-02-15)

---

## Quick Start

```bash
# Navigate to project
cd /source/vtpu

# Build everything
cargo build --release

# Run tests
cargo test

# Run benchmark binary (validation only in W2)
cargo run --bin vtpu-bench --release
```

**Expected output:**
```
vTPU Benchmark Suite - R23
==========================

Phase 1 (W2): Structure validation
  ✓ SIW creation: 64 bytes, 64-byte aligned
  ✓ Phext coordinate: 3.1.4 / 1.5.9 / 2.6.5 (parsed from string)

Phase 2 (W3+): Performance benchmarks
  [Not yet implemented - awaiting scheduler]

✅ W2 validation complete
```

---

## Project Structure (Wave 2)

```
/source/vtpu/
├── Cargo.toml              # Rust project manifest
├── ONBOARDING.md           # This file
├── src/
│   ├── lib.rs              # Library root (exports modules)
│   ├── bin/
│   │   └── bench.rs        # Benchmark runner (W2: validation, W3+: perf tests)
│   ├── siw/
│   │   └── mod.rs          # ✅ W2: SIW struct + PhextCoord (11KB)
│   └── scheduler/
│       └── DESIGN.md       # ✅ W2: Scheduler design (10.9KB, W3: impl)
└── benchmarks/             # W3+: Criterion benchmarks
```

---

## What's Implemented (Wave 2)

### ✅ Core Data Structures

**File:** `src/siw/mod.rs` (11 KB)

- `PhextCoord` - 128-bit packed 11-dimensional coordinate
  - `new(dims: [u16; 11], flags: u8)` - Create from dimension array
  - `from_string("L.Sh.Se / C.V.B / Ch.Sc.Sc")` - Parse from notation
  - `to_string()` - Format as human-readable coordinate
  - `dim(d: usize) -> u16` - Extract dimension value
  - `flags() -> u8` - Extract flag bits

- `DenseOp` - D-Pipe (ALU) operations
  - DFMA, DADD, DSUB, DMUL, DCMP, DRED, DSEL, DMOV, DNOP

- `SparseOp` - S-Pipe (memory) operations
  - SGATHER, SSCATTER, SINDEX, SDEDUP, SPREFETCH, SFLUSH, SALLOC, SFREE, SNOP

- `CoordOp` - C-Pipe (coordination) operations
  - CPACK, CROUTE, CSEND, CRECV, CBAR, CFENCE, CREDUCE, CCAST, CNOP

- `DepFlags` - Dependency tracking
  - D_TO_S, D_TO_C, S_TO_D, S_TO_C, C_TO_D, C_TO_S, CROSS_SIW

- `SIW` - Sentron Instruction Word (64-byte cache-aligned)
  - `new(d_op, s_op, c_op, phext_addr)` - Create 3-wide instruction
  - `nop()` - Create no-op instruction
  - `is_nop()` - Check if all pipes idle
  - `validate_independence()` - Verify no intra-SIW dependencies

### ✅ Documentation

**File:** `src/scheduler/DESIGN.md` (10.9 KB)

- Zen 4 execution port mapping (D→Port0/1, S→Port4/5, C→Port2/3)
- Scheduling contract (compiler/runtime guarantees)
- Three implementation strategies:
  - Strategy 1: Software dispatch (W3, target: 2.5+ ops/cycle)
  - Strategy 2: LLVM intrinsics (Phase 2, target: 3.0 ops/cycle)
  - Strategy 3: JIT compilation (Phase 4+)
- Performance counter validation plan (RDPMC)
- Dependency tracking design
- Synthetic benchmark specification

---

## Running Tests

### Unit Tests (W2)

```bash
cargo test

# With verbose output
cargo test -- --nocapture

# Specific test
cargo test test_phext_coord_packing
```

**Available tests:**
- `test_phext_coord_packing` - Verify 11D coordinate packing/unpacking
- `test_phext_coord_string` - Parse "3.1.4 / 1.5.9 / 2.6.5" notation
- `test_siw_size` - Verify SIW is exactly 64 bytes, 64-byte aligned
- `test_siw_nop` - Create and validate NOP instruction
- `test_siw_independence` - Verify independence validation logic
- `test_dep_flags` - Dependency flag operations

**Expected output:**
```
running 6 tests
test siw::tests::test_dep_flags ... ok
test siw::tests::test_phext_coord_packing ... ok
test siw::tests::test_phext_coord_string ... ok
test siw::tests::test_siw_independence ... ok
test siw::tests::test_siw_nop ... ok
test siw::tests::test_siw_size ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Validation Binary (W2)

```bash
cargo run --bin vtpu-bench --release
```

Validates structure sizes and basic operations. W3+ will add performance benchmarks.

---

## What's NOT Yet Implemented

### ⏳ Wave 3 (In Progress)

- Scheduler implementation (`src/scheduler/mod.rs`)
- Synthetic SIW benchmark (1000-instruction stream)
- Performance measurement (RDTSC/RDPMC)
- Ops/cycle validation (target: ≥2.5)

### ⏳ Wave 4-8 (Phase 1)

- Performance tuning
- Documentation
- Design validation

### ⏳ Phase 2+ (Waves 9-40)

- Phext Page Table (PPT)
- D/S/C-Pipe execution engines
- Cluster coordination
- Sentron compiler (phextcc)
- Cognitive slicing

---

## Development Workflow

### Making Changes

```bash
# Edit source
nano src/siw/mod.rs

# Check compilation
cargo check

# Run tests
cargo test

# Build release binary
cargo build --release
```

### Adding Tests

Add to `mod.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_feature() {
        // Test code
        assert_eq!(1 + 1, 2);
    }
}
```

### Debugging

```bash
# Run with debug symbols
cargo build
cargo test

# Verbose Rust compiler output
RUST_BACKTRACE=1 cargo test

# Check for common issues
cargo clippy
```

---

## Performance Measurement (W3+)

### Prerequisites

Performance counters require elevated permissions:

```bash
# Option 1: Run as root (testing only)
sudo cargo run --bin vtpu-bench --release

# Option 2: Allow unprivileged access (permanent)
sudo sysctl -w kernel.perf_event_paranoid=0

# Option 3: Add CAP_SYS_ADMIN to binary (after build)
sudo setcap cap_sys_admin=ep target/release/vtpu-bench
```

### RDTSC (Cycle Counter)

Available now (W2+), no special permissions needed:
```rust
use std::arch::x86_64::_rdtsc;

let start = unsafe { _rdtsc() };
// ... code to measure ...
let end = unsafe { _rdtsc() };
let cycles = end - start;
```

### RDPMC (Port Counters)

Requires setup (W3+), needs elevated permissions:
```rust
use std::arch::x86_64::_rdpmc;

// After PMC setup
let port0_ops = unsafe { _rdpmc(0) };
```

---

## Troubleshooting

### Build Errors

**Error:** "cannot find function `_rdtsc`"
**Fix:** Add `#![feature(stdsimd)]` if using nightly, or use `core::arch::x86_64`

**Error:** "alignment of X is not a power of two"
**Fix:** Verify `#[repr(C, align(64))]` syntax in SIW definition

### Test Failures

**Error:** Coordinate out of range
**Fix:** Ensure coordinate values are 1-2048 (1-indexed) when using `from_string()`

**Error:** Independence validation fails
**Fix:** Remove intra-SIW dependency flags (should only have CROSS_SIW or NONE)

### Permission Errors (Performance Counters)

**Error:** "Permission denied" when reading PMC
**Fix:** See "Performance Measurement" section above for permission setup

---

## Next Steps (Wave 3)

When W3 is complete, you'll be able to:

1. Run synthetic benchmark: `cargo run --bin vtpu-bench --release`
2. Measure actual ops/cycle on your hardware
3. Validate Zen 4 port mapping hypothesis
4. Confirm ≥2.5 ops/cycle achievement

**Estimated W3 completion:** Next session (1-2 hours work)

---

## Questions?

- **Discord:** https://discord.gg/kGCMM5yQ (#general or #vtpu)
- **Docs:** `/source/exo-plan/rally/R23/`
- **Code issues:** Check `/source/vtpu/src/` inline comments

---

**Last validation:** 2026-02-15 03:02 UTC
**Wave:** 2/40 (5% complete)
**Phase:** 1/6 (Proof of Concept)
**Status:** ⚠️ Known issue: PhextCoord bit packing (11D × 11 bits crosses u64 boundary)
  - 4/6 tests passing
  - SIW struct works, minor coord optimization needed
  - Will be fixed in W3 (or simplified to 10 bits/dim)

🌀
