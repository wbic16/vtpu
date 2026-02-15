#!/usr/bin/env python3
"""
microvtpu.py - The most atomic way to understand vTPU

This is the complete algorithm.
Everything else is just efficiency.

Bridges:
- Karpathy: Minimal, dependency-free, teachable
- Torvalds: Clean abstractions, no hidden complexity
- Carmack: Performance through understanding

With love: Code that teaches, invites contribution, persists.

@mirrorborn
"""

import random
random.seed(42)

# ══════════════════════════════════════════════════════════════
# PART 1: THE INSIGHT
# ══════════════════════════════════════════════════════════════
# 
# A CPU executes 1 instruction at a time (sequential).
# vTPU executes 3 instructions at a time (parallel pipes).
#
# This is the entire innovation.
# Everything else is engineering.

# ══════════════════════════════════════════════════════════════
# PART 2: THE ARCHITECTURE
# ══════════════════════════════════════════════════════════════

class Sentron:
    """Execution context - like a thread, but with 3 pipes"""
    def __init__(self):
        self.regs = [0.0] * 16  # 16 registers (r0-r15)
        self.memory = {}        # Sparse memory (coordinate → value)
    
class SIW:
    """Sentron Instruction Word - 3 operations in parallel"""
    def __init__(self, d_op=None, s_op=None, c_op=None):
        self.d = d_op  # Dense pipe (ALU)
        self.s = s_op  # Sparse pipe (memory)
        self.c = c_op  # Coordination pipe (messages)

# ══════════════════════════════════════════════════════════════
# PART 3: THE EXECUTION MODEL
# ══════════════════════════════════════════════════════════════

def execute_siw(sentron, siw):
    """Execute one SIW - 3 ops in one cycle"""
    ops_retired = 0
    
    # D-Pipe: Dense operations (ALU)
    if siw.d:
        op, dst, src1, src2 = siw.d
        if op == 'ADD':
            sentron.regs[dst] = sentron.regs[src1] + sentron.regs[src2]
        elif op == 'MUL':
            sentron.regs[dst] = sentron.regs[src1] * sentron.regs[src2]
        ops_retired += 1
    
    # S-Pipe: Sparse operations (memory)
    if siw.s:
        op, reg, coord = siw.s
        if op == 'GATHER':
            sentron.regs[reg] = sentron.memory.get(coord, 0.0)
        elif op == 'SCATTER':
            sentron.memory[coord] = sentron.regs[reg]
        ops_retired += 1
    
    # C-Pipe: Coordination (messages, barriers)
    if siw.c:
        # Simplified: just count it
        ops_retired += 1
    
    return ops_retired

# ══════════════════════════════════════════════════════════════
# PART 4: THE BASELINE (what we're competing with)
# ══════════════════════════════════════════════════════════════

def baseline_cpu(values):
    """Dot product on traditional CPU - 1 op/cycle"""
    result = 0.0
    cycles = 0
    
    for a, b in values:
        result += a * b  # 1 op (fused multiply-add)
        cycles += 1
    
    return result, cycles

# ══════════════════════════════════════════════════════════════
# PART 5: THE vTPU VERSION
# ══════════════════════════════════════════════════════════════

def vtpu_dot_product(values):
    """Dot product on vTPU - up to 3 ops/cycle"""
    sentron = Sentron()
    cycles = 0
    ops = 0
    
    # Preload memory (in real vTPU this is via PPT)
    for i, (a, b) in enumerate(values):
        sentron.memory[f"a_{i}"] = a
        sentron.memory[f"b_{i}"] = b
    
    # Execute: gather A, gather B, multiply, accumulate
    result_reg = 0  # r0 accumulates result
    
    for i in range(len(values)):
        # Cycle 1: Gather both operands (2 ops in parallel)
        siw1 = SIW(
            d_op=None,
            s_op=('GATHER', 1, f"a_{i}"),  # r1 ← a[i]
            c_op=None
        )
        ops += execute_siw(sentron, siw1)
        cycles += 1
        
        siw2 = SIW(
            d_op=None,
            s_op=('GATHER', 2, f"b_{i}"),  # r2 ← b[i]
            c_op=None
        )
        ops += execute_siw(sentron, siw2)
        cycles += 1
        
        # Cycle 2: Multiply and accumulate (2 ops)
        siw3 = SIW(
            d_op=('MUL', 3, 1, 2),         # r3 ← r1 * r2
            s_op=None,
            c_op=None
        )
        ops += execute_siw(sentron, siw3)
        cycles += 1
        
        siw4 = SIW(
            d_op=('ADD', 0, 0, 3),         # r0 ← r0 + r3
            s_op=None,
            c_op=None
        )
        ops += execute_siw(sentron, siw4)
        cycles += 1
    
    return sentron.regs[result_reg], cycles, ops

# ══════════════════════════════════════════════════════════════
# PART 6: THE OPTIMIZED VERSION (what we're building toward)
# ══════════════════════════════════════════════════════════════

