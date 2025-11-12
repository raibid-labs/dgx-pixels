# SDXL Optimization Benchmarks

## Directory Contents

This directory contains performance benchmark results for optimized SDXL inference on DGX-Spark GB10.

### Expected Files (After Benchmarking)

**Baseline Benchmarks**:
- `baseline_benchmark.json` - Unoptimized performance metrics
- `memory_baseline.json` - Memory usage without optimizations

**Optimized Benchmarks**:
- `optimized_benchmark.json` - Performance with all optimizations enabled
- `memory_optimized.json` - Memory usage with optimizations

**Comparisons**:
- `baseline_vs_optimized.json` - Side-by-side performance comparison
- `memory_comparison.json` - Memory usage comparison across configs

**Batch Processing**:
- `batch_performance.json` - Batch size scaling analysis (1, 4, 8, 16)
- `memory_batch_*.json` - Memory profiles for different batch sizes

## Running Benchmarks

### Prerequisites

1. Start ComfyUI server:
```bash
cd /home/beengud/raibid-labs/dgx-pixels/comfyui
python main.py --listen 0.0.0.0 --port 8188
```

2. Verify SDXL model is available:
```bash
ls -lh /home/beengud/raibid-labs/dgx-pixels/models/checkpoints/sd_xl_base_1.0.safetensors
```

### Run Benchmarks

```bash
cd /home/beengud/raibid-labs/dgx-pixels

# Run all benchmarks (baseline + optimized + comparison)
python3 python/optimization/benchmark_optimized.py --mode all

# Or run individually:
python3 python/optimization/benchmark_optimized.py --mode baseline
python3 python/optimization/benchmark_optimized.py --mode optimized
python3 python/optimization/benchmark_optimized.py --mode compare
```

### View Results

```bash
# View comparison
cat bench/optimization/baseline_vs_optimized.json

# View batch performance
cat bench/optimization/batch_performance.json

# View memory profiles
ls -lh bench/optimization/memory_*.json
```

## Benchmark Metrics

Each benchmark file contains:

- **inference_time_s**: Time to generate image(s) in seconds
- **throughput_imgs_per_min**: Images generated per minute
- **memory_allocated_gb**: GPU memory allocated
- **memory_reserved_gb**: GPU memory reserved
- **batch_size**: Number of images generated simultaneously
- **image_resolution**: (width, height) in pixels
- **config**: Optimization configuration used

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Single sprite (1024×1024) | <3s | ⏳ Pending |
| Batch-8 sprites | <15s total | ⏳ Pending |
| VRAM usage | <60GB | ⏳ Pending |
| Throughput | >20 imgs/min | ⏳ Pending |

## Optimization Configurations

### Baseline
- Precision: FP32
- Attention: Default PyTorch
- Other optimizations: Disabled

### Optimized
- Precision: FP16
- Attention: PyTorch SDPA (Scaled Dot Product Attention)
- Channels-last: Enabled
- cuDNN benchmark: Enabled
- VAE slicing: Enabled

## Directory Status

**Current Status**: ✅ Infrastructure ready, ⏳ Benchmark results pending

To generate results, run the benchmark commands above.

---

**Version**: 1.0
**Last Updated**: 2025-11-11
