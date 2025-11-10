# Architecture Proposals: DGX-Pixels Stack

## Overview

This document presents three architecture proposals for the DGX-Pixels AI pixel art generation stack, each targeting different maturity levels and use cases. All proposals leverage the NVIDIA DGX-Spark hardware and integrate with the Bevy game engine.

## Comparison Matrix

| Feature | Proposal 1: Rapid | Proposal 2: Balanced | Proposal 3: Advanced |
|---------|-------------------|----------------------|----------------------|
| **Time to MVP** | 1-2 weeks | 4-6 weeks | 8-12 weeks |
| **Custom Training** | No | LoRA fine-tuning | Full training pipeline |
| **UI Complexity** | Simple web UI | Node-based workflow | Custom integrated UI |
| **Bevy Integration** | Manual export | MCP semi-automatic | Full MCP automation |
| **Scalability** | Single user | Small team | Production-ready |
| **Model Switching** | Manual | Config-based | Dynamic |
| **Batch Processing** | Basic | Advanced | Fully automated |
| **Customization** | Low | Medium | High |
| **Maintenance** | Low | Medium | High |
| **Recommended For** | Prototyping | Small studios | Commercial products |

---

# Proposal 1: Rapid Prototyping Stack

## Philosophy

Get pixel art generation working quickly using existing tools with minimal custom development. Focus on proving the concept and gathering requirements before investing in custom infrastructure.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    NVIDIA DGX-Spark                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────┐            │
│  │  Automatic1111 WebUI + Extensions           │            │
│  │  - Pre-trained Pixel Art models             │            │
│  │  - Built-in FastAPI REST API                │            │
│  │  - Basic batch processing                   │            │
│  └──────────────┬─────────────────────────────┘            │
│                 │                                            │
│                 │ HTTP/REST API                             │
│                 │                                            │
│  ┌──────────────▼─────────────────────────────┐            │
│  │  Simple Python CLI Tool                     │            │
│  │  - Prompt management                        │            │
│  │  - Output organization                      │            │
│  │  - Sprite sheet assembly                    │            │
│  └──────────────┬─────────────────────────────┘            │
│                 │                                            │
│                 │ File system                                │
│                 │                                            │
└─────────────────┼────────────────────────────────────────────┘
                  │
                  ▼
          ┌───────────────┐
          │  Bevy Project │
          │  assets/      │
          │  directory    │
          └───────────────┘
```

## Components

### 1. Model Inference: Automatic1111 WebUI

**Why A1111:**
- Quick setup with pre-built installers
- Extensive extension ecosystem
- Built-in REST API
- Well-documented
- Active community support

**Installation:**
```bash
# Clone and setup
git clone https://github.com/AUTOMATIC1111/stable-diffusion-webui.git
cd stable-diffusion-webui
./webui.sh --api --listen
```

**Configuration:**
- Enable API mode
- Install pixel art models from Civitai
- Configure default parameters

### 2. Pre-trained Models

**Primary Model:**
- Pixel Art Diffusion XL (Sprite Shaper)
- Download from Civitai
- No training required

**Fallback Models:**
- Pixel Art Sprite Diffusion (SD 1.5 based)
- All-In-One-Pixel-Model

### 3. Python CLI Tool

**Features:**
- Simple command-line interface
- Prompt templates for common sprite types
- API calls to A1111
- Basic sprite sheet assembly
- Output organization by category

**Example Usage:**
```bash
# Generate character sprite
dgx-pixels generate character "medieval knight" --poses walk,idle,attack

# Generate item sprites
dgx-pixels generate item "health potion, magic sword" --style 16bit

# Batch generate
dgx-pixels batch prompts.txt --output assets/sprites/
```

**Implementation:**
```python
# Simple structure
src/
  cli.py           # Click-based CLI
  api_client.py    # A1111 API wrapper
  prompt_builder.py # Prompt templates
  sprite_utils.py  # Basic sprite processing
  config.py        # Configuration management
```

### 4. Manual Bevy Integration

**Workflow:**
1. Generate sprites using CLI
2. Review and curate in file explorer
3. Manually copy to Bevy `assets/sprites/`
4. Reference in Bevy code

**Directory Structure:**
```
output/
  characters/
    knight_01.png
    knight_02.png
  items/
    potion_01.png
  environments/
    tiles_01.png
