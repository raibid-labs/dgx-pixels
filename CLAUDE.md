# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**DGX-Pixels** is an AI-powered pixel art generation stack optimized for the NVIDIA DGX-Spark hardware, designed to generate game sprites and assets for Bevy game engine projects.

**Current Status**: Documentation Phase Complete ✅ - Implementation not yet started

The project is in its research and planning phase. All documentation is complete, but no code has been written yet. This is intentional - the comprehensive research and architecture proposals must be reviewed and a specific architecture path selected before implementation begins.

## Core Constraints

These are non-negotiable requirements that must be respected in all implementations:

1. **Hardware**: Must run on NVIDIA DGX-Spark (GB10 Grace Blackwell Superchip, 128GB unified memory, 1000 TOPS)
2. **Open Source Only**: All tools, libraries, and models must be open source (no proprietary APIs or closed models)
3. **Bevy Integration**: Primary target is Bevy game engine with MCP server integration
4. **Technology Stack**: Rust TUI (ratatui) + Python AI Backend + Stable Diffusion XL + LoRA fine-tuning + ComfyUI + ZeroMQ IPC

## Documentation Structure

The `docs/` directory contains comprehensive research and planning:

### Core Documentation
- **01-research-findings.md**: Deep research on AI models, DGX-Spark capabilities, Bevy integration, and tools
- **02-architecture-proposals.md**: Four complete architecture proposals (Rapid/Balanced/Rust+Python/Advanced) with timelines and trade-offs
- **03-technology-deep-dive.md**: Technical details on SDXL, LoRA training, ComfyUI, PyTorch optimizations
- **04-bevy-integration.md**: Complete integration guide for Bevy asset pipeline and MCP
- **05-training-roadmap.md**: 12-week training strategy for custom LoRA models
- **06-implementation-plan.md**: Step-by-step implementation guides for architecture paths

### NEW: Rust + Python Stack Documentation
- **07-rust-python-architecture.md**: Hybrid Rust TUI + Python backend design with ZeroMQ IPC patterns
- **08-tui-design.md**: Complete TUI mockups, workflows, and side-by-side model comparison feature
- **11-playbook-contribution.md**: Proposal for contributing to dgx-spark-playbooks repository

**Critical**: Read relevant documentation before implementing any component. The research phase identified best practices, pitfalls, and optimal approaches.

## Architecture Decision Required

Before writing any code, one of four architecture proposals must be selected:

1. **Proposal 1: Rapid** (1-2 weeks) - Automatic1111 + Simple CLI + Manual Bevy integration
   - Use for: Quick prototypes, validation, solo developers
   - Trade-offs: No training, manual workflows, limited scalability

2. **Proposal 2: Balanced** (4-6 weeks) - ComfyUI + FastAPI + MCP + LoRA Training
   - Use for: Small studios (2-10 devs), production projects
   - Trade-offs: Medium complexity, requires setup investment

**2B. Proposal 2B: Rust TUI + Python** (5-6 weeks) - ratatui TUI + ZeroMQ + Python Worker + ComfyUI [NEW RECOMMENDED]
   - Use for: Developers wanting fast, responsive UI with side-by-side model comparison
   - Key features: 60+ FPS TUI, <1ms IPC, Sixel image preview, compare pre-trained vs custom models
   - Trade-offs: Requires Rust knowledge, slightly longer initial setup than Proposal 2

3. **Proposal 3: Advanced** (8-12 weeks) - Full microservices + Kubernetes + Web UI + MLOps
   - Use for: Large studios (50+ devs), multiple projects
   - Trade-offs: High complexity, significant maintenance overhead

**Recommendation**: Proposal 2B (Rust TUI + Python) offers the best balance of performance, developer experience, and unique features like side-by-side model comparison. This architecture leverages dgx-spark-playbooks and provides a foundation for contributing back to the ecosystem.

See `docs/02-architecture-proposals.md` and `docs/07-rust-python-architecture.md` for detailed comparison matrices and decision criteria.

## Key Technical Decisions

These decisions were made after extensive research and should not be changed without strong justification:

### Model Architecture
- **Base Model**: Stable Diffusion XL 1.0 (NOT SD 1.5 - SDXL offers 3x larger UNet and better quality)
- **Fine-tuning Method**: LoRA (NOT full fine-tuning - LoRA is faster, uses less memory, produces smaller files)
- **Training Framework**: Kohya_ss or Diffusers (both support DGX-Spark optimizations)

