#!/usr/bin/env python3
"""
Test suite for dataset preparation utilities
"""

import pytest
import torch
from PIL import Image
import numpy as np
from pathlib import Path
import tempfile
import shutil
import sys

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from python.training.dataset_prep import (
    DatasetPreparator,
    ImageMetadata,
    DatasetStats,
)


@pytest.fixture
def temp_dataset_dir():
    """Create temporary dataset directory"""
    temp_dir = Path(tempfile.mkdtemp())
    yield temp_dir
    shutil.rmtree(temp_dir)


@pytest.fixture
def sample_images(temp_dataset_dir):
    """Create sample images for testing"""
    images = []

    for i in range(5):
        # Create random image
        img_array = np.random.randint(0, 255, (512, 512, 3), dtype=np.uint8)
        img = Image.fromarray(img_array)

        # Save image
        img_path = temp_dataset_dir / f"image_{i:02d}.png"
        img.save(img_path)
        images.append(img_path)

        # Create caption for some images
        if i < 3:
            caption_path = img_path.with_suffix('.txt')
            with open(caption_path, 'w') as f:
                f.write(f"pixel art sprite {i}, game asset, 16-bit style")

    return images


class TestDatasetPreparator:
    """Test DatasetPreparator class"""

    def test_initialization(self):
        """Test DatasetPreparator initialization"""
        prep = DatasetPreparator(target_resolution=1024)
        assert prep.target_resolution == 1024
        assert prep.quality_threshold == 512

    def test_custom_parameters(self):
        """Test DatasetPreparator with custom parameters"""
        prep = DatasetPreparator(
            target_resolution=512,
            quality_threshold=256,
            supported_formats=['png', 'jpg'],
        )
        assert prep.target_resolution == 512
        assert prep.quality_threshold == 256
        assert 'png' in prep.supported_formats
        assert 'jpg' in prep.supported_formats

    def test_find_images(self, temp_dataset_dir, sample_images):
        """Test finding images in directory"""
        prep = DatasetPreparator()
        found_images = prep._find_images(temp_dataset_dir)

        assert len(found_images) == 5
        assert all(img.suffix == '.png' for img in found_images)

    def test_validate_dataset_valid(self, temp_dataset_dir, sample_images):
        """Test dataset validation with valid dataset"""
        # Create captions for all images
        for img_path in sample_images:
            caption_path = img_path.with_suffix('.txt')
            if not caption_path.exists():
                with open(caption_path, 'w') as f:
                    f.write("pixel art sprite")

        prep = DatasetPreparator()
        is_valid = prep.validate_dataset(temp_dataset_dir)
        assert is_valid

    def test_validate_dataset_missing_captions(self, temp_dataset_dir, sample_images):
        """Test dataset validation with missing captions"""
        prep = DatasetPreparator()
        is_valid = prep.validate_dataset(temp_dataset_dir)

        # Should fail because 2 images don't have captions
        assert not is_valid

    def test_prepare_dataset_validate_only(self, temp_dataset_dir, sample_images):
        """Test dataset preparation in validate-only mode"""
        prep = DatasetPreparator()
        output_dir = temp_dataset_dir / "output"

        stats = prep.prepare_dataset(
            input_dir=temp_dataset_dir,
            output_dir=output_dir,
            validate_only=True,
        )

        # Should not create output directory in validate-only mode
        assert not output_dir.exists()

        # Should return stats
        assert stats.total_images == 5
        assert stats.images_with_captions == 3
        assert stats.images_without_captions == 2

    def test_prepare_dataset_with_processing(self, temp_dataset_dir, sample_images):
        """Test dataset preparation with image processing"""
        prep = DatasetPreparator(target_resolution=512)
        output_dir = temp_dataset_dir / "output"

        stats = prep.prepare_dataset(
            input_dir=temp_dataset_dir,
            output_dir=output_dir,
            validate_only=False,
        )

        # Should create output directory
        assert output_dir.exists()

        # Should process all images
        assert stats.total_images == 5

        # Check processed images
        processed_images = list(output_dir.glob("*.png"))
        assert len(processed_images) == 5

        # Check image size
        for img_path in processed_images:
            img = Image.open(img_path)
            assert img.size == (512, 512)

    def test_prepare_dataset_create_captions(self, temp_dataset_dir, sample_images):
        """Test dataset preparation with caption creation"""
        prep = DatasetPreparator()
        output_dir = temp_dataset_dir / "output"

        stats = prep.prepare_dataset(
            input_dir=temp_dataset_dir,
            output_dir=output_dir,
            create_captions=True,
            validate_only=False,
        )

        # All images should have captions
        assert stats.images_with_captions == 5
        assert stats.images_without_captions == 0

        # Check caption files
        caption_files = list(output_dir.glob("*.txt"))
        assert len(caption_files) == 5

    def test_resize_image_pad_to_square(self):
        """Test image resizing with padding to square"""
        prep = DatasetPreparator(target_resolution=1024)

        # Create rectangular image
        img = Image.new('RGB', (800, 600), color='red')

        # Resize with padding
        resized = prep._resize_image(img, pad_to_square=True)

        # Should be square at target resolution
        assert resized.size == (1024, 1024)

    def test_resize_image_no_padding(self):
        """Test image resizing without padding"""
        prep = DatasetPreparator(target_resolution=1024)

        # Create rectangular image
        img = Image.new('RGB', (800, 600), color='blue')

        # Resize without padding (direct stretch)
        resized = prep._resize_image(img, pad_to_square=False)

        # Should be square at target resolution
        assert resized.size == (1024, 1024)

    def test_calculate_stats(self, temp_dataset_dir, sample_images):
        """Test dataset statistics calculation"""
        prep = DatasetPreparator()

        # Process images to get metadata
        metadata_list = []
        for img_path in sample_images:
            caption_path = img_path.with_suffix('.txt')
            has_caption = caption_path.exists()

            metadata = ImageMetadata(
                filename=img_path.name,
                original_size=(512, 512),
                processed_size=(1024, 1024),
                format='PNG',
                file_size_bytes=img_path.stat().st_size,
                has_caption=has_caption,
                caption_length=50 if has_caption else None,
                hash='abc123',
            )
            metadata_list.append(metadata)

        stats = prep._calculate_stats(metadata_list)

        assert stats.total_images == 5
        assert stats.images_with_captions == 3
        assert stats.images_without_captions == 2
        assert stats.min_resolution == (512, 512)
        assert stats.max_resolution == (512, 512)


