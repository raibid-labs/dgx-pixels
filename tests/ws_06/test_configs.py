#!/usr/bin/env python3
"""
Test training configuration files
"""

import pytest
import yaml
from pathlib import Path
import sys

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))


class TestConfigFiles:
    """Test YAML configuration files"""

    def test_base_config_exists(self):
        """Test that base config file exists"""
        config_path = Path("configs/training/pixel_art_base.yaml")
        assert config_path.exists(), f"Config file not found: {config_path}"

    def test_fast_config_exists(self):
        """Test that fast config file exists"""
        config_path = Path("configs/training/fast_training.yaml")
        assert config_path.exists(), f"Config file not found: {config_path}"

    def test_quality_config_exists(self):
        """Test that quality config file exists"""
        config_path = Path("configs/training/quality_training.yaml")
        assert config_path.exists(), f"Config file not found: {config_path}"

    def test_base_config_valid_yaml(self):
        """Test that base config is valid YAML"""
        config_path = Path("configs/training/pixel_art_base.yaml")
        with open(config_path) as f:
            config = yaml.safe_load(f)
        assert config is not None
        assert isinstance(config, dict)

    def test_base_config_structure(self):
        """Test base config has required sections"""
        config_path = Path("configs/training/pixel_art_base.yaml")
        with open(config_path) as f:
            config = yaml.safe_load(f)

        assert 'model' in config
        assert 'lora' in config
        assert 'training' in config
        assert 'optimization' in config
        assert 'dataset' in config

    def test_base_config_lora_params(self):
        """Test base config LoRA parameters"""
        config_path = Path("configs/training/pixel_art_base.yaml")
        with open(config_path) as f:
            config = yaml.safe_load(f)

        lora = config['lora']
        assert 'rank' in lora
        assert 'alpha' in lora
        assert 'dropout' in lora
        assert lora['rank'] == 32
        assert lora['alpha'] == 32

    def test_base_config_training_params(self):
        """Test base config training parameters"""
        config_path = Path("configs/training/pixel_art_base.yaml")
        with open(config_path) as f:
            config = yaml.safe_load(f)

        training = config['training']
        assert 'learning_rate' in training
        assert 'batch_size' in training
        assert 'max_train_steps' in training
        assert training['max_train_steps'] == 3000

    def test_fast_config_faster_than_base(self):
        """Test that fast config has fewer steps than base"""
        base_config = Path("configs/training/pixel_art_base.yaml")
        fast_config = Path("configs/training/fast_training.yaml")

        with open(base_config) as f:
            base = yaml.safe_load(f)

        with open(fast_config) as f:
            fast = yaml.safe_load(f)

        assert fast['training']['max_train_steps'] < base['training']['max_train_steps']

    def test_quality_config_more_steps_than_base(self):
        """Test that quality config has more steps than base"""
        base_config = Path("configs/training/pixel_art_base.yaml")
        quality_config = Path("configs/training/quality_training.yaml")

        with open(base_config) as f:
            base = yaml.safe_load(f)

        with open(quality_config) as f:
            quality = yaml.safe_load(f)

        assert quality['training']['max_train_steps'] > base['training']['max_train_steps']

    def test_quality_config_higher_rank(self):
        """Test that quality config uses higher LoRA rank"""
        base_config = Path("configs/training/pixel_art_base.yaml")
        quality_config = Path("configs/training/quality_training.yaml")

        with open(base_config) as f:
            base = yaml.safe_load(f)

        with open(quality_config) as f:
            quality = yaml.safe_load(f)

        assert quality['lora']['rank'] > base['lora']['rank']


class TestModuleStructure:
    """Test Python module structure"""

    def test_training_module_exists(self):
        """Test training module directory exists"""
        assert Path("python/training").exists()
        assert Path("python/training/__init__.py").exists()

    def test_training_scripts_exist(self):
        """Test training scripts exist"""
        assert Path("python/training/lora_trainer.py").exists()
        assert Path("python/training/dataset_prep.py").exists()
        assert Path("python/training/captioning.py").exists()

    def test_validation_module_exists(self):
        """Test validation module exists"""
        assert Path("python/training/validation").exists()
        assert Path("python/training/validation/__init__.py").exists()
        assert Path("python/training/validation/validate_lora.py").exists()

    def test_requirements_exists(self):
        """Test training requirements file exists"""
        assert Path("python/training/requirements.txt").exists()

    def test_config_directory_exists(self):
        """Test config directory exists"""
        assert Path("configs/training").exists()

    def test_dataset_directory_exists(self):
        """Test dataset directory exists"""
        assert Path("datasets").exists()


class TestDocumentation:
    """Test documentation files"""

    def test_training_guide_exists(self):
        """Test LoRA training guide exists"""
        assert Path("docs/lora-training-guide.md").exists()

    def test_dataset_guide_exists(self):
        """Test dataset preparation guide exists"""
        assert Path("docs/dataset-preparation.md").exists()

    def test_training_guide_not_empty(self):
        """Test training guide has content"""
        guide_path = Path("docs/lora-training-guide.md")
        content = guide_path.read_text()
        assert len(content) > 1000  # Should have substantial content


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
