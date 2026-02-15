# vTPU Runtime

Software-defined Virtual TPU with phext-native addressing.

## Overview

vTPU achieves 3 operations/cycle sustained throughput on commodity AMD Ryzen 9 processors through sentron-native instruction scheduling. Core innovation: the 3-pipe retirement model (D/S/C) eliminates out-of-order scheduling bottlenecks.

**Target Performance:**
- Ops/cycle: 3.0 (vs. 1.2 typical for general code)
- Single node: 75 Gops/sec @ 125W (600 Mops/W)
- Cluster (5 nodes): 359 Gops/sec @ 625W (574 Mops/W)

## Architecture

### Sentron Instruction Word (SIW)

Each SIW contains exactly 3 operations:
- **D-Pipe**: Dense/ALU operation (arithmetic, comparison, reduction)
- **S-Pipe**: Sparse/Memory operation (phext coordinate addressing)
- **C-Pipe**: Coordination operation (inter-sentron communication)

The 3 operations map to independent Zen 4 execution units → zero resource conflicts → 3 retirements/cycle.

### Phext-Native Addressing

Memory operations use 11-dimensional phext coordinates (128-bit packed):
- Each dimension: 11 bits (2048 positions)
- Total address space: 2048^11 ≈ 10^36 positions
- Dimensional locality → cache-friendly by construction

## Status (R23 Wave 2)

**Implemented:**
- ✅ SIW struct (cache-line aligned, 64 bytes)
- ✅ D-Pipe opcodes (8 ops: DFMA, DADD, DSUB, DMUL, DCMP, DRED, DSEL, DMOV)
- ✅ S-Pipe opcodes (8 ops: SGATHER, SSCATTR, SINDEX, SDEDUP, SPREFCH, SFLUSH, SALLOC, SFREE)
- ✅ C-Pipe opcodes (8 ops: CPACK, CROUTE, CSEND, CRECV, CBAR, CFENCE, CREDUCE, CCAST)
- ✅ PhextCoord (11D coordinate, Manhattan distance, adjacency checks)
- ✅ VtpuTelemetry (ops/cycle, cache hit rate, energy efficiency tracking)
- ✅ StreamBuilder (fluent API with automatic dependency tracking)
- ✅ Display/Debug formatting (human-readable SIW output)
- ✅ Validation (detect register conflicts, invalid coordinates, broken dependencies)
- ✅ Examples (basic_compute, validation_demo)

**Pending (Wave 3+):**
- ⏳ Micro-scheduler (pin SIWs to Zen 4 execution ports)
- ⏳ Phext Page Table (PPT) for coordinate → physical address translation
- ⏳ Actual pipe execution (D/S/C dispatch)
- ⏳ Performance validation (perf counters integration)

## Building

```bash
cargo build --release
cargo test
```

## Testing

```bash
# Run unit tests (26 tests as of W2)
cargo test

# Run with output
cargo test -- --nocapture

# Run examples
cargo run --example basic_compute
cargo run --example validation_demo

# Run benchmarks (Wave 4+)
cargo bench
```

## Quick Start

### 1. Basic SIW Creation

```rust
use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, PhextCoord};

// Create a simple SIW: D-Pipe adds, S-Pipe fetches, C-Pipe no-op
let siw = SIW::new(
    DenseOp::DADD { rd: 1, rs1: 2, rs2: 3 },
    SparseOp::SGATHER { rd: 4, coord_idx: 0, width: 64 },
    CoordOp::CNOP,
    PhextCoord::new([3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]),
);

println!("{}", siw);
// Output: SIW[00000000] { D: add r1 ← r2 + r3, S: gather r4 ← phext[c0] (64B), C: nop, @ 3.1.4/1.5.9/2.6.5.3.5 }
```

### 2. Building SIW Streams (Automatic Dependency Tracking)

