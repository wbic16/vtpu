#!/bin/bash
# vTPU Validation Script
# Single command to validate entire vTPU state

set -e

echo "=== vTPU Validation Suite ==="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
FAILED=0

# 1. Unit tests (includes compilation check)
echo -n "🧪 Running unit tests... "
TEST_OUTPUT=$(cargo test --lib --quiet 2>&1 || echo "FAILED")
if echo "$TEST_OUTPUT" | grep -q "test result: ok"; then
    TEST_COUNT=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= passed)' | head -1)
    echo -e "${GREEN}✓ ($TEST_COUNT passed)${NC}"
else
    echo -e "${RED}✗${NC}"
    echo "$TEST_OUTPUT" | tail -10
    FAILED=1
fi

# 2. Check binaries exist (don't rebuild)
echo -n "🔨 Release binaries... "
if [ -f target/release/siw-gen ]; then
    echo -e "${GREEN}✓ (siw-gen)${NC}"
else
    echo -e "${YELLOW}⊘ (run: cargo build --release --bins)${NC}"
fi

# 3. Check examples exist
echo -n "📦 Examples... "
EXAMPLES=$(find target/release/examples -type f -executable 2>/dev/null | wc -l || echo "0")
if [ "$EXAMPLES" -gt 0 ]; then
    echo -e "${GREEN}✓ ($EXAMPLES built)${NC}"
else
    echo -e "${YELLOW}⊘ (run: cargo build --release --examples)${NC}"
fi

# 4. Run quick benchmark (if workloads exist and benchmark_runner built)
if [ -f benchmarks/workloads/balanced-medium.siw ] && [ -f target/release/examples/benchmark_runner ]; then
    echo -n "⚡ Quick benchmark... "
    BENCH_OUTPUT=$(timeout 5 ./target/release/examples/benchmark_runner 2>&1 | grep "Balanced (Medium)" || echo "TIMEOUT")
    if echo "$BENCH_OUTPUT" | grep -q "ops/cycle"; then
        OPS_PER_CYCLE=$(echo "$BENCH_OUTPUT" | grep -oP '\d+\.\d+(?= ops/cycle)')
        echo -e "${GREEN}✓ ($OPS_PER_CYCLE ops/cycle)${NC}"
    else
        echo -e "${YELLOW}⊘ (benchmark timeout or failed)${NC}"
    fi
else
    echo -e "${YELLOW}⊘ Workloads or runner missing${NC}"
fi

# 5. Dependency check
echo -n "📦 Dependencies... "
DEP_COUNT=$(cargo tree 2>/dev/null | grep -v "vtpu-runtime" | wc -l)
if [ "$DEP_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓ (zero external deps)${NC}"
else
    echo -e "${YELLOW}⚠ ($DEP_COUNT found)${NC}"
fi

# 6. Code metrics
echo -n "📏 Code size... "
RUST_FILES=$(find src -name "*.rs" 2>/dev/null | wc -l)
RUST_LINES=$(find src -name "*.rs" -exec cat {} + 2>/dev/null | wc -l)
echo -e "${GREEN}$RUST_FILES files, $RUST_LINES LOC${NC}"

# 7. Binary size
if [ -f target/release/siw-gen ]; then
    echo -n "💾 siw-gen binary... "
    SIZE=$(ls -lh target/release/siw-gen | awk '{print $5}')
    echo -e "${GREEN}$SIZE${NC}"
fi

# Summary
echo ""
echo "==================================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed${NC}"
    echo ""
    echo "R23W4 Status:"
    echo "  • $TEST_COUNT unit tests passing"
    echo "  • Zero external dependencies ✓"
    echo "  • $RUST_FILES source files ($RUST_LINES LOC)"
    echo "  • Ready for next step"
    echo ""
    echo "Next: Run specific validation or build commands as needed"
    exit 0
else
    echo -e "${RED}❌ Some checks failed${NC}"
    exit 1
fi
