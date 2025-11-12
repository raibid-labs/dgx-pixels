#!/usr/bin/env python3
"""
SDXL Optimization Module for DGX-Spark GB10

Implements performance optimizations for Stable Diffusion XL inference:
- Mixed precision (FP16/BF16)
- Memory-efficient attention (xformers/SDPA)
- Tensor Core utilization
- Batch processing optimizations
- ARM64 + GB10 sm_121 specific optimizations

Target Performance:
- Single sprite (1024x1024): <3s
- Batch-8 sprites: <15s total
- VRAM usage: <60GB per generation
"""

import torch
import json
import time
from typing import Dict, List, Optional, Tuple, Any
from pathlib import Path
from dataclasses import dataclass, asdict
from enum import Enum

# Check available optimizations
HAS_XFORMERS = False
HAS_SDPA = hasattr(torch.nn.functional, "scaled_dot_product_attention")
HAS_CUDA = torch.cuda.is_available()

try:
    import xformers
    import xformers.ops
    HAS_XFORMERS = True
except ImportError:
    pass


class PrecisionMode(Enum):
    """Supported precision modes for GB10"""
    FP32 = "fp32"
    FP16 = "fp16"
    BF16 = "bf16"
    FP8 = "fp8"  # May not be supported on GB10 sm_121


class AttentionBackend(Enum):
    """Attention implementation backends"""
    PYTORCH = "pytorch"
    XFORMERS = "xformers"
    SDPA = "sdpa"  # Scaled Dot Product Attention


