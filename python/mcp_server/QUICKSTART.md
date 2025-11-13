# FastMCP Server - Quick Start Guide

This guide will get you up and running with the DGX-Pixels FastMCP server in 5 minutes.

## Prerequisites

Before starting, ensure you have:

- ✅ Python 3.10+ installed
- ✅ DGX-Pixels Python backend worker running (WS-10)
- ✅ ComfyUI server running on port 8188
- ✅ Git repository checked out

## Step 1: Install Dependencies

```bash
cd /home/beengud/raibid-labs/dgx-pixels
pip install -r python/mcp_server/requirements.txt
```

Or install manually:

```bash
pip install fastmcp pyzmq pyyaml aiohttp msgpack
```

## Step 2: Configure the Server

Edit `config/mcp_config.yaml` and update the Bevy assets path:

```yaml
deployment:
  bevy_assets_base: "/path/to/your/bevy/project/assets"  # Update this!
```

All other defaults should work if you're running:
- Backend worker on `tcp://localhost:5555`
- ComfyUI on `http://localhost:8188`

## Step 3: Start Backend Services

In separate terminals:

**Terminal 1: Python Backend Worker**
```bash
cd /home/beengud/raibid-labs/dgx-pixels
python -m python.workers.zmq_server
```

**Terminal 2: ComfyUI**
```bash
# Start ComfyUI (adjust path as needed)
cd /path/to/ComfyUI
python main.py
```

## Step 4: Start the MCP Server

In a new terminal:

```bash
cd /home/beengud/raibid-labs/dgx-pixels
python -m python.mcp_server
```

You should see:

```
[2025-11-12 22:30:00] INFO - dgx-pixels-mcp - Loaded configuration from default location
[2025-11-12 22:30:00] INFO - dgx-pixels-mcp - Initialized dgx-pixels v0.1.0
[2025-11-12 22:30:00] INFO - dgx-pixels-mcp - Starting FastMCP server...
[2025-11-12 22:30:00] INFO - dgx-pixels-mcp - Backend endpoint: tcp://localhost:5555
[2025-11-12 22:30:00] INFO - dgx-pixels-mcp - ComfyUI URL: http://localhost:8188
```

## Step 5: Test the Server

In another terminal, run the test suite:

```bash
cd /home/beengud/raibid-labs/dgx-pixels
python -m python.mcp_server.test_client
```

This will test all three MCP tools:
- `generate_sprite` - Single sprite generation
- `generate_batch` - Batch generation
- `deploy_to_bevy` - Deployment to assets

## Step 6: Verify MCP Tools

You can also test individual tools using the MCP Inspector:

```bash
# Install MCP Inspector (if not already installed)
npm install -g @modelcontextprotocol/inspector

# Run inspector
mcp-inspector python -m python.mcp_server
```

This opens a web UI where you can:
1. See all available tools
2. Call tools with parameters
3. View responses in real-time

## Common Issues

### "Backend connection failed"

**Problem:** Backend worker not running

**Solution:**
```bash
# Start backend worker
python -m python.workers.zmq_server
```

### "ComfyUI unavailable"

**Problem:** ComfyUI not running or wrong URL

**Solution:**
```bash
# Check ComfyUI is running
curl http://localhost:8188/system_stats

# If not, start ComfyUI
cd /path/to/ComfyUI && python main.py

# Or update config with correct URL
export DGX_PIXELS_COMFYUI_URL=http://your-comfyui:8188
```

### "Config file not found"

**Problem:** Configuration file missing

**Solution:**
```bash
# Check config exists
ls config/mcp_config.yaml

# Or specify custom config
export DGX_PIXELS_CONFIG=/path/to/your/config.yaml
```

## Next Steps

Now that your MCP server is running, you can:

1. **Integrate with Bevy** (WS-14)
   - Install `bevy_brp_mcp` plugin
   - Configure Bevy to connect to this MCP server
   - Call MCP tools from Bevy systems

2. **Generate Your First Sprite**
   ```python
   # Using the test client as a template
   result = await tools.generate_sprite(
       prompt="your game character here",
       style="pixel_art",
       resolution="1024x1024"
   )
   ```

3. **Deploy to Bevy**
   ```python
   result = await tools.deploy_to_bevy(
       sprite_path=generated_sprite_path,
       bevy_assets_dir="/path/to/bevy/assets",
       sprite_name="my_character"
   )
   ```

4. **Customize Workflows**
   - Edit workflows in `workflows/` directory
   - Create custom LoRA models (see WS-06)
   - Train on your game's art style

## Architecture Overview

```
Your Tool/Bevy
       ↓
FastMCP Server (port: stdio)
       ↓
Python Backend Worker (port: 5555)
       ↓
ComfyUI (port: 8188)
       ↓
NVIDIA DGX-Spark GPU
```

## Environment Variables

Override configuration with environment variables:

```bash
export DGX_PIXELS_CONFIG=/custom/config.yaml
export DGX_PIXELS_ZMQ_ENDPOINT=tcp://remote:5555
export DGX_PIXELS_COMFYUI_URL=http://remote:8188
export DGX_PIXELS_BEVY_ASSETS=/game/assets
```

## Performance Tips

1. **Use LoRA models** for consistent style (train in WS-06)
2. **Batch generations** when creating multiple sprites
3. **Lower steps** (20-25) for faster prototyping
4. **Use 512x512** for very fast iterations
5. **Enable hot-reload** in Bevy for instant sprite updates

## Help & Support

- **Issues**: See `python/mcp_server/README.md` for detailed troubleshooting
- **Architecture**: See `docs/07-rust-python-architecture.md`
- **MCP Protocol**: See https://modelcontextprotocol.io
- **Logs**: Check console output for detailed error messages

## Production Checklist

Before deploying to production:

- [ ] Configure proper Bevy assets path
- [ ] Set up proper logging (log files, not just console)
- [ ] Configure rate limiting (if exposed to network)
- [ ] Test all workflows with your game's art style
- [ ] Train custom LoRA models for consistent style
- [ ] Set up monitoring for backend health
- [ ] Configure proper error handling in Bevy

## What's Next?

See **WS-14: Bevy Plugin Integration** for connecting this MCP server to your Bevy game engine.

The MCP server provides the bridge - now you need to connect Bevy to it!
