#!/usr/bin/env python3
"""
Memory Bandwidth Benchmark for DGX-Spark GB10 Unified Memory

Measures memory bandwidth for CPU-GPU transfers in the unified memory architecture.
Tests different access patterns to characterize memory system performance.

Expected Performance (GB10 with unified memory):
- Unified Memory Bandwidth: 435 GB/s (specification)
- Actual measured may vary based on access patterns

Usage:
    python3 bench/memory_bandwidth.py

Output:
    bench/baselines/memory_baseline.json
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


def benchmark_memory_bandwidth(direction, size_mb=1024, iterations=100):
    """
    Benchmark memory bandwidth for different transfer directions.

    Args:
        direction: 'gpu_to_gpu', 'cpu_to_gpu', 'gpu_to_cpu'
        size_mb: Size of data to transfer in MB
        iterations: Number of iterations

    Returns:
        bandwidth_gbs: Achieved bandwidth in GB/s
    """
    device = torch.device("cuda:0")
    size_bytes = size_mb * 1024 * 1024
    size_elements = size_bytes // 4  # float32 = 4 bytes

    if direction == 'gpu_to_gpu':
        # GPU to GPU copy (within GPU memory)
        src = torch.randn(size_elements, dtype=torch.float32, device=device)
        dst = torch.empty(size_elements, dtype=torch.float32, device=device)

        # Warmup
        for _ in range(10):
            dst.copy_(src)
        torch.cuda.synchronize()

        # Benchmark
        start_event = torch.cuda.Event(enable_timing=True)
        end_event = torch.cuda.Event(enable_timing=True)

        start_event.record()
        for _ in range(iterations):
            dst.copy_(src)
        end_event.record()

        torch.cuda.synchronize()
        elapsed_ms = start_event.elapsed_time(end_event)

    elif direction == 'cpu_to_gpu':
        # CPU to GPU transfer (unified memory - pinned memory)
        src = torch.randn(size_elements, dtype=torch.float32, device='cpu').pin_memory()
        dst = torch.empty(size_elements, dtype=torch.float32, device=device)

        # Warmup
        for _ in range(10):
            dst.copy_(src, non_blocking=False)
        torch.cuda.synchronize()

        # Benchmark
        start_event = torch.cuda.Event(enable_timing=True)
        end_event = torch.cuda.Event(enable_timing=True)

        start_event.record()
        for _ in range(iterations):
            dst.copy_(src, non_blocking=False)
        end_event.record()

        torch.cuda.synchronize()
        elapsed_ms = start_event.elapsed_time(end_event)

    elif direction == 'gpu_to_cpu':
        # GPU to CPU transfer
        src = torch.randn(size_elements, dtype=torch.float32, device=device)
        dst = torch.empty(size_elements, dtype=torch.float32, device='cpu').pin_memory()

        # Warmup
        for _ in range(10):
            dst.copy_(src, non_blocking=False)
        torch.cuda.synchronize()

        # Benchmark
        start_event = torch.cuda.Event(enable_timing=True)
        end_event = torch.cuda.Event(enable_timing=True)

        start_event.record()
        for _ in range(iterations):
            dst.copy_(src, non_blocking=False)
        end_event.record()

        torch.cuda.synchronize()
        elapsed_ms = start_event.elapsed_time(end_event)

    else:
        raise ValueError(f"Unknown direction: {direction}")

    # Calculate bandwidth
    total_bytes = size_bytes * iterations
    elapsed_s = elapsed_ms / 1000.0
    bandwidth_gbs = (total_bytes / elapsed_s) / (1024**3)

    return bandwidth_gbs


def get_memory_info():
    """Get memory information."""
    if not torch.cuda.is_available():
        raise RuntimeError("CUDA not available")

    device_id = 0
    props = torch.cuda.get_device_properties(device_id)

    # Get current memory usage
    allocated = torch.cuda.memory_allocated(device_id) / (1024**3)
    reserved = torch.cuda.memory_reserved(device_id) / (1024**3)
    total = props.total_memory / (1024**3)

    return {
        "total_gb": round(total, 2),
        "allocated_gb": round(allocated, 2),
        "reserved_gb": round(reserved, 2),
        "architecture": "unified",  # GB10 has unified CPU-GPU memory
        "specification_bandwidth_gbs": 435,  # GB10 specification
    }


def main():
    print("=" * 70)
    print("Memory Bandwidth Benchmark - DGX-Spark GB10 Unified Memory")
    print("=" * 70)
    print()

    # Check GPU availability
    if not torch.cuda.is_available():
        print("ERROR: CUDA not available", file=sys.stderr)
        sys.exit(1)

    # Get memory info
    mem_info = get_memory_info()
    print(f"Memory Architecture: {mem_info['architecture']}")
    print(f"Total Memory: {mem_info['total_gb']} GB")
    print(f"Specification Bandwidth: {mem_info['specification_bandwidth_gbs']} GB/s")
    print()

    # Benchmark parameters
    size_mb = 512  # 512 MB transfers
    iterations = 50

    print(f"Benchmark Parameters:")
    print(f"  Transfer Size: {size_mb} MB")
    print(f"  Iterations: {iterations}")
    print()

    results = {
        "version": "1.0",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "memory": mem_info,
    }

    # GPU to GPU Bandwidth
    print("Running GPU-to-GPU bandwidth test...")
    print("  (Internal GPU memory copy)")
    gpu_to_gpu_gbs = benchmark_memory_bandwidth('gpu_to_gpu', size_mb=size_mb, iterations=iterations)
    results["gpu_to_gpu_gbs"] = round(gpu_to_gpu_gbs, 2)
    print(f"  Result: {gpu_to_gpu_gbs:.2f} GB/s")
    print()

    # CPU to GPU Bandwidth
    print("Running CPU-to-GPU bandwidth test...")
    print("  (Unified memory: CPU → GPU transfer)")
    cpu_to_gpu_gbs = benchmark_memory_bandwidth('cpu_to_gpu', size_mb=size_mb, iterations=iterations)
    results["cpu_to_gpu_gbs"] = round(cpu_to_gpu_gbs, 2)
    print(f"  Result: {cpu_to_gpu_gbs:.2f} GB/s")
    print()

    # GPU to CPU Bandwidth
    print("Running GPU-to-CPU bandwidth test...")
    print("  (Unified memory: GPU → CPU transfer)")
    gpu_to_cpu_gbs = benchmark_memory_bandwidth('gpu_to_cpu', size_mb=size_mb, iterations=iterations)
    results["gpu_to_cpu_gbs"] = round(gpu_to_cpu_gbs, 2)
    print(f"  Result: {gpu_to_cpu_gbs:.2f} GB/s")
    print()

    # Calculate average bandwidth
    avg_bandwidth = (cpu_to_gpu_gbs + gpu_to_cpu_gbs) / 2
    results["bandwidth_gbs"] = round(avg_bandwidth, 2)

    # Add benchmark metadata
    results["benchmark_params"] = {
        "transfer_size_mb": size_mb,
        "iterations": iterations,
    }

    # Save to JSON
    output_dir = Path(__file__).parent / "baselines"
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / "memory_baseline.json"

    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)

    print("=" * 70)
    print(f"✓ Memory bandwidth baseline saved to: {output_file}")
    print("=" * 70)
    print()

    # Summary
    print("Summary:")
    print(f"  GPU-to-GPU: {gpu_to_gpu_gbs:.2f} GB/s")
    print(f"  CPU-to-GPU: {cpu_to_gpu_gbs:.2f} GB/s")
    print(f"  GPU-to-CPU: {gpu_to_cpu_gbs:.2f} GB/s")
    print(f"  Average Bandwidth: {avg_bandwidth:.2f} GB/s")
    print(f"  Specification: {mem_info['specification_bandwidth_gbs']} GB/s")
    print()

    # Comparison with specification
    utilization_percent = (avg_bandwidth / mem_info['specification_bandwidth_gbs']) * 100
    print(f"Bandwidth Utilization: {utilization_percent:.1f}% of specification")
    print()

    # Validation
    if avg_bandwidth < 50:
        print("⚠ WARNING: Average bandwidth lower than expected (< 50 GB/s)")
        print("  Note: Unified memory architecture may show different patterns than discrete GPU")
    else:
        print("✓ Memory bandwidth within acceptable range")

    return 0


if __name__ == "__main__":
    sys.exit(main())
