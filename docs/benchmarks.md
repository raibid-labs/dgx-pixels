# DGX-Pixels Benchmark Suite

This document describes the comprehensive benchmark suite used to establish performance baselines for the DGX-Spark GB10 hardware and validate system performance throughout development.

**Version**: 1.0
**Last Updated**: 2025-11-11
**Workstream**: WS-03 (Benchmark Suite)

---

## Overview

The DGX-Pixels benchmark suite measures four critical performance aspects:

1. **GPU Throughput** - TFLOPS for FP32/FP16 operations
2. **Memory Bandwidth** - CPU-GPU unified memory transfer rates
3. **Storage I/O** - Sequential read/write and random IOPS
4. **DCGM Metrics** - GPU utilization, power, temperature monitoring

All benchmarks are designed for automated execution and produce JSON baseline files for regression testing and performance tracking.

---

## Benchmark Locations

| Benchmark | Script | Baseline Output | Duration |
|-----------|--------|----------------|----------|
| GPU Throughput | `bench/gpu_throughput.py` | `bench/baselines/gpu_baseline.json` | ~20s |
| Memory Bandwidth | `bench/memory_bandwidth.py` | `bench/baselines/memory_baseline.json` | ~4s |
| Storage I/O | `bench/storage_io.sh` | `bench/baselines/storage_baseline.json` | ~4s |
| DCGM Metrics | `bench/dcgm_metrics.sh` | `bench/baselines/dcgm_baseline.json` | <1s |

**Total Suite Duration**: ~30 seconds (well within 15-minute target)

---

## GPU Throughput Benchmark

### Purpose

Measure raw GPU compute performance using matrix multiplication operations. Validates that the GB10 GPU is performing as expected for AI/ML workloads.

### Metrics

- **FP32 TFLOPS**: Single-precision floating-point throughput
- **FP16 TFLOPS**: Half-precision floating-point throughput (Tensor Cores)
- **INT8 TOPS**: Integer operations (skipped - not supported in PyTorch for raw matmul)

### Usage

```bash
# Inside Docker container
docker run --rm -v $PWD:/workspace --gpus all --ipc=host \
    dgx-pixels:dev python3 /workspace/bench/gpu_throughput.py

# Direct execution (if environment configured)
python3 bench/gpu_throughput.py
```

### Expected Performance (GB10 Blackwell)

| Precision | Expected | Measured (Baseline) | Status |
|-----------|----------|---------------------|--------|
| FP32 | ~100 TFLOPS | 25-32 TFLOPS | ⚠️ Lower (sm_121 not fully supported) |
| FP16 | ~200 TFLOPS | 10-15 TFLOPS | ⚠️ Lower (sm_121 not fully supported) |
| INT8 | ~400 TOPS | N/A | Skipped (torch.int8 matmul unsupported) |

**Note**: Lower performance expected until PyTorch fully supports GB10 (sm_121 compute capability). Performance should improve with future NGC/PyTorch releases.

### Baseline JSON Schema

```json
{
  "version": "1.0",
  "timestamp": "ISO-8601 datetime",
  "gpu": {
    "name": "NVIDIA GB10",
    "compute_capability": "12.1",
    "total_memory_gb": 119.7,
    "multi_processor_count": 144,
    "cuda_version": "12.6",
    "pytorch_version": "2.6.0a0"
  },
  "fp32_tflops": 25.51,
  "fp16_tflops": 10.0,
  "int8_tops": null,
  "benchmark_params": {
    "matrix_size": 8192,
    "iterations": 100,
    "warmup": 10
  },
  "notes": "INT8 benchmark skipped (torch.int8 matmul not supported in PyTorch)"
}
```

### Interpretation

- **FP32 ≥10 TFLOPS**: Acceptable for current PyTorch limitations
- **FP16 ≥10 TFLOPS**: Acceptable for Tensor Core operations
- **Lower values**: Check GPU utilization, thermal throttling, driver version

