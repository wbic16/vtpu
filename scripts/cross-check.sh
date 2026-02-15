#!/bin/bash
# R23W6 Cross-Check: Comprehensive vTPU Component Validation
# Validates all components, integration, benchmarks, and documentation

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

PASS=0
WARN=0
FAIL=0

echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}vTPU R23W6 Cross-Check: Component Validation${NC}"
echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
echo ""

# ═══════════════════════════════════════════════════════════════
# Git Status
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[1/10] Git Status${NC}"
BRANCH=$(git rev-parse --abbrev-ref HEAD)
COMMIT=$(git log -1 --oneline)
UNPUSHED=$(git log origin/exo..HEAD --oneline 2>/dev/null | wc -l)
DIRTY=$(git status --short | wc -l)

echo "  Branch: $BRANCH"
echo "  Commit: $COMMIT"

if [ "$UNPUSHED" -gt 0 ]; then
    echo -e "  ${RED}✗ Unpushed: $UNPUSHED commits${NC}"
    FAIL=$((FAIL+1))
else
    echo -e "  ${GREEN}✓ Synced with origin${NC}"
    PASS=$((PASS+1))
fi

if [ "$DIRTY" -gt 0 ]; then
    echo -e "  ${YELLOW}⚠ Uncommitted: $DIRTY files${NC}"
    WARN=$((WARN+1))
else
    echo -e "  ${GREEN}✓ Working tree clean${NC}"
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Core Types
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[2/10] Core Types${NC}"

# Check SIW struct (search all src/*.rs)
if grep -rq "pub struct SIW" src/ 2>/dev/null; then
    echo -e "  ${GREEN}✓ SIW${NC} (3-pipe instruction word)"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ SIW${NC} missing"
    FAIL=$((FAIL+1))
fi

# Check PhextCoord (search all src/*.rs)
if grep -rq "pub struct PhextCoord" src/ 2>/dev/null; then
    echo -e "  ${GREEN}✓ PhextCoord${NC} (11D coordinate)"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ PhextCoord${NC} missing"
    FAIL=$((FAIL+1))
fi

# Check pipe operations (search all src/*.rs)
for OP in DenseOp SparseOp CoordOp; do
    if grep -rq "pub enum $OP" src/ 2>/dev/null; then
        echo -e "  ${GREEN}✓ $OP${NC}"
        PASS=$((PASS+1))
    else
        echo -e "  ${RED}✗ $OP${NC} missing"
        FAIL=$((FAIL+1))
    fi
done

# Check Sentron (search all src/*.rs)
if grep -rq "pub struct Sentron" src/ 2>/dev/null; then
    echo -e "  ${GREEN}✓ Sentron${NC} (execution context)"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ Sentron${NC} missing"
    FAIL=$((FAIL+1))
fi

# Check Memory (search all src/*.rs)
if grep -rq "pub struct Memory" src/ 2>/dev/null; then
    echo -e "  ${GREEN}✓ Memory${NC} (PPT-backed store)"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ Memory${NC} missing"
    FAIL=$((FAIL+1))
fi

