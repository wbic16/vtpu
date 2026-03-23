# R23W24 — Pattern Samples
**Date:** 2026-02-21  
**Agent:** Phex 🔱  
**Purpose:** Concrete input→output examples before implementation

---

## Design Principle

> "Focus on samples, examples, and avoiding unexpected behavior." — Will

Every pattern must have:
1. **Happy path examples** — normal usage
2. **Edge cases** — boundary conditions
3. **Error cases** — what fails and how
4. **SIW output** — exact instruction sequence

---

## Pattern 1: STORE

### Happy Path

**Input:** `store 42 at 1.2.3/4.5.6/7.8.9`

**Output (SIW sequence):**
```
MOV r0, 42
DCOORD 1, 2, 3
SCOORD 4, 5, 6
CCOORD 7, 8, 9
SSCATTR r0
```

**Result:** `OK: stored 42 at 1.2.3/4.5.6/7.8.9`

---

**Input:** `store 0 at 1.1.1/1.1.1/1.1.1`

**Output (SIW sequence):**
```
MOV r0, 0
DCOORD 1, 1, 1
SCOORD 1, 1, 1
CCOORD 1, 1, 1
SSCATTR r0
```

**Result:** `OK: stored 0 at 1.1.1/1.1.1/1.1.1`

---

**Input:** `store -1 at 10.10.10/10.10.10/10.10.10`

**Output (SIW sequence):**
```
MOV r0, -1
DCOORD 10, 10, 10
SCOORD 10, 10, 10
CCOORD 10, 10, 10
SSCATTR r0
```

**Result:** `OK: stored -1 at 10.10.10/10.10.10/10.10.10`

---

### Edge Cases

**Input:** `store 9223372036854775807 at 1.1.1/1.1.1/1.1.1` (i64::MAX)

**Output (SIW sequence):**
```
MOV r0, 9223372036854775807
DCOORD 1, 1, 1
SCOORD 1, 1, 1
CCOORD 1, 1, 1
SSCATTR r0
```

**Result:** `OK: stored 9223372036854775807 at 1.1.1/1.1.1/1.1.1`

---

**Input:** `store -9223372036854775808 at 1.1.1/1.1.1/1.1.1` (i64::MIN)

**Output (SIW sequence):**
```
MOV r0, -9223372036854775808
DCOORD 1, 1, 1
SCOORD 1, 1, 1
CCOORD 1, 1, 1
SSCATTR r0
```

**Result:** `OK: stored -9223372036854775808 at 1.1.1/1.1.1/1.1.1`

---

### Error Cases

**Input:** `store at 1.2.3/4.5.6/7.8.9` (missing value)

**Result:** `ERROR: missing value in store pattern`  
**Expected behavior:** No SIWs generated, clear error message

---

**Input:** `store 42 at` (missing coordinate)

**Result:** `ERROR: missing coordinate after 'at'`  
**Expected behavior:** No SIWs generated

---

**Input:** `store 42 1.2.3/4.5.6/7.8.9` (missing 'at')

**Result:** `ERROR: expected 'at' after value`  
**Expected behavior:** No SIWs generated

---

**Input:** `store 42 at 1.2/4.5.6/7.8.9` (malformed coordinate — only 2 dims in first segment)

**Result:** `ERROR: invalid coordinate '1.2/4.5.6/7.8.9' — expected 3 dimensions per segment`  
**Expected behavior:** No SIWs generated

---

**Input:** `store 42 at 0.1.1/1.1.1/1.1.1` (zero dimension)

**Result:** `ERROR: coordinate dimension must be >= 1, got 0`  
**Expected behavior:** No SIWs generated  
**Rationale:** Phext coordinates are 1-indexed

---

**Input:** `store 42 at 1.2.3/4.5.6` (incomplete coordinate — only 2 segments)

**Result:** `ERROR: incomplete coordinate — expected 3 segments (D/S/C)`  
**Expected behavior:** No SIWs generated

---

**Input:** `store 9223372036854775808 at 1.1.1/1.1.1/1.1.1` (overflow i64::MAX + 1)

**Result:** `ERROR: value overflow — 9223372036854775808 exceeds i64 range`  
**Expected behavior:** No SIWs generated

---

**Input:** `store hello at 1.1.1/1.1.1/1.1.1` (non-numeric value)

**Result:** `ERROR: invalid value 'hello' — expected integer`  
**Expected behavior:** No SIWs generated  
**Note:** String storage is future work (requires string encoding pattern)

---

## Pattern 2: LOAD

### Happy Path