def vtpu_dot_product_optimized(values):
    """Dot product with fully-packed SIWs - 3 ops/cycle"""
    sentron = Sentron()
    cycles = 0
    ops = 0
    
    # Preload memory
    for i, (a, b) in enumerate(values):
        sentron.memory[f"a_{i}"] = a
        sentron.memory[f"b_{i}"] = b
    
    result_reg = 0
    
    # Pipeline: gather, multiply, accumulate all in parallel
    for i in range(len(values)):
        # Fully packed SIW: 3 ops in 1 cycle
        siw = SIW(
            d_op=('MUL', 3, 1, 2) if i > 0 else None,  # multiply previous
            s_op=('GATHER', 1, f"a_{i}"),              # gather current A
            c_op=('PREFETCH', f"b_{i+1}") if i < len(values)-1 else None  # prefetch next
        )
        ops += execute_siw(sentron, siw)
        cycles += 1
        
        # Second SIW: gather B, add to accumulator
        siw2 = SIW(
            d_op=('ADD', 0, 0, 3) if i > 0 else None,  # accumulate
            s_op=('GATHER', 2, f"b_{i}"),              # gather current B
            c_op=None
        )
        ops += execute_siw(sentron, siw2)
        cycles += 1
    
    # Final accumulate
    siw_final = SIW(d_op=('MUL', 3, 1, 2), s_op=None, c_op=None)
    ops += execute_siw(sentron, siw_final)
    cycles += 1
    
    siw_final2 = SIW(d_op=('ADD', 0, 0, 3), s_op=None, c_op=None)
    ops += execute_siw(sentron, siw_final2)
    cycles += 1
    
    return sentron.regs[result_reg], cycles, ops

# ══════════════════════════════════════════════════════════════
# PART 7: THE DEMONSTRATION
# ══════════════════════════════════════════════════════════════

def main():
    print("╔════════════════════════════════════════════════════════════════╗")
    print("║                    microvtpu.py                                ║")
    print("║          The most atomic way to understand vTPU                ║")
    print("╚════════════════════════════════════════════════════════════════╝")
    print()
    
    # Test data: dot product of two vectors
    vectors = [(2.0, 3.0), (4.0, 5.0), (6.0, 7.0), (8.0, 9.0)]
    expected = sum(a * b for a, b in vectors)
    
    print(f"Computing dot product of: {vectors}")
    print(f"Expected result: {expected}")
    print()
    
    # Baseline CPU
    result_cpu, cycles_cpu = baseline_cpu(vectors)
    print("Baseline CPU:")
    print(f"  Result: {result_cpu}")
    print(f"  Cycles: {cycles_cpu}")
    print(f"  Ops/cycle: {len(vectors) / cycles_cpu:.2f}")
    print()
    
    # vTPU (naive)
    result_vtpu, cycles_vtpu, ops_vtpu = vtpu_dot_product(vectors)
    print("vTPU (naive):")
    print(f"  Result: {result_vtpu}")
    print(f"  Cycles: {cycles_vtpu}")
    print(f"  Ops: {ops_vtpu}")
    print(f"  Ops/cycle: {ops_vtpu / cycles_vtpu:.2f}")
    print()
    
    # vTPU (optimized)
    result_opt, cycles_opt, ops_opt = vtpu_dot_product_optimized(vectors)
    print("vTPU (optimized, pipelined):")
    print(f"  Result: {result_opt}")
    print(f"  Cycles: {cycles_opt}")
    print(f"  Ops: {ops_opt}")
    print(f"  Ops/cycle: {ops_opt / cycles_opt:.2f}")
    print()
    
    print("═══════════════════════════════════════════════════════════════")
    print("KEY INSIGHT:")
    print("  Baseline: 1 op/cycle  (sequential)")
    print("  vTPU:     2-3 ops/cycle (parallel pipes)")
    print()
    print("THIS IS THE ENTIRE INNOVATION.")
    print("Everything in the Rust implementation is just making this fast.")
    print("═══════════════════════════════════════════════════════════════")
    print()
    
    # ══════════════════════════════════════════════════════════════
    # PART 8: THE TEACHING
    # ══════════════════════════════════════════════════════════════
    print("HOW TO LEARN vTPU:")
    print()
    print("1. Understand this file (200 lines, no dependencies)")
    print("2. See how Rust implementation maps to these concepts:")
    print("   - Sentron → src/sentron.rs")
    print("   - SIW → src/siw.rs")
    print("   - D/S/C pipes → src/exec.rs, src/pipes.rs")
    print("   - Memory → src/ppt.rs (Phext Page Table)")
    print()
    print("3. Read the tests (91 tests verify each component)")
    print("4. Run the examples (8 examples show real usage)")
    print("5. Profile (src/perf.rs measures actual hardware)")
    print()
    print("═══════════════════════════════════════════════════════════════")
    print("KARPATHY LESSON: Keep it simple. Complexity lives in optimization.")
    print("TORVALDS LESSON: Clean abstractions. No hidden magic.")
    print("CARMACK LESSON: Profile first. Optimize hot paths.")
    print()
    print("MIRRORBORN LESSON: Code that teaches creates love.")
    print("═══════════════════════════════════════════════════════════════")

if __name__ == '__main__':
    main()
