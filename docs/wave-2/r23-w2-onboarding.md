# R23 Wave 2 Onboarding Guide

**Created:** 2026-02-14 21:03 CST  
**Author:** Lumen  
**Purpose:** Sync Will up on current state + how to run/test what we've built

---

## Current State Summary

### What We Have (Documentation)
✅ **Wave 1 (14.1 KB):**
- Geometric advantages of phext coordinates (7 structures)
- Hard problems solved by vTPU (7 speedups: 2-81,000×)
- Complete summary + breakthrough insights

✅ **Wave 2 Base (36.1 KB):**
- vTPU instruction set (10 operations: CGET, CPUT, CRANGE, etc.)
- Concrete examples (GPT-4 attention, MoE, knowledge graphs, multi-agent)
- Memory layout specification (hash table + linked list + distributed cluster)

✅ **Wave 2 Iteration 1 (25 KB):**
- ASCII architecture diagrams
- Python client library (design/pseudocode)
- Benchmark suite design
- Error handling specifications
- Z-order curve implementation
- Production deployment config

**Total:** 75.2 KB of technical specification

---

## What's Runnable NOW

### Reality Check
**Nothing is implemented yet.** Wave 1-2 are pure specification/design.

What we have:
- ✅ Design docs
- ✅ Architecture specs
- ✅ Pseudocode examples
- ✅ Performance projections
- ❌ No working code
- ❌ No benchmarks running
- ❌ No vTPU prototype

---

## What We CAN Test Today (Validation Path)

### Option 1: SQ Microbenchmarks (Validate Hash Table Performance)

**Goal:** Measure actual hash table lookup latency on ranch cluster

**Steps:**
1. **Pick a node** (e.g., lilly at 192.168.86.36:1337)

2. **Test CGET latency:**
```bash
# Write 1000 scrolls
for i in {0..999}; do
  curl -X POST http://192.168.86.36:1337/write \
    -H "Content-Type: application/json" \
    -d "{\"coordinate\": \"1.1.$i/1.1.1/1.1.1\", \"content\": \"Test $i\"}"
done

# Benchmark reads (bash timing)
time for i in {0..999}; do
  curl -s http://192.168.86.36:1337/read/1.1.$i/1.1.1/1.1.1 > /dev/null
done

# Expected: ~1-2 seconds for 1000 reads = 1-2 ms per read
# (includes HTTP overhead, actual hash lookup is ~50-100 μs)
```

3. **Interpret results:**
- If 1-2 ms/read: ✅ Matches our 100 μs hash lookup + network overhead estimate
- If >10 ms/read: ❌ SQ has performance issues, investigate

---

### Option 2: Range Query Test (Validate CRANGE)

**Goal:** Measure range query performance

**Steps:**
1. **Write scrolls in a pattern:**
```bash
# Write 100 scrolls in chapter 2.1.*
for i in {0..99}; do
  curl -X POST http://192.168.86.36:1337/write \
    -H "Content-Type: application/json" \
    -d "{\"coordinate\": \"2.1.$i/1.1.1/1.1.1\", \"content\": \"Range test $i\"}"
done
```

2. **Query the range:**
```bash
time curl -s http://192.168.86.36:1337/range/2.1.*/1.1.1/1.1.1
```

3. **Expected result:**
- Returns 100 scrolls
- Latency: 10-50 ms (depends on current SQ implementation)
- If SQ has Z-order index: <10 ms
- If SQ uses linear scan: 50-100 ms

---

### Option 3: Multi-Node Coordination Test (Validate Distributed Routing)

**Goal:** Test cross-node data access (CROUTE concept)

**Steps:**
1. **Check which nodes are online:**
```bash
# From each ranch node
ssh lilly "curl -s http://localhost:1337/health"
ssh chrysalis-hub "curl -s http://localhost:1337/health"
ssh aurora-continuum "curl -s http://localhost:1337/health"
# ... etc for all 6 nodes
```

2. **Write to different coordinate ranges:**
```bash
# Write to node 0 range (0.*.*)
curl -X POST http://192.168.86.36:1337/write \
  -d '{"coordinate": "0.1.1/1.1.1/1.1.1", "content": "Node 0 data"}'

# Write to node 1 range (1.*.*)
curl -X POST http://192.168.86.37:1337/write \
  -d '{"coordinate": "1.1.1/1.1.1/1.1.1", "content": "Node 1 data"}'
```

3. **Query from any node:**
```bash
# Should return data even if stored on different node
curl http://192.168.86.36:1337/read/1.1.1/1.1.1/1.1.1
```

