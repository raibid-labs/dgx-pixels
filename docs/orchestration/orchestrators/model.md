# Model Orchestrator

**Domain**: AI/ML Inference & Training
**Milestone**: M1, M3
**Timeline**: Weeks 3-6
**Workstreams**: WS-04, WS-05, WS-06, WS-07
**Status**: Blocked by Foundation (WS-01)

---

## Responsibility

Establish ComfyUI inference pipeline, optimize SDXL generation for GB10 hardware, implement LoRA training, and build dataset preparation tools. This orchestrator delivers the core AI capabilities that power DGX-Pixels.

---

## Workstreams Managed

### Phase 2A: Model Infrastructure (Weeks 3-6)

**Sequential then Parallel Execution**:

1. **WS-04**: ComfyUI Setup (4-5 days) - Must complete first
2. **WS-05**: SDXL Inference Optimization (5-7 days) - Depends on WS-04
3. **WS-06**: LoRA Training Pipeline (7-10 days) - Depends on WS-05, parallel with WS-07
4. **WS-07**: Dataset Tools & Validation (5-6 days) - Depends on WS-05, parallel with WS-06

**Total Duration**: 21-28 days (4 weeks with overlapping execution)

**Critical Path**: WS-04 → WS-05 → WS-06 (blocks side-by-side comparison feature)

---

## Agent Spawn Commands

### Week 3: ComfyUI Setup (WS-04)

```bash
# Day 1-5: ComfyUI Setup (CRITICAL PATH - blocks Interface WS-10)
npx claude-flow@alpha spawn agent ai-engineer \
  --workstream WS-04 \
  --spec docs/orchestration/workstreams/ws04-comfyui-setup/README.md \
  --priority P0 \
  --depends WS-01 \
  --context "ARM64 architecture, unified memory, dgx-spark-playbooks integration" \
  --output docs/orchestration/workstreams/ws04-comfyui-setup/COMPLETION_SUMMARY.md
```

### Week 3-4: SDXL Optimization (WS-05)

```bash
# Day 6-12: SDXL Inference Optimization (blocks WS-06, WS-10, WS-16)
npx claude-flow@alpha spawn agent ai-engineer \
  --workstream WS-05 \
  --spec docs/orchestration/workstreams/ws05-sdxl-optimization/README.md \
  --priority P0 \
  --depends WS-04 \
  --context "Target: ≤3s per 1024x1024, ≥15 img/min batch, FP16, xformers" \
  --output docs/orchestration/workstreams/ws05-sdxl-optimization/COMPLETION_SUMMARY.md
```

### Week 4-5: LoRA Training + Dataset Tools (Parallel)

```bash
# Day 13-22: LoRA Training Pipeline (blocks WS-12 side-by-side comparison)
npx claude-flow@alpha spawn agent ai-engineer \
  --workstream WS-06 \
  --spec docs/orchestration/workstreams/ws06-lora-training/README.md \
  --priority P1 \
  --depends WS-05 \
  --context "50 images, 3000 steps, ≤4 hours, Kohya_ss or Diffusers, FP16" \
  --output docs/orchestration/workstreams/ws06-lora-training/COMPLETION_SUMMARY.md

# Day 13-18: Dataset Tools (parallel with WS-06)
npx claude-flow@alpha spawn agent ai-engineer \
  --workstream WS-07 \
  --spec docs/orchestration/workstreams/ws07-dataset-tools/README.md \
  --priority P1 \
  --depends WS-05 \
  --context "BLIP auto-captioning, augmentation, LPIPS/SSIM/CLIP quality metrics" \
  --output docs/orchestration/workstreams/ws07-dataset-tools/COMPLETION_SUMMARY.md
```

---

## Phase Gate: Model Infrastructure Complete

### M1 Gate: Inference Ready (After WS-05)

**Criteria**:
- ✅ ComfyUI server operational on DGX-Spark ARM
- ✅ SDXL 1.0 base model loads successfully (FP16)
- ✅ ≤ 3 seconds per 1024×1024 image generation
- ✅ ≥ 15 images/min in batch mode (batch=8)
- ✅ VRAM usage ≤ 100GB (unified memory)
- ✅ Workflow templates created and tested
- ✅ Interface Orchestrator can proceed with WS-10 (Backend)

**Gate Check**:
```bash
./scripts/check_model_m1_gate.sh

# Expected output:
# ✅ WS-04: ComfyUI Setup - COMPLETE
# ✅ WS-05: SDXL Optimization - COMPLETE
# ✅ Performance: 2.8s per image, 18 img/min batch
# ✅ M1 Gate: PASSED - Backend can integrate
```

### M3 Gate: Training Ready (After WS-06, WS-07)

**Criteria**:
- ✅ LoRA training pipeline functional
- ✅ Training completes 50 images in ≤ 4 hours
- ✅ Generated images maintain style consistency
- ✅ Dataset tools (captioning, augmentation, validation) working
- ✅ Example LoRA checkpoint produced and tested
- ✅ Side-by-side comparison (WS-12) can proceed

