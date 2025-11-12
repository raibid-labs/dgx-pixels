#!/usr/bin/env python3
"""
Dataset Preparation Utilities for LoRA Training

Handles:
- Image preprocessing and validation
- Dataset organization and structure
- Image resizing and formatting for SDXL (1024x1024)
- Quality checks and validation
- Dataset statistics and reporting
"""

import torch
from PIL import Image
import numpy as np
from pathlib import Path
from typing import List, Tuple, Optional, Dict, Any
import json
import logging
from dataclasses import dataclass, asdict
from tqdm import tqdm
import hashlib

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class ImageMetadata:
    """Metadata for a training image"""
    filename: str
    original_size: Tuple[int, int]
    processed_size: Tuple[int, int]
    format: str
    file_size_bytes: int
    has_caption: bool
    caption_length: Optional[int]
    hash: str


@dataclass
class DatasetStats:
    """Statistics about a training dataset"""
    total_images: int
    images_with_captions: int
    images_without_captions: int
    avg_caption_length: float
    min_resolution: Tuple[int, int]
    max_resolution: Tuple[int, int]
    total_size_mb: float
    formats: Dict[str, int]

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


class DatasetPreparator:
    """Prepare and validate datasets for LoRA training"""

    def __init__(
        self,
        target_resolution: int = 1024,
        supported_formats: List[str] = None,
        quality_threshold: int = 512,
    ):
        self.target_resolution = target_resolution
        self.supported_formats = supported_formats or ['png', 'jpg', 'jpeg', 'PNG', 'JPG', 'JPEG']
        self.quality_threshold = quality_threshold

        logger.info(f"Dataset preparator initialized: target_resolution={target_resolution}")

    def prepare_dataset(
        self,
        input_dir: Path,
        output_dir: Path,
        create_captions: bool = False,
        pad_to_square: bool = True,
        validate_only: bool = False,
    ) -> DatasetStats:
        """
        Prepare training dataset from raw images

        Args:
            input_dir: Directory containing raw images
            output_dir: Directory for processed images
            create_captions: Create default captions for images without captions
            pad_to_square: Pad images to square before resizing
            validate_only: Only validate, don't process

        Returns:
            DatasetStats object with dataset statistics
        """
        input_dir = Path(input_dir)
        output_dir = Path(output_dir)

        if not validate_only:
            output_dir.mkdir(parents=True, exist_ok=True)

        # Find all images
        image_files = self._find_images(input_dir)
        logger.info(f"Found {len(image_files)} images in {input_dir}")

        if len(image_files) == 0:
            raise ValueError(f"No images found in {input_dir}")

        # Check minimum dataset size
        if len(image_files) < 20:
            logger.warning(
                f"Dataset has only {len(image_files)} images. "
                f"Recommended minimum is 50 images for good LoRA training."
            )

        # Process images
        metadata_list = []
        progress_bar = tqdm(image_files, desc="Processing images")

        for image_path in progress_bar:
            try:
                metadata = self._process_image(
                    image_path=image_path,
                    output_dir=output_dir,
                    pad_to_square=pad_to_square,
                    create_caption=create_captions,
                    validate_only=validate_only,
                )
                metadata_list.append(metadata)
            except Exception as e:
                logger.error(f"Error processing {image_path}: {e}")
                continue

        # Calculate statistics
        stats = self._calculate_stats(metadata_list)

        # Save metadata
        if not validate_only:
            self._save_metadata(output_dir, metadata_list, stats)

        # Print summary
        self._print_summary(stats)

        return stats

    def _find_images(self, directory: Path) -> List[Path]:
        """Find all supported image files in directory"""
        image_files = []
        for fmt in self.supported_formats:
            image_files.extend(list(directory.glob(f"*.{fmt}")))

        # Remove duplicates and sort
        image_files = sorted(set(image_files))

        return image_files

    def _process_image(
        self,
        image_path: Path,
        output_dir: Path,
        pad_to_square: bool,
        create_caption: bool,
        validate_only: bool,
    ) -> ImageMetadata:
        """Process a single image"""
        # Load image
        image = Image.open(image_path)
        original_size = image.size
        original_format = image.format or 'PNG'

        # Convert to RGB if needed
        if image.mode not in ['RGB', 'RGBA']:
            image = image.convert('RGB')

        # Check quality threshold
        if min(image.size) < self.quality_threshold:
            logger.warning(
                f"{image_path.name}: Resolution {image.size} is below quality threshold "
                f"{self.quality_threshold}. May result in poor training quality."
            )

        # Process image
        if not validate_only:
            processed_image = self._resize_image(image, pad_to_square)

            # Save processed image
            output_path = output_dir / f"{image_path.stem}.png"
            processed_image.save(output_path, 'PNG', optimize=True)
            processed_size = processed_image.size
        else:
            processed_size = self._calculate_output_size(image.size, pad_to_square)

        # Check for caption
        caption_path = image_path.with_suffix('.txt')
        has_caption = caption_path.exists()
        caption_length = None

        if has_caption:
            with open(caption_path, 'r') as f:
                caption = f.read().strip()
                caption_length = len(caption)
        elif create_caption and not validate_only:
            # Create default caption
            caption = "pixel art, game sprite, 16-bit style"
            output_caption_path = output_dir / f"{image_path.stem}.txt"
            with open(output_caption_path, 'w') as f:
                f.write(caption)
            has_caption = True
            caption_length = len(caption)
            logger.info(f"Created default caption for {image_path.name}")

        # Copy caption if it exists
        if has_caption and not validate_only and caption_path.exists():
            output_caption_path = output_dir / f"{image_path.stem}.txt"
            with open(caption_path, 'r') as f:
                caption_content = f.read()
            with open(output_caption_path, 'w') as f:
                f.write(caption_content)

        # Calculate file hash
        file_hash = self._calculate_hash(image_path)

        return ImageMetadata(
            filename=image_path.name,
            original_size=original_size,
            processed_size=processed_size,
            format=original_format,
            file_size_bytes=image_path.stat().st_size,
            has_caption=has_caption,
            caption_length=caption_length,
            hash=file_hash,
        )

    def _resize_image(self, image: Image.Image, pad_to_square: bool) -> Image.Image:
        """Resize image to target resolution"""
        if pad_to_square:
            # Pad to square first
            max_dim = max(image.size)
            canvas = Image.new('RGB', (max_dim, max_dim), (0, 0, 0))

            # Center the image
            offset = ((max_dim - image.size[0]) // 2, (max_dim - image.size[1]) // 2)

            # Handle RGBA images
            if image.mode == 'RGBA':
                # Create white background for transparent areas
                background = Image.new('RGB', (max_dim, max_dim), (255, 255, 255))
                background.paste(image, offset, image if image.mode == 'RGBA' else None)
                canvas = background
            else:
                canvas.paste(image, offset)

            # Resize to target resolution
            resized = canvas.resize(
                (self.target_resolution, self.target_resolution),
                Image.LANCZOS
            )
        else:
            # Direct resize
            resized = image.resize(
                (self.target_resolution, self.target_resolution),
                Image.LANCZOS
            )

        return resized

    def _calculate_output_size(self, input_size: Tuple[int, int], pad_to_square: bool) -> Tuple[int, int]:
        """Calculate what the output size would be without actually processing"""
        return (self.target_resolution, self.target_resolution)

    def _calculate_hash(self, file_path: Path) -> str:
        """Calculate SHA256 hash of file"""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()[:16]

    def _calculate_stats(self, metadata_list: List[ImageMetadata]) -> DatasetStats:
        """Calculate dataset statistics"""
        total_images = len(metadata_list)
        images_with_captions = sum(1 for m in metadata_list if m.has_caption)
        images_without_captions = total_images - images_with_captions

        caption_lengths = [m.caption_length for m in metadata_list if m.caption_length is not None]
        avg_caption_length = sum(caption_lengths) / len(caption_lengths) if caption_lengths else 0

        # Find min/max resolutions
        all_sizes = [m.original_size for m in metadata_list]
        min_resolution = tuple(min(sizes) for sizes in zip(*all_sizes))
        max_resolution = tuple(max(sizes) for sizes in zip(*all_sizes))

        # Calculate total size
        total_size_mb = sum(m.file_size_bytes for m in metadata_list) / (1024 * 1024)

        # Count formats
        formats = {}
        for m in metadata_list:
            formats[m.format] = formats.get(m.format, 0) + 1

        return DatasetStats(
            total_images=total_images,
            images_with_captions=images_with_captions,
            images_without_captions=images_without_captions,
            avg_caption_length=avg_caption_length,
            min_resolution=min_resolution,
            max_resolution=max_resolution,
            total_size_mb=total_size_mb,
            formats=formats,
        )

    def _save_metadata(self, output_dir: Path, metadata_list: List[ImageMetadata], stats: DatasetStats):
        """Save dataset metadata and statistics"""
        # Save individual image metadata
        metadata_file = output_dir / "dataset_metadata.json"
        metadata_data = [asdict(m) for m in metadata_list]

        with open(metadata_file, 'w') as f:
            json.dump(metadata_data, f, indent=2)

        logger.info(f"Saved image metadata to {metadata_file}")

        # Save statistics
        stats_file = output_dir / "dataset_stats.json"
        with open(stats_file, 'w') as f:
            json.dump(stats.to_dict(), f, indent=2)

        logger.info(f"Saved dataset statistics to {stats_file}")

    def _print_summary(self, stats: DatasetStats):
        """Print dataset summary"""
        logger.info("\n" + "="*60)
        logger.info("DATASET SUMMARY")
        logger.info("="*60)
        logger.info(f"Total images: {stats.total_images}")
        logger.info(f"Images with captions: {stats.images_with_captions}")
        logger.info(f"Images without captions: {stats.images_without_captions}")
        logger.info(f"Average caption length: {stats.avg_caption_length:.1f} characters")
        logger.info(f"Resolution range: {stats.min_resolution} to {stats.max_resolution}")
        logger.info(f"Total dataset size: {stats.total_size_mb:.2f} MB")
        logger.info(f"Image formats: {stats.formats}")
        logger.info("="*60)

        # Warnings
        if stats.total_images < 50:
            logger.warning(
                f"\nWARNING: Dataset has only {stats.total_images} images. "
                f"Recommended minimum is 50 images for quality LoRA training."
            )

        if stats.images_without_captions > 0:
            logger.warning(
                f"\nWARNING: {stats.images_without_captions} images are missing captions. "
                f"Consider using auto-captioning or creating captions manually."
            )

    def validate_dataset(self, dataset_dir: Path) -> bool:
        """
        Validate that a dataset is ready for training

        Returns:
            True if dataset is valid, False otherwise
        """
        dataset_dir = Path(dataset_dir)

        if not dataset_dir.exists():
            logger.error(f"Dataset directory does not exist: {dataset_dir}")
            return False

        # Find images
        image_files = self._find_images(dataset_dir)

        if len(image_files) == 0:
            logger.error(f"No images found in {dataset_dir}")
            return False

        # Check each image
        issues = []

        for image_path in image_files:
            # Check if caption exists
            caption_path = image_path.with_suffix('.txt')
            if not caption_path.exists():
                issues.append(f"{image_path.name}: Missing caption file")

            # Check image validity
            try:
                image = Image.open(image_path)
                if image.size[0] < self.quality_threshold or image.size[1] < self.quality_threshold:
                    issues.append(
                        f"{image_path.name}: Resolution {image.size} below threshold {self.quality_threshold}"
                    )
            except Exception as e:
                issues.append(f"{image_path.name}: Cannot open image - {e}")

        # Report issues
        if issues:
            logger.warning(f"Found {len(issues)} validation issues:")
            for issue in issues[:10]:  # Show first 10
                logger.warning(f"  - {issue}")
            if len(issues) > 10:
                logger.warning(f"  ... and {len(issues) - 10} more issues")

        # Determine if valid
        is_valid = len(issues) == 0

        if is_valid:
            logger.info(f"Dataset validation passed: {len(image_files)} images ready for training")
        else:
            logger.error(f"Dataset validation failed with {len(issues)} issues")

        return is_valid


def main():
    """CLI interface for dataset preparation"""
    import argparse

    parser = argparse.ArgumentParser(description="Prepare datasets for LoRA training")
    parser.add_argument("--input", type=str, required=True, help="Input directory with raw images")
    parser.add_argument("--output", type=str, required=True, help="Output directory for processed dataset")
    parser.add_argument("--resolution", type=int, default=1024, help="Target resolution (default: 1024)")
    parser.add_argument("--create-captions", action="store_true", help="Create default captions for images without captions")
    parser.add_argument("--validate-only", action="store_true", help="Only validate, don't process")
    parser.add_argument("--no-pad", action="store_true", help="Don't pad to square before resizing")

    args = parser.parse_args()

    preparator = DatasetPreparator(target_resolution=args.resolution)

    if args.validate_only:
        is_valid = preparator.validate_dataset(Path(args.input))
        exit(0 if is_valid else 1)
    else:
        stats = preparator.prepare_dataset(
            input_dir=Path(args.input),
            output_dir=Path(args.output),
            create_captions=args.create_captions,
            pad_to_square=not args.no_pad,
            validate_only=False,
        )

        logger.info(f"\nDataset preparation complete!")
        logger.info(f"Processed {stats.total_images} images")
        logger.info(f"Output directory: {args.output}")


if __name__ == "__main__":
    main()