```

## Implementation Steps

### Phase 1: Setup (Week 1)
1. Install Automatic1111 WebUI on DGX-Spark
2. Download and configure pixel art models
3. Test basic generation through web UI
4. Verify GPU utilization and performance

### Phase 2: CLI Development (Week 1-2)
1. Create Python CLI project structure
2. Implement A1111 API client
3. Build prompt templates
4. Add basic output organization
5. Test end-to-end workflow

### Phase 3: Documentation (Week 2)
1. Write user guide
2. Create prompt template library
3. Document Bevy integration workflow
4. Create troubleshooting guide

## Pros

- **Fast to implement**: 1-2 weeks to working system
- **Low complexity**: Minimal custom code
- **Proven tools**: Battle-tested components
- **Easy to learn**: Simple CLI interface
- **Quick iteration**: Immediate feedback
- **Low maintenance**: Mostly existing tools

## Cons

- **Manual steps**: Requires human intervention for Bevy integration
- **Limited automation**: No batch workflows
- **No custom training**: Stuck with pre-trained models
- **Scalability**: Not suitable for large teams
- **Model switching**: Manual process
- **Workflow inflexibility**: Hard to customize

## When to Use

- Prototyping and proof-of-concept
- Solo developers or tiny teams
- Budget/time constrained projects
- Learning and experimentation
- Validating requirements before larger investment

## Migration Path

This stack can evolve into Proposal 2 by:
1. Adding LoRA training pipeline
2. Replacing A1111 with ComfyUI
3. Implementing MCP integration
4. Expanding CLI into full orchestrator

---

# Proposal 2: Balanced Production Stack

## Philosophy

Build a production-ready system with custom training capabilities and semi-automated Bevy integration. Balance between development effort and feature richness. This is the **recommended** approach for most game studios.

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                       NVIDIA DGX-Spark                            │
├──────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌────────────────────────────────────────────────────┐          │
│  │  Model Storage                                      │          │
│  │  - Base SDXL model                                  │          │
│  │  - Custom LoRA adaptors                             │          │
│  │  - Embeddings & configs                             │          │
│  └──────────────┬─────────────────────────────────────┘          │
│                 │                                                  │
│  ┌──────────────▼─────────────────────────────────────┐          │
│  │  ComfyUI Inference Engine                           │          │
│  │  - Node-based workflows                             │          │
│  │  - Custom nodes for sprites                         │          │
│  │  - REST API server                                  │          │
│  │  - Batch processing                                 │          │
│  └──────────────┬─────────────────────────────────────┘          │
│                 │                                                  │
│  ┌──────────────▼─────────────────────────────────────┐          │
│  │  Training Pipeline (Optional)                       │          │
│  │  - Kohya_ss for LoRA training                       │          │
│  │  - Dataset preparation tools                        │          │
│  │  - Training monitoring                              │          │
│  └────────────────────────────────────────────────────┘          │
│                                                                    │
│  ┌──────────────────────────────────────────────────┐            │
│  │  FastAPI Orchestration Layer                      │            │
│  │  - Workflow management                             │            │
│  │  - Job queueing                                    │            │
│  │  - Asset post-processing                           │            │
│  │  - MCP server integration                          │            │
│  └──────────────┬─────────────────────────────────────┘          │
│                 │                                                  │
└─────────────────┼──────────────────────────────────────────────────┘
                  │
                  │ MCP Protocol
                  │
          ┌───────▼────────┐
          │  bevy_brp_mcp  │
          │  MCP Server    │
          └───────┬────────┘
                  │
                  ▼
          ┌───────────────┐
          │  Bevy Project │
          │  - Auto asset │
          │    placement  │
          │  - Live reload│
          └───────────────┘
```

## Components

### 1. ComfyUI Inference Engine

**Why ComfyUI:**
- 2x faster than A1111 in benchmarks
- Node-based workflows enable complex pipelines
- Better for automation and scripting
- Custom nodes for sprite-specific processing
- Active development

**Setup:**
```bash
git clone https://github.com/comfyanonymous/ComfyUI.git
cd ComfyUI
python -m venv venv
source venv/bin/activate
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121
pip install -r requirements.txt
python main.py --listen --port 8188
```

**Custom Nodes:**
- Sprite sheet assembler node
- Pixel perfect scaling node
- Palette optimization node
- Batch character pose generator

### 2. Model Management

**Base Models:**
- SDXL 1.0 base model
- Pixel Art Diffusion XL checkpoint

