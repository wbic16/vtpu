# vTPU Cache Benchmarks - Wave 2

**Purpose**: Establish baseline cache performance metrics (KPIs #2, #3, #4)

## Benchmarks

### 1. `sequential_access.c` - Cache Tier Sweep
Tests L1/L2/L3/RAM access latency with sequential memory access patterns.

**What it measures**:
- L1 hit rate (target: 95%)
- L2 hit rate (target: 85%)
- L3 hit rate (target: 70%)
- Memory bandwidth (target: 85 GB/s)

**How it works**:
- Sweeps through array sizes: 8 KB → 128 MB
- Sequential read pattern (high cache locality)
- Times total access latency
- Reports latency per access across different array sizes

### 2. `random_access.c` - Cache Pressure Test
Tests cache performance under random access patterns (worst-case scenario).

**What it measures**:
- Cache miss rate under pressure
- Random access latency
- TLB performance
- Memory subsystem resilience

**How it works**:
- Random access pattern (low cache locality)
- Multiple array sizes
- Pointer chasing to prevent prefetching
- Reports latency distribution

### 3. `coordinate_patterns.c` - Dimensional Locality
Tests phext coordinate access patterns (dimensional locality hypothesis).

**What it measures**:
- Same-scroll vs cross-scroll access latency
- Same-section vs cross-section latency
- Dimensional locality correlation with cache performance

**How it works**:
- Simulates phext coordinate navigation
- Tests locality at different dimensional boundaries
- Validates "coordinate locality = memory locality" hypothesis

## How to Run

### Quick Start
```bash
# Compile all benchmarks
make

# Run all benchmarks (no profiling)
make run
```

### With Performance Counters (Linux only)
```bash
# Profile with perf stat
make profile

# Or manually for detailed output
perf stat -e cache-references,cache-misses,L1-dcache-loads,L1-dcache-load-misses,cycles,instructions ./sequential_access
```

### Manual Execution
```bash
# Compile
gcc -O2 -Wall -march=native -std=c11 -o sequential_access sequential_access.c -lm

# Run
./sequential_access

# Profile
perf stat -e cache-references,cache-misses,cycles,instructions ./sequential_access
```

## Expected Output

### Sequential Access
```
Array size: 8 KB, Avg latency: 2.5 ns (L1 hit)
Array size: 64 KB, Avg latency: 3.8 ns (L1/L2 boundary)
Array size: 512 KB, Avg latency: 7.2 ns (L2 hit)
Array size: 4 MB, Avg latency: 12.5 ns (L2/L3 boundary)
Array size: 16 MB, Avg latency: 18.0 ns (L3 hit)
Array size: 64 MB, Avg latency: 85.0 ns (RAM access)
```

### Random Access
```
Array size: 8 KB, Avg latency: 3.2 ns (L1 + TLB)
Array size: 64 KB, Avg latency: 8.5 ns (L1/L2 thrashing)
Array size: 512 KB, Avg latency: 15.2 ns (L2 pressure)
Array size: 4 MB, Avg latency: 28.5 ns (L3 pressure)
Array size: 16 MB, Avg latency: 42.0 ns (L3 + RAM)
Array size: 64 MB, Avg latency: 125.0 ns (RAM dominant)
```

### Coordinate Patterns
```
Same-scroll access: 2.8 ns (L1 locality)
Same-section access: 4.2 ns (L1/L2 locality)
Same-chapter access: 8.5 ns (L2 locality)
Cross-chapter access: 16.0 ns (L2/L3 boundary)
Cross-volume access: 95.0 ns (RAM access)
```

## Interpreting Results

### Cache Hit Rates (from `perf stat`)
```
cache-references:     100,000,000
cache-misses:          10,000,000
L1-dcache-loads:      100,000,000
L1-dcache-load-misses: 15,000,000
```

**Calculations**:
- Cache hit rate: `(cache-references - cache-misses) / cache-references`
  - Example: `(100M - 10M) / 100M = 90%`
- L1 hit rate: `(L1-loads - L1-misses) / L1-loads`
  - Example: `(100M - 15M) / 100M = 85%`

### Ops Per Cycle
```
cycles:        50,000,000
instructions: 200,000,000
```

**Calculation**: `instructions / cycles`
- Example: `200M / 50M = 4.0 IPC` (instructions per cycle)
- vTPU target: `3.0 ops/cycle` (retired operations, not just instructions)

## Baseline Recording

After running benchmarks, record results in:
```
/source/exo-plan/rallies/R23-WAVE2-BASELINES.md
```

**Required metrics** (KPI framework):
1. **Ops/cycle**: Current IPC from perf stat
2. **L1 hit rate**: % from perf counters
3. **L2 hit rate**: Calculate from cache hierarchy
4. **L3 hit rate**: Calculate from cache hierarchy
5. **Memory bandwidth**: GB/s from sequential access latency
6. **Sequential latency**: ns per access (by array size)
7. **Random latency**: ns per access (by array size)

## Hardware Requirements

- **CPU**: AMD R9 8945HS or similar
- **Cache**: L1=32KB, L2=512KB, L3=16MB (typical AMD R9)
- **RAM**: 16GB+ DDR5
- **OS**: Linux with perf tools

### Installing perf (Ubuntu/Debian)
```bash
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-`uname -r`
```

## Troubleshooting

### "perf not found"
Install perf tools (see above) or run without profiling:
```bash
make run  # No perf required
```

### "Permission denied" for perf
```bash
# Temporary (current session)
sudo sysctl -w kernel.perf_event_paranoid=-1

# Permanent (survives reboot)
echo "kernel.perf_event_paranoid = -1" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Unexpected latencies
- Check CPU frequency scaling: `cpupower frequency-info`
- Disable turbo boost for consistent results: `echo 0 | sudo tee /sys/devices/system/cpu/intel_pmu/allow_tsx_force_abort`
- Close background applications
- Run with `taskset` to pin to specific core: `taskset -c 0 ./sequential_access`

## Next Steps (Wave 3+)

After establishing baselines:
1. **Wave 3**: Micro-benchmark suite (synthetic SIW generator)
2. **Wave 4**: Coordinate access pattern validation
3. **Wave 5**: SMT analysis (single vs dual thread)
4. **Wave 6-7**: Deep profiling (Qwen3, PyG)
5. **Wave 8-10**: Phase 1 design (PPT architecture)

---

**Wave**: 2/40  
**Status**: 🟡 In Progress  
**Updated**: 2026-02-14 21:05 CST
