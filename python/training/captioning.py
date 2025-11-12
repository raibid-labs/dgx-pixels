#!/usr/bin/env python3
"""
Auto-Captioning for Training Images

Uses BLIP-2 for automatic image captioning to generate training captions
for images without manual annotations.

Features:
- BLIP-2 image captioning
- Pixel art specific prompt engineering
- Batch processing for efficiency
- Caption quality validation
- Manual refinement workflow
"""

import torch
from PIL import Image
from pathlib import Path
from typing import List, Optional, Dict, Union
import logging
from dataclasses import dataclass
from tqdm import tqdm
import json

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class CaptionConfig:
    """Configuration for auto-captioning"""
    model_name: str = "Salesforce/blip2-opt-2.7b"
    batch_size: int = 4
    max_length: int = 50
    min_length: int = 10
    num_beams: int = 3
    prefix: str = "pixel art, "
    suffix: str = ", game sprite, 16-bit style"
    device: str = "cuda" if torch.cuda.is_available() else "cpu"


class AutoCaptioner:
    """Automatic image captioning using BLIP-2"""

    def __init__(self, config: Optional[CaptionConfig] = None):
        self.config = config or CaptionConfig()
        self.processor = None
        self.model = None

        logger.info(f"Initializing AutoCaptioner with model: {self.config.model_name}")

    def load_model(self):
        """Load BLIP-2 model and processor"""
        if self.model is not None:
            logger.info("Model already loaded")
            return

        try:
            from transformers import Blip2Processor, Blip2ForConditionalGeneration

            logger.info(f"Loading model: {self.config.model_name}")

            self.processor = Blip2Processor.from_pretrained(self.config.model_name)
            self.model = Blip2ForConditionalGeneration.from_pretrained(
                self.config.model_name,
                torch_dtype=torch.float16 if self.config.device == "cuda" else torch.float32,
            )
            self.model.to(self.config.device)
            self.model.eval()

            logger.info("Model loaded successfully")

        except ImportError:
            logger.error(
                "transformers library not installed. "
                "Install with: pip install transformers"
            )
            raise
        except Exception as e:
            logger.error(f"Error loading model: {e}")
            raise

    def caption_image(self, image: Union[Image.Image, Path, str]) -> str:
        """
        Generate caption for a single image

        Args:
            image: PIL Image or path to image file

        Returns:
            Generated caption string
        """
        if self.model is None:
            self.load_model()

        # Load image if path provided
        if isinstance(image, (Path, str)):
            image = Image.open(image).convert('RGB')

        # Process image
        inputs = self.processor(
            images=image,
            return_tensors="pt"
        ).to(self.config.device, torch.float16 if self.config.device == "cuda" else torch.float32)

        # Generate caption
        with torch.no_grad():
            generated_ids = self.model.generate(
                **inputs,
                max_length=self.config.max_length,
                min_length=self.config.min_length,
                num_beams=self.config.num_beams,
            )

        # Decode caption
        caption = self.processor.batch_decode(
            generated_ids,
            skip_special_tokens=True
        )[0].strip()

        # Add pixel art context
        full_caption = f"{self.config.prefix}{caption}{self.config.suffix}"

        return full_caption

    def caption_dataset(
        self,
        dataset_dir: Union[str, Path],
        overwrite: bool = False,
        interactive: bool = False,
    ) -> Dict[str, int]:
        """
        Caption all images in a dataset directory

        Args:
            dataset_dir: Directory containing images
            overwrite: Overwrite existing caption files
            interactive: Allow manual editing of captions

        Returns:
            Statistics about captioning process
        """
        dataset_dir = Path(dataset_dir)

        # Find all images without captions
        image_extensions = ['*.png', '*.jpg', '*.jpeg', '*.PNG', '*.JPG', '*.JPEG']
        all_images = []
        for ext in image_extensions:
            all_images.extend(list(dataset_dir.glob(ext)))

        # Filter images that need captions
        images_to_caption = []
        for image_path in all_images:
            caption_path = image_path.with_suffix('.txt')
            if overwrite or not caption_path.exists():
                images_to_caption.append(image_path)

        if len(images_to_caption) == 0:
            logger.info("All images already have captions")
            return {'total': len(all_images), 'captioned': 0, 'skipped': len(all_images)}

        logger.info(f"Found {len(images_to_caption)} images to caption")

        # Load model
        self.load_model()

        # Caption images
        stats = {'total': len(all_images), 'captioned': 0, 'skipped': 0, 'failed': 0}

        progress_bar = tqdm(images_to_caption, desc="Captioning images")

        for image_path in progress_bar:
            try:
                # Generate caption
                caption = self.caption_image(image_path)

                # Interactive editing
                if interactive:
                    print(f"\nImage: {image_path.name}")
                    print(f"Generated caption: {caption}")
                    user_input = input("Edit caption (press Enter to accept): ").strip()
                    if user_input:
                        caption = user_input

                # Save caption
                caption_path = image_path.with_suffix('.txt')
                with open(caption_path, 'w') as f:
                    f.write(caption)

                stats['captioned'] += 1

            except Exception as e:
                logger.error(f"Error captioning {image_path.name}: {e}")
                stats['failed'] += 1
                continue

        progress_bar.close()

        # Calculate skipped
        stats['skipped'] = stats['total'] - stats['captioned'] - stats['failed']

        # Print summary
        logger.info("\n" + "="*60)
        logger.info("CAPTIONING SUMMARY")
        logger.info("="*60)
        logger.info(f"Total images: {stats['total']}")
        logger.info(f"Newly captioned: {stats['captioned']}")
        logger.info(f"Already had captions: {stats['skipped']}")
        logger.info(f"Failed: {stats['failed']}")
        logger.info("="*60)

        return stats

    def validate_captions(
        self,
        dataset_dir: Union[str, Path],
        min_length: int = 20,
        max_length: int = 200,
    ) -> Dict[str, List[str]]:
        """
        Validate captions in a dataset

        Args:
            dataset_dir: Directory containing images and captions
            min_length: Minimum caption length
            max_length: Maximum caption length

        Returns:
            Dictionary with validation issues
        """
        dataset_dir = Path(dataset_dir)

        issues = {
            'missing': [],
            'too_short': [],
            'too_long': [],
            'empty': [],
        }

        # Find all images
        image_extensions = ['*.png', '*.jpg', '*.jpeg', '*.PNG', '*.JPG', '*.JPEG']
        all_images = []
        for ext in image_extensions:
            all_images.extend(list(dataset_dir.glob(ext)))

        logger.info(f"Validating captions for {len(all_images)} images")

        for image_path in all_images:
            caption_path = image_path.with_suffix('.txt')

            # Check if caption exists
            if not caption_path.exists():
                issues['missing'].append(image_path.name)
                continue

            # Read caption
            with open(caption_path, 'r') as f:
                caption = f.read().strip()

            # Check caption length
            if len(caption) == 0:
                issues['empty'].append(image_path.name)
            elif len(caption) < min_length:
                issues['too_short'].append(image_path.name)
            elif len(caption) > max_length:
                issues['too_long'].append(image_path.name)

        # Print summary
        total_issues = sum(len(v) for v in issues.values())

        logger.info("\n" + "="*60)
        logger.info("CAPTION VALIDATION")
        logger.info("="*60)
        logger.info(f"Total images: {len(all_images)}")
        logger.info(f"Missing captions: {len(issues['missing'])}")
        logger.info(f"Empty captions: {len(issues['empty'])}")
        logger.info(f"Too short: {len(issues['too_short'])} (< {min_length} chars)")
        logger.info(f"Too long: {len(issues['too_long'])} (> {max_length} chars)")
        logger.info(f"Total issues: {total_issues}")
        logger.info("="*60)

        if total_issues > 0:
            logger.warning(f"Found {total_issues} caption issues that should be addressed")
        else:
            logger.info("All captions validated successfully!")

        return issues