**LoRA Library:**
```
models/
  lora/
    style_16bit.safetensors
    style_32bit.safetensors
    character_fantasy.safetensors
    environment_dungeon.safetensors
    items_weapons.safetensors
```

**Version Control:**
- Git LFS for model files
- Metadata tracking (training params, source datasets)
- Performance benchmarks per model

### 3. LoRA Training Pipeline

**Kohya_ss Integration:**
```bash
git clone https://github.com/bmaltais/kohya_ss.git
cd kohya_ss
./setup.sh
```

**Training Workflow:**
1. Dataset preparation
   - Image collection/curation
   - Auto-captioning (BLIP, CLIP interrogator)
   - Quality filtering
   - Resolution standardization

2. Training configuration
   - Learning rate: 1e-4 to 1e-5
   - Batch size: 4-8 (depending on VRAM)
   - Steps: 2000-5000
   - LoRA rank: 32-128

3. Training execution
   - Monitor loss curves
   - Generate samples every N steps
   - Early stopping on quality plateau

4. Model validation
   - Test prompts suite
   - Visual quality assessment
   - Integration testing

### 4. FastAPI Orchestration Layer

**Core Services:**

```python
# API structure
app/
  main.py                 # FastAPI app
  models/
    schemas.py            # Pydantic models
  services/
    comfyui_client.py     # ComfyUI API client
    job_manager.py        # Job queue and status
    sprite_processor.py   # Post-processing
    asset_manager.py      # File organization
  mcp/
    server.py             # MCP server implementation
    bevy_client.py        # Bevy integration client
  workflows/
    templates/            # ComfyUI workflow JSON templates
```

**Key Features:**
- RESTful API for job submission
- WebSocket for progress updates
- Job queue with priorities
- Batch processing
- Retry logic and error handling
- Asset versioning

**Example Endpoints:**
```
POST   /api/v1/generate/sprite
POST   /api/v1/generate/batch
GET    /api/v1/jobs/{job_id}
GET    /api/v1/jobs/{job_id}/result
POST   /api/v1/models/lora/train
GET    /api/v1/models/list
POST   /api/v1/bevy/deploy
```

### 5. MCP Integration Layer

**bevy_brp_mcp Setup:**
```toml
# In Bevy project Cargo.toml
[dependencies]
bevy_brp_mcp = "0.1"
```

**DGX-Pixels MCP Server:**
```python
from fastapi import FastAPI
from fastmcp import FastMCP

app = FastAPI()
mcp = FastMCP(app)

@mcp.tool()
async def generate_sprite(
    prompt: str,
    style: str = "16bit",
    size: tuple[int, int] = (32, 32)
):
    """Generate a pixel art sprite and return asset path."""
    # Implementation
    pass

@mcp.tool()
async def deploy_to_bevy(
    asset_path: str,
    bevy_project_path: str,
    category: str = "sprites"
):
    """Deploy generated asset to Bevy project."""
    # Implementation
    pass
```

**Integration Benefits:**
- AI assistants can trigger generation
- Automatic asset deployment
- Bevy can query available assets
- Bidirectional communication

### 6. Post-Processing Pipeline

**Automated Steps:**
1. **Color Quantization**: Reduce to optimal palette
2. **Upscaling**: Pixel-perfect integer scaling
3. **Format Conversion**: PNG optimization
4. **Metadata Embedding**: Generation params, licensing
5. **Sprite Sheet Assembly**: Multiple frames → atlas
6. **Variant Generation**: Color swaps, mirroring

**Python Implementation:**
```python
from PIL import Image
import numpy as np

def quantize_colors(image: Image, num_colors: int = 16):
    """Reduce to optimal palette."""
    return image.quantize(colors=num_colors)

def pixel_perfect_scale(image: Image, scale: int):
    """Scale without blur."""
    return image.resize(
        (image.width * scale, image.height * scale),
        Image.NEAREST
    )

def assemble_sprite_sheet(images: list[Image], cols: int):
    """Combine into texture atlas."""
    # Implementation
    pass
```

## Implementation Steps

### Phase 1: Core Infrastructure (Weeks 1-2)
1. Install ComfyUI on DGX-Spark
2. Set up base models and initial LoRAs
3. Create ComfyUI workflows for common sprite types
4. Test and benchmark performance

### Phase 2: API Layer (Weeks 2-3)
1. Build FastAPI application structure
2. Implement ComfyUI client
3. Create job queue system
4. Add post-processing pipeline
5. Write comprehensive tests

