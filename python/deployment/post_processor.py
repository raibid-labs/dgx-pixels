"""Post-processing for generated sprites

Provides optional transformations:
- Color quantization (reduce to optimal palette)
- Scaling (with nearest-neighbor for pixel-perfect)
- Cropping (remove empty borders)
- Background removal/replacement
- Format optimization
"""

from pathlib import Path
from typing import Optional, Tuple, List
from dataclasses import dataclass
from PIL import Image
import numpy as np


@dataclass
class ProcessingOptions:
    """Options for post-processing"""

    # Color quantization
    quantize_colors: bool = False
    max_colors: int = 256  # Reduce to N-color palette

    # Scaling
    scale: Optional[float] = None  # Scale factor (e.g., 0.5 for half size)
    target_size: Optional[Tuple[int, int]] = None  # Specific target size

    # Cropping
    auto_crop: bool = False  # Remove empty borders
    crop_box: Optional[Tuple[int, int, int, int]] = None  # (left, top, right, bottom)

    # Background
    remove_background: bool = False
    background_color: Optional[Tuple[int, int, int, int]] = None  # RGBA replacement

    # Optimization
    optimize: bool = True  # PNG optimization
    compression_level: int = 6  # 0-9 (higher = smaller file, slower)


class PostProcessor:
    """Post-processes generated sprites for game use

    Transforms sprites to meet game requirements:
    - Reduce color palette for retro look
    - Scale to target resolution
    - Remove unnecessary borders
    - Optimize file size
    """

    def __init__(self):
        pass

    def process_image(
        self, input_path: Path, output_path: Path, options: ProcessingOptions
    ) -> bool:
        """Process a single image

        Args:
            input_path: Path to input image
            output_path: Path to save processed image
            options: Processing options

        Returns:
            True if processing succeeded

        Raises:
            FileNotFoundError: If input file doesn't exist
            ValueError: If processing fails
        """
        input_path = Path(input_path)
        output_path = Path(output_path)

        if not input_path.exists():
            raise FileNotFoundError(f"Input file not found: {input_path}")

        try:
            with Image.open(input_path) as img:
                # Convert to RGBA for processing
                if img.mode != "RGBA":
                    img = img.convert("RGBA")

                # Apply transformations
                img = self._apply_transformations(img, options)

                # Ensure output directory exists
                output_path.parent.mkdir(parents=True, exist_ok=True)

                # Save with optimization
                save_kwargs = {
                    "format": "PNG",
                    "optimize": options.optimize,
                }

                if options.compression_level:
                    save_kwargs["compress_level"] = options.compression_level

                img.save(output_path, **save_kwargs)

                return True

        except Exception as e:
            raise ValueError(f"Processing failed: {str(e)}")

    def _apply_transformations(
        self, img: Image.Image, options: ProcessingOptions
    ) -> Image.Image:
        """Apply all transformations to image

        Args:
            img: PIL Image
            options: Processing options

        Returns:
            Transformed image
        """
        # Auto-crop empty borders
        if options.auto_crop:
            img = self._auto_crop(img)

        # Manual crop
        if options.crop_box:
            img = img.crop(options.crop_box)

        # Scale
        if options.scale:
            new_size = (
                int(img.width * options.scale),
                int(img.height * options.scale),
            )
            img = img.resize(new_size, Image.NEAREST)  # Nearest for pixel art

        # Target size
        if options.target_size:
            img = img.resize(options.target_size, Image.NEAREST)

        # Remove/replace background
        if options.remove_background:
            img = self._remove_background(img)
        elif options.background_color:
            img = self._replace_background(img, options.background_color)

        # Color quantization
        if options.quantize_colors:
            img = self._quantize_colors(img, options.max_colors)

        return img

    def _auto_crop(self, img: Image.Image) -> Image.Image:
        """Remove empty borders from image

        Args:
            img: PIL Image with RGBA mode

        Returns:
            Cropped image
        """
        # Get alpha channel
        alpha = np.array(img)[:, :, 3]

        # Find non-transparent pixels
        rows = np.any(alpha > 0, axis=1)
        cols = np.any(alpha > 0, axis=0)

        if not rows.any() or not cols.any():
            # Image is completely transparent
            return img

        # Get bounding box
        row_min, row_max = np.where(rows)[0][[0, -1]]
        col_min, col_max = np.where(cols)[0][[0, -1]]

        # Crop
        return img.crop((col_min, row_min, col_max + 1, row_max + 1))

    def _remove_background(self, img: Image.Image) -> Image.Image:
        """Remove background (make transparent)

        Uses simple threshold-based approach.
        For more advanced removal, consider rembg library.

        Args:
            img: PIL Image with RGBA mode

        Returns:
            Image with background removed
        """
        # Convert to numpy array
        data = np.array(img)

        # Simple approach: assume background is most common color
        # For pixel art, this often works well
        rgb = data[:, :, :3]
        alpha = data[:, :, 3]

        # Find most common color (ignoring already transparent pixels)
        opaque_mask = alpha > 0
        if not opaque_mask.any():
            return img

        opaque_pixels = rgb[opaque_mask].reshape(-1, 3)

        # Count color frequencies (quantized to reduce complexity)
        quantized = (opaque_pixels // 32) * 32  # Quantize to 32-step intervals
        unique, counts = np.unique(
            quantized.reshape(-1, 3), axis=0, return_counts=True
        )

        # Most common color is likely background
        bg_color = unique[counts.argmax()]

        # Create mask for background color (with tolerance)
        tolerance = 40
        color_diff = np.abs(rgb.astype(int) - bg_color.astype(int))
        is_background = np.all(color_diff < tolerance, axis=2)

        # Set background pixels to transparent
        data[:, :, 3] = np.where(is_background, 0, alpha)

        return Image.fromarray(data, mode="RGBA")

    def _replace_background(
        self, img: Image.Image, color: Tuple[int, int, int, int]
    ) -> Image.Image:
        """Replace transparent background with solid color

        Args:
            img: PIL Image with RGBA mode
            color: RGBA color tuple

        Returns:
            Image with replaced background
        """
        # Create background layer
        background = Image.new("RGBA", img.size, color)

        # Composite sprite over background
        return Image.alpha_composite(background, img)

    def _quantize_colors(self, img: Image.Image, max_colors: int) -> Image.Image:
        """Reduce image to N-color palette

        Args:
            img: PIL Image
            max_colors: Maximum number of colors

        Returns:
            Quantized image
        """
        # Convert to P mode (palette) with specified colors
        # Keep alpha channel separate
        alpha = img.split()[-1]

        # Quantize RGB channels
        img_rgb = img.convert("RGB")
        img_quantized = img_rgb.quantize(colors=max_colors, method=2)  # method=2 = median cut

        # Convert back to RGBA and restore alpha
        img_quantized = img_quantized.convert("RGBA")
        img_quantized.putalpha(alpha)

        return img_quantized

    def process_batch(
        self, input_files: List[Path], output_dir: Path, options: ProcessingOptions
    ) -> List[Tuple[Path, bool]]:
        """Process multiple images

        Args:
            input_files: List of input file paths
            output_dir: Output directory
            options: Processing options

        Returns:
            List of (output_path, success) tuples
        """
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        results = []

        for input_path in input_files:
            input_path = Path(input_path)
            output_path = output_dir / input_path.name

            try:
                success = self.process_image(input_path, output_path, options)
                results.append((output_path, success))
            except Exception as e:
                print(f"Error processing {input_path}: {e}")
                results.append((output_path, False))

        return results


def create_preset_options(preset: str) -> ProcessingOptions:
    """Create ProcessingOptions from preset name

    Args:
        preset: Preset name (pixel_art, retro, modern, minimal)

    Returns:
        ProcessingOptions configured for preset
    """
    presets = {
        "pixel_art": ProcessingOptions(
            quantize_colors=True,
            max_colors=64,
            auto_crop=True,
            optimize=True,
            compression_level=9,
        ),
        "retro": ProcessingOptions(
            quantize_colors=True,
            max_colors=16,
            auto_crop=True,
            optimize=True,
            compression_level=9,
        ),
        "modern": ProcessingOptions(
            quantize_colors=False,
            auto_crop=True,
            optimize=True,
            compression_level=6,
        ),
        "minimal": ProcessingOptions(
            quantize_colors=False,
            auto_crop=False,
            optimize=True,
            compression_level=6,
        ),
    }

    if preset not in presets:
        raise ValueError(
            f"Unknown preset: {preset}. Available: {list(presets.keys())}"
        )

    return presets[preset]


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 3:
        print("Usage: python -m python.deployment.post_processor <input> <output> [preset]")
        print("Presets: pixel_art, retro, modern, minimal")
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    preset = sys.argv[3] if len(sys.argv) > 3 else "pixel_art"

    processor = PostProcessor()
    options = create_preset_options(preset)

    try:
        success = processor.process_image(input_path, output_path, options)
        if success:
            print(f"✓ Processed: {input_path} → {output_path}")
        else:
            print(f"✗ Failed: {input_path}")
            sys.exit(1)
    except Exception as e:
        print(f"✗ Error: {e}")
        sys.exit(1)
