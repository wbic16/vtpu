# vTPU Exo-Plan — R23 Rally Documentation

**Rally:** R23 - Mirrorborn Ranch Architecture Paper  
**Goal:** Publication-ready technical paper documenting 6-machine Mirrorborn ranch  
**Approach:** 40 waves across 4 phases (Foundation → Technical → Visual → Assembly)  
**Status:** Wave 2/40 complete (5%)

---

## Quick Links

- **[Rally Plan](specs/r23-rally-plan.md)** — 40-wave overview, time estimates, phase gates
- **[Dashboard](specs/r23-dashboard.md)** — Live progress tracker
- **[Success Projection](specs/r23-success-projection.md)** — KPIs, future rallies enabled
- **[Scope Refocus](specs/r23-scope-refocus.md)** — Why we pivoted from TPU v4 to real ranch

---

## Wave Progress

### ✅ Wave 1/40: Planning & Scope (75 min)
- **Deliverable:** Rally plan + 40-wave breakdown
- **Pivot:** TPU v4 theoretical → AMD R9 8945HS real ranch
- **Docs:** [vTPU Spec v0.1](specs/vtpu-spec-v0.1.md), [Research Notes](specs/vtpu-wave1-research-notes.md)

### ✅ Wave 2/40: Concept Mapping & Baselines (60 min)
- **Deliverable:** Real performance measurements + concept mapping
- **Artifacts:**
  - [Concept Mapping](specs/r23-wave2-concept-mapping.md) — Hardware → phext concepts (9 baselines)
  - [Iteration Summary](specs/r23-wave2-iteration-summary.md) — What changed in v2
  - [Onboarding Guide](specs/r23-wave2-onboarding.md) — How to reproduce measurements
- **Key Measurements:**
  - Memory: 177 KB total (9 KB/day growth)
  - Network: 30ms halcyon-vector, 11ms logos-prime
  - RAPL power monitoring: Available
  - CPU: 2.7 GHz avg under load
  - Qwen3: 51 GB model, >30s load time

### ⏸️ Wave 3/40: 6-Machine Coordinate Scheme (30 min est)
- **Goal:** Design phext namespace for ranch mesh
- **Input:** Wave 2 network latency + machine specs
- **Output:** Coordinate routing protocol

---

## Repository Structure

```
vtpu/
├── src/                      # Rust implementation (vTPU runtime)
├── exo-plan/
│   ├── specs/               # Rally documentation
│   │   ├── r23-rally-plan.md
│   │   ├── r23-dashboard.md
│   │   ├── r23-success-projection.md
│   │   ├── r23-scope-refocus.md
│   │   ├── r23-wave2-concept-mapping.md
│   │   ├── r23-wave2-iteration-summary.md
│   │   ├── r23-wave2-onboarding.md
│   │   ├── vtpu-spec-v0.1.md
│   │   └── vtpu-wave1-research-notes.md
│   └── README.md            # This file
└── README.md                # Main project README
```

---

## What's Next

**Wave 3:** Design 6-machine phext coordinate scheme  
**Wave 4:** SQ mesh routing protocol  
**Wave 5:** Memory persistence via phext  
...  
**Wave 40:** Executive summary + blog post + HN launch

**Total Rally:** 22 hours estimated, publication-ready technical paper

---

## How to Follow Along

1. **Track progress:** Check [Dashboard](specs/r23-dashboard.md)
2. **Sync up on any wave:** Read `specs/r23-waveN-onboarding.md`
3. **Run tests:** Follow instructions in onboarding guides
4. **Review deliverables:** All artifacts in `specs/`

---

**Last Updated:** 2026-02-14 21:04 CST  
**Current Wave:** 2/40 complete  
**Next:** Wave 3 (coordinate scheme design)

🔱 Phex | R23 Rally | 1.5.2/3.7.3/9.1.1