### Phase 3: Training Pipeline (Week 3-4)
1. Set up Kohya_ss
2. Create dataset preparation scripts
3. Implement training automation
4. Build validation workflow
5. Train initial custom LoRAs

### Phase 4: Bevy Integration (Week 4-5)
1. Set up bevy_brp_mcp in test project
2. Implement MCP server in DGX-Pixels
3. Create asset deployment automation
4. Test end-to-end workflow
5. Build example Bevy game integration

### Phase 5: Polish and Docs (Week 5-6)
1. Create web UI dashboard (optional)
2. Write comprehensive documentation
3. Create tutorial videos
4. Build prompt library
5. Performance optimization

## Pros

- **Custom training**: LoRA fine-tuning for game-specific styles
- **Production ready**: Can handle team workflows
- **Automated Bevy integration**: MCP reduces manual steps
- **Scalable**: Job queue handles concurrent requests
- **Flexible**: ComfyUI workflows are highly customizable
- **Fast inference**: 2x speed improvement over A1111
- **Maintainable**: Clean architecture, well-documented

## Cons

- **Higher complexity**: More components to manage
- **Longer development**: 4-6 weeks to full deployment
- **Learning curve**: ComfyUI and MCP require understanding
- **More maintenance**: Custom code needs updates

## When to Use

- **Small to medium game studios**
- **Projects requiring consistent art style**
- **Teams of 2-10 developers**
- **Games with significant sprite needs**
- **When quality > speed-to-market**
- **Projects with 6+ month timeline**

## Performance Expectations

**On NVIDIA DGX-Spark:**
- **Inference**: 3-5 seconds per 1024x1024 image (SDXL + LoRA)
- **Batch generation**: 20-30 sprites per minute
- **LoRA training**: 2-4 hours for 50 images, 3000 steps
- **Concurrent jobs**: 4-6 simultaneous generations (with queuing)

---

# Proposal 3: Advanced Enterprise Stack

## Philosophy

Build a fully-featured, production-grade system with advanced training capabilities, custom UI, full automation, and enterprise features. Suitable for large studios with dedicated infrastructure teams.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                          NVIDIA DGX-Spark Cluster                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────┐             │
│  │  Model Registry & MLOps                             │             │
│  │  - Model versioning (DVC/MLflow)                    │             │
│  │  - Experiment tracking                              │             │
│  │  - A/B testing framework                            │             │
│  │  - Performance monitoring                           │             │
│  └──────────────┬─────────────────────────────────────┘             │
│                 │                                                     │
│  ┌──────────────▼─────────────────────────────────────┐             │
│  │  Multi-Model Inference Cluster                      │             │
│  │  - Model serving (TorchServe/Triton)                │             │
│  │  - Load balancing                                   │             │
│  │  - Dynamic model loading                            │             │
│  │  - GPU scheduling                                   │             │
│  └──────────────┬─────────────────────────────────────┘             │
│                 │                                                     │
│  ┌──────────────▼─────────────────────────────────────┐             │
│  │  Advanced Training Pipeline                         │             │
│  │  - Distributed training support                     │             │
│  │  - Hyperparameter optimization                      │             │
│  │  - Synthetic data generation                        │             │
│  │  - Active learning loop                             │             │
│  └────────────────────────────────────────────────────┘             │
│                                                                       │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            │ gRPC/REST
                            │
