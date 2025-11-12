#!/usr/bin/env python3
"""
Integration tests for LoRA training pipeline
"""

import pytest
import torch
import sys
from pathlib import Path
import tempfile
import shutil
from PIL import Image
import numpy as np

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from python.training.lora_trainer import LoRATrainer, LoRAConfig, PixelArtDataset
from python.training.dataset_prep import DatasetPreparator


@pytest.fixture
def temp_dirs():
    """Create temporary directories for testing"""
    temp_dir = Path(tempfile.mkdtemp())
    dataset_dir = temp_dir / "dataset"
    output_dir = temp_dir / "output"

    dataset_dir.mkdir()
    output_dir.mkdir()

    yield {
        'root': temp_dir,
        'dataset': dataset_dir,
        'output': output_dir,
    }

    shutil.rmtree(temp_dir)


@pytest.fixture
def mock_dataset(temp_dirs):
    """Create mock dataset for testing"""
    dataset_dir = temp_dirs['dataset']

    # Create 10 sample images with captions
    for i in range(10):
        # Create random image
        img_array = np.random.randint(0, 255, (512, 512, 3), dtype=np.uint8)
        img = Image.fromarray(img_array)

        # Save image
        img_path = dataset_dir / f"sprite_{i:02d}.png"
        img.save(img_path)

        # Create caption
        caption_path = img_path.with_suffix('.txt')
        with open(caption_path, 'w') as f:
            f.write(f"pixel art sprite {i}, game character, 16-bit style")

    return dataset_dir


class TestPixelArtDataset:
    """Test PixelArtDataset class"""

    def test_dataset_creation(self, mock_dataset):
        """Test creating PixelArtDataset"""
        # Mock tokenizers (would normally come from model)
        from transformers import CLIPTokenizer

        tokenizer = CLIPTokenizer.from_pretrained("openai/clip-vit-large-patch14")

        dataset = PixelArtDataset(
            data_dir=mock_dataset,
            tokenizer_1=tokenizer,
            tokenizer_2=tokenizer,
            resolution=512,
        )

        assert len(dataset) == 10

    def test_dataset_getitem(self, mock_dataset):
        """Test getting item from dataset"""
        from transformers import CLIPTokenizer

        tokenizer = CLIPTokenizer.from_pretrained("openai/clip-vit-large-patch14")

        dataset = PixelArtDataset(
            data_dir=mock_dataset,
            tokenizer_1=tokenizer,
            tokenizer_2=tokenizer,
            resolution=512,
        )

        sample = dataset[0]

        assert 'pixel_values' in sample
        assert 'input_ids_1' in sample
        assert 'input_ids_2' in sample
        assert 'caption' in sample

        # Check tensor shapes
        assert sample['pixel_values'].shape == (3, 512, 512)
        assert sample['input_ids_1'].dim() == 1
        assert sample['input_ids_2'].dim() == 1

    def test_dataset_missing_caption(self, temp_dirs):
        """Test dataset with missing caption"""
        dataset_dir = temp_dirs['dataset']

        # Create image without caption
        img = Image.new('RGB', (512, 512), color='red')
        img_path = dataset_dir / "test.png"
        img.save(img_path)

        from transformers import CLIPTokenizer
        tokenizer = CLIPTokenizer.from_pretrained("openai/clip-vit-large-patch14")

        dataset = PixelArtDataset(
            data_dir=dataset_dir,
            tokenizer_1=tokenizer,
            tokenizer_2=tokenizer,
        )

        # Should still work with default caption
        sample = dataset[0]
        assert 'caption' in sample


class TestLoRATrainer:
    """Test LoRATrainer class"""

    def test_trainer_initialization(self):
        """Test LoRATrainer initialization"""
        config = LoRAConfig(
            lora_rank=16,
            max_train_steps=100,
        )

        trainer = LoRATrainer(
            config=config,
            output_dir="./test_output",
        )

        assert trainer.config.lora_rank == 16
        assert trainer.config.max_train_steps == 100
        assert trainer.global_step == 0

    @pytest.mark.skipif(not torch.cuda.is_available(), reason="CUDA not available")
    def test_trainer_device_setup(self):
        """Test device setup"""
        trainer = LoRATrainer(device="cuda")
        assert trainer.device == "cuda"

    def test_config_to_dict(self):
        """Test config serialization"""
        config = LoRAConfig(lora_rank=32)
        trainer = LoRATrainer(config=config)

        config_dict = trainer.config.to_dict()
        assert isinstance(config_dict, dict)
        assert config_dict['lora_rank'] == 32


class TestDatasetPreparation:
    """Test dataset preparation integration"""

    def test_prepare_and_validate(self, mock_dataset, temp_dirs):
        """Test preparing and validating dataset"""
        prep = DatasetPreparator(target_resolution=1024)
        output_dir = temp_dirs['output'] / "prepared"

        # Prepare dataset
        stats = prep.prepare_dataset(
            input_dir=mock_dataset,
            output_dir=output_dir,
        )

        assert stats.total_images == 10
        assert stats.images_with_captions == 10

        # Validate prepared dataset
        is_valid = prep.validate_dataset(output_dir)
        assert is_valid

    def test_end_to_end_dataset_flow(self, temp_dirs):
        """Test complete dataset preparation flow"""
        raw_dir = temp_dirs['dataset']
        prepared_dir = temp_dirs['output'] / "prepared"

        # Create raw images
        for i in range(5):
            img = Image.new('RGB', (800, 600), color=(i*50, 100, 150))
            img.save(raw_dir / f"img_{i}.png")

        # Prepare dataset
        prep = DatasetPreparator(target_resolution=1024)
        stats = prep.prepare_dataset(
            input_dir=raw_dir,
            output_dir=prepared_dir,
            create_captions=True,
        )

        # Check results
        assert stats.total_images == 5
        assert stats.images_with_captions == 5

        # Verify processed images
        processed_images = list(prepared_dir.glob("*.png"))
        assert len(processed_images) == 5

        for img_path in processed_images:
            img = Image.open(img_path)
            assert img.size == (1024, 1024)


class TestTrainingMetrics:
    """Test training metrics tracking"""

    def test_metrics_collection(self):
        """Test collecting training metrics"""
        from python.training.lora_trainer import TrainingMetrics

        metrics = TrainingMetrics(
            step=100,
            epoch=1,
            loss=0.5,
            learning_rate=1e-4,
            time_per_step=2.0,
            memory_allocated_gb=40.0,
            timestamp="2025-01-01T12:00:00",
        )

        assert metrics.step == 100
        assert metrics.loss == 0.5

    def test_metrics_serialization(self):
        """Test metrics serialization"""
        from python.training.lora_trainer import TrainingMetrics
        import json

        metrics = TrainingMetrics(
            step=200,
            epoch=2,
            loss=0.3,
            learning_rate=5e-5,
            time_per_step=1.5,
            memory_allocated_gb=35.0,
            timestamp="2025-01-01T13:00:00",
        )

        metrics_dict = metrics.to_dict()
        metrics_json = json.dumps(metrics_dict)

        # Should be serializable
        assert isinstance(metrics_json, str)

        # Should be deserializable
        deserialized = json.loads(metrics_json)
        assert deserialized['step'] == 200


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
