# DGX-Pixels Roadmap (Aligned with RFD [gpt5])

> **Status:** Draft (to be ratified)  
> **Owner:** raibid-labs  
> **Last Updated:** 2025-11-10  

| Milestone | Goal | Deliverables | Metrics |
|------------|------|---------------|----------|
| **M0 — Reproducibility Backbone** | Single-GPU E2E run | `/repro/run.sh`, pinned NGC Dockerfile | 10 images ✔ + env log |
| **M1 — Multi-GPU Inference** | NCCL + scaling | `/bench/nccl.sh`, `throughput.py` | ≥ 1.7× scale (2→4 GPUs) |
| **M2 — Data Pipeline** | Optimize I/O & preprocessing | CUDA/DALI pipeline, `/data/README.md` | ≥ 8 GB/s throughput |
| **M3 — LoRA Fine-Tuning** | Controlled training | Scripts + checkpoints + metrics | Loss ≤ target; ≤ Y hours |
| **M4 — Product Service + SLOs** | Deployable GPU service | `/deploy/k8s/` stack + dashboards | p95 latency ≤ target |

### Future Milestones
- **M5:** Edge/Portable Compose Deployment  
- **M6:** Community LoRA Registry + Bevy Showcase  

### Revision History
| Date | Change | Author |
|------|---------|--------|
| 2025-11-10 | Initial draft (GPT-5 alignment) | GPT-5 |