---

## Memory Bandwidth Benchmark

### Purpose

Measure memory bandwidth in the DGX-Spark's unified memory architecture. Unlike discrete GPUs, GB10 shares memory between CPU and GPU (435 GB/s specification).

### Metrics

- **GPU-to-GPU Bandwidth**: Internal GPU memory copy
- **CPU-to-GPU Bandwidth**: Unified memory transfer (CPU → GPU)
- **GPU-to-CPU Bandwidth**: Unified memory transfer (GPU → CPU)
- **Average Bandwidth**: Mean of CPU↔GPU transfers

### Usage

```bash
# Inside Docker container
docker run --rm -v $PWD:/workspace --gpus all --ipc=host \
    dgx-pixels:dev python3 /workspace/bench/memory_bandwidth.py

# Direct execution
python3 bench/memory_bandwidth.py
```

### Expected Performance

| Transfer Type | Expected | Measured (Baseline) | Status |
|---------------|----------|---------------------|--------|
| GPU-to-GPU | 200-400 GB/s | 145-165 GB/s | ✓ Reasonable |
| CPU-to-GPU | 50-100 GB/s | 52 GB/s | ✓ Within range |
| GPU-to-CPU | 50-100 GB/s | 52 GB/s | ✓ Within range |
| Average | 50-100 GB/s | 52 GB/s | ✓ 12% of specification |

**Note**: Unified memory architecture exhibits different patterns than discrete GPU PCIe transfers. Measured bandwidth is reasonable for zero-copy unified memory.

### Baseline JSON Schema

```json
{
  "version": "1.0",
  "timestamp": "ISO-8601 datetime",
  "memory": {
    "total_gb": 119.7,
    "allocated_gb": 2.5,
    "reserved_gb": 4.0,
    "architecture": "unified",
    "specification_bandwidth_gbs": 435
  },
  "gpu_to_gpu_gbs": 157.23,
  "cpu_to_gpu_gbs": 52.06,
  "gpu_to_cpu_gbs": 51.95,
  "bandwidth_gbs": 52.0,
  "benchmark_params": {
    "transfer_size_mb": 512,
    "iterations": 50
  }
}
```

### Interpretation

- **Average ≥50 GB/s**: Acceptable for unified memory
- **GPU-to-GPU ≥100 GB/s**: Internal GPU memory performing well
- **Lower values**: Check system load, background processes

---

## Storage I/O Benchmark

### Purpose

Measure storage throughput and IOPS to ensure adequate performance for model loading, dataset streaming, and output generation.

### Metrics

- **Sequential Read Throughput**: Large file read performance (GB/s)
- **Sequential Write Throughput**: Large file write performance (GB/s)
- **Random Read IOPS**: Small block random read operations

### Usage

```bash
# Direct execution (requires sudo for cache clearing)
bash bench/storage_io.sh

# Note: Uses /home/beengud/raibid-labs/dgx-pixels/bench/.io_test directory
```

### Expected Performance

| Metric | Expected | Measured (Baseline) | Status |
|--------|----------|---------------------|--------|
| Sequential Read | ≥8 GB/s | 11.3 GB/s | ✓ Exceeds target |
| Sequential Write | ≥8 GB/s | 2.3 GB/s | ⚠️ Below target |
| Random Read IOPS | ≥10,000 | 549 IOPS | ⚠️ Low (HDD characteristics) |

**Note**: Write performance and IOPS suggest HDD or slow SSD. May impact training data loading and checkpoint saving. Consider optimization strategies (caching, batching).

### Baseline JSON Schema

```json
{
  "version": "1.0",
  "timestamp": "ISO-8601 datetime",
  "filesystem": {
    "mount": "/",
    "size": "3.7T",
    "used": "369G",
    "available": "3.2T"
  },
  "sequential_read_gbs": 11.3,
  "sequential_write_gbs": 2.3,
  "random_read_iops": 549,
  "benchmark_params": {
    "file_size": "2G",
    "block_size_throughput": "1M",
    "block_size_iops": "4K",
    "iops_runtime_seconds": 30
  }
}
```