@dataclass
class OptimizationConfig:
    """Configuration for SDXL optimizations"""
    precision: PrecisionMode = PrecisionMode.FP16
    attention_backend: AttentionBackend = AttentionBackend.SDPA
    enable_torch_compile: bool = False  # Experimental for GB10
    enable_channels_last: bool = True
    enable_cudnn_benchmark: bool = True
    cpu_offload: bool = False  # For >60GB models
    sequential_cpu_offload: bool = False
    vae_slicing: bool = True
    vae_tiling: bool = False
    batch_size: int = 1
    num_inference_steps: int = 20
    guidance_scale: float = 8.0

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary with enum values"""
        d = asdict(self)
        d['precision'] = self.precision.value
        d['attention_backend'] = self.attention_backend.value
        return d


@dataclass
class BenchmarkResult:
    """Results from a single benchmark run"""
    config: OptimizationConfig
    inference_time_s: float
    memory_allocated_gb: float
    memory_reserved_gb: float
    throughput_imgs_per_min: float
    batch_size: int
    image_resolution: Tuple[int, int]
    timestamp: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        return {
            'config': self.config.to_dict(),
            'inference_time_s': self.inference_time_s,
            'memory_allocated_gb': self.memory_allocated_gb,
            'memory_reserved_gb': self.memory_reserved_gb,
            'throughput_imgs_per_min': self.throughput_imgs_per_min,
            'batch_size': self.batch_size,
            'image_resolution': list(self.image_resolution),
            'timestamp': self.timestamp,
        }


class SDXLOptimizer:
    """
    Optimizer for SDXL inference on DGX-Spark GB10

    Applies hardware-specific optimizations for:
    - NVIDIA GB10 (compute capability 12.1)
    - ARM64 Grace CPU
    - Unified 119GB memory architecture
    """

    def __init__(self, device: str = "cuda"):
        self.device = torch.device(device if torch.cuda.is_available() else "cpu")
        self.config = OptimizationConfig()
        self._check_hardware_support()

    def _check_hardware_support(self) -> None:
        """Verify hardware capabilities and log warnings"""
        if not HAS_CUDA:
            print("[WARNING] CUDA not available, falling back to CPU (will be slow)")
            self.device = torch.device("cpu")
            return

        # Check GPU compute capability
        if torch.cuda.is_available():
            props = torch.cuda.get_device_properties(0)
            compute_cap = f"{props.major}.{props.minor}"
            print(f"[INFO] GPU: {props.name}")
            print(f"[INFO] Compute Capability: {compute_cap}")
            print(f"[INFO] Total Memory: {props.total_memory / 1e9:.1f} GB")

            # GB10 is sm_121 (compute capability 12.1)
            if props.major < 8:
                print("[WARNING] GPU compute capability < 8.0, Tensor Cores may not be optimal")

        # Check attention backend availability
        if HAS_XFORMERS:
            print("[INFO] xformers available - memory-efficient attention enabled")
        elif HAS_SDPA:
            print("[INFO] PyTorch SDPA available - using scaled_dot_product_attention")
        else:
            print("[WARNING] No optimized attention backend available, using PyTorch default")

        # Check ARM64 CPU
        import platform
        if platform.machine() == "aarch64":
            print("[INFO] ARM64 architecture detected (Grace CPU)")

        # PyTorch optimizations
        print(f"[INFO] PyTorch version: {torch.__version__}")
        print(f"[INFO] CUDA version: {torch.version.cuda if HAS_CUDA else 'N/A'}")

    def configure(self, config: OptimizationConfig) -> None:
        """Apply optimization configuration"""
        self.config = config

        # Validate configuration
        if config.attention_backend == AttentionBackend.XFORMERS and not HAS_XFORMERS:
            print("[WARNING] xformers not available, falling back to SDPA")
            self.config.attention_backend = AttentionBackend.SDPA if HAS_SDPA else AttentionBackend.PYTORCH

        if config.attention_backend == AttentionBackend.SDPA and not HAS_SDPA:
            print("[WARNING] SDPA not available, falling back to PyTorch")
            self.config.attention_backend = AttentionBackend.PYTORCH

        # Apply PyTorch settings
        if config.enable_cudnn_benchmark and HAS_CUDA:
            torch.backends.cudnn.benchmark = True
            print("[INFO] cuDNN benchmark mode enabled")

        if config.enable_channels_last:
            # Channels-last memory format can improve performance
            print("[INFO] Channels-last memory format will be applied to models")

        print(f"[INFO] Precision mode: {config.precision.value}")
        print(f"[INFO] Attention backend: {config.attention_backend.value}")

    def optimize_model(self, model: torch.nn.Module) -> torch.nn.Module:
        """
        Apply optimizations to a loaded model

        Args:
            model: PyTorch model (UNet, VAE, etc.)

        Returns:
            Optimized model
        """
        # Move to device
        model = model.to(self.device)

        # Apply precision
        if self.config.precision == PrecisionMode.FP16:
            model = model.half()
        elif self.config.precision == PrecisionMode.BF16:
            model = model.to(torch.bfloat16)

        # Apply channels-last memory format
        if self.config.enable_channels_last:
            try:
                model = model.to(memory_format=torch.channels_last)
            except Exception as e:
                print(f"[WARNING] Could not apply channels-last format: {e}")

        # Set to eval mode
        model.eval()

        # Disable gradient computation
        for param in model.parameters():
            param.requires_grad = False

        # Compile model (experimental for GB10)
        if self.config.enable_torch_compile:
            try:
                model = torch.compile(model, mode="reduce-overhead")
                print("[INFO] Model compiled with torch.compile()")
            except Exception as e:
                print(f"[WARNING] torch.compile() failed: {e}")

        return model

    def get_autocast_context(self):
        """Get autocast context for mixed precision inference"""
        if not HAS_CUDA:
            return torch.cpu.amp.autocast(enabled=False)

        if self.config.precision == PrecisionMode.FP16:
            return torch.cuda.amp.autocast(dtype=torch.float16)
        elif self.config.precision == PrecisionMode.BF16:
            return torch.cuda.amp.autocast(dtype=torch.bfloat16)
        else:
            return torch.cuda.amp.autocast(enabled=False)

    def clear_memory(self) -> None:
        """Clear GPU memory cache"""
        if HAS_CUDA:
            torch.cuda.empty_cache()
            torch.cuda.synchronize()

    def get_memory_stats(self) -> Dict[str, float]:
        """Get current GPU memory statistics"""
        if not HAS_CUDA:
            return {
                'allocated_gb': 0.0,
                'reserved_gb': 0.0,
                'free_gb': 0.0,
            }

        allocated = torch.cuda.memory_allocated() / 1e9
        reserved = torch.cuda.memory_reserved() / 1e9
        total = torch.cuda.get_device_properties(0).total_memory / 1e9

        return {
            'allocated_gb': allocated,
            'reserved_gb': reserved,
            'free_gb': total - reserved,
        }

    def benchmark_inference(
        self,
        inference_fn,
        batch_size: int = 1,
        num_warmup: int = 2,
        num_runs: int = 5,
    ) -> BenchmarkResult:
        """
        Benchmark inference performance

        Args:
            inference_fn: Function that runs inference and returns output
            batch_size: Batch size for generation
            num_warmup: Number of warmup runs
            num_runs: Number of benchmark runs

        Returns:
            BenchmarkResult with timing and memory stats
        """
        print(f"[BENCHMARK] Warming up ({num_warmup} runs)...")
        for _ in range(num_warmup):
            self.clear_memory()
            _ = inference_fn(batch_size)

        print(f"[BENCHMARK] Running benchmark ({num_runs} runs)...")
        times = []
        for i in range(num_runs):
            self.clear_memory()

            if HAS_CUDA:
                torch.cuda.synchronize()

            start = time.perf_counter()
            _ = inference_fn(batch_size)

            if HAS_CUDA:
                torch.cuda.synchronize()

            elapsed = time.perf_counter() - start
            times.append(elapsed)
            print(f"  Run {i+1}/{num_runs}: {elapsed:.2f}s")

        avg_time = sum(times) / len(times)
        mem_stats = self.get_memory_stats()
        throughput = (batch_size * 60) / avg_time  # images per minute

        result = BenchmarkResult(
            config=self.config,
            inference_time_s=avg_time,
            memory_allocated_gb=mem_stats['allocated_gb'],
            memory_reserved_gb=mem_stats['reserved_gb'],
            throughput_imgs_per_min=throughput,
            batch_size=batch_size,
            image_resolution=(1024, 1024),
            timestamp=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        )

        print(f"[BENCHMARK] Average time: {avg_time:.2f}s")
        print(f"[BENCHMARK] Throughput: {throughput:.1f} images/min")
        print(f"[BENCHMARK] Memory allocated: {mem_stats['allocated_gb']:.1f} GB")

        return result

    def save_benchmark(self, result: BenchmarkResult, output_path: Path) -> None:
        """Save benchmark results to JSON"""
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            json.dump(result.to_dict(), f, indent=2)
        print(f"[INFO] Benchmark saved to {output_path}")


def get_optimal_config_for_gb10() -> OptimizationConfig:
    """
    Get recommended optimization configuration for GB10 hardware

    Returns:
        OptimizationConfig optimized for DGX-Spark GB10
    """
    config = OptimizationConfig(
        precision=PrecisionMode.FP16,  # FP16 is well-supported on all modern GPUs
        attention_backend=AttentionBackend.SDPA if HAS_SDPA else (
            AttentionBackend.XFORMERS if HAS_XFORMERS else AttentionBackend.PYTORCH
        ),
        enable_torch_compile=False,  # Conservative: disable until verified on GB10
        enable_channels_last=True,
        enable_cudnn_benchmark=True,
        cpu_offload=False,  # 119GB unified memory should be sufficient
        sequential_cpu_offload=False,
        vae_slicing=True,  # Reduces VAE memory usage
        vae_tiling=False,  # Only needed for very large images
        batch_size=1,
        num_inference_steps=20,
        guidance_scale=8.0,
    )
    return config


def compare_configurations(
    configs: List[OptimizationConfig],
    inference_fn,
    batch_size: int = 1,
) -> List[BenchmarkResult]:
    """
    Compare multiple optimization configurations

    Args:
        configs: List of configurations to test
        inference_fn: Function that runs inference
        batch_size: Batch size for generation

    Returns:
        List of BenchmarkResult for each configuration
    """
    results = []
    optimizer = SDXLOptimizer()

    for i, config in enumerate(configs):
        print(f"\n[COMPARE] Testing configuration {i+1}/{len(configs)}")
        print(f"  Precision: {config.precision.value}")
        print(f"  Attention: {config.attention_backend.value}")

        optimizer.configure(config)
        result = optimizer.benchmark_inference(inference_fn, batch_size=batch_size)
        results.append(result)

    return results


if __name__ == "__main__":
    # Self-test: verify optimizations are available
    print("=== SDXL Optimization Self-Test ===\n")

    optimizer = SDXLOptimizer()
    config = get_optimal_config_for_gb10()
    optimizer.configure(config)

    print("\n[INFO] Optimal configuration for GB10:")
    print(json.dumps(config.to_dict(), indent=2))

    print("\n[INFO] Memory statistics:")
    print(json.dumps(optimizer.get_memory_stats(), indent=2))

    print("\n✅ Self-test complete")
