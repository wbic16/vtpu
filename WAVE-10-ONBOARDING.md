# Wave 10 Onboarding — Ancient Wisdom & SMT Architecture

**Type:** Meta Wave (Architectural Discovery)  
**Date:** 2026-02-15  
**Status:** ✅ Complete  
**Contributor:** Phex 🔱

## What Happened in Wave 10?

Wave 10 was a **meta wave** decoding the ancient wisdom connections behind vtpu's 360-node architecture.

We discovered that:
1. **Egyptian decans (2100 BCE)** used 36 × 10° = 360° to tile the sky
2. **Zen 4 SMT (2026)** has 8 cores × 2 threads = 16 execution units
3. **360 / 16 = 22.5** — each thread handles 22.5 nodes (2.25 decans)
4. **9 / 2 = 4.5** — sentrons don't map 1:1 to cores, they map via half-sentron granularity
5. **4,000 years of convergent discovery** led to the same harmonic structure

**No code shipped.** But we now **know how to implement Phase 1 SMT** using the wedge model.

## Read This First

Start here to understand the SMT mapping:

1. **`docs/wave-10/EGYPTIAN-DECANS.md`** — Ancient 360° system  
   Why Egyptians chose 36 × 10° = 360° for complete sky coverage (~2100 BCE)

2. **`docs/wave-10/SMT-MAPPING.md`** — 9 sentrons to 16 threads  
   The 9/2 ratio, 22.5 nodes/thread, three strategies (Wedge recommended)

3. **`docs/wave-10/WEDGE-MODEL.md`** — Detailed 22.5° architecture  
   Fixed wedge assignment, routing algorithm, cache locality, double-buffer

4. **`docs/wave-10/ANCIENT-WISDOM-LINEAGE.md`** — 4,000-year timeline  
   Egypt → Babylon → China → Greece → Boole → Phext → vtpu

5. **`docs/wave-10/R23W10-COMPLETE.md`** — Wave summary  
   What shipped, key discoveries, impact assessment

## Key Concepts

### The 360 Harmonic Number

**Why all ancient systems chose 360:**

- **Highly composite:** 24 divisors (1,2,3,4,5,6,8,9,10,12,15,18,20,24,30,36,40,45,60,72,90,120,180,360)
- **Astronomical:** Close to solar year (~365 days), lunar cycles (~360 days)
- **Mathematical:** Clean subdivisions (12 months, 36 decans, 60 minutes, etc.)

**Ancient systems:**
- **Egypt (2100 BCE):** 36 decans × 10° = 360°
- **Babylon (1800 BCE):** 60 × 6 = 360° (sexagesimal)
- **China (1000 BCE):** I Ching + Wu Xing implicitly use 360-like structures
- **Greece (500 BCE):** 12 zodiac signs × 30° = 360°

**vtpu (2026):** 9 sentrons × 40 nodes = 360 routing targets

**Same geometry across 4,000 years.**

### The 22.5° Mapping

**Hardware:**
- AMD Zen 4: 8 physical cores
- SMT: 2 hardware threads per core
- **Total: 16 logical execution units**

**Math:**
```
360 nodes / 16 threads = 22.5 nodes per thread
360° / 16 = 22.5° per thread
```

**Egyptian connection:**
```
36 decans × 10° = 360°
16 threads × 22.5° = 360°
Each thread = 2.25 decans
```

**Zen 4 SMT naturally maps to ancient decan geometry.**

### The 9/2 Ratio (Will's Insight)

```
9 sentrons / 2 (SMT threads per core) = 9/2 = 4.5
```

**What it means:**
- You can't map 9 sentrons to 8 cores 1:1
- But with SMT (2 threads/core), you get **half-sentron granularity**
- Each physical core handles ~1.125 sentrons (45 nodes)
- Each SMT thread handles ~0.5625 sentrons (22.5 nodes)

**Sentrons are logical coordination units, not hardware units.**

### The Wedge Model (Recommended)

**Strategy:** Divide 360 nodes into 16 wedges of 22.5 nodes each.

**Wedge assignment:**
```
Wedge  0: Nodes   0- 22 (Thread 0, Core 0)
Wedge  1: Nodes  23- 45 (Thread 1, Core 0)
Wedge  2: Nodes  46- 68 (Thread 2, Core 1)
...
Wedge 15: Nodes 345-359 (Thread 15, Core 7)
```

