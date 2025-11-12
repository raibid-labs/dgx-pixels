#!/usr/bin/env python3
"""
GPU Throughput Benchmark for DGX-Spark GB10

Measures TFLOPS performance for FP32 and FP16 operations using PyTorch.
This benchmark validates GPU compute capabilities against GB10 specifications.

Expected Performance (GB10 Blackwell):
- FP32: ~100 TFLOPS
- FP16: ~200 TFLOPS (Tensor Cores)

Note: INT8 matmul not supported in current PyTorch for torch.int8 type,
      so we skip INT8 benchmarking.

Usage:
    python3 bench/gpu_throughput.py

Output:
    bench/baselines/gpu_baseline.json
"""

import json
import time
from datetime import datetime, timezone
from pathlib import Path
import sys

try:
    import torch
except ImportError:
    print("ERROR: PyTorch not installed", file=sys.stderr)
    sys.exit(1)


def benchmark_matmul(dtype, size=8192, iterations=100, warmup=10):
    """
    Benchmark matrix multiplication throughput.

    Args:
        dtype: torch.float32 or torch.float16
        size: Matrix size (NxN)
        iterations: Number of iterations for measurement
        warmup: Number of warmup iterations

    Returns:
        tflops: Achieved TFLOPS for this dtype
    """
    device = torch.device("cuda:0")

    # Create random matrices
    A = torch.randn(size, size, dtype=dtype, device=device)
    B = torch.randn(size, size, dtype=dtype, device=device)

    # Warmup
    for _ in range(warmup):
        C = torch.matmul(A, B)
    torch.cuda.synchronize()

    # Benchmark
    start_event = torch.cuda.Event(enable_timing=True)
    end_event = torch.cuda.Event(enable_timing=True)

    start_event.record()
    for _ in range(iterations):
        C = torch.matmul(A, B)
    end_event.record()

    torch.cuda.synchronize()
    elapsed_ms = start_event.elapsed_time(end_event)

    # Calculate TFLOPS
    # Matrix multiplication: 2 * N^3 operations (multiply-add)
    ops_per_iter = 2 * size * size * size
    total_ops = ops_per_iter * iterations
    elapsed_s = elapsed_ms / 1000.0
    tflops = (total_ops / elapsed_s) / 1e12

    return tflops


def get_gpu_info():
    """Get GPU information."""
    if not torch.cuda.is_available():
        raise RuntimeError("CUDA not available")

    device_id = 0
    props = torch.cuda.get_device_properties(device_id)

    return {
        "name": torch.cuda.get_device_name(device_id),
        "compute_capability": f"{props.major}.{props.minor}",
        "total_memory_gb": round(props.total_memory / (1024**3), 2),
        "multi_processor_count": props.multi_processor_count,
        "cuda_version": torch.version.cuda,
        "pytorch_version": torch.__version__,
    }


def main():
    print("=" * 70)
    print("GPU Throughput Benchmark - DGX-Spark GB10")
    print("=" * 70)
    print()

    # Check GPU availability
    if not torch.cuda.is_available():
        print("ERROR: CUDA not available", file=sys.stderr)
        sys.exit(1)

    # Get GPU info
    gpu_info = get_gpu_info()
    print(f"GPU: {gpu_info['name']}")
    print(f"Compute Capability: {gpu_info['compute_capability']}")
    print(f"Memory: {gpu_info['total_memory_gb']:.1f} GB")
    print(f"CUDA: {gpu_info['cuda_version']}")
    print(f"PyTorch: {gpu_info['pytorch_version']}")
    print()

    # Benchmark parameters
    matrix_size = 8192
    iterations = 100
    warmup = 10

    print(f"Benchmark Parameters:")
    print(f"  Matrix Size: {matrix_size}x{matrix_size}")
    print(f"  Iterations: {iterations}")
    print(f"  Warmup: {warmup}")
    print()

    results = {
        "version": "1.0",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "gpu": gpu_info,
    }

    # FP32 Benchmark
    print("Running FP32 benchmark...")
    print("  Expected: ~100 TFLOPS (GB10 base performance)")
    fp32_tflops = benchmark_matmul(torch.float32, size=matrix_size, iterations=iterations, warmup=warmup)
    results["fp32_tflops"] = round(fp32_tflops, 2)
    print(f"  Result: {fp32_tflops:.2f} TFLOPS")
    print()

    # FP16 Benchmark
    print("Running FP16 benchmark...")
    print("  Expected: ~200 TFLOPS (Tensor Cores)")
    fp16_tflops = benchmark_matmul(torch.float16, size=matrix_size, iterations=iterations, warmup=warmup)
    results["fp16_tflops"] = round(fp16_tflops, 2)
    print(f"  Result: {fp16_tflops:.2f} TFLOPS")
    print()

    # INT8 Benchmark (skipped - not supported for torch.int8 matmul)
    print("Skipping INT8 benchmark (torch.int8 matmul not supported)")
    print("  Note: INT8 inference supported via quantized models, not raw matmul")
    results["int8_tops"] = None
    print()

    # Add benchmark metadata
    results["benchmark_params"] = {
        "matrix_size": matrix_size,
        "iterations": iterations,
        "warmup": warmup,
    }
    results["notes"] = "INT8 benchmark skipped (torch.int8 matmul not supported in PyTorch)"

    # Save to JSON
    output_dir = Path(__file__).parent / "baselines"
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / "gpu_baseline.json"

    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)

    print("=" * 70)
    print(f"✓ GPU throughput baseline saved to: {output_file}")
    print("=" * 70)
    print()

    # Summary
    print("Summary:")
    print(f"  FP32: {fp32_tflops:.2f} TFLOPS")
    print(f"  FP16: {fp16_tflops:.2f} TFLOPS")
    print(f"  INT8: N/A (skipped)")
    print()

    # Validation
    if fp32_tflops < 10:
        print("⚠ WARNING: FP32 performance lower than expected (< 10 TFLOPS)")
    else:
        print(f"✓ FP32 performance acceptable ({fp32_tflops:.2f} TFLOPS)")

    if fp16_tflops < 10:
        print("⚠ WARNING: FP16 performance lower than expected (< 10 TFLOPS)")
    else:
        print(f"✓ FP16 performance acceptable ({fp16_tflops:.2f} TFLOPS)")

    print()
    print("Note: GB10 (sm_121) not yet fully supported in PyTorch NGC 24.11")
    print("      Performance may improve with future PyTorch/CUDA updates")

    return 0


if __name__ == "__main__":
    sys.exit(main())
