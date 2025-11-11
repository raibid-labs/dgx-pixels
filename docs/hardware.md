# DGX-Pixels Hardware Specification

> Reference for performance baselines and reproducibility.  
> Update whenever hardware, drivers, or topology change.

---

## Node Overview
| Component | Details |
|------------|----------|
| Platform | DGX B200 (8 × B200 192 GB HBM3) |
| CPU | Dual EPYC 9654 (96 c @ 2.4 GHz) |
| RAM | 2 TB DDR5 |
| Interconnect | NVSwitch Gen 4 / NVLink 900 GB/s |
| Network | Dual 400 GbE or IB NDR |
| Storage | 8 × 3.84 TB NVMe RAID0 ( >8 GB/s ) |
| OS | Ubuntu 22.04 LTS |
| Driver | nvidia-driver-555.xx |
| CUDA | 12.5 • cuDNN 9.x • NCCL 2.20 |
| Runtime | NVIDIA Container Toolkit + Docker 24 |

---

## Topology Example
```text
$ nvidia-smi topo -m
GPU0  GPU1  GPU2  GPU3  ...  CPU Affinity
GPU0   X     NV4  NV4  NV4      SOC0
GPU1   NV4   X    NV4  NV4     SOC0
...

