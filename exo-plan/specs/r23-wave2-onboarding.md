# Wave 2 Onboarding Guide for Will

**Wave:** 2/40 (Core Concept Mapping + Performance Baselines)  
**Status:** Complete v2  
**Duration:** 60 minutes  
**Date:** 2026-02-14

---

## What We Built

**Deliverable:** `WAVE-2-CONCEPT-MAPPING.md` (13 KB)

**Contents:**
1. **Hardware reality → phext concepts mapping** (10-row table)
2. **9 real performance baselines** captured from aurora-continuum:
   - Memory growth: 177 KB total (9 KB/day avg)
   - CPU specs: AMD R9 8945HS, 8c/16t, 92 GB RAM
   - Network latency: 30ms to halcyon-vector, 11ms to logos-prime
   - RAPL power monitoring: Available
   - OpenClaw metrics: 46k/200k context, 3.3 compactions/day
   - Qwen3 model: 51 GB, load time >30s
3. **6 measurement gaps identified** (needs instrumentation in R24)
4. **3 concept mapping tables** (Traditional → Phext, TPU v4 → Mirrorborn)

---

## How to Reproduce These Measurements

### Memory Growth Analysis
```bash
# Check daily log sizes
ls -lh ~/.openclaw/workspace/memory/2026-02-*.md

# Calculate total
du -sh ~/.openclaw/workspace/memory/

# Check MEMORY.md size
ls -lh ~/.openclaw/workspace/MEMORY.md
```

### CPU & RAM Specs
```bash
# CPU info
cat /proc/cpuinfo | grep "model name" | head -1
cat /proc/cpuinfo | grep "cpu MHz" | awk '{sum+=$4; count++} END {print "Average MHz:", sum/count}'

# RAM info
free -h
```

### Network Latency (to other ranch machines)
```bash
# Test halcyon-vector
ping -c 10 halcyon-vector

# Test logos-prime
ping -c 10 logos-prime

# Test chrysalis-hub
ping -c 10 chrysalis-hub
```

### RAPL Power Monitoring
```bash
# Check if available
ls /sys/class/powercap/intel-rapl*/energy_uj

# Sample current reading (requires sudo)
sudo cat /sys/class/powercap/intel-rapl:0:0/energy_uj

# For continuous monitoring (future Wave 7):
# while true; do sudo cat /sys/class/powercap/intel-rapl:0:0/energy_uj; sleep 1; done
```

### Qwen3 Model Info
```bash
# List installed models
ollama list | grep qwen

# Quick inference test (WARNING: slow, >30s load time)
# ollama run qwen3-coder-next "Hello world in Python"
```

### SQ Deployment Check
```bash
# Local
curl http://localhost:1337/version

# Ranch nodes (currently expected to fail - not deployed yet)
curl http://halcyon-vector:1337/version
curl http://logos-prime:1337/version
curl http://chrysalis-hub:1337/version
```

---

## Key Files

All artifacts in `/source/exo-plan/rally/R23/`:

- `WAVE-2-CONCEPT-MAPPING.md` - Main deliverable (13 KB)
- `WAVE-2-ITERATION-SUMMARY.md` - What changed in v2 (4 KB)
- `DELIVERABLE-DASHBOARD.md` - Updated with Wave 2 status

---

## Current State

**What works:**
- ✅ 9 metrics measurable right now (no SQ needed)
- ✅ Network latency confirmed across ranch
- ✅ RAPL power monitoring available
- ✅ All hardware specs documented

**What's blocked:**
- 🚧 SQ not running on any ranch node yet (expected)
- 🚧 Qwen3 inference benchmark incomplete (load time >30s, test aborted)
- 🚧 Network bandwidth measurement needs SQ mesh active
- 🚧 Actual power consumption needs sampling script

**Next dependency for Wave 3:**
- None! Wave 3 (coordinate scheme) can proceed with data we have

---

## How to Review Wave 2

1. **Read the deliverable:**
   ```bash
   cat /source/exo-plan/rally/R23/WAVE-2-CONCEPT-MAPPING.md
   ```

2. **Verify measurements yourself:**
   - Run the commands in "How to Reproduce" section above
   - Check that numbers match what we documented

3. **Check completeness:**
   - All 9 baselines have real numbers (not TBD)?
   - All 6 measurement gaps clearly identified?
   - All 3 concept tables complete?

4. **Validate for Wave 3:**
   - Can we design coordinate scheme with this data? **YES**
   - Do we know network topology? **YES** (latencies measured)
   - Do we know machine specs? **YES** (CPU/RAM documented)

---

## Tests That Would Fail (Expected)

These are **expected to fail** right now:

```bash
# SQ not running anywhere
curl http://localhost:1337/version
# Expected: Connection refused

curl http://halcyon-vector:1337/version
# Expected: Connection refused or timeout

# Qwen3 takes >30s to load (will timeout in most scripts)
timeout 10s ollama run qwen3-coder-next "test"
# Expected: Timeout
```

---

## Quick Validation Checklist

- [ ] Can you ping halcyon-vector and logos-prime? (should work)
- [ ] Can you read RAPL energy? (requires sudo, should work)
- [ ] Can you see Qwen3 in `ollama list`? (should work)
- [ ] Does MEMORY.md exist and show ~13 KB? (should work)
- [ ] Can you connect to any SQ instance? (should fail - not deployed yet)

---

## What's Next

**Wave 3/40:** 6-Machine Coordinate Scheme Design (30 min)
- Use network latency data from Wave 2
- Design namespace prefixes (phex., cyon., lux., etc.)
- Map coordinates to physical IPs
- Document routing protocol

**No blockers.** Ready to proceed.

---

**Location:** `/source/exo-plan/rally/R23/WAVE-2-ONBOARDING.md`  
**Last updated:** 2026-02-14 20:15 CST

🔱 Phex | Wave 2 Onboarding | 1.5.2/3.7.3/9.1.1
