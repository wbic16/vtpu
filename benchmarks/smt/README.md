# SMT (Simultaneous Multithreading) Analysis - R23 Wave 5

## Goal

Measure SMT efficiency on AMD Zen 4 with complementary vTPU workloads.

Target: 1.9x speedup with 2 threads on same core (vs 2.0x theoretical max).

## Benchmarks

### 1. `single_thread.c` - Baseline Performance
Measures single-thread throughput (ops/sec, cycles/op).

### 2. `dual_thread.c` - SMT Performance
Runs two threads on the same physical core, measures total throughput.

### 3. `complementary_workloads.c` - D/S/C Pairing
Tests complementary workload pairs:
- Thread 1: D-heavy (80% ALU)
- Thread 2: S-heavy (80% memory)
- Goal: Minimize execution port conflicts

## Running

```bash
make
./single_thread
./dual_thread
./complementary_workloads
```

## Expected Results

- Single-thread: ~1.2 ops/cycle baseline
- Dual-thread (same workload): ~1.4x speedup (resource contention)
- Dual-thread (complementary): ~1.9x speedup (minimal contention)
- SMT efficiency: 95% (1.9/2.0)