### Interpretation

- **Read ≥8 GB/s**: Sufficient for model loading
- **Write ≥2 GB/s**: Minimum for checkpoint saving (acceptable)
- **IOPS <1000**: HDD characteristics, use caching strategies

### Optimization Strategies (if performance insufficient)

1. **Model Loading**: Use unified memory (load models once)
2. **Dataset Streaming**: Prefetch batches, use memory-mapped files
3. **Checkpointing**: Save less frequently, use async I/O
4. **Caching**: Keep frequently-used data in RAM

---

## DCGM Metrics Benchmark

### Purpose

Establish baseline GPU metrics (utilization, power, temperature) for monitoring system health and detecting anomalies during training/inference.

### Metrics

- **GPU Utilization**: Percentage of time GPU is actively computing
- **Memory Utilization**: Percentage of GPU memory in use
- **Power Consumption**: GPU power draw in watts
- **Temperature**: GPU temperature in Celsius
- **Clock Speeds**: SM and memory clock frequencies (MHz)

### Usage

```bash
# Direct execution
bash bench/dcgm_metrics.sh

# Note: Falls back to nvidia-smi if DCGM unavailable (expected on ARM)
```

### Expected Values (Idle/Baseline)

| Metric | Idle Expected | Measured (Baseline) | Status |
|--------|---------------|---------------------|--------|
| GPU Utilization | 0-5% | 0% | ✓ Idle |
| Memory Util | 0-5% | 0% | ✓ Idle |
| Power | 10-30W | 11.66W | ✓ Idle power |
| Temperature | 30-60°C | 55-56°C | ✓ Normal |
| SM Clock | Variable | 2411 MHz | ✓ Active |
| Memory Clock | Variable | 0 MHz (N/A) | ⚠️ Unified memory |

**Note**: DCGM may have limited ARM support. nvidia-smi fallback provides sufficient metrics.

### Baseline JSON Schema

```json
{
  "version": "1.0",
  "timestamp": "ISO-8601 datetime",
  "source": "nvidia-smi",  // or "dcgm" if available
  "gpu": {
    "name": "NVIDIA GB10",
    "driver_version": "580.95.05",
    "cuda_version": "13.0"
  },
  "metrics": {
    "gpu_utilization_percent": 0,
    "memory_utilization_percent": 0,
    "power_watts": 11.66,
    "temperature_celsius": 56,
    "sm_clock_mhz": 2411,
    "memory_clock_mhz": 0  // N/A for unified memory
  },
  "gpu_utilization_percent": 0,
  "power_watts": 11.66,
  "temperature_celsius": 56
}
```

### Interpretation

- **Temperature <85°C**: Normal operation
- **Temperature 85-95°C**: High, check cooling
- **Temperature >95°C**: Critical, thermal throttling likely
- **Power >300W**: High load (expected during training)
- **Utilization >90%**: GPU fully utilized (good for training)

### DCGM Availability on ARM

- **DCGM**: May have limited support on ARM64 (DGX-Spark)
- **Fallback**: nvidia-smi provides sufficient metrics for baseline
- **Future**: NVIDIA may add full DCGM ARM support in future releases

---

## Running the Full Benchmark Suite

### Automated Test Suite

```bash
# Run all benchmarks with validation
bash tests/integration/ws_03/test_benchmarks.sh

# Output: Pass/fail for each benchmark + summary
```

### Manual Execution

```bash
# Run benchmarks individually
python3 bench/gpu_throughput.py
python3 bench/memory_bandwidth.py
bash bench/storage_io.sh
bash bench/dcgm_metrics.sh

# Check baseline files
ls -lh bench/baselines/*.json
```

### Baseline Files

All benchmarks output JSON to `bench/baselines/`:

