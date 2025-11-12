# SDXL Optimization Troubleshooting Guide

## Overview

This guide addresses common issues encountered when optimizing SDXL inference on the DGX-Spark GB10 hardware.

---

## Hardware & Environment Issues

### Issue: GB10 Not Detected or Wrong Compute Capability

**Symptoms**:
- PyTorch reports wrong GPU model
- Compute capability != 12.1
- CUDA operations failing

**Diagnosis**:
```bash
# Check GPU info
nvidia-smi --query-gpu=name,compute_cap,driver_version --format=csv

# Expected output:
# name, compute_cap, driver_version
# NVIDIA GB10, 12.1, 580.95.05

# In Python:
python3 -c "import torch; print(torch.cuda.get_device_properties(0))"
```

**Solutions**:
1. Verify driver version >= 580.95.05
2. Ensure PyTorch NGC container is 25.01 or newer
3. Check CUDA version >= 12.8

---

### Issue: NGC Container Warning "GB10 may not yet be supported"

**Symptoms**:
```
WARNING: Detected NVIDIA GB10 GPU, which may not yet be supported in this version of the container
```

**Impact**: None observed in testing, all features working

**Explanation**:
- GB10 is a new architecture (sm_121)
- NGC containers may not have extensive testing yet
- Core features (FP16, SDPA, cuDNN) all work

**Actions**:
1. ✅ Proceed with testing
2. Monitor for unexpected failures
3. Report issues to NVIDIA if found

**Workaround**: None needed, warning is informational

---

### Issue: nvidia-smi Shows `[N/A]` for GPU Memory

**Symptoms**:
```bash
nvidia-smi
# Shows:
# | 0  NVIDIA GB10   | [N/A]
```

**Explanation**:
- GB10 uses unified memory architecture
- CPU and GPU share 128.5 GB memory pool
- nvidia-smi can't distinguish GPU-only memory

**Solutions**:
1. Use PyTorch memory tracking:
```python
import torch
allocated = torch.cuda.memory_allocated() / 1e9
reserved = torch.cuda.memory_reserved() / 1e9
print(f"Allocated: {allocated:.1f} GB")
print(f"Reserved: {reserved:.1f} GB")
```

2. Use system memory tools:
```bash
free -h  # Total system memory
```

3. Use memory profiler:
```python
from memory_profiler import MemoryProfiler
profiler = MemoryProfiler()
stats = profiler._get_gpu_memory()
```

---

### Issue: SHMEM Allocation Limit (64MB)

**Symptoms**:
```
NOTE: The SHMEM allocation limit is set to the default of 64MB.
This may be insufficient for PyTorch.
```

**Impact**: May cause issues with multi-process data loading

**Solutions**:
1. Use `--ipc=host` in Docker:
```bash
docker run --gpus all --ipc=host \
  -v /home/beengud/raibid-labs/dgx-pixels:/workspace \
  nvcr.io/nvidia/pytorch:25.01-py3 \
  python3 script.py
```

2. Or increase SHMEM size:
```bash
docker run --gpus all --shm-size=8g \
  -v /home/beengud/raibid-labs/dgx-pixels:/workspace \
  nvcr.io/nvidia/pytorch:25.01-py3 \
  python3 script.py
```

---

## Optimization Issues

### Issue: Out of Memory (OOM) Errors

**Symptoms**:
```
RuntimeError: CUDA out of memory. Tried to allocate X GB
```

**Diagnosis**:
```python
from sdxl_optimizations import SDXLOptimizer
optimizer = SDXLOptimizer()
stats = optimizer.get_memory_stats()
print(f"Allocated: {stats['allocated_gb']:.1f} GB")
print(f"Free: {stats['free_gb']:.1f} GB")
```

**Solutions (in order of preference)**:

1. **Enable VAE slicing** (minimal speed impact):
```python
config = OptimizationConfig(vae_slicing=True)
```

2. **Reduce batch size**:
```python
config = OptimizationConfig(batch_size=4)  # or lower
```

