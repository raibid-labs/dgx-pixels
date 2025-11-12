#!/usr/bin/env python3
"""
Unit Tests for SDXL Optimization Module

Tests optimization configurations, memory profiling, and benchmarking utilities.
"""

import pytest
import torch
import json
import sys
from pathlib import Path

# Add python directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "python" / "optimization"))

from sdxl_optimizations import (
    SDXLOptimizer,
    OptimizationConfig,
    PrecisionMode,
    AttentionBackend,
    BenchmarkResult,
    get_optimal_config_for_gb10,
)
from memory_profiler import MemoryProfiler, MemorySnapshot, MemoryProfile


class TestOptimizationConfig:
    """Test OptimizationConfig dataclass"""

    def test_default_config(self):
        """Test default configuration values"""
        config = OptimizationConfig()
        assert config.precision == PrecisionMode.FP16
        assert config.batch_size == 1
        assert config.num_inference_steps == 20
        assert config.enable_channels_last is True

    def test_config_to_dict(self):
        """Test configuration serialization"""
        config = OptimizationConfig(
            precision=PrecisionMode.FP16,
            attention_backend=AttentionBackend.SDPA,
        )
        d = config.to_dict()
        assert isinstance(d, dict)
        assert d['precision'] == 'fp16'
        assert d['attention_backend'] == 'sdpa'

    def test_custom_config(self):
        """Test custom configuration"""
        config = OptimizationConfig(
            precision=PrecisionMode.BF16,
            batch_size=4,
            num_inference_steps=30,
        )
        assert config.precision == PrecisionMode.BF16
        assert config.batch_size == 4
        assert config.num_inference_steps == 30


class TestSDXLOptimizer:
    """Test SDXLOptimizer class"""

    def test_optimizer_init(self):
        """Test optimizer initialization"""
        optimizer = SDXLOptimizer()
        assert optimizer.device is not None
        assert optimizer.config is not None

    def test_configure(self):
        """Test configuration application"""
        optimizer = SDXLOptimizer()
        config = OptimizationConfig(precision=PrecisionMode.FP16)
        optimizer.configure(config)
        assert optimizer.config.precision == PrecisionMode.FP16

    def test_get_memory_stats(self):
        """Test memory statistics retrieval"""
        optimizer = SDXLOptimizer()
        stats = optimizer.get_memory_stats()
        assert isinstance(stats, dict)
        assert 'allocated_gb' in stats
        assert 'reserved_gb' in stats
        assert 'free_gb' in stats
        assert all(v >= 0 for v in stats.values())

    def test_clear_memory(self):
        """Test memory clearing"""
        optimizer = SDXLOptimizer()
        # Should not raise
        optimizer.clear_memory()

    def test_autocast_context_fp16(self):
        """Test FP16 autocast context"""
        optimizer = SDXLOptimizer()
        config = OptimizationConfig(precision=PrecisionMode.FP16)
        optimizer.configure(config)
        context = optimizer.get_autocast_context()
        assert context is not None

    def test_autocast_context_bf16(self):
        """Test BF16 autocast context"""
        optimizer = SDXLOptimizer()
        config = OptimizationConfig(precision=PrecisionMode.BF16)
        optimizer.configure(config)
        context = optimizer.get_autocast_context()
        assert context is not None

    @pytest.mark.skipif(not torch.cuda.is_available(), reason="CUDA not available")
    def test_optimize_model_fp16(self):
        """Test model optimization with FP16"""
        optimizer = SDXLOptimizer()
        config = OptimizationConfig(precision=PrecisionMode.FP16)
        optimizer.configure(config)

        # Create dummy model
        model = torch.nn.Linear(10, 10)
        optimized = optimizer.optimize_model(model)

        assert optimized.weight.dtype == torch.float16

    def test_optimal_config_generation(self):
        """Test optimal configuration for GB10"""
        config = get_optimal_config_for_gb10()
        assert isinstance(config, OptimizationConfig)
        assert config.precision in [PrecisionMode.FP16, PrecisionMode.BF16]
        assert config.enable_cudnn_benchmark is True


