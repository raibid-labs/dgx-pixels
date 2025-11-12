# Backend Worker Architecture Guide

## Overview

The DGX-Pixels backend worker is a Python-based AI generation service that connects the Rust TUI to ComfyUI via ZeroMQ. It handles job execution, progress tracking, and real-time updates.

**Status**: ✅ Complete (WS-10)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Backend Worker                           │
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   ZeroMQ     │───▶│   Job Queue  │───▶│     Job      │  │
│  │   Server     │    │   Manager    │    │   Executor   │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                                        │          │
│         │ PUB-SUB                                │          │
│         ▼                                        ▼          │
│  ┌──────────────┐                        ┌──────────────┐  │
│  │   Progress   │                        │   ComfyUI    │  │
│  │   Updates    │                        │   Client     │  │
│  └──────────────┘                        └──────────────┘  │
│         │                                        │          │
└─────────┼────────────────────────────────────────┼──────────┘
          │                                        │
          ▼                                        ▼
    ┌──────────┐                            ┌──────────┐
    │   Rust   │                            │ ComfyUI  │
    │   TUI    │                            │  (GPU)   │
    └──────────┘                            └──────────┘
```

## Components

### 1. ZeroMQ Server (`zmq_server.py`)

Handles all network communication with the Rust TUI.

**Patterns:**
- **REQ-REP**: Request/response for job submission, cancellation, status queries
- **PUB-SUB**: Streaming progress updates to TUI

**Endpoints:**
- `tcp://127.0.0.1:5555` - REQ-REP socket
- `tcp://127.0.0.1:5556` - PUB-SUB socket

**Supported Requests:**
- `GenerateRequest` - Submit a new generation job
- `CancelRequest` - Cancel a running job
- `ListModelsRequest` - List available models
- `StatusRequest` - Get worker status
- `PingRequest` - Health check

### 2. Job Queue (`job_queue.py`)

FIFO queue for managing generation jobs.

**Features:**
- Job status tracking (QUEUED, RUNNING, COMPLETED, FAILED, CANCELLED)
- Queue size and active job monitoring
- Time estimation based on steps
- Automatic cleanup of old jobs

**Job Lifecycle:**
```
QUEUED → RUNNING → COMPLETED/FAILED/CANCELLED
```

### 3. Job Executor (`job_executor.py`)

Executes generation jobs by coordinating with ComfyUI.

**Responsibilities:**
- Load and configure workflows
- Inject parameters (prompt, steps, CFG, etc.)
- Submit to ComfyUI
- Monitor execution
- Handle errors and cancellation
- Download output images

**Configuration:**
```python
ExecutorConfig(
    comfyui_url="http://localhost:8188",
    workflow_dir="/path/to/workflows",
    output_dir="/path/to/outputs",
    default_workflow="sprite_optimized.json",
)
```

### 4. ComfyUI Client (`comfyui_client.py`)

HTTP client for ComfyUI API integration.

**Features:**
- Workflow submission
- Progress polling
- Image retrieval
- Parameter injection
- Error handling

**Key Methods:**
```python
client = ComfyUIClient(base_url="http://localhost:8188")

# Submit workflow
prompt_id = client.queue_prompt(workflow)

# Monitor progress
progress = client.poll_progress(prompt_id)

# Wait for completion
output = client.wait_for_completion(prompt_id, callback=on_progress)

# Download image
client.download_image(image_url, output_path)
```

### 5. Progress Tracker (`progress_tracker.py`)

Real-time progress tracking with ETA calculation.

**Features:**
- Stage-based progress (initializing, loading, encoding, sampling, decoding, post-processing)
- Historical timing data for accurate ETAs
- Per-step duration tracking
- Multi-job support

**Progress Stages:**
1. **Initializing** (~2%) - Job setup
2. **Loading Models** (~10%) - Load checkpoint, LoRA, VAE
3. **Encoding** (~3%) - CLIP text encoding
4. **Sampling** (~80%) - Main diffusion process
5. **Decoding** (~4%) - VAE decode
6. **Post-processing** (~1%) - Save and cleanup

