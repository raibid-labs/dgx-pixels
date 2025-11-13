"""Configuration management for MCP server"""

import os
import yaml
from pathlib import Path
from typing import Any, Dict, List, Optional
from dataclasses import dataclass


@dataclass
class MCPServerConfig:
    """MCP server configuration"""

    name: str
    version: str
    description: str
    transports: List[str]
    host: str
    port: int


@dataclass
class BackendConfig:
    """Backend worker configuration"""

    zmq_endpoint: str
    timeout_s: float
    max_retries: int
    retry_delay_s: float
    comfyui_url: str


@dataclass
class GenerationConfig:
    """Generation settings"""

    default_workflow: str
    workflow_dir: str
    default_resolution: str
    default_steps: int
    default_cfg_scale: float
    default_model: str
    output_dir: str
    max_batch_size: int
    batch_timeout_s: float


@dataclass
class DeploymentConfig:
    """Bevy deployment configuration"""

    bevy_assets_base: str
    sprite_subdir: str
    manifest_file: str
    filename_pattern: str
    validate_bevy_structure: bool


@dataclass
class ValidationConfig:
    """Validation rules"""

    min_prompt_length: int
    max_prompt_length: int
    allowed_resolutions: List[str]
    min_steps: int
    max_steps: int
    min_cfg_scale: float
    max_cfg_scale: float
    allowed_styles: List[str]


@dataclass
class Config:
    """Complete MCP server configuration"""

    mcp_server: MCPServerConfig
    backend: BackendConfig
    generation: GenerationConfig
    deployment: DeploymentConfig
    validation: ValidationConfig
    error_handling: Dict[str, Any]
    logging: Dict[str, Any]
    performance: Dict[str, Any]


def load_config(config_path: Optional[str] = None) -> Config:
    """Load configuration from YAML file

    Args:
        config_path: Path to config file (defaults to config/mcp_config.yaml)

    Returns:
        Config object

    Raises:
        FileNotFoundError: If config file not found
        ValueError: If config is invalid
    """
    if config_path is None:
        # Default to config/mcp_config.yaml relative to project root
        project_root = Path(__file__).parent.parent.parent
        config_path = project_root / "config" / "mcp_config.yaml"

    config_path = Path(config_path)

    if not config_path.exists():
        raise FileNotFoundError(f"Config file not found: {config_path}")

    with open(config_path, "r") as f:
        data = yaml.safe_load(f)

    # Parse nested configurations
    mcp_server_data = data.get("mcp_server", {})
    backend_data = data.get("backend", {})
    generation_data = data.get("generation", {})
    deployment_data = data.get("deployment", {})
    validation_data = data.get("validation", {})

    # Support environment variable overrides
    backend_data["zmq_endpoint"] = os.getenv(
        "DGX_PIXELS_ZMQ_ENDPOINT", backend_data.get("zmq_endpoint")
    )
    backend_data["comfyui_url"] = os.getenv(
        "DGX_PIXELS_COMFYUI_URL", backend_data.get("comfyui_url")
    )
    deployment_data["bevy_assets_base"] = os.getenv(
        "DGX_PIXELS_BEVY_ASSETS", deployment_data.get("bevy_assets_base")
    )

    # Build config objects
    mcp_server = MCPServerConfig(
        name=mcp_server_data.get("name", "dgx-pixels"),
        version=mcp_server_data.get("version", "0.1.0"),
        description=mcp_server_data.get("description", ""),
        transports=mcp_server_data.get("transports", ["stdio"]),
        host=mcp_server_data.get("host", "127.0.0.1"),
        port=mcp_server_data.get("port", 3000),
    )

    backend = BackendConfig(
        zmq_endpoint=backend_data.get("zmq_endpoint", "tcp://localhost:5555"),
        timeout_s=backend_data.get("timeout_s", 300),
        max_retries=backend_data.get("max_retries", 3),
        retry_delay_s=backend_data.get("retry_delay_s", 2.0),
        comfyui_url=backend_data.get("comfyui_url", "http://localhost:8188"),
    )

    generation = GenerationConfig(
        default_workflow=generation_data.get("default_workflow", "sprite_optimized.json"),
        workflow_dir=generation_data.get("workflow_dir", "./workflows"),
        default_resolution=generation_data.get("default_resolution", "1024x1024"),
        default_steps=generation_data.get("default_steps", 30),
        default_cfg_scale=generation_data.get("default_cfg_scale", 7.5),
        default_model=generation_data.get("default_model", "SDXL Base 1.0"),
        output_dir=generation_data.get("output_dir", "./outputs"),
        max_batch_size=generation_data.get("max_batch_size", 20),
        batch_timeout_s=generation_data.get("batch_timeout_s", 600),
    )

    deployment = DeploymentConfig(
        bevy_assets_base=deployment_data.get("bevy_assets_base", "./bevy/assets"),
        sprite_subdir=deployment_data.get("sprite_subdir", "sprites"),
        manifest_file=deployment_data.get("manifest_file", "asset_manifest.json"),
        filename_pattern=deployment_data.get("filename_pattern", "{sprite_name}.png"),
        validate_bevy_structure=deployment_data.get("validate_bevy_structure", True),
    )

    validation = ValidationConfig(
        min_prompt_length=validation_data.get("min_prompt_length", 3),
        max_prompt_length=validation_data.get("max_prompt_length", 500),
        allowed_resolutions=validation_data.get(
            "allowed_resolutions", ["512x512", "1024x1024", "2048x2048"]
        ),
        min_steps=validation_data.get("min_steps", 10),
        max_steps=validation_data.get("max_steps", 100),
        min_cfg_scale=validation_data.get("min_cfg_scale", 1.0),
        max_cfg_scale=validation_data.get("max_cfg_scale", 20.0),
        allowed_styles=validation_data.get(
            "allowed_styles", ["pixel_art", "16bit", "8bit", "retro", "game_sprite"]
        ),
    )

    return Config(
        mcp_server=mcp_server,
        backend=backend,
        generation=generation,
        deployment=deployment,
        validation=validation,
        error_handling=data.get("error_handling", {}),
        logging=data.get("logging", {}),
        performance=data.get("performance", {}),
    )