class ManualCaptionEditor:
    """Interactive tool for manual caption editing"""

    def __init__(self, dataset_dir: Union[str, Path]):
        self.dataset_dir = Path(dataset_dir)

        # Find all images
        image_extensions = ['*.png', '*.jpg', '*.jpeg', '*.PNG', '*.JPG', '*.JPEG']
        self.images = []
        for ext in image_extensions:
            self.images.extend(list(self.dataset_dir.glob(ext)))

        self.images.sort()
        self.current_index = 0

        logger.info(f"Loaded {len(self.images)} images for editing")

    def edit_captions(self):
        """Interactive caption editing session"""
        print("\n" + "="*60)
        print("MANUAL CAPTION EDITOR")
        print("="*60)
        print("Commands:")
        print("  <caption text> - Set caption")
        print("  n - Next image")
        print("  p - Previous image")
        print("  s - Skip (keep existing caption)")
        print("  q - Quit")
        print("="*60 + "\n")

        while self.current_index < len(self.images):
            image_path = self.images[self.current_index]
            caption_path = image_path.with_suffix('.txt')

            # Display current image info
            print(f"\n[{self.current_index + 1}/{len(self.images)}] {image_path.name}")

            # Load existing caption
            if caption_path.exists():
                with open(caption_path, 'r') as f:
                    current_caption = f.read().strip()
                print(f"Current caption: {current_caption}")
            else:
                current_caption = ""
                print("No caption yet")

            # Get user input
            user_input = input("\nEnter new caption (or command): ").strip()

            # Handle commands
            if user_input.lower() == 'q':
                print("Quitting...")
                break
            elif user_input.lower() == 'n':
                self.current_index += 1
                continue
            elif user_input.lower() == 'p':
                self.current_index = max(0, self.current_index - 1)
                continue
            elif user_input.lower() == 's':
                self.current_index += 1
                continue
            elif user_input:
                # Save new caption
                with open(caption_path, 'w') as f:
                    f.write(user_input)
                print(f"Saved caption: {user_input}")
                self.current_index += 1

        print(f"\nEdited captions for {self.current_index} images")


