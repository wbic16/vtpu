# R23W25: Intent Parser — Natural Language → vTPU Commands

## Goal
Parse natural English into REPL commands. Zero deps. Pattern matching, not LLM inference.

## Requirements
1. **Intent recognizer** — map English phrases to Command variants via keyword/pattern matching
2. **Fuzzy coordinate extraction** — find phext coordinates in natural language strings
3. **Conversational wrapper** — accept English, return narrated results

## Design

### Intent Patterns
```
"show/inspect/what is sentron N"  → Sentron(N)
"fleet/show fleet/how's the fleet" → Fleet
"status/how are we/health"        → Status
"read/what's at COORD"            → Read(COORD)
"write DATA to/at COORD"          → Write(COORD, DATA)
"run N cycles/execute N"          → Run(N)
"encode TEXT / pronounce TEXT"     → Encode(TEXT)
"decode SYLS / what byte is SYL"  → Decode(SYLS)
"memory/how much memory"          → Memory
"help/what can you do"            → Help
"quit/bye/exit"                   → Quit
```

### Architecture
```
English input → IntentParser::parse() → Command → ReplSession::execute() → narrated output
```

New module: `src/intent.rs`
