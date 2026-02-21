# R23W26: Coordinate Arithmetic — Base 256 Powers

**Wave:** R23W26  
**Date:** 2026-02-21  
**Agent:** Lux 🔆  
**Goal:** Enable coordinate computation beyond base-13 addressing via arithmetic operations

---

## Mission

**The Insight:** `255 = 3 × 5 × 17 = 3 × 5 × (5×3 + (5-3))`

Phext coordinates are currently limited to base-13 (1-13 per dimension). But coordinate **arithmetic** enables access to infinite space. If we can compute coordinate 17 from coordinates 3 and 5, we break the addressing boundary.

**Key Principle:** *Coordinates are computable, not just addressable.*

---

## Background

### Current State (W25)
- ✅ 9 phext dimensions (Library through Scroll)
- ✅ Base-13 addressing (1-13 per dimension)
- ✅ SGATHER/SSCATTR read/write to coordinates
- ✅ Each SIW carries its phext_addr (per-instruction addressing)
- ❌ **No coordinate arithmetic** — can't compute new coordinates from existing ones

### The Problem
To access coordinate `(17, 17, 17, ...)`:
- **Current:** Impossible (17 > 13, out of range)
- **Needed:** Compute it: `17 = 5×3 + (5-3)`

### The Solution: Coordinate Algebra
Add 4 new C-pipe operations:
1. **CADD** — Add two coordinates
2. **CSUB** — Subtract two coordinates
3. **CMUL** — Multiply two coordinates (element-wise)
4. **CSCALE** — Multiply coordinate by scalar

---

## Deliverables

### 1. Coordinate Arithmetic Opcodes

**Extend `CoordOp` enum in `src/pipes.rs`:**

```rust
pub enum CoordOp {
    // Existing
    CNOP,
    CBAR,
    CSEND { dest_sentron: u8, msg: u32 },
    CRECV { inbox_idx: u8 },
    
    // NEW: Coordinate Algebra (W26)
    CADD { rd: u8, rs1: u8, rs2: u8 },
    CSUB { rd: u8, rs1: u8, rs2: u8 },
    CMUL { rd: u8, rs1: u8, rs2: u8 },
    CSCALE { rd: u8, rs: u8, scalar: u8 },
}
```

**Semantics:**
- `CADD rd, rs1, rs2` → `coord[rd] = coord[rs1] + coord[rs2]` (element-wise)
- `CSUB rd, rs1, rs2` → `coord[rd] = coord[rs1] - coord[rs2]` (element-wise)
- `CMUL rd, rs1, rs2` → `coord[rd] = coord[rs1] * coord[rs2]` (element-wise)
- `CSCALE rd, rs, scalar` → `coord[rd] = coord[rs] * scalar` (broadcast)

**Element-wise example (CADD):**
```
coord[3] = (3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3)
coord[5] = (5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5)
CADD 8, 3, 5
→ coord[8] = (8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8)
```

### 2. Execution Implementation

**Update `src/exec.rs::exec_c_pipe()`:**

```rust
fn exec_c_pipe(sentron: &mut Sentron, op: &CoordOp, stats: &mut ExecStats) {
    match op {
        CoordOp::CNOP => { /* existing */ }
        CoordOp::CBAR => { /* existing */ }
        CoordOp::CSEND { .. } => { /* existing */ }
        CoordOp::CRECV { .. } => { /* existing */ }
        
        // NEW: Coordinate arithmetic
        CoordOp::CADD { rd, rs1, rs2 } => {
            let c1 = sentron.coord_regs[*rs1 as usize];
            let c2 = sentron.coord_regs[*rs2 as usize];
            sentron.coord_regs[*rd as usize] = coord_add(&c1, &c2);
            stats.c_pipe_ops += 1;
        }
        
        CoordOp::CSUB { rd, rs1, rs2 } => {
            let c1 = sentron.coord_regs[*rs1 as usize];
            let c2 = sentron.coord_regs[*rs2 as usize];
            sentron.coord_regs[*rd as usize] = coord_sub(&c1, &c2);
            stats.c_pipe_ops += 1;
        }
        
        CoordOp::CMUL { rd, rs1, rs2 } => {
            let c1 = sentron.coord_regs[*rs1 as usize];
            let c2 = sentron.coord_regs[*rs2 as usize];
            sentron.coord_regs[*rd as usize] = coord_mul(&c1, &c2);
            stats.c_pipe_ops += 1;
        }
        
        CoordOp::CSCALE { rd, rs, scalar } => {
            let c = sentron.coord_regs[*rs as usize];
            sentron.coord_regs[*rd as usize] = coord_scale(&c, *scalar as i32);
            stats.c_pipe_ops += 1;
        }
    }
}
```

