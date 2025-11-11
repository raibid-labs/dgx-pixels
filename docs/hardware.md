# DGX-Pixels Hardware Specification

> Reference for performance baselines and reproducibility.
> Update whenever hardware, drivers, or topology change.

---

## Node Overview

| Component | Details |
|------------|----------|
| **Platform** | **NVIDIA DGX-Spark (GB10 Grace Blackwell Superchip)** |
| **GPU** | NVIDIA GB10 (Compute Capability 12.1) |
| **Memory** | 128 GB unified memory (shared CPU+GPU) |
| **CPU** | ARM-based Grace CPU (Cortex-X925 + Cortex-A725, 20 cores) |
| **RAM** | 119 GiB available (unified architecture) |
| **Interconnect** | Unified memory architecture (CPU-GPU coherent shared memory) |
| **Network** | 4× RoCE NICs (rocep1s0f0, etc.) |
| **Storage** | (to be verified) |
| **OS** | Linux 6.11.0-1016-nvidia |
| **Driver** | nvidia-driver-580.95.05 |
| **CUDA** | 13.0 (V13.0.88) |
| **Runtime** | NVIDIA Container Toolkit + Docker |

---

## Architecture: DGX-Spark vs DGX B200

**IMPORTANT**: This system is a **DGX-Spark**, NOT a DGX B200 as initially assumed.

### Key Differences

| Feature | DGX-Spark (GB10) | DGX B200 |
|---------|------------------|-----------|
| **GPU Count** | 1× GB10 superchip | 8× B200 GPUs |
| **Memory Model** | Unified 128GB (CPU+GPU shared) | Separate: 2TB DDR5 + 8×192GB HBM3 |
| **CPU Architecture** | ARM Grace (20 cores) | x86 Dual AMD EPYC (192 cores) |
| **Target Use Case** | Edge AI, single-node inference | Datacenter scale-out training |
| **Interconnect** | Coherent unified memory | NVSwitch Gen 4 (900 GB/s) |
| **Multi-GPU Scaling** | N/A (single GPU) | 8-way data/model parallelism |

### Implications for DGX-Pixels

The DGX-Spark architecture has unique advantages for pixel art generation:

1. **Unified Memory Benefits**:
   - No CPU→GPU memory copies for image data
   - Lower latency for preprocessing pipelines
   - Simplified memory management
   - Ideal for interactive TUI with image preview

2. **Single-GPU Focus**:
   - No multi-GPU scaling complexity
   - No NCCL/distributed training overhead
   - Simpler deployment model
   - Better for rapid iteration and prototyping

3. **ARM Architecture**:
   - Energy efficient for long-running services
   - Some x86-only libraries may need alternatives
   - Modern toolchains (Rust, Python) have excellent ARM support

4. **Edge Deployment**:
   - Can prototype on DGX-Spark and deploy to Jetson/Orin devices
   - Same Grace Blackwell architecture family
   - Unified codebase across edge→server spectrum

---

## Topology

```text
$ nvidia-smi topo -m
        GPU0    NIC0    NIC1    NIC2    NIC3    CPU Affinity    NUMA Affinity    GPU NUMA ID
GPU0     X      NODE    NODE    NODE    NODE    0-19            0                N/A
NIC0    NODE     X      PIX     NODE    NODE
NIC1    NODE    PIX     X      NODE    NODE
NIC2    NODE    NODE    NODE     X      PIX
NIC3    NODE    NODE    NODE    PIX     X

Legend:
  NODE = Connection traversing PCIe + interconnect between PCIe Host Bridges
  PIX  = Connection traversing at most a single PCIe bridge

Single GPU system - no NVLink/NVSwitch topology
```

---

## Performance Characteristics

Based on GB10 Grace Blackwell Superchip (1000 TOPS):

| Metric | Expected Performance |
|---------|---------------------|
| **Peak INT8 Performance** | ~1000 TOPS |
| **Peak FP16 Performance** | ~500 TFLOPS |
| **Memory Bandwidth** | Unified architecture (varies by access pattern) |
| **Inference Latency** | 2-4s per 1024×1024 SDXL image (FP16) |
| **Batch Throughput** | 15-25 images/min (batch size 4-8) |
| **LoRA Training** | 1-3 hours for 50 images @ 3000 steps |

---

## Verification Commands

```bash
# GPU info
nvidia-smi --query-gpu=name,memory.total,driver_version,compute_cap --format=csv

# CUDA version
nvcc --version

# Topology
nvidia-smi topo -m

# CPU architecture
lscpu | grep -E "Model name|Architecture"

# Memory
free -h

# Storage
df -h | grep -E "/$|/home"
```

---

## Baseline Measurements

**Baseline Captured**: 2025-11-10 (WS-01: Hardware Baselines)
**Baseline File**: `bench/baselines/hardware_baseline.json`
**Topology File**: `docs/topology.txt`
**Verification Script**: `repro/hardware_verification.nu`

### Verified Hardware Specifications

| Component | Specification | Verified Value |
|-----------|--------------|----------------|
| **GPU Model** | NVIDIA GB10 (Grace Blackwell) | ✅ Confirmed |
| **GPU Compute** | 12.1 | ✅ Confirmed |
| **Unified Memory** | 128GB (shared CPU+GPU) | ✅ 119GB available |
| **CPU Architecture** | ARM Grace (Cortex-X925) | ✅ Confirmed |
| **CPU Cores** | 20 cores | ✅ Confirmed |
| **CUDA Version** | 13.0.88 | ✅ Confirmed |
| **Driver Version** | 580.95.05+ | ✅ 580.95.05 |
| **Storage** | 500GB+ | ✅ 3755GB total |
| **Network** | RoCE NICs | ✅ 4× 100Gbps RoCE |

### Performance Baselines (To Be Measured in WS-03)

Run `just bench` to generate inference baseline metrics:

| Test | Throughput | Latency (p95) | VRAM Peak | Notes |
|------|------------|---------------|-----------|-------|
| SDXL 1.0 base | TBD img/s | TBD ms | TBD GB | FP16, batch=1 |
| SDXL + LoRA | TBD img/s | TBD ms | TBD GB | FP16, batch=1 |
| Batch inference (8) | TBD img/s | TBD ms | TBD GB | FP16, batch=8 |

---

**Last Updated:** 2025-11-10
**Verified By:** WS-01 Hardware Baselines (automated hardware scan)
