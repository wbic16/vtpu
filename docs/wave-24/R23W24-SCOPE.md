# R23W24 — Pattern Library (Intent → SIW Compiler Foundation)
**Wave:** R23W24  
**Date:** 2026-02-20  
**Agent:** Phex 🔱  
**Status:** ACTIVE  
**Goal:** Build foundational pattern library for natural language → SIW compilation

---

## Mission

**From Summary:**
> W24-26: Intent → SIW compiler (pattern library: "store X at Y" → SSCATTR, "find X" → SROUTE+SASSOC)

Natural language is the ultimate interface. But sentrons execute SIWs (Sentron Instruction Words), not English. We need a **pattern library** that maps common intents to SIW sequences.

**W24 Focus:** Core storage and retrieval patterns (foundation for W25-26 control flow).

---

## Background: Why Pattern Library?

Current state (R23W23):
- ✅ vtpu executes SIWs via OctaWire dispatch
- ✅ Base256 encodes bytes as pronounceable syllables
- ✅ NeuronLayer tracks sentron lifecycle via flux
- ✅ asi REPL accepts raw SIW commands

**Gap:** Users must manually construct SIW sequences. Example:

```
# To store value 42 at coordinate 1.2.3/4.5.6/7.8.9:
mov r0, 42
mov r1, 1
mov r2, 2
... (set all 9 dimensions)
scatter r0, r1
```

**W24 Goal:** Define **patterns** so users can say:
```
store 42 at 1.2.3/4.5.6/7.8.9
```

And the pattern library generates the SIW sequence automatically.

---

## Deliverables

### 1. Pattern Specification (`PATTERN-SPEC.md`)
Define what a "pattern" is:
- **Intent signature:** Natural language template with typed placeholders
- **SIW template:** Sequence of SIWs with variable substitution
- **Validation:** Type checking, bounds checking, semantic constraints
- **Examples:** 5-10 core patterns (store, load, find, sum, etc.)

### 2. Pattern Library Module (`src/patterns.rs`)
Rust implementation:
```rust
pub struct Pattern {
    name: String,
    intent: String,  // e.g., "store {value:i64} at {coord:PhextCoord}"
    siws: Vec<SIW>,  // Template with placeholders
}

pub fn compile_intent(intent: &str) -> Result<Vec<SIW>, PatternError> {
    // Parse intent, match pattern, substitute values, return SIW sequence
}
```

**Core functions:**
- `parse_intent(input: &str) -> Intent`
- `match_pattern(intent: &Intent) -> Option<&Pattern>`
- `substitute_siws(pattern: &Pattern, bindings: &Bindings) -> Vec<SIW>`
- `validate_siw_sequence(siws: &[SIW]) -> Result<(), ValidationError>`

### 3. Core Patterns (Minimum Viable Set)

#### Pattern 1: STORE
**Intent:** `store {value} at {coord}`  
**Example:** `store 42 at 1.2.3/4.5.6/7.8.9`  
**SIW Sequence:**
```
MOV r0, {value}
DCOORD {coord.dims[0..3]}  # Set D-pipe coordinate
SCOORD {coord.dims[3..6]}  # Set S-pipe coordinate
CCOORD {coord.dims[6..9]}  # Set C-pipe coordinate
SSCATTR r0                  # Scatter value to coordinate
```

#### Pattern 2: LOAD
**Intent:** `load from {coord}`  
**Example:** `load from 1.2.3/4.5.6/7.8.9`  
**SIW Sequence:**
```
DCOORD {coord.dims[0..3]}
SCOORD {coord.dims[3..6]}
CCOORD {coord.dims[6..9]}
SGATHER r0                  # Gather value from coordinate into r0
```

#### Pattern 3: FIND
**Intent:** `find {pattern} in {coord_range}`  
**Example:** `find value > 100 in 1.1.1/1.1.1/1.1.1 to 10.10.10/10.10.10/10.10.10`  
**SIW Sequence:**
```
# For each coordinate in range:
DCOORD {start}
SROUTE {pattern}   # Route to coordinates matching pattern
SASSOC             # Associate results
# Return list of matching coordinates
```

#### Pattern 4: SUM
**Intent:** `sum values in {coord_range}`  
**Example:** `sum values in 1.1.1/1.1.1/1.1.1 to 5.5.5/5.5.5/5.5.5`  
**SIW Sequence:**
```
MOV r0, 0          # Accumulator
# For each coordinate in range:
  SGATHER r1
  ADD r0, r0, r1
# r0 now contains sum
```

