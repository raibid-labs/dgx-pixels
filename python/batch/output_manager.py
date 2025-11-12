#!/usr/bin/env python3
"""
Output Management for Batch Processing

Handles:
- Organized output directory structure
- Filename conventions
- Metadata JSON generation
- Batch result aggregation
- Image post-processing hooks
"""

import json
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from datetime import datetime


@dataclass
class ImageMetadata:
    """Metadata for a single generated image"""
    filename: str
    prompt: str
    negative_prompt: Optional[str] = None
    seed: Optional[int] = None
    steps: int = 20
    cfg_scale: float = 8.0
    model: Optional[str] = None
    lora: Optional[str] = None
    width: int = 1024
    height: int = 1024
    batch_index: int = 0
    generation_time_s: Optional[float] = None
    comfyui_prompt_id: Optional[str] = None


@dataclass
class BatchMetadata:
    """Metadata for an entire batch"""
    batch_id: str
    timestamp: str
    total_images: int
    total_prompts: int
    batch_size: int
    workflow_path: str
    duration_s: float
    images: List[ImageMetadata]
    statistics: Dict[str, Any]


class OutputManager:
    """
    Manages batch generation outputs

    Provides:
    - Standardized directory structure
    - Consistent filename conventions
    - Metadata JSON generation
    - Batch result aggregation
    """

    def __init__(
        self,
        base_output_dir: Path = Path("outputs/batches"),
        filename_template: str = "sprite_{index:04d}_{seed}",
    ):
        """
        Initialize output manager

        Args:
            base_output_dir: Base directory for all batch outputs
            filename_template: Template for image filenames
        """
        self.base_output_dir = Path(base_output_dir)
        self.base_output_dir.mkdir(parents=True, exist_ok=True)
        self.filename_template = filename_template

    def create_batch_directory(
        self,
        batch_id: str,
        timestamp: Optional[str] = None,
    ) -> Path:
        """
        Create a new batch output directory

        Structure:
            outputs/batches/batch_YYYYMMDD_HHMMSS_<batch_id>/
                images/
                metadata/
                batch_info.json

        Args:
            batch_id: Unique batch identifier
            timestamp: Optional timestamp string (auto-generated if None)

        Returns:
            Path to batch directory
        """
        if timestamp is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        batch_dir_name = f"batch_{timestamp}_{batch_id[:8]}"
        batch_dir = self.base_output_dir / batch_dir_name

        # Create subdirectories
        batch_dir.mkdir(parents=True, exist_ok=True)
        (batch_dir / "images").mkdir(exist_ok=True)
        (batch_dir / "metadata").mkdir(exist_ok=True)

        return batch_dir

    def save_image(
        self,
        source_path: Path,
        batch_dir: Path,
        index: int,
        seed: Optional[int] = None,
        custom_name: Optional[str] = None,
    ) -> Path:
        """
        Copy/move image to batch output directory with standardized naming

        Args:
            source_path: Original image path
            batch_dir: Batch output directory
            index: Image index in batch
            seed: Generation seed (for filename)
            custom_name: Optional custom filename (overrides template)

        Returns:
            Path to saved image
        """
        images_dir = batch_dir / "images"

        # Generate filename
        if custom_name:
            filename = custom_name
        else:
            if seed is not None:
                filename = self.filename_template.format(index=index, seed=seed)
            else:
                filename = self.filename_template.format(index=index, seed="unknown")

        # Preserve extension
        extension = source_path.suffix
        if not filename.endswith(extension):
            filename += extension

        # Copy image
        dest_path = images_dir / filename
        shutil.copy2(source_path, dest_path)

        return dest_path

    def save_image_metadata(
        self,
        batch_dir: Path,
        metadata: ImageMetadata,
    ) -> Path:
        """
        Save metadata for a single image

        Args:
            batch_dir: Batch output directory
            metadata: Image metadata

        Returns:
            Path to metadata JSON file
        """
        metadata_dir = batch_dir / "metadata"
        metadata_file = metadata_dir / f"{Path(metadata.filename).stem}.json"

        with open(metadata_file, "w") as f:
            json.dump(asdict(metadata), f, indent=2)

        return metadata_file

    def save_batch_metadata(
        self,
        batch_dir: Path,
        metadata: BatchMetadata,
    ) -> Path:
        """
        Save metadata for entire batch

        Args:
            batch_dir: Batch output directory
            metadata: Batch metadata

        Returns:
            Path to batch_info.json
        """
        batch_info_file = batch_dir / "batch_info.json"

        with open(batch_info_file, "w") as f:
            json.dump(asdict(metadata), f, indent=2)

        return batch_info_file

    def load_batch_metadata(self, batch_dir: Path) -> Optional[BatchMetadata]:
        """Load batch metadata from directory"""
        batch_info_file = batch_dir / "batch_info.json"

        if not batch_info_file.exists():
            return None

        with open(batch_info_file) as f:
            data = json.load(f)

        # Convert images list back to ImageMetadata objects
        images = [ImageMetadata(**img) for img in data.get("images", [])]
        data["images"] = images

        return BatchMetadata(**data)

    def list_batches(
        self,
        limit: Optional[int] = None,
        sort_by_date: bool = True,
    ) -> List[Path]:
        """
        List all batch directories

        Args:
            limit: Maximum number of batches to return
            sort_by_date: Sort by date (newest first)

        Returns:
            List of batch directory paths
        """
        batches = []

        for item in self.base_output_dir.iterdir():
            if item.is_dir() and item.name.startswith("batch_"):
                batches.append(item)

        if sort_by_date:
            batches.sort(key=lambda p: p.stat().st_mtime, reverse=True)

        if limit:
            batches = batches[:limit]

        return batches

    def get_batch_statistics(self, batch_dir: Path) -> Dict[str, Any]:
        """
        Calculate statistics for a batch

        Args:
            batch_dir: Batch directory

        Returns:
            Statistics dictionary
        """
        images_dir = batch_dir / "images"
        metadata_dir = batch_dir / "metadata"

        # Count files
        image_count = len(list(images_dir.glob("*.png"))) + len(list(images_dir.glob("*.jpg")))
        metadata_count = len(list(metadata_dir.glob("*.json")))

        # Calculate total size
        total_size_bytes = sum(f.stat().st_size for f in images_dir.iterdir() if f.is_file())
        total_size_mb = total_size_bytes / (1024 * 1024)

        # Load batch info if available
        batch_info = {}
        batch_info_file = batch_dir / "batch_info.json"
        if batch_info_file.exists():
            with open(batch_info_file) as f:
                batch_info = json.load(f)

        return {
            "batch_dir": str(batch_dir),
            "image_count": image_count,
            "metadata_count": metadata_count,
            "total_size_mb": round(total_size_mb, 2),
            "duration_s": batch_info.get("duration_s"),
            "throughput_images_per_min": (
                round(image_count / (batch_info["duration_s"] / 60), 2)
                if batch_info.get("duration_s")
                else None
            ),
        }

    def aggregate_batches(
        self,
        batch_dirs: List[Path],
        output_path: Path,
    ) -> None:
        """
        Aggregate multiple batches into a summary report

        Args:
            batch_dirs: List of batch directories
            output_path: Where to save the aggregated report
        """
        aggregated_data = {
            "generated_at": datetime.now().isoformat(),
            "total_batches": len(batch_dirs),
            "batches": [],
        }

        total_images = 0
        total_duration = 0.0

        for batch_dir in batch_dirs:
            stats = self.get_batch_statistics(batch_dir)
            aggregated_data["batches"].append(stats)

            total_images += stats["image_count"]
            if stats.get("duration_s"):
                total_duration += stats["duration_s"]

        # Summary statistics
        aggregated_data["summary"] = {
            "total_images": total_images,
            "total_duration_s": round(total_duration, 2),
            "avg_throughput_images_per_min": (
                round(total_images / (total_duration / 60), 2)
                if total_duration > 0
                else None
            ),
        }

        # Save report
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w") as f:
            json.dump(aggregated_data, f, indent=2)

        print(f"[OUTPUT] Aggregated report saved to {output_path}")

    def cleanup_old_batches(
        self,
        max_age_days: int = 30,
        dry_run: bool = True,
    ) -> List[Path]:
        """
        Clean up old batch directories

        Args:
            max_age_days: Maximum age in days
            dry_run: If True, only list batches that would be deleted

        Returns:
            List of deleted (or would-be deleted) batch directories
        """
        import time

        current_time = time.time()
        max_age_seconds = max_age_days * 24 * 3600

        deleted = []

        for batch_dir in self.list_batches(sort_by_date=False):
            age_seconds = current_time - batch_dir.stat().st_mtime

            if age_seconds > max_age_seconds:
                if dry_run:
                    print(f"[CLEANUP] Would delete: {batch_dir.name} (age: {age_seconds / 86400:.1f} days)")
                else:
                    print(f"[CLEANUP] Deleting: {batch_dir.name}")
                    shutil.rmtree(batch_dir)

                deleted.append(batch_dir)

        return deleted


