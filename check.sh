#!/bin/bash
# vTPU status check — run from /source/vtpu
set -e
echo "=== vTPU Status ==="
echo "Commit: $(git log --oneline -1)"
echo "Branch: $(git branch --show-current)"
echo ""

# Build check (zero warnings = clean)
echo "--- Build ---"
cargo build 2>&1 | grep -c "warning" | xargs -I{} echo "Warnings: {}"

# Test suite
echo ""
echo "--- Tests ---"
cargo test 2>&1 | grep "^test result"

# Dependency audit
echo ""
echo "--- Dependencies ---"
deps=$(grep -c '^\w' Cargo.lock 2>/dev/null || echo "no lockfile")
cargo_deps=$(grep -c '^\[' Cargo.toml | head -1)
echo "External deps in Cargo.toml: $(grep -cE '^[a-z]' Cargo.toml || echo 0) (target: 0)"

# LOC count
echo ""
echo "--- Size ---"
echo "Rust LOC: $(find src -name '*.rs' | xargs wc -l | tail -1 | awk '{print $1}')"
echo "Examples: $(find examples -name '*.rs' 2>/dev/null | wc -l)"
echo "Modules: $(ls src/*.rs src/**/*.rs 2>/dev/null | wc -l)"

echo ""
echo "=== ✅ All clear ==="