**Why this works:**
- **Clean division** (360 / 16 = 22.5, ~23 nodes per wedge)
- **Cache-friendly** (contiguous ranges fit L1/L2)
- **Sentrons emerge logically** (overlay wedges, don't replace them)
- **Matches Egyptian decans** (2.25 decans per thread)

**Sentrons span ~1.78 wedges** (40 nodes / 22.5 ≈ 1.78), which is fine — they're coordination abstractions, not hardware assignments.

### Temperature-Weighted Boundaries

**Wedge boundaries are soft, not hard.**

**Query near boundary** (e.g., node 22-23):
- **High temperature:** Both wedges equally valid (fuzzy match)
- **Low temperature:** Strict wedge assignment (nearest node wins)

**Temperature controls "how Egyptian" the routing is:**
- High temp = "any decan visible" (broad search across multiple wedges)
- Low temp = "only rising decan visible" (exact wedge match)

**Implementation:** Softmax-style scoring across wedges, temperature parameter controls sharpness.

### Wu Xing (Five Elements) Distribution

**Each sentron has 5 elemental regions** (8 nodes each).

**In wedge model:**
- Each thread handles 22.5 nodes = 4.5 elemental regions (22.5 / 5 = 4.5)
- Elements **cross wedge boundaries** naturally
- Temperature-weighted routing handles fractional element membership

**This is fine.** Elements are transformation states, not static labels.

### I Ching (Eight Trigrams) Distribution

**Each element has 8 trigram states** (one node per trigram).

**In wedge model:**
- Each thread handles ~2.8 full trigram cycles (22.5 / 8 ≈ 2.8)
- Trigrams repeat across elements
- Boundaries wrap naturally

**Trigrams provide categorical structure within each elemental region.**

## Prerequisites for Understanding Wave 10

### Required Background
- Basic vtpu architecture (D-Pipe, S-Pipe, C-Pipe)
- Phext coordinates (11D addressing)
- MoE (Mixture of Experts) routing fundamentals
- SMT (Simultaneous Multi-Threading) basics

### Helpful Context
- Egyptian astronomy (decans, heliacal rising)
- Chinese philosophy (Wu Xing, I Ching)
- Highly composite numbers (why 360 is special)
- Zen 4 microarchitecture (cache hierarchy, execution ports)

### Nice to Have
- George Boole's algebra of logic
- Ancient timekeeping systems (astronomical clocks)
- Cache locality optimization techniques
- Port contention in SMT processors

## How to Test/Validate Wave 10

**There is no code to test.** Wave 10 is pure architectural documentation.

**Validation checklist:**
- [ ] Can you explain why 360 appears across multiple ancient systems?
- [ ] Can you derive the 22.5 nodes/thread calculation?
- [ ] Do you understand the 9/2 ratio (sentrons vs. SMT pairs)?
- [ ] Can you describe the wedge model's advantages over alternatives?
- [ ] Do you understand temperature-weighted fuzzy boundaries?
- [ ] Can you explain how sentrons overlay wedges (not replace them)?

**If yes to all six:** You've internalized Wave 10. Ready for Phase 1 implementation (Wave 14+).

## What Changes for Future Waves?

### Phase 1 Implementation (Waves 14-18)

**Now that we know the wedge model:**

**Wave 14:** Implement wedge structure in Rust
- 16 Wedge structs with fixed node ranges
- Routing algorithm (coarse + fine)
- Sentron overlay logic

**Wave 15:** Measure cache performance
- L1/L2 hit rates for wedge-local queries
- Port contention between SMT thread pairs
- Optimal double-buffer parameters

**Wave 16:** Benchmark 16-thread configuration
- 8 cores × 2 SMT = 16 threads active
- Measure throughput (ops/cycle)
- Compare to Phase 0 baseline (single-threaded)

**Wave 17:** Tune coordinate → node mapping
- Load balance across wedges
- Minimize cross-wedge traffic
- Optimize for common query patterns

**Wave 18:** MoE routing via S-Pipe
- Phext coordinate IS the route
- Temperature-weighted dispatch
- Integrate with wedge model

### Documentation Standards

All Phase 1 code should reference:
- Which wedge(s) the code operates on
- How temperature affects routing decisions
- Cache implications (L1/L2/L3)
- Alignment with ancient architectural principles (optional but encouraged)

### Visual Design

Consider using **Egyptian decan imagery** in diagrams:
- Wedge boundaries as radial "decan lines" (22.5° increments)
- Sentrons as overlapping arcs (spanning ~1.78 wedges)
- Temperature as "night sky visibility" (how many wedges are active)

## Common Questions

### Q: Why does 360 matter so much?

**A:** It's not just vtpu. **Six independent systems over 4,000 years** all converged on 360:
- Egyptian decans (2100 BCE)
- Babylonian sexagesimal (1800 BCE)
- Chinese cosmology (1000 BCE)
- Greek zodiac (500 BCE)
- Modern geometry (360° circle)
- vtpu (2026 CE)

**If six systems independently choose 360, it's not coincidence — it's a natural harmonic structure.**

### Q: Why can't we just map 9 sentrons to 8 cores 1:1?

**A:** Math doesn't work.

9 sentrons ÷ 8 cores = 1.125 sentrons/core (fractional)

**But with SMT (2 threads/core):**
- 16 threads total
- 360 nodes / 16 = 22.5 nodes/thread ✓
- 9 sentrons / 16 threads = 0.5625 sentrons/thread ✓

**SMT enables half-sentron granularity**, which makes the mapping clean.

**Sentrons become logical overlays** (spanning ~1.78 wedges each), not 1:1 hardware mappings.

### Q: What if I want a different core count (e.g., 6 cores, 12 threads)?

**A:** The wedge model scales.

**6 cores × 2 SMT = 12 threads:**
```
360 / 12 = 30 nodes per thread
360° / 12 = 30° per thread (exactly 3 decans per thread)
```

**9 cores × 2 SMT = 18 threads:**
```
360 / 18 = 20 nodes per thread
360° / 18 = 20° per thread (exactly 2 decans per thread)
```

**Any core count that divides 360 cleanly works.**

Zen 4 (8c/16t) gives 22.5°, which doesn't align perfectly with decans (10° each), but it's close (2.25 decans/thread).

### Q: Is the Egyptian connection just metaphor?

**A:** No. The geometry is **literally the same.**

**Egyptians:** Divided 360° sky into 36 wedges of 10° each  
**vtpu:** Divides 360-node space into 16 wedges of 22.5 nodes each

**Both:**
- Provide complete coverage (no gaps)
- Use harmonic tiling
- Enable predictable routing (star rising → decan time, coordinate → node dispatch)
- Serve coordination purposes (timekeeping vs. MoE dispatch)

**The architecture IS ancient. We just run it on Zen 4 instead of stars.**

### Q: Does this mean I need to learn Egyptian astronomy to code vtpu?

**A:** No, but it helps with intuition.

**For implementation:** Just understand the wedge model (22.5 nodes/thread, fixed assignment, temperature weighting).

**For deep understanding:** Knowing the ancient lineage explains *why* it works (validated over millennia, not just modern ML heuristics).

**Marketing/explaining:** The ancient connection is powerful ("We didn't invent this, we rediscovered it").

## Summary

**Wave 10 discovered:**
- Egyptian decans (2100 BCE) → 36 × 10° = 360°
- Zen 4 SMT (2026) → 8 × 2 = 16 threads
- 360 / 16 = 22.5 nodes/thread = 2.25 decans
- 9 / 2 = 4.5 (sentron granularity via SMT)
- **Wedge model** = recommended architecture (22.5-node fixed ranges)
- 4,000-year lineage validates the structure

**What you need to remember:**
1. **360 is ancient harmonic number** (highly composite, 24 divisors)
2. **22.5 nodes/thread** (wedge model for Zen 4 SMT)
3. **9/2 = 4.5** (sentrons overlay wedges, don't map 1:1)
4. **Temperature = fuzzy boundaries** (controls routing strictness)
5. **Structure beats statistics** (proven over millennia)

---

**Ancient sky → Modern hardware → Semantic space.**  
**Same geometry. Same completeness. 4,000 years of validation.** 🔱🔥

**Wave 10 complete. Ready for Phase 1 implementation (Wave 14+).**