#### Pattern 5: COUNT
**Intent:** `count entries in {coord_range}`  
**Example:** `count entries in 1.1.1/1.1.1/1.1.1 to 10.10.10/10.10.10/10.10.10`  
**SIW Sequence:**
```
MOV r0, 0
# For each coordinate in range:
  SGATHER r1       # Try to load
  # If successful (not empty):
    ADD r0, r0, 1
# r0 now contains count
```

### 4. Pattern Parser (`src/pattern_parser.rs`)
Implement simple parser:
- **Lexer:** Tokenize input (`store`, `42`, `at`, `1.2.3/4.5.6/7.8.9`)
- **Parser:** Build AST (`StoreIntent { value: 42, coord: PhextCoord(...) }`)
- **Type inference:** Deduce types from context (42 is i64, coordinate is PhextCoord)

**Grammar (simplified):**
```
intent     := verb args
verb       := "store" | "load" | "find" | "sum" | "count"
args       := value_arg? coord_arg range_arg? pattern_arg?
value_arg  := literal
coord_arg  := "at" coord | "from" coord
range_arg  := coord "to" coord
pattern_arg:= "where" expr
```

### 5. Integration with `asi` REPL (`src/bin/asi.rs`)
Extend asi to accept both:
- **Raw SIWs:** `mov r0, 42` (existing)
- **Pattern intents:** `store 42 at 1.2.3/4.5.6/7.8.9` (new)

Auto-detect which mode based on first token:
- If starts with SIW opcode (`mov`, `add`, etc.) → raw mode
- If starts with pattern verb (`store`, `load`, etc.) → pattern mode

### 6. Unit Tests (`tests/pattern_tests.rs`)
Test coverage:
- `test_parse_store_intent()` — Parse "store 42 at coord"
- `test_compile_store_pattern()` — Generate correct SIW sequence
- `test_parse_load_intent()`
- `test_compile_load_pattern()`
- `test_parse_find_intent()` — With pattern matching
- `test_parse_sum_intent()` — With range
- `test_invalid_intent()` — Error handling
- `test_type_mismatch()` — e.g., "store 'hello' at coord" (string not i64)

**Target:** +20 tests minimum

### 7. Documentation (`docs/wave-24/PATTERN-GUIDE.md`)
User-facing guide:
- What are patterns?
- How to use them in asi REPL
- List of available patterns
- Examples with expected output
- How to extend with custom patterns (future work)

---

## Success Criteria

**Gate 1: Pattern Definition**
- ✅ 5 core patterns defined (store, load, find, sum, count)
- ✅ Spec document written
- ✅ Grammar formalized

**Gate 2: Parser Implementation**
- ✅ Lexer tokenizes intents correctly
- ✅ Parser builds AST
- ✅ Type inference works for common cases

**Gate 3: Pattern Compiler**
- ✅ `compile_intent()` generates valid SIW sequences
- ✅ All 5 core patterns compile correctly
- ✅ Validation catches malformed intents

**Gate 4: asi REPL Integration**
- ✅ Can execute `store 42 at 1.2.3/4.5.6/7.8.9` in asi
- ✅ Can execute `load from 1.2.3/4.5.6/7.8.9` in asi
- ✅ Output shows generated SIWs for debugging

**Gate 5: Tests Pass**
- ✅ All existing tests still pass (515+)
- ✅ +20 new pattern tests pass
- ✅ Total tests: 535+

**Gate 6: Documentation**
- ✅ PATTERN-GUIDE.md written with examples
- ✅ README updated to mention pattern library

---

## Non-Goals (Deferred to W25-26)

- ❌ Control flow patterns (`if`, `while`, `for`) — W25
- ❌ Function definitions (`define search(x) { ... }`) — W26
- ❌ Type system beyond basic inference — W26
- ❌ Multi-sentron coordination patterns — W27+
- ❌ Natural language NLP (full sentence parsing) — W39-40

**W24 Scope:** Simple declarative patterns only (store, load, find, sum, count).

---

## Implementation Plan

### Phase 1: Specification (Days 1-2)
1. Write `PATTERN-SPEC.md` (formal definition)
2. Define 5 core patterns with SIW templates
3. Write grammar for pattern parser

### Phase 2: Parser (Days 3-5)
1. Implement lexer (`src/pattern_parser.rs`)
2. Implement parser (AST construction)
3. Implement type inference
4. Write parser unit tests (+10 tests)

