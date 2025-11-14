#!/usr/bin/env python3
"""
Test suite for validation generation in LoRA trainer
"""

import pytest
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from python.training.lora_trainer import (
    LoRAConfig,
    ValidationMetrics,
)


class TestValidationConfig:
    def test_default_validation_prompts(self):
        config = LoRAConfig()
        assert len(config.validation_prompts) > 0

    def test_validation_interval(self):
        config = LoRAConfig(validation_every_n_steps=100)
        assert config.validation_every_n_steps == 100


class TestValidationMetrics:
    def test_validation_metrics_creation(self):
        metrics = ValidationMetrics(
            step=500,
            epoch=5,
            num_samples=3,
            generation_time_avg=4.2,
            output_dir="/path/to/validation",
            timestamp="2025-01-01T12:00:00",
            prompts=["prompt1"],
            seeds=[42],
        )
        assert metrics.step == 500


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