def main():
    """CLI interface for auto-captioning"""
    import argparse

    parser = argparse.ArgumentParser(description="Auto-caption training images")
    parser.add_argument("--dataset", type=str, required=True, help="Dataset directory")
    parser.add_argument("--mode", type=str, choices=['caption', 'validate', 'edit'], default='caption',
                        help="Operation mode")
    parser.add_argument("--model", type=str, default="Salesforce/blip2-opt-2.7b", help="BLIP-2 model to use")
    parser.add_argument("--overwrite", action="store_true", help="Overwrite existing captions")
    parser.add_argument("--interactive", action="store_true", help="Interactive caption editing")
    parser.add_argument("--prefix", type=str, default="pixel art, ", help="Caption prefix")
    parser.add_argument("--suffix", type=str, default=", game sprite, 16-bit style", help="Caption suffix")

    args = parser.parse_args()

    if args.mode == 'caption':
        # Auto-caption mode
        config = CaptionConfig(
            model_name=args.model,
            prefix=args.prefix,
            suffix=args.suffix,
        )
        captioner = AutoCaptioner(config)
        stats = captioner.caption_dataset(
            dataset_dir=args.dataset,
            overwrite=args.overwrite,
            interactive=args.interactive,
        )

        logger.info(f"Captioning complete! Captioned {stats['captioned']} images")

    elif args.mode == 'validate':
        # Validation mode
        captioner = AutoCaptioner()
        issues = captioner.validate_captions(dataset_dir=args.dataset)

        # Exit with error code if issues found
        total_issues = sum(len(v) for v in issues.values())
        exit(0 if total_issues == 0 else 1)

    elif args.mode == 'edit':
        # Manual editing mode
        editor = ManualCaptionEditor(dataset_dir=args.dataset)
        editor.edit_captions()


if __name__ == "__main__":
    main()