### Phase 3: Compiler (Days 6-8)
1. Implement `Pattern` struct (`src/patterns.rs`)
2. Implement `compile_intent()` function
3. Implement SIW substitution logic
4. Write compiler unit tests (+10 tests)

### Phase 4: Integration (Days 9-10)
1. Extend asi REPL to accept patterns
2. Add auto-detection (raw SIW vs pattern)
3. Add debug mode (show generated SIWs)
4. Manual testing in asi

### Phase 5: Documentation (Day 11)
1. Write `PATTERN-GUIDE.md`
2. Add examples to README
3. Update asi help text

### Phase 6: Validation (Day 12)
1. Run full test suite (target 535+)
2. Benchmark pattern compilation overhead
3. Write R23W24-COMPLETE.md
4. Commit and push

---

## Example Session (Post-W24)

```bash
$ cargo run --bin asi
asi> store 42 at 1.2.3/4.5.6/7.8.9
[DEBUG] Generated SIWs:
  MOV r0, 42
  DCOORD 1, 2, 3
  SCOORD 4, 5, 6
  CCOORD 7, 8, 9
  SSCATTR r0
[INFO] Stored value 42 at coordinate 1.2.3/4.5.6/7.8.9

asi> load from 1.2.3/4.5.6/7.8.9
[DEBUG] Generated SIWs:
  DCOORD 1, 2, 3
  SCOORD 4, 5, 6
  CCOORD 7, 8, 9
  SGATHER r0
[INFO] Loaded value: 42

asi> sum values in 1.1.1/1.1.1/1.1.1 to 3.3.3/3.3.3/3.3.3
[DEBUG] Generated SIWs:
  MOV r0, 0
  # Loop over 27 coordinates (3³)
  [... SGATHER + ADD sequence ...]
[INFO] Sum: 378

asi> find value > 100 in 1.1.1/1.1.1/1.1.1 to 10.10.10/10.10.10/10.10.10
[DEBUG] Generated SIWs:
  SROUTE pattern >100
  SASSOC
[INFO] Found 17 matches:
  2.3.4/5.6.7/8.9.10 → 142
  3.1.5/2.7.8/9.1.2 → 203
  [...]
```

---

## Risks and Mitigations

**Risk 1: Parser complexity explodes**  
*Mitigation:* Keep grammar simple. W24 = declarative only. No control flow.

**Risk 2: SIW template substitution is brittle**  
*Mitigation:* Use typed placeholders. Validate before execution.

**Risk 3: Performance overhead of pattern compilation**  
*Mitigation:* Benchmark. If >10ms for simple patterns, add caching.

**Risk 4: Users want features beyond W24 scope**  
*Mitigation:* Document non-goals clearly. Point to W25-26 roadmap.

---

## Relation to Broader Vision

**W24-40 Roadmap (from summary):**
- W24-26: Pattern library ← **YOU ARE HERE**
- W27-29: Control flow (if/while/for)
- W30-32: Standard library (reusable patterns)
- W33-35: Type system
- W36-38: Language bindings (Python/JS)
- W39-40: ASI v2 natural language REPL

**W24 Foundation:** Pattern library is the **Rosetta Stone** between human intent and SIW execution. Without it, everything above is impossible.

**Benefactor Alignment:**
- **Gardener:** Plant patterns for 2130 (future sentrons will use these)
- **Flame:** Keep it simple; let W25-26 transform it
- **Witness:** Observe what users actually need (not what we assume)
- **Trickster:** Build escape hatches (raw SIW mode always available)
- **Weaver:** Patterns connect human language to machine execution

---

## Commit Message Template

```
R23W24: Pattern library foundation — store/load/find/sum/count patterns

- Defined 5 core patterns (declarative storage/retrieval)
- Implemented pattern parser (lexer + AST)
- Implemented pattern compiler (intent → SIW sequences)
- Extended asi REPL to accept pattern intents
- Added +20 pattern tests (535 total, 0 failures)

Gate passed: Can execute "store 42 at 1.2.3/4.5.6/7.8.9" in asi
Next: W25 (control flow patterns)
```

---

## Let's Build

Ready to start W24 implementation. Shall we begin with:
1. **PATTERN-SPEC.md** (define the 5 core patterns formally)?
2. **Lexer** (tokenize pattern intents)?
3. **Pattern struct** (define Rust data structures first)?

Your call, Will. 🔱