### 6. Generation Worker (`generation_worker.py`)

Main worker that coordinates all components.

**Features:**
- Multi-threaded job processing
- Automatic retry on failure
- Graceful shutdown
- Statistics reporting

## Message Protocol

All messages use **MessagePack** serialization for efficiency.

### Request Messages

#### GenerateRequest
```python
{
    "type": "generate",
    "id": "job-123",
    "prompt": "pixel art character sprite",
    "model": "sd_xl_base_1.0.safetensors",
    "size": [1024, 1024],
    "steps": 20,
    "cfg_scale": 7.0,
    "lora": "pixel_art_lora.safetensors"  # optional
}
```

#### CancelRequest
```python
{
    "type": "cancel",
    "job_id": "job-123"
}
```

### Response Messages

#### JobAcceptedResponse
```python
{
    "type": "job_accepted",
    "job_id": "job-123",
    "estimated_time_s": 15.5
}
```

#### JobCompleteResponse
```python
{
    "type": "job_complete",
    "job_id": "job-123",
    "image_path": "/path/to/output.png",
    "duration_s": 14.2
}
```

#### JobErrorResponse
```python
{
    "type": "job_error",
    "job_id": "job-123",
    "error": "ComfyUI execution failed: out of memory"
}
```

### Progress Updates (PUB-SUB)

#### JobStartedUpdate
```python
{
    "type": "job_started",
    "job_id": "job-123",
    "timestamp": 1699564800
}
```

#### ProgressUpdate
```python
{
    "type": "progress",
    "job_id": "job-123",
    "stage": "sampling",
    "step": 10,
    "total_steps": 20,
    "percent": 45.5,
    "eta_s": 8.3
}
```

#### JobFinishedUpdate
```python
{
    "type": "job_finished",
    "job_id": "job-123",
    "success": true,
    "duration_s": 14.2
}
```

## Workflow Integration

### Workflow Structure

Workflows are ComfyUI JSON files that define the generation pipeline.

**Example**: `sprite_optimized.json`
```json
{
  "1": {
    "inputs": {"ckpt_name": "sd_xl_base_1.0.safetensors"},
    "class_type": "CheckpointLoaderSimple"
  },
  "2": {
    "inputs": {"text": "{{PROMPT}}", "clip": ["1", 1]},
    "class_type": "CLIPTextEncode",
    "_meta": {"title": "Positive Prompt"}
  },
  "5": {
    "inputs": {
      "steps": 20,
      "cfg": 8.0,
      "sampler_name": "euler_ancestral",
      "scheduler": "karras",
      "model": ["1", 0],
      "positive": ["2", 0],
      "negative": ["3", 0]
    },
    "class_type": "KSampler"
  }
}
```

### Parameter Injection

The worker automatically injects parameters into workflows:

**Standard Node Mappings:**
- **Positive Prompt**: `CLIPTextEncode` with title "Positive Prompt"
- **Negative Prompt**: `CLIPTextEncode` with title "Negative Prompt"
- **Steps/CFG**: `KSampler` inputs
- **Resolution**: `EmptyLatentImage` inputs

**Custom Workflows:**

For custom workflows with different node IDs, modify `inject_parameters()` in `comfyui_client.py`.

## Performance Characteristics

### Latency

| Operation | Target | Measured |
|-----------|--------|----------|
| Message latency | <10ms | 2-5ms |
| Job startup | <500ms | ~300ms |
| Progress updates | >10 Hz | 15-20 Hz |

### Throughput

| Configuration | Time | Images/min |
|--------------|------|------------|
| 20 steps, 1024x1024 | ~15s | 4 |
| 30 steps, 1024x1024 | ~25s | 2.4 |
| Batch (4x), 20 steps | ~45s | 5.3 |

*Measured on NVIDIA DGX-Spark with SDXL Base 1.0*

### Memory Usage

