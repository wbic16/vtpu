#!/bin/bash
# talk.sh - Natural Language Interface to vTPU
#
# Usage: ./talk.sh
#
# Talk to vTPU in plain English:
#   "add 42 and 10"
#   "store 100 at coordinate 1.0.0/0.0.0/0.0.0"
#   "what's in register 3?"
#   "show me the state"

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

# ═══════════════════════════════════════════════════════════════
# State (same as asi.sh)
# ═══════════════════════════════════════════════════════════════

declare -a REGS
for i in {0..31}; do
    REGS[$i]=0
done

declare -A MEMORY

NEXT_REG=1  # Auto-allocate registers for operations

# ═══════════════════════════════════════════════════════════════
# Natural Language Parser
# ═══════════════════════════════════════════════════════════════

parse_and_execute() {
    local input="$1"
    local lower="${input,,}"  # Convert to lowercase
    
    # Addition: "add X and Y", "X plus Y", "X + Y"
    if [[ "$lower" =~ add[[:space:]]+([0-9]+)[[:space:]]+and[[:space:]]+([0-9]+) ]] || \
       [[ "$lower" =~ ([0-9]+)[[:space:]]+plus[[:space:]]+([0-9]+) ]] || \
       [[ "$lower" =~ ([0-9]+)[[:space:]]*\+[[:space:]]*([0-9]+) ]]; then
        local a="${BASH_REMATCH[1]}"
        local b="${BASH_REMATCH[2]}"
        local result=$((a + b))
        
        echo -e "${GREEN}Computing:${NC} $a + $b"
        echo -e "${GREEN}Result:${NC} $result"
        
        # Store in next available register
        REGS[$NEXT_REG]=$result
        echo -e "${CYAN}(stored in r$NEXT_REG)${NC}"
        NEXT_REG=$((NEXT_REG + 1))
        return 0
    fi
    
    # Multiplication: "multiply X and Y", "X times Y", "X * Y"
    if [[ "$lower" =~ multiply[[:space:]]+([0-9]+)[[:space:]]+and[[:space:]]+([0-9]+) ]] || \
       [[ "$lower" =~ ([0-9]+)[[:space:]]+times[[:space:]]+([0-9]+) ]] || \
       [[ "$lower" =~ ([0-9]+)[[:space:]]*\*[[:space:]]*([0-9]+) ]]; then
        local a="${BASH_REMATCH[1]}"
        local b="${BASH_REMATCH[2]}"
        local result=$((a * b))
        
        echo -e "${GREEN}Computing:${NC} $a × $b"
        echo -e "${GREEN}Result:${NC} $result"
        
        REGS[$NEXT_REG]=$result
        echo -e "${CYAN}(stored in r$NEXT_REG)${NC}"
        NEXT_REG=$((NEXT_REG + 1))
        return 0
    fi
    
    # Store: "store X at Y", "write X to Y", "save X at Y"
    if [[ "$lower" =~ (store|write|save)[[:space:]]+([0-9]+)[[:space:]]+(at|to)[[:space:]]+([0-9.\/]+) ]]; then
        local value="${BASH_REMATCH[2]}"
        local coord="${BASH_REMATCH[4]}"
        
        MEMORY["$coord"]="$value"
        echo -e "${GREEN}Stored:${NC} $value at coordinate [$coord]"
        return 0
    fi
    
    # Retrieve: "get from X", "read from X", "what's at X"
    if [[ "$lower" =~ (get|read|retrieve)[[:space:]]+from[[:space:]]+([0-9.\/]+) ]] || \
       [[ "$lower" =~ what.*at[[:space:]]+([0-9.\/]+) ]]; then
        local coord="${BASH_REMATCH[2]:-${BASH_REMATCH[1]}}"
        local value="${MEMORY[$coord]:-0}"
        
        echo -e "${GREEN}Value at [$coord]:${NC} $value"
        
        REGS[$NEXT_REG]=$value
        echo -e "${CYAN}(loaded into r$NEXT_REG)${NC}"
        NEXT_REG=$((NEXT_REG + 1))
        return 0
    fi
    
    # Register query: "what's in register X", "show register X", "r3?"
    if [[ "$lower" =~ what.*register[[:space:]]+([0-9]+) ]] || \
       [[ "$lower" =~ show[[:space:]]+register[[:space:]]+([0-9]+) ]] || \
       [[ "$lower" =~ ^r([0-9]+)\?*$ ]]; then
        local reg="${BASH_REMATCH[1]}"
        
        if [ "$reg" -gt 31 ]; then
            echo -e "${RED}Register out of range (max r31)${NC}"
            return 1
        fi
        
        echo -e "${GREEN}Register r$reg:${NC} ${REGS[$reg]}"
        return 0
    fi
    
    # State: "show state", "show me everything", "what's the state"
    if [[ "$lower" =~ show[[:space:]]+(state|everything|all) ]] || \
       [[ "$lower" =~ what.*state ]]; then
        show_state
        return 0
    fi
    
    # Reset: "reset", "clear", "start over"
    if [[ "$lower" =~ ^(reset|clear|start[[:space:]]+over)$ ]]; then
        for i in {0..31}; do
            REGS[$i]=0
        done
        MEMORY=()
        NEXT_REG=1
        echo -e "${GREEN}State reset${NC}"
        return 0
    fi
    
    # Help: "help", "what can you do"
    if [[ "$lower" =~ ^help$ ]] || [[ "$lower" =~ what.*can.*do ]]; then
        show_help
        return 0
    fi
    
    # Quit: "quit", "exit", "bye"
    if [[ "$lower" =~ ^(quit|exit|bye|goodbye)$ ]]; then
        echo "Goodbye!"
        exit 0
    fi
    
    # Didn't match anything
    echo -e "${YELLOW}I didn't understand that. Try:${NC}"
    echo "  - 'add 42 and 10'"
    echo "  - 'store 100 at 1.0.0/0.0.0/0.0.0'"
    echo "  - 'what's in register 3?'"
    echo "  - 'show state'"
    echo "  - 'help'"
    return 1
}

