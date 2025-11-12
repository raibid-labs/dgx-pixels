# IPC Communication Guide

## Quick Start

### Starting the Python Backend

```bash
cd /home/beengud/raibid-labs/dgx-pixels
python3 python/workers/zmq_server.py
```

Output:
```
Starting ZeroMQ server v1.0.0
REQ-REP endpoint: tcp://127.0.0.1:5555
PUB-SUB endpoint: tcp://127.0.0.1:5556
Server started successfully
```

### Running the Rust TUI

```bash
cd /home/beengud/raibid-labs/dgx-pixels/rust
cargo run --release
```

The TUI will automatically connect to the backend on startup.

## Usage Examples

### Example 1: Sending a Generation Request

**Rust Code:**

```rust
use dgx_pixels::messages::*;
use dgx_pixels::zmq_client::ZmqClient;

let client = ZmqClient::new_default()?;

// Send generation request
let request = Request::Generate {
    id: "job-001".to_string(),
    prompt: "16-bit knight sprite, pixel art".to_string(),
    model: "sdxl-base".to_string(),
    lora: Some("pixelart".to_string()),
    size: (1024, 1024),
    steps: 30,
    cfg_scale: 7.5,
};

client.send_request(request)?;

// Wait for response
let response = client.recv_response_timeout(Duration::from_secs(5))?;

match response {
    Response::JobAccepted { job_id, estimated_time_s } => {
        println!("Job {} accepted, ETA: {:.1}s", job_id, estimated_time_s);
    }
    Response::JobError { job_id, error } => {
        eprintln!("Job {} failed: {}", job_id, error);
    }
    _ => {}
}
```

**Python Server (automatic handling):**

```python
# Server automatically:
# 1. Receives request
# 2. Adds job to queue
# 3. Returns JobAcceptedResponse
# 4. Publishes JobStartedUpdate
```

### Example 2: Receiving Progress Updates

**Rust Code:**

```rust
// Non-blocking check for updates
while let Some(update) = client.try_recv_update() {
    match update {
        ProgressUpdate::Progress { job_id, step, total_steps, percent, .. } => {
            println!("[{}] Step {}/{} ({:.1}%)", job_id, step, total_steps, percent);
        }
        ProgressUpdate::Preview { job_id, image_path, .. } => {
            println!("[{}] Preview available: {}", job_id, image_path);
        }
        ProgressUpdate::JobFinished { job_id, success, duration_s } => {
            if success {
                println!("[{}] Completed in {:.1}s", job_id, duration_s);
            } else {
                println!("[{}] Failed", job_id);
            }
        }
        _ => {}
    }
}
```

### Example 3: Listing Available Models

**Rust Code:**

```rust
client.send_request(Request::ListModels)?;

let response = client.recv_response_timeout(Duration::from_secs(2))?;

if let Response::ModelList { models } = response {
    for model in models {
        println!("{} ({}) - {} MB",
            model.name,
            model.model_type,
            model.size_mb
        );
    }
}
```

**Output:**
```
SDXL Base 1.0 (checkpoint) - 6500 MB
Pixel Art LoRA (lora) - 144 MB
```

### Example 4: Checking Backend Status

**Rust Code:**

```rust
client.send_request(Request::Status)?;

let response = client.recv_response_timeout(Duration::from_secs(2))?;

if let Response::StatusInfo { version, queue_size, active_jobs, uptime_s } = response {
    println!("Backend v{}", version);
    println!("Queue: {} jobs", queue_size);
    println!("Active: {} jobs", active_jobs);
    println!("Uptime: {}s", uptime_s);
}
```

### Example 5: Cancelling a Job

**Rust Code:**

```rust
client.send_request(Request::Cancel {
    job_id: "job-001".to_string()
})?;

let response = client.recv_response_timeout(Duration::from_secs(2))?;

match response {
    Response::JobCancelled { job_id } => {
        println!("Job {} cancelled", job_id);
    }
    Response::JobError { error, .. } => {
        eprintln!("Cannot cancel: {}", error);
    }
    _ => {}
}
```

## Integration with TUI

### In App State

```rust
pub struct App {
    // ... other fields
    zmq_client: Option<ZmqClient>,
    active_jobs: HashMap<String, JobStatus>,
    pending_responses: VecDeque<Response>,
}

impl App {
    pub fn init_backend(&mut self) -> Result<()> {
        self.zmq_client = Some(ZmqClient::new_default()?);
        Ok(())
    }

    pub fn update(&mut self) {
        if let Some(client) = &self.zmq_client {
            // Check for responses
            while let Some(resp) = client.try_recv_response() {
                self.pending_responses.push_back(resp);
            }

            // Check for updates
            while let Some(update) = client.try_recv_update() {
                self.handle_update(update);
            }
        }
    }
}
```