if __name__ == "__main__":
    # Self-test
    print("=== Output Manager Self-Test ===\n")

    # Create temporary test directory
    import tempfile

    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)
        manager = OutputManager(base_output_dir=tmpdir / "batches")

        # Test 1: Create batch directory
        print("Test 1: Create batch directory...")
        batch_dir = manager.create_batch_directory("test123")
        assert (batch_dir / "images").exists()
        assert (batch_dir / "metadata").exists()
        print(f"✅ Created: {batch_dir}")

        # Test 2: Save metadata
        print("\nTest 2: Save metadata...")
        img_metadata = ImageMetadata(
            filename="sprite_0001.png",
            prompt="pixel art warrior",
            seed=42,
            generation_time_s=3.5,
        )
        metadata_file = manager.save_image_metadata(batch_dir, img_metadata)
        assert metadata_file.exists()
        print(f"✅ Saved: {metadata_file}")

        # Test 3: Batch metadata
        print("\nTest 3: Save batch metadata...")
        batch_metadata = BatchMetadata(
            batch_id="test123",
            timestamp="20250111_120000",
            total_images=10,
            total_prompts=10,
            batch_size=1,
            workflow_path="workflows/batch_optimized.json",
            duration_s=35.5,
            images=[img_metadata],
            statistics={"throughput": 16.9},
        )
        batch_file = manager.save_batch_metadata(batch_dir, batch_metadata)
        assert batch_file.exists()
        print(f"✅ Saved: {batch_file}")

        # Test 4: Load metadata
        print("\nTest 4: Load batch metadata...")
        loaded = manager.load_batch_metadata(batch_dir)
        assert loaded is not None
        assert loaded.batch_id == "test123"
        print(f"✅ Loaded batch: {loaded.batch_id}")

        # Test 5: Statistics
        print("\nTest 5: Calculate statistics...")
        stats = manager.get_batch_statistics(batch_dir)
        print(f"✅ Statistics: {json.dumps(stats, indent=2)}")

        # Test 6: List batches
        print("\nTest 6: List batches...")
        batches = manager.list_batches()
        print(f"✅ Found {len(batches)} batches")

    print("\n✅ Self-test complete")