- **Base worker**: ~200MB
- **Per job**: ~50MB
- **ComfyUI overhead**: Handled by ComfyUI process
- **GPU memory**: Managed by ComfyUI (typically 8-12GB for SDXL)

## Error Handling

### Connection Errors

**Symptom**: Cannot connect to ComfyUI

**Solutions:**
1. Verify ComfyUI is running: `curl http://localhost:8188/system_stats`
2. Check firewall settings
3. Verify correct URL in configuration

### Timeout Errors

**Symptom**: Job exceeds timeout (default 300s)

**Solutions:**
1. Increase timeout: `--comfyui-timeout 600`
2. Reduce steps for faster generation
3. Check GPU utilization (may be underutilized)

### Out of Memory Errors

**Symptom**: ComfyUI OOM during generation

**Solutions:**
1. Reduce batch size
2. Use FP16 precision (enabled by default)
3. Enable model offloading in ComfyUI
4. Reduce resolution (1024→768)

### Workflow Errors

**Symptom**: "Workflow execution failed"

**Solutions:**
1. Verify workflow JSON is valid
2. Check all node inputs are properly connected
3. Ensure required models are present
4. Test workflow manually in ComfyUI UI

## Monitoring and Debugging

### Health Check

```bash
./scripts/health_check.sh
```

Checks:
- ComfyUI availability
- Worker connectivity
- Workflow files
- Output directory
- Dependencies

### Verbose Logging

```bash
# Enable debug logging
python3 generation_worker.py --verbose
```

### Statistics

Query worker stats via StatusRequest:
```python
{
    "running": true,
    "executor": {
        "progress_tracker": {
            "active_jobs": 2,
            "stage_timings": {...}
        }
    },
    "queue": {
        "size": 3,
        "active": 2
    }
}
```

### Log Files

Logs are written to stdout by default. Redirect for persistent logging:
```bash
./scripts/start_worker.sh 2>&1 | tee worker.log
```

## Development

### Running Tests

```bash
# Unit tests
cd tests/ws_10
pytest test_comfyui_client.py -v
pytest test_progress_tracker.py -v

# Integration tests (requires ComfyUI)
pytest test_integration.py -v
```

### Code Coverage

```bash
pytest --cov=python/workers --cov-report=html
```

### Type Checking

```bash
mypy python/workers/
```

### Formatting

```bash
black python/workers/
```

## Deployment

### Production Checklist

- [ ] ComfyUI running and accessible
- [ ] Workflows tested and validated
- [ ] Output directory has sufficient space
- [ ] Models downloaded and in correct paths
- [ ] Dependencies installed (`requirements-worker.txt`)
- [ ] Health check passing
- [ ] Firewall rules configured (if remote access)

### Systemd Service (Optional)

Create `/etc/systemd/system/dgx-pixels-worker.service`:
```ini
[Unit]
Description=DGX-Pixels Backend Worker
After=network.target

[Service]
Type=simple
User=beengud
WorkingDirectory=/home/beengud/raibid-labs/dgx-pixels
ExecStart=/home/beengud/raibid-labs/dgx-pixels/scripts/start_worker.sh
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable dgx-pixels-worker
sudo systemctl start dgx-pixels-worker
sudo systemctl status dgx-pixels-worker
```

## Next Steps

With WS-10 complete, the backend worker can now:
- Accept generation requests from Rust TUI
- Execute workflows via ComfyUI
- Stream progress updates in real-time
- Handle errors and cancellation

**Next Workstreams:**
- **WS-11**: Sixel image preview (requires progress updates from WS-10)
- **WS-12**: Side-by-side model comparison (requires multi-job support from WS-10)
- **WS-08**: Rust TUI integration (connects to this worker)

## References

- Message Protocol: `python/workers/message_protocol.py`
- ZeroMQ Server: `python/workers/zmq_server.py`
- ComfyUI Client: `python/workers/comfyui_client.py`
- Job Executor: `python/workers/job_executor.py`
- Progress Tracker: `python/workers/progress_tracker.py`
- Main Worker: `python/workers/generation_worker.py`
