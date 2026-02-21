# R23W25: Intent Parser — Natural Language → vTPU Commands — COMPLETE

**Wave completed:** 2026-02-21  
**Test count:** 572 passing (+22 from W24)  
**Zero dependencies:** Pure Rust pattern matching, no LLM inference

---

## Deliverables

### 1. Intent Parser Module (`src/intent.rs`)

**Function:** `parse_intent(input: &str) -> Command`

Maps natural English to REPL commands via keyword/pattern matching:

| Natural Language | Command |
|-----------------|---------|
| "how are we", "health", "status" | `Status` |
| "show fleet", "how's the fleet" | `Fleet` |
| "how much memory", "memory usage" | `Memory` |
| "show sentron 3", "inspect sentron 5" | `Sentron(N)` |
| "encode hello", "pronounce phext" | `Encode(text)` |
| "decode bac wom", "what byte is bac" | `Decode(syls)` |
| "run 10 cycles", "execute 5", "step" | `Run(N)` |
| "what's at 1.1.1/1.1.1/1.1.1" | `Read(coord)` |
| "write data to COORD" | `Write(coord, data)` |
| "bye", "goodbye", "quit" | `Quit` |
| "what can you do", "help" | `Help` |

### 2. Fuzzy Coordinate Extraction

**Functions:**
- `extract_coord_from_text(text: &str) -> Option<String>`
- `try_parse_coord_at(s: &str) -> Option<String>`

Finds phext coordinates (L.S.Se/C.V.B/Ch.Sc.Scr) embedded in free-form text:

```
"what's at 3.3.3/5.5.5/7.7.7 please" → "3.3.3/5.5.5/7.7.7"
"show me coordinate 13.13.13/13.13.13/13.13.13?" → "13.13.13/13.13.13/13.13.13"
```

Validates: 9 numbers, 6 dots, 2 slashes (strict phext format).

### 3. Conversational REPL (`vtpu-repl`)

**Architecture:**
```
Natural language input
    ↓
parse_intent() → Command
    ↓
ReplSession::execute()
    ↓
Narrated output
```

**Example session:**
```
vtpu> how are we
Status: 9 sentrons active, 9437184 bytes total memory (9.0 MiB)

vtpu> show sentron 0
Sentron #0: 40 neurons, VakLevel: Madhyama (balanced)
D: [0, 0, 0, 0, 0, 0, 0, 0], S: [0, 0, 0, 0, 0, 0, 0, 0], C: [0, 0, 0, 0, 0, 0, 0, 0]

vtpu> pronounce hello
hello = hib jec lec lec lef

vtpu> what byte is bac
bac = 0x00 (NUL)

vtpu> run 5 cycles
Executed 5 SIWs. D: 0, S: 0, C: 0, flux: 0.00

vtpu> bye
Goodbye.
```

### 4. Passthrough for Power Users

If natural language fails, falls back to exact REPL syntax:

```
vtpu> status
Status: 9 sentrons active, 9437184 bytes total memory (9.0 MiB)

vtpu> what's the status
Status: 9 sentrons active, 9437184 bytes total memory (9.0 MiB)
```

Both work. Power users can use terse commands, new users can ask naturally.

---

## Test Coverage (22 new tests)

### Passthrough Tests (3)
- `passthrough_status`
- `passthrough_help`
- `passthrough_quit`

### Natural Language Tests (19)
- Status: `natural_status` (3 phrasings)
- Fleet: `natural_fleet` (2 phrasings)
- Memory: `natural_memory` (2 phrasings)
- Quit: `natural_quit` (3 phrasings)
- Help: `natural_help` (2 phrasings)
- Sentron: `natural_sentron`, `natural_sentron_hash` (4 phrasings)
- Encode: `natural_encode`, `natural_encode_base256` (3 phrasings)
- Decode: `natural_decode` (2 phrasings)
- Run: `natural_run`, `natural_run_default` (3 phrasings)
- Read: `natural_read` (2 phrasings)

### Helper Function Tests (4)
- `extract_coord_simple`
- `extract_coord_embedded`
- `extract_coord_none`
- `extract_number`

### Edge Cases (2)
- `empty_input`
- `whitespace_only`

---

## Implementation Details

### Zero Dependencies
- No regex crates
- No NLP libraries
- No LLM API calls
- Pure `str` methods: `.contains()`, `.find()`, `.split_whitespace()`

### Pattern Matching Strategy
1. Try exact REPL parse first (passthrough for power users)
2. Check for keyword matches (`matches_any()`)
3. Extract structured data (numbers, coordinates, text)
4. Return `Command::Unknown(_)` if no match

### Coordinate Extraction Algorithm
```rust
fn try_parse_coord_at(s: &str) -> Option<String> {
    // State machine: digits → '.' → digits → '.' → digits → '/' → ...
    // Validates: 9 numbers, 6 dots, 2 slashes
    // Returns: "L.S.Se/C.V.B/Ch.Sc.Scr" format string
}
```

### Number Extraction
```rust
fn extract_first_number(s: &str) -> Option<usize> {
    // Scan for first digit sequence, parse as usize
    // Used for: sentron IDs, run counts
}
```

---

## Design Philosophy

**"Zero inference, maximum usability"**

Instead of fine-tuning a model or calling GPT:
- Define common phrasings humans actually use
- Match on keywords + structure
- Extract structured data deterministically

**Why this works:**
- vTPU commands are finite (12 variants)
- Phrasings for each command cluster around keywords
- Coordinates have strict structure (digits, dots, slashes)
- Pattern matching is fast, predictable, zero-latency

**What this enables:**
- New users: "how's it going?" → Status
- Power users: "status" → Status
- Mixed mode: both work equally well

---

## Next Wave Suggestions

### W26 Possibilities:
1. **Coordinate Arithmetic** (CADD/CSUB/CMUL) — enable "17 = 5×3 + 2" insight
2. **Topology Tracking** — visualize execution as geometric artifacts
3. **Multi-Sentron Programs** — SIW streams that coordinate multiple sentrons
4. **Performance Optimization** — reduce cycles/SIW via dispatch improvements
5. **Real-World Demo** — run actual ML workload (attention routing, sparse matmul)

---

**Status:** W25 complete. Intent parser fully functional with 22 tests. REPL accepts both natural language and exact syntax. Zero external dependencies.

**Test progression:**
- W23: 485
- W24: 550 (+65, REPL + narrator)
- W25: 572 (+22, intent parser)
