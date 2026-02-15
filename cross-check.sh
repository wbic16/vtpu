#!/bin/bash
# vTPU Cross-Check — Comprehensive component validation
# Usage: ./cross-check.sh [--verbose]
set -e

VERBOSE=0
[[ "${1:-}" == "--verbose" || "${1:-}" == "-v" ]] && VERBOSE=1

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'
FAIL=0

echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║              vTPU Component Cross-Check                        ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# ── Repository Status ──
echo -e "${YELLOW}Repository:${NC}"
BRANCH=$(git rev-parse --abbrev-ref HEAD)
COMMIT=$(git log -1 --oneline)
echo "  Branch: $BRANCH"
echo "  Commit: $COMMIT"
UNPUSHED=$(git log origin/exo..HEAD --oneline 2>/dev/null | wc -l)
DIRTY=$(git status --short | wc -l)
if [ "$UNPUSHED" -gt 0 ]; then
    echo -e "  ${RED}⚠ Unpushed: $UNPUSHED commits${NC}"
    FAIL=1
fi
if [ "$DIRTY" -gt 0 ]; then
    echo -e "  ${YELLOW}⚠ Uncommitted: $DIRTY files${NC}"
fi
echo ""

# ── Core Modules ──
echo -e "${YELLOW}Core Modules (src/):${NC}"
MODULES=(
    "lib.rs:Library root"
    "exec.rs:D-Pipe executor"
    "pipes.rs:S-Pipe ops"
    "ppt.rs:Phext Page Table"
    "phext_coord.rs:Coordinate handling"
    "sentron.rs:Execution context"
    "siw.rs:Instruction words"
    "memory.rs:Memory backend"
    "perf.rs:Hardware counters"
    "hdc.rs:HDC operations"
    "display.rs:Display formatting"
    "stream.rs:Stream processing"
    "scheduler.rs:Scheduler"
)

for entry in "${MODULES[@]}"; do
    module="${entry%%:*}"
    desc="${entry#*:}"
    if [ -f "src/$module" ]; then
        # Check if module has tests
        TESTS=$(grep "^#\[test\]" "src/$module" 2>/dev/null | wc -l)
        TESTS=${TESTS:-0}
        if [ "$TESTS" -gt 0 ]; then
            echo -e "  ${GREEN}✓${NC} $module ($desc) — $TESTS tests"
        else
            echo -e "  ${YELLOW}○${NC} $module ($desc) — no tests"
        fi
    else
        echo -e "  ${RED}✗${NC} $module (missing)"
        FAIL=1
    fi
done
echo ""

# ── Analysis Modules ──
echo -e "${YELLOW}Analysis Modules (src/analysis/):${NC}"
ANALYSIS=(
    "mod.rs:Module root"
    "port_conflicts.rs:Port conflict detection"
    "cache_thrash.rs:Cache thrashing analysis"
    "memory_patterns.rs:Memory pattern analysis"
    "locality.rs:Dimensional locality"
    "smt.rs:SMT efficiency"
)

for entry in "${ANALYSIS[@]}"; do
    module="${entry%%:*}"
    desc="${entry#*:}"
    if [ -f "src/analysis/$module" ]; then
        TESTS=$(grep "^#\[test\]" "src/analysis/$module" 2>/dev/null | wc -l)
        TESTS=${TESTS:-0}
        if [ "$TESTS" -gt 0 ]; then
            echo -e "  ${GREEN}✓${NC} $module ($desc) — $TESTS tests"
        else
            echo -e "  ${YELLOW}○${NC} $module ($desc) — no tests"
        fi
    else
        echo -e "  ${RED}✗${NC} $module (missing)"
        FAIL=1
    fi
done
echo ""

# ── Curve Modules ──
echo -e "${YELLOW}Curve Modules (src/curves/):${NC}"
CURVES=(
    "mod.rs:Module root"
    "zorder.rs:Z-order (Morton)"
    "hilbert.rs:Hilbert curve"
)

for entry in "${CURVES[@]}"; do
    module="${entry%%:*}"
    desc="${entry#*:}"
    if [ -f "src/curves/$module" ]; then
        TESTS=$(grep "^#\[test\]" "src/curves/$module" 2>/dev/null | wc -l)
        TESTS=${TESTS:-0}
        if [ "$TESTS" -gt 0 ]; then
            echo -e "  ${GREEN}✓${NC} $module ($desc) — $TESTS tests"
        else
            echo -e "  ${YELLOW}○${NC} $module ($desc) — no tests"
        fi
    else
        echo -e "  ${RED}✗${NC} $module (missing)"
        FAIL=1
    fi