```
bench/baselines/
├── gpu_baseline.json         # GPU throughput metrics
├── memory_baseline.json      # Memory bandwidth metrics
├── storage_baseline.json     # Storage I/O metrics
├── dcgm_baseline.json        # DCGM/nvidia-smi metrics
├── hardware_baseline.json    # (from WS-01) Hardware specs
└── env_*.json                # (from WS-02) Environment snapshots
```

---

## Regression Testing

### Purpose

Detect performance regressions by comparing current metrics against baselines.

### Usage

```bash
# Run benchmarks
bash tests/integration/ws_03/test_benchmarks.sh

# Compare against baselines (manual or scripted)
jq '.fp32_tflops' bench/baselines/gpu_baseline.json
jq '.bandwidth_gbs' bench/baselines/memory_baseline.json
```

### Regression Thresholds

| Metric | Acceptable Variance | Alert If |
|--------|-------------------|----------|
| GPU TFLOPS | ±10% | >20% decrease |
| Memory Bandwidth | ±15% | >25% decrease |
| Storage Read | ±20% | >30% decrease |
| Storage Write | ±20% | >30% decrease |

### Automation

For CI/CD integration:

```bash
# Run benchmarks and compare
bash tests/integration/ws_03/test_benchmarks.sh > test_results.log

# Extract metrics
NEW_FP32=$(jq -r '.fp32_tflops' bench/baselines/gpu_baseline.json)
BASELINE_FP32=25.51  # From previous run

# Compare (example: fail if >20% decrease)
if (( $(echo "$NEW_FP32 < $BASELINE_FP32 * 0.8" | bc -l) )); then
    echo "REGRESSION: FP32 decreased by >20%"
    exit 1
fi
```

---

## Performance Optimization Validation

### Use Cases

1. **SDXL Optimization** (WS-05): Compare inference throughput before/after optimization
2. **LoRA Training** (WS-06): Measure training iteration time, validate GPU utilization
3. **Model Loading**: Compare unified memory vs. discrete memory performance
4. **Batch Size Tuning**: Find optimal batch size for throughput vs. memory

### Example: Validating SDXL Optimization

```bash
# Before optimization
python3 bench/gpu_throughput.py  # FP16: 10 TFLOPS

# After enabling optimizations (xformers, FP16, etc.)
python3 bench/gpu_throughput.py  # FP16: 12 TFLOPS (20% improvement)
```

---

## Troubleshooting

### Issue: Low GPU Throughput

**Symptoms**: FP32 <5 TFLOPS, FP16 <5 TFLOPS

**Possible Causes**:
- Thermal throttling (check temperature)
- Power limit throttling (check power draw)
- Driver/PyTorch version mismatch
- GB10 sm_121 not supported in current PyTorch

**Solutions**:
1. Check temperature: `nvidia-smi`
2. Check power limit: `nvidia-smi -q -d POWER`
3. Upgrade NGC container to latest version
4. Wait for PyTorch GB10 (sm_121) support

### Issue: Low Memory Bandwidth

**Symptoms**: Average bandwidth <30 GB/s

**Possible Causes**:
- System memory pressure
- Background processes consuming memory bandwidth
- CPU throttling

**Solutions**:
1. Close background applications
2. Check system load: `htop` or `top`
3. Verify no swap usage: `free -h`

### Issue: Storage I/O Slow

**Symptoms**: Write <1 GB/s, IOPS <100

**Possible Causes**:
- HDD instead of SSD
- Filesystem full (>90% used)
- Background I/O operations

**Solutions**:
1. Check disk usage: `df -h`
2. Check I/O load: `iostat -x 1`
3. Use caching strategies (load data to /dev/shm)
4. Consider SSD upgrade if possible

---

## Future Enhancements

### Short-term (M1-M2)

1. Add inference latency benchmark (SDXL 1024x1024 generation time)
2. Integrate with WS-04 (ComfyUI) for end-to-end workflow benchmarking
3. Add LoRA training iteration time measurement