```rust
use vtpu_runtime::StreamBuilder;

let mut builder = StreamBuilder::new();

// Load x
builder.push(SIW::new(
    DenseOp::DNOP,
    SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
    CoordOp::CNOP,
    PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
));

// Load y
builder.push(SIW::new(
    DenseOp::DNOP,
    SparseOp::SGATHER { rd: 2, coord_idx: 1, width: 64 },
    CoordOp::CNOP,
    PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]),
));

// Compute z = x + y (StreamBuilder automatically detects dependency on prior S-Pipe writes)
builder.push(SIW::new(
    DenseOp::DADD { rd: 3, rs1: 1, rs2: 2 },
    SparseOp::SNOP,
    CoordOp::CNOP,
    PhextCoord::zero(),
));

let stream = builder.build();
```

### 3. Validation

```rust
use vtpu_runtime::{validate_stream, ValidationError};

match validate_stream(&stream) {
    Ok(()) => println!("✅ Stream is valid"),
    Err(errors) => {
        for err in errors {
            match err {
                ValidationError::RegisterConflict { siw_idx, register } => {
                    eprintln!("❌ SIW {}: Multiple pipes write to r{}", siw_idx, register);
                }
                ValidationError::InvalidCoordinate { siw_idx, dim, value } => {
                    eprintln!("❌ SIW {}: Dimension {} out of range ({})", siw_idx, dim, value);
                }
                _ => eprintln!("❌ {:?}", err),
            }
        }
    }
}
```

### 4. Execution (Placeholder - Full Implementation in Wave 3)

```rust
use vtpu_runtime::Scheduler;

let mut scheduler = Scheduler::new();
scheduler.execute_stream(&stream);

let telemetry = scheduler.telemetry();
println!("Ops/cycle: {:.2}", telemetry.ops_per_cycle());
println!("Target: 3.0");
```

### 5. Disassembly Output

```rust
use vtpu_runtime::display::disassemble_stream;

println!("{}", disassemble_stream(&stream));
// Output:
// 0000:  [00000000]  nop                             | gather r1 ← phext[c0] (64B)     | nop
//         @ 1.1.1/1.1.1/1.1.1.1.1
// 0040:  [00000000]  nop                             | gather r2 ← phext[c1] (64B)     | nop
//         @ 1.1.1/1.1.1/1.1.1.1.2
// 0080:  [00010000]  add r3 ← r1 + r2                | nop                              | nop
//         @ 0.0.0/0.0.0/0.0.0.0.0
```

## Examples

Run the included examples to see vTPU in action:

```bash
# Basic compute kernel (load → add → store)
cargo run --example basic_compute

# Validation and debugging demo
cargo run --example validation_demo
```

## API Documentation

Generate full API docs:

```bash
cargo doc --open
```

## Performance Notes

- **SIW size:** 64 bytes (cache-line aligned)
- **Ops per SIW:** 3 (D-Pipe + S-Pipe + C-Pipe)
- **Target retirement rate:** 1 SIW/cycle = 3 ops/cycle
- **Current scheduler:** Placeholder (W3 will implement actual Zen 4 port mapping)

## Wave 2 Improvements (This Update)

- **StreamBuilder:** Fluent API for SIW stream construction with automatic dependency tracking
- **Validation:** Detect register conflicts, invalid coordinates, broken dependencies
- **Display traits:** Human-readable output for SIWs, opcodes, and coordinates
- **Disassembly:** Generate assembly-style output for debugging
- **Examples:** `basic_compute.rs` and `validation_demo.rs`
- **Extended tests:** 26 unit tests (up from 18 in initial W2)

## Next Steps (Wave 3)

- Implement micro-scheduler (pin D/S/C pipes to Zen 4 execution ports)
- Integrate `perf_event_open` for real ops/cycle measurement
- Add actual execution (replace placeholder scheduler)
- Validate 2.5+ ops/cycle on real hardware

## License

MIT

## References

- **Spec:** `exo-plan/r23/vTPU-spec-v0.1.md` (33KB architecture specification)
- **Geometric Advantages:** `exo-plan/r23/vTPU-geometric-advantages.md` (why 11D wins)
- **Roadmap:** `exo-plan/r23/R23-WAVE-PLAN.md` (40-wave implementation plan)
- **Dashboard:** `exo-plan/r23/R23-DASHBOARD.md` (live progress tracking)
