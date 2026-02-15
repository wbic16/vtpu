#!/bin/bash
# R23 Rally Status Check - Single command validation
# Usage: ./scripts/r23-status.sh

set -e

echo "═══════════════════════════════════════════════════════"
echo "  R23 Rally Status Check"
echo "  vtpu: Virtual TPU on AMD R9 8945HS + Phext"
echo "═══════════════════════════════════════════════════════"
echo ""

# 1. Git status
echo "┌─ Git Status ───────────────────────────────────────────"
BRANCH=$(git rev-parse --abbrev-ref HEAD)
COMMIT=$(git log -1 --oneline | head -c 60)
echo "│ Branch: $BRANCH"
echo "│ Latest: $COMMIT"
echo "│ Uncommitted changes:"
git status --short | head -10 | sed 's/^/│   /'
if [ $(git status --short | wc -l) -gt 10 ]; then
    echo "│   ... ($(git status --short | wc -l) total)"
fi
echo "└────────────────────────────────────────────────────────"
echo ""

# 2. Build status
echo "┌─ Build Status ─────────────────────────────────────────"
if cargo build --release 2>&1 | grep -q "Finished"; then
    echo "│ ✅ Release build: PASSING"
else
    echo "│ ❌ Release build: FAILED"
    cargo build --release 2>&1 | tail -20 | sed 's/^/│   /'
    exit 1
fi
echo "└────────────────────────────────────────────────────────"
echo ""

# 3. Test status
echo "┌─ Test Status ──────────────────────────────────────────"
TEST_OUTPUT=$(cargo test --lib 2>&1)
TEST_COUNT=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= passed)' | tail -1)
FAILED_COUNT=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= failed)' || echo "0")

if [ "$FAILED_COUNT" = "0" ]; then
    echo "│ ✅ Unit tests: $TEST_COUNT passed, 0 failed"
else
    echo "│ ❌ Unit tests: $TEST_COUNT passed, $FAILED_COUNT failed"
    echo "$TEST_OUTPUT" | grep -A 5 "failures:" | sed 's/^/│   /'
fi
echo "└────────────────────────────────────────────────────────"
echo ""

# 4. Example binaries
echo "┌─ Example Binaries ─────────────────────────────────────"
EXAMPLES=$(ls examples/*.rs 2>/dev/null | wc -l)
echo "│ Examples available: $EXAMPLES"
ls examples/*.rs 2>/dev/null | sed 's|examples/||;s|.rs$||' | sed 's/^/│   - /'
echo "│"
echo "│ Run example:"
echo "│   cargo run --release --example <name>"
echo "└────────────────────────────────────────────────────────"
echo ""

# 5. Wave progress
echo "┌─ R23 Wave Progress ────────────────────────────────────"
WAVES_DIR="docs"
if [ -d "$WAVES_DIR" ]; then
    WAVE_DIRS=$(ls -d $WAVES_DIR/wave-* 2>/dev/null | wc -l)
    echo "│ Completed waves: $WAVE_DIRS"
    ls -d $WAVES_DIR/wave-* 2>/dev/null | sed 's|docs/||' | sed 's/^/│   /'
    
    # Show latest wave
    LATEST_WAVE=$(ls -d $WAVES_DIR/wave-* 2>/dev/null | sort -V | tail -1)
    if [ -n "$LATEST_WAVE" ]; then
        echo "│"
        echo "│ Latest wave: $(basename $LATEST_WAVE)"
        if [ -f "$LATEST_WAVE/README.md" ]; then
            head -5 "$LATEST_WAVE/README.md" | sed 's/^/│   /'
        fi
    fi
else
    echo "│ No wave documentation found"
fi
echo "└────────────────────────────────────────────────────────"
echo ""

# 6. KPI Dashboard (if exists)
echo "┌─ KPI Status ───────────────────────────────────────────"
DASHBOARD="docs/R23-DELIVERABLE-DASHBOARD.md"
if [ -f "$DASHBOARD" ]; then
    echo "│ Dashboard: $DASHBOARD"
    echo "│"
    # Extract KPI section
    sed -n '/## KPI Tracking/,/^## /p' "$DASHBOARD" | head -20 | sed 's/^/│ /'
else
    echo "│ No KPI dashboard found"
fi
echo "└────────────────────────────────────────────────────────"
echo ""

# 7. Benchmarks (if any exist)
echo "┌─ Available Benchmarks ─────────────────────────────────"
BENCH_COUNT=$(ls examples/*benchmark.rs 2>/dev/null | wc -l)
if [ $BENCH_COUNT -gt 0 ]; then
    echo "│ Benchmarks: $BENCH_COUNT"
    ls examples/*benchmark.rs 2>/dev/null | sed 's|examples/||;s|.rs$||' | sed 's/^/│   - /'
    echo "│"
    echo "│ Run benchmark:"
    echo "│   ./target/release/examples/<benchmark-name>"
else
    echo "│ No benchmarks found"
fi
echo "└────────────────────────────────────────────────────────"
echo ""

# 8. Next steps (from latest wave or dashboard)
echo "┌─ Next Steps ───────────────────────────────────────────"
if [ -f "$DASHBOARD" ]; then
    NEXT_WAVE=$(grep -A 10 "Wave.*Status.*QUEUED" "$DASHBOARD" | head -5 | sed 's/^/│ /')
    if [ -n "$NEXT_WAVE" ]; then
        echo "$NEXT_WAVE"
    else
        echo "│ Check dashboard for next wave:"
        echo "│   cat $DASHBOARD"
    fi
else
    echo "│ No dashboard found - check docs/ for latest wave"
fi
echo "└────────────────────────────────────────────────────────"
echo ""

echo "═══════════════════════════════════════════════════════"
echo "  Status check complete"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Quick commands:"
echo "  Build:     cargo build --release"
echo "  Test:      cargo test"
echo "  Benchmark: cargo run --release --example ddr_benchmark"
echo "  Format:    cargo fmt"
echo "  Lint:      cargo clippy"
echo ""