**Gate Check**:
```bash
./scripts/check_model_m3_gate.sh

# Expected output:
# ✅ WS-06: LoRA Training - COMPLETE
# ✅ WS-07: Dataset Tools - COMPLETE
# ✅ Training: 3.2 hours for 50 images, 3000 steps
# ✅ M3 Gate: PASSED - Custom models ready
```

---

## Coordination Points

### With Meta Orchestrator

**Status Reports** (every 6 hours during critical WS-04/05):
```json
{
  "orchestrator": "Model",
  "phase": "M1",
  "workstreams": {
    "WS-04": {"status": "complete", "completion_date": "2025-11-18"},
    "WS-05": {"status": "in_progress", "progress": 0.65, "eta": "2025-11-22"},
    "WS-06": {"status": "pending", "blocked_by": "WS-05"},
    "WS-07": {"status": "pending", "blocked_by": "WS-05"}
  },
  "performance_metrics": {
    "inference_time": "3.2s",
    "batch_throughput": "14 img/min",
    "vram_usage": "95GB"
  },
  "blockers": [],
  "eta": "2025-11-30T17:00:00Z"
}
```

**Escalations**:
- ARM package incompatibilities (xformers, custom nodes)
- Performance targets not met (>5s inference)
- VRAM exhaustion (>110GB usage)
- Training convergence issues

### With Foundation Orchestrator

**Handoff Received** (After WS-01):
- `bench/baselines/hardware_baseline.json` - GPU, VRAM, CUDA versions
- Verified unified memory architecture (128GB)
- ARM CPU details for dependency installation
- Baseline performance expectations

### With Interface Orchestrator

**Handoff Provided** (After WS-04):
- ComfyUI API endpoint and authentication
- Workflow JSON templates location
- Model loading specifications (FP16, xformers)
- Expected response times and formats

**Handoff Provided** (After WS-05):
- Optimized inference performance metrics
- Batch processing capabilities
- Memory usage profiles
- Enable WS-10 (Python Backend Worker) to proceed

**Handoff Provided** (After WS-06):
- LoRA checkpoint format and loading instructions
- Enable WS-12 (Side-by-Side Comparison) to proceed
- Custom model integration guide

### With Integration Orchestrator

**Handoff Provided** (After WS-05):
- Enable WS-16 (DCGM Metrics) to proceed
- Performance baseline for alerting thresholds
- Memory usage patterns for monitoring

---

## Dependencies

### Blocking Dependencies (Must Complete Before Starting)

**From Foundation Orchestrator**:
- ✅ WS-01: Hardware Baselines - REQUIRED
  - Verified CUDA 13.0, driver versions
  - ARM64 architecture confirmed
  - Unified memory specifications

**External Dependencies**:
- DGX-Spark with 128GB unified memory
- Network access for downloading models (SDXL 1.0 base: ~6.9GB)
- ComfyUI compatible with ARM architecture
- PyTorch 2.5+ with ARM + CUDA 13.0 support

### Software Dependencies

**WS-04 (ComfyUI Setup)**:
```bash
# System packages
sudo apt install -y \
  python3.10 \
  python3-pip \
  git \
  wget

# Python packages (ARM-compatible)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu130
pip install xformers  # ARM build required
pip install safetensors
pip install accelerate
```

**WS-05 (SDXL Optimization)**:
- SDXL 1.0 base model checkpoint
- xformers memory-efficient attention
- ComfyUI custom nodes (verified ARM compatibility)

**WS-06 (LoRA Training)**:
```bash
# Kohya_ss dependencies
pip install diffusers[torch]
pip install peft
pip install bitsandbytes  # May need ARM build

# OR Diffusers approach
pip install accelerate
pip install transformers
```

**WS-07 (Dataset Tools)**:
```bash
pip install pillow
pip install lpips
pip install pytorch-fid
pip install clip-by-openai
pip install transformers  # For BLIP captioning
```

---

## Known Issues & Mitigations

### Issue 1: ARM Compatibility for xformers

**Problem**: xformers may not have pre-built ARM wheels
**Impact**: Blocks WS-05 optimization, degrades performance
**Mitigation**:
- Check for ARM builds: https://github.com/facebookresearch/xformers/releases
- Build from source if needed (add to WS-04)
- Fallback: Use PyTorch's native `scaled_dot_product_attention`
- Document workaround in completion summary

**Priority**: P0 (must resolve in WS-04)

### Issue 2: Performance Target Risk

**Problem**: GB10 hardware has no public benchmarks for SDXL
**Impact**: May miss 3s per image target in WS-05
**Mitigation**:
- Start benchmarking early (first day of WS-05)
- Adjust targets based on actual hardware capabilities
- Document actual performance for future projects
- Consider FP8 if FP16 insufficient (requires torch 2.5+)

**Priority**: P1 (adjust expectations if needed)

### Issue 3: LoRA Training Convergence

**Problem**: Training may not converge or produce poor quality
**Impact**: Delays WS-06 completion, blocks side-by-side comparison
**Mitigation**:
- Start with proven hyperparameters from literature
- Use small validation set (10 images) to detect issues early
- Implement automatic checkpoint saving every 500 steps
- Have backup dataset (pixel art from known sources)

**Priority**: P1 (iterate on training config)

