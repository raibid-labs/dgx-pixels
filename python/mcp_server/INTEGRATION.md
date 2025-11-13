# MCP Server Integration Guide

This document explains how the FastMCP Server integrates with the DGX-Pixels ecosystem.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Bevy Game Engine                        │
│                  (WS-14 - Future Work)                      │
└───────────────────────────┬─────────────────────────────────┘
                            │ MCP Protocol (stdio)
                            │
┌───────────────────────────▼─────────────────────────────────┐
│               FastMCP Server (WS-13 - This Module)          │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ server.py   │  │  tools.py    │  │ backend_     │      │
│  │             │──│              │──│ client.py    │      │
│  │ MCP Tools   │  │ Validation   │  │ ZeroMQ Client│      │
│  └─────────────┘  └──────────────┘  └──────┬───────┘      │
└─────────────────────────────────────────────┼──────────────┘
                                              │ ZeroMQ REQ-REP
                                              │ tcp://localhost:5555
┌─────────────────────────────────────────────▼──────────────┐
│        Python Backend Worker (WS-10 - Existing)            │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │zmq_server.py│──│comfyui_      │──│job_queue.py  │     │
│  │             │  │client.py     │  │              │     │
│  └─────────────┘  └──────┬───────┘  └──────────────┘     │
└──────────────────────────┼────────────────────────────────┘
                           │ HTTP API
┌──────────────────────────▼────────────────────────────────┐
│                    ComfyUI Server                          │
│              http://localhost:8188                         │
└───────────────────────────┬────────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────┐
│               NVIDIA DGX-Spark GPU                          │
│     SDXL + LoRA Models → Pixel Art Generation              │
└────────────────────────────────────────────────────────────┘
```

## Integration Points

### 1. Bevy Game Engine (WS-14 - Future)

**Protocol**: Model Context Protocol (MCP)
**Transport**: stdio (standard input/output)
**Direction**: Bevy → FastMCP Server

**Bevy Side** (to be implemented in WS-14):
```rust
use bevy_brp_mcp::McpClient;

fn setup(mut commands: Commands) {
    let mcp_client = McpClient::new("python -m python.mcp_server");
    commands.insert_resource(mcp_client);
}

fn generate_sprite_system(mcp: Res<McpClient>) {
    let result = mcp.call_tool("generate_sprite", json!({
        "prompt": "knight character",
        "style": "pixel_art"
    }));
}
```

**MCP Server Side** (already implemented):
```python
# server.py
@mcp.tool()
async def generate_sprite(prompt: str, style: str = "pixel_art", ...):
    # Validate parameters
    # Call backend via ZeroMQ
    # Return structured response
```

**Data Flow**:
1. Bevy calls MCP tool via stdio
2. FastMCP deserializes request
3. Tool validates parameters
4. Tool calls backend via ZeroMQ
5. Backend processes request
6. Response flows back to Bevy

### 2. Python Backend Worker (WS-10 - Existing)

**Protocol**: ZeroMQ REQ-REP pattern + MessagePack serialization
**Endpoint**: tcp://localhost:5555
**Direction**: Bidirectional (FastMCP ↔ Backend)

**FastMCP Side**:
```python
# backend_client.py
from message_protocol import GenerateRequest, serialize, deserialize_response

async def generate_sprite(self, prompt: str, model: str, ...):
    request = GenerateRequest(
        id=job_id,
        prompt=prompt,
        model=model,
        size=[width, height],
        steps=steps,
        cfg_scale=cfg_scale
    )

    data = serialize(request)
    await self.socket.send(data)

    response_data = await self.socket.recv()
    response = deserialize_response(response_data)

    return response
```

**Backend Side** (python/workers/zmq_server.py):
```python
# Receives serialized request via ZeroMQ
request = deserialize_request(data)

# Process request
job = self.job_queue.add_job(...)
response = JobAcceptedResponse(job_id=job.job_id, ...)

# Send serialized response
response_data = serialize(response)
self.rep_socket.send(response_data)
```

**Message Protocol** (python/workers/message_protocol.py):
- Request types: GenerateRequest, CancelRequest, StatusRequest, ListModelsRequest
- Response types: JobAcceptedResponse, JobCompleteResponse, JobErrorResponse
- Serialization: MessagePack (binary format)

### 3. ComfyUI Server (Indirect - via Backend)

**Protocol**: HTTP REST API
**Endpoint**: http://localhost:8188
**Direction**: Backend → ComfyUI

**FastMCP does NOT directly communicate with ComfyUI**. Instead:
1. FastMCP sends request to Backend Worker
2. Backend Worker uses comfyui_client.py to call ComfyUI
3. ComfyUI processes workflow and generates sprite
4. Backend Worker retrieves result
5. Backend Worker sends response back to FastMCP

**Why this design?**
- Separation of concerns (MCP server doesn't need GPU knowledge)
- Backend can handle multiple requests (queue management)
- Backend manages ComfyUI lifecycle
- Easier to scale (multiple MCP servers → one backend)

### 4. Configuration System

**File**: config/mcp_config.yaml
**Loaded by**: python/mcp_server/config_loader.py
**Used by**: All MCP server components

**Configuration Sections**:

```yaml
# MCP Server settings
mcp_server:
  name: "dgx-pixels"
  transports: ["stdio", "sse"]

