# DGX-Pixels Roadmap

> **Status:** Active (revised for DGX-Spark architecture)
> **Owner:** raibid-labs
> **Last Updated:** 2025-11-10

---

## Hardware Context

**Target Platform:** NVIDIA DGX-Spark (GB10 Grace Blackwell Superchip)
- Single GPU with 128GB unified memory
- ARM Grace CPU (20 cores)
- Optimized for edge AI inference and interactive workloads

**Note:** Original GPT-5 feedback assumed DGX B200 (8×GPU datacenter system). This roadmap is adjusted for our single-GPU, unified-memory architecture which is better suited for rapid prototyping and interactive pixel art generation.

---

## Milestones

| Milestone | Goal | Deliverables | Success Metrics |
|------------|------|---------------|-----------------|
| **M0 — Foundation & Reproducibility** | Establish baseline performance | Hardware docs, repro scripts, smoke tests | 10 test images generated ✔ + env capture |
| **M1 — Core Inference Pipeline** | Optimized single-GPU SDXL inference | ComfyUI workflows, batch processing | ≤ 3s per 1024×1024 image @ FP16 |
| **M2 — Interactive TUI** | Rust TUI with ZeroMQ backend | ratatui app + Python worker + Sixel preview | <100ms UI responsiveness, image preview working |
| **M3 — LoRA Training Pipeline** | Custom model fine-tuning | Training scripts + dataset tools + validation | Loss convergence ≤ 2-4 hours; visual quality ≥ baseline |
| **M4 — Bevy Integration** | MCP-based game engine integration | FastMCP server + bevy_brp_mcp client + examples | Asset auto-deploy working; hot-reload ≤ 1s |
| **M5 — Production Readiness** | Observability, metrics, deployment | DCGM metrics + Docker compose + CI/CD | 95% uptime; p95 latency ≤ target |

---

## M0 — Foundation & Reproducibility (Week 1-2)

**Goal:** Document hardware, establish baseline performance, create reproducible environment.

### Deliverables
- [x] `/docs/hardware.md` with verified DGX-Spark GB10 specs
- [ ] `/repro/run.sh` - Environment capture + smoke test script
- [ ] `/repro/Dockerfile` - Pinned NGC base image (PyTorch + CUDA 13.0)
- [ ] Baseline measurements for SDXL inference

### Success Criteria
- Generate 10 test images end-to-end
- Document: GPU model, driver, CUDA version, commit SHA
- Record baseline: img/s, latency, VRAM usage

### Commands
```bash
./repro/run.sh                    # Run smoke test
cat bench/baselines/latest.json   # View metrics
```

---

## M1 — Core Inference Pipeline (Week 3-4)

**Goal:** Optimize SDXL inference on single GB10 GPU, create reusable workflows.

### Deliverables
- [ ] ComfyUI installation + ARM compatibility verification
- [ ] Workflow templates: `single_sprite.json`, `batch_generation.json`, `tileset.json`
- [ ] FP16 optimization + memory-efficient attention (xformers)
- [ ] `/bench/throughput.py` - Single-GPU performance measurement

### Success Criteria
- ≤ 3 seconds per 1024×1024 image (FP16, batch=1)
- ≥ 15 images/min in batch mode (batch=8)
- VRAM usage ≤ 100 GB (leaving headroom in unified memory)

### Optimizations for GB10 Unified Memory
- Zero-copy image loading (CPU and GPU share memory)
- Minimize intermediate tensors
- Batch size tuned for 128GB unified pool

---

## M2 — Interactive TUI (Week 5-6)

**Goal:** Build responsive Rust TUI with live image preview and side-by-side model comparison.

### Deliverables
- [ ] Rust TUI application (`rust/src/`) with ratatui framework
- [ ] ZeroMQ IPC layer (REQ-REP for jobs, PUB-SUB for status)
- [ ] Python backend worker (`python/workers/generation_worker.py`)
- [ ] Sixel image preview in terminal
- [ ] Side-by-side comparison: pre-trained vs custom LoRA

### Success Criteria
- 60 FPS UI rendering
- <1ms ZeroMQ IPC latency
- Image preview working in supported terminals
- Can compare outputs from multiple models simultaneously

### Unique Advantage
The unified memory architecture makes CPU↔GPU image transfers essentially free, enabling:
- Instant preview updates
- Real-time preprocessing visualization
- Low-latency interactive workflows

---

## M3 — LoRA Training Pipeline (Week 7-9)