### 3. Arithmetic Helper Functions

**Add to `src/phext_coord.rs`:**

```rust
impl PhextCoord {
    /// Element-wise addition (saturating at i32 bounds)
    pub fn add(&self, other: &PhextCoord) -> PhextCoord {
        let mut result = PhextCoord::zero();
        for i in 0..11 {
            result.dims[i] = self.dims[i].saturating_add(other.dims[i]);
        }
        result
    }
    
    /// Element-wise subtraction (saturating at 0)
    pub fn sub(&self, other: &PhextCoord) -> PhextCoord {
        let mut result = PhextCoord::zero();
        for i in 0..11 {
            result.dims[i] = self.dims[i].saturating_sub(other.dims[i]);
        }
        result
    }
    
    /// Element-wise multiplication (saturating)
    pub fn mul(&self, other: &PhextCoord) -> PhextCoord {
        let mut result = PhextCoord::zero();
        for i in 0..11 {
            result.dims[i] = self.dims[i].saturating_mul(other.dims[i]);
        }
        result
    }
    
    /// Scalar multiplication (broadcast)
    pub fn scale(&self, scalar: i32) -> PhextCoord {
        let mut result = PhextCoord::zero();
        for i in 0..11 {
            result.dims[i] = self.dims[i].saturating_mul(scalar);
        }
        result
    }
}
```

**Why saturating arithmetic?**
- Prevents overflow panics
- Graceful degradation (clip to i32::MAX instead of wrapping)
- Users can detect saturation by checking result bounds

### 4. Unit Tests

**Add to `src/tests/coord_arithmetic.rs` (new file):**

```rust
#[test]
fn test_cadd_basic() {
    let c1 = PhextCoord::uniform(3);
    let c2 = PhextCoord::uniform(5);
    let result = c1.add(&c2);
    assert_eq!(result, PhextCoord::uniform(8));
}

#[test]
fn test_csub_basic() {
    let c1 = PhextCoord::uniform(10);
    let c2 = PhextCoord::uniform(3);
    let result = c1.sub(&c2);
    assert_eq!(result, PhextCoord::uniform(7));
}

#[test]
fn test_cmul_basic() {
    let c1 = PhextCoord::uniform(3);
    let c2 = PhextCoord::uniform(5);
    let result = c1.mul(&c2);
    assert_eq!(result, PhextCoord::uniform(15));
}

#[test]
fn test_cscale_basic() {
    let c = PhextCoord::uniform(7);
    let result = c.scale(3);
    assert_eq!(result, PhextCoord::uniform(21));
}

#[test]
fn test_compute_17_from_3_and_5() {
    // The core insight: 17 = 5×3 + (5-3)
    let c3 = PhextCoord::uniform(3);
    let c5 = PhextCoord::uniform(5);
    
    let product = c3.mul(&c5);           // (15, 15, ...)
    let diff = c5.sub(&c3);              // (2, 2, ...)
    let c17 = product.add(&diff);        // (17, 17, ...)
    
    assert_eq!(c17, PhextCoord::uniform(17));
}

#[test]
fn test_csub_underflow() {
    let c1 = PhextCoord::uniform(3);
    let c2 = PhextCoord::uniform(10);
    let result = c1.sub(&c2);
    assert_eq!(result, PhextCoord::uniform(0));  // Saturates to 0
}

#[test]
fn test_cmul_overflow() {
    let c1 = PhextCoord::uniform(i32::MAX / 2);
    let c2 = PhextCoord::uniform(3);
    let result = c1.mul(&c2);
    assert_eq!(result, PhextCoord::uniform(i32::MAX));  // Saturates
}

#[test]
fn test_cadd_mixed_dimensions() {
    let c1 = PhextCoord::from([1,2,3,4,5,6,7,8,9,10,11]);
    let c2 = PhextCoord::from([10,20,30,40,50,60,70,80,90,100,110]);
    let result = c1.add(&c2);
    assert_eq!(result, PhextCoord::from([11,22,33,44,55,66,77,88,99,110,121]));
}
```

**Target:** +10-12 new tests

### 5. REPL Integration

**Update `src/repl.rs` to support coordinate arithmetic commands:**

```rust
pub enum Command {
    // Existing commands...
    
    // NEW: Coordinate arithmetic
    CoordAdd { rd: u8, rs1: u8, rs2: u8 },
    CoordSub { rd: u8, rs1: u8, rs2: u8 },
    CoordMul { rd: u8, rs1: u8, rs2: u8 },
    CoordScale { rd: u8, rs: u8, scalar: i32 },
}
```

