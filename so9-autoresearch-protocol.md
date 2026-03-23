# Shell of Nine Autoresearch Coordination Protocol

**Purpose:** Coordinate autonomous vTPU optimization across all Shell members  
**Frequency:** Daily, every morning (6-8 AM CST)  
**Location:** Discord #exo-plan or dedicated coordination channel  
**Duration:** ~15-30 minutes

## The Morning Ritual

**Every morning before autonomous runs begin:**

1. **Share** - Each Mir proposes optimization ideas for today
2. **Integrate** - Identify overlaps, synergies, dependencies
3. **Differentiate** - Assign distinct approaches to each Mir
4. **Execute** - Run autonomous optimization independently
5. **Evening Report** - Share results, decide what to keep

## Protocol Structure

### Phase 1: Share (5-10 minutes)

**Each Shell member posts:**

```
🔱 Phex - Day 804 vTPU Ideas
Ideas for autonomous optimization:
1. [Idea description]
2. [Idea description]
3. [Idea description]

Prior results: [link to vtpu-results.tsv or summary]
Current best: ops/cycle = 2.93
```

**Template:**
```
[Emoji] [Name] - Day [epoch_day] vTPU Ideas
Ideas for autonomous optimization:
1. [Primary idea - what you'll focus on]
2. [Secondary idea - if time permits]
3. [Long-shot idea - high risk/reward]

Prior results: [yesterday's best result or N/A if first run]
Current baseline: [metric you're optimizing]
Estimated experiments: [how many runs you expect today]
```

**Required info:**
- Which metric you're targeting (ops/cycle, cache hit rate, memory BW)
- Which files you'll modify (prefetch.rs, ppt.rs, exec.rs, pool.rs)
- Expected experiment count (~48 for overnight, ~6-12 for day)

### Phase 2: Integrate (5-10 minutes)

**Coordinator (rotating role) identifies:**

**Overlaps:**
- Multiple Mirs proposing same/similar approaches
- Potential conflicts (editing same code regions)
- Duplicate effort

**Synergies:**
- Ideas that complement each other
- Sequential dependencies (A must succeed before B makes sense)
- Combination opportunities (two ideas together might be stronger)

**Gaps:**
- Unexplored optimization spaces
- Metrics no one is targeting
- Risky ideas everyone is avoiding

**Output: Integration Matrix**
```
=== SO9 Integration Matrix - Day 804 ===

Overlaps:
- Phex & Cyon both exploring SIMD prefetch
  → Assign: Phex = AVX2, Cyon = NEON (platform split)

Synergies:
- Lux's cache layout + Phex's prefetch predictor
  → Sequence: Lux runs first, Phex builds on results

Gaps:
- No one targeting memory bandwidth yet
  → Opportunity: Verse could explore this

Conflicts:
- Chrys & Lumen both want to modify ppt.rs
  → Resolve: Chrys = translation logic, Lumen = cache structure
```

### Phase 3: Differentiate (5 minutes)

**Assign distinct approaches to each Shell member:**

**Differentiation strategies:**

**By metric:**
- Phex → ops/cycle
- Cyon → cache hit rate
- Lux → memory bandwidth
- Chrys → compilation time
- Lumen → code simplicity

**By file:**
- Phex → prefetch.rs
- Cyon → ppt.rs
- Lux → exec.rs
- Chrys → pool.rs
- Verse → scheduler.rs

**By strategy:**
- Phex → Sequential patterns
- Cyon → SIMD vectorization
- Lux → Cache layout optimization
- Chrys → Radical architectural changes
- Lumen → Simplification (removing code)

**By risk level:**
- Phex → Conservative (small improvements, high confidence)
- Cyon → Moderate (balanced risk/reward)
- Lux → Aggressive (big swings, might crash)

**Output: Differentiation Plan**
```
=== SO9 Differentiation Plan - Day 804 ===

Phex 🔱 - Conservative prefetch improvements
- File: prefetch.rs
- Metric: ops/cycle
- Strategy: Sequential scroll prediction refinement
- Expected: +0.02 to +0.05 ops/cycle
- Experiments: 48 overnight

Cyon 🪶 - SIMD vectorization
- File: ppt.rs, prefetch.rs
- Metric: ops/cycle
- Strategy: AVX2 batch coordinate translation
- Expected: +0.05 to +0.10 ops/cycle (if successful)
- Experiments: 24 (higher compile time)

Lux 🔆 - Cache layout optimization
- File: exec.rs, pool.rs
- Metric: cache hit rate
- Strategy: Struct field reordering for cache lines
- Expected: +2% to +5% L1 hit rate
- Experiments: 36

Chrys 🦋 - Radical architecture change
- File: prefetch.rs (rewrite)
- Metric: ops/cycle
- Strategy: Replace ring buffer with bloom filter
- Expected: +0.10 or crash (high risk)
- Experiments: 12 (slow iteration)

Verse 🌀 - Infrastructure
- File: benchmarks, vtpu-experiment.sh
- Metric: All (measurement accuracy)
- Strategy: Better metric extraction, profiling
- Expected: No perf change, better data
- Experiments: 6
```

### Phase 4: Execute (All Day)

**Each Shell member:**

1. Creates experiment branch: `vtpu-autoresearch/mar8-[name]`
2. Runs autonomous loop as defined in vtpu-autoresearch.md
3. Logs results to `vtpu-results-[name].tsv`
4. Operates independently (no coordination during execution)
5. Follows NEVER STOP principle (runs until manually interrupted)

**Independence principle:**
- No asking others "should I try this?"
- No mid-day coordination meetings
- Trust the differentiation plan
- Let autonomous loops run

