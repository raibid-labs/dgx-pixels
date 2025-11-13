"""Asset validation for deployment pipeline

Validates generated sprites meet requirements for Bevy integration:
- Correct format (PNG)
- Appropriate resolution (power-of-2)
- Proper naming convention
- Valid color modes
- Size limits
"""

import re
from pathlib import Path
from typing import List, Optional, Dict, Any
from dataclasses import dataclass
from PIL import Image


@dataclass
class ValidationError:
    """Validation error details"""

    field: str
    message: str
    severity: str  # "error", "warning"
    file_path: Optional[str] = None


@dataclass
class ValidationResult:
    """Result of asset validation"""

    valid: bool
    errors: List[ValidationError]
    warnings: List[ValidationError]
    file_path: str
    metadata: Dict[str, Any]

    @property
    def error_count(self) -> int:
        return len(self.errors)

    @property
    def warning_count(self) -> int:
        return len(self.warnings)

    def summary(self) -> str:
        """Generate human-readable summary"""
        if self.valid:
            msg = f"✓ Valid: {self.file_path}"
            if self.warnings:
                msg += f" ({len(self.warnings)} warnings)"
            return msg
        else:
            return f"✗ Invalid: {self.file_path} ({len(self.errors)} errors, {len(self.warnings)} warnings)"


class AssetValidator:
    """Validates assets for deployment to Bevy projects

    Checks:
    1. File format (PNG with RGB/RGBA)
    2. Resolution (power-of-2 recommended)
    3. File size (<10MB)
    4. Naming convention (category_name_variant_frame.ext)
    5. Color mode compatibility
    """

    # Naming convention: category_name_variant_frame.ext
    # Examples: character_knight_idle_0001.png, tile_grass_001.png
    NAMING_PATTERN = re.compile(
        r"^([a-z]+)_([a-z0-9]+)(?:_([a-z0-9]+))?(?:_(\d{3,4}))?\.png$"
    )

    ALLOWED_FORMATS = {"PNG"}
    ALLOWED_COLOR_MODES = {"RGB", "RGBA", "L"}  # RGB, RGBA, or grayscale
    RECOMMENDED_RESOLUTIONS = {128, 256, 512, 1024, 2048}
    MAX_FILE_SIZE_MB = 10

    def __init__(
        self,
        enforce_naming: bool = True,
        enforce_power_of_2: bool = False,  # Warning only by default
        allow_non_square: bool = True,
    ):
        """Initialize validator

        Args:
            enforce_naming: Require naming convention compliance
            enforce_power_of_2: Require power-of-2 resolutions (False = warning only)
            allow_non_square: Allow non-square images
        """
        self.enforce_naming = enforce_naming
        self.enforce_power_of_2 = enforce_power_of_2
        self.allow_non_square = allow_non_square

    def validate_file(self, file_path: Path) -> ValidationResult:
        """Validate a single asset file

        Args:
            file_path: Path to asset file

        Returns:
            ValidationResult with errors and warnings
        """
        errors: List[ValidationError] = []
        warnings: List[ValidationError] = []
        metadata: Dict[str, Any] = {}

        file_path = Path(file_path)

        # Check file exists
        if not file_path.exists():
            errors.append(
                ValidationError(
                    field="file",
                    message=f"File not found: {file_path}",
                    severity="error",
                    file_path=str(file_path),
                )
            )
            return ValidationResult(
                valid=False,
                errors=errors,
                warnings=warnings,
                file_path=str(file_path),
                metadata=metadata,
            )

        # Check file size
        file_size_mb = file_path.stat().st_size / (1024 * 1024)
        metadata["file_size_mb"] = round(file_size_mb, 2)

        if file_size_mb > self.MAX_FILE_SIZE_MB:
            errors.append(
                ValidationError(
                    field="size",
                    message=f"File too large: {file_size_mb:.2f}MB (max {self.MAX_FILE_SIZE_MB}MB)",
                    severity="error",
                    file_path=str(file_path),
                )
            )

        # Check naming convention
        filename = file_path.name
        metadata["filename"] = filename

        if self.enforce_naming:
            match = self.NAMING_PATTERN.match(filename)
            if not match:
                errors.append(
                    ValidationError(
                        field="naming",
                        message=f"Invalid naming convention. Expected: category_name_variant_frame.png",
                        severity="error",
                        file_path=str(file_path),
                    )
                )
            else:
                category, name, variant, frame = match.groups()
                metadata["category"] = category
                metadata["name"] = name
                metadata["variant"] = variant
                metadata["frame"] = frame

        # Validate image properties
        try:
            with Image.open(file_path) as img:
                # Check format
                if img.format not in self.ALLOWED_FORMATS:
                    errors.append(
                        ValidationError(
                            field="format",
                            message=f"Invalid format: {img.format}. Must be PNG",
                            severity="error",
                            file_path=str(file_path),
                        )
                    )

                metadata["format"] = img.format
                metadata["mode"] = img.mode
                metadata["size"] = img.size

                # Check color mode
                if img.mode not in self.ALLOWED_COLOR_MODES:
                    errors.append(
                        ValidationError(
                            field="color_mode",
                            message=f"Invalid color mode: {img.mode}. Must be RGB, RGBA, or L (grayscale)",
                            severity="error",
                            file_path=str(file_path),
                        )
                    )

                # Check resolution
                width, height = img.size

                # Check if square (warning only)
                if not self.allow_non_square and width != height:
                    warnings.append(
                        ValidationError(
                            field="resolution",
                            message=f"Non-square image: {width}x{height}",
                            severity="warning",
                            file_path=str(file_path),
                        )
                    )

                # Check if power of 2
                def is_power_of_2(n: int) -> bool:
                    return n > 0 and (n & (n - 1)) == 0

                width_pow2 = is_power_of_2(width)
                height_pow2 = is_power_of_2(height)

                if not (width_pow2 and height_pow2):
                    msg = f"Resolution not power-of-2: {width}x{height}. Recommended: {sorted(self.RECOMMENDED_RESOLUTIONS)}"
                    if self.enforce_power_of_2:
                        errors.append(
                            ValidationError(
                                field="resolution",
                                message=msg,
                                severity="error",
                                file_path=str(file_path),
                            )
                        )
                    else:
                        warnings.append(
                            ValidationError(
                                field="resolution",
                                message=msg,
                                severity="warning",
                                file_path=str(file_path),
                            )
                        )

        except Exception as e:
            errors.append(
                ValidationError(
                    field="image",
                    message=f"Failed to open image: {str(e)}",
                    severity="error",
                    file_path=str(file_path),
                )
            )

        valid = len(errors) == 0

        return ValidationResult(
            valid=valid,
            errors=errors,
            warnings=warnings,
            file_path=str(file_path),
            metadata=metadata,
        )

    def validate_directory(self, directory: Path) -> List[ValidationResult]:
        """Validate all PNG files in a directory

        Args:
            directory: Directory to scan

        Returns:
            List of ValidationResult for each file
        """
        directory = Path(directory)
        results = []

        for png_file in directory.rglob("*.png"):
            result = self.validate_file(png_file)
            results.append(result)

        return results

    def validate_batch(self, file_paths: List[Path]) -> List[ValidationResult]:
        """Validate multiple files

        Args:
            file_paths: List of file paths to validate

        Returns:
            List of ValidationResult
        """
        return [self.validate_file(path) for path in file_paths]