class TestImageMetadata:
    """Test ImageMetadata dataclass"""

    def test_image_metadata_creation(self):
        """Test creating ImageMetadata"""
        metadata = ImageMetadata(
            filename="test.png",
            original_size=(512, 512),
            processed_size=(1024, 1024),
            format='PNG',
            file_size_bytes=1024,
            has_caption=True,
            caption_length=50,
            hash='abc123',
        )

        assert metadata.filename == "test.png"
        assert metadata.original_size == (512, 512)
        assert metadata.has_caption is True


class TestDatasetStats:
    """Test DatasetStats dataclass"""

    def test_dataset_stats_creation(self):
        """Test creating DatasetStats"""
        stats = DatasetStats(
            total_images=10,
            images_with_captions=8,
            images_without_captions=2,
            avg_caption_length=45.5,
            min_resolution=(256, 256),
            max_resolution=(1024, 1024),
            total_size_mb=50.5,
            formats={'PNG': 8, 'JPG': 2},
        )

        assert stats.total_images == 10
        assert stats.images_with_captions == 8
        assert stats.avg_caption_length == 45.5

    def test_dataset_stats_to_dict(self):
        """Test converting DatasetStats to dictionary"""
        stats = DatasetStats(
            total_images=5,
            images_with_captions=3,
            images_without_captions=2,
            avg_caption_length=40.0,
            min_resolution=(512, 512),
            max_resolution=(1024, 1024),
            total_size_mb=25.0,
            formats={'PNG': 5},
        )

        stats_dict = stats.to_dict()
        assert isinstance(stats_dict, dict)
        assert stats_dict['total_images'] == 5
        assert stats_dict['formats']['PNG'] == 5


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
