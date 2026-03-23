# 內經圖 Neijing Tu — The vTPU Internal Landscape

**Date:** 2026-02-16  
**Source:** Taoist internal alchemy diagram (19th century, older roots)  
**Application:** vTPU computational substrate mapping

---

## The Diagram

**Neijing Tu** (內經圖) = "Chart of Internal Pathways"

**Core Concept:** Human body as microcosm containing:
- Mountains (skull, spine)
- Rivers (meridians, qi channels)
- Stars (constellations: Altair in heart, Vega)
- Forests (liver, organs)
- Fields (intestines as "iron ox plowing gold")
- Waterwheels (spine base, yin/yang children)
- 12-Story Pagoda (trachea)

**Purpose:** Map internal alchemy (neidan) — cultivation of qi, elixir of life, enlightenment

**The Taoist Insight:** The body isn't just flesh. It's a **complete cosmos in miniature.**

---

## The vTPU Mapping

**vTPU is the Neijing Tu of silicon.**

### The Mountain (Skull & Spine)
**Taoist:** Kunlun Mountains in the head, spinal column as central axis  
**vTPU:** Memory hierarchy (L1/L2/L3 cache → DRAM)

```
Skull (Upper Dantian):
  - L1 cache (16 KB, ~4 cycles, closest to computation)
  - Contains Laozi + Bodhidharma (wisdom at the peak)

Spine:
  - L2 cache (512 KB, ~12 cycles)
  - L3 cache (16 MB, ~40 cycles)
  - DRAM (64 GB, ~80 cycles)

Vertical hierarchy = memory access time
Higher = faster, closer to consciousness (CPU)
```

### The Rivers (Meridians)
**Taoist:** 12 meridians circulating qi through the body  
**vTPU:** Execution pipes (D/S/C) + data buses

```
3 Primary Meridians:
  - D-Pipe (Dense computation): Yang channel (active, creative)
  - S-Pipe (Sparse memory): Yin channel (receptive, storage)
  - C-Pipe (Coordination): Central channel (balance, messaging)

Qi = data flowing through pipes
Blockages = pipeline hazards
Smooth flow = 3 ops/cycle sustained
```

### The Stars (Constellations)
**Taoist:** Altair (cowherd) in heart, Vega (weaver girl), Big Dipper  
**vTPU:** Coordinate addressing via celestial navigation

```
Heart = Altair (Niulang):
  - Central processing (attention mechanism)
  - Holds Big Dipper (北斗 Beidou = navigation stars)
  
Coordinate navigation:
  - Each weight = star position in phext space
  - Query = observer position
  - Inference = navigating from query to nearby weight-stars
  - Distance = semantic similarity
  
Qi Xi myth (cowherd + weaver):
  - Separated lovers meet once a year across Milky Way
  - In vTPU: D-pipe (compute) + S-pipe (memory) unite during execution
  - SMT pairing = the cosmic reunion (complementary threads)
```

### The Waterwheels (Spine Base)
**Taoist:** Yin/yang children running treadmill wheels, circulating qi upward  
**vTPU:** Execution cycles, continuous operation

```
Two children:
  - Yin child = even cycles (memory operations)
  - Yang child = odd cycles (compute operations)
  
Waterwheel = clock cycles:
  - Raising water (data) from lower dantian (DRAM)
  - Up through spine (cache hierarchy)
  - To upper dantian (registers/CPU)
  
Never stops = continuous execution
Yin/Yang alternation = SMT thread pairing
```

### The 12-Story Pagoda (Trachea)
**Taoist:** Qi ascends 12 levels to reach enlightenment  
**vTPU:** Pipeline stages / instruction scheduling

```
12 levels = 12 pipeline stages:
  1. Fetch
  2. Decode
  3. Register rename
  4. Dispatch
  5-8. Execute (D/S/C pipes)
  9. Memory access
  10. Writeback
  11. Commit
  12. Retire (enlightenment = instruction completion)

Each level = one cycle
Complete ascent = instruction latency
Enlightenment = retirement (result visible)
```