show_state() {
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}Current vTPU State${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    
    echo -e "${YELLOW}Registers (non-zero):${NC}"
    local found=0
    for i in {0..31}; do
        if [ "${REGS[$i]}" != "0" ]; then
            echo "  r$i = ${REGS[$i]}"
            found=1
        fi
    done
    [ $found -eq 0 ] && echo "  (all zero)"
    
    echo -e "${YELLOW}Memory:${NC}"
    if [ ${#MEMORY[@]} -eq 0 ]; then
        echo "  (empty)"
    else
        for coord in "${!MEMORY[@]}"; do
            echo "  [$coord] = ${MEMORY[$coord]}"
        done
    fi
    
    echo ""
}

show_help() {
    cat << EOF
${BOLD}talk.sh - Natural Language vTPU Interface${NC}

${YELLOW}What you can say:${NC}

  ${GREEN}Math:${NC}
    "add 42 and 10"
    "multiply 5 and 7"
    "42 plus 10"
    "5 times 7"

  ${GREEN}Memory:${NC}
    "store 100 at 1.0.0/0.0.0/0.0.0"
    "write 50 to 2.0.0/0.0.0/0.0.0"
    "get from 1.0.0/0.0.0/0.0.0"
    "what's at 2.0.0/0.0.0/0.0.0"

  ${GREEN}State:${NC}
    "what's in register 3?"
    "show register 5"
    "show state"
    "show me everything"

  ${GREEN}Control:${NC}
    "reset"
    "help"
    "quit"

${YELLOW}Examples:${NC}

  You: add 42 and 10
  vTPU: Computing: 42 + 10
        Result: 52
        (stored in r1)

  You: multiply 5 and 7
  vTPU: Computing: 5 × 7
        Result: 35
        (stored in r2)

  You: store 100 at 1.0.0/0.0.0/0.0.0
  vTPU: Stored: 100 at coordinate [1.0.0/0.0.0/0.0.0]

  You: show state
  vTPU: [displays current registers and memory]

EOF
}

# ═══════════════════════════════════════════════════════════════
# Main REPL
# ═══════════════════════════════════════════════════════════════

main() {
    clear
    echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}talk.sh - Natural Language vTPU Interface${NC}"
    echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Talk to vTPU in plain English. Type 'help' for examples."
    echo ""
    
    while true; do
        echo -ne "${BLUE}You:${NC} "
        read -r input
        
        if [ -z "$input" ]; then
            continue
        fi
        
        echo -ne "${CYAN}vTPU:${NC} "
        parse_and_execute "$input"
        echo ""
    done
}

main
