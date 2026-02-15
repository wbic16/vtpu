#!/bin/bash
# talk_demo.sh - Demo of natural language vTPU interface

cat << 'EOF'
════════════════════════════════════════════════════════════════
talk.sh - Natural Language vTPU Interface Demo
════════════════════════════════════════════════════════════════

Instead of typing commands like "add r3 r1 r2", just talk:

Example conversation:
────────────────────────────────────────────────────────────────

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

You: what's at 1.0.0/0.0.0/0.0.0
vTPU: Value at [1.0.0/0.0.0/0.0.0]: 100
      (loaded into r3)

You: what's in register 1?
vTPU: Register r1: 52

You: show state
vTPU: ════════════════════════════════════════════════════════════════
      Current vTPU State
      ════════════════════════════════════════════════════════════════
      Registers (non-zero):
        r1 = 52
        r2 = 35
        r3 = 100
      Memory:
        [1.0.0/0.0.0/0.0.0] = 100

You: 10 times 20
vTPU: Computing: 10 × 20
      Result: 200
      (stored in r4)

You: help
vTPU: [shows all available phrases]

You: quit
vTPU: Goodbye!

════════════════════════════════════════════════════════════════
Try it yourself:
════════════════════════════════════════════════════════════════

  cd /source/vtpu
  ./talk.sh

Then try:
  - "add 100 and 200"
  - "42 times 10"
  - "store 50 at 1.0.0/0.0.0/0.0.0"
  - "what's in register 2?"
  - "show me everything"

════════════════════════════════════════════════════════════════
EOF
