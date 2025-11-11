# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**DGX-Pixels** is an AI-powered pixel art generation stack optimized for the NVIDIA DGX-Spark hardware, designed to generate game sprites and assets for Bevy game engine projects.

**Current Status**: Documentation Phase Complete ✅ - Implementation not yet started

The project is in its research and planning phase. All documentation is complete, but no code has been written yet. This is intentional - the comprehensive research and architecture proposals must be reviewed and a specific architecture path selected before implementation begins.

## Core Constraints

These are non-negotiable requirements that must be respected in all implementations:

1. **Hardware**: Must run on NVIDIA DGX-Spark (GB10 Grace Blackwell Superchip, **single GPU**, 128GB unified memory, 1000 TOPS, ARM CPU)
   - **VERIFIED**: See `docs/hardware.md` for actual hardware specifications
   - **IMPORTANT**: This is NOT a multi-GPU DGX B200 system (see `docs/adr/0001-dgx-spark-not-b200.md`)
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

### Rust + Python Stack Documentation
- **07-rust-python-architecture.md**: Hybrid Rust TUI + Python backend design with ZeroMQ IPC patterns
- **08-tui-design.md**: Complete TUI mockups, workflows, and side-by-side model comparison feature
- **11-playbook-contribution.md**: Proposal for contributing to dgx-spark-playbooks repository

### Operations & Project Management (NEW)
- **hardware.md**: Verified DGX-Spark GB10 hardware specifications, topology, and performance characteristics
- **metrics.md**: Performance, quality, and observability metrics framework (adapted for single-GPU)
- **adr/0001-dgx-spark-not-b200.md**: Architecture Decision Record explaining hardware differences
- **docs/ROADMAP.md**: Milestone-based development roadmap (M0-M5)
- **docs/rfds/gpt5-dgx-pixels.md**: External review feedback (note: assumes DGX B200, not applicable to our GB10)

**Critical**: Read relevant documentation before implementing any component. The research phase identified best practices, pitfalls, and optimal approaches.

**Hardware Context**: The system runs on DGX-Spark GB10 (single GPU, unified memory), NOT a multi-GPU DGX B200. This changes many architectural decisions. Always consult `docs/hardware.md` and `docs/adr/0001-dgx-spark-not-b200.md` when making hardware-related decisions.

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
- **Unified Memory**: Exploit zero-copy CPU↔GPU transfers (no cudaMemcpy overhead)
- **Single GPU Focus**: No multi-GPU scaling complexity, simpler deployment
- **ARM Compatibility**: Ensure all dependencies support ARM64 architecture

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
7. **Don't assume multi-GPU scaling** - This is a single-GPU system, focus on batch optimization instead
8. **Don't ignore ARM compatibility** - Verify all dependencies support ARM64 architecture
9. **Don't waste unified memory** - Exploit zero-copy transfers, avoid unnecessary cudaMemcpy calls

## Development Workflow

### Agent Workflow (Automated)

When agents implement workstreams, they follow this workflow:

1. **Create Branch**: `just branch WS-XX` or `gh-create-branch "wsXX-name"`
2. **Implement Changes**: Follow TDD (tests first!)
3. **Run Quality Checks**: `just ci` (fmt, lint, test)
4. **Create PR**: `just pr "Title"` or `gh-create-pr "Title"`
5. **Rebase onto Main**: `gh-rebase-main` (before merge)
6. **Auto-merge**: `gh-auto-merge --merge-method squash` (after CI passes)

### Manual Workflow (Human Developers)

See `CONTRIBUTING.md` for detailed manual workflow guidelines.

### Project Commands (justfile)

The project uses `just` for task automation. Key commands:

```bash
# Setup
just init              # Initialize project (first time)
just validate-gpu      # Verify DGX-Spark hardware

# Development
just tui               # Run Rust TUI (debug)
just backend           # Start Python backend worker
just comfyui           # Start ComfyUI server

# Testing
just test              # Run all tests
just test-coverage     # Run tests with coverage
just ci                # Run all CI checks (fmt, lint, test)

# Code Quality
just fmt               # Format Rust code
just lint              # Run Rust clippy
just fmt-python        # Format Python code

# Models
just models-list       # List available models
just download-model    # Download SDXL base model
just train-lora DATASET  # Train LoRA on dataset

# Monitoring
just gpu-status        # Show GPU stats
just gpu-watch         # Monitor GPU (live)
just hw-info           # Show all hardware info

# Git
just status            # Show git status
just branch WS-XX      # Create branch for workstream
just pr "Title"        # Create pull request
just rebase            # Rebase onto main

# Documentation
just docs              # Generate and open Rust docs
just docs-serve        # Serve docs locally

# Full list
just --list            # Show all available commands
```

### Nushell Scripts

The project uses nushell for automation scripts:

**Location**: `scripts/nu/`