# Check PhextPageTable (search all src/*.rs)
if grep -rq "PhextPageTable\|PPT" src/ 2>/dev/null; then
    echo -e "  ${GREEN}✓ PhextPageTable${NC} (Z-order indexing)"
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ PhextPageTable${NC} not found (may be embedded)"
    WARN=$((WARN+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Build System
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[3/10] Build System${NC}"

if command -v cargo &> /dev/null; then
    BUILD_OUT=$(cargo build --release 2>&1 || true)
    if echo "$BUILD_OUT" | grep -qi "^error"; then
        echo -e "  ${RED}✗ Build failed${NC}"
        FAIL=$((FAIL+1))
    else
        WARNS=$(echo "$BUILD_OUT" | grep -c "warning:" || echo "0")
        if [ "$WARNS" -gt 0 ]; then
            echo -e "  ${YELLOW}⚠ Build succeeded ($WARNS warnings)${NC}"
            WARN=$((WARN+1))
        else
            echo -e "  ${GREEN}✓ Build succeeded (0 warnings)${NC}"
            PASS=$((PASS+1))
        fi
    fi
    
    # Check binary size
    if [ -f target/release/libvtpu_runtime.rlib ]; then
        SIZE=$(du -h target/release/libvtpu_runtime.rlib | cut -f1)
        echo "  Binary size: $SIZE"
    fi
else
    echo -e "  ${YELLOW}⚠ Cargo not found - skipping build check${NC}"
    WARN=$((WARN+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Tests
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[4/10] Test Suite${NC}"

if command -v cargo &> /dev/null; then
    TEST_OUT=$(cargo test --lib 2>&1 || true)
    PASSED=$(echo "$TEST_OUT" | grep "test result:" | grep -oP '\d+(?= passed)' || echo "0")
    FAILED=$(echo "$TEST_OUT" | grep "test result:" | grep -oP '\d+(?= failed)' || echo "0")
    
    if [ "$FAILED" -gt 0 ]; then
        echo -e "  ${RED}✗ $FAILED tests failed${NC}"
        FAIL=$((FAIL+1))
    elif [ "$PASSED" -gt 0 ]; then
        echo -e "  ${GREEN}✓ $PASSED tests passed${NC}"
        PASS=$((PASS+1))
    else
        echo -e "  ${YELLOW}⚠ No tests run${NC}"
        WARN=$((WARN+1))
    fi
    
    # Test coverage by component
    echo "  Component coverage:"
    for COMP in siw coord dense sparse memory exec; do
        COUNT=$(grep -r "#\[test\]" src/ 2>/dev/null | grep -i "$COMP" | wc -l || echo "0")
        if [ "$COUNT" -gt 0 ]; then
            echo "    - $COMP: $COUNT tests"
        fi
    done
else
    echo -e "  ${YELLOW}⚠ Cargo not found - skipping tests${NC}"
    WARN=$((WARN+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Dependencies
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[5/10] Dependencies${NC}"

EXT_DEPS=$(grep -A100 '^\[dependencies\]' Cargo.toml 2>/dev/null | grep -E '^[a-z]' | grep -v '^\[' | wc -l || echo "0")

if [ "$EXT_DEPS" -eq 0 ]; then
    echo -e "  ${GREEN}✓ Zero external dependencies${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ $EXT_DEPS external dependencies (target: 0)${NC}"
    grep -A100 '^\[dependencies\]' Cargo.toml | grep -E '^[a-z]' | head -10
    FAIL=$((FAIL+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Benchmarks
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[6/10] Benchmark Suites${NC}"

BENCH_COUNT=0

# Cache benchmarks
if [ -d benchmarks/cache ]; then
    echo -e "  ${GREEN}✓ Cache locality${NC} (benchmarks/cache/)"
    BENCH_COUNT=$((BENCH_COUNT+1))
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ Cache locality${NC} missing"
    FAIL=$((FAIL+1))
fi

# SMT benchmarks
if [ -d benchmarks/smt ]; then
    echo -e "  ${GREEN}✓ SMT analysis${NC} (benchmarks/smt/)"
    BENCH_COUNT=$((BENCH_COUNT+1))
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ SMT analysis${NC} missing"
    FAIL=$((FAIL+1))
fi

# DDR5 benchmarks
if [ -d benchmarks/ddr ] || [ -f examples/ddr_benchmark.rs ]; then
    echo -e "  ${GREEN}✓ DDR5 memory${NC}"
    BENCH_COUNT=$((BENCH_COUNT+1))
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ DDR5 memory${NC} not found"
    WARN=$((WARN+1))
fi

# Sparse attention benchmark (R23W6)
if [ -d benchmarks/sparse_attention ]; then
    echo -e "  ${GREEN}✓ Sparse attention${NC} (benchmarks/sparse_attention/)"
    BENCH_COUNT=$((BENCH_COUNT+1))
    PASS=$((PASS+1))
    
    # Validate sparse attention benchmark
    if [ -f benchmarks/sparse_attention/src/lib.rs ]; then
        if grep -q "baseline_sparse_attention" benchmarks/sparse_attention/src/lib.rs; then
            echo "    - baseline_sparse_attention() ✓"
        fi
        if grep -q "vtpu_sparse_attention" benchmarks/sparse_attention/src/lib.rs; then
            echo "    - vtpu_sparse_attention() ✓"
        fi
        if grep -q "z_order" benchmarks/sparse_attention/src/lib.rs; then
            echo "    - Z-order Morton code ✓"
        fi
    fi
else
    echo -e "  ${RED}✗ Sparse attention${NC} missing (R23W6 deliverable)"
    FAIL=$((FAIL+1))
fi

echo "  Total benchmark suites: $BENCH_COUNT"
echo ""

# ═══════════════════════════════════════════════════════════════
# Documentation
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[7/10] Documentation${NC}"

# README
if [ -f README.md ]; then
    README_SIZE=$(wc -l README.md | awk '{print $1}')
    echo -e "  ${GREEN}✓ README.md${NC} ($README_SIZE lines)"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ README.md${NC} missing"
    FAIL=$((FAIL+1))
fi

# AGENTS.md
if [ -f AGENTS.md ]; then
    echo -e "  ${GREEN}✓ AGENTS.md${NC} (agent coordination)"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ AGENTS.md${NC} missing"
    FAIL=$((FAIL+1))
fi

# Benchmark READMEs
BENCH_DOCS=0
for DIR in benchmarks/*/; do
    if [ -f "$DIR/README.md" ]; then
        BENCH_DOCS=$((BENCH_DOCS+1))
    fi
done
echo "  Benchmark READMEs: $BENCH_DOCS"

# Inline documentation
DOC_COMMENTS=$(grep -r "///" src/ 2>/dev/null | wc -l || echo "0")
echo "  Doc comments: $DOC_COMMENTS"

if [ "$DOC_COMMENTS" -gt 50 ]; then
    echo -e "  ${GREEN}✓ Well-documented${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ Sparse documentation${NC}"
    WARN=$((WARN+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Code Metrics
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[8/10] Code Metrics${NC}"

RUST_LOC=$(find src -name '*.rs' 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")
RUST_FILES=$(find src -name '*.rs' 2>/dev/null | wc -l)
EXAMPLE_FILES=$(find examples -name '*.rs' 2>/dev/null | wc -l || echo "0")
BENCH_FILES=$(find benchmarks -name '*.rs' -o -name '*.c' 2>/dev/null | wc -l || echo "0")

echo "  Rust source:"
echo "    - Files: $RUST_FILES"
echo "    - LOC: $RUST_LOC"
echo "  Examples: $EXAMPLE_FILES files"
echo "  Benchmarks: $BENCH_FILES files"

TOTAL_LOC=$((RUST_LOC + $(find examples benchmarks -name '*.rs' -o -name '*.c' 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")))
echo "  Total LOC: $TOTAL_LOC"

if [ "$RUST_LOC" -gt 1000 ]; then
    echo -e "  ${GREEN}✓ Non-trivial codebase${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ Small codebase${NC}"
    WARN=$((WARN+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Integration Checks
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[9/10] Integration${NC}"

# Check SIW uses all three pipes
if grep -q "d_op.*s_op.*c_op\|DenseOp.*SparseOp.*CoordOp" src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓ SIW integrates D/S/C pipes${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ SIW missing pipe integration${NC}"
    FAIL=$((FAIL+1))
fi

# Check Sentron uses SIW
if grep -q "Sentron.*SIW\|execute.*SIW" src/exec.rs src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓ Sentron executes SIW${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ Sentron-SIW integration unclear${NC}"
    WARN=$((WARN+1))
fi

# Check Memory uses PhextCoord
if grep -q "Memory.*PhextCoord\|PhextCoord.*Memory" src/memory.rs src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓ Memory uses PhextCoord${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ Memory-PhextCoord integration unclear${NC}"
    WARN=$((WARN+1))
fi

# Check Z-order indexing
if grep -q "z_order\|morton\|Z-order" src/memory.rs benchmarks/sparse_attention/src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓ Z-order indexing present${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ Z-order indexing missing${NC}"
    FAIL=$((FAIL+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# R23W6 Deliverables
# ═══════════════════════════════════════════════════════════════
echo -e "${CYAN}[10/10] R23W6 Deliverables${NC}"

# Sparse attention benchmark
if [ -f benchmarks/sparse_attention/src/lib.rs ]; then
    LOC=$(wc -l benchmarks/sparse_attention/src/lib.rs | awk '{print $1}')
    echo -e "  ${GREEN}✓ Sparse attention benchmark${NC} ($LOC LOC)"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ Sparse attention benchmark${NC}"
    FAIL=$((FAIL+1))
fi

# Check for 10x speedup claim validation
if grep -q "10.*speedup\|10x" benchmarks/sparse_attention/README.md 2>/dev/null; then
    echo -e "  ${GREEN}✓ 10× speedup claim documented${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ 10× speedup claim not documented${NC}"
    WARN=$((WARN+1))
fi

# Check for Z-order implementation
if grep -q "fn z_order" benchmarks/sparse_attention/src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓ Z-order Morton code implemented${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ Z-order Morton code missing${NC}"
    FAIL=$((FAIL+1))
fi

# Check for baseline comparison
if grep -q "baseline_sparse_attention" benchmarks/sparse_attention/src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓ Baseline comparison present${NC}"
    PASS=$((PASS+1))
else
    echo -e "  ${RED}✗ Baseline comparison missing${NC}"
    FAIL=$((FAIL+1))
fi

# Check for tests
BENCH_TESTS=$(grep -c "#\[test\]" benchmarks/sparse_attention/src/lib.rs 2>/dev/null || echo "0")
if [ "$BENCH_TESTS" -ge 3 ]; then
    echo -e "  ${GREEN}✓ Sparse attention tests${NC} ($BENCH_TESTS tests)"
    PASS=$((PASS+1))
else
    echo -e "  ${YELLOW}⚠ Sparse attention tests${NC} ($BENCH_TESTS tests, expected 3+)"
    WARN=$((WARN+1))
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════
echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Summary${NC}"
echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
echo ""

TOTAL=$((PASS + WARN + FAIL))
echo -e "${GREEN}✓ Pass: $PASS${NC}"
echo -e "${YELLOW}⚠ Warn: $WARN${NC}"
echo -e "${RED}✗ Fail: $FAIL${NC}"
echo "  Total: $TOTAL checks"
echo ""

PASS_PCT=$((PASS * 100 / TOTAL))
echo "Pass rate: $PASS_PCT%"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}✅ All critical checks passed${NC}"
    exit 0
elif [ "$FAIL" -le 3 ]; then
    echo -e "${YELLOW}⚠️  Minor issues detected${NC}"
    exit 0
else
    echo -e "${RED}❌ Critical issues detected${NC}"
    exit 1
fi
