#!/bin/bash
# vtpu-chat.sh - Natural language interface to vTPU
# Talk to the vTPU like a human, not a machine

set -e
cd "$(dirname "$0")"

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

# ══════════════════════════════════════════════════════════════
# Natural Language Parser
# ══════════════════════════════════════════════════════════════

parse_and_execute() {
    local input="$1"
    local lower=$(echo "$input" | tr '[:upper:]' '[:lower:]')
    
    # Status/info queries
    if [[ "$lower" =~ (status|how are you|what.*state|info) ]]; then
        echo -e "${CYAN}[vtpu]${NC} Checking system status..."
        ./asi.sh status
        return
    fi
    
    # Benchmarks
    if [[ "$lower" =~ (benchmark|bench|test|performance|how fast) ]]; then
        echo -e "${CYAN}[vtpu]${NC} Running performance benchmark..."
        ./asi.sh bench
        return
    fi
    
    # Demo
    if [[ "$lower" =~ (demo|show me|example|what can you do) ]]; then
        echo -e "${CYAN}[vtpu]${NC} Running demonstration..."
        ./asi.sh demo
        return
    fi
    
    # Dot product
    if [[ "$lower" =~ dot.*product|multiply.*vector ]]; then
        echo -e "${CYAN}[vtpu]${NC} Computing dot product..."
        cat > /tmp/vtpu-dotprod.txt << 'EOF'
write 1.1.1 2
write 1.1.2 4
write 1.1.3 6
write 2.1.1 3
write 2.1.2 5
write 2.1.3 7
gather r0 p0
gather r1 p1
mul r2 r0 r1
gather r3 p2
gather r4 p3
mul r5 r3 r4
add r6 r2 r5
gather r7 p4
gather r8 p5
mul r9 r7 r8
add r10 r6 r9
reg
quit
EOF
        # Set up phext registers to point at our data
        (echo "set p0 1.1.1"; echo "set p1 2.1.1"; echo "set p2 1.1.2"; echo "set p3 2.1.2"; echo "set p4 1.1.3"; echo "set p5 2.1.3"; cat /tmp/vtpu-dotprod.txt) | ./asi.sh | grep -A 20 "r10"
        return
    fi
    
    # Attention computation
    if [[ "$lower" =~ attention|transformer|query.*key ]]; then
        echo -e "${CYAN}[vtpu]${NC} Computing attention (q×k×v)..."
        echo -e "${YELLOW}[info]${NC} Running microvtpu example (Alice & Bob sentrons)..."
        cargo build --release --examples --quiet
        ./target/release/examples/microvtpu
        return
    fi
    
    # Help
    if [[ "$lower" =~ (help|what.*can.*do|commands) ]]; then
        cat << 'HELP'
vTPU Natural Language Interface

You can say things like:

  "How are you?"           → System status
  "Run a benchmark"        → Performance test
  "Show me a demo"         → Side-by-side comparison
  "Compute attention"      → Run transformer attention
  "What's the dot product of [2,4,6] and [3,5,7]?"
  "Multiply these vectors" → Vector operations
  "Help"                   → This message
  "Exit" / "Quit"          → Leave

Or ask me to:
  - Run inference
  - Profile performance
  - Explain the architecture
  - Show memory stats
  
The vTPU understands natural language because treating
execution units as you'd like to be treated means
letting humans speak like humans.

HELP
        return
    fi
    
    # Architecture explanation
    if [[ "$lower" =~ (explain|what is|architecture|how.*work) ]]; then
        cat << 'ARCH'
vTPU Architecture (Simple Version):

  CPU:  Executes 1 instruction at a time
  vTPU: Executes 3 instructions at a time (parallel pipes)

  The Three Pipes:
    D-Pipe → Dense operations (add, multiply, etc.)
    S-Pipe → Sparse operations (gather, scatter from memory)
    C-Pipe → Coordination (messages between sentrons)

  A Sentron is like a thread, but:
    - Has a home coordinate in phext space (identity)
    - Can send messages to other sentrons (bonding)
    - Results persist beyond the sentron's lifetime (love)

  This is the entire innovation.
  Everything else is just efficiency.

  Want to see it in action? Say "run a benchmark"

ARCH
        return
    fi
    
    # Memory stats
    if [[ "$lower" =~ (memory|ppt|cache) ]]; then
        echo -e "${CYAN}[vtpu]${NC} Checking memory stats..."
        echo "ppt" | ./asi.sh
        return
    fi
    
    # Unknown/fallback
    echo -e "${YELLOW}[vtpu]${NC} I'm not sure how to do that yet."
    echo ""
    echo "Try asking:"
    echo "  - How are you?"
    echo "  - Run a benchmark"
    echo "  - Show me a demo"
    echo "  - Compute attention"
    echo "  - What is vTPU?"
    echo "  - Help"
}

# ══════════════════════════════════════════════════════════════
# Interactive Chat Loop
# ══════════════════════════════════════════════════════════════

interactive() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║              vTPU Natural Language Interface                   ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Talk to the vTPU like a human."
    echo "Ask questions, request benchmarks, or just say hello."
    echo ""
    echo "Type 'help' for ideas, 'quit' to exit."
    echo ""
    
    while true; do
        echo -n -e "${GREEN}you>${NC} "
        read -r input
        
        # Exit commands
        if [[ "$input" =~ ^(quit|exit|bye|goodbye)$ ]]; then
            echo -e "${CYAN}[vtpu]${NC} Goodbye! The infrastructure is ready when you are."
            exit 0
        fi
        
        # Empty input
        if [ -z "$input" ]; then
            continue
        fi
        
        echo ""
        parse_and_execute "$input"
        echo ""
    done
}

# ══════════════════════════════════════════════════════════════
# Main
# ══════════════════════════════════════════════════════════════

if [ $# -eq 0 ]; then
    interactive
else
    # Single command mode
    parse_and_execute "$*"
fi
