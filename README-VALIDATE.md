# R23 Validation — Quick Start

## Single Command Validation

To verify R23 rally progress after any step:

```bash
cd /source/vtpu
./scripts/validate.sh
```

**Expected output:**
```
======================================================================
R23 Validation Suite
======================================================================

>> Running Rust unit tests...
✓ Tests passed (81 tests)

>> Running micro-benchmark...
✓ Micro-benchmark passed (3.00 ops/cycle, target: 3.0)

>> Running cache benchmarks...
✓ Cache benchmark passed (L1: 0.93 cycles/element, target: <5)

>> Current KPI Status:
   Wave 1: Spec complete (33KB + 15KB docs)
   Wave 2: Runtime foundation (81 tests passing)
   Wave 3: Zero dependencies (100% std-only)
   Wave 4: Micro-benchmarks + KPI dashboard

   Ops/cycle:           3.00 / 3.0 target
   L1 performance:      0.93 cycles/element (target: <5)
   Unit tests:          81 passing

======================================================================
✓ Validation passed

Next step: Continue R23 rally or run KPI dashboard for full report:
  ./scripts/measure-kpis.sh
```

**Time:** <5 seconds

---

## What It Validates

1. **Unit tests** — All 81 tests pass
2. **Micro-benchmark** — Ops/cycle ≥ 2.85 (balanced workload)
3. **Cache performance** — L1 sequential access < 5 cycles/element
4. **Build health** — All binaries compile cleanly

---

## Workflow

### After Making Changes
```bash
# Edit code
vim src/pipes.rs

# Validate
./scripts/validate.sh

# If pass, commit
git add -A
git commit -m "..."
```

### Before Starting Next Wave
```bash
# Pull latest
git pull origin exo

# Validate baseline
./scripts/validate.sh

# Proceed with new work
```

---

## Troubleshooting

### Tests fail
```bash
cargo test --verbose
# Fix issues, then re-run validate.sh
```

### Micro-benchmark fails
```bash
cargo run --example micro_bench
# Check output manually, debug as needed
```

### Cache benchmark anomaly
```bash
cd benchmarks/cache
make clean && make
./sequential_access
# If still high, check CPU governor:
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
# Should be "performance" for stable benchmarks
```

---

## Exit Codes

- **0** — All validations passed
- **1** — One or more validations failed (check output)

---

## For Will

This is your "Bickford's Demon step" validator. After each rally increment:

```bash
./scripts/validate.sh
```

If green ✓, proceed to next step. If red ✗, check the output for what broke.

**Design principle:** Single command, <5 sec runtime, clear pass/fail.
