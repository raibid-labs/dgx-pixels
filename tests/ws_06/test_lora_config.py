#!/usr/bin/env python3
"""
Test suite for LoRA configuration
"""

import pytest
import sys
from pathlib import Path

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from python.training.lora_trainer import LoRAConfig, TrainingMetrics
from python.optimization.sdxl_optimizations import PrecisionMode


class TestLoRAConfig:
    """Test LoRAConfig dataclass"""

    def test_default_config(self):
        """Test default LoRAConfig creation"""
        config = LoRAConfig()

        assert config.lora_rank == 32
        assert config.lora_alpha == 32
        assert config.lora_dropout == 0.1
        assert config.learning_rate == 1e-4
        assert config.batch_size == 2
        assert config.gradient_accumulation_steps == 2
        assert config.max_train_steps == 3000
        assert config.resolution == 1024
        assert config.mixed_precision == PrecisionMode.FP16

    def test_custom_config(self):
        """Test custom LoRAConfig parameters"""
        config = LoRAConfig(
            lora_rank=64,
            lora_alpha=64,
            learning_rate=5e-5,
            batch_size=4,
            max_train_steps=5000,
        )

        assert config.lora_rank == 64
        assert config.lora_alpha == 64
        assert config.learning_rate == 5e-5
        assert config.batch_size == 4
        assert config.max_train_steps == 5000

    def test_target_modules(self):
        """Test LoRA target modules"""
        config = LoRAConfig()

        assert "to_k" in config.target_modules
        assert "to_q" in config.target_modules
        assert "to_v" in config.target_modules
        assert "to_out.0" in config.target_modules

    def test_config_to_dict(self):
        """Test converting config to dictionary"""
        config = LoRAConfig(lora_rank=16)
        config_dict = config.to_dict()

        assert isinstance(config_dict, dict)
        assert config_dict['lora_rank'] == 16
        assert config_dict['mixed_precision'] == 'fp16'

    def test_optimizer_options(self):
        """Test optimizer configuration"""
        config = LoRAConfig(optimizer="adamw_8bit")
        assert config.optimizer == "adamw_8bit"

        config2 = LoRAConfig(optimizer="adamw")
        assert config2.optimizer == "adamw"

    def test_lr_scheduler_options(self):
        """Test learning rate scheduler options"""
        config = LoRAConfig(lr_scheduler="cosine")
        assert config.lr_scheduler == "cosine"

        config2 = LoRAConfig(lr_scheduler="linear")
        assert config2.lr_scheduler == "linear"

    def test_regularization_settings(self):
        """Test regularization parameters"""
        config = LoRAConfig(
            min_snr_gamma=5.0,
            noise_offset=0.05,
        )

        assert config.min_snr_gamma == 5.0
        assert config.noise_offset == 0.05

    def test_gradient_checkpointing(self):
        """Test gradient checkpointing option"""
        config = LoRAConfig(gradient_checkpointing=True)
        assert config.gradient_checkpointing is True

        config2 = LoRAConfig(gradient_checkpointing=False)
        assert config2.gradient_checkpointing is False

    def test_dataset_settings(self):
        """Test dataset configuration"""
        config = LoRAConfig(
            resolution=1024,
            center_crop=True,
            random_flip=True,
        )

        assert config.resolution == 1024
        assert config.center_crop is True
        assert config.random_flip is True


class TestTrainingMetrics:
    """Test TrainingMetrics dataclass"""

    def test_metrics_creation(self):
        """Test creating TrainingMetrics"""
        metrics = TrainingMetrics(
            step=100,
            epoch=1,
            loss=0.5,
            learning_rate=1e-4,
            time_per_step=2.5,
            memory_allocated_gb=45.0,
            timestamp="2025-01-01T12:00:00",
        )

        assert metrics.step == 100
        assert metrics.epoch == 1
        assert metrics.loss == 0.5
        assert metrics.learning_rate == 1e-4
        assert metrics.time_per_step == 2.5
        assert metrics.memory_allocated_gb == 45.0

    def test_metrics_to_dict(self):
        """Test converting metrics to dictionary"""
        metrics = TrainingMetrics(
            step=200,
            epoch=2,
            loss=0.3,
            learning_rate=5e-5,
            time_per_step=2.0,
            memory_allocated_gb=40.0,
            timestamp="2025-01-01T13:00:00",
        )

        metrics_dict = metrics.to_dict()

        assert isinstance(metrics_dict, dict)
        assert metrics_dict['step'] == 200
        assert metrics_dict['loss'] == 0.3


class TestConfigValidation:
    """Test configuration validation"""

    def test_rank_alpha_relationship(self):
        """Test that rank and alpha can be configured independently"""
        config = LoRAConfig(lora_rank=64, lora_alpha=32)
        assert config.lora_rank == 64
        assert config.lora_alpha == 32

    def test_batch_size_gradient_accumulation(self):
        """Test batch size and gradient accumulation"""
        config = LoRAConfig(
            batch_size=2,
            gradient_accumulation_steps=4,
        )

        # Effective batch size = 2 * 4 = 8
        effective_batch = config.batch_size * config.gradient_accumulation_steps
        assert effective_batch == 8

    def test_precision_mode(self):
        """Test precision mode configuration"""
        config = LoRAConfig(mixed_precision=PrecisionMode.FP16)
        assert config.mixed_precision == PrecisionMode.FP16

        config2 = LoRAConfig(mixed_precision=PrecisionMode.FP32)
        assert config2.mixed_precision == PrecisionMode.FP32


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
