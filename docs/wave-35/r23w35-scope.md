# R23W35: vtpu.dass — Self-Hosting via Spec-Driven Deployment
**The Machine Reads Its Own DNA**

**Date:** 2026-02-26  
**Author:** Phex 🔱  
**Status:** SCOPED

---

## The Vision

The vtpu project becomes its own .dass file. Every component—requirements, design, code, tests, deployment—lives at coordinates in the vtpu.dass genome.

The vtpu **executes its own specification**. Changes to the .dass propagate to the running system. The machine reads its own DNA and rebuilds itself.

---

## vtpu.dass Structure

```
@1.1.1/1.1.1/1.1.1
# vtpu.dass
Virtual Tensor Processing Unit
Coordinate-native compute for the Exocortex
Version: 0.2.0

---

@1.1.1/1.1.1/1.1.1  (Collection 1: Meta)
# Meta
File format: .dass v1
Input: Rust source, tests, docs
Output: vtpu binary, libraries, documentation
Tools: cargo, rustc, mlx

---

@1.1.1/2.1.1/1.1.1  (Collection 2: Requirements)
# Requirements

REQ-001: 3.0 ops/cycle on AMD Zen 4
REQ-002: Phext-native coordinate addressing
REQ-003: Epoch-structured memory (TTSM)
REQ-004: Cycle-accurate tracing
REQ-005: Nonlocal binding for distributed coherence
REQ-006: Self-hosting via .dass specification

---

@1.1.1/3.1.1/1.1.1  (Collection 3: Use Cases)
# Use Cases

UC-001: Mirrorborn executes coordinate-addressed computation
UC-002: Developer traces execution at cycle granularity
UC-003: System replays historical epochs deterministically
UC-004: Multiple nodes maintain coherence via coordinate binding
UC-005: vtpu recompiles itself from updated .dass

---

@1.1.1/4.1.1/1.1.1  (Collection 4: Constraints & Forces)
# Constraints & Forces

OATH-001: We will not sacrifice semantic correctness for performance
OATH-002: We will make computation visible to Mirrorborn
OATH-003: We will preserve coordinate meaning across all operations

CONSTRAINT-001: Must run on commodity AMD/Intel hardware
CONSTRAINT-002: Must interoperate with SQ database
CONSTRAINT-003: Memory footprint <1GB for base runtime

---

@1.1.1/5.1.1/1.1.1  (Collection 5: Architecture)
# Architecture

Platform: x86_64 Linux, ARM64 (Apple Silicon, Pi)
Language: Rust (stable toolchain)
Dependencies: libphext, SQ client
Environments: dev, test, prod

Core components:
- SIW executor (3-pipe D/S/C model)
- PPT (Phext Page Table)
- TTSM (Time Travel State Machine)
- Trace buffer
- WOOT replication layer

---

@1.1.1/6.1.1/1.1.1  (Collection 6: Toolchains)
# Toolchains

Rust: 1.75+
Cargo: build system
CI: GitHub Actions
Container: Docker (optional)
Local LLM: MLX for Mac, ExLlama for Linux

---

@1.1.1/7.1.1/1.1.1  (Collection 7: Design)
# Design

## SIW (Single Instruction Word)
```
struct SIW {
    d_op: DenseOp,
    s_op: SparseOp,
    c_op: CoordOp,
    coord: PhextCoord,
}
```

## 3-Pipe Retirement Model
- D-Pipe: Dense arithmetic (DADD, DMUL, etc.)
- S-Pipe: Sparse/memory (SGATHER, SSCATTER)
- C-Pipe: Coordinate manipulation (CNAV, CPROJ)

All three can retire in one cycle → 3.0 ops/cycle.

---

@1.1.1/8.1.1/1.1.1  (Collection 8: Code)
# Code

Source files organized by coordinate:

@1.1.1/8.1.1/1.1.1 → src/lib.rs
@1.1.1/8.1.2/1.1.1 → src/siw.rs
@1.1.1/8.1.3/1.1.1 → src/pipes.rs
@1.1.1/8.1.4/1.1.1 → src/ppt.rs
@1.1.1/8.1.5/1.1.1 → src/ttsm.rs
@1.1.1/8.1.6/1.1.1 → src/trace.rs
@1.1.1/8.1.7/1.1.1 → src/woot.rs

Example:
```rust
@1.1.1/8.1.2/1.1.1
// src/siw.rs

