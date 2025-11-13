# DGX-Pixels FastMCP Server

FastMCP server for integrating DGX-Pixels AI sprite generation with Bevy game engine via Model Context Protocol (MCP).

## Overview

This MCP server exposes three core tools that enable Bevy game projects to programmatically generate and deploy pixel art sprites:

1. **generate_sprite** - Generate a single pixel art sprite
2. **generate_batch** - Generate multiple sprites in batch
3. **deploy_to_bevy** - Deploy sprites to Bevy assets directory

The server acts as a bridge between Bevy game engine and the DGX-Pixels Python backend worker, which communicates with ComfyUI for AI generation.

## Architecture

```
Bevy Game Engine
       ↓ (MCP Tools)
FastMCP Server (this module)
       ↓ (ZeroMQ REQ-REP)
Python Backend Worker
       ↓ (HTTP API)
ComfyUI
       ↓ (Inference)
NVIDIA DGX-Spark GPU
```

## Installation

### Prerequisites

- Python 3.10+
- DGX-Pixels backend worker running (WS-10)
- ComfyUI server running
- ZeroMQ endpoints accessible

### Install Dependencies

```bash
cd python/mcp_server
pip install -r requirements.txt
```

Or install individually:

```bash
pip install fastmcp pyzmq pyyaml aiohttp msgpack
```

## Configuration

Edit `config/mcp_config.yaml` to configure the server:

```yaml
mcp_server:
  name: "dgx-pixels"
  version: "0.1.0"
  transports:
    - stdio  # Standard MCP transport
    - sse    # Optional: Server-Sent Events

backend:
  zmq_endpoint: "tcp://localhost:5555"  # Backend worker endpoint
  comfyui_url: "http://localhost:8188"  # ComfyUI server

generation:
  default_workflow: "sprite_optimized.json"
  output_dir: "/home/beengud/raibid-labs/dgx-pixels/outputs"
  default_resolution: "1024x1024"

deployment:
  bevy_assets_base: "/path/to/bevy/assets"  # Update for your project
  sprite_subdir: "sprites"
```

### Environment Variables

Override configuration with environment variables:

```bash
export DGX_PIXELS_CONFIG=/path/to/custom/config.yaml
export DGX_PIXELS_ZMQ_ENDPOINT=tcp://localhost:5555
export DGX_PIXELS_COMFYUI_URL=http://localhost:8188
export DGX_PIXELS_BEVY_ASSETS=/path/to/bevy/assets
```

## Usage

### Starting the Server

Start the MCP server with stdio transport (default):

```bash
python -m python.mcp_server.server
```

The server will listen for MCP tool calls via standard input/output.

### Using from Bevy (WS-14)

From Bevy, you'll use the `bevy_brp_mcp` plugin to call MCP tools:

```rust
// This will be implemented in WS-14 (Bevy Plugin Integration)
// For now, this is a forward reference

use bevy_brp_mcp::McpClient;

fn generate_sprite_system(mcp: Res<McpClient>) {
    let result = mcp.call_tool("generate_sprite", json!({
        "prompt": "medieval knight with sword",
        "style": "pixel_art",
        "resolution": "1024x1024"
    }));

    // Handle result...
}
```

## MCP Tools

### 1. generate_sprite

Generate a single pixel art sprite.

**Parameters:**

- `prompt` (string, required) - Text description of sprite
- `style` (string, optional) - Art style: "pixel_art" (default), "16bit", "8bit", "retro", "game_sprite"
- `resolution` (string, optional) - Output size: "512x512", "1024x1024" (default), "2048x2048"
- `steps` (int, optional) - Sampling steps: 10-100 (default: 30)
- `cfg_scale` (float, optional) - CFG scale: 1.0-20.0 (default: 7.5)
- `lora` (string, optional) - LoRA model name for custom style
- `output_path` (string, optional) - Custom output path

**Returns:**

```json
{
  "status": "success",
  "job_id": "abc123...",
  "output_path": "/path/to/output.png",
  "generation_time": 15.3
}
```

**Example:**

```json
{
  "prompt": "medieval knight character sprite",
  "style": "pixel_art",
  "resolution": "1024x1024",
  "steps": 30
}
```

### 2. generate_batch

Generate multiple sprites from a list of prompts.

**Parameters:**

- `prompts` (list[string], required) - List of text descriptions (max 20)
- `style` (string, optional) - Art style for all sprites
- `resolution` (string, optional) - Output resolution
- `steps` (int, optional) - Sampling steps per sprite
- `cfg_scale` (float, optional) - CFG scale value
- `output_dir` (string, optional) - Output directory

**Returns:**

```json
{
  "status": "success",
  "job_ids": ["job1", "job2", "job3"],
  "output_paths": ["/path/1.png", "/path/2.png", "/path/3.png"],
  "successful": 3,
  "failed": 0,
  "total_time": 45.2
}
```

**Example:**

```json
{
  "prompts": [
    "knight character sprite",
    "wizard character sprite",
    "archer character sprite"
  ],
  "style": "16bit",
  "resolution": "1024x1024"
}
```

### 3. deploy_to_bevy

Deploy a generated sprite to Bevy assets directory.

**Parameters:**

- `sprite_path` (string, required) - Path to generated sprite
- `bevy_assets_dir` (string, required) - Path to Bevy assets directory
- `sprite_name` (string, required) - Name for sprite (without .png)
- `update_manifest` (bool, optional) - Update asset manifest (default: true)

**Returns:**

```json
{
  "status": "success",
  "deployed_path": "/path/to/bevy/assets/sprites/player_knight.png",
  "manifest_updated": true
}
```

**Example:**

