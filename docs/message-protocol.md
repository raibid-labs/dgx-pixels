# Message Protocol Specification v1.0

## Protocol Overview

**Version**: 1.0.0
**Serialization**: MessagePack (binary format)
**Transport**: ZeroMQ (REQ-REP + PUB-SUB patterns)
**Encoding**: UTF-8 for strings

## Message Categories

1. **Request Messages**: TUI → Backend (REQ-REP)
2. **Response Messages**: Backend → TUI (REQ-REP)
3. **Progress Updates**: Backend → TUI (PUB-SUB)

## Request Messages

All requests have a `type` field that identifies the message type.

### Generate Request

Initiates a new sprite generation job.

```json
{
  "type": "generate",
  "id": "job-001",
  "prompt": "16-bit knight sprite, pixel art style",
  "model": "sdxl-base",
  "lora": "pixelart",  // Optional
  "size": [1024, 1024],
  "steps": 30,
  "cfg_scale": 7.5
}
```

**Fields:**
- `id` (string): Unique job identifier
- `prompt` (string): Text description of desired sprite
- `model` (string): Base model name
- `lora` (string, optional): LoRA model name
- `size` (array[int, int]): Output dimensions [width, height]
- `steps` (int): Number of diffusion steps
- `cfg_scale` (float): Classifier-free guidance scale

### Cancel Request

Cancels a running or queued job.

```json
{
  "type": "cancel",
  "job_id": "job-001"
}
```

**Fields:**
- `job_id` (string): Job to cancel

### List Models Request

Retrieves list of available models.

```json
{
  "type": "list_models"
}
```

No additional fields.

### Status Request

Gets backend status information.

```json
{
  "type": "status"
}
```

No additional fields.

### Ping Request

Health check / connectivity test.

```json
{
  "type": "ping"
}
```

No additional fields.

## Response Messages

All responses have a `type` field that identifies the message type.

### Job Accepted Response

Confirms job was queued.

```json
{
  "type": "job_accepted",
  "job_id": "job-001",
  "estimated_time_s": 3.5
}
```

**Fields:**
- `job_id` (string): Job identifier
- `estimated_time_s` (float): Estimated generation time in seconds

### Job Complete Response

Job finished successfully.

```json
{
  "type": "job_complete",
  "job_id": "job-001",
  "image_path": "/output/sprite-001.png",
  "duration_s": 3.2
}
```

**Fields:**
- `job_id` (string): Job identifier
- `image_path` (string): Path to generated image
- `duration_s` (float): Actual generation time in seconds

### Job Error Response

Job failed with error.

```json
{
  "type": "job_error",
  "job_id": "job-001",
  "error": "Model not found: sdxl-custom"
}
```

**Fields:**
- `job_id` (string): Job identifier
- `error` (string): Error message

### Job Cancelled Response

Job was cancelled.

```json
{
  "type": "job_cancelled",
  "job_id": "job-001"
}
```

**Fields:**
- `job_id` (string): Job identifier

### Model List Response

List of available models.

```json
{
  "type": "model_list",
  "models": [
    {
      "name": "SDXL Base 1.0",
      "path": "/models/checkpoints/sd_xl_base_1.0.safetensors",
      "model_type": "checkpoint",
      "size_mb": 6500
    },
    {
      "name": "Pixel Art LoRA",
      "path": "/models/loras/pixelart.safetensors",
      "model_type": "lora",
      "size_mb": 144
    }
  ]
}
```

**Model Info Fields:**
- `name` (string): Human-readable model name
- `path` (string): File system path
- `model_type` (enum): "checkpoint" | "lora" | "vae"
- `size_mb` (int): Model file size in megabytes

### Status Info Response

Backend status information.

```json
{
  "type": "status_info",
  "version": "1.0.0",
  "queue_size": 3,
  "active_jobs": 1,
  "uptime_s": 3600
}
```

**Fields:**
- `version` (string): Protocol version
- `queue_size` (int): Number of queued jobs
- `active_jobs` (int): Number of running jobs
- `uptime_s` (int): Server uptime in seconds

