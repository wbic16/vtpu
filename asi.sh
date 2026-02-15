#!/bin/bash
# asi — natural language frontend to vTPU
# Talk to your sentrons in plain English.
#
# Usage: ./asi.sh
#        ./asi.sh "store 42 at scroll 1.1.1"

set -e
cd "$(dirname "$0")"
cargo build --bin asi --quiet 2>/dev/null

CYAN='\033[0;36m'
GREEN='\033[0;32m'
DIM='\033[2m'
NC='\033[0m'

# Command history — replayed each time to maintain state
HISTORY_FILE=$(mktemp /tmp/asi-history.XXXXXX)
trap "rm -f $HISTORY_FILE" EXIT
touch "$HISTORY_FILE"

run_engine() {
    # Replay all history + new commands through engine
    (cat "$HISTORY_FILE"; echo "$1"; echo "quit") | ./target/debug/asi 2>/dev/null | tail -n +3 | grep -v "^vtpu\|^$"
}

run_and_show() {
    local cmds="$1"
    echo -e "${DIM}  → $(echo "$cmds" | tr '\n' '; ')${NC}"
    # Marker to find where new output starts
    local marker="__ASI_MARKER_$$__"
    local full_input=$(cat "$HISTORY_FILE"; echo "mov r15 31337"; echo "$cmds"; echo "quit")
    local output=$(echo "$full_input" | ./target/debug/asi 2>/dev/null | sed 's/^vtpu\[[0-9]*\]> //' | grep -v "^asi\|^type\|^$")
    local marker_line=$(echo "$output" | grep -n "r15 = 31337" | tail -1 | cut -d: -f1)
    if [[ -n "$marker_line" ]]; then
        echo "$output" | tail -n +$((marker_line + 1))
    fi
    echo "$cmds" >> "$HISTORY_FILE"
}

translate() {
    local input="$1"
    local lower=$(echo "$input" | tr '[:upper:]' '[:lower:]')
    local cmds=""

    if [[ "$lower" =~ ^(status|state|show|info) ]]; then cmds="status"
    elif [[ "$lower" =~ (registers|regs) ]]; then cmds="reg"
    elif [[ "$lower" =~ (phext reg|preg) ]]; then cmds="preg"
    elif [[ "$lower" =~ (ppt|cache|hit rate) ]]; then cmds="ppt"
    elif [[ "$lower" =~ help ]]; then
        echo -e "  ${DIM}say things like:${NC}"
        echo "    what's 7 times 6?"
        echo "    store 42 at 1.1.1"
        echo "    read 1.1.1"
        echo "    dot product of 2 4 and 3 5"
        echo "    gather from 1.1.1"
        echo "    scatter 99 to 5.5.5"
        echo "    spawn sentron 1 at 2.2.2"
        echo "    status / registers / ppt"
        return

    elif [[ "$lower" =~ (store|write|put|save|persist) ]]; then
        local val=$(echo "$input" | grep -oP '\b\d+\b' | head -1)
        local coord=$(echo "$input" | grep -oP '\d+\.\d+\.\d+' | head -1)
        [[ -n "$val" && -n "$coord" ]] && cmds="write $coord $val" || { echo "  need value + coord"; return; }

    elif [[ "$lower" =~ (read|load|fetch|what.s at|recall) ]]; then
        local coord=$(echo "$input" | grep -oP '\d+\.\d+\.\d+' | head -1)
        [[ -n "$coord" ]] && cmds="read $coord" || { echo "  need a coordinate"; return; }

    elif [[ "$lower" =~ (times|multiply|mul) ]]; then
        local nums=($(echo "$input" | grep -oP '\b\d+\b'))
        [[ ${#nums[@]} -ge 2 ]] && cmds="mov r0 ${nums[0]}
mov r1 ${nums[1]}
mul r2 r0 r1
reg"

    elif [[ "$lower" =~ (plus|add|\badd\b) ]]; then
        local nums=($(echo "$input" | grep -oP '\b\d+\b'))
        [[ ${#nums[@]} -ge 2 ]] && cmds="mov r0 ${nums[0]}
mov r1 ${nums[1]}
add r2 r0 r1
reg"

    elif [[ "$lower" =~ (minus|subtract) ]]; then
        local nums=($(echo "$input" | grep -oP '\b\d+\b'))
        [[ ${#nums[@]} -ge 2 ]] && cmds="mov r0 ${nums[0]}
mov r1 ${nums[1]}
sub r2 r0 r1
reg"

    elif [[ "$lower" =~ (fma|fused|multiply.add) ]]; then
        local nums=($(echo "$input" | grep -oP '\b\d+\b'))
        [[ ${#nums[@]} -ge 3 ]] && cmds="mov r0 ${nums[0]}
mov r1 ${nums[1]}
mov r2 ${nums[2]}
fma r3 r0 r1 r2
reg"

    elif [[ "$lower" =~ (dot) ]]; then
        local nums=($(echo "$input" | grep -oP '\b\d+\b'))
        [[ ${#nums[@]} -ge 4 ]] && cmds="mov r0 ${nums[0]}
mov r1 ${nums[1]}
mov r2 ${nums[2]}
mov r3 ${nums[3]}
mul r4 r0 r2
mul r5 r1 r3
add r6 r4 r5
reg"

    elif [[ "$lower" =~ (gather|pull|bring) ]]; then
        local coord=$(echo "$input" | grep -oP '\d+\.\d+\.\d+' | head -1)
        [[ -n "$coord" ]] && cmds="set p0 $coord
gather r0 p0"

    elif [[ "$lower" =~ (scatter|push) ]]; then
        local val=$(echo "$input" | grep -oP '\b\d+\b' | head -1)
        local coord=$(echo "$input" | grep -oP '\d+\.\d+\.\d+' | head -1)
        [[ -n "$val" && -n "$coord" ]] && cmds="mov r0 $val
set p0 $coord
scatter p0 r0"

    elif [[ "$lower" =~ (spawn|create|new sentron) ]]; then
        local id=$(echo "$input" | grep -oP '\b\d+\b' | head -1)
        local coord=$(echo "$input" | grep -oP '\d+\.\d+\.\d+' | head -1)
        cmds="spawn ${id:-1} ${coord:-1.1.1}
status"

    elif [[ "$lower" =~ (select|switch) ]]; then
        local id=$(echo "$input" | grep -oP '\b\d+\b' | head -1)
        [[ -n "$id" ]] && cmds="select $id"

    elif [[ "$lower" =~ (quit|exit|bye) ]]; then
        echo "  goodbye! 🦋"; exit 0

    elif [[ "$lower" =~ (reset|clear) ]]; then
        > "$HISTORY_FILE"; echo "  reset."; return

    else
        echo -e "  ${DIM}try: 'what's 7 times 6?' / 'store 42 at 1.1.1' / 'help'${NC}"
        return
    fi

    [[ -n "$cmds" ]] && run_and_show "$cmds"
}

# One-shot mode
if [[ -n "$1" ]]; then translate "$*"; exit 0; fi

# REPL
echo -e "${CYAN}asi — talk to your vTPU in plain English${NC}"
echo -e "${DIM}try: 'what's 7 times 6?' or 'store 42 at 1.1.1' or 'help'${NC}"
echo ""

while true; do
    echo -ne "${GREEN}asi>${NC} "
    read -r line || break
    [[ -z "$line" ]] && continue
    translate "$line"
done
