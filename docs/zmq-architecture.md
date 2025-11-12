# ZeroMQ IPC Architecture

## Overview

DGX-Pixels uses ZeroMQ for high-performance inter-process communication (IPC) between the Rust TUI frontend and Python AI backend. This architecture enables sub-10ms latency for request/response cycles and supports real-time progress updates during generation.

## Architecture Design

### Communication Patterns

We implement two ZeroMQ patterns:

1. **REQ-REP (Request-Reply)**: TUI sends requests, backend responds
   - Address: `tcp://127.0.0.1:5555` (default)
   - Use case: Generate sprites, cancel jobs, list models, get status
   - Synchronous: Each request gets exactly one response

2. **PUB-SUB (Publish-Subscribe)**: Backend publishes updates, TUI subscribes
   - Address: `tcp://127.0.0.1:5556` (default)
   - Use case: Progress updates, job status changes, previews
   - Asynchronous: TUI receives updates without blocking

### Message Serialization

**Format**: MessagePack (MsgPack)
- Binary serialization format
- Smaller than JSON (30-50% reduction)
- Faster than JSON (2-3x)
- Cross-language support (Rust + Python)

**Protocol Version**: 1.0.0

### Threading Model

**Rust TUI**:
- Main thread: UI rendering (60+ FPS target)
- REQ thread: Sends requests, receives responses
- SUB thread: Receives progress updates
- Uses `std::sync::mpsc` channels for cross-thread communication

**Python Backend**:
- Main thread: ZeroMQ event loop
- REP socket: Handles requests
- PUB socket: Publishes updates
- Job queue: FIFO queue for generation tasks

## Message Types

### Request Messages (TUI → Backend)

```rust
enum Request {
    Generate {
        id: String,
        prompt: String,
        model: String,
        lora: Option<String>,
        size: (u32, u32),
        steps: u32,
        cfg_scale: f32,
    },
    Cancel { job_id: String },
    ListModels,
    Status,
    Ping,
}
```

### Response Messages (Backend → TUI)

```rust
enum Response {
    JobAccepted { job_id: String, estimated_time_s: f32 },
    JobComplete { job_id: String, image_path: String, duration_s: f32 },
    JobError { job_id: String, error: String },
    JobCancelled { job_id: String },
    ModelList { models: Vec<ModelInfo> },
    StatusInfo { version: String, queue_size: u32, active_jobs: u32, uptime_s: u64 },
    Pong,
    Error { message: String },
}
```

### Progress Updates (Backend → TUI via PUB-SUB)

```rust
enum ProgressUpdate {
    JobStarted { job_id: String, timestamp: u64 },
    Progress {
        job_id: String,
        stage: GenerationStage,
        step: u32,
        total_steps: u32,
        percent: f32,
        eta_s: f32,
    },
    Preview { job_id: String, image_path: String, step: u32 },
    JobFinished { job_id: String, success: bool, duration_s: f32 },
}
```

## Performance Characteristics

### Latency

| Metric | Target | Measured (ARM64 DGX-Spark) |
|--------|--------|----------------------------|
| Message serialization | <0.1ms | ~0.05ms |
| Network round-trip (local) | <1ms | ~0.5-2ms |
| P95 request/response | <10ms | ~3-8ms |
| Progress update frequency | 10+ Hz | 10-100 Hz |

### Throughput

| Metric | Target | Measured |
|--------|--------|----------|
| Requests/second | >100 | ~200-300 |
| Updates/second | >100 | ~500-1000 |
| Message size (average) | <1KB | 200-500 bytes |

## Implementation Details

### Rust Client (`rust/src/zmq_client.rs`)

```rust
pub struct ZmqClient {
    req_sender: Sender<ClientRequest>,
    resp_receiver: Receiver<Response>,
    update_receiver: Receiver<ProgressUpdate>,
    // ...
}

// Usage
let client = ZmqClient::new_default()?;
client.send_request(Request::Ping)?;
let response = client.recv_response_timeout(Duration::from_secs(5))?;
```

### Python Server (`python/workers/zmq_server.py`)

```python
class ZmqServer:
    def start(self):
        # Bind REP socket
        self.rep_socket.bind(self.req_addr)

        # Bind PUB socket
        self.pub_socket.bind(self.pub_addr)

        # Main event loop
        while self.running:
            request = self.rep_socket.recv()
            response = self._handle_request(request)
            self.rep_socket.send(response)
```

## Error Handling

### Connection Errors

- **Rust**: Timeout after 5 seconds, send error response to UI
- **Python**: Return `ErrorResponse` with details
- **Recovery**: Automatic reconnection on next request

### Serialization Errors

- **Rust**: Log error, show "Communication error" in UI
- **Python**: Send `ErrorResponse` with serialization details
- **Prevention**: Message validation before serialization

### Request Timeouts

- **Client**: 5 second timeout on recv()
- **Server**: 1 second timeout on recv() (allows checking `running` flag)
- **UI**: Show "Backend not responding" after timeout

## Security Considerations

### Network Binding

- Default: `tcp://127.0.0.1:*` (localhost only)
- Production: Use `ipc://` sockets for better isolation
- No authentication: Assumes trusted local environment

### Message Validation

- All messages validated after deserialization
- Invalid messages return `ErrorResponse`
- No arbitrary code execution (unlike pickle)

## Future Optimizations

### Potential Improvements

1. **IPC Sockets**: Switch from TCP to Unix domain sockets (<0.5ms latency)
2. **Zero-copy**: Use shared memory for large images
3. **Batching**: Combine multiple progress updates
4. **Compression**: Enable zlib compression for large prompts
5. **Connection pooling**: Reuse connections for multiple requests

### Scaling

- Current: Single-threaded Python server
- Future: Multi-process worker pool for parallel generation
- Target: 10+ concurrent generation jobs

## Monitoring

### Metrics to Track

- Request latency (p50, p95, p99)
- Message size distribution
- Error rate
- Queue depth
- Active connections

### Debug Mode

Set `RUST_LOG=debug` to see all message traffic:

```bash
RUST_LOG=debug ./dgx-pixels-tui
```

## Cross-Language Compatibility

### Message Protocol Guarantees

1. **Type safety**: Enum discriminants are strings ("generate", "ping", etc.)
2. **Optional fields**: Use `Option<T>` (Rust) and `Optional[T]` (Python)
3. **Field names**: snake_case everywhere (enforced by serde)
4. **Arrays vs Tuples**: Tuples are encoded as arrays (MessagePack limitation)

### Testing

- Unit tests: Serialize in Rust, deserialize in Python (and vice versa)
- Integration tests: Full round-trip through ZeroMQ
- Property tests: Fuzz testing with random messages

## References

- [ZeroMQ Guide](https://zguide.zeromq.org/)
- [MessagePack Specification](https://msgpack.org/)
- [rmp-serde Documentation](https://docs.rs/rmp-serde/)
- [PyZMQ Documentation](https://pyzmq.readthedocs.io/)