### In Event Handler

```rust
// When user presses 'g' to generate
if key.code == KeyCode::Char('g') {
    if let Some(client) = &app.zmq_client {
        let request = Request::Generate {
            id: Uuid::new_v4().to_string(),
            prompt: app.input_buffer.clone(),
            model: app.selected_model.clone(),
            lora: app.selected_lora.clone(),
            size: (1024, 1024),
            steps: app.settings.steps,
            cfg_scale: app.settings.cfg_scale,
        };

        if let Err(e) = client.send_request(request) {
            app.show_error(&format!("Failed to send request: {}", e));
        }
    }
}
```

## Error Handling Patterns

### Connection Failures

```rust
match ZmqClient::new_default() {
    Ok(client) => self.zmq_client = Some(client),
    Err(e) => {
        self.show_error(&format!("Backend not available: {}", e));
        self.show_error("Start the backend with: python3 python/workers/zmq_server.py");
    }
}
```

### Request Timeouts

```rust
match client.recv_response_timeout(Duration::from_secs(5)) {
    Ok(response) => self.handle_response(response),
    Err(e) => {
        self.show_error("Backend not responding");
        self.show_error("Check if python/workers/zmq_server.py is running");
    }
}
```

### Invalid Responses

```rust
match response {
    Response::Error { message } => {
        self.show_error(&format!("Backend error: {}", message));
    }
    expected => {
        // Handle expected response
    }
}
```

## Performance Tips

### Non-blocking Operations

Always use `try_recv_*()` methods in UI render loop:

```rust
// ✓ Good: Non-blocking
while let Some(update) = client.try_recv_update() {
    process_update(update);
}

// ✗ Bad: Blocks UI rendering
let update = client.recv_update(); // This blocks!
```

### Batch Requests

If sending multiple requests, batch them:

```rust
// ✓ Good: Send all at once, then collect responses
for prompt in prompts {
    client.send_request(generate_request(prompt))?;
}
for _ in 0..prompts.len() {
    responses.push(client.recv_response_timeout(timeout)?);
}

// ✗ Bad: Interleaved send/receive
for prompt in prompts {
    client.send_request(generate_request(prompt))?;
    responses.push(client.recv_response_timeout(timeout)?);
}
```

### Connection Reuse

Keep one client instance for the lifetime of the app:

```rust
// ✓ Good: Reuse connection
struct App {
    client: ZmqClient, // Created once in init()
}

// ✗ Bad: Create new connection each time
fn send_request() {
    let client = ZmqClient::new_default()?; // Don't do this!
    client.send_request(...)?;
}
```

## Debugging

### Enable Logging

**Rust:**
```bash
RUST_LOG=debug cargo run
```

**Python:**
```bash
python3 -u python/workers/zmq_server.py  # Unbuffered output
```

### Inspect Messages

Add message dumping in development:

```rust
let serialized = serialize(&request)?;
eprintln!("Sending {} bytes", serialized.len());
eprintln!("Message: {:?}", request);
```

### Monitor Network

```bash
# Check if sockets are listening
lsof -i :5555
lsof -i :5556

# Monitor ZeroMQ traffic (requires tcpdump)
sudo tcpdump -i lo port 5555 -X
```

## Common Issues

### "Resource temporarily unavailable"

**Cause**: Backend not running or connection timeout
**Fix**: Start `python/workers/zmq_server.py` first

### "Address already in use"

**Cause**: Previous server still running
**Fix**: Kill old server: `killall python3` or `lsof -ti:5555 | xargs kill`

### Messages not received

**Cause**: Subscriber connected before publisher bound
**Fix**: Add 100ms delay after connecting SUB socket

```rust
socket.connect(addr)?;
std::thread::sleep(Duration::from_millis(100));
```

### High latency

**Cause**: TCP slow start, large messages
**Fix**:
- Switch to IPC sockets: `ipc:///tmp/dgx-pixels.sock`
- Enable compression for large prompts
- Reduce message frequency

## Next Steps

- See `docs/zmq-architecture.md` for architecture details
- See `examples/` for complete working examples
- See `tests/ws_09/` for test patterns