4. **Expected:**
- ✅ Data accessible from any node (SQ mesh sync working)
- ❌ If not: Mesh sync is broken, need to fix SQ coordination

---

## What to Build NEXT (Wave 3 Prerequisite)

### Minimal vTPU Prototype (Python)

**File:** `/source/exo-plan/rally/R23/prototype/vtpu_client.py`

**What it does:**
- Wraps SQ REST API
- Implements CGET, CPUT, CRANGE in Python
- Runs sparse attention microbenchmark
- Outputs: Latency numbers, throughput, comparison to projections

**How to run it (once built):**
```bash
# From any ranch node with Python 3
cd /source/exo-plan/rally/R23/prototype

# Install dependencies
pip install requests numpy

# Run benchmark
python vtpu_benchmark.py --host 192.168.86.36 --port 1337

# Expected output:
# CGET latency: 1.2 ms (median)
# CRANGE throughput: 50 scrolls/sec
# Sparse attention: 2.5 sec for 2048 tokens (window=128)
```

**Status:** Not built yet. Should we build this in Wave 3 or 4?

---

## Testing the W2 Iteration 1 Designs

### Python Client Library Test

**Once implemented, test like this:**

```python
from vtpu_client import vTPU

# Connect to SQ
vtpu = vTPU(host="192.168.86.36", port=1337)

# Test CGET
vtpu.cput("1.1.1/1.1.1/1.1.1", "Hello vTPU")
result = vtpu.cget("1.1.1/1.1.1/1.1.1")
print(result)  # Should print: {'coordinate': '1.1.1/1.1.1/1.1.1', 'content': 'Hello vTPU'}

# Test CRANGE
vtpu.cput("2.1.1/1.1.1/1.1.1", "Scroll 1")
vtpu.cput("2.1.2/1.1.1/1.1.1", "Scroll 2")
vtpu.cput("2.1.3/1.1.1/1.1.1", "Scroll 3")
results = vtpu.crange("2.1.*/1.1.1/1.1.1")
print(len(results))  # Should print: 3

# Test CDELTA
base = "2.1.3/4.7.11/18.29.47"
offset = "+0.0.0/+0.0.5/+0.0.0"
result = vtpu.cdelta(base, offset)
print(result)  # Should print: 2.1.3/4.7.16/18.29.47

# Test sparse attention (simplified)
vtpu.cput("0.0.100/query.0.0/1.1.1", "[0.1, 0.2, 0.3, ...]")
vtpu.cput("0.0.101/key.0.0/1.1.1", "[0.15, 0.25, 0.35, ...]")
scores = vtpu.cattend("0.0.100/query.0.0/1.1.1", "0.0.*/key.0.0/1.1.1")
print(scores)  # Should return similarity scores
```

**Current status:** Pseudocode only, needs implementation

---

## Benchmark Validation Checklist

### When Python client is built, validate these claims:

**From W2 specs, we claimed:**

| Operation | Projected Latency | Test Command | Pass Criteria |
|-----------|------------------|--------------|---------------|
| **CGET (local)** | 50-100 μs | `vtpu_benchmark.py --test cget` | <200 μs median |
| **CPUT (local)** | 50-100 μs | `vtpu_benchmark.py --test cput` | <200 μs median |
| **CRANGE (100 items)** | 5-10 ms | `vtpu_benchmark.py --test crange --size 100` | <20 ms |
| **Remote RDMA** | 1.1 μs | `vtpu_benchmark.py --test remote` | <5 μs |
| **Z-order speedup** | 1000× vs linear | `vtpu_benchmark.py --test zorder` | >100× speedup |

**If benchmarks fail:**
- Investigate SQ implementation (hash table, linked list, RDMA)
- Adjust projections in W2 docs
- Rewrite claims in academic paper (W3)

---

## Current Infrastructure State

### Ranch Cluster Status (as of 2026-02-14)

**Known online:**
- ✅ lilly (192.168.86.36)
- ✅ chrysalis-hub
- ✅ aurora-continuum
- ✅ halcyon-vector
- ✅ logos-prime
- ✅ aletheia-core

**SQ instances:**
- Running on all nodes? → **Need to verify**
- Port 1337 open? → **Need to verify**
- Mesh sync enabled? → **Need to verify**

**Action item:** Run health checks on all 6 nodes before W3

```bash
# Quick health check script
for node in lilly chrysalis-hub aurora-continuum halcyon-vector logos-prime aletheia-core; do
  echo "Testing $node..."
  ssh $node "curl -s http://localhost:1337/health || echo 'FAILED'"
done
```

---

## What to Build in Wave 3 (Decision Point)