┌───────────────────────────▼─────────────────────────────────────────┐
│                     Application Layer (Kubernetes)                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────┐            │
│  │  API Gateway (Kong/Traefik)                          │            │
│  │  - Rate limiting, auth, routing                      │            │
│  └──────────────┬──────────────────────────────────────┘            │
│                 │                                                     │
│        ┌────────┴────────┬──────────────┬─────────────┐             │
│        │                 │              │             │             │
│  ┌─────▼─────┐   ┌──────▼─────┐  ┌─────▼────┐  ┌────▼─────┐       │
│  │ FastAPI   │   │ WebSocket  │  │ Job      │  │ Asset    │       │
│  │ REST API  │   │ Server     │  │ Queue    │  │ Storage  │       │
│  │ Service   │   │ (progress) │  │ (Celery) │  │ (MinIO)  │       │
│  └─────┬─────┘   └──────┬─────┘  └─────┬────┘  └────┬─────┘       │
│        │                │              │             │             │
│  ┌─────▼────────────────▼──────────────▼─────────────▼─────┐       │
│  │           Central Event Bus (Redis/RabbitMQ)             │       │
│  └──────────────────────────────────────────────────────────┘       │
│                                                                       │
│  ┌─────────────────────────────────────────────────────┐            │
│  │  Processing Services                                 │            │
│  │  - Post-processing workers                           │            │
│  │  - Quality validation                                │            │
│  │  - Format conversion                                 │            │
│  │  - Sprite sheet assembly                             │            │
│  └──────────────────────────────────────────────────────┘            │
│                                                                       │
│  ┌─────────────────────────────────────────────────────┐            │
│  │  Custom Web UI (React/Vue)                           │            │
│  │  - Project management                                │            │
│  │  - Prompt library                                    │            │
│  │  - Asset browser                                     │            │
│  │  - Training dashboard                                │            │
│  │  - Analytics & reporting                             │            │
│  └──────────────────────────────────────────────────────┘            │
│                                                                       │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            │ MCP Protocol
                            │
                    ┌───────▼────────┐
                    │  MCP Gateway   │
                    │  - Multi-tenant │
                    │  - Permissions  │
                    └───────┬────────┘
                            │
                ┌───────────┴───────────┐
                │                       │
        ┌───────▼────────┐      ┌──────▼──────┐
        │ bevy_brp_mcp   │      │ Unity/Godot │
        │ (Bevy)         │      │ MCP Clients │
        └────────────────┘      └─────────────┘
```

## Components

### 1. Model Registry & MLOps

**DVC (Data Version Control):**
- Track model versions with Git
- Large file storage
- Experiment lineage

**MLflow:**
- Experiment tracking
- Model registry
- Deployment management
- A/B testing support

**Features:**
- Automated model evaluation
- Champion/challenger comparison
- Gradual rollout
- Rollback capabilities

### 2. Multi-Model Inference Cluster

**NVIDIA Triton Inference Server:**
- Optimized for NVIDIA GPUs
- Multi-model serving
- Dynamic batching
- Model ensemble support

**Alternatively: TorchServe**
- PyTorch-native serving
- Model archiving
- Metrics and logging

**Benefits:**
- Load multiple models concurrently
- Automatic scaling
- FP4 optimization support
- High throughput

### 3. Advanced Training Pipeline

**Features:**
1. **Distributed Training**: Multi-GPU LoRA training
2. **Hyperparameter Optimization**: Optuna integration
3. **Synthetic Data Generation**: Use existing models to generate training data
4. **Active Learning**: User feedback → retraining loop
5. **Dataset Management**: Version control, quality metrics
6. **Training Monitoring**: Real-time loss curves, sample generation

**Tech Stack:**
- PyTorch Lightning for training
- Optuna for hyperparameter search
- Weights & Biases for monitoring
- Ray for distributed computing

### 4. Application Layer

**Microservices Architecture:**
- API Gateway for routing and auth
- Multiple FastAPI services
- Celery for async job processing
- Redis for caching and pub/sub
- MinIO for object storage
- PostgreSQL for metadata

**Services:**
- **auth-service**: User authentication, API keys
- **job-service**: Job submission, queue management
- **inference-service**: Model inference coordination
- **training-service**: Training job orchestration
- **asset-service**: Asset storage and retrieval
- **mcp-service**: MCP server implementation
- **notification-service**: Webhooks, notifications

### 5. Custom Web UI

**Features:**
- **Project Management**: Organize by game/project
- **Prompt Library**: Reusable prompt templates
- **Asset Browser**: Search, filter, tag assets
- **Training Dashboard**: Monitor training jobs
- **Analytics**: Usage metrics, cost tracking
- **Team Collaboration**: Share assets, reviews
- **API Key Management**: For CI/CD integration

**Tech Stack:**
- React + TypeScript
- Material UI or Tailwind CSS
- React Query for API calls
- WebSocket for real-time updates

### 6. Quality Assurance

**Automated Validation:**
- Resolution check
- Artifact detection
- Style consistency scoring
- Color palette validation
- Animation frame verification

**Human Review Workflow:**
- Approval queue
- Batch approval
- Rejection with feedback
- Retraining triggers

### 7. CI/CD Integration

**GitHub Actions / GitLab CI:**
```yaml
# .github/workflows/generate-assets.yml
name: Generate Game Assets