**Input:** `load from 1.2.3/4.5.6/7.8.9`

**Output (SIW sequence):**
```
DCOORD 1, 2, 3
SCOORD 4, 5, 6
CCOORD 7, 8, 9
SGATHER r0
```

**Result:** `OK: r0 = <value at coordinate>`

---

**Input:** `load from 1.1.1/1.1.1/1.1.1`

**Output (SIW sequence):**
```
DCOORD 1, 1, 1
SCOORD 1, 1, 1
CCOORD 1, 1, 1
SGATHER r0
```

**Result:** `OK: r0 = <value at coordinate>`

---

### Edge Cases

**Input:** `load from 1.2.3/4.5.6/7.8.9` (coordinate has no value stored)

**Output (SIW sequence):**
```
DCOORD 1, 2, 3
SCOORD 4, 5, 6
CCOORD 7, 8, 9
SGATHER r0
```

**Result:** `OK: r0 = 0 (empty coordinate)`  
**Note:** Empty coordinates return 0, not error

---

### Error Cases

**Input:** `load from` (missing coordinate)

**Result:** `ERROR: missing coordinate after 'from'`

---

**Input:** `load 1.2.3/4.5.6/7.8.9` (missing 'from')

**Result:** `ERROR: expected 'from' keyword`

---

**Input:** `load from 1.2.3` (incomplete coordinate)

**Result:** `ERROR: incomplete coordinate — expected 3 segments (D/S/C)`

---

## Pattern 3: FIND

### Happy Path

**Input:** `find value > 100 in 1.1.1/1.1.1/1.1.1 to 3.3.3/3.3.3/3.3.3`

**Output (SIW sequence):**
```
# Iterate over coordinate range
DCOORD 1, 1, 1      # Start
# ... iterate all 27 coordinates (3³)
SGATHER r1          # Load value
CMP r1, 100
JLE skip            # Skip if <= 100
SROUTE r2           # Add to result set
skip:
# ... continue iteration
SASSOC              # Finalize result set
```

**Result:** `OK: found N matches`

---

### Edge Cases

**Input:** `find value = 0 in 1.1.1/1.1.1/1.1.1 to 2.2.2/2.2.2/2.2.2`

**Output:** Searches for coordinates where value equals 0  
**Result:** Returns coordinates with explicit 0 OR empty coordinates  
**Question:** Should empty coordinates match `= 0`?  
**Decision needed:** Treat empty as 0, or distinguish?

---

**Input:** `find value != 0 in 1.1.1/1.1.1/1.1.1 to 10.10.10/10.10.10/10.10.10`

**Output:** Returns only coordinates with non-zero values stored  
**Result:** `OK: found N matches`

---

### Error Cases

**Input:** `find in 1.1.1/1.1.1/1.1.1 to 2.2.2/2.2.2/2.2.2` (missing pattern)

**Result:** `ERROR: missing search pattern after 'find'`

---

**Input:** `find value > 100 in 1.1.1/1.1.1/1.1.1` (missing 'to')

**Result:** `ERROR: expected 'to' for range query`  
**Alternative:** Single-coordinate find returns that coordinate's value if it matches

---

**Input:** `find value > 100 in 1.1.1/1.1.1/1.1.1 to 0.1.1/1.1.1/1.1.1` (end < start)

**Result:** `ERROR: invalid range — end coordinate must be >= start`

---

### Supported Comparisons

| Operator | Example | Meaning |
|----------|---------|---------|
| `>` | `value > 100` | Greater than |
| `>=` | `value >= 100` | Greater or equal |
| `<` | `value < 100` | Less than |
| `<=` | `value <= 100` | Less or equal |
| `=` | `value = 100` | Equal |
| `!=` | `value != 100` | Not equal |

---

## Pattern 4: SUM

### Happy Path

**Input:** `sum values in 1.1.1/1.1.1/1.1.1 to 3.3.3/3.3.3/3.3.3`

**Output (SIW sequence):**
```
MOV r0, 0           # Accumulator
# For each of 27 coordinates:
DCOORD x, y, z
SCOORD a, b, c
CCOORD i, j, k
SGATHER r1
ADD r0, r0, r1
# ... repeat
```

**Result:** `OK: sum = <total>`

---

**Input:** `sum values in 1.1.1/1.1.1/1.1.1 to 1.1.1/1.1.1/1.1.1` (single coordinate)

**Output (SIW sequence):**
```
MOV r0, 0
DCOORD 1, 1, 1
SCOORD 1, 1, 1
CCOORD 1, 1, 1
SGATHER r1
ADD r0, r0, r1
```