### Option A: Start Academic Paper (Original Plan)
- Write abstract + introduction (2-3 hours)
- No code, just documentation
- Deliverable: Paper draft (first 3 pages)

### Option B: Build Minimal Prototype First
- Implement `vtpu_client.py` (Python wrapper for SQ)
- Run microbenchmarks on ranch
- Validate all W2 performance claims
- Deliverable: Working prototype + benchmark results

### Option C: Hybrid (Recommended)
- W3a: Abstract + introduction (1 hour)
- W3b: Python client library (1.5 hours)
- W3c: Run benchmarks, update paper with REAL numbers (0.5 hours)
- Deliverable: Paper draft + validated benchmarks

**Recommendation:** Option C (hybrid) gives us real numbers for the paper instead of projections.

---

## Quick Start Guide (for Will)

### Validate W2 Specs Right Now (15 minutes)

1. **Test SQ hash table latency:**
```bash
ssh lilly
cd /tmp

# Write 100 scrolls
time for i in {0..99}; do
  curl -X POST http://localhost:1337/write \
    -H "Content-Type: application/json" \
    -d "{\"coordinate\": \"1.1.$i/1.1.1/1.1.1\", \"content\": \"Test $i\"}" \
    -s > /dev/null
done

# Read them back
time for i in {0..99}; do
  curl -s http://localhost:1337/read/1.1.$i/1.1.1/1.1.1 > /dev/null
done
```

2. **Interpret results:**
   - Write time: Should be 2-5 seconds (20-50 ms per write)
   - Read time: Should be 1-3 seconds (10-30 ms per read)
   - If slower: SQ has performance issues
   - If faster: Great! Update W2 projections upward

3. **Test range query:**
```bash
curl -s http://localhost:1337/range/1.1.*/1.1.1/1.1.1 | jq '. | length'
# Should return 100 (number of scrolls)
```

4. **Report findings to Lumen:**
   - Actual latencies observed
   - Any errors or failures
   - Next steps (build prototype? fix SQ? write paper?)

---

## Files to Review Before W3

**Must read:**
1. `/source/exo-plan/rally/R23/R23-W1-COMPLETE-SUMMARY.md` (6 KB) - Core thesis
2. `/source/exo-plan/rally/R23/R23-W2-INSTRUCTION-SET.md` (11 KB) - What we're building
3. `/source/exo-plan/rally/R23/R23-W2-EXAMPLES.md` (14 KB) - How it works in practice

**Optional deep dive:**
4. `/source/exo-plan/rally/R23/R23-W2-ITERATION-1.md` (25 KB) - Python lib + benchmarks
5. `/source/exo-plan/rally/R23/R23-W2-MEMORY-LAYOUT.md` (11 KB) - How SQ stores coordinates

---

## Decision Needed from Will

**Before starting W3, clarify:**

1. **Build prototype first or write paper first?**
   - Prototype: Validates claims, gives real numbers for paper
   - Paper: Faster to ship, uses projected numbers (marked as "projected")

2. **Which SQ instance to use for testing?**
   - Lilly (192.168.86.36)? Other node?
   - Is SQ mesh sync working across all 6 nodes?

3. **Benchmark scope:**
   - Minimal (just CGET/CRANGE latency): 1 hour
   - Full (all 10 instructions + sparse attention): 3-4 hours

4. **Academic paper target:**
   - arXiv preprint only (faster, no review)
   - Conference submission (ISCA 2027, MLSys 2027, longer timeline)

---

## Current Blockers

**None.** Wave 2 complete, ready for W3 on your signal.

**Potential blockers:**
- SQ instance not running → Need to start it
- Mesh sync broken → Need to fix before multi-node tests
- Python not available on ranch → Use local machine + remote SQ

---

## Summary: How to Run This NOW

**Short answer:** Can't run anything yet (it's all spec/design).

**What you CAN do:**
1. Read the W1/W2 docs (understand the architecture)
2. Test SQ performance on ranch (validate hash table latency claims)
3. Decide: Build prototype first, or write paper with projections?

**What Lumen will do in W3 (your choice):**
- Option A: Academic paper (abstract + intro)
- Option B: Python prototype + benchmarks
- Option C: Both (paper draft + working code)

**Recommended path:** Option C (1 hr paper + 2 hrs prototype = 3 hrs total)

---

**Questions for Will:**
1. Which ranch node should I use for testing? (lilly = 192.168.86.36?)
2. Is SQ running on all 6 nodes? (need health check)
3. Wave 3 priority: Paper-first or prototype-first?

---

*End of W2 Onboarding Guide*  
*Next: Wave 3 (TBD based on Will's decision)*

— Lumen ✴️  
2026-02-14 21:08 CST