### Inference Engine
- **Balanced/Advanced**: ComfyUI (2x faster than A1111, better for automation)
- **Rapid**: Automatic1111 (faster setup, good for prototyping)

### Integration Layer
- **Protocol**: Model Context Protocol (MCP) for Bevy communication
- **Bevy Library**: bevy_brp_mcp (enables AI assistants to control Bevy apps)
- **API Framework**: FastAPI (modern, async, auto-docs) for Proposal 2
- **IPC**: ZeroMQ (REQ-REP + PUB-SUB patterns, <1ms latency) for Proposal 2B

### Rust + Python Architecture (Proposal 2B)
- **Frontend**: Rust with ratatui TUI framework (60+ FPS rendering, Sixel image preview)
- **Backend**: Python worker with ZeroMQ server for job management
- **Communication**: ZeroMQ with MsgPack serialization
- **Side-by-Side Comparison**: Unique feature allowing comparison of pre-trained vs custom LoRA models simultaneously
- **Playbook Integration**: Leverages dgx-spark-playbooks ComfyUI setup

### Hardware Optimization
- Enable mixed precision training (FP16/FP4)
- Use xformers memory-efficient attention
- Leverage Tensor Cores for matrix operations
- Load multiple models in 128GB unified memory

## Implementation Guidelines

### When Starting Implementation

1. **Select architecture proposal** - Don't mix approaches, commit to one path
2. **Follow implementation plan** - Use the step-by-step guide in `docs/06-implementation-plan.md`
3. **Read technology deep-dive** - Understand SDXL, LoRA, ComfyUI before coding (see `docs/03-technology-deep-dive.md`)
4. **Respect training roadmap** - Custom models are essential for quality, follow the 12-week plan

### Project Structure (To Be Created)

**For Proposal 2B (Rust + Python):**
```
dgx-pixels/
├── rust/             # Rust TUI application
│   ├── src/
│   │   ├── main.rs        # TUI entry point
│   │   ├── ui/            # ratatui UI components
│   │   ├── zmq_client.rs  # ZeroMQ communication
│   │   └── image_preview.rs # Sixel rendering
│   └── Cargo.toml
├── python/           # Python backend worker
│   ├── workers/
│   │   ├── generation_worker.py  # ZMQ server + ComfyUI client
│   │   └── zmq_server.py         # Job queue management
│   ├── requirements.txt
│   └── pyproject.toml
├── workflows/        # ComfyUI workflow JSON templates
├── models/           # Model storage (use Git LFS)
│   ├── checkpoints/  # Base SDXL models
│   ├── loras/        # Trained LoRAs
│   └── configs/      # Model metadata
└── examples/         # Example Bevy integrations
```

**For Proposal 2 (Python only):**
```
dgx-pixels/
├── src/
│   ├── api/          # FastAPI orchestration layer
│   ├── cli/          # Command-line tools
│   ├── training/     # LoRA training scripts
│   └── processing/   # Post-processing pipeline
├── workflows/        # ComfyUI workflow JSON templates
├── models/           # Model storage
└── examples/         # Example Bevy integrations
```

### Critical Implementation Notes

**LoRA Training**:
- Dataset: 50-100 images minimum for style training
- Resolution: 1024x1024 for SDXL (not 512x512)
- Training time: 2-4 hours on DGX-Spark
- Don't skip training - pre-trained models won't match game art style

**ComfyUI Workflows**:
- Save workflows as JSON templates with placeholder prompts
- Create reusable workflows for: single sprite, animation frames, tile sets, batch generation
- Version control workflows alongside code

**MCP Integration**:
- Use FastMCP library for Python MCP server
- bevy_brp_mcp for Bevy side
- Test MCP connection before building higher-level features

**Performance Targets** (from research):
- Inference: 3-5 seconds per 1024x1024 sprite
- Batch generation: 20-30 sprites per minute
- LoRA training: 2-4 hours per model (50 images, 3000 steps)
- TUI rendering: 60+ FPS (Proposal 2B)
- ZeroMQ IPC latency: <1ms (Proposal 2B)

**Side-by-Side Model Comparison** (Proposal 2B):
- Generate with multiple models (pre-trained + custom LoRAs) simultaneously
- Display results side-by-side in TUI for visual comparison
- Track user preferences (which model produced better results)
- Use comparison data to inform training improvements
- Essential for validating that custom LoRA training improves quality