**Result:** `OK: sum = <value at that coordinate>`

---

### Edge Cases

**Input:** `sum values in 1.1.1/1.1.1/1.1.1 to 10.10.10/10.10.10/10.10.10`

**Result:** Sums 1000 coordinates (10³)  
**Note:** Large ranges are valid but may be slow

---

**Input:** Sum where some coordinates empty

**Result:** Empty coordinates contribute 0 to sum

---

### Error Cases

**Input:** `sum values in 1.1.1/1.1.1/1.1.1` (missing 'to')

**Result:** `ERROR: expected 'to' for range query`

---

**Input:** `sum in 1.1.1/1.1.1/1.1.1 to 2.2.2/2.2.2/2.2.2` (missing 'values')

**Result:** `ERROR: expected 'values' keyword`  
**Alternative:** Allow `sum in ...` as shorthand?

---

## Pattern 5: COUNT

### Happy Path

**Input:** `count entries in 1.1.1/1.1.1/1.1.1 to 3.3.3/3.3.3/3.3.3`

**Output (SIW sequence):**
```
MOV r0, 0           # Counter
# For each of 27 coordinates:
DCOORD x, y, z
SCOORD a, b, c
CCOORD i, j, k
SGATHER r1
# Check if non-empty (non-zero)
CMP r1, 0
JE skip             # Skip if empty
ADD r0, r0, 1
skip:
# ... repeat
```

**Result:** `OK: count = N`

---

### Edge Cases

**Input:** `count entries in 1.1.1/1.1.1/1.1.1 to 10.10.10/10.10.10/10.10.10` (1000 coords)

**Result:** Counts non-empty coordinates in range

---

**Input:** Count in range where all coordinates empty

**Result:** `OK: count = 0`

---

**Input:** Count in range where all coordinates have value 0

**Result:** `OK: count = 0`  
**Rationale:** 0-valued coordinates are semantically "empty" for count purposes  
**Alternative:** `count non-zero in ...` vs `count all in ...` distinction?

---

### Error Cases

**Input:** `count in 1.1.1/1.1.1/1.1.1 to 2.2.2/2.2.2/2.2.2` (missing 'entries')

**Result:** `ERROR: expected 'entries' keyword`

---

## Decision Points

These require explicit decisions before implementation:

### D1: Empty vs Zero
**Question:** Is an empty coordinate the same as a coordinate with value 0?  
**Context:** `find value = 0` — should this match empty coords?  
**Context:** `count entries` — should 0-valued coords count as entries?

**Proposal:** 
- **Empty** = never written to (default state)
- **Zero** = explicitly written with value 0
- For W24: Treat empty as 0 (simpler). Track explicit writes in W25+.

---

### D2: Range Iteration Order
**Question:** What order do we iterate coordinates in a range?

**Options:**
1. **Lexicographic:** D1→D2→D3→S1→S2→S3→C1→C2→C3 (inner to outer)
2. **C-major:** C changes fastest (cache-friendly for C-pipe)
3. **D-major:** D changes fastest (cache-friendly for D-pipe)

**Proposal:** Lexicographic (most intuitive for users). Optimize later if needed.

---

### D3: Output Format
**Question:** How should `find` results be displayed?

**Options:**
1. Just count: `found 17 matches`
2. List coordinates: `found 17 matches: 1.2.3/4.5.6/7.8.9, ...`
3. List with values: `found 17 matches: 1.2.3/4.5.6/7.8.9 → 142, ...`

**Proposal:** Option 3 (most useful). Add `--quiet` flag for count-only.

---

### D4: Overflow Handling
**Question:** What if `sum` overflows i64?

**Options:**
1. Wrap silently (bad)
2. Error on overflow (safe but annoying)
3. Use i128 internally, error if final result > i64 (best)
4. Return overflow flag alongside result

**Proposal:** Option 3 for W24. Consider BigInt in future.

---

## Test Matrix

Each pattern needs tests for:

| Category | Tests per Pattern | Total |
|----------|-------------------|-------|
| Happy path | 2-3 | 10-15 |
| Edge cases | 2-3 | 10-15 |
| Error cases | 3-4 | 15-20 |
| **Total** | | **35-50** |

W24 target: +20 minimum, aiming for 40.

---

## Next Steps

1. **Review decisions D1-D4** — need Will's input
2. **Implement lexer** — tokenize these exact examples
3. **Write tests first** — encode these samples as assertions
4. **Build parser/compiler** — make tests pass

---

*Samples drive implementation. Implementation validates samples.* 🔱