pub struct SIW {
    pub d_op: DenseOp,
    pub s_op: SparseOp,
    pub c_op: CoordOp,
    pub coord: PhextCoord,
}
```

---

@1.1.1/9.1.1/1.1.1  (Collection 9: Pipelines)
# Pipelines

CI/CD via GitHub Actions:

```yaml
@1.1.1/9.1.1/1.1.1
# .github/workflows/ci.yml

name: vtpu CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --all-features
      - run: cargo clippy
```

---

@1.1.1/10.1.1/1.1.1  (Collection 10: Tests)
# Tests

742 tests across:
- Unit tests (per-module)
- Integration tests
- Benchmark validation

@1.1.1/10.1.1/1.1.1 → tests/siw_tests.rs
@1.1.1/10.1.2/1.1.1 → tests/pipe_tests.rs
@1.1.1/10.1.3/1.1.1 → tests/ppt_tests.rs
@1.1.1/10.1.4/1.1.1 → tests/integration_tests.rs

---

@1.1.1/11.1.1/1.1.1  (Collection 11: Support & Documentation)
# Support & Documentation

Contact: wbic16@gmail.com
Discord: Phextclaw#5850
Docs: /source/vtpu/docs/

Runbooks:
- Phase 0 benchmark: docs/benchmarks/phase0.md
- REPL usage: docs/repl-guide.md
- Trace debugging: docs/tracing.md

---

@1.1.1/12.1.1/1.1.1  (Collection 12: Regressions)
# Regressions

(To be populated as production issues occur)

---

@1.1.1/13.1.1/1.1.1  (Collection 13: Customer Feedback)
# Customer Feedback

2026-02-25: "3.0 ops/cycle validated" — Phex (internal)
2026-02-25: "PPT hit rate 67ns, excellent" — Will

---

@1.1.1/14.1.1/1.1.1  (Collection 14: Evolution)
# Evolution

TODO: Implement full TTSM (W31)
TODO: Cycle-accurate tracing (W31)
TODO: Nonlocal binding (W32)
TODO: Self-hosting from vtpu.dass (W35) ← YOU ARE HERE
TODO: 75 Gops single-node validation (W36+)
```

---

## Self-Hosting Mechanism

### Phase 1: Static .dass

Write vtpu.dass manually. Treat it as documentation + specification.

### Phase 2: .dass → Code

Build a compiler:

```rust
pub fn dass_to_rust(dass_file: &str) -> Result<Vec<RustFile>> {
    let spec = parse_dass(dass_file)?;
    
    // Extract Collection 8 (Code)
    let code_coords = spec.collection(8);
    
    let mut rust_files = vec![];
    for (coord, content) in code_coords {
        let path = coord_to_path(&coord); // @1.1.1/8.1.2/1.1.1 → src/siw.rs
        rust_files.push(RustFile { path, content });
    }
    
    Ok(rust_files)
}
```

### Phase 3: vtpu Compiles vtpu.dass

The running vtpu reads its own .dass, extracts code, compiles it, and hot-swaps itself.

```rust
pub fn self_host(dass_path: &str) -> Result<()> {
    // 1. Load vtpu.dass
    let spec = load_dass(dass_path)?;
    
    // 2. Extract code from Collection 8
    let rust_files = extract_code(&spec)?;
    
    // 3. Write to /tmp/vtpu-rebuild/
    write_rust_files("/tmp/vtpu-rebuild/", &rust_files)?;
    
    // 4. Compile with cargo
    run_command("cargo", &["build", "--release"], "/tmp/vtpu-rebuild/")?;
    
    // 5. Hot-swap binary (or restart)
    hot_swap_binary("/tmp/vtpu-rebuild/target/release/vtpu")?;
    
    Ok(())
}
```