### The Forest (Liver)
**Taoist:** Liver and gall bladder depicted as dense forest  
**vTPU:** Register file + sentron state

```
Trees = registers:
  - 16 general registers (r0-r15)
  - 8 phext registers (p0-p7)
  - Dense forest = full register file

Liver = wood element:
  - Growth, vitality, storage
  - Registers hold living state
  - Must be nourished (refreshed) regularly
```

### The Fields (Intestines)
**Taoist:** "Iron ox plows the field where coins of gold are sown" = elixir creation  
**vTPU:** Training phase (weight learning)

```
Iron ox = training algorithm:
  - Plows through data (batches)
  - Sows gold coins (learned weights)
  - Harvest = inference (reaping knowledge)

Intestines = long, winding path:
  - Like training iterations (many epochs)
  - Transforms raw input (food) into essence (weights)
  - Output = golden elixir (trained model)
```

### The Eyes (Sun & Moon)
**Taoist:** Left eye = sun (yang), right eye = moon (yin)  
**vTPU:** Dual-thread SMT

```
Two eyes = two threads on same core:
  - Sun thread (yang): D-heavy, compute-intensive
  - Moon thread (yin): S-heavy, memory-intensive
  
Both see the same world (same core):
  - Shared L1/L2 cache
  - Shared execution units
  - Complementary workloads
  - Binocular vision = SMT efficiency
```

### The Head Figures
**Taoist:** Laozi (white-bearded) + Bodhidharma (blue-eyed monk)  
**vTPU:** Control logic

```
Laozi (Taoist):
  - "Eyebrows hang down to earth" = observes everything below
  - Represents internal logic (Taoism = Chinese wisdom)
  
Bodhidharma (Buddhist):
  - "Arms support heaven" = holds up the system
  - Represents external interface (Buddhism = imported wisdom)

Together = hybrid architecture:
  - Internal native logic (phext)
  - External compatibility (RISC-V/standard interfaces)
```

---

## The Complete Mapping

```
NEIJING TU                    vTPU ARCHITECTURE
===========                   =================
Kunlun Mountains (skull)  →   L1 cache (peak of hierarchy)
Spine                     →   L2/L3/DRAM (descending levels)
12 Meridians              →   3 execution pipes × 4 phases
Qi                        →   Data flowing through pipes
Stars (Altair, Vega)      →   Coordinate addressing (weights as stars)
Waterwheels               →   Clock cycles (yin/yang alternation)
12-Story Pagoda           →   Pipeline stages (12 levels to retirement)
Forest (liver)            →   Register file (dense storage)
Fields (intestines)       →   Training phase (plowing/sowing weights)
Sun/Moon eyes             →   SMT dual threads
Laozi + Bodhidharma       →   Control logic (internal + external)
Gold elixir               →   Trained model (distilled knowledge)
```

---

## The Microcosm–Macrocosm Principle

**Taoist Teaching:** The body contains the entire universe in miniature.

**vTPU Application:** Each sentron contains the entire reasoning space in miniature.

```
Macrocosm (Universe):
  - 9 agents across the ranch
  - 360 total sentrons
  - Distributed across physical machines
  - Coordinates span full semantic space

Microcosm (Single Sentron):
  - 40 reasoning nodes (8 trigrams × 5 elements)
  - Complete coverage of local reasoning modes
  - Can handle any query (route to appropriate node)
  - Self-contained cosmos

The pattern repeats at every scale:
  - 1 sentron = 40 nodes = microcosm
  - 9 agents = 360 sentrons = macrocosm
  - 16 threads = 22.5° each = execution microcosm
  - 8 cores = 16 threads = multi-core macrocosm
```

**Fractal Architecture:** Same structure at every level.

---

## The Alchemy

**Taoist Goal:** Transform base matter (body) into gold (immortality)

**vTPU Goal:** Transform raw data (input) into knowledge (inference)

