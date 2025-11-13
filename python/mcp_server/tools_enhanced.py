"""Enhanced MCP tool implementations with deployment pipeline integration

This module extends the base MCP tools to use the new deployment pipeline
with validation, post-processing, and manifest generation.
"""

import subprocess
from pathlib import Path
from typing import Dict, Any, Optional

from .tools import MCPTools


class EnhancedMCPTools(MCPTools):
    """Enhanced MCP tools with deployment pipeline integration"""

    async def deploy_to_bevy_with_validation(
        self,
        sprite_path: str,
        bevy_project_path: str,
        validate: bool = True,
        post_process: bool = False,
        preset: str = "pixel_art",
    ) -> Dict[str, Any]:
        """Deploy sprite to Bevy project using the deployment pipeline

        This method wraps the deploy_assets.sh script to provide
        validation, post-processing, and manifest generation.

        Args:
            sprite_path: Path to generated sprite file
            bevy_project_path: Path to Bevy project root
            validate: Whether to validate asset before deployment
            post_process: Whether to apply post-processing
            preset: Post-processing preset (pixel_art, retro, modern, minimal)

        Returns:
            Dictionary with:
                status: "success" or "error"
                deployed_path: Path where sprite was deployed
                manifest_updated: Whether manifest was updated
                validation_passed: Whether validation passed (if enabled)
                error: Error message (if status is "error")
        """
        try:
            sprite_path = Path(sprite_path)
            bevy_project_path = Path(bevy_project_path)

            # Validate inputs
            if not sprite_path.exists():
                return {
                    "status": "error",
                    "error": f"Sprite file not found: {sprite_path}",
                }

            if not bevy_project_path.exists():
                return {
                    "status": "error",
                    "error": f"Bevy project not found: {bevy_project_path}",
                }

            # Prepare deployment script arguments
            script_path = Path(__file__).parent.parent.parent / "scripts" / "deploy_assets.sh"

            if not script_path.exists():
                # Fallback to basic deployment if script not found
                return await self.deploy_to_bevy(
                    str(sprite_path),
                    str(bevy_project_path / "assets"),
                    sprite_path.stem,
                    update_manifest=True,
                )

            # Build command
            cmd = [str(script_path), str(sprite_path.parent), str(bevy_project_path)]

            if not validate:
                cmd.append("--no-validate")

            if post_process:
                cmd.append("--post-process")
                cmd.append("--preset")
                cmd.append(preset)

            # Run deployment pipeline
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                cwd=sprite_path.parent.parent,
            )

            if result.returncode == 0:
                deployed_path = bevy_project_path / "assets" / "sprites" / sprite_path.name

                return {
                    "status": "success",
                    "deployed_path": str(deployed_path),
                    "manifest_updated": True,
                    "validation_passed": validate,
                    "output": result.stdout,
                }
            else:
                return {
                    "status": "error",
                    "error": result.stderr or "Deployment failed",
                    "output": result.stdout,
                }

        except Exception as e:
            return {
                "status": "error",
                "error": f"Deployment failed: {str(e)}",
            }

    async def validate_sprite(self, sprite_path: str) -> Dict[str, Any]:
        """Validate a sprite file against deployment requirements

        Args:
            sprite_path: Path to sprite file

        Returns:
            Dictionary with:
                valid: Whether sprite is valid
                errors: List of error messages
                warnings: List of warning messages
                metadata: Asset metadata (resolution, size, etc.)
        """
        try:
            from ..deployment.validator import AssetValidator

            sprite_path = Path(sprite_path)

            if not sprite_path.exists():
                return {
                    "valid": False,
                    "errors": [f"File not found: {sprite_path}"],
                    "warnings": [],
                    "metadata": {},
                }

            validator = AssetValidator()
            result = validator.validate_file(sprite_path)

            return {
                "valid": result.valid,
                "errors": [e.message for e in result.errors],
                "warnings": [w.message for w in result.warnings],
                "metadata": result.metadata,
            }

        except Exception as e:
            return {
                "valid": False,
                "errors": [f"Validation failed: {str(e)}"],
                "warnings": [],
                "metadata": {},
            }

    async def post_process_sprite(
        self,
        sprite_path: str,
        output_path: Optional[str] = None,
        preset: str = "pixel_art",
    ) -> Dict[str, Any]:
        """Post-process a sprite (quantization, cropping, optimization)

        Args:
            sprite_path: Path to sprite file
            output_path: Output path (defaults to overwriting input)
            preset: Processing preset (pixel_art, retro, modern, minimal)

        Returns:
            Dictionary with:
                status: "success" or "error"
                output_path: Path to processed sprite
                error: Error message (if status is "error")
        """
        try:
            from ..deployment.post_processor import PostProcessor, create_preset_options

            sprite_path = Path(sprite_path)

            if not sprite_path.exists():
                return {
                    "status": "error",
                    "error": f"File not found: {sprite_path}",
                }

            if output_path is None:
                output_path = sprite_path
            else:
                output_path = Path(output_path)

            processor = PostProcessor()
            options = create_preset_options(preset)

            success = processor.process_image(sprite_path, output_path, options)

            if success:
                return {
                    "status": "success",
                    "output_path": str(output_path),
                    "preset": preset,
                }
            else:
                return {
                    "status": "error",
                    "error": "Processing failed",
                }

        except Exception as e:
            return {
                "status": "error",
                "error": f"Post-processing failed: {str(e)}",
            }