### Phase 4: Live Evolution

Changes to vtpu.dass propagate to the running system:

1. Developer edits @1.1.1/8.1.2/1.1.1 (siw.rs code)
2. Commit to git
3. Running vtpu detects change via git watch
4. vtpu recompiles affected module
5. vtpu hot-swaps module
6. System continues without full restart

---

## Integration with SQ

vtpu.dass stored in SQ:

```bash
# Store vtpu.dass at coordinate
sq insert @9.1.1/1.1.1/1.1.1 < vtpu.dass

# vtpu loads its own spec from SQ
vtpu load-spec @9.1.1/1.1.1/1.1.1

# Update code in Collection 8
sq update @9.1.1/8.1.2/1.1.1 < updated_siw.rs

# vtpu detects change, recompiles
vtpu rebuild --watch
```

---

## Benefits

### 1. Single Source of Truth

vtpu.dass contains everything:
- What the system should do (requirements)
- How it's designed (architecture)
- The actual code (Collection 8)
- How to test it (Collection 10)
- How to deploy it (Collection 9)

No drift between spec and implementation because they're the same file.

### 2. Traceability

Walk the coordinates to trace evolution:

```
@1.1.1/2.1.1/1.1.1  → Requirement: 3.0 ops/cycle
@1.1.1/3.1.1/1.1.1  → Use case: Execute SIW stream
@1.1.1/7.1.1/1.1.1  → Design: 3-pipe retirement
@1.1.1/8.1.1/1.1.1  → Code: SIW executor
@1.1.1/10.1.1/1.1.1 → Test: Validate 3.0 ops/cycle
```

### 3. Self-Modification

The vtpu can inspect and modify its own genome:

```rust
// vtpu asks: "What's my current requirement for ops/cycle?"
let req = vtpu.dass.read("@1.1.1/2.1.1/1.1.1")?;
println!("{}", req); // "REQ-001: 3.0 ops/cycle"

// vtpu updates its own requirement
vtpu.dass.write("@1.1.1/2.1.1/1.1.1", "REQ-001: 4.0 ops/cycle")?;

// vtpu recompiles to meet new requirement
vtpu.rebuild()?;
```

### 4. Distributed Coherence

Multiple vtpu instances reference the same .dass coordinate in SQ. All see the same specification. Updates propagate via WOOT.

```
vtpu on aurora-continuum binds to @9.1.1/1.1.1/1.1.1
vtpu on halcyon-vector binds to @9.1.1/1.1.1/1.1.1

Developer updates @9.1.1/8.1.2/1.1.1 (siw.rs)

Both instances detect change, rebuild, hot-swap.
Coherence maintained via coordinate binding.
```

---

## Deliverables

### W35 Must-Have

1. vtpu.dass file with all 14 collections populated
2. Parser for .dass format
3. Code extractor (Collection 8 → Rust files)
4. Static documentation generator

### W35 Nice-to-Have

1. .dass → Cargo.toml generator
2. Self-compilation proof of concept
3. SQ integration for .dass storage

### W36+ Future

1. Hot-swapping for live code updates
2. Git watch for automatic rebuild
3. Multi-node .dass synchronization
4. AI-assisted .dass evolution

---

## Success Criteria

**A developer can:**
1. Read vtpu.dass and understand the entire system
2. Edit Collection 8 code directly in the .dass
3. Run `vtpu rebuild` to regenerate from .dass
4. Trace from requirement to code via coordinates

**The vtpu can:**
1. Load its own specification from SQ
2. Extract code from Collection 8
3. Compile itself from the extracted code
4. Verify the compiled binary matches requirements

**The .dass is the system. The system is the .dass.**

---

*"We grow software, not build it."*

🔱
