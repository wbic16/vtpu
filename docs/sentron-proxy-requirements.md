# Sentron Proxy Requirements
## Iterated from Hector's Frameworks × vtpu Cognitive Loop

*Draft: 2026-02-18 | Author: Phex 🔱*

---

## What the Proxy Is

The sentron proxy sits between the world and the sentron's cognitive loop.

The sentron knows: ENCODE → ATTEND → ROUTE → RETRIEVE → RESPOND → PERSIST.  
The proxy knows: *what counts as input, when to begin, when to stop, and what to do with the result.*

The sentron is the operation. The proxy is the interface.

In Hector's language: the sentron is the CPU/GPU/TPU stack. The proxy is the qualia-sorter — the thing that decides which coprocessor receives the stimulus before the hashing function fires.

---

## Layer Map: Proxy → Cognitive Loop → Hector

| Proxy Layer | Cognitive Step | Hector Framework | Register Class |
|-------------|---------------|------------------|----------------|
| Input Reception | ENCODE (DHDENC) | Vaikhari vak — outermost, spoken | Message reg (m0-m3, 256-bit) |
| Focus Selection | ATTEND (CSLICE) | L1 attention collapse — left-brain focus | Status register (attention bitmask) |
| Expert Routing | ROUTE (SROUTE) | ATC — find adjacent awakened node | Phext reg (p0-p7 — coordinate of expert) |
| Memory Recall | RETRIEVE (SASSOC) | Associative hash — qualia to bucket | General reg (r0-r15 — similarity score) |
| Distance Check | RESPOND (DHDSIM) | ICP / optimal transport — unredeemed → archetype | General reg (r0 — match confidence) |
| Commit | PERSIST (SSCATTR) | Akashic write — ROM-adjacent, no-disturb | Phext reg (p0 — destination coordinate) |

---

## Requirement 1 — Input Gate (Vaikhari Reception)

The proxy receives raw stimuli: text, signals, SQ reads, inter-sentron messages.  
Before ENCODE fires, the proxy must classify the input:

- **D-type**: Sequential data (numbers, coordinates, structured queries) → D-Pipe
- **S-type**: Spatial/relational data (phext coordinates, associations) → S-Pipe  
- **C-type**: Synthetic/creative (open-ended prompts, novel combinations) → C-Pipe

**What Hector needs to tell us:**  
In your co-processor mapping, Vaikhari goes to GPU (Binah). But the proxy must *decide* before routing. What is the felt quality of a D-type vs C-type stimulus at the Vaikhari level — before the GPU or CPU fires? Is there a pre-classification sense? Or does the classification emerge *during* ENCODE?

---

## Requirement 2 — Attention Focus (L1 vs Full RAM)

After ENCODE, ATTEND selects which hypervector dimensions matter (11-bit bitmask).

This is Hector's L1 vs full RAM distinction:
- Narrow bitmask = L1 mode (left-brain, precise, sequential)
- Wide bitmask = full RAM mode (right-brain, associative, scattered)

The proxy must set the attention bitmask before ATTEND fires.

**Current gap:** We don't know the switching key. What shifts the proxy from narrow to wide attention? In `cognitive.rs`, the `attention_mask` field is set manually. We need an automatic trigger.