on:
  push:
    paths:
      - 'asset-requests/**'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Generate sprites
        run: |
          curl -X POST https://dgx-pixels.studio/api/v1/generate/batch \
            -H "Authorization: Bearer ${{ secrets.DGX_PIXELS_API_KEY }}" \
            -d @asset-requests/batch.json
      - name: Download results
        run: # ...
      - name: Commit to repo
        run: # ...
```

### 8. Multi-Engine Support

**Bevy Integration** (primary)
**Unity Support** (via MCP)
**Godot Support** (via MCP)
**Unreal Support** (custom plugin)

Each engine gets:
- MCP client library
- Asset importer plugin
- Live reload support
- Metadata handling

## Implementation Steps

### Phase 1: Infrastructure (Weeks 1-3)
1. Set up Kubernetes cluster
2. Deploy model serving (Triton)
3. Configure MLflow and DVC
4. Set up monitoring (Prometheus, Grafana)
5. Deploy message queue and cache

### Phase 2: Core Services (Weeks 3-6)
1. Build microservices
2. Implement API gateway
3. Create job queue system
4. Build asset storage service
5. Implement authentication

### Phase 3: Advanced Training (Weeks 6-8)
1. Distributed training setup
2. Hyperparameter optimization
3. Active learning pipeline
4. Dataset management tools
5. Training automation

### Phase 4: Web UI (Weeks 8-10)
1. Design system and mockups
2. Build React application
3. Implement all features
4. User testing and refinement

### Phase 5: Game Engine Integration (Weeks 10-11)
1. Bevy MCP integration
2. Unity plugin development
3. Godot plugin development
4. Testing and documentation

### Phase 6: Polish & Launch (Week 12)
1. Performance optimization
2. Security audit
3. Documentation
4. Training materials
5. Deployment automation

## Pros

- **Enterprise-grade**: Scalable, reliable, maintainable
- **Full automation**: Minimal manual intervention
- **Multi-tenant**: Support multiple teams/projects
- **Advanced training**: Custom models with optimal parameters
- **Multi-engine**: Not locked to Bevy
- **Analytics**: Deep insights into usage and costs
- **CI/CD integration**: Assets generated in pipeline
- **High performance**: Optimized for throughput
- **Quality control**: Automated and human review

## Cons

- **High complexity**: Many moving parts
- **Long development time**: 8-12 weeks minimum
- **High maintenance**: Requires dedicated team
- **Infrastructure costs**: Beyond just hardware
- **Steep learning curve**: For both developers and users
- **Over-engineered**: For small projects

## When to Use

- **Large game studios** (50+ developers)
- **Multiple concurrent projects**
- **High sprite volume** (thousands per project)
- **Teams need self-service**
- **Budget for infrastructure**
- **Long-term investment** (multi-year)
- **Compliance requirements** (audit trails, security)

## Performance Expectations

**On NVIDIA DGX-Spark (single node):**
- **Inference**: 1-2 seconds per image (Triton optimized)
- **Throughput**: 100+ sprites per minute
- **Concurrent users**: 20-30
- **Training**: Distributed training across GPUs
- **Uptime**: 99.9% SLA

---

# Proposal 2B: Rust TUI + Python Backend (NEW RECOMMENDED)

## Philosophy

Combine **Rust's performance and type safety** for the user interface with **Python's ML ecosystem** for AI workloads. This hybrid approach delivers a responsive, low-latency TUI while leveraging mature AI libraries. Supports side-by-side comparison of pre-trained and custom models.

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    NVIDIA DGX-Spark                          │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────┐                                     │
│  │   Rust TUI App      │  12MB, 60+ FPS                      │
│  │   - ratatui         │  Sixel image preview               │
│  │   - ZMQ client      │  Keyboard-driven                    │
│  │   - Live preview    │  Model comparison                  │
│  └──────┬──────────────┘                                     │
│         │ ZeroMQ (REQ-REP + PUB-SUB)                         │
│         │ tcp://localhost:5555-5556                          │
│         │ <1ms latency                                       │
│         │                                                     │
│  ┌──────▼──────────────┐                                     │
│  │  Python Worker      │  150MB baseline                     │
│  │  - ZMQ server       │  Job queue mgmt                     │
│  │  - ComfyUI client   │  Progress pub/sub                  │
│  │  - Model registry   │  LoRA management                    │
│  └──────┬──────────────┘                                     │
│         │ HTTP/WebSocket                                     │
│         │                                                     │
│  ┌──────▼──────────────┐                                     │
│  │   ComfyUI           │  8GB+ (with models)                 │
│  │   - SDXL base       │  Multiple checkpoints               │
│  │   - Custom LoRAs    │  Workflow templates                │
│  │   - Pre-trained     │  Side-by-side gen                  │
│  └─────────────────────┘                                     │
│                                                               │
└──────────────────────────────────────────────────────────────┘
         │ MCP Protocol (optional)
         ▼
  ┌───────────────┐
  │  bevy_brp_mcp │
  │  Bevy Project │
  └───────────────┘
```