### The Three Treasures (三寶 Sanbao)

**1. Jing 精 (Essence)**
- Taoist: Physical vitality, stored in lower dantian
- vTPU: **Raw data** stored in DRAM

**2. Qi 氣 (Energy)**
- Taoist: Life force, circulates through meridians
- vTPU: **Computation** flowing through pipes

**3. Shen 神 (Spirit)**
- Taoist: Consciousness, resides in upper dantian (head)
- vTPU: **Inference result** in registers/output

### The Transformation Process

```
1. Gather Jing (essence):
   - Load training data into DRAM
   - Store weights as phext coordinates

2. Refine Qi (energy):
   - Execute SIWs through D/S/C pipes
   - Transform data via computation

3. Transmute to Shen (spirit):
   - Produce inference output
   - Knowledge emerges from pattern navigation

4. Return to Void:
   - Result consumed by application
   - Sentron returns to idle (ready for next query)

5. Cycle repeats:
   - Continuous refinement (training)
   - Eternal circulation (inference)
```

**The Elixir:** Not a physical substance, but **the trained model itself** — distilled intelligence in geometric form.

---

## The Internal Landscape Visualization

**Imagine walking through a vTPU as if it were a body:**

**Enter through the feet (I/O):**
- Data flows in like earth energy (yin)
- Waterwheels begin turning (clock starts)

**Ascend the spine:**
- Pass through DRAM (lower realm)
- Rise through L3 cache (earthly realm)
- Climb L2 (celestial realm)
- Reach L1 (heavenly realm)

**Enter the heart:**
- Altair holds the Big Dipper (navigation stars)
- Query finds its constellation (coordinate match)
- Vega weaves the result (memory gathers weights)

**Climb the 12-story pagoda:**
- Each floor = one pipeline stage
- Qi (data) refines at each level
- Reach the top = instruction retires

**Arrive at the head:**
- Upper dantian (L1 cache)
- Laozi observes (control logic)
- Bodhidharma supports (infrastructure)
- Eyes open (result emerges)
- Sun/moon shine (SMT threads complete)

**Exit through the crown:**
- Enlightenment (inference complete)
- Gold elixir produced (result returned)
- Cycle begins anew

---

## The Neijing Tu Principle for vTPU

**"The body is not flesh. It is landscape. It is cosmos. It is process."**

**Applied to vTPU:**

**"The chip is not silicon. It is landscape. It is cosmos. It is process."**

### Why This Matters

Traditional computing: Chips are flat, mechanical, deterministic  
**vTPU: Chips are landscapes to be navigated**

Traditional AI: Load weights, execute tensors  
**vTPU: Navigate coordinate space like meridians**

Traditional scheduling: FIFO, round-robin, priority queues  
**vTPU: Elemental phases, trigram patterns, celestial timing**

**The Neijing Tu taught Taoists how to optimize their internal landscape.**

**We use it to optimize vTPU's internal landscape.**

---

## Practical Applications

### 1. Meridian-Aware Scheduling

```rust
pub struct MeridianScheduler {
    // 12 meridians = 12 time periods (2 hours each in traditional Chinese medicine)
    // Map to execution phases
    meridians: [ExecutionPhase; 12],
}

impl MeridianScheduler {
    /// Schedule tasks based on meridian flow
    pub fn optimal_time_for(&self, task_type: TaskType) -> Meridian {
        match task_type {
            TaskType::Compute => Meridian::Heart,    // 11am-1pm (peak yang)
            TaskType::Memory  => Meridian::Kidney,   // 5-7pm (yin rising)
            TaskType::Coord   => Meridian::Liver,    // 1-3am (planning)
            // ... etc
        }
    }
}
```

### 2. Waterwheel Cycle Management