### Pong Response

Response to ping.

```json
{
  "type": "pong"
}
```

No additional fields.

### Error Response

Generic error response.

```json
{
  "type": "error",
  "message": "Internal server error"
}
```

**Fields:**
- `message` (string): Error description

## Progress Updates (PUB-SUB)

These messages are published asynchronously via the PUB-SUB channel.

### Job Started Update

Job has begun processing.

```json
{
  "type": "job_started",
  "job_id": "job-001",
  "timestamp": 1699564800
}
```

**Fields:**
- `job_id` (string): Job identifier
- `timestamp` (int): Unix timestamp

### Progress Update

Generation progress information.

```json
{
  "type": "progress",
  "job_id": "job-001",
  "stage": "sampling",
  "step": 15,
  "total_steps": 30,
  "percent": 50.0,
  "eta_s": 1.8
}
```

**Fields:**
- `job_id` (string): Job identifier
- `stage` (enum): Generation stage (see below)
- `step` (int): Current step number
- `total_steps` (int): Total steps for this job
- `percent` (float): Completion percentage (0-100)
- `eta_s` (float): Estimated time to completion in seconds

**Generation Stages:**
- `initializing`: Setting up generation
- `loading_models`: Loading model files
- `encoding`: Encoding prompt
- `sampling`: Diffusion sampling loop
- `decoding`: Decoding latents to image
- `post_processing`: Color quantization, upscaling

### Preview Update

Intermediate preview image available.

```json
{
  "type": "preview",
  "job_id": "job-001",
  "image_path": "/tmp/preview-001.png",
  "step": 10
}
```

**Fields:**
- `job_id` (string): Job identifier
- `image_path` (string): Path to preview image
- `step` (int): Step number when preview was generated

### Job Finished Update

Job completed (success or failure).

```json
{
  "type": "job_finished",
  "job_id": "job-001",
  "success": true,
  "duration_s": 3.2
}
```

**Fields:**
- `job_id` (string): Job identifier
- `success` (bool): True if completed successfully
- `duration_s` (float): Total generation time in seconds

## Message Size Limits

- Maximum prompt length: 10,000 characters
- Maximum message size: 1 MB
- Recommended message size: <10 KB

Messages exceeding these limits may be rejected or truncated.

## Error Codes

The protocol does not use numeric error codes. Errors are communicated via the `error` field in `JobErrorResponse` or `ErrorResponse`.

Common error messages:
- "Model not found: {name}"
- "Invalid parameter: {field}"
- "Queue full"
- "Backend busy"
- "Communication error"

## Protocol Extensions

Future versions may add:
- Batch generation requests
- Streaming image data
- Model download/update messages
- Training status updates

Version compatibility: Clients should check `version` in `StatusInfoResponse` and gracefully handle unknown message types.

## MessagePack Encoding Notes

- Strings: UTF-8 encoded
- Arrays: Fixed-length MessagePack arrays
- Tuples: Encoded as arrays (Rust `(u32, u32)` → `[1024, 1024]`)
- Optional fields: Omitted if None/null
- Enums: Tagged with `type` field (adjacently tagged)

## Example Message Exchange

**Client sends:**
```
[MessagePack binary] → { "type": "generate", "id": "job-001", ... }
```

**Server responds:**
```
[MessagePack binary] ← { "type": "job_accepted", "job_id": "job-001", ... }
```

**Server publishes updates:**
```
[MessagePack binary] → { "type": "job_started", "job_id": "job-001", ... }
[MessagePack binary] → { "type": "progress", "job_id": "job-001", "step": 1, ... }
[MessagePack binary] → { "type": "progress", "job_id": "job-001", "step": 2, ... }
...
[MessagePack binary] → { "type": "job_finished", "job_id": "job-001", ... }
```

**Client sends:**
```
[MessagePack binary] → { "type": "ping" }
```

**Server responds:**
```
[MessagePack binary] ← { "type": "pong" }
```