# Backend connection (WS-10 integration point)
backend:
  zmq_endpoint: "tcp://localhost:5555"
  comfyui_url: "http://localhost:8188"

# Generation defaults (used by tools)
generation:
  default_workflow: "sprite_optimized.json"
  output_dir: "/path/to/outputs"

# Bevy integration (WS-14 integration point)
deployment:
  bevy_assets_base: "/path/to/bevy/assets"
  sprite_subdir: "sprites"
  manifest_file: "asset_manifest.json"

# Validation rules (enforced by tools)
validation:
  allowed_resolutions: ["512x512", "1024x1024", "2048x2048"]
  allowed_styles: ["pixel_art", "16bit", "8bit", "retro"]
```

**Environment Variable Overrides**:
```bash
export DGX_PIXELS_ZMQ_ENDPOINT=tcp://remote:5555
export DGX_PIXELS_COMFYUI_URL=http://remote:8188
export DGX_PIXELS_BEVY_ASSETS=/game/assets
```

## Data Flow Examples

### Example 1: Generate Single Sprite

```
[Bevy] → generate_sprite(prompt="knight")
   ↓
[MCP Server] → tools.generate_sprite()
   ↓ validate parameters
   ↓
[Backend Client] → GenerateRequest via ZeroMQ
   ↓
[Backend Worker] → Add to job queue
   ↓
[ComfyUI Client] → Submit workflow
   ↓
[ComfyUI] → Generate sprite (SDXL + LoRA)
   ↓
[ComfyUI] → Save to outputs/
   ↓
[Backend Worker] → JobCompleteResponse
   ↓
[Backend Client] → Deserialize response
   ↓
[MCP Server] → Return {status: "success", path: "..."}
   ↓
[Bevy] → Receive sprite path
```

**Timing**: 15-30 seconds total (mostly ComfyUI generation)

### Example 2: Deploy Sprite to Bevy

```
[Bevy] → deploy_to_bevy(sprite_path="/tmp/knight.png", ...)
   ↓
[MCP Server] → tools.deploy_to_bevy()
   ↓ validate sprite exists
   ↓ validate bevy_assets_dir exists
   ↓
[File System] → Copy sprite to assets/sprites/
   ↓
[File System] → Update asset_manifest.json
   ↓
[MCP Server] → Return {status: "success", deployed_path: "..."}
   ↓
