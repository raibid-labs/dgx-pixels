#!/usr/bin/env python3
"""
Unit tests for Output Manager
"""

import pytest
import json
import time
import tempfile
from pathlib import Path
import sys

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from python.batch.output_manager import (
    OutputManager,
    ImageMetadata,
    BatchMetadata,
)


class TestImageMetadata:
    """Test ImageMetadata dataclass"""

    def test_image_metadata_creation(self):
        """Test ImageMetadata creation"""
        metadata = ImageMetadata(
            filename="sprite_001.png",
            prompt="pixel art warrior",
            seed=42,
            steps=20,
        )

        assert metadata.filename == "sprite_001.png"
        assert metadata.prompt == "pixel art warrior"
        assert metadata.seed == 42
        assert metadata.steps == 20

    def test_image_metadata_defaults(self):
        """Test ImageMetadata default values"""
        metadata = ImageMetadata(
            filename="test.png",
            prompt="test prompt",
        )

        assert metadata.negative_prompt is None
        assert metadata.seed is None
        assert metadata.steps == 20
        assert metadata.cfg_scale == 8.0
        assert metadata.width == 1024
        assert metadata.height == 1024
        assert metadata.batch_index == 0


class TestBatchMetadata:
    """Test BatchMetadata dataclass"""

    def test_batch_metadata_creation(self):
        """Test BatchMetadata creation"""
        img_metadata = ImageMetadata(
            filename="sprite_001.png",
            prompt="test prompt",
        )

        batch_metadata = BatchMetadata(
            batch_id="batch_123",
            timestamp="20250111_120000",
            total_images=10,
            total_prompts=10,
            batch_size=1,
            workflow_path="workflow.json",
            duration_s=35.5,
            images=[img_metadata],
            statistics={"throughput": 16.9},
        )

        assert batch_metadata.batch_id == "batch_123"
        assert batch_metadata.total_images == 10
        assert batch_metadata.duration_s == 35.5
        assert len(batch_metadata.images) == 1


