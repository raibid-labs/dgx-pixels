"""FastMCP Server for DGX-Pixels Bevy Integration

This server provides MCP (Model Context Protocol) tools for Bevy game engine
to generate and deploy pixel art sprites using the DGX-Pixels AI backend.

Usage:
    python -m python.mcp_server.server

    Or with custom config:
    DGX_PIXELS_CONFIG=/path/to/config.yaml python -m python.mcp_server.server
"""

import asyncio
import logging
import os
import sys
from pathlib import Path
from typing import Dict, Any, List, Optional

from fastmcp import FastMCP

from .config_loader import load_config, Config
from .tools import MCPTools

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="[%(asctime)s] %(levelname)s - %(name)s - %(message)s",
)
logger = logging.getLogger("dgx-pixels-mcp")

# Load configuration
config_path = os.getenv("DGX_PIXELS_CONFIG")
try:
    config = load_config(config_path)
    logger.info(f"Loaded configuration from {config_path or 'default location'}")
except Exception as e:
    logger.error(f"Failed to load configuration: {e}")
    sys.exit(1)

# Initialize MCP server
mcp = FastMCP(
    name=config.mcp_server.name,
    version=config.mcp_server.version,
)

# Initialize tools
tools = MCPTools(config)

logger.info(f"Initialized {config.mcp_server.name} v{config.mcp_server.version}")


# ============================================================================
# MCP Tool Definitions
# ============================================================================


@mcp.tool()
async def generate_sprite(
    prompt: str,
    style: str = "pixel_art",
    resolution: str = "1024x1024",
    steps: Optional[int] = None,
    cfg_scale: Optional[float] = None,
    lora: Optional[str] = None,
    output_path: Optional[str] = None,
) -> Dict[str, Any]:
    """Generate a pixel art sprite using SDXL + LoRA

    This tool generates a single pixel art sprite based on a text prompt.
    It uses Stable Diffusion XL with optional LoRA fine-tuning for consistent
    game art style.

    Args:
        prompt: Text description of the sprite (e.g., "medieval knight with sword")
        style: Art style - pixel_art (default), 16bit, 8bit, retro, game_sprite
        resolution: Output resolution - 512x512, 1024x1024 (default), 2048x2048
        steps: Number of sampling steps (10-100, default: 30)
        cfg_scale: CFG scale value (1.0-20.0, default: 7.5)
        lora: Optional LoRA model name for custom style
        output_path: Optional output path (auto-generated if not provided)

    Returns:
        Dictionary with generation results:
            status: "success" or "error"
            job_id: Unique job identifier
            output_path: Path to generated sprite
            generation_time: Time taken in seconds
            error: Error message (if status is "error")

    Example:
        result = await generate_sprite(
            prompt="pixel art knight character",
            style="16bit",
            resolution="1024x1024"
        )
        print(f"Generated sprite at: {result['output_path']}")
    """
    if config.logging.get("log_tool_calls", True):
        logger.info(f"generate_sprite called: prompt='{prompt}', style={style}, resolution={resolution}")

    result = await tools.generate_sprite(
        prompt=prompt,
        style=style,
        resolution=resolution,
        steps=steps,
        cfg_scale=cfg_scale,
        lora=lora,
        output_path=output_path,
    )

    if result["status"] == "success":
        logger.info(f"Sprite generated successfully: job_id={result['job_id']}")
    else:
        logger.error(f"Sprite generation failed: {result.get('error')}")

    return result