3. **Enable VAE tiling** (for very large images):
```python
config = OptimizationConfig(vae_tiling=True)
```

4. **Enable model CPU offloading** (slower, last resort):
```python
config = OptimizationConfig(cpu_offload=True)
```

5. **Clear GPU cache between generations**:
```python
optimizer.clear_memory()
```

**Memory Budget Guidelines**:
- Single sprite (1024×1024): ~12 GB
- Batch-4: ~30 GB
- Batch-8: ~55 GB
- Batch-16: ~100 GB (may exceed 128GB with other processes)

---

### Issue: Slow Inference (>5s per sprite)

**Symptoms**:
- Generation takes >5s for 1024×1024 image
- Much slower than expected

**Diagnosis**:
```python
from sdxl_optimizations import SDXLOptimizer
optimizer = SDXLOptimizer()
optimizer._check_hardware_support()
```

Expected output:
```
[INFO] GPU: NVIDIA GB10
[INFO] Compute Capability: 12.1
[INFO] PyTorch SDPA available - using scaled_dot_product_attention
[INFO] ARM64 architecture detected (Grace CPU)
```

**Solutions**:

1. **Verify FP16 is enabled**:
```python
config = OptimizationConfig(precision=PrecisionMode.FP16)
optimizer.configure(config)
```

2. **Check attention backend**:
```python
# Should be SDPA or XFORMERS, not PYTORCH
config = OptimizationConfig(attention_backend=AttentionBackend.SDPA)
```

3. **Enable cuDNN benchmark**:
```python
config = OptimizationConfig(enable_cudnn_benchmark=True)
```

4. **Verify model is on GPU**:
```python
print(next(model.parameters()).device)  # Should be 'cuda:0'
```

5. **Check for CPU fallback**:
```python
import torch
print(torch.cuda.is_available())  # Should be True
```

---

### Issue: FP16 Precision Not Working

**Symptoms**:
- Model still in FP32 despite FP16 config
- No speedup from FP16

**Diagnosis**:
```python
model = optimizer.optimize_model(model)
print(next(model.parameters()).dtype)  # Should be torch.float16
```

**Solutions**:

1. **Verify FP16 is set in config**:
```python
config = OptimizationConfig(precision=PrecisionMode.FP16)
optimizer.configure(config)
```

2. **Check model optimization was applied**:
```python
model = optimizer.optimize_model(model)  # Must call this!
```

3. **Use autocast context for inference**:
```python
with optimizer.get_autocast_context():
    output = model(input)
```

4. **Verify Tensor Cores are active**:
```bash
# During inference, check GPU utilization
nvidia-smi dmon -s u
# Look for high "sm" (streaming multiprocessor) utilization
```

---

### Issue: Memory-Efficient Attention Not Available

**Symptoms**:
```
[WARNING] No optimized attention backend available, using PyTorch default
```

**Diagnosis**:
```python
import torch.nn.functional as F
print(hasattr(F, "scaled_dot_product_attention"))  # Should be True
```

**Solutions**:

1. **Update PyTorch NGC container**:
```bash
docker pull nvcr.io/nvidia/pytorch:25.01-py3
```

2. **If SDPA not available, try xformers**:
```bash
pip install xformers
```

3. **Verify PyTorch version**:
```python
import torch
print(torch.__version__)  # Should be >= 2.6.0
```

---

### Issue: torch.compile() Fails

**Symptoms**:
```
[WARNING] torch.compile() failed: <error message>
```

**Explanation**:
- `torch.compile()` is experimental for GB10 sm_121
- May have limited support for new architecture

**Solutions**:

1. **Disable torch.compile()** (default, recommended):
```python
config = OptimizationConfig(enable_torch_compile=False)
```

2. **If you want to try it**:
```python
config = OptimizationConfig(enable_torch_compile=True)
# But be prepared for it to fail or be slower
```

**Impact**: Minimal, other optimizations provide sufficient speedup

---