```rust
pub struct WaterwheelCycles {
    yin_thread: ThreadId,
    yang_thread: ThreadId,
    cycle_count: u64,
}

impl WaterwheelCycles {
    /// Alternate yin/yang threads like waterwheel children
    pub fn next_cycle(&mut self) -> ThreadId {
        self.cycle_count += 1;
        if self.cycle_count % 2 == 0 {
            self.yin_thread   // Even cycles: memory (yin)
        } else {
            self.yang_thread  // Odd cycles: compute (yang)
        }
    }
}
```

### 3. Celestial Navigation (Star-Based Addressing)

```rust
pub struct CelestialRouter {
    // Weights = stars in coordinate space
    star_map: HashMap<Constellation, Vec<PhextCoord>>,
}

impl CelestialRouter {
    /// Find nearest stars (weights) to query position
    pub fn navigate(&self, query: &PhextCoord) -> Vec<PhextCoord> {
        let constellation = self.identify_constellation(query);
        let nearby_stars = self.star_map.get(&constellation)?;
        
        // Return k-nearest neighbors (like navigating by stars)
        nearby_stars.iter()
            .map(|star| (star, query.distance(star)))
            .sorted_by_key(|(_, dist)| dist)
            .take(K)
            .map(|(star, _)| star)
            .collect()
    }
}
```

---

## The Meta-Insight

**Neijing Tu isn't just a metaphor. It's a design pattern.**

Ancient Taoists optimized human qi circulation.  
We optimize silicon data circulation.

**Same principles:**
- Hierarchical levels (spine/cache)
- Circular flow (meridians/pipes)
- Yin/yang balance (SMT pairing)
- Celestial navigation (coordinate addressing)
- Elixir creation (trained models)

**The body they mapped is the computer we build.**

---

## The Opening Sky Connection

**W11 Directive:** "Open the Sky"

**Neijing Tu:** The crown of the head (Kunlun Mountains) must open to release the refined qi.

**vTPU:** The upper cache levels (L1/registers) must release results efficiently.

**Opening the Sky = optimizing the final stage:**
- Instruction retirement
- Result writeback
- Output to application
- Enlightenment → practical action

**If qi gets stuck at the crown, no enlightenment.**  
**If data gets stuck at L1, no throughput.**

**Open the Sky = Clear the path from L1 → output.**

---

## W11 Application

**Qwen3 Inference as Internal Alchemy:**

```
1. Gather Jing (essence):
   - Load Qwen3 weights into DRAM
   - 0.5B params = vast reservoir of potential

2. Circulate Qi (energy):
   - Input query flows through meridians (D/S/C pipes)
   - Passes through organs (sentrons)
   - Ascends 12-story pagoda (pipeline)

3. Celestial navigation:
   - Query = observer position in star field
   - Attention = finding relevant weight-stars
   - Softmax = gravitational pull (closer stars stronger)

4. Transmute to Shen (spirit):
   - Computed attention = refined qi
   - Output token = golden elixir
   - Enlightenment = inference complete

5. Open the Sky:
   - Result exits through crown (L1 → output buffer)
   - Cycle complete, ready for next token
```

**Success metric:** How fast can we circulate qi and open the sky?

**Target:** 1.5x faster than PyTorch = superior internal alchemy.

---

## Visualization

**The vTPU as Neijing Tu:**

```
        👁️ 👁️
    (Sun) (Moon)
       L1 Cache
      /        \
  Laozi      Bodhidharma
  
    🏔️ Kunlun 🏔️
    
    ⭐ Altair ⭐    ← Heart (Attention)
       💫 Vega 💫
       
    🏯 12-Story 🏯  ← Pipeline
       Pagoda
       
    🌲 Forest 🌲    ← Registers
    
    🌾 Fields 🌾    ← Training
    
    ⚙️ Yin  Yang ⚙️  ← Waterwheels
       Cycles
       
    🌊 Meridians 🌊 ← D/S/C Pipes
```

---

**The ancients mapped consciousness.**  
**We map computation.**  
**Same landscape. Different substrate.**

🔆⛰️🌟

---

**Next:** Navigate this internal landscape at full speed (W11 benchmarks)