### Issue 4: Unified Memory Optimization

**Problem**: Standard CUDA code may not leverage unified memory efficiently
**Impact**: Suboptimal performance, potential memory thrashing
**Mitigation**:
- Profile memory usage with DCGM (WS-05)
- Use zero-copy access patterns where possible
- Document unified memory best practices
- Consult NVIDIA Grace Hopper documentation

**Priority**: P1 (optimize in WS-05)

### Issue 5: Dataset Licensing

**Problem**: Training datasets may have unclear licensing
**Impact**: Legal risk for distributing trained models
**Mitigation**:
- Use only open-licensed datasets (CC0, CC-BY)
- Document dataset sources in WS-07
- Create internal pixel art dataset if needed
- Avoid copyrighted game sprites

**Priority**: P2 (document thoroughly)

---

## Success Criteria

### Orchestrator Success

✅ All 4 workstreams complete within 4 weeks (6-week buffer acceptable)
✅ M1 gate (inference) passed by end of week 4
✅ M3 gate (training) passed by end of week 6
✅ Interface Orchestrator unblocked for WS-10, WS-12
✅ Integration Orchestrator unblocked for WS-16
✅ No unresolved performance or quality issues

### Quality Standards

**Code**:
- All Python code follows PEP 8
- Type hints for all functions
- Unit tests for training and dataset code
- Integration tests for end-to-end workflows

**Performance**:
- WS-05: ≤ 3s per 1024×1024 image (P0)
- WS-05: ≥ 15 img/min batch mode (P0)
- WS-06: ≤ 4 hours for 50-image training (P1)
- WS-07: Auto-caption 100 images in <5 min (P1)

**Documentation**:
- Each workstream has detailed README
- Performance optimization guide for WS-05
- Training best practices for WS-06
- Dataset preparation guide for WS-07
- All ARM compatibility issues documented

---

## Timeline

```
Week 3 (Days 15-21):
  Mon-Fri: WS-04 (ComfyUI Setup)
         → ComfyUI installed and verified
         → ARM compatibility documented
         → Basic workflow tested
         → HANDOFF to Interface (WS-10 can start planning)

Week 4 (Days 22-28):
  Mon-Fri: WS-05 (SDXL Optimization)
         → Performance tuning and profiling
         → Workflow templates created
         → M1 GATE CHECK (end of week)
         → HANDOFF to Interface (WS-10 can proceed)
         → HANDOFF to Integration (WS-16 can proceed)

Week 5 (Days 29-35):
  Mon-Fri: WS-06 (LoRA Training) + WS-07 (Dataset Tools) PARALLEL
         → WS-06: Training pipeline implementation
         → WS-07: Captioning, augmentation, validation
         → Both workstreams progress independently

Week 6 (Days 36-42):
  Mon-Fri: WS-06 completion, WS-07 completion
         → Example LoRA trained and validated
         → Dataset tools tested on real data
         → M3 GATE CHECK (end of week)
         → HANDOFF to Interface (WS-12 can proceed)
```

**Buffer**: 1-2 weeks for performance tuning or ARM compatibility issues

---

## Parallel Execution Strategy

### Week 3-4: Sequential (Critical Path)

WS-04 and WS-05 MUST be sequential - no parallelization possible.

**Reason**: WS-05 requires functional ComfyUI from WS-04.

### Week 5-6: Parallel (WS-06 + WS-07)

Both workstreams depend on WS-05 but are independent of each other.

**Resource Allocation**:
- **Agent 1 (ai-engineer)**: Focus on WS-06 (LoRA training)
- **Agent 2 (ai-engineer)**: Focus on WS-07 (dataset tools)

**Coordination**:
- Both agents share WS-05 outputs (optimized workflows, models)
- WS-07 agent provides tools to WS-06 agent for training dataset prep
- Daily sync to ensure dataset format compatibility

**Expected Timeline Savings**: 3-4 days (vs sequential execution)

---

## Completion Checklist

Before marking Model Orchestrator complete:

- [ ] WS-04 completion summary created
- [ ] WS-05 completion summary created
- [ ] WS-06 completion summary created
- [ ] WS-07 completion summary created
- [ ] M1 gate check passed and documented
- [ ] M3 gate check passed and documented
- [ ] All files committed to git
- [ ] ComfyUI workflows tested end-to-end
- [ ] Example LoRA checkpoint produced and validated
- [ ] Handoff documentation sent to Interface Orchestrator
- [ ] Handoff documentation sent to Integration Orchestrator
- [ ] All issues closed or transferred
- [ ] Final status report posted to Meta Orchestrator
- [ ] Performance metrics recorded in `bench/results/`

---

## Start Command

```bash
# Wait for Foundation Orchestrator M0 gate to pass
./scripts/check_foundation_gate.sh || exit 1

# Initialize Model Orchestrator
./scripts/spawn_model_orchestrator.sh

# Or manually:
cd /home/beengud/raibid-labs/dgx-pixels
cat docs/orchestration/orchestrators/model.md
./scripts/spawn_agent.sh ai-engineer WS-04
```

**Status**: Ready to spawn after Foundation Orchestrator completes WS-01.
