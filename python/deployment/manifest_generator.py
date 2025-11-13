"""Manifest generator for Bevy asset integration

Generates asset manifests that Bevy projects can use to discover
and load AI-generated sprites. Supports both JSON and TOML formats.
"""

import json
import toml
from pathlib import Path
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict
from datetime import datetime


@dataclass
class AssetMetadata:
    """Metadata for a single asset"""

    name: str
    path: str  # Relative to assets/ directory
    category: Optional[str] = None
    variant: Optional[str] = None
    frames: int = 1
    resolution: List[int] = None  # [width, height]
    generated_at: Optional[str] = None
    prompt: Optional[str] = None
    workflow: Optional[str] = None
    lora: Optional[str] = None
    file_size_kb: Optional[float] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary, excluding None values"""
        data = asdict(self)
        return {k: v for k, v in data.items() if v is not None}


class ManifestGenerator:
    """Generates Bevy asset manifests from sprite directories

    Creates manifest files that list all available sprites with metadata,
    allowing Bevy projects to programmatically load assets.

    Supported formats:
    - JSON (.json)
    - TOML (.toml)
    """

    def __init__(self, assets_dir: Path):
        """Initialize manifest generator

        Args:
            assets_dir: Path to Bevy assets directory
        """
        self.assets_dir = Path(assets_dir)

    def scan_sprites(self, sprites_subdir: str = "sprites") -> List[AssetMetadata]:
        """Scan sprites directory and extract metadata

        Args:
            sprites_subdir: Subdirectory containing sprites (relative to assets_dir)

        Returns:
            List of AssetMetadata for each sprite
        """
        sprites_dir = self.assets_dir / sprites_subdir

        if not sprites_dir.exists():
            return []

        sprites = []

        # Group sprites by animation sequences
        sprite_groups = self._group_animation_frames(sprites_dir)

        for base_name, files in sprite_groups.items():
            # Parse first file for metadata
            first_file = sorted(files)[0]

            try:
                from PIL import Image

                with Image.open(first_file) as img:
                    resolution = list(img.size)
                    file_size_kb = first_file.stat().st_size / 1024
            except Exception:
                resolution = None
                file_size_kb = None

            # Parse name components
            parts = base_name.split("_")
            category = parts[0] if len(parts) > 0 else None
            name = parts[1] if len(parts) > 1 else base_name
            variant = parts[2] if len(parts) > 2 else None

            # Relative path from assets directory
            rel_path = str(first_file.relative_to(self.assets_dir))

            metadata = AssetMetadata(
                name=base_name,
                path=rel_path,
                category=category,
                variant=variant,
                frames=len(files),
                resolution=resolution,
                generated_at=datetime.fromtimestamp(
                    first_file.stat().st_mtime
                ).isoformat(),
                file_size_kb=round(file_size_kb, 2) if file_size_kb else None,
            )

            sprites.append(metadata)

        return sprites

    def _group_animation_frames(self, directory: Path) -> Dict[str, List[Path]]:
        """Group sprite files by animation sequence

        Files like character_walk_0001.png, character_walk_0002.png
        are grouped under "character_walk".

        Args:
            directory: Directory to scan

        Returns:
            Dict mapping base name to list of frame files
        """
        import re

        groups: Dict[str, List[Path]] = {}

        # Pattern to match frame numbers at end
        frame_pattern = re.compile(r"(.+?)_(\d{3,4})$")

        for png_file in directory.glob("*.png"):
            stem = png_file.stem  # filename without extension

            # Check if it has frame number
            match = frame_pattern.match(stem)
            if match:
                base_name = match.group(1)
            else:
                base_name = stem

            if base_name not in groups:
                groups[base_name] = []

            groups[base_name].append(png_file)

        return groups

    def generate_manifest(
        self,
        output_path: Path,
        format: str = "json",
        include_metadata: bool = True,
        sprites_subdir: str = "sprites",
    ) -> bool:
        """Generate asset manifest file

        Args:
            output_path: Path to save manifest
            format: Output format ("json" or "toml")
            include_metadata: Include detailed metadata (prompt, workflow, etc.)
            sprites_subdir: Subdirectory containing sprites

        Returns:
            True if generation succeeded
        """
        output_path = Path(output_path)

        # Scan sprites
        sprites = self.scan_sprites(sprites_subdir)

        if not sprites:
            print(f"Warning: No sprites found in {self.assets_dir / sprites_subdir}")

        # Build manifest structure
        manifest = {
            "version": "1.0",
            "generated_at": datetime.now().isoformat(),
            "assets_dir": str(self.assets_dir),
            "sprite_count": len(sprites),
            "sprites": [s.to_dict() for s in sprites],
        }

        # Write manifest
        try:
            output_path.parent.mkdir(parents=True, exist_ok=True)

            if format == "json":
                with open(output_path, "w") as f:
                    json.dump(manifest, f, indent=2)
            elif format == "toml":
                with open(output_path, "w") as f:
                    # TOML doesn't handle nested arrays well, flatten structure
                    toml_data = {
                        "manifest": {
                            "version": manifest["version"],
                            "generated_at": manifest["generated_at"],
                            "sprite_count": manifest["sprite_count"],
                        }
                    }

                    # Add each sprite as [[sprites]] array entry
                    toml_data["sprites"] = manifest["sprites"]

                    toml.dump(toml_data, f)
            else:
                raise ValueError(f"Unsupported format: {format}")

            return True

        except Exception as e:
            print(f"Failed to write manifest: {e}")
            return False

    def update_manifest(
        self,
        manifest_path: Path,
        new_sprite: AssetMetadata,
        format: str = "json",
    ) -> bool:
        """Update existing manifest with new sprite

        Args:
            manifest_path: Path to existing manifest
            new_sprite: Metadata for new sprite
            format: Manifest format

        Returns:
            True if update succeeded
        """
        manifest_path = Path(manifest_path)

        # Load existing manifest
        if manifest_path.exists():
            try:
                if format == "json":
                    with open(manifest_path, "r") as f:
                        manifest = json.load(f)
                elif format == "toml":
                    with open(manifest_path, "r") as f:
                        manifest = toml.load(f)
                        # Flatten manifest structure if needed
                        if "manifest" in manifest:
                            for key, value in manifest["manifest"].items():
                                manifest[key] = value
                            del manifest["manifest"]
                else:
                    raise ValueError(f"Unsupported format: {format}")
            except Exception as e:
                print(f"Failed to load existing manifest: {e}")
                # Start fresh
                manifest = {"version": "1.0", "sprites": []}
        else:
            manifest = {"version": "1.0", "sprites": []}

        # Update sprite entry or add new
        sprite_dict = new_sprite.to_dict()

        existing_idx = None
        for i, entry in enumerate(manifest.get("sprites", [])):
            if entry.get("name") == new_sprite.name:
                existing_idx = i
                break

        if existing_idx is not None:
            manifest["sprites"][existing_idx] = sprite_dict
        else:
            if "sprites" not in manifest:
                manifest["sprites"] = []
            manifest["sprites"].append(sprite_dict)

        # Update metadata
        manifest["generated_at"] = datetime.now().isoformat()
        manifest["sprite_count"] = len(manifest["sprites"])

        # Write updated manifest
        try:
            if format == "json":
                with open(manifest_path, "w") as f:
                    json.dump(manifest, f, indent=2)
            elif format == "toml":
                # Restructure for TOML
                toml_data = {
                    "manifest": {
                        "version": manifest["version"],
                        "generated_at": manifest["generated_at"],
                        "sprite_count": manifest["sprite_count"],
                    },
                    "sprites": manifest["sprites"],
                }
                with open(manifest_path, "w") as f:
                    toml.dump(toml_data, f)

            return True

        except Exception as e:
            print(f"Failed to write manifest: {e}")
            return False

    def get_sprite_list(self, manifest_path: Path, format: str = "json") -> List[str]:
        """Get list of sprite names from manifest

        Args:
            manifest_path: Path to manifest file
            format: Manifest format

        Returns:
            List of sprite names
        """
        try:
            if format == "json":
                with open(manifest_path, "r") as f:
                    manifest = json.load(f)
            elif format == "toml":
                with open(manifest_path, "r") as f:
                    manifest = toml.load(f)
                    if "manifest" in manifest:
                        for key, value in manifest["manifest"].items():
                            manifest[key] = value
            else:
                raise ValueError(f"Unsupported format: {format}")

            return [s["name"] for s in manifest.get("sprites", [])]

        except Exception as e:
            print(f"Failed to read manifest: {e}")
            return []


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 3:
        print("Usage: python -m python.deployment.manifest_generator <assets_dir> <output_file> [format]")
        print("Formats: json, toml")
        sys.exit(1)

    assets_dir = Path(sys.argv[1])
    output_file = Path(sys.argv[2])
    format = sys.argv[3] if len(sys.argv) > 3 else "json"

    generator = ManifestGenerator(assets_dir)

    if generator.generate_manifest(output_file, format=format):
        sprites = generator.scan_sprites()
        print(f"✓ Generated manifest: {output_file}")
        print(f"  Found {len(sprites)} sprites")
    else:
        print(f"✗ Failed to generate manifest")
        sys.exit(1)
