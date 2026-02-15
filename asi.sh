#!/bin/bash
# asi — vTPU interactive frontend
# Builds (if needed) and launches the Rust REPL (real vTPU engine, not a simulation)
cd "$(dirname "$0")"
cargo build --bin asi --quiet 2>/dev/null
exec ./target/debug/asi "$@"