### Issue: Poor Image Quality with Optimizations

**Symptoms**:
- Blurry or low-detail sprites
- Artifacts in generated images
- Quality worse than expected

**Diagnosis**:

1. **Check if using FP8 or excessive quantization**:
```python
print(config.precision)  # Should be FP16 or BF16, NOT FP8
```

2. **Compare with baseline**:
```bash
# Generate same prompt with baseline and optimized
# Visual comparison
```

**Solutions**:

1. **Use FP16, not FP8**:
```python
config = OptimizationConfig(precision=PrecisionMode.FP16)
```

2. **Increase inference steps**:
```python
config = OptimizationConfig(num_inference_steps=25)  # or 30
```

3. **Increase CFG scale**:
```python
config = OptimizationConfig(guidance_scale=9.0)  # or 10.0
```

4. **Use quality-focused sampler**:
```json
{
  "sampler_name": "euler_ancestral",
  "scheduler": "karras"
}
```

5. **Verify VAE slicing doesn't cause issues**:
```python
# Try disabling VAE slicing if quality is poor
config = OptimizationConfig(vae_slicing=False)
```

---

## Testing & Debugging Issues

### Issue: Unit Tests Failing

**Symptoms**:
```bash
pytest tests/ws_05/ -v
# Some tests fail
```

**Solutions**:

1. **Run in Docker with PyTorch**:
```bash
docker run --rm --gpus all \
  -v /home/beengud/raibid-labs/dgx-pixels:/workspace \
  -w /workspace \
  nvcr.io/nvidia/pytorch:25.01-py3 \
  bash -c "pip install -q pytest && pytest tests/ws_05/ -v"
```

2. **Check for missing dependencies**:
```bash
pip install torch psutil pytest
```

3. **Run individual test files**:
```bash
pytest tests/ws_05/test_optimizations.py -v
pytest tests/ws_05/test_comfyui_integration.py -v
```

4. **Skip CUDA tests if no GPU**:
```bash
pytest tests/ws_05/ -v -m "not cuda"
```

---

### Issue: ComfyUI Integration Tests Skipped

**Symptoms**:
```
tests/ws_05/test_comfyui_integration.py::test_* SKIPPED
Reason: ComfyUI not available
```

**Explanation**: Tests require ComfyUI server running

**Solutions**:

1. **Start ComfyUI**:
```bash
cd /home/beengud/raibid-labs/dgx-pixels/comfyui
python main.py --listen 0.0.0.0 --port 8188
```

2. **Verify ComfyUI is accessible**:
```bash
curl http://localhost:8188/system_stats
```

3. **Re-run tests**:
```bash
pytest tests/ws_05/test_comfyui_integration.py -v
```

---

### Issue: Benchmarks Not Generating Results

**Symptoms**:
```bash
python3 python/optimization/benchmark_optimized.py --mode baseline
# No output files in bench/optimization/
```

**Solutions**:

1. **Check ComfyUI is running**:
```bash
curl http://localhost:8188/queue
```

2. **Verify workflow files exist**:
```bash
ls -l workflows/*.json
```

3. **Check output directory permissions**:
```bash
mkdir -p bench/optimization
chmod 755 bench/optimization
```

4. **Run with verbose output**:
```bash
python3 python/optimization/benchmark_optimized.py --mode baseline --verbose
```

---

## Workflow Issues

### Issue: Workflow Fails to Load in ComfyUI

**Symptoms**:
- Workflow upload fails
- "Invalid workflow" error

**Diagnosis**:
```python
import json
with open("workflows/sprite_optimized.json") as f:
    workflow = json.load(f)  # Check for JSON errors
```

**Solutions**:

1. **Validate JSON syntax**:
```bash
python3 -m json.tool workflows/sprite_optimized.json
```

2. **Check required nodes exist**:
```bash
curl http://localhost:8188/object_info | grep "CheckpointLoaderSimple"
```

