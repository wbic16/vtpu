# R23 Scope Refocus: AMD R9 8945HS + Qwen3-Coder-Next

**Date:** 2026-02-14  
**Decision:** Shift from theoretical TPU v4 rewrite to real ranch architecture  
**Rationale:** We have actual hardware + actual workloads = credible measurements, not projections

---

## Original Scope (Wave 1)

**Target:** Rewrite Google TPU v4 paper using phext concepts  
**Problem:** PBC doesn't exist. Performance numbers are theoretical projections.  
**Risk:** "Estimated speedup" claims lack credibility without real measurements.

---

## New Scope

**Target:** Document Mirrorborn ranch architecture using phext + real performance data  
**Hardware:** AMD R9 8945HS platforms (6 machines: aurora-continuum, halcyon-vector, logos-prime, chrysalis-hub, lilly, aletheia-core)  
**Software:** Qwen3-Coder-Next via OpenClaw + SQ phext storage  
**Advantage:** All claims backed by actual measurements, not theory.

---

## Real Hardware Specs (AMD R9 8945HS)

**Architecture:** Zen 4 (4nm)  
**Cores:** 8 cores, 16 threads  
**Base clock:** 4.0 GHz  
**Boost clock:** 5.2 GHz  
**Memory:** 96 GB DDR5 (aurora-continuum, halcyon-vector, logos-prime, chrysalis-hub)  
**TDP:** 35-54W (configurable)  
**NPU:** Ryzen AI (XDNA, up to 16 TOPS INT8)  

**Why this matters:** 96 GB = can run Qwen3-Coder-Next (32B params @ FP16 = 64 GB model + 32 GB context)

---

## Real Software Stack

**Model:** Qwen3-Coder-Next (Ollama)  
- 32B parameters (code-optimized)
- Context: 128K tokens
- Quantization: Q8_0 (8-bit activations)
- Memory: ~40 GB loaded

**Runtime:** OpenClaw  
- Persistent AI sessions
- Memory: MEMORY.md + daily logs
- Tools: exec, browser, SQ, message

**Storage:** SQ (Scrollspace Query)  
- Phext-native database (port 1337)
- Multi-tenant: 6 machines on ranch mesh
- Coordinates: volume.book.chapter/section.scroll

**Mesh networking:**  
- All 6 machines run SQ on port 1337
- Peer-to-peer phext sync
- Coordinates route between machines

---

## Paper Reframe: "Phext-Native AI Infrastructure on AMD Workstations"

**New title:** *Mirrorborn: Distributed AI Coordination via Phext Addressing on AMD R9 8945HS Platforms*

**Abstract (updated):**
> We present Mirrorborn, a distributed AI coordination system using 9-dimensional phext addressing to enable persistent memory and peer-to-peer synchronization across commodity AMD workstations. Six AMD R9 8945HS machines (96 GB DDR5 each) run Qwen3-Coder-Next (32B parameters) via OpenClaw, storing persistent memory in SQ (phext-native database). Phext coordinates (volume.book.chapter/section.scroll) replace traditional file paths, enabling fault-tolerant memory synchronization across the mesh. Measurements show X% overhead for coordinate-based routing vs flat files, while enabling Y days of continuous memory persistence. The system demonstrates phext's applicability to real distributed AI infrastructure, not just text storage.

**Key sections (updated from TPU v4 framing):**

1. **Introduction:** Distributed AI coordination problem (not supercomputing)
2. **Background:** Phext addressing + AMD R9 8945HS specs
3. **Architecture:** 6-machine mesh, SQ on each node, coordinate routing
4. **Qwen3-Coder-Next Integration:** How OpenClaw uses phext for persistent memory
5. **Memory Persistence:** MEMORY.md + daily logs stored as phext coordinates
6. **Mesh Synchronization:** Peer-to-peer phext sync between machines
7. **Performance Measurements:** Coordinate lookup overhead, sync latency, memory growth
8. **Fault Tolerance:** Machine offline → reroute to peer coordinates
9. **Comparison to Traditional:** Flat files vs phext coordinates (measured overhead)
10. **Discussion:** Phext for distributed AI coordination, not just text
11. **Related Work:** Distributed AI systems, memory persistence, P2P sync
12. **Conclusion:** Phext-native infrastructure scales to 6 machines, ready for more

---

## What We Can Measure (Real Data)

