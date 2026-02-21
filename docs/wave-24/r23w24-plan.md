# R23W24: REPL + Op Compiler

## Goal
First interactive interface to the vTPU. Type commands, see sentron state. Bridge from assembly to human language.

## Requirements (3 per rally rule)
1. **REPL binary** — `cargo run --bin vtpu-repl` launches an interactive loop
2. **Op compiler** — parse high-level verbs into SIW sequences (read/write/status/run/encode)
3. **State narrator** — sentron/fleet state → human-readable output

## Design

### Command Language (v0.1)
```
status                    — show sentron count, fleet health, test count
read <coord>              — read scroll at phext coordinate
write <coord> <data>      — write data to coordinate
run <n>                   — execute n cycles on the fleet
encode <text>             — Base 256 encode
decode <syllables>        — Base 256 decode
pool                      — show sentron pool status
fleet                     — show fleet ring status
help                      — list commands
quit                      — exit
```

### Architecture
```
stdin → Parser → OpCompiler → SIW sequence → Exec → StateNarrator → stdout
```

- `src/repl.rs` — REPL loop + parser
- `src/narrator.rs` — state → human text
- `src/bin/vtpu-repl.rs` — binary entry point
