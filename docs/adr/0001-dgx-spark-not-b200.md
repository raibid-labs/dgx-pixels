# ADR 0001: DGX-Spark GB10 Hardware Clarification

**Status:** Accepted
**Date:** 2025-11-10
**Deciders:** Claude Code (based on hardware verification)
**Context:** Response to GPT-5 RFD feedback

---

## Context

GPT-5's RFD feedback (see `docs/rfds/gpt5-dgx-pixels.md`) provided valuable architectural guidance but was based on an incorrect hardware assumption. The feedback assumed we were targeting a **DGX B200** system (8× B200 GPUs, dual x86 EPYC CPUs, 2TB DDR5, NVSwitch, etc.).

After verifying the actual hardware, we confirmed this system is a **DGX-Spark** with a **GB10 Grace Blackwell Superchip**, which is fundamentally different.

---

## Hardware Verification

```bash
$ nvidia-smi --query-gpu=name,memory.total,compute_cap --format=csv
NVIDIA GB10, [N/A], 12.1

$ lscpu | grep "Model name"
Cortex-X925
Cortex-A725

$ free -h | grep Mem
Mem: 119Gi (128GB total)

$ nvcc --version
Cuda compilation tools, release 13.0, V13.0.88
```

---

## Decision

**We design DGX-Pixels specifically for the DGX-Spark GB10 architecture**, which has:

1. **Single GPU**: No multi-GPU scaling, NCCL, or NVSwitch
2. **Unified Memory**: 128GB shared between CPU and GPU (coherent, zero-copy)
3. **ARM CPU**: Grace CPU (20 cores), not x86
4. **Edge Focus**: Designed for edge AI inference, not datacenter scale-out

---

## Consequences

### What Changes (vs GPT-5 Feedback)

**Not Applicable:**
- Multi-GPU scaling benchmarks (NCCL bandwidth, 2→4→8 GPU tests)
- NVSwitch topology mapping
- Multi-GPU training/inference strategies
- Slurm cluster management (single-node system)
- Heavy Kubernetes orchestration (simpler orchestration sufficient)

**Still Valuable (Adapted):**
- Reproducibility framework (`/repro/`) ✅
- Single-GPU performance benchmarking ✅
- Metrics framework (LPIPS, SSIM, CLIP, DCGM) ✅
- Quality evaluation pipeline ✅
- Security practices (non-root containers, SBOM) ✅
- Observability (Prometheus, Grafana) ✅

### New Opportunities (Unified Memory)

The GB10's unified memory architecture provides unique advantages:

1. **Zero-Copy Image Transfers**: CPU and GPU share memory coherently
   - Eliminates `cudaMemcpy` overhead for image loading
   - Preprocessing on CPU, inference on GPU, with zero transfers
   - Ideal for interactive TUI with real-time preview

2. **Simplified Memory Management**: Single 128GB pool
   - No juggling separate CPU/GPU allocations
   - Larger batch sizes without OOM errors
   - Easier debugging (single address space)

3. **Lower Latency**: <1μs CPU↔GPU access
   - Instant image preview in TUI
   - Real-time preprocessing visualization
   - Interactive parameter tuning

4. **Edge Deployment Path**: Same architecture as Jetson
   - Code developed on DGX-Spark runs on Jetson AGX Orin/Thor
   - Unified codebase across edge→server spectrum

### Architecture Implications

**Proposal 2B (Rust TUI + Python) is even more ideal:**
- Rust TUI leverages low-latency unified memory for instant image updates
- Python backend doesn't need complex GPU memory management
- Sixel preview benefits from zero-copy CPU access to GPU-rendered images
- Side-by-side model comparison works smoothly (load multiple models in 128GB)

**Training Strategy:**
- LoRA fine-tuning fits comfortably in 128GB unified memory
- Can train with larger batch sizes than separate-memory systems
- FP16 training: ~60GB for SDXL + LoRA, leaving 68GB for data

**Deployment Strategy:**
- Docker Compose sufficient (no Kubernetes complexity needed)
- Single-node observability (simpler Prometheus/Grafana setup)
- Direct DCGM integration (no multi-node coordination)

---

## Revised Performance Targets

| Metric | DGX-Spark GB10 Target | DGX B200 (8-GPU) Target |
|--------|----------------------|------------------------|
| Inference Latency | ≤ 3s per 1024×1024 image | ≤ 0.5s (8-way parallel) |
| Batch Throughput | 15-25 images/min | 100+ images/min |
| LoRA Training | 2-4 hours (50 images) | 30 min (distributed) |
| Memory Usage | ≤ 100GB unified | ≤ 180GB per GPU |
| Scaling | 1× (batch optimization) | 8× (data parallel) |

---

## Updated Roadmap Alignment

See `docs/ROADMAP.md` for revised milestones:
- M0: Hardware verification ✅
- M1: Single-GPU SDXL optimization with unified memory
- M2: Rust TUI leveraging zero-copy image preview
- M3: LoRA training with large unified memory pool
- M4: Bevy integration (unchanged)
- M5: Single-node observability + Docker Compose deployment

---

## References

- GPT-5 RFD: `docs/rfds/gpt5-dgx-pixels.md`
- Hardware verification: `docs/hardware.md`
- Revised roadmap: `docs/ROADMAP.md`
- Metrics framework: `docs/metrics.md`

---

## Notes for GPT-5 (or Future Reviewers)

Thank you for the detailed feedback! The multi-GPU guidance is excellent for DGX B200 systems. However, our DGX-Spark architecture offers different (and in some ways superior) advantages for this use case:

1. **Simpler is better**: Single-GPU focus eliminates distributed training complexity
2. **Unified memory wins**: Zero-copy transfers give us latency advantages for interactive use
3. **Edge path**: DGX-Spark → Jetson deployment path aligns with game studio edge infrastructure
4. **Rapid iteration**: Single-node simplicity accelerates prototyping (our core goal)

We've retained all applicable feedback (reproducibility, metrics, quality evaluation, security) and adapted it for our single-GPU, unified-memory reality.

---

**Next ADRs:**
- 0002: Unified Memory Optimization Strategies
- 0003: Rust TUI Architecture for Zero-Copy Preview
- 0004: Single-GPU Training vs Inference Trade-offs