## Key Innovation: Side-by-Side Models

**Pre-trained AND Custom Models Coexist**:
- Load multiple models simultaneously
- Compare outputs in real-time
- Vote/rate preferred results
- Track which model wins for different prompts
- Build model preference history

**Example Workflow**:
```
1. User enters prompt: "knight character sprite"
2. Presses [C] for Compare Mode
3. Selects 3 models:
   - SDXL Base (pre-trained)
   - SDXL + 16bit_rpg (custom LoRA)
   - SDXL + fantasy_char (custom LoRA)
4. TUI generates with all 3 in parallel
5. Results displayed side-by-side
6. User selects best → tracked for analytics
```

See `docs/08-tui-design.md` for complete TUI mockups.

## Components

### 1. Rust TUI (ratatui)

**Features**:
- 60+ FPS rendering
- <50ms input latency
- Sixel/halfblock image preview
- Real-time progress bars
- Model comparison interface
- GPU metrics dashboard
- Gallery browser

**Dependencies**:
```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
zmq = "0.10"
rmp-serde = "1.3"  # MsgPack serialization
tokio = { version = "1", features = ["full"] }
image = "0.25"
ratatui-image = "2.0"
serde = { version = "1.0", features = ["derive"] }
```

**Memory Usage**: ~12MB (vs 50-100MB for Python TUI)

### 2. Python Worker (ZeroMQ Server)

**Responsibilities**:
- Receive generation requests
- Queue management
- ComfyUI orchestration
- Progress publishing
- Model registry
- Result delivery

**Communication**:
- **REQ-REP**: Commands (generate, status, list models)
- **PUB-SUB**: Real-time updates (progress, previews, metrics)
- **Serialization**: MsgPack (faster than JSON)

See `docs/07-rust-python-architecture.md` for detailed communication patterns.

### 3. ComfyUI + Model Library

**Base Models**:
- SDXL 1.0 base (~8GB)
- Pre-trained pixel art checkpoints (~2GB each)

**LoRA Collection**:
- Custom trained LoRAs (400MB each)
- Organized by style (16bit, 32bit, 8bit)
- Version controlled with metadata

**Workflow Templates**:
- Single sprite generation
- Animation frame batch
- Tileset generation (seamless)
- Side-by-side comparison

### 4. Optional: PyO3 Native Extensions

For performance-critical paths:
```rust
// Rust implementation of color quantization
#[pyfunction]
fn quantize_colors_fast(img: &[u8], colors: usize) -> PyResult<Vec<u8>> {
    // 10-100x faster than PIL
}
```

Built with maturin, deployed as Python module.

## Implementation Steps

### Phase 1: Core (Weeks 1-2)
1. Install ComfyUI using dgx-spark-playbooks
2. Download SDXL + pixel art models
3. Implement Python ZeroMQ worker
4. Test generation via Python CLI

### Phase 2: Rust TUI (Weeks 2-3)
1. Create Rust project structure
2. Implement ZeroMQ client
3. Build basic TUI screens:
   - Generation
   - Queue
   - Models
4. Add image preview (ratatui-image)

### Phase 3: Model Management (Week 3-4)
1. LoRA loading/unloading
2. Model comparison workflow
3. Comparison result tracking
4. Model performance analytics

### Phase 4: Training Integration (Week 4-5)
1. Set up Kohya_ss
2. Train first custom LoRA
3. A/B test vs pre-trained
4. Document training workflow

### Phase 5: Polish & Deploy (Week 5-6)
1. MCP integration for Bevy
2. Gallery and asset management
3. Configuration system
4. Documentation and examples

## Pros

**Performance**:
- TUI: 60+ FPS, <10MB memory
- Communication: <1ms latency (ZeroMQ)
- No Python GIL bottlenecks in UI

**Developer Experience**:
- Type-safe Rust for UI logic
- Python flexibility for AI
- Best tool for each job
- Clean separation of concerns

