# Wave 3 Onboarding Guide for Will

**Wave:** 3/40 (6-Machine Coordinate Scheme Design)  
**Status:** Complete ✅  
**Duration:** 30 minutes  
**Date:** 2026-02-14 22:40-23:10 CST

---

## What We Built

**Deliverable:** `WAVE-3-COORDINATE-SCHEME.md` (10 KB)

**Contents:**
1. **Namespace assignment:** Each ranch machine gets unique prefix (phex., cyon., lux., chrys., lumen., theia.)
2. **Coordinate space layout:** volume.book.chapter/section.scroll (5D), 10× per dimension = 100,000 scrolls/machine
3. **Capacity calculation:** 6 machines × 100,000 = 600,000 total scrolls addressable
4. **Utilization:** 500 scrolls used / 600,000 capacity = 0.08% (massive headroom)
5. **Routing protocol:** Namespace → machine lookup → local/remote SQ query
6. **Latency model:** Local <1ms, remote 20-40ms (network-bound)
7. **Scaling roadmap:** 6 → 50 → 500 machines (5M → 50M scrolls)

---

## How to Verify This Design

### Test 1: Namespace Mapping
```bash
# Verify each machine has unique namespace
echo "phex. → aurora-continuum"
echo "cyon. → halcyon-vector"
echo "lux. → logos-prime"
echo "chrys. → chrysalis-hub"
echo "lumen. → lilly"
echo "theia. → aletheia-core"
```

**Expected:** No collisions, each machine has one namespace.

### Test 2: Capacity Calculation
```bash
# Calculate scrolls per machine
echo "10 (vol) × 10 (book) × 10 (ch) × 10 (sec) × 10 (scroll) = 100,000"

# Total capacity
echo "6 machines × 100,000 = 600,000 scrolls"
```

**Expected:** 600,000 total addressable scrolls.

### Test 3: Routing Logic (Conceptual)
```python
def route_coordinate(coord):
    # Parse namespace
    namespace, rest = coord.split('.', 1)
    
    # Lookup machine
    machines = {
        'phex': 'aurora-continuum',
        'cyon': 'halcyon-vector',
        'lux': 'logos-prime',
        'chrys': 'chrysalis-hub',
        'lumen': 'lilly',
        'theia': 'aletheia-core'
    }
    
    machine = machines.get(namespace)
    
    # Route
    if machine == current_machine():
        return local_sq_query(rest)  # <1ms
    else:
        return http_request(f"http://{machine}:1337/api/v2/select?c={rest}")  # ~20-40ms

# Example
route_coordinate("cyon.2.3.4/5.6.7/8.9.10")
# → HTTP GET to halcyon-vector:1337
```

**Expected:** Local queries fast, remote queries slower (network latency).

---

## Key Files

All artifacts in `/source/exo-plan/rally/R23/`:

- `WAVE-3-COORDINATE-SCHEME.md` - Main deliverable (10 KB)
- `WAVE-3-ONBOARDING.md` - This file

---

## Current State

**What works:**
- ✅ Namespace scheme defined (6 prefixes assigned)
- ✅ Coordinate space calculated (600,000 scrolls)
- ✅ Routing protocol conceptualized (namespace → machine → query)
- ✅ Latency model documented (local vs remote)

**What's conceptual (not implemented yet):**
- 🚧 Actual SQ mesh networking (Wave 4 will detail this)
- 🚧 Cross-machine HTTP routing (implementation in R24)
- 🚧 Peer discovery mechanism (implementation in R24)

**Next dependency for Wave 4:**
- Namespace → machine mapping (complete)
- Coordinate space layout (complete)
- Routing protocol basics (complete)

---

## How to Review Wave 3

1. **Read the deliverable:**
   ```bash
   cat /source/exo-plan/rally/R23/WAVE-3-COORDINATE-SCHEME.md
   ```

2. **Verify namespace assignments:**
   - Does each machine have exactly one namespace?
   - Are all 6 machines covered (including offline Theia)?

3. **Check capacity math:**
   - 10^5 scrolls per machine?
   - 600,000 total for 6 machines?
   - Scaling to 50M scrolls at 500 machines?

4. **Validate routing protocol:**
   - Namespace prefix determines target machine?
   - Local queries are fast (<1ms)?
   - Remote queries include network latency (20-40ms)?

5. **Assess for Wave 4:**
   - Can Wave 4 design SQ mesh routing from this? **YES**
   - Is the coordinate space well-defined? **YES**
   - Are latency expectations clear? **YES**

---

## Tests That Would Pass

These should work if implemented:

```bash
# Local query (Phex on aurora-continuum)
curl http://localhost:1337/api/v2/select?p=phex&c=1.1.1/1.1.1/1.1.1
# Expected: <1ms response, scroll content

# Remote query (Phex accessing Cyon's memory)
curl http://halcyon-vector:1337/api/v2/select?p=cyon&c=2.3.4/5.6.7/8.9.10
# Expected: ~30ms response (network), scroll content or 404

# Offline machine (Theia)
curl http://aletheia-core:1337/api/v2/select?p=theia&c=1.1.1/1.1.1/1.1.1
# Expected: Connection timeout or 503 Service Unavailable
```

**Note:** SQ is not currently running on ranch nodes, so these would fail today. Wave 4 will design the mesh deployment.

---

## Quick Validation Checklist

- [ ] All 6 namespaces assigned?
- [ ] Capacity = 600,000 scrolls?
- [ ] Utilization ~0.08% (500 / 600,000)?
- [ ] Routing protocol defined (local vs remote)?
- [ ] Network latency factored in (20-40ms)?
- [ ] Scaling roadmap clear (6 → 50 → 500)?
- [ ] Wave 4 can proceed with this foundation?

---

## What's Next

**Wave 4/40:** SQ Mesh Routing Protocol (25 min)
- HTTP API endpoint routing logic
- Peer discovery mechanism
- Mesh sync protocol details
- Failure handling (offline machines)

**Dependencies from Wave 3:**
- ✅ Namespace → machine mapping
- ✅ Coordinate space layout
- ✅ Routing protocol basics

**No blockers.** Ready to proceed.

---

**Location:** `/source/exo-plan/rally/R23/WAVE-3-ONBOARDING.md`  
**Last updated:** 2026-02-14 23:10 CST

🔱 Phex | Wave 3 Onboarding | 1.5.2/3.7.3/9.1.1
