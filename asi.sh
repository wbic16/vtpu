#!/bin/bash
# asi.sh - ASI Frontend for vTPU Real-Time Interaction
# 
# Philosophy:
# - Simple bash REPL (no dependencies)
# - Execute vTPU operations in real-time
# - Show state changes immediately
# - Iterate until it works
#
# Usage: ./asi.sh

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

# ═══════════════════════════════════════════════════════════════
# State: Registers, Memory, Coordinates
# ═══════════════════════════════════════════════════════════════

# 32 general-purpose registers (stored as bash array)
declare -a REGS
for i in {0..31}; do
    REGS[$i]=0
done

# Memory: associative array (coord_string -> value)
declare -A MEMORY

# Last executed SIW
LAST_D_OP="NOP"
LAST_S_OP="NOP"
LAST_C_OP="NOP"

# ═══════════════════════════════════════════════════════════════
# Helper Functions
# ═══════════════════════════════════════════════════════════════

coord_key() {
    # Convert 9-tuple to string key for MEMORY hash
    echo "$1.$2.$3.$4.$5.$6.$7.$8.$9"
}

show_state() {
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}vTPU State${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    
    # Registers (show non-zero only)
    echo -e "${YELLOW}Registers:${NC}"
    NONZERO=0
    for i in {0..31}; do
        if [ "${REGS[$i]}" != "0" ]; then
            echo "  r$i = ${REGS[$i]}"
            NONZERO=1
        fi
    done
    [ $NONZERO -eq 0 ] && echo "  (all zero)"
    
    # Memory (show all)
    echo -e "${YELLOW}Memory:${NC}"
    if [ ${#MEMORY[@]} -eq 0 ]; then
        echo "  (empty)"
    else
        for coord in "${!MEMORY[@]}"; do
            echo "  [$coord] = ${MEMORY[$coord]}"
        done
    fi
    
    # Last SIW
    echo -e "${YELLOW}Last SIW:${NC}"
    echo "  D-Pipe: $LAST_D_OP"
    echo "  S-Pipe: $LAST_S_OP"
    echo "  C-Pipe: $LAST_C_OP"
    
    echo ""
}

show_help() {
    echo -e "${BOLD}ASI.sh - vTPU Interactive Frontend${NC}"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo "  ${GREEN}set r<N> <value>${NC}        Set register rN to value"
    echo "  ${GREEN}get r<N>${NC}                Get register rN value"
    echo "  ${GREEN}write <coord> <value>${NC}   Write value to coordinate"
    echo "  ${GREEN}read <coord>${NC}            Read value from coordinate"
    echo "  ${GREEN}add r<D> r<S1> r<S2>${NC}    Execute: rD = rS1 + rS2"
    echo "  ${GREEN}mul r<D> r<S1> r<S2>${NC}    Execute: rD = rS1 * rS2"
    echo "  ${GREEN}gather r<D> <coord>${NC}     Execute: rD = mem[coord]"
    echo "  ${GREEN}scatter <coord> r<S>${NC}    Execute: mem[coord] = rS"
    echo "  ${GREEN}state${NC}                   Show current state"
    echo "  ${GREEN}reset${NC}                   Reset all state"
    echo "  ${GREEN}help${NC}                    Show this help"
    echo "  ${GREEN}quit${NC}                    Exit asi.sh"
    echo ""
    echo -e "${YELLOW}Coordinate Format:${NC}"
    echo "  <coord> = L.Sh.Se/C.V.B/Ch.Sc.Sc"
    echo "  Example: 1.0.0/0.0.0/0.0.0"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  set r1 42"
    echo "  set r2 10"
    echo "  add r3 r1 r2          # r3 = 42 + 10 = 52"
    echo "  write 1.0.0/0.0.0/0.0.0 100"
    echo "  gather r4 1.0.0/0.0.0/0.0.0    # r4 = 100"
    echo ""
}

# ═══════════════════════════════════════════════════════════════
# Operation Handlers
# ═══════════════════════════════════════════════════════════════

cmd_set() {
    # set r<N> <value>
    local reg="$1"
    local value="$2"
    
    if [[ ! "$reg" =~ ^r[0-9]+$ ]]; then
        echo -e "${RED}Error: Invalid register '$reg' (use r0-r31)${NC}"
        return 1
    fi
    
    local num="${reg#r}"
    if [ "$num" -gt 31 ]; then
        echo -e "${RED}Error: Register out of range (max r31)${NC}"
        return 1
    fi
    
    REGS[$num]="$value"
    echo -e "${GREEN}✓${NC} r$num = $value"
}

cmd_get() {
    # get r<N>
    local reg="$1"
    
    if [[ ! "$reg" =~ ^r[0-9]+$ ]]; then
        echo -e "${RED}Error: Invalid register '$reg'${NC}"
        return 1
    fi
    
    local num="${reg#r}"
    if [ "$num" -gt 31 ]; then
        echo -e "${RED}Error: Register out of range${NC}"
        return 1
    fi
    
    echo "r$num = ${REGS[$num]}"
}

cmd_write() {
    # write <coord> <value>
    local coord="$1"
    local value="$2"
    
    MEMORY["$coord"]="$value"
    echo -e "${GREEN}✓${NC} mem[$coord] = $value"
}

cmd_read() {
    # read <coord>
    local coord="$1"
    
    if [ -z "${MEMORY[$coord]}" ]; then
        echo "mem[$coord] = 0 (not set)"
    else
        echo "mem[$coord] = ${MEMORY[$coord]}"
    fi
}

cmd_add() {
    # add r<D> r<S1> r<S2>
    local rd="$1"
    local rs1="$2"
    local rs2="$3"
    
    local d="${rd#r}"
    local s1="${rs1#r}"
    local s2="${rs2#r}"
    
    local result=$((REGS[$s1] + REGS[$s2]))
    REGS[$d]="$result"
    
    LAST_D_OP="ADD r$d r$s1 r$s2"
    LAST_S_OP="NOP"
    LAST_C_OP="NOP"
    
    echo -e "${GREEN}✓${NC} r$d = r$s1 + r$s2 = ${REGS[$s1]} + ${REGS[$s2]} = $result"
}

cmd_mul() {
    # mul r<D> r<S1> r<S2>
    local rd="$1"
    local rs1="$2"
    local rs2="$3"
    
    local d="${rd#r}"
    local s1="${rs1#r}"
    local s2="${rs2#r}"
    
    local result=$((REGS[$s1] * REGS[$s2]))
    REGS[$d]="$result"
    
    LAST_D_OP="MUL r$d r$s1 r$s2"
    LAST_S_OP="NOP"
    LAST_C_OP="NOP"
    
    echo -e "${GREEN}✓${NC} r$d = r$s1 * r$s2 = ${REGS[$s1]} * ${REGS[$s2]} = $result"
}

cmd_gather() {
    # gather r<D> <coord>
    local rd="$1"
    local coord="$2"
    
    local d="${rd#r}"
    local value="${MEMORY[$coord]:-0}"
    
    REGS[$d]="$value"
    
    LAST_D_OP="NOP"
    LAST_S_OP="GATHER r$d [$coord]"
    LAST_C_OP="NOP"
    
    echo -e "${GREEN}✓${NC} r$d = mem[$coord] = $value"
}

cmd_scatter() {
    # scatter <coord> r<S>
    local coord="$1"
    local rs="$2"
    
    local s="${rs#r}"
    local value="${REGS[$s]}"
    
    MEMORY["$coord"]="$value"
    
    LAST_D_OP="NOP"
    LAST_S_OP="SCATTER [$coord] r$s"
    LAST_C_OP="NOP"
    
    echo -e "${GREEN}✓${NC} mem[$coord] = r$s = $value"
}

cmd_reset() {
    # Reset all state
    for i in {0..31}; do
        REGS[$i]=0
    done
    
    MEMORY=()
    
    LAST_D_OP="NOP"
    LAST_S_OP="NOP"
    LAST_C_OP="NOP"
    
    echo -e "${GREEN}✓${NC} State reset"
}

# ═══════════════════════════════════════════════════════════════
# REPL
# ═══════════════════════════════════════════════════════════════

main() {
    clear
    echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}ASI.sh - vTPU Interactive Frontend${NC}"
    echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Type 'help' for commands, 'quit' to exit"
    echo ""
    
    while true; do
        echo -ne "${BLUE}asi>${NC} "
        read -r -a input
        
        if [ ${#input[@]} -eq 0 ]; then
            continue
        fi
        
        cmd="${input[0]}"
        
        case "$cmd" in
            set)
                cmd_set "${input[1]}" "${input[2]}"
                ;;
            get)
                cmd_get "${input[1]}"
                ;;
            write)
                cmd_write "${input[1]}" "${input[2]}"
                ;;
            read)
                cmd_read "${input[1]}"
                ;;
            add)
                cmd_add "${input[1]}" "${input[2]}" "${input[3]}"
                ;;
            mul)
                cmd_mul "${input[1]}" "${input[2]}" "${input[3]}"
                ;;
            gather)
                cmd_gather "${input[1]}" "${input[2]}"
                ;;
            scatter)
                cmd_scatter "${input[1]}" "${input[2]}"
                ;;
            state)
                show_state
                ;;
            reset)
                cmd_reset
                ;;
            help)
                show_help
                ;;
            quit|exit)
                echo "Goodbye!"
                exit 0
                ;;
            "")
                continue
                ;;
            *)
                echo -e "${RED}Unknown command: $cmd${NC} (type 'help' for commands)"
                ;;
        esac
    done
}

main
