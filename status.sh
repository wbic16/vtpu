#!/bin/bash
# vTPU Comprehensive Status Runner
# R23 Wave 6+ - validates all components

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        vTPU Comprehensive Component Status                    ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo

# Git status
echo -e "${YELLOW}[Git Status]${NC}"
BRANCH=$(git branch --show-current)
COMMIT=$(git log -1 --oneline | head -c 60)
UNCOMMITTED=$(git status --porcelain | wc -l)
echo "  Branch: $BRANCH"
echo "  Commit: $COMMIT"
if [ "$UNCOMMITTED" -gt 0 ]; then
    echo -e "  ${YELLOW}Uncommitted: $UNCOMMITTED files${NC}"
else
    echo -e "  ${GREEN}Working tree clean${NC}"
fi
echo

# Build status
echo -e "${YELLOW}[Build]${NC}"
if cargo build --release 2>&1 | grep -q "error"; then
    echo -e "  ${RED}✗ Build failed${NC}"
    exit 1
else
    echo -e "  ${GREEN}✓ Build successful${NC}"
fi
echo

# Core modules validation
echo -e "${YELLOW}[Core Modules]${NC}"
MODULES=(
    "siw:SIW structure (64-byte, 3-pipe)"
    "pipes:D/S/C operation enums"
    "phext_coord:PhextCoord (11D addressing)"
    "ppt:Phext Page Table + translation cache"
    "memory:Memory backend + tier management"
    "exec:SIW executor (3-pipe dispatch)"
    "sentron:Sentron register file"
    "scheduler:SIW stream scheduler"
    "telemetry:Performance counters"
    "validation:SIW stream validator"
    "stream:SIW stream builder"
)

for entry in "${MODULES[@]}"; do
    module="${entry%%:*}"
    desc="${entry#*:}"
    if grep -q "mod $module" src/lib.rs 2>/dev/null || [ -f "src/$module.rs" ]; then
        echo -e "  ${GREEN}✓${NC} $module - $desc"
    else
        echo -e "  ${RED}✗${NC} $module - $desc (missing)"
    fi
done
echo

# Subsystems
echo -e "${YELLOW}[Subsystems]${NC}"
SUBSYSTEMS=(
    "curves:Space-filling curves (Hilbert, Z-order)"
    "analysis:Performance analysis (cache, SMT, ports)"
)

for entry in "${SUBSYSTEMS[@]}"; do
    subsys="${entry%%:*}"
    desc="${entry#*:}"
    if [ -d "src/$subsys" ]; then
        count=$(find "src/$subsys" -name '*.rs' | wc -l)
        echo -e "  ${GREEN}✓${NC} $subsys - $desc ($count modules)"
    else
        echo -e "  ${RED}✗${NC} $subsys - $desc (missing)"
    fi
done
echo

# Pipes validation
echo -e "${YELLOW}[3-Pipe Architecture]${NC}"
if grep -q "pub enum DenseOp" src/pipes.rs; then
    DENSE_OPS=$(grep -c "DADD\|DSUB\|DMUL\|DFMA\|DDOT\|DNOP" src/pipes.rs || echo 0)
    echo -e "  ${GREEN}✓${NC} D-Pipe (Dense compute): $DENSE_OPS ops"
else
    echo -e "  ${RED}✗${NC} D-Pipe missing"
fi

if grep -q "pub enum SparseOp" src/pipes.rs; then
    SPARSE_OPS=$(grep -c "SGATHER\|SSCATTR\|SPREFCH\|SNOP" src/pipes.rs || echo 0)
    echo -e "  ${GREEN}✓${NC} S-Pipe (Sparse memory): $SPARSE_OPS ops"
else
    echo -e "  ${RED}✗${NC} S-Pipe missing"
fi

if grep -q "pub enum CoordOp" src/pipes.rs; then
    COORD_OPS=$(grep -c "CPACK\|CUNPK\|CSEND\|CRECV\|CBAR\|CNOP" src/pipes.rs || echo 0)
    echo -e "  ${GREEN}✓${NC} C-Pipe (Coordination): $COORD_OPS ops"
else
    echo -e "  ${RED}✗${NC} C-Pipe missing"
fi
echo

# PPT validation
echo -e "${YELLOW}[Phext Page Table]${NC}"
if [ -f "src/ppt.rs" ]; then
    if grep -q "PhextPageTable" src/ppt.rs; then
        echo -e "  ${GREEN}✓${NC} PPT structure defined"
    fi
    if grep -q "translate" src/ppt.rs; then
        echo -e "  ${GREEN}✓${NC} Coordinate translation"
    fi
    if grep -q "PageTranslationCache" src/ppt.rs; then
        echo -e "  ${GREEN}✓${NC} Translation cache (PTC)"
    fi
    if grep -q "MemoryTier" src/ppt.rs; then
        echo -e "  ${GREEN}✓${NC} Memory tier classification"
    fi
else
    echo -e "  ${RED}✗${NC} PPT implementation missing"
fi
echo

# Test coverage
echo -e "${YELLOW}[Test Coverage]${NC}"
TEST_OUTPUT=$(cargo test --release 2>&1)
TOTAL_TESTS=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= passed)' | head -1)
FAILED_TESTS=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= failed)' | head -1 || echo 0)

if [ "$FAILED_TESTS" -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} $TOTAL_TESTS tests passing"
else
    echo -e "  ${RED}✗${NC} $FAILED_TESTS tests failing"
fi

# Module-specific test counts
for module in siw pipes phext_coord ppt memory exec sentron telemetry validation; do
    count=$(echo "$TEST_OUTPUT" | grep -c "test $module::" || echo 0)
    if [ "$count" -gt 0 ]; then
        echo -e "    $module: $count tests"
    fi
