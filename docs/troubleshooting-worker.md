# Backend Worker Troubleshooting Guide

## Common Issues and Solutions

### 1. Worker Won't Start

#### Symptom
```
Failed to connect to ComfyUI: Cannot connect to ComfyUI at http://localhost:8188
```

**Cause**: ComfyUI is not running or not accessible.

**Solution**:
```bash
# Check if ComfyUI is running
curl http://localhost:8188/system_stats

# If not, start ComfyUI
cd /workspace/ComfyUI
python main.py --listen 0.0.0.0 --port 8188
```

---

#### Symptom
```
ModuleNotFoundError: No module named 'zmq'
```

**Cause**: Python dependencies not installed.

**Solution**:
```bash
# Install dependencies
pip install -r python/requirements-worker.txt

# Verify installation
python3 -c "import zmq, msgpack, requests; print('OK')"
```

---

### 2. Jobs Not Executing

#### Symptom
Jobs accepted but never complete. No progress updates.

**Cause**: Worker thread crashed or ComfyUI hung.

**Solution**:
```bash
# Check worker logs
# Look for errors in worker output

# Restart ComfyUI
# Sometimes ComfyUI needs a restart after errors

# Check ComfyUI queue
curl http://localhost:8188/queue | python3 -m json.tool

# Clear stuck jobs in ComfyUI
curl -X POST http://localhost:8188/queue -d '{"clear": true}'
```

---

#### Symptom
```
Job failed: Workflow execution failed: Unknown error
```

**Cause**: Workflow or model issues.

**Solution**:
```bash
# Test workflow manually in ComfyUI UI
# 1. Open http://localhost:8188 in browser
# 2. Load workflow: workflows/sprite_optimized.json
# 3. Click Queue Prompt
# 4. Check for errors in console

# Verify models exist
ls -lh /workspace/models/checkpoints/sd_xl_base_1.0.safetensors

# Check workflow JSON is valid
python3 -m json.tool workflows/sprite_optimized.json
```

---

### 3. Timeout Errors

#### Symptom
```
TimeoutError: Workflow execution exceeded timeout (300s)
```

**Cause**: Generation taking longer than expected.

**Solutions**:

**Option 1**: Increase timeout
```bash
./scripts/start_worker.sh --comfyui-timeout 600
```

**Option 2**: Reduce generation complexity
- Fewer steps (20 → 10)
- Smaller resolution (1024 → 768)
- Remove LoRA if not needed

**Option 3**: Check GPU utilization
```bash
# Monitor GPU during generation
watch -n 1 nvidia-smi

# Look for:
# - Low GPU utilization (<50%)
# - Memory not being used
# - Process not showing up
```

---

### 4. Memory Issues

#### Symptom
```
CUDA out of memory
```

**Cause**: GPU memory exhausted.

**Solutions**:

**Option 1**: Enable model offloading in ComfyUI
```bash
# Edit ComfyUI config
# Set: lowvram=True or normalvram=True
cd /workspace/ComfyUI
python main.py --lowvram
```

**Option 2**: Reduce memory usage
- Use FP16 (enabled by default)
- Reduce batch size to 1
- Clear GPU cache between jobs

**Option 3**: Simplify workflow
- Remove unnecessary nodes
- Use VAE tiling for large images
- Disable preview generation

---

### 5. Connection Issues

#### Symptom
```
zmq.error.Again: Resource temporarily unavailable
```

**Cause**: ZeroMQ socket timeout or connection issue.

**Solutions**:

**Option 1**: Check if worker is running
```bash
./scripts/health_check.sh
```

**Option 2**: Verify ports are not in use
```bash
# Check if ports are already bound
netstat -tuln | grep 5555
netstat -tuln | grep 5556

# Kill conflicting processes if needed
lsof -ti:5555 | xargs kill -9
lsof -ti:5556 | xargs kill -9
```

**Option 3**: Increase socket timeout
```python
# In client code
socket.setsockopt(zmq.RCVTIMEO, 10000)  # 10 seconds
```

---

### 6. Progress Updates Not Received

#### Symptom
Client receives JobAccepted but no progress updates.

**Cause**: PUB-SUB socket not connected or not subscribed.

**Solutions**:

**Option 1**: Verify subscription
```python
# In client code
sub_socket.subscribe(b"")  # Subscribe to ALL messages
```

**Option 2**: Check binding order
```bash
# Server must bind before client connects
# Wait a moment after starting worker
sleep 2
```

**Option 3**: Test PUB-SUB manually
```python
import zmq
import msgpack

context = zmq.Context()
socket = context.socket(zmq.SUB)
socket.connect("tcp://127.0.0.1:5556")
socket.subscribe(b"")

while True:
    data = socket.recv()
    msg = msgpack.unpackb(data, raw=False)
    print(msg)
```

---

### 7. Image Output Issues

#### Symptom
Job completes but no image file found.

**Cause**: Output directory issue or ComfyUI save path mismatch.

**Solutions**:

**Option 1**: Check output directory
```bash
# Verify directory exists and is writable
ls -ld outputs/
chmod 755 outputs/

# Check recent files
ls -lth outputs/ | head -10
```

**Option 2**: Check ComfyUI output path
```bash
# ComfyUI saves to its own output directory
ls -lth /workspace/ComfyUI/output/ | head -10

# Worker downloads from ComfyUI to outputs/
```

**Option 3**: Debug download process
```python
# Add logging to comfyui_client.py download_image()
print(f"Downloading: {image_url}")
print(f"Saving to: {output_path}")
```

---

### 8. Performance Issues

#### Symptom
Generation is very slow (>60s for 20 steps).

**Solutions**:

**Option 1**: Verify GPU acceleration
```bash
# Check PyTorch sees GPU
python3 -c "import torch; print(torch.cuda.is_available())"

# Check ComfyUI is using GPU
curl http://localhost:8188/system_stats | python3 -m json.tool
# Look for device: "cuda"
```

**Option 2**: Enable optimizations
- xformers: `pip install xformers`
- FP16: Already enabled
- Karras scheduler: Using by default

**Option 3**: Profile generation
```bash
# Monitor during generation
nvidia-smi dmon -s pucvmet

# Look for:
# - SM (streaming multiprocessor) utilization
# - Memory bandwidth utilization
# - Power consumption
```

---

### 9. Workflow Errors

#### Symptom
```
KeyError: 'node_id'
ValueError: Invalid workflow structure
```

**Cause**: Workflow JSON malformed or incompatible.

**Solutions**:

**Option 1**: Validate workflow
```bash
# Check JSON syntax
python3 -m json.tool workflows/sprite_optimized.json > /dev/null
echo $?  # Should be 0

# Test in ComfyUI UI
```

**Option 2**: Re-export workflow
1. Open ComfyUI UI
2. Load working setup
3. Export → "Save (API Format)"
4. Replace workflow JSON

**Option 3**: Check node compatibility
- Verify all node types are installed
- Check custom node dependencies
- Update ComfyUI if needed

---

### 10. Debugging Tips

#### Enable Verbose Logging
```bash
# Add debug prints
# In generation_worker.py, add:
import logging
logging.basicConfig(level=logging.DEBUG)
```

#### Monitor Worker State
```python
# Send status request
request = StatusRequest()
# Check queue_size, active_jobs, uptime
```

#### Test Components Individually

**Test ComfyUI client**:
```bash
cd tests/ws_10
pytest test_comfyui_client.py -v -s
```

**Test progress tracker**:
```bash
pytest test_progress_tracker.py -v -s
```

**Test integration**:
```bash
pytest test_integration.py -v -s
```

#### Check System Resources
```bash
# CPU usage
top -b -n 1 | grep python

# Memory usage
free -h

# Disk space (for outputs)
df -h outputs/

# GPU status
nvidia-smi
```

---

## Error Code Reference

| Error | Cause | Solution |
|-------|-------|----------|
| `ConnectionError` | ComfyUI not accessible | Start ComfyUI, check URL |
| `TimeoutError` | Generation too slow | Increase timeout or reduce steps |
| `ValueError` | Invalid parameter | Check request format |
| `FileNotFoundError` | Missing workflow | Verify workflow path |
| `KeyError` | Workflow structure issue | Validate workflow JSON |
| `zmq.Again` | Socket timeout | Increase timeout or check connection |
| `CUDA OOM` | Out of GPU memory | Reduce batch size or resolution |

---

## Getting Help

### Collect Diagnostic Info

```bash
# Run health check
./scripts/health_check.sh -v > diagnostics.txt

# Add system info
echo "=== System Info ===" >> diagnostics.txt
nvidia-smi >> diagnostics.txt
python3 --version >> diagnostics.txt
pip list | grep -E "zmq|msgpack|requests" >> diagnostics.txt

# Add recent logs
echo "=== Recent Logs ===" >> diagnostics.txt
# (if logging to file)
tail -100 worker.log >> diagnostics.txt
```

### Test Workflow Manually

```bash
# Create minimal test
cd /workspace/ComfyUI
python3 << EOF
import json
workflow = json.load(open("../dgx-pixels/workflows/sprite_optimized.json"))
print("Workflow loaded successfully")
print(f"Nodes: {len(workflow)}")
EOF
```

### Verify Dependencies

```bash
# Check all dependencies
pip check

# Reinstall if needed
pip install --upgrade --force-reinstall -r python/requirements-worker.txt
```

---

## Known Issues

### Issue: First job takes longer
**Cause**: Model loading overhead
**Status**: Expected behavior
**Workaround**: Subsequent jobs will be faster

### Issue: Progress updates lag behind actual progress
**Cause**: Polling interval + network latency
**Status**: By design (100ms polling)
**Workaround**: Reduce poll_interval_ms (increases CPU usage)

### Issue: ETA inaccurate for first few jobs
**Cause**: No historical timing data
**Status**: Expected behavior
**Workaround**: ETA improves after ~5 jobs

---

## Best Practices

1. **Always run health check before starting**: `./scripts/health_check.sh`
2. **Monitor first few jobs**: Watch for errors or performance issues
3. **Keep ComfyUI updated**: New optimizations frequently released
4. **Clean output directory**: Disk space fills up quickly
5. **Test workflows in ComfyUI UI first**: Catch errors before automation
6. **Use version control for workflows**: Track changes to generation settings
7. **Monitor GPU temperature**: Ensure adequate cooling during heavy loads

---

## Recovery Procedures

### Full Reset

```bash
# 1. Stop worker
pkill -f generation_worker.py

# 2. Clear ComfyUI queue
curl -X POST http://localhost:8188/queue -d '{"clear": true}'

# 3. Restart ComfyUI
pkill -f "python.*ComfyUI"
cd /workspace/ComfyUI
python main.py &

# 4. Wait for ComfyUI to start
sleep 5

# 5. Start worker
./scripts/start_worker.sh
```

### Clear Output Directory

```bash
# Backup old outputs
mkdir -p outputs/archive
mv outputs/*.png outputs/archive/ 2>/dev/null

# Or delete all
rm -f outputs/*.png
```

### Reset Job Queue

Job queue is in-memory only, so restarting worker clears it.

```bash
# Restart worker
pkill -f generation_worker.py
./scripts/start_worker.sh
```