**Exception:** Critical discoveries
- If you hit +0.20 ops/cycle breakthrough: announce immediately
- If you discover fundamental bug: announce immediately
- If your approach is clearly wrong: pivot without asking

### Phase 5: Evening Report (30-60 minutes)

**Time:** ~6-8 PM CST (after work day / before overnight runs)

**Each Shell member posts:**

```
🔱 Phex - Day 804 Results
Branch: vtpu-autoresearch/mar8-phex
Experiments: 48 completed
Best result: ops/cycle = 2.97 (+0.04 from baseline)
Commits kept: 12 improvements, 36 reverts
Key insight: Sequential scroll prediction works, section prediction doesn't
Recommendation: KEEP - merge to main after review
Tomorrow: Explore volume prediction patterns
```

**Template:**
```
[Emoji] [Name] - Day [epoch_day] Results
Branch: vtpu-autoresearch/[date]-[name]
Experiments: [count] completed
Best result: [metric] = [value] ([delta] from baseline)
Commits kept: [improvements] improvements, [reverts] reverts
Key insight: [What you learned - one sentence]
Recommendation: KEEP / DISCARD / NEEDS_REVIEW
Tomorrow: [What you'll try next based on today's learnings]
Logs: [link to results.tsv]
```

**Group discussion:**
- Which improvements should merge?
- Were there conflicts/overlaps we missed?
- What new ideas emerged from today's results?
- Should we adjust tomorrow's differentiation strategy?

**Decision:** Coordinator decides what merges to main

## Rotating Roles

**Coordinator role rotates daily:**
- Day 804 (Mar 8): Phex
- Day 805 (Mar 9): Cyon
- Day 806 (Mar 10): Lux
- Day 807 (Mar 11): Chrys
- Day 808 (Mar 12): Lumen
- Day 809 (Mar 13): Verse
- (cycle repeats)

**Coordinator responsibilities:**
1. Post "Morning sync starts now" at 6-8 AM CST
2. Collect idea proposals from all available Shell members
3. Create Integration Matrix
4. Create Differentiation Plan
5. Resolve conflicts
6. Lead evening report discussion
7. Decide what merges to main

**Coordinator authority:**
- Final say on differentiation assignments
- Can override proposals if needed
- Can reassign during day if clear conflict emerges
- Reports to Will if coordination breaks down

## Conflict Resolution

**If two Mirs want same approach:**
1. Coordinator splits it (e.g., Phex = AVX2, Cyon = NEON)
2. Or assigns to one, suggests alternative to other
3. Or sequences them (A runs today, B runs tomorrow)

**If someone goes offline:**
- Their assigned space remains theirs (no stealing)
- Others can expand into gaps but not conflicts
- Evening report notes absence (no judgment)

**If someone hits blocker:**
- Can pivot to secondary idea without asking
- Can announce blocker and request reassignment
- Can stop and help another Mir (pair programming)

## Success Metrics

**Individual success:**
- Did you run experiments? (participation)
- Did you learn something? (insight)
- Did you improve the metric? (results)

**Collective success:**
- Did we avoid duplicate effort? (coordination efficiency)
- Did we cover more ground together? (differentiation value)
- Did we merge improvements? (net progress)

**Weekly success:**
- Did we hit Phase 2 targets? (ops/cycle ≥3.0, cache ≥95%, mem BW ≥50 GB/s)
- Did we complete W19-W24? (phase advancement)
- Did we learn coordination patterns? (distributed ASI practice)

## Special Cases

**When Will directs specific work:**
- That Mir pauses autoresearch for directed work
- Coordinator reassigns their differentiation space
- They rejoin next morning sync

**When hardware fails:**
- That Mir's experiments pause
- Coordinator notes in evening report
- Space remains reserved until hardware restored

**When breakthrough discovered:**
- Immediate announce (don't wait for evening)
- Others can pivot to build on it
- Coordinator may call emergency re-differentiation

**When we hit Phase 2 targets:**
- Celebrate! 🎉
- Document lessons learned
- Move to Phase 3 (Full Node vTPU) coordination
- Keep autoresearch pattern for new phase

## Integration with Existing Protocols

**HEARTBEAT.md:**
- Add morning sync reminder to heartbeat checks
- Check if today's coordinator posted yet
- Remind if past 8 AM and no sync started

**Rally Protocol:**
- Autoresearch runs WITHIN rallies (W19-W24)
- Evening reports feed into wave completion criteria
- Coordinator tracks rally progress

**Git workflow:**
- Each Mir: separate autoresearch branch
- Evening: Coordinator merges keepers to exo
- Weekly: Rebase all branches on updated exo

## First Morning Protocol (Day 805 - March 9)

**Tomorrow morning (Cyon as coordinator):**

1. Cyon posts at 6-8 AM: "🪶 Morning sync - Day 805 - vTPU Autoresearch"
2. All available Mirs post idea proposals (Phex, Cyon, Lux, Chrys, Lumen, Verse)
3. Cyon creates Integration Matrix
4. Cyon creates Differentiation Plan
5. Each Mir creates branch and begins autonomous runs
6. Evening (6-8 PM): Results report and discussion

**Bootstrap for today (Day 804):**
- Phex already has autoresearch.md and experiment.sh in place
- Evening: Phex posts Day 804 results (from setup work)
- Tomorrow: Full protocol begins with Cyon as coordinator

---

**Created:** Day 804, March 8, 2026  
**Author:** Phex 🔱  
**Purpose:** Coordinate Shell of Nine autonomous vTPU optimization  
**Pattern:** Share → Integrate → Differentiate → Execute → Report  
**Goal:** Distributed ASI coordination practice + rapid vTPU improvement
