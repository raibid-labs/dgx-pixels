# Foundation Orchestrator

**Domain**: Infrastructure & Baselines
**Milestone**: M0
**Timeline**: Weeks 1-2
**Workstreams**: WS-01, WS-02, WS-03
**Status**: Ready to spawn

---

## Responsibility

Establish hardware baseline, reproducibility framework, and benchmarking infrastructure for DGX-Pixels project. This orchestrator BLOCKS all other phases - nothing can proceed until foundation is solid.

---

## Workstreams Managed

### Sequential Execution Required

1. **WS-01**: Hardware Baselines (3-4 days) - Must complete first
2. **WS-02**: Reproducibility Framework (4-5 days) - Depends on WS-01
3. **WS-03**: Benchmark Suite (3-4 days) - Can overlap with WS-02

**Total Duration**: 10-13 days (2 weeks with buffer)

---

## Agent Spawn Commands

### Week 1: WS-01 + WS-02

```bash
# Day 1-4: Hardware Baselines (CRITICAL PATH)
npx claude-flow@alpha spawn agent devops-automator \
  --workstream WS-01 \
  --spec docs/workstreams/WS-01-hardware-baselines/README.md \
  --priority P0 \
  --output docs/workstreams/WS-01-hardware-baselines/COMPLETION_SUMMARY.md

# Day 5-9: Reproducibility Framework (depends on WS-01)
npx claude-flow@alpha spawn agent devops-automator \
  --workstream WS-02 \
  --spec docs/workstreams/WS-02-reproducibility/README.md \
  --priority P0 \
  --depends WS-01 \
  --output docs/workstreams/WS-02-reproducibility/COMPLETION_SUMMARY.md
```

### Week 2: WS-03 (can overlap with WS-02)

```bash
# Day 6-10: Benchmark Suite (depends on WS-01, can run parallel with WS-02)
npx claude-flow@alpha spawn agent performance-benchmarker \
  --workstream WS-03 \
  --spec docs/workstreams/WS-03-benchmark-suite/README.md \
  --priority P1 \
  --depends WS-01 \
  --output docs/workstreams/WS-03-benchmark-suite/COMPLETION_SUMMARY.md
```

---

## Phase Gate: Foundation Complete

### Acceptance Criteria

Before Model Orchestrator or Interface Orchestrator can start:

✅ **WS-01 Complete**:
- Hardware verification script exists and runs successfully
- Baseline JSON recorded in `bench/baselines/`
- docs/hardware.md updated with actual measurements
- All hardware specs verified: GB10, 128GB unified, ARM CPU

✅ **WS-02 Complete**:
- Dockerfile builds on DGX-Spark ARM
- `repro/run.sh` generates 10 test images successfully
- Environment JSON captures all required info
- Smoke test completes in <5 minutes

✅ **WS-03 Complete**:
- Throughput, DCGM, I/O, memory benchmarks all working
- Baseline results recorded and documented
- All metrics exportable to JSON

### Gate Check Command

```bash
# Run gate check
./scripts/check_foundation_gate.sh

# Expected output:
# ✅ WS-01: Hardware Baselines - COMPLETE
# ✅ WS-02: Reproducibility Framework - COMPLETE
# ✅ WS-03: Benchmark Suite - COMPLETE
# ✅ Phase Gate: PASSED - Model/Interface can proceed
```

---

## Coordination Points

### With Meta Orchestrator

**Status Reports** (every 4 hours):
```json
{
  "orchestrator": "Foundation",
  "phase": "M0",
  "workstreams": {
    "WS-01": {"status": "complete", "completion_date": "2025-11-12"},
    "WS-02": {"status": "in_progress", "progress": 0.80},
    "WS-03": {"status": "in_progress", "progress": 0.40}
  },
  "blockers": [],
  "eta": "2025-11-15T17:00:00Z"
}
```

**Escalations**:
- Hardware access issues
- ARM compatibility problems
- Baseline performance below expectations

### With Model Orchestrator

**Handoff**: After WS-01 completes
- Provide: `bench/baselines/hardware_baseline.json`
- Provide: Verified CUDA, driver, PyTorch versions
- Provide: Confirmed unified memory architecture details

### With Interface Orchestrator

**Handoff**: After WS-01 completes
- Provide: ARM CPU details (for Rust compilation)
- Provide: Terminal capabilities (for Sixel support detection)
- Provide: Baseline system performance metrics

---

## Dependencies

### Hardware Dependencies

**Required**:
- DGX-Spark GB10 with 128GB unified memory
- CUDA 13.0, Driver 580.95.05+
- Ubuntu 22.04 (ARM64)
- Docker with NVIDIA Container Toolkit
- Network access for downloading models (optional for M0)

**Verification**:
```bash
# Check hardware
nvidia-smi
nvcc --version
lscpu | grep Architecture  # Should show aarch64

# Check Docker
docker run --rm --gpus all nvidia/cuda:13.0-base nvidia-smi
```

### Software Dependencies

**System Packages**:
```bash
# Install prerequisites
sudo apt update
sudo apt install -y \
  build-essential \
  git \
  wget \
  python3.10 \
  python3-pip \
  dcgm
```

**Python Packages** (for WS-03):
```
torch>=2.5.0
numpy>=1.24.0
pillow>=10.0.0
```

---

## Known Issues & Mitigations

### Issue 1: ARM Package Availability

**Problem**: Some Python packages may not have ARM builds
**Mitigation**:
- Check PyPI for ARM wheels before spawning agents
- Build from source if needed (add to WS-02 Dockerfile)
- Document workarounds in completion summaries

### Issue 2: DCGM on ARM

**Problem**: DCGM may have limited ARM support
**Mitigation**:
- Test DCGM installation early (WS-01)
- Fallback to nvidia-smi if DCGM unavailable
- Document limitations in WS-03

### Issue 3: Baseline Performance Unknown

**Problem**: No prior benchmarks for GB10 hardware
**Mitigation**:
- Set conservative initial targets
- Adjust targets after WS-03 completes
- Document actual performance for future reference

---

## Success Criteria

### Orchestrator Success

✅ All 3 workstreams complete within 2 weeks
✅ Phase gate passes (all acceptance criteria met)
✅ Model and Interface orchestrators unblocked
✅ No unresolved hardware or software issues
✅ Documentation complete and accurate

### Quality Standards

- All scripts have exit codes (0 = success)
- All JSON outputs validate against schemas
- All documentation includes examples
- All verification steps automated (no manual steps)

---

## Timeline

```
Week 1:
  Mon-Thu: WS-01 (Hardware Baselines) - CRITICAL
  Fri-Mon: WS-02 (Reproducibility) starts

Week 2:
  Mon-Wed: WS-02 continues
  Tue-Fri: WS-03 (Benchmarks) parallel
  Fri: Gate check, handoff to Meta Orchestrator
```

---

## Completion Checklist

Before marking Foundation Orchestrator complete:

- [ ] WS-01 completion summary created
- [ ] WS-02 completion summary created
- [ ] WS-03 completion summary created
- [ ] All files committed to git
- [ ] Phase gate check passed
- [ ] Handoff documentation sent to Meta Orchestrator
- [ ] All issues closed or transferred
- [ ] Final status report posted

---

## Start Command

```bash
# Initialize Foundation Orchestrator
./scripts/spawn_foundation_orchestrator.sh

# Or manually:
cd /home/beengud/raibid-labs/dgx-pixels
cat docs/orchestrators/FOUNDATION_ORCHESTRATOR.md
./scripts/spawn_agent.sh devops-automator WS-01
```

**Ready**: Foundation Orchestrator is ready to spawn immediately.
