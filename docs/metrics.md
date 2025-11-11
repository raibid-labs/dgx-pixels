# DGX-Pixels Metrics & Measurement Framework

> **Purpose:** Define consistent quantitative and qualitative metrics for evaluating DGX-Pixels performance, quality, and cost across all milestones.  
> **Status:** Draft aligned with RFD [gpt5] (2025-11-10)  
> **Owner:** raibid-labs / DGX-Pixels maintainers  

---

## 1. Metric Categories

| Category | Description | Example Tools |
|-----------|--------------|----------------|
| **Performance** | GPU/CPU throughput, scaling efficiency, latency | DCGM, NCCL-tests, bench/throughput.py |
| **Quality** | Visual fidelity and style consistency | LPIPS, SSIM, PSNR, CLIP distance |
| **Observability** | Health, utilization, thermals, stability | DCGM Exporter, Prometheus, Grafana |
| **Efficiency** | GPU-hour cost, power draw, throughput per watt | DCGM energy, job accounting |
| **Reproducibility** | Deterministic outputs & reproducible baselines | /repro/run.sh, git SHA tracking |

---

## 2. Performance Metrics

| Metric | Definition | Target / Threshold | Measurement Script |
|---------|-------------|--------------------|--------------------|
| **Images / Second** | Mean generated images per second per GPU | ≥ 1.7× scaling (2→4 GPUs) | `/bench/throughput.py` |
| **Latency (p95)** | 95th percentile inference latency per image | ≤ 200 ms @ 1024×1024 | `/bench/throughput.py` |
| **VRAM Utilization** | Peak VRAM usage during inference | ≤ 90 % | `/bench/dmon.sh` |
| **NCCL Bandwidth** | Aggregate inter-GPU communication throughput | ≥ 200 GB/s (8 GPUs) | `/bench/nccl.sh` |
| **I/O Throughput** | Sustained data read/write rate | ≥ 8 GB/s | `/bench/io_test.sh` (future) |
| **Scaling Efficiency** | Speedup(n) / n vs single GPU | ≥ 0.85 (ideal linear = 1.0) | `/bench/throughput.py` |

---

## 3. Quality Metrics

| Metric | Description | Goal | Tool / Implementation |
|---------|--------------|------|------------------------|
| **LPIPS** | Learned perceptual similarity (lower = better) | ≤ 0.20 | `eval/lpips_eval.py` |
| **SSIM** | Structural similarity (higher = better) | ≥ 0.85 | `eval/ssim_eval.py` |
| **PSNR** | Signal-to-noise ratio (higher = better) | ≥ 25 dB | `eval/psnr_eval.py` |
| **CLIP Distance** | Style/semantic embedding similarity | ≤ 0.10 | `eval/clip_distance.py` |
| **Human Rating** | Mean opinion score for readability (1-5) | ≥ 4.0 | `/eval/human_rubric.md` |

**Sprite Evaluation Protocol**
1. Resize outputs to 16×16 / 32×32.  
2. Present side-by-side with ground-truth or reference palette.  
3. Collect 3–5 human ratings; compute mean & variance.  
4. Combine human + LPIPS weighted score for final grade.

---

## 4. Observability Metrics

| Metric | Description | Collection Method |
|---------|-------------|-------------------|
| **GPU Utilization (%)** | Average core activity over runtime | DCGM → Prometheus |
| **Memory BW / Clocks** | Average throughput / clock rates | DCGM + nvidia-smi dmon |
| **Power Draw (W)** | Mean + peak during run | DCGM energy plugin |
| **Temperature (°C)** | Max GPU die temperature | DCGM |
| **NCCL Errors** | Collective communication errors | NCCL log parse |
| **Job Success Rate** | Completed / attempted runs | CI metrics |

---

## 5. Efficiency & Cost Metrics

| Metric | Definition | Target | Source |
|---------|-------------|---------|---------|
| **GPU-Hours** | Total GPU count × runtime (h) | — | Job metadata |
| **Throughput / Watt** | Images / s per average power draw | ↑ | DCGM power logs |
| **Energy / Image** | kWh per image | ≤ 0.0002 kWh / img | Derived |
| **Cost / Image** | $ per image (based on power + GPU depreciation) | tracked | Cost model |
| **Storage Efficiency** | Artifact MB / image | ↓ | Artifact registry |

---

## 6. Reporting & Visualization

- All metrics exported to Prometheus (`dgx_pixels_*` namespace).  
- Grafana dashboards:
  - **Performance View:** img/s, latency, scaling curves.  
  - **Quality View:** LPIPS/SSIM over time.  
  - **System Health:** GPU utilization, thermals, power.  
  - **Cost Dashboard:** GPU-hours, kWh, estimated $.  
- Automated job annotations include: commit SHA, model, LoRA, dataset, runtime.

---

## 7. Evaluation Frequency

| Phase | Frequency | Responsible |
|--------|------------|--------------|
| Dev Iterations | Each PR / merge to `main` | CI pipeline |
| Benchmarks | Weekly | Infra engineer |
| Quality Review | Per milestone | Research lead |
| Power/Cost Audit | Monthly | Platform SRE |

---

## 8. Example Benchmark Command

```bash
make bench THROUGHPUT_GPU=4 DATASET=pixels-16x16