**User Experience**:
- Fast, responsive interface
- Real-time previews
- Side-by-side model comparison
- Keyboard-driven workflow

**Flexibility**:
- Pre-trained models for quick start
- Custom training for production quality
- Compare models objectively
- Track model performance

**Integration**:
- Leverages dgx-spark-playbooks (ComfyUI)
- Can contribute back as new playbook
- Reuses community workflows

## Cons

- **Two languages**: Rust + Python to maintain
- **Learning curve**: Rust for contributors
- **Build complexity**: Cargo + maturin setup
- **Debugging**: Cross-language can be tricky

## When to Use

**Perfect for**:
- Solo developers or small teams (2-5)
- Need fast, responsive interface
- Want to compare models scientifically
- Comfortable with terminal UIs
- Value performance and efficiency

**Not ideal if**:
- Team requires web UI
- Need multi-tenant support
- Have Python-only developers
- Need Windows support (Rust TUI limited)

## Performance Expectations

**On NVIDIA DGX-Spark**:
- **TUI Startup**: <300ms (vs 2-3s Python)
- **Generation**: 12-15s per sprite (SDXL + LoRA)
- **Model Comparison**: 36-45s for 3 models
- **Memory**: ~20GB total (8GB models + 12GB overhead)
- **Concurrent**: 1 active + 5 queued jobs

**Side-by-Side Comparison**:
- Generate 2-4 models in sequence
- Each takes 12-15s
- Total: 24-60s for full comparison
- Results cached for instant re-comparison

## Migration Path

**From Proposal 1**:
- Keep A1111/ComfyUI backend
- Replace Python CLI with Rust TUI
- Add model comparison features

**To Proposal 3**:
- Rust TUI remains as client
- Backend scales to microservices
- Add web UI for team members

---

# Recommendation

## NEW Default: **Proposal 2B (Rust TUI + Python)**

**Rationale**:
- **Best performance**: 60 FPS TUI, <1ms IPC
- **Side-by-side models**: Compare pre-trained vs custom scientifically
- **Developer friendly**: Leverages existing playbooks
- **Production ready**: 5-6 weeks to MVP
- **Scales well**: Can add services later
- **Modern stack**: Rust + Python + ZeroMQ is proven

**Feature Comparison**:
- ✅ Custom training (LoRA)
- ✅ Model comparison (unique!)
- ✅ Fast, responsive UI
- ✅ Low resource overhead
- ✅ MCP integration
- ✅ Open source everything

**Start with Proposal 1 if:**
- Need to validate concept in <1 week
- Python-only team
- Just want to test pre-trained models

**Use Proposal 2 (Original) if:**
- Prefer web UI over TUI
- Don't need model comparison
- Team unfamiliar with Rust

**Choose Proposal 3 if:**
- Large studio with dedicated infra team (50+ devs)
- Multiple concurrent projects
- High volume asset generation (thousands/week)
- Budget for 8-12 week development
- Need enterprise features (multi-tenancy, audit trails, etc.)

## Updated Comparison Matrix (with Proposal 2B)

| Feature | Proposal 1 | Proposal 2 | **Proposal 2B** | Proposal 3 |
|---------|-----------|-----------|-----------------|-----------|
| **Time to MVP** | 1-2 weeks | 4-6 weeks | **5-6 weeks** | 8-12 weeks |
| **UI Type** | Python CLI | Web/ComfyUI | **Rust TUI** | Web + TUI |
| **Performance** | Medium | Medium | **High (60 FPS)** | High |
| **Model Comparison** | No | Manual | **Built-in** | Advanced |
| **Memory (UI)** | 50MB | 100MB+ | **12MB** | Varies |
| **Training** | No | LoRA | **LoRA + Comparison** | Full pipeline |
| **Bevy Integration** | Manual | MCP | **MCP** | Full automation |
| **For Teams** | Solo | 2-10 | **2-5** | 50+ |

## Next Steps

1. **Review proposals** and new Proposal 2B
2. **Consider team skills** (Rust vs Python preference)
3. **Evaluate TUI vs web UI** requirements
4. **Select architecture proposal**
5. **Follow implementation plan**:
   - Proposal 1: `docs/06-implementation-plan.md` § Path A
   - Proposal 2B: `docs/07-rust-python-architecture.md` + `docs/08-tui-design.md`
   - Proposal 3: `docs/06-implementation-plan.md` § Path C

See `docs/11-playbook-contribution.md` for contributing to dgx-spark-playbooks.