**Goal:** Train custom LoRA models for pixel art style consistency.

### Deliverables
- [ ] Training script (Kohya_ss or Diffusers)
- [ ] Dataset preparation tools (auto-captioning, augmentation)
- [ ] Training config templates (resolution, steps, learning rate)
- [ ] Validation pipeline (LPIPS, SSIM comparison)
- [ ] Model registry (`models/loras/`)

### Success Criteria
- Train 50-image dataset in ≤ 4 hours @ 3000 steps
- Loss convergence verified
- Visual quality ≥ pre-trained models (validated via side-by-side comparison)
- Generated sprites maintain style consistency

### Training Optimizations
- FP16 mixed precision training
- Gradient checkpointing for memory efficiency
- Unified memory allows larger batch sizes

---

## M4 — Bevy Integration (Week 10-11)

**Goal:** Seamless integration with Bevy game engine via Model Context Protocol.

### Deliverables
- [ ] FastMCP server implementation (`src/mcp_server/`)
- [ ] Bevy plugin example with `bevy_brp_mcp`
- [ ] Asset deployment automation (generate → review → deploy)
- [ ] Hot-reload support for rapid iteration
- [ ] Example Bevy project with AI-generated sprites

### Success Criteria
- MCP server responds to generate/deploy commands
- Assets appear in Bevy project within 1 second of generation
- Hot-reload triggers automatic sprite updates in running game
- Example game showcases workflow

### Integration Patterns
- Manual: Generate → Review → Copy to `assets/`
- Automated: MCP command → Auto-deploy → Hot-reload

---

## M5 — Production Readiness (Week 12+)

**Goal:** Metrics, observability, deployment packaging, CI/CD.

### Deliverables
- [ ] DCGM metrics export (GPU utilization, VRAM, temperature, power)
- [ ] Prometheus + Grafana dashboards
- [ ] Docker Compose deployment (`docker-compose.yml`)
- [ ] CI pipeline (test, benchmark, quality checks)
- [ ] Security: non-root containers, pinned dependencies, SBOM

### Success Criteria
- 95% uptime over 7-day test period
- p95 latency meets target
- Automated benchmarks run on every commit
- Security scan passes (no high/critical CVEs)

### Observability Metrics
- Performance: img/s, latency, VRAM peak
- Quality: LPIPS, SSIM, human ratings
- System: GPU utilization, temperature, power draw
- Cost: GPU-hours, kWh per image

---

## Future Milestones (Post-M5)

### M6 — Edge Deployment
- Package for Jetson AGX Orin / Jetson Thor
- Optimize for lower VRAM (INT8 quantization)
- Create portable inference runtime

### M7 — Community Features
- Public LoRA model registry
- Showcase gallery of Bevy games using DGX-Pixels
- Contribution guide for new workflows

### M8 — Advanced Features
- Multi-frame animation generation
- Style transfer between sprites
- Procedural tileset generation with constraints

---

## Non-Applicable GPT-5 Feedback (DGX B200 Specific)

The following suggestions from GPT-5's RFD are **not applicable** to DGX-Spark:

- Multi-GPU scaling tests (we have 1 GPU)
- NCCL bandwidth benchmarks (no multi-GPU communication)
- NVSwitch topology mapping (no NVSwitch in DGX-Spark)
- 8-GPU parallelism strategies (single-GPU system)
- Slurm vs Kubernetes (single-node system, simpler orchestration)

**However, we retain these valuable suggestions:**
- Reproducibility framework ✅
- Metrics and benchmarking (adapted for single GPU) ✅
- Quality evaluation (LPIPS, SSIM, CLIP) ✅
- Security and supply chain practices ✅
- Observability (DCGM, Prometheus, Grafana) ✅

---

## Architecture Decision Records (ADRs)

Creating `/docs/adr/` for key decisions:

- **0001-dgx-spark-not-b200.md** — Hardware clarification and implications
- **0002-unified-memory-advantages.md** — Leveraging GB10's unified memory
- **0003-rust-tui-architecture.md** — Why Rust + Python hybrid
- **0004-single-gpu-focus.md** — Design decisions for 1-GPU optimization

---

## Revision History

| Date | Change | Author |
|------|---------|--------|
| 2025-11-10 | Initial draft (GPT-5 alignment) | GPT-5 |
| 2025-11-10 | Revised for DGX-Spark GB10 hardware | Claude Code |

---

**Next Review:** After M1 completion