class TestOutputManager:
    """Test OutputManager functionality"""

    @pytest.fixture
    def temp_dir(self):
        """Create a temporary directory"""
        with tempfile.TemporaryDirectory() as tmpdir:
            yield Path(tmpdir)

    @pytest.fixture
    def manager(self, temp_dir):
        """Create an OutputManager instance"""
        return OutputManager(base_output_dir=temp_dir / "batches")

    def test_manager_initialization(self, temp_dir):
        """Test OutputManager initialization"""
        manager = OutputManager(base_output_dir=temp_dir / "outputs")

        assert manager.base_output_dir == temp_dir / "outputs"
        assert manager.base_output_dir.exists()

    def test_create_batch_directory(self, manager):
        """Test creating batch directory"""
        batch_dir = manager.create_batch_directory("test_batch_123")

        assert batch_dir.exists()
        assert (batch_dir / "images").exists()
        assert (batch_dir / "metadata").exists()

    def test_create_batch_directory_custom_timestamp(self, manager):
        """Test creating batch directory with custom timestamp"""
        batch_dir = manager.create_batch_directory(
            "test_batch_456",
            timestamp="20250111_123456",
        )

        assert "20250111_123456" in str(batch_dir)
        assert "test_bat" in str(batch_dir)

    def test_save_image(self, manager, temp_dir):
        """Test saving image to batch directory"""
        # Create a dummy image file
        source_image = temp_dir / "source.png"
        source_image.write_text("fake image data")

        # Create batch directory
        batch_dir = manager.create_batch_directory("test_batch")

        # Save image
        saved_path = manager.save_image(
            source_path=source_image,
            batch_dir=batch_dir,
            index=1,
            seed=42,
        )

        assert saved_path.exists()
        assert saved_path.parent == batch_dir / "images"
        assert "0001" in saved_path.name
        assert "42" in saved_path.name

    def test_save_image_custom_name(self, manager, temp_dir):
        """Test saving image with custom name"""
        source_image = temp_dir / "source.png"
        source_image.write_text("fake image data")

        batch_dir = manager.create_batch_directory("test_batch")

        saved_path = manager.save_image(
            source_path=source_image,
            batch_dir=batch_dir,
            index=1,
            custom_name="custom_sprite.png",
        )

        assert saved_path.name == "custom_sprite.png"

    def test_save_image_metadata(self, manager):
        """Test saving image metadata"""
        batch_dir = manager.create_batch_directory("test_batch")

        metadata = ImageMetadata(
            filename="sprite_001.png",
            prompt="pixel art warrior",
            seed=42,
            generation_time_s=3.5,
        )

        metadata_path = manager.save_image_metadata(batch_dir, metadata)

        assert metadata_path.exists()
        assert metadata_path.parent == batch_dir / "metadata"

        # Load and verify
        with open(metadata_path) as f:
            loaded = json.load(f)

        assert loaded["filename"] == "sprite_001.png"
        assert loaded["prompt"] == "pixel art warrior"
        assert loaded["seed"] == 42

    def test_save_batch_metadata(self, manager):
        """Test saving batch metadata"""
        batch_dir = manager.create_batch_directory("test_batch")

        img_metadata = ImageMetadata(
            filename="sprite_001.png",
            prompt="test",
        )

        batch_metadata = BatchMetadata(
            batch_id="test_123",
            timestamp="20250111_120000",
            total_images=10,
            total_prompts=10,
            batch_size=1,
            workflow_path="workflow.json",
            duration_s=35.5,
            images=[img_metadata],
            statistics={"throughput": 16.9},
        )

        metadata_path = manager.save_batch_metadata(batch_dir, batch_metadata)

        assert metadata_path.exists()
        assert metadata_path.name == "batch_info.json"

        # Load and verify
        with open(metadata_path) as f:
            loaded = json.load(f)

        assert loaded["batch_id"] == "test_123"
        assert loaded["total_images"] == 10
        assert loaded["duration_s"] == 35.5

    def test_load_batch_metadata(self, manager):
        """Test loading batch metadata"""
        batch_dir = manager.create_batch_directory("test_batch")

        # Save metadata first
        img_metadata = ImageMetadata(filename="test.png", prompt="test")
        batch_metadata = BatchMetadata(
            batch_id="test_123",
            timestamp="20250111_120000",
            total_images=5,
            total_prompts=5,
            batch_size=1,
            workflow_path="workflow.json",
            duration_s=20.0,
            images=[img_metadata],
            statistics={},
        )

        manager.save_batch_metadata(batch_dir, batch_metadata)

        # Load it back
        loaded = manager.load_batch_metadata(batch_dir)

        assert loaded is not None
        assert loaded.batch_id == "test_123"
        assert loaded.total_images == 5
        assert len(loaded.images) == 1
        assert isinstance(loaded.images[0], ImageMetadata)

    def test_load_batch_metadata_nonexistent(self, manager, temp_dir):
        """Test loading metadata from directory without batch_info.json"""
        batch_dir = temp_dir / "empty_batch"
        batch_dir.mkdir()

        loaded = manager.load_batch_metadata(batch_dir)
        assert loaded is None

    def test_list_batches(self, manager):
        """Test listing batch directories"""
        import uuid
        # Create multiple batches with unique IDs and timestamps
        manager.create_batch_directory(str(uuid.uuid4()), timestamp="20250111_120000")
        manager.create_batch_directory(str(uuid.uuid4()), timestamp="20250111_120001")
        manager.create_batch_directory(str(uuid.uuid4()), timestamp="20250111_120002")

        batches = manager.list_batches()

        assert len(batches) == 3

    def test_list_batches_with_limit(self, manager):
        """Test listing batches with limit"""
        import uuid
        for i in range(5):
            manager.create_batch_directory(str(uuid.uuid4()), timestamp=f"2025011112{i:04d}")

        batches = manager.list_batches(limit=3)

        assert len(batches) == 3

    def test_get_batch_statistics(self, manager, temp_dir):
        """Test calculating batch statistics"""
        batch_dir = manager.create_batch_directory("test_batch")

        # Create some fake images
        images_dir = batch_dir / "images"
        for i in range(3):
            (images_dir / f"sprite_{i:03d}.png").write_text("fake image data")

        # Create batch info
        batch_info = {
            "duration_s": 30.0,
        }
        with open(batch_dir / "batch_info.json", "w") as f:
            json.dump(batch_info, f)

        stats = manager.get_batch_statistics(batch_dir)

        assert stats["image_count"] == 3
        assert stats["duration_s"] == 30.0
        assert stats["throughput_images_per_min"] is not None

    def test_aggregate_batches(self, manager, temp_dir):
        """Test aggregating multiple batches"""
        # Create multiple batch directories with metadata
        batch_dirs = []

        for i in range(3):
            batch_dir = manager.create_batch_directory(f"batch_{i:03d}")
            batch_dirs.append(batch_dir)

            # Create fake images
            images_dir = batch_dir / "images"
            for j in range(2):
                (images_dir / f"sprite_{j:03d}.png").write_text("fake")

            # Create batch info
            batch_info = {"duration_s": 10.0 * (i + 1)}
            with open(batch_dir / "batch_info.json", "w") as f:
                json.dump(batch_info, f)

        # Aggregate
        output_path = temp_dir / "aggregated_report.json"
        manager.aggregate_batches(batch_dirs, output_path)

        assert output_path.exists()

        # Verify report
        with open(output_path) as f:
            report = json.load(f)

        assert report["total_batches"] == 3
        assert len(report["batches"]) == 3
        assert "summary" in report
        assert report["summary"]["total_images"] == 6

    def test_cleanup_old_batches_dry_run(self, manager):
        """Test cleanup dry run"""
        # Create a batch
        batch_dir = manager.create_batch_directory("old_batch")

        # Run dry-run cleanup (should not delete anything)
        deleted = manager.cleanup_old_batches(max_age_days=0, dry_run=True)

        # Batch should still exist
        assert batch_dir.exists()

        # But it should be in the list of what would be deleted
        assert len(deleted) == 1

    def test_cleanup_old_batches_real(self, manager):
        """Test real cleanup"""
        # Create a batch
        batch_dir = manager.create_batch_directory("old_batch")

        # Run real cleanup
        deleted = manager.cleanup_old_batches(max_age_days=0, dry_run=False)

        # Batch should be deleted
        assert not batch_dir.exists()
        assert len(deleted) == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