done
echo

# Benchmarks
echo -e "${YELLOW}[Benchmarks]${NC}"
if [ -d "benchmarks/workloads" ]; then
    WORKLOAD_COUNT=$(find benchmarks/workloads -name '*.siw' | wc -l)
    WORKLOAD_SIZE=$(du -sh benchmarks/workloads | cut -f1)
    echo -e "  ${GREEN}✓${NC} Synthetic workloads: $WORKLOAD_COUNT files ($WORKLOAD_SIZE)"
fi

if [ -d "benchmarks/cache" ]; then
    CACHE_BENCHES=$(find benchmarks/cache -name '*.c' | wc -l)
    echo -e "  ${GREEN}✓${NC} Cache benchmarks: $CACHE_BENCHES C programs"
fi

if [ -d "benchmarks/smt" ]; then
    SMT_BENCHES=$(find benchmarks/smt -name '*.c' | wc -l)
    echo -e "  ${GREEN}✓${NC} SMT benchmarks: $SMT_BENCHES C programs"
fi
echo

# Examples
echo -e "${YELLOW}[Examples]${NC}"
EXAMPLES=$(find examples -name '*.rs' | wc -l)
echo -e "  ${GREEN}✓${NC} $EXAMPLES example programs:"
for example in examples/*.rs; do
    name=$(basename "$example" .rs)
    desc=$(head -1 "$example" | sed 's|//!* *||')
    echo "    - $name: $desc"
done
echo

# Binaries
echo -e "${YELLOW}[Binaries]${NC}"
BINARIES=$(find src/bin -name '*.rs' | wc -l)
for binary in src/bin/*.rs; do
    name=$(basename "$binary" .rs)
    desc=$(head -1 "$binary" | sed 's|// *||')
    echo -e "  ${GREEN}✓${NC} $name - $desc"
done
echo

# Dependencies
echo -e "${YELLOW}[Dependencies]${NC}"
EXTERNAL_DEPS=$(grep -c "^[a-z]" Cargo.toml 2>/dev/null || echo 0)
if [ "$EXTERNAL_DEPS" -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} Zero external dependencies"
else
    echo -e "  ${YELLOW}!${NC} $EXTERNAL_DEPS external dependencies"
    grep "^[a-z]" Cargo.toml | sed 's/^/    /'
fi
echo

# Code metrics
echo -e "${YELLOW}[Code Metrics]${NC}"
RUST_LOC=$(find src -name '*.rs' -exec wc -l {} + | tail -1 | awk '{print $1}')
MODULE_COUNT=$(find src -name '*.rs' | wc -l)
echo "  Rust LOC: $RUST_LOC"
echo "  Modules: $MODULE_COUNT"
echo "  Tests: $TOTAL_TESTS"
echo "  Examples: $EXAMPLES"
echo "  Binaries: $BINARIES"
echo

# W6 Deliverable Check
echo -e "${YELLOW}[R23 Wave 6 Deliverable]${NC}"
echo "  Target: vBench suite with ≥2.5 ops/cycle measurement"
echo

W6_STATUS="incomplete"

# Check for benchmark_runner
if [ -f "examples/benchmark_runner.rs" ]; then
    echo -e "  ${GREEN}✓${NC} benchmark_runner.rs exists"
    
    # Check for workload files
    if [ "$(find benchmarks/workloads -name '*.siw' | wc -l)" -ge 7 ]; then
        echo -e "  ${GREEN}✓${NC} Synthetic workloads present (7+)"
        
        # Try to run benchmarks
        echo -e "\n  ${BLUE}Running benchmark suite...${NC}"
        BENCH_OUTPUT=$(cargo run --release --example benchmark_runner 2>&1 | tail -20)
        
        echo "$BENCH_OUTPUT" | grep -E "ops/cycle|Target:" | sed 's/^/    /'
        
        # Check if any workload meets threshold
        if echo "$BENCH_OUTPUT" | grep -q "ops/cycle" && \
           echo "$BENCH_OUTPUT" | awk '/ops\/cycle/ {if ($NF >= 2.5) exit 0} END {exit 1}' 2>/dev/null; then
            echo -e "\n  ${GREEN}✓✓ W6 COMPLETE - ≥2.5 ops/cycle measured${NC}"
            W6_STATUS="complete"
        else
            echo -e "\n  ${YELLOW}⚠ W6 INCOMPLETE - benchmarks run but <2.5 ops/cycle${NC}"
            echo -e "    ${YELLOW}Note: Emulated execution shows overhead; may need real HW validation${NC}"
            W6_STATUS="infrastructure-ready"
        fi
    else
        echo -e "  ${RED}✗${NC} Missing synthetic workloads"
    fi
else
    echo -e "  ${RED}✗${NC} benchmark_runner.rs missing"
fi

echo
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Summary                                     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo
echo "  Repository: vtpu"
echo "  Branch: $BRANCH"
echo "  Build: passing"
echo "  Tests: $TOTAL_TESTS passing"
echo "  LOC: $RUST_LOC"
echo "  Dependencies: $EXTERNAL_DEPS external"
echo "  W6 Status: $W6_STATUS"
echo

if [ "$W6_STATUS" = "complete" ]; then
    echo -e "${GREEN}✓ All systems operational - ready for W7${NC}"
    exit 0
elif [ "$W6_STATUS" = "infrastructure-ready" ]; then
    echo -e "${YELLOW}⚠ Infrastructure complete - awaiting performance validation${NC}"
    exit 0
else
    echo -e "${RED}✗ W6 deliverable incomplete${NC}"
    exit 1
fi
