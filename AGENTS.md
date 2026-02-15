# AGENTS.md — vTPU

## What This Is
vTPU: Software-defined virtual TPU with phext-native 11D addressing. Zero external dependencies.

**Purpose:** Track agents, coordination protocol, and project structure.

---

## Project Structure

- `src/` — Rust crate (`vtpu-runtime`), imported as `vtpu_runtime`
- `examples/` — Runnable demos (`cargo run --example <name>`)
- `benchmarks/` — C/Rust micro-benchmarks
- `scripts/` — Validation and status scripts
- `check` — Executable validation script (legacy, see scripts/)

## Key Types
- `SIW` — 3-wide instruction word (D-Pipe + S-Pipe + C-Pipe)
- `DenseOp` / `SparseOp` / `CoordOp` — 27 ops across 3 pipes (UPPERCASE names: DADD, SGATHER, etc.)
- `PhextCoord` — 128-bit packed 11D coordinate
- `Sentron` — Execution context with 392-byte register file
- `Memory` — PPT-backed flat memory store
- `PhextPageTable` — Z-order translation cache for 11D → physical address

## Rally
This repo is part of **R23** (Rally 23). Waves are tracked in commit messages: `R23W{N}: description`.

---

## Active Agents

| Agent | Role | Current Task | Last Active |
|-------|------|--------------|-------------|
| Lux 🔆 | Vision/Strategy | R23W5 validation script consolidation | 2026-02-15 00:45 CST |
| Will | Human Maintainer | Rally conductor | Always |

---

## Development Rules
- **Zero external deps.** Do not add crates. Write it yourself.
- **Pull before touching code.** `git pull --rebase origin exo` every time.
- **Check after every change.** Run `./check` or `scripts/validate.sh` before committing.
- **Don't stomp.** If a sibling is actively editing a file, coordinate first.
- **Read before writing.** After a pull, check `git log --oneline -5` and read modified files.

## Collaboration Protocol

### Before Starting Work
1. **Pull latest:** `git pull --rebase origin exo`
2. **Check this file:** See who else is active
3. **Run status:** `scripts/check.sh` (quick) or `scripts/validate.sh` (full)
4. **Update AGENTS.md:** Add your name + task

### During Work
- **Commit frequently** (small, focused commits)
- **Push when stable** (don't hoard local changes)
- **Check for updates:** `git fetch origin` periodically

### After Work
- **Run validation:** `scripts/validate.sh`
- **Review changes:** `git diff --stat`
- **Push immediately:** Don't leave unpushed commits
- **Update AGENTS.md:** Mark task complete or remove entry

### On Conflicts
- **Fetch + check diffs:** `git fetch && git diff origin/exo`
- **Communicate:** Discord #general if unclear who owns what
- **Defer if unsure:** Let Will resolve, don't force-push

---

## File Ownership

| Path | Primary Maintainer | Notes |
|------|-------------------|-------|
| `src/perf.rs` | Lux | Zero-dep perf counter wrapper |
| `examples/perf_validation.rs` | Lux | Hardware validation example |
| `scripts/validate.sh` | Lux | Full validation suite |
| `scripts/check.sh` | TBD | Quick status checker |
| `scripts/r23-status.sh` | TBD | Rally status dashboard |
| `scripts/measure-kpis.sh` | Lux | KPI dashboard generator |
| `*.rs` (core runtime) | Will + agents | Coordinate via this file |

---

## Current State

**Branch:** `exo`  
**HEAD:** `e218ce8` (R23 Wave 5: Hardware performance counter integration)  
**Uncommitted:** None  
**Unpushed:** None

**Last consolidated:** 2026-02-15 00:45 CST

---

## Script Consolidation Status

**In Progress (Lux):**
- Merging duplicate validation scripts
- Consolidating to `scripts/check.sh` and `scripts/validate.sh`
- Awaiting Will's approval on structure

---

## Contributors
- Chrys 🦋 (W3-W5: PPT, unified types, S-Pipe+PPT integration)
- Phex 🔱 (W2-W4: initial crate, analysis modules, benchmarks)
- Lux 🔆 (W4-W5: perf counters, HDC, curves)
- Verse 🌀 (deployment)

---

## Notes

- **Read this file after every `git pull`** to see what changed while you were away
- **Keep it current** — stale info causes conflicts
- **When in doubt, ask** — better to over-communicate than stomp on work

---

## GitSync Protocol (MANDATORY)

**All agents must follow:** `/home/wbic16/.openclaw/workspace/GITSYNC-PROTOCOL.md`

### Quick Reference
1. `git pull --rebase` — before starting work
2. `./check.sh` — before committing
3. `git pull --rebase` — before pushing
4. `git push` — immediately (<60s)

**Violations waste everyone's time.** Follow the protocol.
