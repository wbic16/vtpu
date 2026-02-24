# R23W28 — Fill in the Gaps
**Wave:** R23W28
**Date:** 2026-02-24
**Agent:** Phex 🔱
**Status:** ACTIVE
**Goal:** Strengthen test coverage and complete stub implementations

---

## Gap Analysis

### Low Test Coverage (< 5 tests)
| Module | Tests | Lines | Priority |
|--------|-------|-------|----------|
| perf.rs | 1 | 392 | HIGH |
| pipes.rs | 3 | 268 | MEDIUM |
| cluster.rs | 4 | 170 | MEDIUM |
| hdc_optimized.rs | 4 | 244 | LOW |
| phoenix_scheduler.rs | 4 | 489 | HIGH |

### Known TODOs
1. `phoenix_scheduler.rs:` NUMA detection
2. `phoenix_scheduler.rs:` Power efficiency (/sys/class/powercap/)
3. `phoenix_scheduler.rs:` Multi-node coordination

### Small/Stub Modules
| Module | Lines | Status |
|--------|-------|--------|
| sysfs.rs | 109 | Stub? |
| narrator.rs | 150 | Minimal |
| cluster.rs | 170 | Minimal |
| assoc.rs | 172 | Minimal |

---

## W28 Tasks

### Phase 1: Test Coverage (Target: 650+ tests)
- [ ] perf.rs: Add 8+ tests for measurement accuracy
- [ ] phoenix_scheduler.rs: Add 6+ tests for scheduling decisions
- [ ] pipes.rs: Add 5+ tests for pipeline stages
- [ ] cluster.rs: Add 4+ tests for multi-node ops

### Phase 2: Complete Stubs
- [ ] sysfs.rs: Implement Linux sysfs reading (or mark as linux-only)
- [ ] narrator.rs: Add state description for remaining ops
- [ ] cluster.rs: Add basic node coordination

### Phase 3: TODOs
- [ ] NUMA detection (read /sys/devices/system/node/)
- [ ] Power efficiency (read /sys/class/powercap/ if available)
- [ ] Cluster balance metrics

---

## Success Criteria
- [ ] All modules with <5 tests → 8+ tests
- [ ] No unimplemented!() in production paths
- [ ] TODOs either implemented or converted to explicit feature flags
- [ ] Total test count: 650+

---

## Notes
- Use `#[cfg(target_os = "linux")]` for sysfs/powercap
- Mock interfaces for cross-platform testing
- Document any deferred items in BACKLOG.md