class TestBenchmarkResult:
    """Test BenchmarkResult dataclass"""

    def test_benchmark_result_creation(self):
        """Test benchmark result creation"""
        config = OptimizationConfig()
        result = BenchmarkResult(
            config=config,
            inference_time_s=2.5,
            memory_allocated_gb=45.2,
            memory_reserved_gb=50.0,
            throughput_imgs_per_min=24.0,
            batch_size=1,
            image_resolution=(1024, 1024),
            timestamp="2025-11-11T00:00:00Z",
        )

        assert result.inference_time_s == 2.5
        assert result.memory_allocated_gb == 45.2
        assert result.batch_size == 1

    def test_benchmark_result_to_dict(self):
        """Test benchmark result serialization"""
        config = OptimizationConfig()
        result = BenchmarkResult(
            config=config,
            inference_time_s=2.5,
            memory_allocated_gb=45.2,
            memory_reserved_gb=50.0,
            throughput_imgs_per_min=24.0,
            batch_size=1,
            image_resolution=(1024, 1024),
            timestamp="2025-11-11T00:00:00Z",
        )

        d = result.to_dict()
        assert isinstance(d, dict)
        assert d['inference_time_s'] == 2.5
        assert d['batch_size'] == 1
        assert 'config' in d


class TestMemoryProfiler:
    """Test MemoryProfiler class"""

    def test_profiler_init(self):
        """Test profiler initialization"""
        profiler = MemoryProfiler()
        assert profiler.profiles == []
        assert profiler.current_profile is None

    def test_take_snapshot(self):
        """Test memory snapshot"""
        profiler = MemoryProfiler()
        snapshot = profiler.take_snapshot("test_stage")

        assert isinstance(snapshot, MemorySnapshot)
        assert snapshot.stage == "test_stage"
        assert snapshot.gpu_allocated_gb >= 0
        assert snapshot.system_used_gb > 0

    def test_profile_context(self):
        """Test profile context manager"""
        profiler = MemoryProfiler()

        with profiler.profile("test_config", batch_size=1):
            profiler.take_snapshot("start")
            profiler.take_snapshot("end")

        assert len(profiler.profiles) == 1
        profile = profiler.profiles[0]
        assert profile.config_name == "test_config"
        assert profile.batch_size == 1
        assert len(profile.snapshots) == 2

    def test_memory_snapshot_to_dict(self):
        """Test snapshot serialization"""
        profiler = MemoryProfiler()
        snapshot = profiler.take_snapshot("test")
        d = snapshot.to_dict()

        assert isinstance(d, dict)
        assert 'stage' in d
        assert 'gpu_allocated_gb' in d
        assert d['stage'] == 'test'

    def test_memory_profile_to_dict(self):
        """Test profile serialization"""
        profiler = MemoryProfiler()

        with profiler.profile("test", batch_size=2):
            profiler.take_snapshot("test_stage")

        profile = profiler.profiles[0]
        d = profile.to_dict()

        assert isinstance(d, dict)
        assert d['config_name'] == 'test'
        assert d['batch_size'] == 2
        assert 'snapshots' in d


class TestIntegration:
    """Integration tests for optimization + profiling"""

    def test_optimizer_with_profiler(self):
        """Test optimizer with memory profiler"""
        optimizer = SDXLOptimizer()
        profiler = MemoryProfiler()

        config = get_optimal_config_for_gb10()
        optimizer.configure(config)

        with profiler.profile("integration_test"):
            profiler.take_snapshot("start")
            stats = optimizer.get_memory_stats()
            profiler.take_snapshot("after_stats")

        assert len(profiler.profiles) == 1
        assert stats['allocated_gb'] >= 0

    @pytest.mark.skipif(not torch.cuda.is_available(), reason="CUDA not available")
    def test_model_optimization_with_memory_tracking(self):
        """Test model optimization with memory tracking"""
        optimizer = SDXLOptimizer()
        profiler = MemoryProfiler()

        config = OptimizationConfig(precision=PrecisionMode.FP16)
        optimizer.configure(config)

        model = torch.nn.Linear(1000, 1000)

        with profiler.profile("model_optimization"):
            profiler.take_snapshot("before_optimize")
            optimized = optimizer.optimize_model(model)
            profiler.take_snapshot("after_optimize")

        profile = profiler.profiles[0]
        assert len(profile.snapshots) == 2
        assert profile.peak_gpu_allocated_gb >= 0


def test_precision_modes():
    """Test all precision modes are valid"""
    modes = [PrecisionMode.FP32, PrecisionMode.FP16, PrecisionMode.BF16, PrecisionMode.FP8]
    for mode in modes:
        assert mode.value in ['fp32', 'fp16', 'bf16', 'fp8']


def test_attention_backends():
    """Test all attention backends are valid"""
    backends = [AttentionBackend.PYTORCH, AttentionBackend.XFORMERS, AttentionBackend.SDPA]
    for backend in backends:
        assert backend.value in ['pytorch', 'xformers', 'sdpa']


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v", "--tb=short"])
