# [gpt5] RFD — DGX-Pixels Research & Roadmap Refinement

### Summary
This Request for Discussion consolidates GPT-5’s technical review of the `raibid-labs/dgx-pixels` repository.  
The goal is to strengthen the project’s research direction, reproducibility, and production readiness for DGX-class pixel-art generation systems.

DGX-Pixels already defines a solid niche—AI-assisted pixel art for games—but needs clearer hardware grounding, measurable milestones, and baseline data to mature from concept to deployable system.

---

## 1. Strengths

- 🎯 **Clear focus** – Pixel-art asset creation and enhancement for game pipelines (sprites, tiles, UI).  
- 🧱 **Layered architecture** – “Quick-start → Advanced → Recommended → Enterprise” encourages fast prototyping while keeping an enterprise runway.  
- 🦀 **Stack choice** – Rust TUI controller + Python/ComfyUI backend over ZeroMQ is practical and performant.  
- 🧩 **Game-engine integration** – Bevy + MCP link makes this more than another image generator.  
- 🔥 **Modern tech awareness** – SDXL, LoRA, diffusion, and FP4 all acknowledged.

---

## 2. Areas for Refinement

### a. Hardware & Platform Clarity
- Replace ambiguous “DGX-Spark / 128 GB unified memory / 1000 TOPS” claims with concrete specs:  
  `DGX B200 (8×B200 192 GB) • NVSwitch Gen 4 • Dual 400 GbE • 8 GB/s storage`.
- Add `/docs/hardware.md` containing:
  - `nvidia-smi topo -m` output  
  - GPU model, VRAM, driver versions, CPU/NIC/storage info  
  - MIG vs full-GPU policy (interactive vs batch)

### b. Reproducibility Backbone
Create `/repro/run.sh` that:
- Prints CUDA/cuDNN/NCCL, driver, GPU list, git SHA, DCGM ver, seeds  
- Runs 10-image smoke test via A1111 or ComfyUI  
- Logs img/s, p95 latency, VRAM peak  
- Saves artifacts + metrics under `/bench/baselines/`

### c. Benchmark & Metrics Suite
Establish `/bench/`:
- `nccl.sh` → bandwidth & latency across 2/4/8 GPUs  
- `dmon.sh` → DCGM + nvidia-smi telemetry under load  
- `throughput.py` → multi-LoRA vs single baseline  
Define metrics in `/docs/metrics.md`  
(throughput img/s, p95 latency, VRAM peak, power, scaling efficiency).

### d. Evaluation & Quality
Add `/eval/` with:
- **Objective:** LPIPS, SSIM, PSNR, CLIP distance  
- **Subjective:** human rubric for 16×16/32×32 sprite clarity & palette consistency  
- Store results + reference images per run.

### e. Observability & MLOps
Create `/deploy/k8s/` including:
- NVIDIA GPU Operator + DCGM Exporter + Prometheus + Grafana + KEDA  
- Dashboards for utilization, VRAM, thermals, power, NCCL health  
- Alert examples (e.g., throttling, VRAM > X%, ECC errors)

### f. Roadmap Formalization

| Milestone | Focus | Success Metrics |
|------------|--------|----------------|
| **M0** | Repro & Smoke Test | 10 images end-to-end ✔️ |
| **M1** | Multi-GPU Inference | ≥ 1.7× scaling vs 2-GPU baseline |
| **M2** | Data Pipeline (CUDA/DALI) | Throughput ≥ target GB/s |
| **M3** | LoRA Fine-tune | Loss ≤ L within Y hours |
| **M4** | Productization & SLOs | p95 latency ≤ threshold; autoscale verified |

### g. Security & Supply Chain
- Non-root, read-only containers pinned to NGC tags  
- SBOM + vulnerability scan target  
- Hashes for all model artifacts

### h. Collaboration Hygiene
- Architecture Decision Records under `/docs/adr/`  
  - 0001-k8s-vs-slurm.md  
  - 0002-mig-policy.md  
  - 0003-model-family-and-checkpoints.md  
- `CHANGELOG.md` and weekly `STATUS.md`

### i. Productization & Demos
Ship two early demos:
1. **Super-resolution + Denoise** hybrid  
2. **Multi-GPU Diffusion** with prompt/seed/version tracking  
Deploy via `make demo/<name>` or `docker compose`.

---

## 3. Proposed Deliverables Overview

| Category | Deliverable | Folder |
|-----------|--------------|---------|
| Reproducibility | Env capture + smoke script | `/repro/` |
| Benchmarks | NCCL/DCGM/throughput suite + baselines | `/bench/` |
| Evaluation | Quality metrics + rubric | `/eval/` |
| MLOps | GPU Operator stack + dashboards | `/deploy/k8s/` |
| Documentation | Hardware, metrics, roadmap, ADRs | `/docs/` |

---

## 4. Open Questions

1. Which DGX/GB platform (A100, B200, GB200 NVL72) is primary?  
2. Preferred scheduler — Kubernetes or Slurm?  
3. How tightly should LoRA fine-tuning integrate with core pipeline?  
4. Is Bevy/MCP integration runtime or post-generation?  
5. Include GPU-hour & power telemetry by default?

---

## 5. Next Steps

1. Approve this RFD and link follow-up PRs.  
2. Implement minimal `/repro` and `/bench` backbone first.  
3. Update README to reflect hardware + metrics clarity.  
4. Open ADRs and formal `/ROADMAP.md`.  
5. Schedule design review after M1 completion.

---

**Author:** GPT-5 (analysis & proposal)  
**Date:** 2025-11-10  
**Tags:** #RFD #DGX #PixelAI #MLOps #Performance #gpt5
