#!/bin/bash
# asi — vTPU interactive frontend
# Builds (if needed) and launches the Rust REPL (real vTPU engine, not a simulation)
# Enhanced with quick commands: bench, demo, status

set -e
cd "$(dirname "$0")"

GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

case "${1:-repl}" in
    bench|benchmark)
        echo -e "${CYAN}[bench]${NC} Running microvtpu example..."
        cargo build --release --examples --quiet
        ./target/release/examples/microvtpu
        ;;
    demo)
        echo -e "${CYAN}[demo]${NC} Side-by-side: Python vs Rust"
        echo ""
        echo "Python (microvtpu.py):"
        python3 microvtpu.py | tail -20
        echo ""
        echo "Rust (microvtpu example):"
        cargo build --release --examples --quiet
        ./target/release/examples/microvtpu | tail -15
        ;;
    status)
        echo -e "${CYAN}[status]${NC} vTPU Status"
        echo ""
        echo "Build:"
        cargo build --bin asi --quiet 2>/dev/null && echo -e "  ${GREEN}✓${NC} asi REPL ready" || echo "  Building..."
        [ -f target/release/examples/microvtpu ] && echo -e "  ${GREEN}✓${NC} examples ready" || echo "  Examples: cargo build --release --examples"
        echo ""
        echo "Hardware:"
        echo "  CPU: $(lscpu | grep "Model name" | sed 's/Model name: *//' | xargs)"
        echo "  Cores: $(nproc)"
        echo ""
        echo "Repository:"
        git log -1 --oneline 2>/dev/null
        ;;
    help|-h|--help)
        echo "asi — vTPU interactive frontend"
        echo ""
        echo "Usage: ./asi.sh [command]"
        echo ""
        echo "Commands:"
        echo "  (none)    → Interactive REPL (Rust)"
        echo "  bench     → Quick benchmark (microvtpu example)"
        echo "  demo      → Side-by-side Python vs Rust"
        echo "  status    → System status"
        echo "  help      → This message"
        echo ""
        echo "REPL commands (run ./asi.sh for interactive mode):"
        echo "  mov, add, mul, sub, fma, gather, scatter"
        echo "  spawn, select, status, write, read, ppt"
        echo "  Type 'help' in REPL for full command list"
        ;;
    repl|*)
        cargo build --bin asi --quiet 2>/dev/null
        exec ./target/debug/asi "$@"
        ;;
esac