done
echo ""

# ── Examples ──
echo -e "${YELLOW}Examples (examples/):${NC}"
EXAMPLES=(
    "basic_compute.rs:Basic computation demo"
    "benchmark_runner.rs:Benchmark orchestration"
    "ddr_benchmark.rs:DDR5 memory benchmark"
    "micro_bench.rs:Micro-benchmark suite"
    "perf_validation.rs:Performance validation"
    "port_validation.rs:Port conflict validation"
    "ppt_benchmark.rs:PPT performance"
    "validation_demo.rs:Integration demo"
)

EXAMPLES_OK=0
EXAMPLES_TOTAL=${#EXAMPLES[@]}
for entry in "${EXAMPLES[@]}"; do
    example="${entry%%:*}"
    desc="${entry#*:}"
    if [ -f "examples/$example" ]; then
        echo -e "  ${GREEN}✓${NC} ${example%.rs} ($desc)"
        EXAMPLES_OK=$((EXAMPLES_OK + 1))
    else
        echo -e "  ${RED}✗${NC} ${example%.rs} (missing)"
        FAIL=1
    fi
done
echo "  Summary: $EXAMPLES_OK/$EXAMPLES_TOTAL examples present"
echo ""

# ── Benchmarks ──
echo -e "${YELLOW}Benchmarks (benchmarks/):${NC}"
if [ -d "benchmarks/cache" ]; then
    CACHE_BENCHES=$(ls benchmarks/cache/*.c 2>/dev/null | wc -l)
    echo -e "  ${GREEN}✓${NC} Cache benchmarks ($CACHE_BENCHES C files)"
else
    echo -e "  ${RED}✗${NC} Cache benchmarks (missing)"
    FAIL=1
fi

if [ -d "benchmarks/smt" ]; then
    SMT_BENCHES=$(ls benchmarks/smt/*.rs 2>/dev/null | wc -l)
    echo -e "  ${GREEN}✓${NC} SMT benchmarks ($SMT_BENCHES Rust files)"
else
    echo -e "  ${YELLOW}○${NC} SMT benchmarks (none yet)"
fi

if [ -d "benchmarks/workloads" ]; then
    WORKLOADS=$(ls benchmarks/workloads/*.siw 2>/dev/null | wc -l)
    echo -e "  ${GREEN}✓${NC} Synthetic workloads ($WORKLOADS .siw files)"
else
    echo -e "  ${YELLOW}○${NC} Synthetic workloads (none yet)"
fi
echo ""

# ── Build Status ──
echo -e "${YELLOW}Build:${NC}"
if cargo build --release --quiet 2>&1 | grep -qi "error"; then
    echo -e "  ${RED}✗ Build FAILED${NC}"
    FAIL=1
    cargo build --release 2>&1 | grep "error\[" | head -5
else
    WARNS=$(cargo build --release 2>&1 | grep -c "warning\[" || true)
    if [ "$WARNS" -eq 0 ]; then
        echo -e "  ${GREEN}✓ Build OK (0 warnings)${NC}"
    else
        echo -e "  ${YELLOW}○ Build OK ($WARNS warnings)${NC}"
    fi
fi
echo ""

# ── Test Coverage ──
echo -e "${YELLOW}Test Coverage:${NC}"
TEST_OUT=$(cargo test --quiet 2>&1)
PASSED=$(echo "$TEST_OUT" | grep "^test result:" | grep -oP '\d+(?= passed)' | awk '{s+=$1}END{print s+0}')
FAILED_T=$(echo "$TEST_OUT" | grep "^test result:" | grep -oP '\d+(?= failed)' | awk '{s+=$1}END{print s+0}')

if [ "$FAILED_T" -gt 0 ]; then
    echo -e "  ${RED}✗ $PASSED passed, $FAILED_T FAILED${NC}"
    FAIL=1
else
    echo -e "  ${GREEN}✓ All $PASSED tests passed${NC}"
fi

# Module-specific test breakdown
if [ "$VERBOSE" -eq 1 ]; then
    echo ""
    echo "  Per-module breakdown:"
    echo "$TEST_OUT" | grep "^test " | sed 's/test /    /' | head -30
fi
echo ""

# ── Dependencies ──
echo -e "${YELLOW}Dependencies:${NC}"
EXT_DEPS=$(sed -n '/^\[dependencies\]/,/^\[/p' Cargo.toml | grep -cE '^[a-z]' || true)
EXT_DEPS=${EXT_DEPS:-0}
if [ "$EXT_DEPS" -gt 0 ]; then
    echo -e "  ${RED}✗ $EXT_DEPS external crates (target: 0)${NC}"
    FAIL=1
    sed -n '/^\[dependencies\]/,/^\[/p' Cargo.toml | grep -E '^[a-z]' | sed 's/^/    /'
else
    echo -e "  ${GREEN}✓ Zero external dependencies${NC}"
fi
echo ""

# ── Code Size ──
echo -e "${YELLOW}Code Size:${NC}"
TOTAL_LOC=$(find src -name '*.rs' | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}')
CORE_LOC=$(wc -l src/{lib,exec,pipes,ppt,phext_coord,sentron,siw,memory}.rs 2>/dev/null | tail -1 | awk '{print $1}')
ANALYSIS_LOC=$(find src/analysis -name '*.rs' | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}')
CURVES_LOC=$(find src/curves -name '*.rs' | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}')

echo "  Total: $TOTAL_LOC lines"
echo "  Core:  $CORE_LOC lines (exec, pipes, PPT, sentron, etc.)"
echo "  Analysis: $ANALYSIS_LOC lines"
echo "  Curves: $CURVES_LOC lines"
echo ""

# ── Integration Checks ──
echo -e "${YELLOW}Integration:${NC}"

# Check if PPT can be imported from lib
if grep -q "pub use.*ppt::" src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓${NC} PPT exported from lib"
else
    echo -e "  ${YELLOW}○${NC} PPT not publicly exported"
fi

# Check if curves are exported
if grep -q "pub use.*curves::" src/lib.rs 2>/dev/null || grep -q "pub mod curves" src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓${NC} Curves module exported"
else
    echo -e "  ${YELLOW}○${NC} Curves not publicly exported"
fi

# Check if analysis tools are available
if grep -q "pub use.*analysis::" src/lib.rs 2>/dev/null || grep -q "pub mod analysis" src/lib.rs 2>/dev/null; then
    echo -e "  ${GREEN}✓${NC} Analysis tools exported"
else
    echo -e "  ${YELLOW}○${NC} Analysis tools not publicly exported"
fi

echo ""

# ── Performance Indicators ──
echo -e "${YELLOW}Performance Indicators:${NC}"

# Check if perf module exists and is functional
if [ -f "src/perf.rs" ]; then
    if cargo test --quiet 2>&1 | grep -q "perf::.*ok"; then
        echo -e "  ${GREEN}✓${NC} Hardware perf counters functional"
    else
        echo -e "  ${YELLOW}○${NC} Perf counters present (tests needed)"
    fi
else
    echo -e "  ${RED}✗${NC} Perf module missing"
    FAIL=1
fi

# Check for benchmark infrastructure
if [ -f "examples/perf_validation.rs" ]; then
    echo -e "  ${GREEN}✓${NC} Performance validation framework ready"
else
    echo -e "  ${YELLOW}○${NC} No perf validation example"
fi

echo ""

# ── Documentation ──
echo -e "${YELLOW}Documentation:${NC}"
README_SIZE=$(wc -c README.md 2>/dev/null | awk '{print $1}')
AGENTS_SIZE=$(wc -c AGENTS.md 2>/dev/null | awk '{print $1}')

if [ -f "README.md" ]; then
    echo -e "  ${GREEN}✓${NC} README.md ($README_SIZE bytes)"
else
    echo -e "  ${YELLOW}○${NC} No README.md"
fi

if [ -f "AGENTS.md" ]; then
    echo -e "  ${GREEN}✓${NC} AGENTS.md ($AGENTS_SIZE bytes)"
else
    echo -e "  ${RED}✗${NC} AGENTS.md missing"
    FAIL=1
fi

echo ""

# ── Final Summary ──
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                      Summary                                   ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "  Modules:   $(find src -name '*.rs' | wc -l) files, $TOTAL_LOC lines"
echo "  Tests:     $PASSED passing"
echo "  Examples:  $EXAMPLES_OK/$EXAMPLES_TOTAL present"
echo "  Deps:      $EXT_DEPS external (target: 0)"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}✓ All components verified — ready for R23W6${NC}"
else
    echo -e "${RED}✗ Issues found — review above${NC}"
    exit 1
fi