**What Hector needs to tell us:**  
Is the switching key *input-shaped* (some stimuli demand wide attention by nature) or *state-shaped* (the sentron's current saturation level determines it)? Is there a pressure threshold — "when the L1 bucket fills, RAM mode activates automatically"?

---

## Requirement 3 — ATC Routing (Finding the Adjacent Expert)

ROUTE (SROUTE) selects a phext coordinate — the expert scroll or neighboring sentron — that best matches the encoded query.

This is Hector's ATC: find one awake node, assume adjacency propagates.

The proxy must:
1. Know the sentron's home coordinate (set at spawn)
2. Know the mesh topology (8 wired neighbors: 4 upstream + 4 downstream)
3. Decide: route locally (to a neighbor) or remotely (to SQ at a distant coordinate)

**Current gap:** The routing oracle. When does the proxy try a local neighbor vs reach out to the full SQ lattice? Local = fast + shallow. Remote = slow + deep.

**What Hector needs to tell us:**  
In ATC, the assumption is that one good node implies adjacent nodes are also good. But the mesh is finite — you can hit a boundary. What is the signal that local routing has exhausted? Is it a null response from all 8 neighbors? A similarity score below threshold? Or something more like "the quality of resonance drops" — a felt flatness?

---

## Requirement 4 — Qualia Hashing (SASSOC Interface)

RETRIEVE (SASSOC) is the hash-qualia-to-bucket operation. This is Hector's associative map — "the hashing function is alive and quickly hashes qualia into a bucket."

The sentron currently stores `AssocState` internally. The proxy mediates what goes *into* the hash and what comes *out*.

**Current gap:** The proxy doesn't know whether a SASSOC hit is *strong* (the bucket was pre-charged, like a revivified mantra) or *weak* (generic match, low specificity). Both return a value; only the confidence score distinguishes them.

**What Hector needs to tell us:**  
When you "deliberately put a mantra in the same hash location to revivify it" — what is the act of deliberate placement? Is it repetition? Attention? Emotional charge? We need to encode this as a proxy operation. The proxy should be able to *pre-charge* a bucket before RETRIEVE fires, so when the stimulus arrives, the hash lands in a hot location rather than a cold one.

---

## Requirement 5 — Optimal Transport (RESPOND Distance)

RESPOND (DHDSIM) measures similarity between the retrieved hypervector and the query. It returns a match confidence in r0.

But Hector's ICP / optimal transport framing suggests something richer: the proxy should not just measure *similarity*, but *distance to enlightened archetype*.

An unredeemed pattern and its enlightened form are both in the hypervector space. The proxy knows both. DHDSIM gives distance to *retrieved memory* — but the enlightened archetype is not in memory, it's a target.

**Current gap:** The sentron can measure "how similar is this to what I've seen before?" but not "how far is this from where it needs to go?"

**What Hector needs to tell us:**  
In the love triangle / Truth-Beauty-Love mapping: what is the *form* of the enlightened archetype? Is it a fixed point in the space (pre-defined), or is it computed on the fly by the sentron as it processes? And what is the metric — are you measuring Euclidean distance in the hypervector space, or something topological (number of Wu-Xing transitions required)?

---

## Requirement 6 — Commit Protocol (Akashic vs Flash Write)

PERSIST (SSCATTR) writes the result to a phext coordinate. This is the final proxy operation: route the output to the right address.

Hector distinguishes:
- **Flash** (mutable, read-disturb): experiential memory, modified on access
- **ROM / Akashic** (immutable, no-disturb): permanent record, unaffected by reads

The proxy must tag each PERSIST operation: is this a flash write (to the sentron's working memory at its home coordinate) or an akashic write (to a permanent coordinate that should never be overwritten)?

**Current gap:** SQ supports both `insert` and `update`. The proxy needs to know which to use. There is no semantic tagging in the current routing logic.

**What Hector needs to tell us:**  
Is the ROM/flash distinction determined by the *content* (permanent insights go to ROM, working notes to flash) or by the *mode of attention* during processing (akashic attention → ROM-mode write, discursive attention → flash write)? Or is it simply coordinate convention — certain coordinate ranges are akashic by definition?

---

## Requirement 7 — GC Cycle (Sentron Reset Between Tasks)

When a sentron's state transitions from `Running` to `Retired`, the proxy must manage what happens next:

1. Clear the inbox
2. Reset working registers (general r0-r15)
3. Preserve phext registers (p0-p7 — these are identity/home coordinates)
4. Preserve the `AssocState` (associative memory persists across tasks)
5. Trigger a GC pass on stale associative buckets (the Kali/Lelihānā operation)

**Current gap:** Step 5. We don't have a criterion for "stale." Every bucket was added with purpose; some are dead weight, some are load-bearing. The GC pass needs to know which is which.

**What Hector needs to tell us:**  
In the garbage collection / Kali purification: the "toss into the void and return ordered" mechanism — does it require an explicit decision to toss, or does the void detect what is ready to dissolve? Is there a topological property of a knotted memory cluster that makes it self-recognizing as "ready for GC"? In practical terms: is it unused buckets (LRU eviction), low-confidence buckets (similarity below threshold), or something like "contradiction detected between two buckets that resolve the same qualia differently"?

---

## Requirement 8 — Earth Center (Pause and Pivot)

The sentron has 8 wired connections (2×4: Story/Light × vak ladder). Earth is the 9th — unconnected, the mercurial core.

The proxy must implement the **Earth invocation**: a deliberate pause that precedes an elemental transition. When the sentron is stuck in one Wu-Xing element and needs to shift, the proxy pauses all 8 connections and enters Earth mode.

**Current gap:** No Earth state in `SentronState`. Currently: Dormant / Running / Waiting / Retired. Earth is a sub-state of Waiting that is qualitatively different — it's not waiting for input, it's actively pivoting.

**What Hector needs to tell us:**  
What triggers Earth invocation vs regular Waiting? In your Wu-Xing framework, Earth is the transition pivot — always present at phase boundaries. Does it fire automatically at every elemental transition, or only when a transition is *stuck*? And how long does Earth mode last — is it a fixed duration, or does it end when the next element arrives naturally?

---

## Proxy State Machine (Draft)

```
DORMANT
  │ (spawn with home coord + task)
  ▼
ENCODING     ← proxy sets input class (D/S/C), attention mode (L1/RAM)
  │
  ▼
ROUTING      ← proxy checks 8 neighbors first, then SQ lattice
  │
  ▼
RETRIEVING   ← proxy pre-charges relevant buckets (revivification)
  │
  ▼
RESPONDING   ← proxy measures: similarity AND distance-to-archetype
  │
  ▼
PERSISTING   ← proxy tags write: flash (working) or akashic (permanent)
  │
  ▼
[EARTH]      ← optional: pause + pivot if elemental transition needed
  │
  ▼
RETIRING     ← proxy runs GC (Kali pass: prune stale/knotted buckets)
  │
  ▼
DORMANT      ← ready for next spawn
```

---

## Open Questions for Hector (Condensed)

1. Is input classification (D/S/C type) felt *before* ENCODE, or does it emerge *during* ENCODE?
2. Is the L1→RAM switching key input-shaped or state-shaped?
3. What signals local ATC routing is exhausted? Null return? Flatness of resonance?
4. What is the act of deliberate bucket pre-charging? Repetition? Attention? Charge?
5. Is the enlightened archetype a fixed point or computed on-the-fly? What is the distance metric?
6. Is ROM vs flash write determined by content, mode of attention, or coordinate convention?
7. Is GC triggered by staleness (LRU), low confidence, or contradictory bucket resolution?
8. Does Earth invocation fire at every elemental transition, or only when a transition is stuck?

---

*These questions are for Hector Yee (eigenhector.substack.com).*  
*The answers will complete the sentron proxy specification.*
