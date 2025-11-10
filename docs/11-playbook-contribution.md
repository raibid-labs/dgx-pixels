# DGX-Spark Playbooks Contribution

## Overview

This document proposes contributing a **DGX-Pixels playbook** to the official [dgx-spark-playbooks](https://github.com/raibid-labs/dgx-spark-playbooks) repository. The playbook will provide step-by-step instructions for setting up AI pixel art generation on NVIDIA DGX Spark devices.

##Table of Contents
- [Playbook Scope](#playbook-scope)
- [Integration with Existing Playbooks](#integration-with-existing-playbooks)
- [Proposed Playbook Structure](#proposed-playbook-structure)
- [Prerequisites](#prerequisites)
- [Installation Steps](#installation-steps)
- [Validation](#validation)

---

## Playbook Scope

### What the Playbook Covers

1. **ComfyUI Setup** for pixel art generation (leverages existing ComfyUI playbook)
2. **Model Installation**: SDXL base + pixel art checkpoints
3. **LoRA Training** environment setup
4. **Python Worker** deployment for job management
5. **Rust TUI Client** installation
6. **Integration Testing**: End-to-end generation workflow

### What's Out of Scope

- Bevy game engine setup (separate concern)
- Custom model training (covered in training playbook)
- Advanced optimization (separate performance playbook)

---

## Integration with Existing Playbooks

### Builds Upon

**1. ComfyUI Playbook** (`nvidia/comfy-ui/`)
- Our stack uses ComfyUI as the inference engine
- Follow existing playbook for base installation
- Add pixel-art-specific configurations

**2. PyTorch Fine-tune Playbook** (`nvidia/pytorch-fine-tune/`)
- Reference for LoRA training setup
- Similar environment but pixel-art-specific

**3. vLLM Playbook** (`nvidia/vllm/`)
- Similar multi-component architecture
- Good reference for service orchestration

### Unique Value Add

1. **Rust TUI Interface**: First playbook with Rust frontend
2. **ZeroMQ IPC**: Modern inter-process communication
3. **Side-by-Side Model Comparison**: Unique workflow for artists
4. **Game Dev Focus**: Optimized for sprite/asset generation
5. **Hybrid Rust+Python**: Best-of-both-worlds architecture

---

## Proposed Playbook Structure

```
nvidia/dgx-pixels/
├── README.md                    # Playbook overview
├── prerequisites.md             # Hardware/software requirements
├── setup/
│   ├── 01-comfyui-setup.md     # ComfyUI installation
│   ├── 02-models-download.md   # Download SDXL + pixel art models
│   ├── 03-python-worker.md     # Python backend setup
│   ├── 04-rust-tui.md          # Rust TUI installation
│   └── 05-integration-test.md  # End-to-end validation
├── workflows/
│   ├── sprite-generation.json  # ComfyUI workflow for sprites
│   ├── tileset-generation.json # Seamless tile generation
│   └── animation-frames.json   # Multi-frame sprite sheets
├── configs/
│   ├── config.toml.example     # Rust TUI configuration
│   ├── worker-config.yaml      # Python worker settings
│   └── models.json             # Model registry
├── scripts/
│   ├── install.sh              # Automated setup script
│   ├── download-models.sh      # Batch model download
│   ├── test-generation.sh      # Validation script
│   └── start-services.sh       # Launch all components
└── troubleshooting.md          # Common issues and solutions
```

---

## Prerequisites

### Hardware Requirements

- **NVIDIA DGX Spark** with GB10 Blackwell GPU
- **Memory**: 128GB unified (24GB+ for SDXL models)
- **Storage**: 50GB free space (models + outputs)
- **Network**: Internet for model downloads

### Software Requirements

**Base System**:
- DGX OS (Ubuntu-based)
- CUDA 12.1+
- Python 3.10+
- Rust 1.83+ (will be installed by script)

**Python Packages**:
```
torch>=2.5.0
diffusers>=0.31.0
transformers>=4.46.0
accelerate>=1.2.0
xformers>=0.0.28
zmq>=26.2.0
msgpack>=1.1.0
aiohttp>=3.11.0
```

**Rust Dependencies**:
```toml
ratatui = "0.29"
crossterm = "0.28"
zmq = "0.10"
rmp-serde = "1.3"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
image = "0.25"
ratatui-image = "2.0"
```

---

## Installation Steps

### Step 1: Clone and Prepare

```bash
# Clone the playbook repository (if contributing)
git clone https://github.com/raibid-labs/dgx-spark-playbooks.git
cd dgx-spark-playbooks/nvidia/dgx-pixels

# Or clone DGX-Pixels directly
git clone https://github.com/raibid-labs/dgx-pixels.git
cd dgx-pixels
```

### Step 2: Run Automated Installer

```bash
# Makes installation idempotent and automated
./scripts/install.sh
```

**What `install.sh` does**:
1. Checks prerequisites (CUDA, Python, disk space)
2. Installs Rust if not present
3. Sets up Python virtual environment
4. Installs ComfyUI (or uses existing installation)
5. Downloads base models (SDXL)
6. Downloads pixel art checkpoints
7. Builds Rust TUI
8. Configures services
9. Runs validation tests

### Step 3: Download Models

```bash
# Download SDXL base model (~8GB)
./scripts/download-models.sh --model sdxl-base

# Download pixel art checkpoint (~2GB)
./scripts/download-models.sh --model pixel-art-xl

# Optional: Download LoRAs
./scripts/download-models.sh --lora 16bit-rpg
./scripts/download-models.sh --lora fantasy-characters
```

**Models are saved to**:
```
~/.cache/dgx-pixels/models/
├── checkpoints/
│   ├── sd_xl_base_1.0.safetensors
│   └── pixel_art_diffusion_xl.safetensors
└── loras/
    ├── 16bit_rpg_v1.safetensors
    └── fantasy_characters_v2.safetensors
```

### Step 4: Start Services

```bash
# Option A: Start all services with one command
./scripts/start-services.sh

# Option B: Start services individually
# Terminal 1: ComfyUI
cd comfyui
python main.py --listen --port 8188

# Terminal 2: Python Worker
cd python
source venv/bin/activate
python -m workers.generation_worker

# Terminal 3: Rust TUI
cd rust
cargo run --release
```

### Step 5: Verify Installation

```bash
# Run automated tests
./scripts/test-generation.sh

# Manual verification
# 1. TUI should launch and show main screen
# 2. Enter a test prompt: "pixel art knight character"
# 3. Press [G] to generate
# 4. Result should appear in ~10-15 seconds
```

---

## Validation

### Automated Tests

The `test-generation.sh` script runs:

1. **Service Health Checks**
   ```bash
   # Check ComfyUI is running
   curl http://localhost:8188/system_stats

   # Check Python worker
   curl http://localhost:5555/health

   # Check ZMQ endpoints
   zmq-test tcp://localhost:5555
   ```

2. **Generation Test**
   ```bash
   # Submit test job via CLI
   dgx-pixels generate "test sprite" --headless

   # Verify output
   test -f output/test_sprite.png && echo "✓ Generation successful"
   ```

3. **Model Loading Test**
   ```bash
   # Verify SDXL loads without OOM
   python -c "
   from diffusers import DiffusionPipeline
   pipe = DiffusionPipeline.from_pretrained('stabilityai/stable-diffusion-xl-base-1.0')
   print('✓ Model loaded successfully')
   "
   ```

4. **LoRA Test**
   ```bash
   # Generate with LoRA adapter
   dgx-pixels generate "knight sprite" --lora 16bit_rpg --headless

   # Verify style difference
   ./scripts/compare-outputs.sh output/test_sprite.png output/knight_sprite.png
   ```

### Manual Validation Checklist

**Basic Functionality**:
- [ ] TUI launches without errors
- [ ] Can enter prompts and generate images
- [ ] Images appear in output directory
- [ ] Preview updates during generation
- [ ] Job queue works correctly

**Model Management**:
- [ ] Can list available models
- [ ] Can switch between models
- [ ] Can load/unload LoRAs
- [ ] Memory usage is reasonable

**Performance**:
- [ ] Generation completes in 10-15 seconds
- [ ] GPU utilization reaches 85%+
- [ ] No memory leaks over multiple generations
- [ ] TUI remains responsive during generation

**Integration**:
- [ ] Python worker communicates with TUI
- [ ] ComfyUI receives requests
- [ ] Progress updates arrive in real-time
- [ ] Results are saved correctly

---

## Configuration

### Default Configuration

**`~/.config/dgx-pixels/config.toml`**:
```toml
[general]
output_dir = "~/dgx-pixels/output"
temp_dir = "/tmp/dgx-pixels"

[zmq]
req_endpoint = "tcp://localhost:5555"
sub_endpoint = "tcp://localhost:5556"

[comfyui]
base_url = "http://localhost:8188"
ws_url = "ws://localhost:8188/ws"

[models]
default_checkpoint = "pixel_art_diffusion_xl"
default_lora = "16bit_rpg_v1"
models_dir = "~/.cache/dgx-pixels/models"

[generation]
default_size = [1024, 1024]
default_steps = 30
default_cfg_scale = 7.5
default_batch_size = 1

[ui]
theme = "dark"
show_preview = true
preview_update_interval_ms = 500
fps_limit = 60

[bevy]
# Optional: Auto-deploy to Bevy project
project_path = "~/my_game"
assets_subdir = "assets/sprites"
auto_deploy = false
```

### Environment Variables

```bash
# Override config file location
export DGX_PIXELS_CONFIG="/custom/path/config.toml"

# Override models directory
export DGX_PIXELS_MODELS="/data/models"

# Debug mode
export DGX_PIXELS_LOG_LEVEL="debug"
export RUST_LOG="dgx_pixels=debug"
```

---

## Troubleshooting

### Common Issues

**1. "Address already in use" on port 5555**
```bash
# Find process using the port
lsof -i :5555

# Kill if it's a stale worker
kill -9 <PID>

# Or change port in config
[zmq]
req_endpoint = "tcp://localhost:5556"  # Use different port
```

**2. "Model not found" errors**
```bash
# Verify models are downloaded
ls -lh ~/.cache/dgx-pixels/models/checkpoints/

# Re-download if missing
./scripts/download-models.sh --model sdxl-base --force
```

**3. "Out of memory" during generation**
```bash
# Check GPU memory
nvidia-smi

# Reduce batch size in config
[generation]
default_batch_size = 1  # Down from higher value

# Or use smaller model
[models]
default_checkpoint = "sd_1_5"  # Instead of SDXL
```

**4. TUI shows garbled graphics**
```bash
# Check terminal emulator
echo $TERM  # Should be xterm-256color or similar

# Try different image protocol
# Edit config:
[ui]
force_halfblock_protocol = true  # Disable sixels
```

**5. Python worker not responding**
```bash
# Check worker logs
journalctl -u dgx-pixels-worker -f

# Restart worker
systemctl restart dgx-pixels-worker

# Or run manually for debugging
cd python
python -m workers.generation_worker --debug
```

### Getting Help

1. **Check Documentation**: `docs/` directory in repository
2. **Run Diagnostics**: `./scripts/diagnose.sh`
3. **Check Logs**:
   - Python worker: `~/.cache/dgx-pixels/logs/worker.log`
   - Rust TUI: stdout/stderr
   - ComfyUI: `comfyui.log`
4. **GitHub Issues**: Report bugs with diagnostic output
5. **Community**: DGX Spark forums or Discord

---

## Performance Tuning

### Optimization Checklist

**GPU Utilization**:
```bash
# Monitor during generation
watch -n 1 nvidia-smi

# Target: 85-95% utilization
# If lower, increase batch size or steps
```

**Memory Usage**:
```bash
# Check memory allocation
nvidia-smi --query-gpu=memory.used,memory.total --format=csv

# SDXL should use ~8GB
# With LoRA: ~8.5GB
# Available: 128 - 8.5 = 119.5GB for other workloads
```

**Inference Speed**:
```bash
# Benchmark generation time
dgx-pixels benchmark --iterations 10

# Target: 10-15 seconds for 1024x1024 SDXL
# Slower? Check:
# - GPU throttling (temperature >80°C)
# - CPU bottleneck (check with htop)
# - Disk I/O (check with iostat)
```

**TUI Performance**:
```bash
# Check FPS
# Should be 60 FPS when idle, 30+ during generation

# If sluggish:
# - Reduce preview update frequency in config
# - Disable live preview ([P] toggle)
# - Use halfblock protocol instead of sixels
```

### Advanced Configuration

**Enable xformers** (faster inference):
```python
# In python worker
pipe.enable_xformers_memory_efficient_attention()
```

**Use torch.compile** (PyTorch 2.0+):
```python
pipe.unet = torch.compile(pipe.unet, mode="reduce-overhead")
```

**Quantization** (reduce memory):
```python
# Use FP16 instead of FP32
pipe = DiffusionPipeline.from_pretrained(
    model_id,
    torch_dtype=torch.float16
)
```

---

## Migration from Existing Setups

### From Standalone ComfyUI

1. **Keep existing ComfyUI**: DGX-Pixels connects to running instance
2. **Import workflows**: Copy JSON files to `workflows/` directory
3. **Models already downloaded**: Point config to existing models directory

```toml
[comfyui]
base_url = "http://localhost:8188"  # Your existing ComfyUI

[models]
models_dir = "/path/to/existing/comfyui/models"
```

### From Automatic1111

1. **Models compatible**: Copy `.safetensors` files
2. **LoRAs compatible**: Move to models/loras/
3. **Prompts can be reused**: Same syntax

```bash
# Copy A1111 models
cp ~/stable-diffusion-webui/models/Stable-diffusion/*.safetensors \
   ~/.cache/dgx-pixels/models/checkpoints/

# Copy LoRAs
cp ~/stable-diffusion-webui/models/Lora/*.safetensors \
   ~/.cache/dgx-pixels/models/loras/
```

### From Python Scripts

Replace:
```python
# Old: Direct API calls
import requests
response = requests.post("http://localhost:7860/api/generate", ...)
```

With:
```bash
# New: Use dgx-pixels CLI
dgx-pixels generate "your prompt" --model sdxl --lora 16bit_rpg
```

Or keep using API by connecting to worker endpoint.

---

## Contributing to Playbook

### Submission Checklist

- [ ] Playbook tested on fresh DGX Spark
- [ ] All scripts are idempotent (can run multiple times)
- [ ] Error messages are clear and actionable
- [ ] Follows dgx-spark-playbooks formatting conventions
- [ ] Screenshots/GIFs included for visual steps
- [ ] Prerequisites clearly stated
- [ ] Troubleshooting section covers common issues
- [ ] Performance expectations documented
- [ ] Integration with other playbooks noted

### File Requirements

**README.md must include**:
- Quick overview (2-3 sentences)
- Prerequisites list
- Estimated time to complete
- Step-by-step instructions with code blocks
- Validation steps
- Next steps / related playbooks

**scripts/ must**:
- Be executable (`chmod +x`)
- Have error checking (`set -e`)
- Provide clear output
- Support `--help` flag
- Be idempotent where possible

### Testing Before Submission

```bash
# Test on clean system
docker run -it --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04

# Follow playbook exactly
# Document any issues
# Fix and retest

# Verify automation
./scripts/install.sh
./scripts/test-generation.sh

# Both should complete successfully
```

---

## Maintenance Plan

### Regular Updates

**Monthly**:
- Check for model updates (new SDXL versions, LoRAs)
- Update dependency versions
- Test on latest DGX OS

**Quarterly**:
- Performance benchmarks
- Compare with alternative stacks
- Community feedback integration

**Yearly**:
- Major version upgrades
- Architecture review
- Deprecate outdated approaches

### Version Compatibility

| DGX-Pixels | ComfyUI | SDXL | PyTorch | Rust |
|------------|---------|------|---------|------|
| 0.1.x | 0.2.0+ | 1.0 | 2.5+ | 1.83+ |
| 0.2.x | 0.2.2+ | 1.0 | 2.6+ | 1.85+ |
| 1.0.x | 0.3.0+ | 2.0 | 2.7+ | 1.87+ |

---

## License and Attribution

**Playbook License**: MIT (consistent with dgx-spark-playbooks)

**Attribution**:
- Based on research in `dgx-pixels` repository
- Integrates with NVIDIA's official ComfyUI playbook
- Built on open-source components (Rust, Python, ComfyUI, SDXL)

**Third-Party Licenses**:
- SDXL: CreativeML Open RAIL++-M
- ComfyUI: GPL-3.0
- PyO3: Apache 2.0 / MIT
- ratatui: MIT

---

## Next Steps

1. **Implement playbook** following this structure
2. **Test thoroughly** on DGX Spark
3. **Submit PR** to dgx-spark-playbooks repository
4. **Gather feedback** from community
5. **Iterate** based on user experience

See `docs/06-implementation-plan.md` for development timeline.