[Bevy] → Hot-reload sprite (Bevy's AssetServer)
```

**Timing**: <100ms

### Example 3: Batch Generation

```
[Bevy] → generate_batch(prompts=["knight", "wizard", "archer"])
   ↓
[MCP Server] → tools.generate_batch()
   ↓ validate each prompt
   ↓
[Backend Client] → Send 3x GenerateRequest (sequential)
   ↓
[Backend Worker] → Queue all 3 jobs
   ↓
[ComfyUI] → Generate sprite 1
[ComfyUI] → Generate sprite 2  (after job 1 completes)
[ComfyUI] → Generate sprite 3  (after job 2 completes)
   ↓
[MCP Server] → Return {successful: 3, job_ids: [...]}
   ↓
[Bevy] → Receive all sprite paths
```

**Timing**: 45-90 seconds (sequential processing)

## Error Handling Flow

### Validation Error

```
[Bevy] → generate_sprite(prompt="")  # Empty prompt
   ↓
[MCP Server] → tools._validate_prompt()
   ↓ raise ValidationError("prompt", "Prompt cannot be empty")
   ↓
[MCP Server] → Catch exception, return error response
   ↓
[Bevy] ← {status: "error", error: "Validation error in prompt: ..."}
```

**Timing**: <1ms (no backend call)

### Backend Connection Error

```
[Bevy] → generate_sprite(prompt="knight")
   ↓
[MCP Server] → tools.generate_sprite()
   ↓
[Backend Client] → ZeroMQ send
   ↓ × Backend not running (connection refused)
   ↓
[Backend Client] → raise ConnectionError("Backend unavailable")
   ↓
[MCP Server] → Catch exception, return error response
   ↓
[Bevy] ← {status: "error", error: "Backend connection failed: ..."}
```

**Timing**: <100ms (timeout + retry)

### Generation Timeout

```
[Bevy] → generate_sprite(prompt="knight", steps=100)
   ↓
[MCP Server] → tools.generate_sprite()
   ↓
[Backend Client] → Send GenerateRequest
   ↓
[Backend Worker] → Accept job
   ↓
[ComfyUI] → Start generation (slow, 100 steps)
   ↓ (5 minutes pass, exceeds timeout)
   ↓
[Backend Client] → Timeout, cancel request
   ↓
[MCP Server] → Return timeout error
   ↓
[Bevy] ← {status: "error", error: "Generation timed out after 300s"}
```

**Timing**: 300s (configurable timeout)

## State Management

### MCP Server State

**Stateless**: Each tool call is independent
- No session state
- No request caching
- No connection pooling (recreates per request)

**Why stateless?**
- Simpler to implement
- No memory leaks
- Easier to scale (multiple server instances)
- MCP protocol is request-response (no sessions)

### Backend Worker State (WS-10)

**Stateful**: Maintains job queue and history
- Job queue (pending, running, completed)
- Progress tracking per job
- Historical timing data for ETA calculation

**State stored in**:
- `job_queue.py`: Job queue and status
- `progress_tracker.py`: Progress and ETA data
- Memory only (not persisted to disk)

## Threading and Concurrency

### MCP Server (This Module)

**Model**: Async/await with asyncio

```python
async def generate_sprite(...):
    # Non-blocking I/O
    await backend_client.generate_sprite(...)
```

**Concurrency**:
- Single Python event loop
- Multiple concurrent MCP tool calls supported
- Limited by backend worker capacity (not MCP server)

### Backend Worker (WS-10)

**Model**: Single-threaded event loop

```python
# zmq_server.py main loop
while self.running:
    data = self.rep_socket.recv()  # Blocking
    response = self._handle_request(request)
    self.rep_socket.send(response_data)
```

**Concurrency**:
- One request at a time (REQ-REP pattern)
- Job queue handles multiple jobs (sequential processing)
- No parallel generation (ComfyUI limitation)

**Implication**: If Bevy makes 2 concurrent MCP calls:
1. First call processes immediately
2. Second call blocks until first completes
3. Both requests queued in backend worker

## Performance Optimization

### Current Performance

**MCP Server**:
- Tool invocation: <10ms
- Parameter validation: <1ms
- ZeroMQ communication: <1ms
- Total overhead: <50ms

**Backend Worker**:
- Queue management: <10ms
- ComfyUI API call: <50ms
- Generation: 15-30s (SDXL + LoRA)

**Bottleneck**: ComfyUI generation time (15-30s per sprite)

### Optimization Strategies (Future Work)

**1. Parallel Batch Processing**
```python
# Current: Sequential
for prompt in prompts:
    await backend_client.generate_sprite(prompt)

# Future: Parallel (requires multiple backend workers)
tasks = [backend_client.generate_sprite(p) for p in prompts]
await asyncio.gather(*tasks)
```

**2. Response Caching**
```python
# Cache frequently-used sprites
cache_key = f"{prompt}:{style}:{resolution}"
if cache_key in cache:
    return cache[cache_key]
```

**3. Workflow Pre-loading**
```python
# Pre-load workflows at startup
self.workflows = {
    "pixel_art": load_workflow("pixel_art_workflow.json"),
    "16bit": load_workflow("16bit_workflow.json")
}
```

**4. Connection Pooling**
```python
# Maintain persistent ZeroMQ connections
self.zmq_pool = ConnectionPool(size=5)
socket = self.zmq_pool.acquire()
```

## Monitoring and Observability

### Logging

**Levels**:
- DEBUG: All tool calls, parameters, responses
- INFO: Tool invocations, successes, errors
- WARNING: Validation failures, retries
- ERROR: Backend failures, timeouts

**Configuration**:
```yaml
logging:
  level: "INFO"
  log_tool_calls: true
  log_progress: true
```

**Example Logs**:
```
[2025-11-12 22:30:15] INFO - generate_sprite called: prompt='knight', style=pixel_art
[2025-11-12 22:30:15] INFO - Backend request sent: job_id=abc123
[2025-11-12 22:30:30] INFO - Sprite generated successfully: job_id=abc123
```

### Health Checks

**Endpoint**: `ping` tool (not exposed as MCP tool, internal use)

```python
# backend_client.py
async def ping(self) -> bool:
    try:
        response = await self._send_request(PingRequest())
        return isinstance(response, PongResponse)
    except:
        return False
```

**Usage**:
```python
# Check if backend is alive before processing
if not await backend_client.ping():
    raise ConnectionError("Backend unavailable")
```

## Security Considerations

### Threat Model

**Trusted Environment Assumptions**:
- MCP server runs on localhost
- Bevy game engine is trusted (no malicious input)
- Backend worker is trusted
- ComfyUI is trusted

**Protections**:
1. Input validation (prompt length, parameter ranges)
2. Path validation (no directory traversal)
3. Resource limits (batch size max 20)
4. No shell command execution
5. Localhost-only binding

**NOT Protected Against**:
- Network attacks (server not exposed to network)
- Authentication bypass (no authentication required)
- DoS attacks (no rate limiting)
- Prompt injection (passes prompt to SDXL as-is)

### Recommendations for Production

1. **Network Isolation**: Keep MCP server localhost-only
2. **Rate Limiting**: Implement in Bevy layer
3. **Input Sanitization**: Additional prompt filtering if needed
4. **Resource Limits**: Enforce batch size, timeout limits
5. **Monitoring**: Alert on excessive errors, timeouts

## Troubleshooting Integration Issues

### Issue: "Backend connection failed"

**Symptoms**:
```json
{"status": "error", "error": "Backend connection failed: Connection refused"}
```

**Diagnosis**:
1. Check backend worker is running: `ps aux | grep zmq_server`
2. Check ZeroMQ port: `netstat -an | grep 5555`
3. Check config: `cat config/mcp_config.yaml | grep zmq_endpoint`

**Solution**:
```bash
# Start backend worker
python -m python.workers.zmq_server
```

### Issue: "ComfyUI unavailable"

**Symptoms**: Backend accepts request but fails generation

**Diagnosis**:
1. Check ComfyUI running: `curl http://localhost:8188/system_stats`
2. Check logs: Backend worker logs should show ComfyUI errors

**Solution**:
```bash
# Start ComfyUI
cd /path/to/ComfyUI && python main.py
```

### Issue: "Validation error in prompt"

**Symptoms**:
```json
{"status": "error", "error": "Validation error in prompt: Prompt too short"}
```

**Diagnosis**: Check validation rules in config

**Solution**: Ensure prompt meets requirements:
- Length: 3-500 characters
- Style: Must be in allowed list
- Resolution: Must be in allowed list

### Issue: "Generation times out"

**Symptoms**: Request takes >5 minutes, then fails

**Diagnosis**:
1. Check timeout config: `config/mcp_config.yaml → backend.timeout_s`
2. Check ComfyUI logs for stuck generation

**Solution**:
```yaml
# Increase timeout
backend:
  timeout_s: 600  # 10 minutes
```

## Testing Integration

### Unit Tests

Test individual components in isolation:

```python
# Test config loader
config = load_config()
assert config.backend.zmq_endpoint == "tcp://localhost:5555"

# Test validation
tools = MCPTools(config)
tools._validate_prompt("valid prompt")  # Should not raise

# Test backend client
client = BackendClient("tcp://localhost:5555")
assert await client.ping() == True
```

### Integration Tests

Test component interactions:

```python
# Test MCP tool → Backend Client → Backend Worker
result = await tools.generate_sprite(
    prompt="knight",
    style="pixel_art"
)
assert result["status"] == "success"
assert "job_id" in result
```

### End-to-End Tests (WS-14)

Test full system with Bevy:

```rust
// Bevy integration test
let result = mcp_client.call_tool("generate_sprite", ...);
assert!(result.is_ok());

// Check sprite deployed
let sprite_path = result.unwrap()["deployed_path"];
assert!(Path::new(&sprite_path).exists());
```

## Next Steps

### For WS-14 (Bevy Plugin Integration)

1. **Install bevy_brp_mcp** in Bevy project
2. **Configure MCP client** to run FastMCP server
3. **Create Bevy systems** that call MCP tools
4. **Implement hot-reloading** for deployed sprites
5. **Build asset management** UI/systems
6. **Test end-to-end** workflow

### For Production Deployment

1. **Configure paths** for production environment
2. **Set up monitoring** (logs, metrics, alerts)
3. **Implement error recovery** (auto-restart on failure)
4. **Load test** with realistic workloads
5. **Document** deployment procedures
6. **Train** game developers on usage

## Resources

- FastMCP Documentation: https://github.com/jlowin/fastmcp
- MCP Specification: https://modelcontextprotocol.io
- ZeroMQ Guide: https://zeromq.org/
- Backend Worker (WS-10): `python/workers/README.md`
- Bevy Integration (WS-14): TBD

---

**Version**: 1.0
**Last Updated**: 2025-11-12
**Maintained By**: DGX-Pixels Team