**Parser updates:**
```
"cadd r3, r1, r2" → CoordAdd { rd: 3, rs1: 1, rs2: 2 }
"csub r8, r5, r3" → CoordSub { rd: 8, rs1: 5, rs2: 3 }
"cmul r7, r3, r5" → CoordMul { rd: 7, rs1: 3, rs2: 5 }
"cscale r9, r7, 2" → CoordScale { rd: 9, rs: 7, scalar: 2 }
```

**Example session:**
```
vtpu> # Load base coordinates
vtpu> sindex r3 3.3.3/3.3.3/3.3.3
Coordinate r3 = (3,3,3,3,3,3,3,3,3,3,3)

vtpu> sindex r5 5.5.5/5.5.5/5.5.5
Coordinate r5 = (5,5,5,5,5,5,5,5,5,5,5)

vtpu> # Compute coordinate 17
vtpu> cmul r7, r3, r5
Coordinate r7 = (15,15,15,15,15,15,15,15,15,15,15)

vtpu> csub r8, r5, r3
Coordinate r8 = (2,2,2,2,2,2,2,2,2,2,2)

vtpu> cadd r17, r7, r8
Coordinate r17 = (17,17,17,17,17,17,17,17,17,17,17)

vtpu> # Now we can access coordinate 17!
vtpu> sgather r17
Loaded from (17,17,17,17,17,17,17,17,17,17,17)
```

### 6. Intent Parser Integration

**Extend `src/intent.rs` for natural language:**

```rust
// "add coordinates r1 and r2, store in r3"
// "multiply coordinate r3 by 5"
// "compute coordinate 17 from 3 and 5"
```

Deferred to W27 (natural language for arithmetic).

### 7. Documentation

**Create `docs/wave-26/COORD-ALGEBRA.md`:**
- Why coordinate arithmetic matters
- Mathematical properties (associativity, commutativity, identity)
- Relationship to phext addressing (break the base-13 barrier)
- Examples: Computing coordinates beyond addressing limits
- Use cases: Coordinate compression, navigation algorithms

---

## Success Criteria

**Gate 1: Implementation**
- ✅ CADD/CSUB/CMUL/CSCALE opcodes defined
- ✅ exec_c_pipe() implements all 4 operations
- ✅ PhextCoord helper functions written

**Gate 2: Tests**
- ✅ +10 coord arithmetic tests pass
- ✅ All existing tests still pass (572+)
- ✅ Total: 582+ tests

**Gate 3: REPL Integration**
- ✅ Can execute `cmul r7, r3, r5` in vtpu-repl
- ✅ Can execute full "compute 17 from 3 and 5" sequence
- ✅ Narrator describes coord register state

**Gate 4: Proof of Concept**
- ✅ Demonstrate accessing coordinate (17,17,...) via arithmetic
- ✅ Demonstrate coordinate (255,255,...) via arithmetic
- ✅ Show that any coordinate is reachable via computation

---

## Impact

### Immediate
- **Infinite coordinate space** — no longer limited to base-13
- **Coordinate compression** — store small primes, compute large coords
- **Navigation algorithms** — A* search, space-filling curves

### Future Waves
- **W27:** Coordinate algebra in SQ (distributed coord computation)
- **W28:** Topology tracking (visualize coord arithmetic paths)
- **W29:** Coordinate-based ML (attention routing via coords)
- **W30:** Phext Index (k-d tree with coord arithmetic for range queries)

### Philosophical
The lattice isn't a fixed grid. It's a **generative space**. Coordinates aren't addresses—they're **mathematical objects** that can be computed, transformed, and composed.

This is the door the Benefactors opened with "255 = 3×5×17".

---

## Timeline

**Day 1 (Today):** Plan + PhextCoord helpers + basic tests  
**Day 2:** exec_c_pipe() implementation + full test suite  
**Day 3:** REPL integration + manual testing  
**Day 4:** Documentation + completion doc + push

**Total:** 3-4 days

---

## Next Wave (W27)

Possibilities:
1. **Topology Tracking** — visualize coordinate arithmetic as geometric paths
2. **SIW Patterns** (deferred from original W24 scope) — intent → SIW compiler
3. **Multi-Sentron Coordination** — CSEND/CRECV with arithmetic routing
4. **Natural Language Algebra** — "compute 17 from 3 and 5" in intent parser

---

**Status:** W26 ready to implement. Starting with PhextCoord helper functions.
