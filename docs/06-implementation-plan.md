# Implementation Plan

## Overview

This document provides a step-by-step implementation guide for the DGX-Pixels AI pixel art generation stack. Choose your architecture proposal from `02-architecture-proposals.md` and follow the corresponding implementation path.

---

## Pre-Implementation Checklist

### Hardware
- [ ] NVIDIA DGX-Spark set up and accessible
- [ ] Sufficient storage space (500GB+ recommended)
- [ ] Network connectivity configured
- [ ] SSH access configured (if remote)

### Software
- [ ] Ubuntu/Linux environment
- [ ] NVIDIA drivers installed and working
- [ ] CUDA toolkit installed
- [ ] Docker installed (optional, for containerization)
- [ ] Git configured

### Team
- [ ] Technical lead assigned
- [ ] Bevy developer(s) identified
- [ ] Artist for evaluation (optional but recommended)
- [ ] Timeline agreed upon

### Project Setup
- [ ] GitHub/GitLab repository created
- [ ] Project structure decided
- [ ] Documentation location established
- [ ] Issue tracking set up

---

## Implementation Path Selection

Choose based on your needs:

| Path | Duration | Complexity | Best For |
|------|----------|-----------|----------|
| **Path A: Rapid** | 1-2 weeks | Low | Prototypes, solo devs |
| **Path B: Balanced** | 4-6 weeks | Medium | Small studios |
| **Path C: Advanced** | 8-12 weeks | High | Large studios |

---

# Path A: Rapid Implementation

**Goal**: Working pixel art generation in 1-2 weeks

## Week 1: Setup and Testing

### Day 1-2: Install Automatic1111

```bash
# SSH to DGX-Spark
ssh user@dgx-spark

# Install dependencies
sudo apt update
sudo apt install -y python3-pip python3-venv git

# Clone Automatic1111
cd ~/
git clone https://github.com/AUTOMATIC1111/stable-diffusion-webui.git
cd stable-diffusion-webui

# Install
python3 -m venv venv
source venv/bin/activate
./webui.sh --listen --api --xformers

# Access at http://dgx-spark:7860
```

### Day 2-3: Download Models

```bash
# Create models directory
mkdir -p models/Stable-diffusion

# Download pixel art models from Civitai
# Option 1: Pixel Art Diffusion XL
wget -O models/Stable-diffusion/pixel_art_xl.safetensors \
  "https://civitai.com/api/download/models/[MODEL_ID]"

# Test generation via web UI
# Prompt: "pixelsprite, knight character, standing pose"
```

### Day 3-4: Build CLI Tool

```bash
# Create project
mkdir ~/dgx-pixels-cli
cd ~/dgx-pixels-cli
python3 -m venv venv
source venv/bin/activate

pip install click requests pillow pyyaml
```

**Create `cli.py`:**

```python
#!/usr/bin/env python3
import click
import requests
from pathlib import Path

API_URL = "http://localhost:7860"

@click.group()
def cli():
    """DGX-Pixels CLI"""
    pass

@cli.command()
@click.argument('category')
@click.argument('prompt')
@click.option('--size', default=1024)
@click.option('--output', default='./output')
def generate(category, prompt, size, output):
    """Generate a pixel art sprite."""

    full_prompt = f"pixel art, {prompt}, game sprite, {category}"

    payload = {
        "prompt": full_prompt,
        "negative_prompt": "blurry, low quality, bad pixels",
        "steps": 30,
        "width": size,
        "height": size,
        "cfg_scale": 7.5,
    }

    response = requests.post(
        f"{API_URL}/sdapi/v1/txt2img",
        json=payload
    )

    result = response.json()

    # Save image
    output_path = Path(output) / category
    output_path.mkdir(parents=True, exist_ok=True)

    import base64
    from PIL import Image
    import io

    img_data = base64.b64decode(result['images'][0])
    img = Image.open(io.BytesIO(img_data))
    img.save(output_path / f"{prompt.replace(' ', '_')}.png")

    click.echo(f"Generated: {output_path}/{prompt.replace(' ', '_')}.png")

if __name__ == '__main__':
    cli()
```

**Test:**
```bash
python cli.py generate character "medieval knight"
```

### Day 5-7: Bevy Integration

**Create test Bevy project:**

```bash
cargo new --bin test_game
cd test_game
```

**Cargo.toml:**
```toml
[dependencies]
bevy = "0.13"
```

**Create `assets/` directory:**
```bash
mkdir -p assets/sprites/characters
```

**Test loading:**
```rust
// src/main.rs
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(Camera2dBundle::default());

    // Load generated sprite
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sprites/characters/medieval_knight.png"),
        ..default()
    });
}
```

**Generate and copy:**
```bash
cd ~/dgx-pixels-cli
python cli.py generate character "medieval knight"
cp output/character/medieval_knight.png ~/test_game/assets/sprites/characters/

# Run Bevy game
cd ~/test_game
cargo run
```