@mcp.tool()
async def generate_batch(
    prompts: List[str],
    style: str = "pixel_art",
    resolution: str = "1024x1024",
    steps: Optional[int] = None,
    cfg_scale: Optional[float] = None,
    output_dir: Optional[str] = None,
) -> Dict[str, Any]:
    """Generate multiple sprites in batch

    This tool generates multiple sprites from a list of prompts, useful for
    creating entire sprite sheets or character sets in one call.

    Args:
        prompts: List of text descriptions (max 20 prompts)
        style: Art style for all sprites (pixel_art, 16bit, 8bit, retro, game_sprite)
        resolution: Output resolution (512x512, 1024x1024, 2048x2048)
        steps: Number of sampling steps per sprite (10-100, default: 30)
        cfg_scale: CFG scale value (1.0-20.0, default: 7.5)
        output_dir: Output directory (auto-generated if not provided)

    Returns:
        Dictionary with batch results:
            status: "success", "partial", or "error"
            job_ids: List of job IDs (one per prompt)
            output_paths: List of output paths
            successful: Number of successful generations
            failed: Number of failed generations
            total_time: Total time taken in seconds
            errors: List of error messages (if any)

    Example:
        result = await generate_batch(
            prompts=["knight sprite", "wizard sprite", "archer sprite"],
            style="pixel_art"
        )
        print(f"Generated {result['successful']}/{len(prompts)} sprites")
    """
    if config.logging.get("log_tool_calls", True):
        logger.info(f"generate_batch called: {len(prompts)} prompts, style={style}")

    result = await tools.generate_batch(
        prompts=prompts,
        style=style,
        resolution=resolution,
        steps=steps,
        cfg_scale=cfg_scale,
        output_dir=output_dir,
    )

    if result["status"] == "success":
        logger.info(f"Batch completed: {result['successful']} successful")
    else:
        logger.warning(
            f"Batch completed with errors: {result.get('successful', 0)} successful, "
            f"{result.get('failed', 0)} failed"
        )

    return result


@mcp.tool()
async def deploy_to_bevy(
    sprite_path: str,
    bevy_assets_dir: str,
    sprite_name: str,
    update_manifest: bool = True,
) -> Dict[str, Any]:
    """Deploy generated sprite to Bevy assets directory

    This tool copies a generated sprite to the Bevy game project's assets
    directory and optionally updates the asset manifest for hot-reloading.

    Args:
        sprite_path: Path to generated sprite file
        bevy_assets_dir: Path to Bevy project's assets directory
        sprite_name: Name for the sprite (without .png extension)
        update_manifest: Whether to update asset manifest (default: True)

    Returns:
        Dictionary with deployment results:
            status: "success" or "error"
            deployed_path: Full path where sprite was deployed
            manifest_updated: Whether manifest was updated
            error: Error message (if status is "error")

    Example:
        result = await deploy_to_bevy(
            sprite_path="/tmp/knight.png",
            bevy_assets_dir="/home/user/game/assets",
            sprite_name="player_knight"
        )
        print(f"Deployed to: {result['deployed_path']}")

    Notes:
        - Sprites are deployed to {bevy_assets_dir}/sprites/ by default
        - Manifest is written to {bevy_assets_dir}/asset_manifest.json
        - Bevy must have hot-reloading enabled to see changes immediately
    """
    if config.logging.get("log_tool_calls", True):
        logger.info(
            f"deploy_to_bevy called: sprite={sprite_name}, dest={bevy_assets_dir}"
        )

    result = await tools.deploy_to_bevy(
        sprite_path=sprite_path,
        bevy_assets_dir=bevy_assets_dir,
        sprite_name=sprite_name,
        update_manifest=update_manifest,
    )

    if result["status"] == "success":
        logger.info(f"Sprite deployed successfully: {result['deployed_path']}")
    else:
        logger.error(f"Deployment failed: {result.get('error')}")

    return result


# ============================================================================
# Server Lifecycle
# ============================================================================


def start_server():
    """Start the FastMCP server

    This runs the MCP server with the configured transports (stdio and/or SSE).
    The server will block until interrupted with Ctrl+C.
    """
    logger.info("Starting FastMCP server...")
    logger.info(f"Backend endpoint: {config.backend.zmq_endpoint}")
    logger.info(f"ComfyUI URL: {config.backend.comfyui_url}")
    logger.info(f"Output directory: {config.generation.output_dir}")
    logger.info(f"Transports: {', '.join(config.mcp_server.transports)}")

    try:
        # Ensure output directory exists
        output_dir = Path(config.generation.output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        logger.info(f"Output directory ready: {output_dir}")

        # Run server (FastMCP handles transport selection automatically)
        mcp.run()

    except KeyboardInterrupt:
        logger.info("Server interrupted by user")
    except Exception as e:
        logger.error(f"Server error: {e}", exc_info=True)
        sys.exit(1)


if __name__ == "__main__":
    start_server()