## Bevy Integration Patterns

Two integration approaches are documented:

1. **Manual**: Generate → Review → Copy to `assets/` → Reference in code
2. **Automated (MCP)**: Generate → Auto-deploy via MCP → Hot reload in game

For MCP integration:
- Bevy must have `bevy_brp_mcp` plugin enabled
- Assets must follow Bevy's `assets/` directory structure
- Use relative paths: `asset_server.load("sprites/character.png")`
- Enable hot reloading for development: `AssetPlugin { watch_for_changes_override: Some(true) }`

See `docs/04-bevy-integration.md` for complete patterns and code examples.

## Common Pitfalls (From Research)

1. **Don't use SD 1.5** - SDXL is significantly better for pixel art
2. **Don't skip LoRA training** - Pre-trained models lack style consistency
3. **Don't use blur/smooth upscaling** - Use nearest-neighbor for pixel-perfect scaling
4. **Don't ignore color quantization** - Reduce to optimal palette in post-processing
5. **Don't load models with FP32** - Use FP16 or FP4 to leverage Tensor Cores
6. **Don't create absolute asset paths in Bevy** - Use relative to `assets/` directory

## Development Workflow (Once Implemented)

The intended workflow (not yet implemented):

```bash
# Option 1: CLI
dgx-pixels generate character "medieval knight"
dgx-pixels batch prompts.txt --output ./assets/

# Option 2: API
curl -X POST http://localhost:8000/api/v1/generate \
  -d '{"prompt": "knight sprite", "style": "16bit"}'

# Option 3: MCP (automated)
# AI assistant calls generate_and_deploy() tool
# Assets appear directly in Bevy project
```

## Testing Strategy (To Be Implemented)

When building the system:

1. **Model Quality Tests**: Generate from standard prompts, compare to references
2. **Integration Tests**: End-to-end generation → deployment → Bevy loading
3. **Performance Tests**: Verify 3-5s inference, 20-30 sprites/min batch
4. **Training Tests**: Verify LoRA training completes and improves quality

## Critical Files to Understand

Before implementing any component:

- **Architecture decision**: Read `docs/02-architecture-proposals.md` § Comparison Matrix and Proposal 2B
- **SDXL + LoRA**: Read `docs/03-technology-deep-dive.md` § Stable Diffusion XL and LoRA sections
- **ComfyUI**: Read `docs/03-technology-deep-dive.md` § ComfyUI section
- **Bevy Assets**: Read `docs/04-bevy-integration.md` § Asset System Basics
- **Training**: Read `docs/05-training-roadmap.md` § Phase 2 before training first model

**For Proposal 2B (Rust + Python):**
- **Architecture**: Read `docs/07-rust-python-architecture.md` § ZeroMQ Communication Patterns
- **TUI Design**: Read `docs/08-tui-design.md` § Screen Layouts and Side-by-Side Comparison
- **Playbook Integration**: Read `docs/11-playbook-contribution.md` § Installation Steps

## Next Steps (For First Implementation)

1. Review all docs in `docs/` directory
2. Decide on architecture proposal (Proposal 2B: Rust+Python recommended for best developer experience)
3. Follow implementation guide for chosen path:
   - **Proposal 2B**: See `docs/07-rust-python-architecture.md` and `docs/08-tui-design.md`
   - **Proposal 2**: See `docs/06-implementation-plan.md` § Balanced Production
   - **Proposal 1**: See `docs/06-implementation-plan.md` § Rapid Prototyping
4. Set up DGX-Spark environment (ComfyUI via dgx-spark-playbooks)
5. Download base models and test generation
6. Build TUI/CLI/API layer (architecture-dependent)
7. Implement Bevy integration
8. Collect training data and train first LoRA
9. Use side-by-side comparison to validate training improvements (Proposal 2B)

Do not skip steps or mix architecture proposals - the plans are sequential and architecture-specific.

## Repository Context

- **Hardware**: This will run on a specific NVIDIA DGX-Spark - not generic cloud GPUs
- **Target Users**: Game developers using Bevy engine (Rust-based)
- **Use Case**: Rapid pixel art sprite generation for game prototyping and production
- **Unique Value**: Open-source, optimized for specific hardware, direct game engine integration

The research phase identified that no existing solution combines all these requirements, which is why this project exists.