```json
{
  "sprite_path": "/tmp/knight_abc123.png",
  "bevy_assets_dir": "/home/user/game/assets",
  "sprite_name": "player_knight"
}
```

## Tool Schemas

### generate_sprite Schema

```json
{
  "name": "generate_sprite",
  "description": "Generate a pixel art sprite using SDXL + LoRA",
  "inputSchema": {
    "type": "object",
    "properties": {
      "prompt": {
        "type": "string",
        "description": "Text description of the sprite",
        "minLength": 3,
        "maxLength": 500
      },
      "style": {
        "type": "string",
        "enum": ["pixel_art", "16bit", "8bit", "retro", "game_sprite"],
        "default": "pixel_art"
      },
      "resolution": {
        "type": "string",
        "enum": ["512x512", "1024x1024", "2048x2048"],
        "default": "1024x1024"
      },
      "steps": {
        "type": "integer",
        "minimum": 10,
        "maximum": 100,
        "default": 30
      },
      "cfg_scale": {
        "type": "number",
        "minimum": 1.0,
        "maximum": 20.0,
        "default": 7.5
      }
    },
    "required": ["prompt"]
  }
}
```

## Testing

### Run Test Suite

Test all MCP tools:

```bash
python -m python.mcp_server.test_client
```

This will:
1. Test `generate_sprite` with various parameters
2. Test `generate_batch` with multiple prompts
3. Test `deploy_to_bevy` with file operations
4. Validate error handling

### Manual Testing with MCP Inspector

Use the MCP Inspector tool to interactively test the server:

```bash
# Install MCP Inspector
npm install -g @modelcontextprotocol/inspector

# Run inspector
mcp-inspector python -m python.mcp_server.server
```

## Integration with Backend Worker

The MCP server communicates with the Python backend worker (WS-10) via ZeroMQ:

1. **Connection**: REQ-REP socket on tcp://localhost:5555
2. **Protocol**: MessagePack serialization (see `python/workers/message_protocol.py`)
3. **Request Types**: GenerateRequest, CancelRequest, StatusRequest
4. **Response Types**: JobAcceptedResponse, JobCompleteResponse, ErrorResponse

### Backend Client Usage

The backend client is used internally by MCP tools:

```python
from python.mcp_server.backend_client import BackendClient

async with BackendClient("tcp://localhost:5555") as client:
    # Ping backend
    alive = await client.ping()

    # Get status
    status = await client.get_status()

    # Submit generation
    result = await client.generate_sprite(
        prompt="knight sprite",
        model="SDXL Base 1.0",
        size=[1024, 1024],
        steps=30,
        cfg_scale=7.5
    )
```

## Error Handling

All tools return structured error responses:

```json
{
  "status": "error",
  "error": "Validation error in prompt: Prompt cannot be empty"
}
```

Error categories:

1. **Validation Errors** - Invalid parameters
2. **Connection Errors** - Backend not available
3. **Timeout Errors** - Generation took too long
4. **Runtime Errors** - Unexpected failures

### Common Issues

**Backend connection failed:**
```
Error: Backend connection failed: Connection refused
```
Solution: Ensure Python backend worker is running (`python/workers/zmq_server.py`)

**ComfyUI unavailable:**
```
Error: ComfyUI server unavailable at http://localhost:8188
```
Solution: Start ComfyUI server or update config with correct URL

**Invalid resolution:**
```
Error: Validation error in resolution: Resolution must be one of: 512x512, 1024x1024, 2048x2048
```
Solution: Use one of the allowed resolutions

## Performance

Expected performance metrics:

- **Tool invocation latency**: <200ms (excluding generation time)
- **Single sprite generation**: 15-30 seconds (1024x1024, 30 steps)
- **Batch generation**: ~20 sprites per minute
- **Deployment**: <100ms per sprite
- **Backend connection**: <1ms (ZeroMQ)

## Logging

Configure logging in `config/mcp_config.yaml`:

```yaml
logging:
  level: "INFO"  # DEBUG, INFO, WARNING, ERROR
  log_tool_calls: true
  log_progress: true
```

Logs include:

- Tool invocations with parameters
- Backend communication status
- Generation progress
- Deployment operations
- Errors and exceptions

## Security Considerations

1. **Local Only**: Server binds to localhost by default
2. **Path Validation**: All file paths are validated before operations
3. **Input Sanitization**: Prompts and parameters are validated
4. **No Shell Access**: No shell commands executed
5. **Resource Limits**: Batch size limited to 20 sprites

## Next Steps (WS-14)

This MCP server is ready for Bevy integration:

1. Install `bevy_brp_mcp` plugin in Bevy project
2. Configure MCP client to connect to this server
3. Create Bevy systems that call MCP tools
4. Implement hot-reloading for generated sprites
5. Build game-specific asset management

See WS-14 documentation for Bevy integration details.

## Troubleshooting

### Server won't start

1. Check configuration file exists and is valid YAML
2. Verify Python 3.10+ is installed
3. Install all dependencies: `pip install -r requirements.txt`
4. Check ports 5555 and 5556 are not in use

### Generation fails

1. Ensure backend worker is running
2. Check ComfyUI is accessible
3. Verify workflows exist in `workflows/` directory
4. Check DGX-Spark GPU is available

### Deployment fails

1. Verify Bevy assets directory exists
2. Check file permissions (write access required)
3. Ensure sprite file exists before deploying
4. Validate sprite_name follows naming conventions

## API Reference

See tool docstrings in `server.py` for complete API reference.

## Contributing

When modifying the MCP server:

1. Update tool schemas if parameters change
2. Add tests to `test_client.py`
3. Update this README
4. Validate with MCP Inspector
5. Test integration with backend worker

## License

Part of the DGX-Pixels project. See root LICENSE file.