**Modules**:
- `config.nu` - Project config, logging, utilities
- `modules/comfyui.nu` - ComfyUI API wrapper
- `modules/dgx.nu` - DGX-Spark hardware utilities
- `modules/github.nu` - GitHub automation (PR, branch, merge)

**Usage**:
```nushell
# Load config
use scripts/nu/config.nu *

# Check hardware
use scripts/nu/modules/dgx.nu *
dgx-validate-hardware
dgx-gpu-stats

# GitHub automation
use scripts/nu/modules/github.nu *
gh-create-branch "feature/new-tui"
gh-create-pr "Add new TUI feature" --draft
gh-auto-merge --merge-method squash

# ComfyUI integration
use scripts/nu/modules/comfyui.nu *
comfyui-health-check
comfyui-generate (open workflows/sprite-gen.json)
```

## Testing Strategy (To Be Implemented)

When building the system:

1. **Model Quality Tests**: Generate from standard prompts, compare to references
2. **Integration Tests**: End-to-end generation → deployment → Bevy loading
3. **Performance Tests**: Verify 3-5s inference, 20-30 sprites/min batch
4. **Training Tests**: Verify LoRA training completes and improves quality

## Critical Files to Understand

Before implementing any component:

- **HARDWARE FIRST**: Read `docs/hardware.md` and `docs/adr/0001-dgx-spark-not-b200.md` to understand single-GPU unified memory architecture
- **Roadmap**: Read `docs/ROADMAP.md` for milestone-based development plan (M0-M5)
- **Metrics**: Read `docs/metrics.md` for performance targets and benchmarking strategy
- **Architecture decision**: Read `docs/02-architecture-proposals.md` § Comparison Matrix and Proposal 2B
- **SDXL + LoRA**: Read `docs/03-technology-deep-dive.md` § Stable Diffusion XL and LoRA sections
- **ComfyUI**: Read `docs/03-technology-deep-dive.md` § ComfyUI section
- **Bevy Assets**: Read `docs/04-bevy-integration.md` § Asset System Basics
- **Training**: Read `docs/05-training-roadmap.md` § Phase 2 before training first model

**For Proposal 2B (Rust + Python) - RECOMMENDED:**
- **Architecture**: Read `docs/07-rust-python-architecture.md` § ZeroMQ Communication Patterns
- **TUI Design**: Read `docs/08-tui-design.md` § Screen Layouts and Side-by-Side Comparison
- **Playbook Integration**: Read `docs/11-playbook-contribution.md` § Installation Steps

## Next Steps (For First Implementation)

### 1. Review Documentation

Start with the orchestration summary:
```bash
cat docs/orchestration/project-summary.md
```

Key documents:
- **Architecture**: `docs/02-architecture-proposals.md` (choose Proposal 2B recommended)
- **Hardware**: `docs/hardware.md` (understand GB10 unified memory)
- **Roadmap**: `docs/ROADMAP.md` (M0-M5 milestones)
- **Orchestration**: `docs/orchestration/meta-orchestrator.md` (coordination strategy)
- **Workstreams**: `docs/orchestration/workstream-plan.md` (all 18 workstreams)

### 2. Initialize Project

```bash
# Clone and setup
git clone https://github.com/raibid-labs/dgx-pixels.git
cd dgx-pixels

# Initialize project
just init

# Validate hardware
just validate-gpu

# View hardware info
just hw-info
```

### 3. Start with Foundation Orchestrator (M0)

The project uses **orchestrated workstreams**. Start with Foundation:

```bash
# Review Foundation Orchestrator
cat docs/orchestration/orchestrators/foundation.md

# Review first workstream (WS-01)
cat docs/orchestration/workstreams/ws01-hardware-baselines/README.md

# Create branch for WS-01
just branch WS-01

# Implement following the workstream spec
# (See CONTRIBUTING.md for detailed workflow)
```

### 4. Follow Orchestration Plan

After M0 completes, proceed through:
- **M1**: Model Orchestrator (ComfyUI, SDXL optimization)
- **M2**: Interface Orchestrator (Rust TUI, ZeroMQ, backend)
- **M3**: Model Orchestrator (LoRA training)
- **M4**: Integration Orchestrator (Bevy, MCP)
- **M5**: Integration Orchestrator (observability, deployment)

See `docs/orchestration/meta-orchestrator.md` for coordination details.

Do not skip steps or mix architecture proposals - the plans are sequential and architecture-specific.

## Repository Context

- **Hardware**: This will run on a specific NVIDIA DGX-Spark - not generic cloud GPUs
- **Target Users**: Game developers using Bevy engine (Rust-based)
- **Use Case**: Rapid pixel art sprite generation for game prototyping and production
- **Unique Value**: Open-source, optimized for specific hardware, direct game engine integration

The research phase identified that no existing solution combines all these requirements, which is why this project exists.