def print_validation_summary(results: List[ValidationResult]) -> None:
    """Print validation summary to console

    Args:
        results: List of validation results
    """
    total = len(results)
    valid = sum(1 for r in results if r.valid)
    invalid = total - valid
    total_errors = sum(r.error_count for r in results)
    total_warnings = sum(r.warning_count for r in results)

    print("\n" + "=" * 70)
    print("ASSET VALIDATION SUMMARY")
    print("=" * 70)
    print(f"Total files: {total}")
    print(f"Valid: {valid} | Invalid: {invalid}")
    print(f"Errors: {total_errors} | Warnings: {total_warnings}")
    print("=" * 70)

    if invalid > 0:
        print("\nFAILED FILES:")
        for result in results:
            if not result.valid:
                print(f"\n  {result.file_path}")
                for error in result.errors:
                    print(f"    ✗ [{error.field}] {error.message}")
                for warning in result.warnings:
                    print(f"    ⚠ [{warning.field}] {warning.message}")

    if total_warnings > 0:
        print("\nWARNINGS:")
        for result in results:
            if result.valid and result.warnings:
                print(f"\n  {result.file_path}")
                for warning in result.warnings:
                    print(f"    ⚠ [{warning.field}] {warning.message}")

    print()


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: python -m python.deployment.validator <directory>")
        sys.exit(1)

    directory = Path(sys.argv[1])
    validator = AssetValidator()
    results = validator.validate_directory(directory)
    print_validation_summary(results)

    # Exit with error code if any validation failed
    if any(not r.valid for r in results):
        sys.exit(1)