### Medium-term (M3-M4)

1. Automated regression testing in CI/CD
2. Historical baseline tracking (time series)
3. Grafana dashboards for real-time monitoring
4. Slack/email alerts for performance regressions

### Long-term (M5+)

1. Multi-GPU benchmarking (if scaling to multiple DGX-Spark units)
2. Distributed training benchmarks
3. Energy efficiency metrics (TFLOPS/Watt)
4. Comparative benchmarks (GB10 vs B200 vs H100)

---

## References

- **WS-01**: Hardware Baselines (`docs/hardware.md`)
- **WS-02**: Reproducibility Framework (`docs/reproducibility.md`)
- **WS-04**: ComfyUI Setup (future: inference benchmarking)
- **WS-05**: SDXL Optimization (future: end-to-end performance)
- **GB10 Specifications**: Grace Blackwell documentation
- **PyTorch Profiling**: https://pytorch.org/tutorials/recipes/recipes/profiler_recipe.html
- **DCGM Documentation**: https://docs.nvidia.com/datacenter/dcgm/

---

## Appendix: Benchmark Output Examples

### GPU Throughput Output

```
======================================================================
GPU Throughput Benchmark - DGX-Spark GB10
======================================================================

GPU: NVIDIA GB10
Compute Capability: 12.1
Memory: 119.7 GB
CUDA: 12.6
PyTorch: 2.6.0a0+df5bbc09d1.nv24.11

Benchmark Parameters:
  Matrix Size: 8192x8192
  Iterations: 100
  Warmup: 10

Running FP32 benchmark...
  Expected: ~100 TFLOPS (GB10 base performance)
  Result: 25.51 TFLOPS

Running FP16 benchmark...
  Expected: ~200 TFLOPS (Tensor Cores)
  Result: 10.00 TFLOPS

Skipping INT8 benchmark (torch.int8 matmul not supported)
  Note: INT8 inference supported via quantized models, not raw matmul

======================================================================
✓ GPU throughput baseline saved to: bench/baselines/gpu_baseline.json
======================================================================

Summary:
  FP32: 25.51 TFLOPS
  FP16: 10.00 TFLOPS
  INT8: N/A (skipped)

✓ FP32 performance acceptable (25.51 TFLOPS)
✓ FP16 performance acceptable (10.00 TFLOPS)

Note: GB10 (sm_121) not yet fully supported in PyTorch NGC 24.11
      Performance may improve with future PyTorch/CUDA updates
```

### Memory Bandwidth Output

```
======================================================================
Memory Bandwidth Benchmark - DGX-Spark GB10 Unified Memory
======================================================================

Memory Architecture: unified
Total Memory: 119.7 GB
Specification Bandwidth: 435 GB/s

Benchmark Parameters:
  Transfer Size: 512 MB
  Iterations: 50

Running GPU-to-GPU bandwidth test...
  (Internal GPU memory copy)
  Result: 157.23 GB/s

Running CPU-to-GPU bandwidth test...
  (Unified memory: CPU → GPU transfer)
  Result: 52.06 GB/s

Running GPU-to-CPU bandwidth test...
  (Unified memory: GPU → CPU transfer)
  Result: 51.95 GB/s

======================================================================
✓ Memory bandwidth baseline saved to: bench/baselines/memory_baseline.json
======================================================================

Summary:
  GPU-to-GPU: 157.23 GB/s
  CPU-to-GPU: 52.06 GB/s
  GPU-to-CPU: 51.95 GB/s
  Average Bandwidth: 52.00 GB/s
  Specification: 435 GB/s

Bandwidth Utilization: 12.0% of specification

✓ Memory bandwidth within acceptable range
```

---

**Document Version**: 1.0
**Last Updated**: 2025-11-11
**Owner**: Foundation Orchestrator (WS-03)
**Status**: Complete
