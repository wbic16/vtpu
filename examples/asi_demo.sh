#!/bin/bash
# asi_demo.sh - Demo script showing asi.sh usage
#
# Run: ./examples/asi_demo.sh

cat << 'EOF'
════════════════════════════════════════════════════════════════
ASI.sh Demo - Interactive vTPU Frontend
════════════════════════════════════════════════════════════════

This demo shows how to use asi.sh for real-time vTPU interaction.

Example session:
────────────────────────────────────────────────────────────────

asi> help
  (shows all commands)

asi> set r1 42
✓ r1 = 42

asi> set r2 10
✓ r2 = 10

asi> add r3 r1 r2
✓ r3 = r1 + r2 = 42 + 10 = 52

asi> state
════════════════════════════════════════════════════════════════
vTPU State
════════════════════════════════════════════════════════════════
Registers:
  r1 = 42
  r2 = 10
  r3 = 52
Memory:
  (empty)
Last SIW:
  D-Pipe: ADD r3 r1 r2
  S-Pipe: NOP
  C-Pipe: NOP

asi> write 1.0.0/0.0.0/0.0.0 100
✓ mem[1.0.0/0.0.0/0.0.0] = 100

asi> gather r4 1.0.0/0.0.0/0.0.0
✓ r4 = mem[1.0.0/0.0.0/0.0.0] = 100

asi> mul r5 r3 r4
✓ r5 = r3 * r4 = 52 * 100 = 5200

asi> state
════════════════════════════════════════════════════════════════
vTPU State
════════════════════════════════════════════════════════════════
Registers:
  r1 = 42
  r2 = 10
  r3 = 52
  r4 = 100
  r5 = 5200
Memory:
  [1.0.0/0.0.0/0.0.0] = 100
Last SIW:
  D-Pipe: MUL r5 r3 r4
  S-Pipe: NOP
  C-Pipe: NOP

asi> scatter 2.0.0/0.0.0/0.0.0 r5
✓ mem[2.0.0/0.0.0/0.0.0] = r5 = 5200

asi> quit
Goodbye!

════════════════════════════════════════════════════════════════
Try it yourself:
════════════════════════════════════════════════════════════════

  cd /source/vtpu
  ./asi.sh

Then experiment with:
  - set/get registers
  - write/read coordinates
  - add/mul operations
  - gather/scatter memory ops

════════════════════════════════════════════════════════════════
EOF