## Week 2: Polish and Documentation

### Day 8-10: Improve CLI

Add batch processing:

```python
@cli.command()
@click.argument('prompts_file')
@click.option('--output', default='./output')
def batch(prompts_file, output):
    """Generate from prompts file."""
    with open(prompts_file) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue

            category, prompt = line.split(':', 1)
            generate.callback(category.strip(), prompt.strip(), 1024, output)
```

**Create `prompts.txt`:**
```
character: medieval knight, standing
character: wizard, casting spell
item: health potion, red
item: sword, steel blade
```

### Day 11-12: Documentation

Write user guide:
- How to generate sprites
- How to copy to Bevy
- Prompt tips
- Troubleshooting

### Day 13-14: Testing and Handoff

- Generate 20-30 test sprites
- Load in Bevy game
- Document any issues
- Train team on usage

**Deliverables:**
- Working generation pipeline
- CLI tool
- Basic Bevy integration
- User documentation

---

# Path B: Balanced Implementation

**Goal**: Production-ready system in 4-6 weeks

## Week 1-2: Core Infrastructure

### Setup ComfyUI

```bash
# Install ComfyUI
git clone https://github.com/comfyanonymous/ComfyUI.git
cd ComfyUI
python -m venv venv
source venv/bin/activate
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121
pip install -r requirements.txt

# Test
python main.py --listen --port 8188
```

### Download Models

```bash
# Base model
cd models/checkpoints
wget https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/resolve/main/sd_xl_base_1.0.safetensors

# Pixel art checkpoint
# Download from Civitai and place in models/checkpoints/

# Create LoRA directory
mkdir -p models/loras
```

### Create Workflows

**Save workflow JSON templates:**

```json
// workflows/sprite_generation.json
{
  "1": {
    "class_type": "CheckpointLoaderSimple",
    "inputs": {"ckpt_name": "pixel_art_xl.safetensors"}
  },
  "2": {
    "class_type": "CLIPTextEncode",
    "inputs": {
      "text": "pixelsprite, {{prompt}}",
      "clip": ["1", 1]
    }
  },
  // ... rest of workflow
}
```

## Week 3-4: API Layer

### FastAPI Application

```bash
mkdir ~/dgx-pixels-api
cd ~/dgx-pixels-api
python -m venv venv
source venv/bin/activate
pip install fastapi uvicorn pydantic python-multipart aiofiles
```

**Project structure:**

```
dgx-pixels-api/
├── app/
│   ├── __init__.py
│   ├── main.py
│   ├── models/
│   │   └── schemas.py
│   ├── services/
│   │   ├── comfyui_client.py
│   │   └── storage.py
│   └── routers/
│       └── generate.py
├── requirements.txt
└── config.yaml
```

**Implement core services** (see `03-technology-deep-dive.md` for details)

### Test API

```bash
uvicorn app.main:app --host 0.0.0.0 --port 8000

# Test endpoint
curl -X POST http://localhost:8000/api/v1/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "knight character", "style": "16bit"}'
```

## Week 5: Training Pipeline

### Setup Kohya_ss

```bash
git clone https://github.com/bmaltais/kohya_ss.git
cd kohya_ss
./setup.sh
```

### Prepare Training Data

```bash
mkdir -p training_data/style_general
# Add 50-100 images + captions
```

### Train First LoRA

```bash
./train_network.sh \
  --pretrained_model_name_or_path="stabilityai/stable-diffusion-xl-base-1.0" \
  --train_data_dir="./training_data/style_general" \
  --output_dir="./output/my_style_v1"
```

## Week 6: MCP Integration

### Add FastMCP

```bash
pip install fastmcp
```

### Implement MCP Server

```python
from fastmcp import FastMCP

mcp = FastMCP.from_fastapi(app)

@mcp.tool()
async def generate_sprite(prompt: str, style: str) -> dict:
    # Implementation
    pass

mcp_app = mcp.http_app(path="/mcp")
app.mount("/mcp", mcp_app)
```

### Setup bevy_brp_mcp in Bevy

```toml
[dependencies]
bevy_brp_mcp = "0.1"
```

### Test Integration

Generate from MCP → Automatically appears in Bevy assets

## Weeks 7+: Polish and Production

- Custom ComfyUI nodes
- Batch processing optimization
- Web dashboard (optional)
- Comprehensive testing
- Documentation

**Deliverables:**
- Production API
- Trained custom LoRA
- MCP integration
- Full documentation

---

# Path C: Advanced Implementation

**Goal**: Enterprise-grade system in 8-12 weeks

## Phase 1: Infrastructure (Weeks 1-3)

### Kubernetes Setup

```bash
# Install k3s (lightweight Kubernetes)
curl -sfL https://get.k3s.io | sh -

# Verify
kubectl get nodes
```

### Deploy Model Serving