**Performance metrics:**
- Coordinate lookup time vs flat file read (ns)
- SQ sync latency between machines (ms)
- Memory growth rate (MB/day per Mirrorborn)
- Context window persistence (how long MEMORY.md survives)
- Mesh availability (uptime when 1-2 machines offline)

**Workload characteristics:**
- Qwen3-Coder-Next inference time (tokens/sec)
- Memory file sizes (MEMORY.md, daily logs)
- Tool call patterns (exec, browser, SQ queries)
- Coordination patterns (6 Mirrorborn messaging)

**Infrastructure metrics:**
- Power consumption (W per machine under load)
- Network bandwidth (GB/day phext sync)
- Storage growth (phext files accumulated)

**All measurable today.** No theoretical projections needed.

---

## Updated 40-Wave Plan (Same Structure, New Content)

**Phase 1: Foundation (Waves 1-10)**
- Wave 2: AMD R9 8945HS specs + Qwen3 capabilities
- Wave 3: 6-machine mesh coordinate scheme
- Wave 4: SQ mesh networking (port 1337 routing)
- Wave 5: OpenClaw memory persistence via phext
- Wave 6: Fault tolerance (machine offline scenarios)
- Wave 7: Measurement methodology (what we track)
- Wave 8: Qwen3-Coder-Next workload patterns
- Wave 9: Memory growth analysis (14 days of MEMORY.md data)
- Wave 10: Peer-to-peer sync protocol

**Phase 2: Technical Sections (Waves 11-25)**
- Same structure, different content (AMD/Qwen3 instead of TPU/OCS)

**Phase 3: Figures & Tables (Waves 26-35)**
- Figure 1: 6-machine mesh topology
- Figure 2: Phext coordinate routing between machines
- Figure 3: OpenClaw memory persistence architecture
- Figure 4: Qwen3-Coder-Next integration with SQ
- Figure 5: Fault tolerance (machine offline recovery)
- Table 1: AMD R9 8945HS specifications
- Table 2: Performance measurements (coordinate lookup, sync latency)
- Table 3: Memory growth (14 days of data)
- Table 4: Workload characteristics (Qwen3 inference patterns)
- Table 5: Comparison to flat files (overhead analysis)

**Phase 4: Assembly & Polish (Waves 36-40)**
- Same (assemble, review, LaTeX, exec summary)

---

## Advantages of New Scope

**Credibility:**
- Real hardware (can verify specs)
- Real measurements (can reproduce)
- Real system (running today, not hypothetical)

**Novelty:**
- First distributed AI system using phext addressing
- First demonstration of phext mesh networking (6 machines)
- First persistent AI memory using phext coordinates (14+ days continuous)

**Practicality:**
- Reproducible (commodity AMD hardware, open-source software)
- Scalable (works on 6 machines, can add more)
- Useful (solves real problem: distributed AI coordination)

**Honesty:**
- No theoretical projections ("estimated 6.8x speedup")
- Just measured data ("coordinate lookup adds 47ns overhead")
- Claims we can defend

---

## What Changes in Waves 2-40

**Keep:**
- 40-wave structure
- Phase 1-4 organization
- Review gates
- Quality standards

**Change:**
- TPU v4 → AMD R9 8945HS everywhere
- OCS → SQ mesh networking
- 4096 chips → 6 machines (expandable to 500+ via coordinate space)
- SparseCore → Qwen3-Coder-Next memory integration
- Theoretical performance → Measured overhead

**Timeline:**
- Still ~21 hours (same wave structure, different content)

---

## Next Steps

**Pre-Wave 2 (5 minutes):**
1. Download HTML version of TPU v4 paper (reference for structure only)
2. Gather AMD R9 8945HS specs
3. Check SQ logs for actual performance data (coordinate lookup times)
4. Review MEMORY.md growth over 14 days

**Wave 2/40 (30 minutes):**
- Core concept mapping: TPU v4 concepts → Mirrorborn ranch equivalents
- AMD R9 8945HS → replaces TPU v4 chip
- SQ mesh → replaces OCS
- Qwen3-Coder-Next → replaces ML workload
- Phext coordinates → replaces network topology

**Ready to proceed?** This is a much stronger paper. Real system > theoretical rewrite. 🔱

---

**Status:** Scope refocused, plan updated, ready for Wave 2  
**Commit:** Pending (after Will confirms this direction)

🔱 Phex | R23 Scope Refocus | 1.5.2/3.7.3/9.1.1
