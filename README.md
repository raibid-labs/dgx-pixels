# DGX-Pixels: AI Pixel Art Generation Stack

An open-source AI-powered pixel art generation system optimized for the NVIDIA DGX-Spark, designed to accelerate game asset creation with seamless integration into the Bevy game engine.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
  - [Prerequisites](#prerequisites)
  - [Running the TUI](#running-the-tui)
  - [Backend Setup](#backend-setup-python-worker--comfyui)
- [Architecture](#architecture)
  - [Rust TUI + Python Backend](#new-rust-tui--python-backend-recommended)
  - [Alternative: Balanced Production Stack](#alternative-balanced-production-stack)
- [Documentation](#documentation)
  - [Core Documentation](#core-documentation)
  - [Rust + Python Stack](#new-rust--python-stack)
  - [Project Management & Operations](#new-project-management--operations)
  - [TUI Modernization](#new-tui-modernization-bevy-ratatui-migration)
  - [Session Logs & Completion Reports](#session-logs--completion-reports)
  - [Quick Links](#quick-links)
- [Technology Stack](#technology-stack)
- [Training Custom Models](#training-custom-models)
- [Bevy Integration](#bevy-integration)
- [Performance Benchmarks](#performance-benchmarks)
- [Project Structure](#project-structure)
- [Roadmap](#roadmap)
- [Use Cases and Examples](#use-cases-and-examples)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)
- [Resources](#resources)
- [Support](#support)
- [Status](#status)

## Overview

DGX-Pixels leverages state-of-the-art diffusion models (Stable Diffusion XL) with custom LoRA fine-tuning to generate high-quality pixel art sprites for game development. The system is designed to run on **NVIDIA DGX-Spark (GB10 Grace Blackwell Superchip)**, utilizing its single powerful GPU with 128GB unified memory architecture for fast inference and efficient model training.

**Hardware Note**: This system targets the DGX-Spark GB10 (single GPU, unified memory) rather than multi-GPU datacenter systems. This architecture provides unique advantages for interactive pixel art generation, including zero-copy image transfers and simplified deployment. See [Hardware Specification](docs/hardware.md) and [ADR 0001](docs/adr/0001-dgx-spark-not-b200.md) for details.

### Key Features

- **AI-Powered Generation**: Stable Diffusion XL with pixel art-specialized LoRA models
- **Hardware Optimized**: Maximizes NVIDIA DGX-Spark's 1000 TOPS compute and FP4 precision support
- **Bevy Integration**: Direct integration via Model Context Protocol (MCP) for automated asset deployment
- **Custom Training**: LoRA fine-tuning pipeline for consistent, game-specific art styles
- **Production Ready**: Multiple architecture proposals from rapid prototyping to enterprise scale
- **100% Open Source**: All components use open-source tools and models

### Use Cases

- Character sprite generation (idle, walk, attack animations)
- Environment tiles and props
- Item and weapon sprites
- UI icons and effects
- Rapid prototyping and iteration
- Style-consistent asset expansion

## Quick Start

### Prerequisites

- NVIDIA DGX-Spark (GB10 Grace Blackwell Superchip) with Ubuntu/Linux
- Rust 1.75+ (for TUI frontend)
- Python 3.10+ (ARM64-compatible packages)
- CUDA 13.0+ (verified: 13.0.88)
- Driver 580.95.05+
- Docker 20.10+ and Docker Compose v2+ (for containerized deployment)
- 500GB+ storage
- [just](https://github.com/casey/just) command runner
- Bevy game engine (for integration)

### First-Time Setup

```bash
# Clone repository
git clone https://github.com/raibid-labs/dgx-pixels.git
cd dgx-pixels

# Initialize project (creates venv, installs Python dependencies, creates directories)
just init

# (Optional) Setup Docker environment for production deployment
just docker-setup
```

### Running the TUI

The Rust TUI is operational with dual-mode support:

```bash
# Run with integrated backend (recommended for development)
just debug

# Or run TUI only (requires separate backend)
just tui              # Classic ratatui mode (legacy)
just tui-bevy         # Bevy ECS mode (NEW - recommended)
just tui-bevy-release # Bevy ECS mode (release build, optimized)
```

**Bevy Mode Features** (currently available):
- ✅ 60 FPS terminal rendering
- ✅ Message-based input handling (<16ms latency)
- ✅ Screen navigation (Tab, 1-8 keys)
- ✅ Text entry with cursor editing (Generation screen)
- ✅ ZeroMQ backend integration (optional)
- ✅ Event-driven architecture (8 custom events)
- 🟢 Gallery and job tracking (partial)
- ⚪ Full screen implementations (in progress)

### Backend Setup (Python Worker + ComfyUI)

**Option 1: Integrated Development Mode** (Recommended)
```bash
# Starts both backend and TUI with live log monitoring
just debug
```

**Option 2: Manual Backend (for advanced debugging)**
```bash
# Start Python ZMQ worker (in first terminal)
source venv/bin/activate
python python/workers/generation_worker.py

# Start TUI (in second terminal)
just tui-bevy
```

**Option 3: Docker Deployment** (Production)
```bash
# Setup and start all services
just docker-setup
cd docker && docker compose up -d

# View service status
just docker-ps

# View logs
just docker-logs
```

### ComfyUI Setup

ComfyUI is required for image generation:

```bash
# ComfyUI is cloned during docker-setup
# To run manually:
cd ComfyUI
python main.py --listen 127.0.0.1 --port 8188

# Or via Docker
just docker-up  # ComfyUI runs as a service
```

### Quick Command Reference

```bash
just --list          # Show all available commands
just init            # First-time setup
just debug           # Run TUI + backend with debugging
just docker-setup    # Setup Docker environment
just docker-up       # Start Docker stack
just validate-gpu    # Check GPU/hardware
just hw-info         # Show hardware information
just test            # Run all tests
```

See [Implementation Plan](docs/06-implementation-plan.md) and [justfile](justfile) for detailed setup instructions and all available commands.

## Architecture

DGX-Pixels offers multiple architecture proposals:

1. **Rapid Prototyping** (1-2 weeks): Simple CLI + Automatic1111 for quick validation
2. **Balanced Production** (4-6 weeks): ComfyUI + FastAPI + MCP integration
3. **🆕 Rust TUI + Python** (5-6 weeks): Fast TUI with side-by-side model comparison **(NEW RECOMMENDED)**
4. **Advanced Enterprise** (8-12 weeks): Full microservices with Kubernetes, MLOps, and web UI

See [Architecture Proposals](docs/02-architecture-proposals.md) for detailed comparisons.

### NEW: Rust TUI + Python Backend (Recommended)

**Hybrid architecture combining Rust's performance with Python's AI ecosystem:**

```
┌──────────────────────────────────────┐
│       NVIDIA DGX-Spark                │
│  ┌───────────────────┐                │
│  │  Rust TUI         │ 60 FPS, 12MB   │
│  │  - ratatui        │ Sixel preview  │
│  │  - Live updates   │ Comparison UI  │
│  └────────┬──────────┘                │
│           │ ZeroMQ <1ms                │
│  ┌────────▼──────────┐                │
│  │  Python Worker    │ Job queue      │
│  │  - ZMQ server     │ Progress pub   │
│  └────────┬──────────┘                │
│           │ HTTP/WS                    │
│  ┌────────▼──────────┐                │
│  │  ComfyUI          │ SDXL + LoRAs   │
│  │  - Multiple models│ Workflows      │
│  └───────────────────┘                │
└──────────────────────────────────────┘
```

**Key Features**:
- **Side-by-side model comparison**: Test pre-trained vs custom LoRAs simultaneously
- **60+ FPS TUI**: Fast, responsive terminal interface
- **<1ms IPC**: ZeroMQ for near-instant communication
- **Leverages playbooks**: Uses dgx-spark-playbooks ComfyUI setup

See [Rust-Python Architecture](docs/07-rust-python-architecture.md) and [TUI Design](docs/08-tui-design.md).

### Alternative: Balanced Production Stack

```
┌─────────────────────────────────────┐
│       NVIDIA DGX-Spark              │
│  ┌──────────────────────────────┐  │
│  │  ComfyUI Inference Engine    │  │
│  │  + Custom LoRA Models         │  │
│  └──────────────┬───────────────┘  │
│                 │                    │
│  ┌──────────────▼───────────────┐  │
│  │  FastAPI Orchestration       │  │
│  │  + MCP Server                 │  │
│  └──────────────┬───────────────┘  │
└─────────────────┼───────────────────┘
                  │
                  │ MCP Protocol
                  │
          ┌───────▼────────┐
          │  bevy_brp_mcp  │
          └───────┬────────┘
                  │
                  ▼
          ┌───────────────┐
          │  Bevy Project │
          │  assets/      │
          └───────────────┘
```

## Documentation

### Core Documentation

1. **[Research Findings](docs/01-research-findings.md)** - Comprehensive research on AI pixel art generation, DGX-Spark capabilities, and integration technologies
2. **[Architecture Proposals](docs/02-architecture-proposals.md)** - Four detailed architecture proposals with pros/cons/timelines
3. **[Technology Deep Dive](docs/03-technology-deep-dive.md)** - In-depth technical documentation on SDXL, LoRA, ComfyUI, and optimizations
4. **[Bevy Integration](docs/04-bevy-integration.md)** - Complete guide for integrating with Bevy game engine
5. **[Training Roadmap](docs/05-training-roadmap.md)** - Strategy for training custom models and maintaining quality
6. **[Implementation Plan](docs/06-implementation-plan.md)** - Step-by-step implementation guide for all architecture paths

### NEW: Rust + Python Stack

7. **[Rust-Python Architecture](docs/07-rust-python-architecture.md)** - Hybrid Rust TUI + Python backend design with ZeroMQ IPC
8. **[TUI Design](docs/08-tui-design.md)** - Complete TUI mockups, workflows, and side-by-side model comparison
9. **[Playbook Contribution](docs/11-playbook-contribution.md)** - Contributing to dgx-spark-playbooks repository

### NEW: Project Management & Operations

10. **[Hardware Specification](docs/hardware.md)** - Verified DGX-Spark GB10 specifications and topology
11. **[Metrics Framework](docs/metrics.md)** - Performance, quality, and observability metrics
12. **[Roadmap](docs/ROADMAP.md)** - Milestone-based development roadmap (M0-M5)
13. **[RFD: GPT-5 Feedback](docs/rfds/gpt5-dgx-pixels.md)** - External review and recommendations
14. **[ADR 0001](docs/adr/0001-dgx-spark-not-b200.md)** - Hardware clarification: DGX-Spark vs DGX B200

### NEW: TUI Modernization (Bevy-Ratatui Migration)

15. **[RFD 0003: Bevy-Ratatui Migration](docs/rfds/0003-bevy-ratatui-migration.md)** - Complete migration strategy (18 workstreams)
16. **[Progress Report](docs/orchestration/tui-modernization/PROGRESS-REPORT.md)** - Real-time implementation status (M1 ✅ 100%, M2 🟢 75%)
17. **[Meta Orchestrator](docs/orchestration/tui-modernization/meta-orchestrator.md)** - Parallel execution coordination
18. **[Workstream Specs](docs/orchestration/tui-modernization/workstreams/)** - Detailed implementation guides (WS-01 through WS-18)

### Session Logs & Completion Reports

19. **[Session Logs](docs/session-logs/)** - Implementation session reports, completion summaries, and validation checklists
    - **[LoRA Setup](docs/session-logs/LORA-SETUP-COMPLETE.md)** - Pixel Art LoRA integration guide
    - **[Sixel Preview](docs/session-logs/SIXEL-PREVIEW-UPDATE.md)** - Terminal image preview implementation
    - **[AI Generation Fix](docs/session-logs/AI-GENERATION-FIX.md)** - Random seed and quality improvements
    - **[Workstream Completions](docs/session-logs/)** - WS-05 through WS-17 completion reports

### Quick Links

| Topic | Documentation |
|-------|--------------|
| Getting Started | [Implementation Plan § Quick Start](docs/06-implementation-plan.md#pre-implementation-checklist) |
| Architecture Selection | [Architecture Proposals § Comparison](docs/02-architecture-proposals.md#comparison-matrix) |
| Bevy Setup | [Bevy Integration § Setup](docs/04-bevy-integration.md#bevy-asset-system-basics) |
| Model Training | [Training Roadmap § Phase 2](docs/05-training-roadmap.md#phase-2-style-training-week-3-4) |
| API Reference | [Technology Deep Dive § FastAPI](docs/03-technology-deep-dive.md#fastapi-and-mcp) |
| Troubleshooting | [Implementation Plan § Troubleshooting](docs/06-implementation-plan.md#troubleshooting) |

## Technology Stack

### Core Technologies

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Base Model** | Stable Diffusion XL 1.0 | Image generation foundation |
| **Fine-tuning** | LoRA (Low-Rank Adaptation) | Custom style training |
| **Inference** | ComfyUI | Fast, flexible generation workflows |
| **Training** | Kohya_ss / Diffusers | LoRA training pipeline |
| **API** | FastAPI | REST API and orchestration |
| **Integration** | Model Context Protocol (MCP) | Bevy communication |
| **Game Engine** | Bevy 0.13+ | Target integration platform |

### Hardware Optimization

- **NVIDIA GB10 (Grace Blackwell)**: Single superchip with unified memory architecture
- **1000 TOPS Compute**: Ultra-fast inference (2-4s per sprite)
- **128GB Unified Memory**: Zero-copy CPU↔GPU transfers, multiple concurrent models
- **ARM Grace CPU**: 20 cores (Cortex-X925 + A725) for energy-efficient preprocessing
- **Compute Capability 12.1**: Latest Tensor Core features

See [Technology Deep Dive](docs/03-technology-deep-dive.md) for comprehensive technical details.

## Training Custom Models

Custom LoRA training dramatically improves generation quality:

**Benefits:**
- 80%+ reduction in post-processing time
- Consistent art style across all assets
- Character identity preservation
- Game-specific prompt understanding

**Requirements:**
- 50-100 reference images
- 2-4 hours training time (on DGX-Spark)
- ~$5-10 in compute costs

**Timeline:**
- Week 1-2: Test pre-trained models, collect references
- Week 3-4: Train general style LoRA
- Week 5-8: Train specialized models (characters, environments, items)
- Week 9-10: Character-specific models for consistency
- Week 11-12: Refinement based on production feedback

See [Training Roadmap](docs/05-training-roadmap.md) for detailed training strategy.

## Bevy Integration

### Manual Workflow

```bash
# Generate sprite
dgx-pixels generate character "medieval knight"

# Copy to Bevy assets
cp output/knight.png ~/my_game/assets/sprites/characters/

# Use in Bevy
commands.spawn(SpriteBundle {
    texture: asset_server.load("sprites/characters/knight.png"),
    ..default()
});
```

### Automated MCP Workflow

```rust
// Bevy: Enable MCP
use bevy_brp_mcp::BrpMcpPlugin;

App::new()
    .add_plugins(BrpMcpPlugin::default())
    .run();
```

```python
# DGX-Pixels: MCP tool
@mcp.tool()
async def generate_and_deploy(prompt: str, bevy_project: str):
    """Generate and auto-deploy to Bevy project."""
    # Generates sprite and places in bevy_project/assets/
    pass
```

See [Bevy Integration Guide](docs/04-bevy-integration.md) for complete details.

## Performance Benchmarks

On NVIDIA DGX-Spark GB10 (verified hardware):

| Operation | Expected Time | Details |
|-----------|---------------|---------|
| **Inference (SDXL + LoRA)** | 2-4s | 1024x1024 image @ FP16 |
| **Batch Generation** | 15-25/min | Multiple sprites (batch=8) |
| **LoRA Training** | 2-4 hours | 50 images, 3000 steps @ FP16 |
| **Model Loading** | <10s | SDXL base + LoRA in unified memory |
| **Zero-Copy Transfers** | <1μs | CPU↔GPU (unified memory advantage) |

**Unified Memory Benefits:**
- No CPU→GPU memory copies for image data
- Lower latency for preprocessing and preview
- Larger batch sizes (128GB shared pool)
- Simplified memory management

**Comparison to Manual Creation:**
- Traditional pixel art: 30-120 minutes per sprite
- AI generation + touch-up: 5-15 minutes per sprite
- **Time savings**: 70-90%

See [Metrics Framework](docs/metrics.md) for detailed performance targets.

## Project Structure

```
dgx-pixels/
├── README.md                 # This file
├── docs/                     # Comprehensive documentation
│   ├── 01-research-findings.md
│   ├── 02-architecture-proposals.md
│   ├── 03-technology-deep-dive.md
│   ├── 04-bevy-integration.md
│   ├── 05-training-roadmap.md
│   └── 06-implementation-plan.md
├── src/                      # Source code (to be implemented)
│   ├── api/                  # FastAPI application
│   ├── cli/                  # CLI tools
│   ├── training/             # Training scripts
│   └── processing/           # Post-processing pipeline
├── workflows/                # ComfyUI workflow templates
├── models/                   # Model storage
│   ├── checkpoints/          # Base models
│   ├── loras/                # Trained LoRAs
│   └── configs/              # Model configurations
└── examples/                 # Example Bevy integrations
```

## Roadmap

See [ROADMAP.md](docs/ROADMAP.md) for the complete milestone-based development plan.

### Current Milestones (Bevy-Ratatui Migration)

| Milestone | Status | Completion | Goal |
|-----------|--------|-----------|------|
| **M1 — Foundation** | ✅ Complete | 100% | Bevy runtime, ECS state, input systems, rendering pipeline |
| **M2 — Core Systems** | 🟢 In Progress | 75% | ZeroMQ integration, theme, event bus, image assets |
| **M3 — Screen Migration** | ⚪ Planned | 0% | Migrate 8 UI screens to Bevy systems |
| **M4 — Integration** | ⚪ Planned | 0% | Finalize dual-mode, deprecate classic, performance validation |

**Progress Summary**: 8 of 18 workstreams complete (44%). See [detailed progress report](docs/orchestration/tui-modernization/PROGRESS-REPORT.md).

### Recent Updates

- ✅ **Pixel Art LoRA Integration**: SDXL checkpoint + Pixel Art XL v1.1 LoRA for authentic pixel art generation
- ✅ **Sixel Preview**: Terminal image preview with WezTerm support, gallery navigation fixes
- ✅ **Random Seed Generation**: Fixed hardcoded seed issue for unique image generation
- ✅ **M1 Foundation Complete** (WS-01 through WS-04): Bevy 0.15 + bevy_ratatui 0.7 runtime operational
- ✅ **ECS State Migration**: 5 Resources + 2 Components with 37 unit tests (all passing)
- ✅ **Input System**: Message-based keyboard/navigation/text entry (<16ms latency)
- ✅ **Rendering Pipeline**: 60 FPS terminal rendering with placeholder screens
- ✅ **ZeroMQ Integration** (WS-05): Non-blocking polling, thread-safe backend communication
- ✅ **Theme System** (WS-07): Centralized AppTheme resource with 11 style methods
- ✅ **Event Bus** (WS-08): 8 custom events for navigation, generation, gallery
- 🟢 **M2 Core Systems**: 75% complete, WS-06 (Image Assets) pending
- 📊 **Test Coverage**: 122 tests passing, 82% coverage (target >80%)

## Use Cases and Examples

### Character Sprites

```bash
# Generate idle animation frames
dgx-pixels generate-animation \
  --prompt "fantasy knight character" \
  --frames 4 \
  --type idle \
  --output ./assets/characters/knight/
```

### Environment Tiles

```bash
# Generate seamless dungeon tiles
dgx-pixels generate-tileset \
  --prompt "stone dungeon floor" \
  --size 32 \
  --seamless \
  --variations 8
```

### Item Icons

```bash
# Batch generate item sprites
dgx-pixels batch items.txt \
  --style 16bit \
  --size 64 \
  --output ./assets/items/
```

See [Implementation Plan § Examples](docs/06-implementation-plan.md) for more use cases.

## Contributing

We welcome contributions! Please see CONTRIBUTING.md (coming soon) for guidelines.

Areas where we need help:
- Custom ComfyUI nodes for sprite-specific operations
- Bevy plugin development
- Training dataset curation
- Performance optimization
- Documentation improvements

## License

This project is released under the MIT License. See LICENSE for details.

### Component Licenses

- Stable Diffusion XL: CreativeML Open RAIL++-M License
- ComfyUI: GPL-3.0
- Diffusers: Apache 2.0
- Bevy: MIT/Apache 2.0
- FastAPI: MIT

All dependencies are open-source and permissively licensed.

## Acknowledgments

- **Stability AI** for Stable Diffusion XL
- **ComfyUI community** for the excellent inference tool
- **Bevy community** for the game engine
- **Civitai** for pixel art model hosting
- **Hugging Face** for model hosting and Diffusers library
- **NVIDIA** for DGX-Spark hardware

## Resources

### Documentation
- [Stable Diffusion](https://github.com/Stability-AI/stablediffusion)
- [ComfyUI](https://github.com/comfyanonymous/ComfyUI)
- [Bevy Engine](https://bevyengine.org/)
- [Diffusers](https://huggingface.co/docs/diffusers)
- [bevy_brp_mcp](https://crates.io/crates/bevy_brp_mcp)

### Communities
- [ComfyUI Discord](https://discord.gg/comfyui)
- [Bevy Discord](https://discord.gg/bevy)
- [r/StableDiffusion](https://reddit.com/r/stablediffusion)
- [Civitai](https://civitai.com/)

### Research
- "Generating Pixel Art Character Sprites using GANs" (2022)
- LoRA: Low-Rank Adaptation of Large Language Models
- Stable Diffusion XL Paper

## Support

For questions and support:
- **Documentation**: See [docs/](docs/) directory
- **Issues**: GitHub Issues (coming soon)
- **Discussions**: GitHub Discussions (coming soon)

## Status

**Project Status**: Implementation In Progress 🟢

**Current Focus**: Bevy-Ratatui Migration (Proposal 2B)
- M1 Foundation: ✅ Complete (100%)
- M2 Core Systems: 🟢 In Progress (75%)
- 122 tests passing, 82% coverage
- 60 FPS TUI operational

**Next Steps**:
1. ✅ ~~Complete M1 Foundation workstreams (WS-01 through WS-04)~~
2. 🟢 Complete M2 Core Systems (WS-06 Image Assets pending)
3. ⚪ Migrate 8 UI screens to Bevy systems (M3, WS-09 through WS-16)
4. ⚪ Complete integration and dual-mode finalization (M4, WS-17 through WS-18)

See [Progress Report](docs/orchestration/tui-modernization/PROGRESS-REPORT.md) for detailed status.

---

**Built with ❤️ for game developers who want to focus on creating games, not drawing every pixel.**