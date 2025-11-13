"""MCP tool implementations for sprite generation and deployment"""

import os
import shutil
import json
import time
import asyncio
from pathlib import Path
from typing import Dict, Any, Optional, List
from dataclasses import dataclass

from .config_loader import Config
from .backend_client import BackendClient


@dataclass
class ValidationError(Exception):
    """Validation error with details"""

    field: str
    message: str


class MCPTools:
    """MCP tool implementations for DGX-Pixels

    Provides three core tools:
    1. generate_sprite - Generate a single pixel art sprite
    2. generate_batch - Generate multiple sprites
    3. deploy_to_bevy - Deploy sprite to Bevy assets directory
    """

    def __init__(self, config: Config):
        """Initialize tools

        Args:
            config: MCP server configuration
        """
        self.config = config
        self.backend_client = BackendClient(
            zmq_endpoint=config.backend.zmq_endpoint,
            timeout_s=config.backend.timeout_s,
        )

    def _validate_prompt(self, prompt: str) -> None:
        """Validate prompt text

        Args:
            prompt: Text prompt

        Raises:
            ValidationError: If validation fails
        """
        if not prompt or not prompt.strip():
            raise ValidationError("prompt", "Prompt cannot be empty")

        if len(prompt) < self.config.validation.min_prompt_length:
            raise ValidationError(
                "prompt",
                f"Prompt must be at least {self.config.validation.min_prompt_length} characters",
            )

        if len(prompt) > self.config.validation.max_prompt_length:
            raise ValidationError(
                "prompt",
                f"Prompt must be at most {self.config.validation.max_prompt_length} characters",
            )

    def _validate_resolution(self, resolution: str) -> List[int]:
        """Validate and parse resolution string

        Args:
            resolution: Resolution string (e.g., "1024x1024")

        Returns:
            [width, height] list

        Raises:
            ValidationError: If validation fails
        """
        if resolution not in self.config.validation.allowed_resolutions:
            raise ValidationError(
                "resolution",
                f"Resolution must be one of: {', '.join(self.config.validation.allowed_resolutions)}",
            )

        try:
            parts = resolution.split("x")
            if len(parts) != 2:
                raise ValueError()

            width = int(parts[0])
            height = int(parts[1])

            return [width, height]
        except (ValueError, IndexError):
            raise ValidationError(
                "resolution", "Resolution must be in format: WIDTHxHEIGHT (e.g., 1024x1024)"
            )

    def _validate_steps(self, steps: int) -> None:
        """Validate steps parameter

        Args:
            steps: Number of sampling steps

        Raises:
            ValidationError: If validation fails
        """
        if steps < self.config.validation.min_steps:
            raise ValidationError(
                "steps", f"Steps must be at least {self.config.validation.min_steps}"
            )

        if steps > self.config.validation.max_steps:
            raise ValidationError(
                "steps", f"Steps must be at most {self.config.validation.max_steps}"
            )

    def _validate_cfg_scale(self, cfg_scale: float) -> None:
        """Validate CFG scale parameter

        Args:
            cfg_scale: CFG scale value

        Raises:
            ValidationError: If validation fails
        """
        if cfg_scale < self.config.validation.min_cfg_scale:
            raise ValidationError(
                "cfg_scale",
                f"CFG scale must be at least {self.config.validation.min_cfg_scale}",
            )

        if cfg_scale > self.config.validation.max_cfg_scale:
            raise ValidationError(
                "cfg_scale",
                f"CFG scale must be at most {self.config.validation.max_cfg_scale}",
            )

    def _validate_style(self, style: str) -> None:
        """Validate style parameter

        Args:
            style: Art style

        Raises:
            ValidationError: If validation fails
        """
        if style not in self.config.validation.allowed_styles:
            raise ValidationError(
                "style",
                f"Style must be one of: {', '.join(self.config.validation.allowed_styles)}",
            )

    async def generate_sprite(
        self,
        prompt: str,
        style: str = "pixel_art",
        resolution: str = "1024x1024",
        steps: Optional[int] = None,
        cfg_scale: Optional[float] = None,
        lora: Optional[str] = None,
        output_path: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Generate a pixel art sprite using SDXL + LoRA

        Args:
            prompt: Text description of the sprite to generate
            style: Art style (pixel_art, 16bit, 8bit, retro, game_sprite)
            resolution: Output resolution (512x512, 1024x1024, 2048x2048)
            steps: Number of sampling steps (10-100, default: 30)
            cfg_scale: CFG scale value (1.0-20.0, default: 7.5)
            lora: Optional LoRA model name
            output_path: Optional output path (uses default if not provided)

        Returns:
            Dictionary with:
                status: "success" or "error"
                job_id: Unique job identifier
                output_path: Path to generated image
                generation_time: Time taken in seconds
                error: Error message (if status is "error")
        """
        try:
            # Validate parameters
            self._validate_prompt(prompt)
            self._validate_style(style)
            size = self._validate_resolution(resolution)

            if steps is None:
                steps = self.config.generation.default_steps
            else:
                self._validate_steps(steps)

            if cfg_scale is None:
                cfg_scale = self.config.generation.default_cfg_scale
            else:
                self._validate_cfg_scale(cfg_scale)

            # Connect to backend
            await self.backend_client.connect()

            # Submit generation request
            start_time = time.time()

            result = await self.backend_client.generate_sprite(
                prompt=f"{style} style: {prompt}",  # Inject style into prompt
                model=self.config.generation.default_model,
                size=size,
                steps=steps,
                cfg_scale=cfg_scale,
                lora=lora,
            )

            job_id = result["job_id"]
            estimated_time = result["estimated_time_s"]

            # Wait for completion
            # NOTE: In a production implementation, we'd use PUB-SUB to listen for completion
            # For now, we return immediately with the job_id
            await asyncio.sleep(estimated_time)  # Simple wait

            generation_time = time.time() - start_time

            # Construct output path
            if output_path is None:
                output_dir = Path(self.config.generation.output_dir)
                output_path = str(output_dir / f"{job_id}.png")

            return {
                "status": "success",
                "job_id": job_id,
                "output_path": output_path,
                "generation_time": generation_time,
            }

        except ValidationError as e:
            return {
                "status": "error",
                "job_id": None,
                "error": f"Validation error in {e.field}: {e.message}",
            }
        except TimeoutError as e:
            return {
                "status": "error",
                "job_id": None,
                "error": f"Generation timed out: {str(e)}",
            }
        except ConnectionError as e:
            return {
                "status": "error",
                "job_id": None,
                "error": f"Backend connection failed: {str(e)}",
            }
        except Exception as e:
            return {
                "status": "error",
                "job_id": None,
                "error": f"Generation failed: {str(e)}",
            }

    async def generate_batch(
        self,
        prompts: List[str],
        style: str = "pixel_art",
        resolution: str = "1024x1024",
        steps: Optional[int] = None,
        cfg_scale: Optional[float] = None,
        output_dir: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Generate multiple sprites in batch

        Args:
            prompts: List of text prompts
            style: Art style for all sprites
            resolution: Output resolution
            steps: Number of sampling steps
            cfg_scale: CFG scale value
            output_dir: Output directory (uses default if not provided)

        Returns:
            Dictionary with:
                status: "success", "partial", or "error"
                job_ids: List of job IDs
                output_paths: List of output paths
                successful: Number of successful generations
                failed: Number of failed generations
                total_time: Total time taken in seconds
                errors: List of error messages (if any)
        """
        try:
            # Validate batch size
            if len(prompts) == 0:
                raise ValidationError("prompts", "Prompts list cannot be empty")

            if len(prompts) > self.config.generation.max_batch_size:
                raise ValidationError(
                    "prompts",
                    f"Batch size cannot exceed {self.config.generation.max_batch_size}",
                )

            # Validate common parameters
            self._validate_style(style)
            size = self._validate_resolution(resolution)

            if steps is None:
                steps = self.config.generation.default_steps
            else:
                self._validate_steps(steps)

            if cfg_scale is None:
                cfg_scale = self.config.generation.default_cfg_scale
            else:
                self._validate_cfg_scale(cfg_scale)

            if output_dir is None:
                output_dir = self.config.generation.output_dir

            # Connect to backend
            await self.backend_client.connect()

            # Submit all generation requests
            start_time = time.time()
            results = []
            errors = []

            for i, prompt in enumerate(prompts):
                try:
                    self._validate_prompt(prompt)

                    result = await self.backend_client.generate_sprite(
                        prompt=f"{style} style: {prompt}",
                        model=self.config.generation.default_model,
                        size=size,
                        steps=steps,
                        cfg_scale=cfg_scale,
                    )

                    results.append(result)

                except Exception as e:
                    errors.append(f"Prompt {i}: {str(e)}")
                    results.append(None)

            # Calculate statistics
            successful = sum(1 for r in results if r is not None)
            failed = len(results) - successful
            total_time = time.time() - start_time

            # Build output paths
            job_ids = [r["job_id"] if r else None for r in results]
            output_paths = [
                str(Path(output_dir) / f"{r['job_id']}.png") if r else None
                for r in results
            ]

            status = "success" if failed == 0 else ("partial" if successful > 0 else "error")

            return {
                "status": status,
                "job_ids": job_ids,
                "output_paths": output_paths,
                "successful": successful,
                "failed": failed,
                "total_time": total_time,
                "errors": errors if errors else None,
            }

        except ValidationError as e:
            return {
                "status": "error",
                "error": f"Validation error in {e.field}: {e.message}",
            }
        except Exception as e:
            return {
                "status": "error",
                "error": f"Batch generation failed: {str(e)}",
            }

    async def deploy_to_bevy(
        self,
        sprite_path: str,
        bevy_assets_dir: str,
        sprite_name: str,
        update_manifest: bool = True,
    ) -> Dict[str, Any]:
        """Deploy generated sprite to Bevy assets directory

        Args:
            sprite_path: Path to generated sprite file
            bevy_assets_dir: Path to Bevy assets directory
            sprite_name: Name for the sprite (without extension)
            update_manifest: Whether to update asset manifest

        Returns:
            Dictionary with:
                status: "success" or "error"
                deployed_path: Path where sprite was deployed
                manifest_updated: Whether manifest was updated
                error: Error message (if status is "error")
        """
        try:
            # Validate sprite file exists
            sprite_path = Path(sprite_path)
            if not sprite_path.exists():
                raise FileNotFoundError(f"Sprite file not found: {sprite_path}")

            # Validate Bevy assets directory
            bevy_assets_dir = Path(bevy_assets_dir)
            if not bevy_assets_dir.exists():
                if self.config.deployment.validate_bevy_structure:
                    raise FileNotFoundError(f"Bevy assets directory not found: {bevy_assets_dir}")
                else:
                    # Create directory if validation is disabled
                    bevy_assets_dir.mkdir(parents=True, exist_ok=True)

            # Create sprites subdirectory if needed
            sprites_dir = bevy_assets_dir / self.config.deployment.sprite_subdir
            sprites_dir.mkdir(parents=True, exist_ok=True)

            # Construct destination path
            filename = self.config.deployment.filename_pattern.format(sprite_name=sprite_name)
            if not filename.endswith(".png"):
                filename += ".png"

            deployed_path = sprites_dir / filename

            # Copy sprite file
            shutil.copy2(sprite_path, deployed_path)

            # Update manifest if requested
            manifest_updated = False
            if update_manifest:
                manifest_path = bevy_assets_dir / self.config.deployment.manifest_file
                manifest = {}

                # Load existing manifest
                if manifest_path.exists():
                    with open(manifest_path, "r") as f:
                        manifest = json.load(f)

                # Add sprite entry
                if "sprites" not in manifest:
                    manifest["sprites"] = []

                sprite_entry = {
                    "name": sprite_name,
                    "path": str(
                        Path(self.config.deployment.sprite_subdir) / filename
                    ),
                    "deployed_at": time.time(),
                }

                # Update or append entry
                existing_idx = None
                for i, entry in enumerate(manifest["sprites"]):
                    if entry.get("name") == sprite_name:
                        existing_idx = i
                        break

                if existing_idx is not None:
                    manifest["sprites"][existing_idx] = sprite_entry
                else:
                    manifest["sprites"].append(sprite_entry)

                # Write manifest
                with open(manifest_path, "w") as f:
                    json.dump(manifest, f, indent=2)

                manifest_updated = True

            return {
                "status": "success",
                "deployed_path": str(deployed_path),
                "manifest_updated": manifest_updated,
            }

        except FileNotFoundError as e:
            return {"status": "error", "error": f"File not found: {str(e)}"}
        except PermissionError as e:
            return {"status": "error", "error": f"Permission denied: {str(e)}"}
        except Exception as e:
            return {"status": "error", "error": f"Deployment failed: {str(e)}"}