3. **Verify checkpoint name**:
```json
{
  "1": {
    "inputs": {
      "ckpt_name": "sd_xl_base_1.0.safetensors"  // Must exist in models/checkpoints/
    }
  }
}
```

4. **Check model file exists**:
```bash
ls -lh models/checkpoints/sd_xl_base_1.0.safetensors
```

---

### Issue: Generation Stuck in Queue

**Symptoms**:
- Workflow submitted but never completes
- Queue shows job as "pending" indefinitely

**Diagnosis**:
```bash
curl http://localhost:8188/queue
# Check queue_pending and queue_running
```

**Solutions**:

1. **Check ComfyUI logs**:
```bash
# In ComfyUI terminal, look for errors
```

2. **Clear queue**:
```bash
curl -X POST http://localhost:8188/queue/clear
```

3. **Restart ComfyUI**:
```bash
# Kill and restart ComfyUI
cd comfyui && python main.py
```

4. **Check for OOM errors in logs**

---

## Performance Debugging

### Enable Detailed Profiling

```python
import torch
from torch.profiler import profile, ProfilerActivity

with profile(
    activities=[ProfilerActivity.CPU, ProfilerActivity.CUDA],
    record_shapes=True,
    profile_memory=True,
) as prof:
    # Run inference
    output = model(input)

print(prof.key_averages().table(sort_by="cuda_time_total", row_limit=10))
```

### Monitor GPU Utilization During Inference

```bash
# Terminal 1: Run inference
python3 benchmark.py

# Terminal 2: Monitor GPU
watch -n 0.5 nvidia-smi
```

### Track Memory Over Time

```python
from memory_profiler import MemoryProfiler

profiler = MemoryProfiler()
with profiler.profile("debug_run", batch_size=1):
    profiler.take_snapshot("start")
    model.load()
    profiler.take_snapshot("model_loaded")
    output = model.generate()
    profiler.take_snapshot("generation_done")
    output.save()
    profiler.take_snapshot("save_done")

profiler.print_profile_summary(profiler.profiles[0])
```

---

## Getting Help

### Check Logs

1. **Optimization module logs**:
```python
from sdxl_optimizations import SDXLOptimizer
optimizer = SDXLOptimizer()
optimizer._check_hardware_support()  # Prints diagnostic info
```

2. **Memory profiler logs**:
```python
profiler.print_profile_summary(profile)
```

3. **ComfyUI logs**: Check terminal where ComfyUI is running

### Collect Debug Information

```bash
# Hardware info
nvidia-smi --query-gpu=name,compute_cap,memory.total,driver_version --format=csv

# PyTorch info
python3 -c "import torch; print(f'PyTorch: {torch.__version__}, CUDA: {torch.version.cuda}')"

# Memory stats
free -h

# Disk space
df -h /home/beengud/raibid-labs/dgx-pixels
```

### Report Issues

When reporting issues, include:
1. Hardware info (nvidia-smi output)
2. PyTorch version and NGC container version
3. Optimization configuration used
4. Error messages (full stack trace)
5. Memory stats at time of failure

---

## Known Issues & Workarounds

### GB10 sm_121 Limited Support

**Issue**: Some features may not be fully optimized for sm_121

**Workarounds**:
- Stick with FP16 (well-supported)
- Use SDPA instead of xformers
- Disable torch.compile()
- Monitor performance vs expected

### Unified Memory Reporting

**Issue**: nvidia-smi shows `[N/A]` for GPU memory

**Workaround**: Use PyTorch memory tracking or system tools

### SHMEM Limit

**Issue**: Default 64MB SHMEM insufficient for multi-process

**Workaround**: Use `--ipc=host` or `--shm-size=8g` in Docker

---

## References

- Optimization Guide: `/docs/optimization-guide.md`
- Performance Results: `/docs/performance-results.md`
- ComfyUI Setup: `/docs/comfyui.md`
- Hardware Baseline: `/bench/baselines/hardware_baseline.json`

---

**Version**: 1.0
**Last Updated**: 2025-11-11
**Status**: Complete
