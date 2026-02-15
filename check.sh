#!/bin/bash
# vTPU status check + side-by-side demo
# Usage: ./check.sh [--verbose|-v]
set -e

VERBOSE=0
[[ "${1:-}" == "--verbose" || "${1:-}" == "-v" ]] && VERBOSE=1

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'
FAIL=0

echo "╔════════════════════════════════════════╗"
echo "║         vTPU Status Check              ║"
echo "╚════════════════════════════════════════╝"
echo ""

# ── Git ──
echo "Git:"
echo "  Branch: $(git rev-parse --abbrev-ref HEAD)"
echo "  Commit: $(git log -1 --oneline)"
UNPUSHED=$(git log origin/exo..HEAD --oneline 2>/dev/null | wc -l)
DIRTY=$(git status --short | wc -l)
[ "$UNPUSHED" -gt 0 ] && echo -e "  ${RED}Unpushed: $UNPUSHED commits${NC}" && FAIL=1
[ "$DIRTY" -gt 0 ] && echo "  Uncommitted: $DIRTY files"
echo ""

# ── Build ──
echo -n "Build: "
BUILD_OUT=$(cargo build --release 2>&1)
if echo "$BUILD_OUT" | grep -qi "^error"; then
    echo -e "${RED}FAILED${NC}"; FAIL=1
else
    WARNS=$(echo "$BUILD_OUT" | grep -c "warning\[" || true)
    echo -e "${GREEN}OK${NC} ($WARNS warnings)"
fi
echo ""

# ── Tests ──
echo -n "Tests: "
TEST_OUT=$(cargo test --quiet 2>&1)
PASSED=$(echo "$TEST_OUT" | grep "^test result:" | grep -oP '\d+(?= passed)' | awk '{s+=$1}END{print s+0}')
FAILED_T=$(echo "$TEST_OUT" | grep "^test result:" | grep -oP '\d+(?= failed)' | awk '{s+=$1}END{print s+0}')
if [ "$FAILED_T" -gt 0 ]; then
    echo -e "${RED}$PASSED passed, $FAILED_T FAILED${NC}"; FAIL=1
else
    echo -e "${GREEN}$PASSED passed${NC}"
fi
echo ""

# ── Dependencies ──
echo -n "Deps: "
EXT_DEPS=$(sed -n '/^\[dependencies\]/,/^\[/p' Cargo.toml | grep -cE '^[a-z]' || true)
EXT_DEPS=${EXT_DEPS:-0}
if [ "$EXT_DEPS" -gt 0 ]; then
    echo -e "${RED}$EXT_DEPS external (target: 0)${NC}"; FAIL=1
else
    echo -e "${GREEN}0 external ✓${NC}"
fi
echo ""

# ── Size ──
echo "Size:"
echo "  Rust LOC: $(find src -name '*.rs' | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}')"
echo "  Modules:  $(find src -name '*.rs' | wc -l)"
echo "  Examples: $(find examples -name '*.rs' 2>/dev/null | wc -l)"
echo ""

# ── Side-by-Side Demo ──
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║              vTPU Side-by-Side Demo                           ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Traditional CPU (sequential):${NC}"
echo "  load r0, mem[addr1]        # 1 op/cycle"
echo "  load r1, mem[addr2]        # 1 op/cycle"
echo "  mul  r2, r0, r1            # 1 op/cycle"
echo "  store mem[addr3], r2       # 1 op/cycle"
echo "  ─────────────────────────"
echo "  4 cycles, 4 ops → 1.0 ops/cycle"
echo ""
echo -e "${YELLOW}vTPU (3-pipe SIW):${NC}"
echo "  SIW 0: D:NOP     S:GATHER r0←p0  C:NOP        # gather A"
echo "  SIW 1: D:NOP     S:GATHER r1←p1  C:NOP        # gather B"
echo "  SIW 2: D:MUL r2  S:PREFCH p2     C:PACK m0    # compute + prefetch + pack"
echo "  SIW 3: D:NOP     S:SCATTER p2←r2 C:SEND m0    # store + send result"
echo "  ─────────────────────────"
echo "  4 cycles, 8 ops → 2.0 ops/cycle (+ prefetch + message)"
echo ""

# Run actual verification
echo -e "${CYAN}Live verification:${NC}"
DEMO_OUT=$(cargo test --quiet exec::tests::gather_scatter_through_ppt 2>&1)
if echo "$DEMO_OUT" | grep -q "ok"; then
    echo -e "  ${GREEN}✓ gather→scatter→compute pipeline verified${NC}"
else
    echo -e "  ${RED}✗ pipeline test failed${NC}"
    FAIL=1
fi

DEMO_OUT2=$(cargo test --quiet exec::tests::execute_three_wide 2>&1)
if echo "$DEMO_OUT2" | grep -q "ok"; then
    echo -e "  ${GREEN}✓ 3.0 ops/cycle on fully-packed SIW verified${NC}"
else
    echo -e "  ${RED}✗ 3-wide test failed${NC}"
    FAIL=1
fi

DEMO_OUT3=$(cargo test --quiet exec::tests::compute_scatter_gather_pipeline 2>&1)
if echo "$DEMO_OUT3" | grep -q "ok"; then
    echo -e "  ${GREEN}✓ dot product via PPT-backed memory verified (26 = [2,4]·[3,5])${NC}"
else
    echo -e "  ${RED}✗ dot product test failed${NC}"
    FAIL=1
fi
echo ""

# ── Verbose ──
if [ "$VERBOSE" -eq 1 ]; then
    echo "Per-module tests:"
    echo "$TEST_OUT" | grep "^test " | sed 's/^test /  /' | head -50
    echo ""
fi

# ── Result ──
if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}✓ All clear — we've got this!${NC}"
else
    echo -e "${RED}✗ Issues found${NC}"
    exit 1
fi