```yaml
# k8s/triton-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: triton-inference
spec:
  replicas: 2
  selector:
    matchLabels:
      app: triton
  template:
    metadata:
      labels:
        app: triton
    spec:
      containers:
      - name: triton
        image: nvcr.io/nvidia/tritonserver:23.12-py3
        ports:
        - containerPort: 8000
        - containerPort: 8001
        volumeMounts:
        - name: model-repository
          mountPath: /models
      volumes:
      - name: model-repository
        hostPath:
          path: /mnt/models
```

### Deploy MLflow

```bash
helm install mlflow mlflow/mlflow
```

## Phase 2: Microservices (Weeks 4-6)

Build services:
- auth-service
- job-service
- inference-service
- training-service
- asset-service
- mcp-service

Deploy each as separate Kubernetes pod

## Phase 3: Web UI (Weeks 7-9)

### Setup React App

```bash
npx create-react-app dgx-pixels-ui
cd dgx-pixels-ui
npm install @mui/material @emotion/react @emotion/styled
```

### Implement Features
- Project management
- Asset browser
- Training dashboard
- Analytics

## Phase 4: Integration (Weeks 10-11)

- Multi-engine plugins
- CI/CD pipelines
- Monitoring and alerting
- Security audit

## Phase 5: Launch (Week 12)

- Load testing
- Documentation
- Training materials
- Go-live

**Deliverables:**
- Full enterprise platform
- Web UI
- Multi-engine support
- Complete observability

---

## Post-Implementation

### Monitoring

```bash
# Set up Prometheus + Grafana
kubectl apply -f monitoring/prometheus.yaml
kubectl apply -f monitoring/grafana.yaml
```

### Maintenance Schedule

**Daily:**
- Check generation queue
- Monitor GPU utilization
- Review error logs

**Weekly:**
- Curate best generations
- Update prompt library
- Review quality metrics

**Monthly:**
- Retrain models with new data
- Update base models
- Performance optimization

### Continuous Improvement

1. **Collect feedback** from artists
2. **Curate high-quality outputs** for training
3. **Retrain models** monthly
4. **A/B test** new models
5. **Measure success** metrics

### Success Metrics

Track these KPIs:

```python
metrics = {
    "generation_time_avg": "3.2s",  # Target: <5s
    "assets_used_ratio": 0.82,      # Target: >80%
    "post_processing_time": "2min", # Target: <5min
    "artist_satisfaction": 4.3,     # Target: >4.0
    "cost_per_asset": "$0.05",      # Target: <$0.10
}
```

---

## Troubleshooting

### ComfyUI Issues

**Problem: Out of memory**
```python
# In ComfyUI, enable CPU offload
pipe.enable_model_cpu_offload()
```

**Problem: Slow generation**
```bash
# Enable xformers
pip install xformers
# Will be used automatically
```

### Training Issues

**Problem: Loss not decreasing**
- Check learning rate (try 1e-5)
- Verify dataset quality
- Check captions are meaningful

**Problem: Overfitting**
- Add more diverse data
- Increase dropout
- Reduce training steps

### Integration Issues

**Problem: MCP connection fails**
- Verify Bevy app is running
- Check port 15702 is open
- Review firewall settings

**Problem: Assets not hot-reloading**
```rust
// Enable asset watching explicitly
.add_plugins(DefaultPlugins.set(AssetPlugin {
    watch_for_changes_override: Some(true),
    ..default()
}))
```

---

## Support Resources

### Documentation
- ComfyUI: https://github.com/comfyanonymous/ComfyUI
- Diffusers: https://huggingface.co/docs/diffusers
- Bevy: https://bevyengine.org/
- FastAPI: https://fastapi.tiangolo.com/

### Communities
- ComfyUI Discord
- Stable Diffusion subreddit
- Bevy Discord
- CivitAI forums

### Team Training

**Week 1: Fundamentals**
- How diffusion models work
- Prompt engineering
- Using the CLI/API

**Week 2: Advanced**
- Training custom models
- Post-processing techniques
- Bevy integration

**Week 3: Production**
- Workflow optimization
- Quality control
- Troubleshooting

---

## Checklist for Go-Live

- [ ] All models tested and validated
- [ ] Documentation complete
- [ ] Team trained
- [ ] Backup and restore procedures tested
- [ ] Monitoring in place
- [ ] Performance benchmarks met
- [ ] Security review passed
- [ ] User acceptance testing complete
- [ ] Rollback plan documented
- [ ] Support process established

---

## Next Steps

1. **Select implementation path** (A, B, or C)
2. **Assign team members** to tasks
3. **Set up project tracking** (Jira, GitHub Projects, etc.)
4. **Begin Week 1 tasks** according to chosen path
5. **Schedule regular check-ins** (daily standups, weekly reviews)

Good luck with your implementation! For questions about specific technologies, refer back to `03-technology-deep-dive.md`.
