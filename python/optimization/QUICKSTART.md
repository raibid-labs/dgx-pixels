# SDXL Optimization Quick Start

## 5-Minute Setup

### 1. Run Self-Tests

```bash
# Verify optimizations work on your system
docker run --rm --gpus all \
  -v /home/beengud/raibid-labs/dgx-pixels:/workspace \
  -w /workspace \
  nvcr.io/nvidia/pytorch:25.01-py3 \
  python3 python/optimization/sdxl_optimizations.py
```

Expected output:
```
✅ Self-test complete
[INFO] GPU: NVIDIA GB10
[INFO] Compute Capability: 12.1
[INFO] PyTorch SDPA available
```

---

### 2. Run Unit Tests

```bash
docker run --rm --gpus all \
  -v /home/beengud/raibid-labs/dgx-pixels:/workspace \
  -w /workspace \
  nvcr.io/nvidia/pytorch:25.01-py3 \
  bash -c "pip install -q pytest && pytest tests/ws_05/test_optimizations.py -v"
```

Expected: `22 passed` (100%)

---

### 3. Start ComfyUI

```bash
cd /home/beengud/raibid-labs/dgx-pixels/comfyui
python main.py --listen 0.0.0.0 --port 8188
```

Verify: `curl http://localhost:8188/system_stats`

---

### 4. Run Benchmarks

```bash
cd /home/beengud/raibid-labs/dgx-pixels

# Run all benchmarks (baseline + optimized + comparison)
python3 python/optimization/benchmark_optimized.py --mode all
```

Results saved to: `bench/optimization/`

---

## Usage Examples

### Example 1: Basic Optimization

```python
from sdxl_optimizations import SDXLOptimizer, get_optimal_config_for_gb10

# Initialize optimizer
optimizer = SDXLOptimizer()

# Get recommended config for GB10
config = get_optimal_config_for_gb10()
optimizer.configure(config)

# Optimize your model
model = optimizer.optimize_model(model)

# Use autocast for inference
with optimizer.get_autocast_context():
    output = model(input)
```

---

### Example 2: Memory Profiling

```python
from memory_profiler import MemoryProfiler

profiler = MemoryProfiler()

with profiler.profile("my_generation", batch_size=8):
    profiler.take_snapshot("start")

    # Your generation code
    images = generate_sprites(prompt, batch_size=8)

    profiler.take_snapshot("complete")

# Print results
profiler.print_profile_summary(profiler.profiles[0])
```

---

### Example 3: Benchmark Custom Workflow

```python
from benchmark_optimized import ComfyUIBenchmark

benchmark = ComfyUIBenchmark(comfyui_url="http://localhost:8188")

# Test different batch sizes
results = benchmark.benchmark_batch_sizes(
    batch_sizes=[1, 4, 8],
    prompt="pixel art character sprite",
    num_runs=3,
)

# Results include timing and throughput
for result in results:
    print(f"Batch {result['batch_size']}: {result['avg_time_s']:.2f}s")
    print(f"Throughput: {result['throughput_imgs_per_min']:.1f} imgs/min")
```

---

## Configuration Presets

### Balanced (Recommended)

```python
from sdxl_optimizations import OptimizationConfig, PrecisionMode, AttentionBackend

config = OptimizationConfig(
    precision=PrecisionMode.FP16,
    attention_backend=AttentionBackend.SDPA,
    enable_channels_last=True,
    enable_cudnn_benchmark=True,
    vae_slicing=True,
    batch_size=1,
    num_inference_steps=20,
    guidance_scale=8.0,
)
```

**Use for**: Single sprite generation, <3s target

---

### Batch Optimized

```python
config = OptimizationConfig(
    precision=PrecisionMode.FP16,
    attention_backend=AttentionBackend.SDPA,
    enable_channels_last=True,
    enable_cudnn_benchmark=True,
    vae_slicing=True,
    batch_size=8,  # Generate 8 at once
    num_inference_steps=20,
    guidance_scale=8.0,
)
```

**Use for**: Batch generation, >20 sprites/minute target

---

### Maximum Quality

```python
config = OptimizationConfig(
    precision=PrecisionMode.FP16,
    attention_backend=AttentionBackend.SDPA,
    enable_channels_last=True,
    enable_cudnn_benchmark=True,
    vae_slicing=True,
    batch_size=1,
    num_inference_steps=30,  # More steps
    guidance_scale=9.0,      # Higher CFG
)
```

**Use for**: Hero sprites, key art, quality > speed

---

## Quick Troubleshooting

### Out of Memory?

```python
# Enable VAE slicing
config.vae_slicing = True

# Reduce batch size
config.batch_size = 4

# Enable CPU offloading (last resort, slower)
config.cpu_offload = True
```

---

### Slow Inference?

```python
# Check optimizations are enabled
optimizer._check_hardware_support()

# Verify FP16 is active
print(next(model.parameters()).dtype)  # Should be torch.float16

# Enable cuDNN benchmark
config.enable_cudnn_benchmark = True
```

---

### Poor Quality?

```python
# Increase steps
config.num_inference_steps = 25

# Increase CFG
config.guidance_scale = 9.0

# Use quality sampler in workflow
{
  "sampler_name": "euler_ancestral",
  "scheduler": "karras"
}
```

---

## Performance Targets

| Metric | Target | Command to Verify |
|--------|--------|-------------------|
| Single sprite | <3s | `benchmark_optimized.py --mode baseline` |
| Batch-8 | <15s | `benchmark_optimized.py --mode optimized` |
| VRAM usage | <60GB | Memory profiler output |
| Throughput | >20 imgs/min | Benchmark results |

---

## File Locations

```
/home/beengud/raibid-labs/dgx-pixels/
├── python/optimization/
│   ├── sdxl_optimizations.py    # Main optimizer
│   ├── memory_profiler.py       # Memory tracking
│   ├── benchmark_optimized.py   # Benchmarking
│   └── requirements.txt         # Dependencies
├── workflows/
│   ├── sprite_optimized.json    # Single sprite
│   ├── batch_optimized.json     # Batch-8
│   └── pixel_art_workflow.json  # Quality preset
├── tests/ws_05/
│   ├── test_optimizations.py    # Unit tests (22)
│   └── test_comfyui_integration.py # Integration tests (13)
├── docs/
│   ├── optimization-guide.md    # Full guide
│   ├── performance-results.md   # Benchmark results
│   └── troubleshooting.md       # Problem solving
└── bench/optimization/
    └── (benchmark results here after running)
```

---

## Next Steps

1. **Run benchmarks** to verify performance targets
2. **Read full guide**: `docs/optimization-guide.md`
3. **Check results**: `docs/performance-results.md`
4. **If issues**: `docs/troubleshooting.md`

---

**Quick Links**:
- Full Guide: `/docs/optimization-guide.md`
- Troubleshooting: `/docs/troubleshooting.md`
- Test Suite: `/tests/ws_05/`
- Workflows: `/workflows/`

**Support**: See troubleshooting guide or check orchestrator logs
