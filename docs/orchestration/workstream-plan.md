# DGX-Pixels Workstream Plan

**Version**: 1.0
**Created**: 2025-11-10
**Timeline**: 12 weeks (M0-M5)
**Total Workstreams**: 18 across 4 domain orchestrators

---

## Executive Summary

This plan breaks down the DGX-Pixels project into 18 parallel workstreams organized under 4 domain orchestrators. The structure enables maximum parallel execution while respecting dependencies.

**Phase Structure**:
- **Phase 1** (Weeks 1-2): Foundation - M0 (3 workstreams, sequential)
- **Phase 2** (Weeks 3-6): Models + Interface - M1, M2, M3 (9 workstreams, parallel)
- **Phase 3** (Weeks 7-12): Integration + Production - M4, M5 (6 workstreams, mixed)

**Expected Timeline**: 12 weeks with proper orchestration, 16-20 weeks sequential

---

## Workstream Overview

| ID | Name | Orchestrator | Milestone | Duration | Dependencies |
|----|------|--------------|-----------|----------|--------------|
| **WS-01** | Hardware Baselines | Foundation | M0 | 3-4 days | None |
| **WS-02** | Reproducibility Framework | Foundation | M0 | 4-5 days | WS-01 |
| **WS-03** | Benchmark Suite | Foundation | M0 | 3-4 days | WS-01 |
| **WS-04** | ComfyUI Setup | Model | M1 | 4-5 days | WS-01 |
| **WS-05** | SDXL Inference Optimization | Model | M1 | 5-7 days | WS-04 |
| **WS-06** | LoRA Training Pipeline | Model | M3 | 7-10 days | WS-05 |
| **WS-07** | Dataset Tools & Validation | Model | M3 | 5-6 days | WS-05 |
| **WS-08** | Rust TUI Core | Interface | M2 | 6-8 days | WS-01 |
| **WS-09** | ZeroMQ IPC Layer | Interface | M2 | 4-5 days | WS-08 |
| **WS-10** | Python Backend Worker | Interface | M2 | 5-6 days | WS-04, WS-09 |
| **WS-11** | Sixel Image Preview | Interface | M2 | 3-4 days | WS-08, WS-10 |
| **WS-12** | Side-by-Side Model Comparison | Interface | M2 | 4-5 days | WS-10, WS-11 |
| **WS-13** | FastMCP Server | Integration | M4 | 5-6 days | WS-10 |
| **WS-14** | Bevy Plugin Integration | Integration | M4 | 6-7 days | WS-13 |
| **WS-15** | Asset Deployment Pipeline | Integration | M4 | 4-5 days | WS-13, WS-14 |
| **WS-16** | DCGM Metrics & Observability | Integration | M5 | 5-6 days | WS-05 |
| **WS-17** | Docker Compose Deployment | Integration | M5 | 4-5 days | WS-10, WS-16 |
| **WS-18** | CI/CD Pipeline | Integration | M5 | 6-8 days | WS-17 |

**Total Effort**: ~90-110 days (sequential) → ~60-70 days (with parallel execution)

---

## Phase 1: Foundation (Weeks 1-2)

**Orchestrator**: Foundation Orchestrator
**Goal**: Establish hardware baselines, reproducibility, and benchmarking infrastructure
**Timeline**: 2 weeks (sequential execution)
**Dependencies**: None (blocking all other phases)

### WS-01: Hardware Baselines

**Owner**: Foundation Orchestrator
**Agent Type**: `devops-automator`
**Duration**: 3-4 days
**Priority**: P0 (critical path)

**Objective**: Document verified DGX-Spark GB10 hardware specifications and establish baseline performance metrics.

**Deliverables**:
1. `repro/hardware_verification.sh` - Automated hardware detection script
2. `bench/baselines/hardware_baseline.json` - Recorded baseline metrics
3. Updated `docs/hardware.md` with actual measurements
4. Topology diagrams and memory architecture documentation

**Acceptance Criteria**:
- ✅ Script captures: GPU model, VRAM, CUDA version, driver, CPU, RAM, topology
- ✅ Baseline JSON includes: nvidia-smi output, lscpu, free -h, storage info
- ✅ Documentation matches actual hardware (GB10, 128GB unified, ARM CPU)
- ✅ Verification script exits 0 on success

**Technical Requirements**:
- Bash script compatible with Ubuntu 22.04
- JSON output format for CI integration
- No manual intervention required (fully automated)

**Related Issues**: PIXELS-001, PIXELS-002

**Estimated LOC**: 200-300 (bash + documentation)

---

### WS-02: Reproducibility Framework

**Owner**: Foundation Orchestrator
**Agent Type**: `devops-automator`
**Duration**: 4-5 days
**Priority**: P0
**Dependencies**: WS-01 (needs hardware baseline)

**Objective**: Create reproducible environment for all DGX-Pixels development and research.

**Deliverables**:
1. `repro/Dockerfile` - Pinned NGC PyTorch base image with all dependencies
2. `repro/run.sh` - Environment capture + smoke test script
3. `repro/requirements.txt` - Python dependencies (pinned versions)
4. `repro/install.sh` - System dependency installation
5. Environment capture in `bench/baselines/env_*.json`

**Acceptance Criteria**:
- ✅ Dockerfile builds successfully on DGX-Spark ARM architecture
- ✅ `repro/run.sh` generates 10 test images end-to-end
- ✅ Environment JSON captures: git SHA, CUDA, cuDNN, NCCL, Python packages
- ✅ Smoke test completes in <5 minutes
- ✅ All dependencies support ARM64

**Technical Requirements**:
- Base image: `nvcr.io/nvidia/pytorch:25.01-py3` (ARM-compatible)
- Python 3.10+
- CUDA 13.0
- Document any x86-only packages and ARM alternatives

**Related Issues**: PIXELS-003, PIXELS-004, PIXELS-005

**Estimated LOC**: 400-500 (Dockerfile + scripts + docs)

---

### WS-03: Benchmark Suite

**Owner**: Foundation Orchestrator
**Agent Type**: `devops-automator` + `performance-benchmarker`
**Duration**: 3-4 days
**Priority**: P1
**Dependencies**: WS-01 (needs baseline metrics)

**Objective**: Create comprehensive benchmark suite for single-GPU performance measurement.

**Deliverables**:
1. `bench/throughput.py` - Images/second measurement for single GPU
2. `bench/dmon.sh` - DCGM + nvidia-smi telemetry under load
3. `bench/io_test.sh` - Storage I/O throughput measurement
4. `bench/memory_profile.py` - Unified memory usage profiling
5. Baseline results in `bench/baselines/`

**Acceptance Criteria**:
- ✅ Throughput script measures: img/s, p95 latency, VRAM peak
- ✅ DCGM monitors: GPU utilization, power, temperature, memory bandwidth
- ✅ I/O test verifies ≥ 8 GB/s sustained throughput
- ✅ Memory profiler tracks unified memory usage (CPU+GPU)
- ✅ All metrics exported to JSON for trending

**Technical Requirements**:
- Python 3.10+ with PyTorch
- DCGM installed and accessible
- Zero-copy memory measurement for unified architecture
- Batch sizes: 1, 4, 8 for throughput testing

**Related Issues**: PIXELS-006, PIXELS-007, PIXELS-008

**Estimated LOC**: 600-800 (Python + bash + docs)

---

## Phase 2A: Model Inference & Training (Weeks 3-6)

**Orchestrator**: Model Orchestrator
**Goal**: Establish SDXL inference and LoRA training pipelines optimized for GB10
**Timeline**: 4 weeks (WS-04/05 sequential, then WS-06/07 parallel)
**Dependencies**: WS-01 (hardware baseline)

### WS-04: ComfyUI Setup

**Owner**: Model Orchestrator
**Agent Type**: `ai-engineer`
**Duration**: 4-5 days
**Priority**: P0 (blocks WS-05, WS-10)
**Dependencies**: WS-01

**Objective**: Install and configure ComfyUI on DGX-Spark with ARM compatibility verified.

**Deliverables**:
1. ComfyUI installation following dgx-spark-playbooks pattern
2. Custom installation script: `setup/install_comfyui.sh`
3. Configuration: `config/comfyui_config.yaml`
4. ARM compatibility verification report
5. Basic workflow templates in `workflows/`

**Acceptance Criteria**:
- ✅ ComfyUI server starts and responds to API calls
- ✅ Can load SDXL 1.0 base model (FP16)
- ✅ Generates test image (512×512) in <10 seconds
- ✅ All custom nodes support ARM64 architecture
- ✅ Memory usage ≤ 80GB for single SDXL model

**Technical Requirements**:
- ComfyUI latest stable version
- Python 3.10+ with torch compiled for ARM + CUDA 13.0
- xformers for memory-efficient attention (ARM build)
- Model directory: `models/checkpoints/`

**Related Issues**: PIXELS-009, PIXELS-010, PIXELS-011

**Estimated LOC**: 300-400 (scripts + config + docs)

---

### WS-05: SDXL Inference Optimization

**Owner**: Model Orchestrator
**Agent Type**: `ai-engineer`
**Duration**: 5-7 days
**Priority**: P0 (blocks WS-06, WS-10, WS-16)
**Dependencies**: WS-04

**Objective**: Optimize SDXL 1.0 inference for DGX-Spark GB10 unified memory architecture.

**Deliverables**:
1. Optimized ComfyUI workflows for pixel art generation
2. Performance tuning documentation
3. Workflow templates: `single_sprite.json`, `batch_generation.json`, `tileset.json`
4. Optimization report with before/after metrics
5. Best practices guide for unified memory

**Acceptance Criteria**:
- ✅ ≤ 3 seconds per 1024×1024 image @ FP16, batch=1
- ✅ ≥ 15 images/min in batch mode (batch=8)
- ✅ VRAM usage ≤ 100 GB (unified memory)
- ✅ Zero-copy image loading verified (no cudaMemcpy)
- ✅ Batch efficiency ≥ 2.5× (batch=8 vs batch=1)

**Technical Requirements**:
- FP16 precision throughout pipeline
- xformers memory-efficient attention enabled
- Gradient checkpointing for memory savings
- Unified memory profiling and optimization
- Test with pixel art-specific prompts

**Related Issues**: PIXELS-012, PIXELS-013, PIXELS-014, PIXELS-015

**Estimated LOC**: 500-600 (workflows + optimization + docs)

---

### WS-06: LoRA Training Pipeline

**Owner**: Model Orchestrator
**Agent Type**: `ai-engineer`
**Duration**: 7-10 days
**Priority**: P1 (blocks WS-12)
**Dependencies**: WS-05

**Objective**: Implement LoRA fine-tuning pipeline for custom pixel art style training.

**Deliverables**:
1. Training script: `python/training/lora_trainer.py`
2. Configuration templates: `config/lora_training_*.yaml`
3. Training dataset loader with augmentation
4. Model validation and comparison tools
5. Example trained LoRA checkpoint

**Acceptance Criteria**:
- ✅ Train 50-image dataset in ≤ 4 hours @ 3000 steps, FP16
- ✅ Loss converges (documented in training logs)
- ✅ Generated images maintain style consistency
- ✅ Checkpoint files ≤ 100MB (LoRA format)
- ✅ Training resumable from checkpoint

**Technical Requirements**:
- Kohya_ss or Diffusers training framework (ARM-compatible)
- FP16 mixed precision training
- Gradient checkpointing enabled
- Unified memory batch size optimization
- Automatic validation every 500 steps

**Related Issues**: PIXELS-016, PIXELS-017, PIXELS-018, PIXELS-019

**Estimated LOC**: 800-1000 (training + validation + docs)

---

### WS-07: Dataset Tools & Validation

**Owner**: Model Orchestrator
**Agent Type**: `ai-engineer`
**Duration**: 5-6 days
**Priority**: P1
**Dependencies**: WS-05 (can run parallel with WS-06)

**Objective**: Build dataset preparation and quality validation tools.

**Deliverables**:
1. Auto-captioning script: `python/data/auto_caption.py`
2. Dataset augmentation: `python/data/augment.py`
3. Quality validation: `python/eval/quality_metrics.py` (LPIPS, SSIM, CLIP)
4. Human evaluation rubric: `docs/eval/human_rubric.md`
5. Dataset collection guide

**Acceptance Criteria**:
- ✅ Auto-captioning generates captions for 100 images in <5 minutes
- ✅ Augmentation preserves pixel-perfect clarity
- ✅ Quality metrics match literature baselines
- ✅ Human rubric tested with 3-5 evaluators
- ✅ Example dataset (50 images) prepared

**Technical Requirements**:
- BLIP or similar for auto-captioning
- PIL/Pillow for augmentation (nearest-neighbor only)
- LPIPS, SSIM, PSNR implementations
- CLIP for semantic similarity
- Dataset format: images + captions (JSONL or text files)

**Related Issues**: PIXELS-020, PIXELS-021, PIXELS-022

**Estimated LOC**: 600-700 (Python + docs)

---

## Phase 2B: Interface Development (Weeks 3-6)

**Orchestrator**: Interface Orchestrator
**Goal**: Build Rust TUI with Python backend and ZeroMQ IPC
**Timeline**: 4 weeks (WS-08 first, then WS-09/10 parallel, then WS-11/12)
**Dependencies**: WS-01, WS-04 (ComfyUI must be working)

### WS-08: Rust TUI Core

**Owner**: Interface Orchestrator
**Agent Type**: `rust-pro`
**Duration**: 6-8 days
**Priority**: P0 (blocks WS-09, WS-11)
**Dependencies**: WS-01

**Objective**: Build core Rust TUI application with ratatui framework.

**Deliverables**:
1. Rust project: `rust/` with Cargo.toml
2. TUI framework: `rust/src/ui/` with ratatui components
3. Screen layouts: `rust/src/ui/screens/` (generation, gallery, settings)
4. Event handling: `rust/src/events.rs`
5. Binary: `dgx-pixels-tui`

**Acceptance Criteria**:
- ✅ 60+ FPS rendering on DGX-Spark
- ✅ Responsive keyboard/mouse navigation
- ✅ Layouts: generation screen, gallery, settings
- ✅ Binary size ≤ 15MB (release build)
- ✅ Memory usage ≤ 50MB (TUI only)

**Technical Requirements**:
- Rust 1.70+ with ARM64 target
- ratatui for TUI rendering
- crossterm for terminal control
- tokio for async runtime
- TDD: unit tests for all UI components

**Related Issues**: PIXELS-023, PIXELS-024, PIXELS-025, PIXELS-026

**Estimated LOC**: 1200-1500 (Rust)

---

### WS-09: ZeroMQ IPC Layer

**Owner**: Interface Orchestrator
**Agent Type**: `rust-pro` + `backend-architect`
**Duration**: 4-5 days
**Priority**: P0 (blocks WS-10, WS-12)
**Dependencies**: WS-08

**Objective**: Implement ZeroMQ IPC for Rust TUI ↔ Python backend communication.

**Deliverables**:
1. ZeroMQ client: `rust/src/zmq_client.rs`
2. Protocol definitions: `rust/src/protocol.rs`
3. Message serialization (MsgPack): `rust/src/messages.rs`
4. Connection management and reconnection logic
5. IPC benchmarks and latency tests

**Acceptance Criteria**:
- ✅ <1ms IPC latency (REQ-REP pattern)
- ✅ <100μs PUB-SUB latency for status updates
- ✅ Automatic reconnection on connection loss
- ✅ MsgPack serialization for all messages
- ✅ ARM-compatible ZeroMQ build verified

**Technical Requirements**:
- zmq crate (Rust ZeroMQ bindings)
- rmp-serde for MsgPack serialization
- REQ-REP pattern for job submission
- PUB-SUB pattern for progress updates
- Error handling for connection failures

**Related Issues**: PIXELS-027, PIXELS-028, PIXELS-029

**Estimated LOC**: 600-800 (Rust)

---

### WS-10: Python Backend Worker

**Owner**: Interface Orchestrator
**Agent Type**: `python-pro` + `ai-engineer`
**Duration**: 5-6 days
**Priority**: P0 (blocks WS-11, WS-12, WS-13)
**Dependencies**: WS-04 (ComfyUI), WS-09 (ZeroMQ)

**Objective**: Build Python backend worker that bridges ZeroMQ IPC to ComfyUI API.

**Deliverables**:
1. ZeroMQ server: `python/workers/zmq_server.py`
2. ComfyUI client: `python/workers/comfyui_client.py`
3. Job queue manager: `python/workers/job_queue.py`
4. Generation worker: `python/workers/generation_worker.py`
5. Progress tracking and status updates

**Acceptance Criteria**:
- ✅ Receives jobs via ZeroMQ REQ-REP
- ✅ Submits workflows to ComfyUI API
- ✅ Publishes progress updates via PUB-SUB
- ✅ Handles multiple concurrent requests (queue)
- ✅ Graceful error handling and recovery

**Technical Requirements**:
- Python 3.10+ with asyncio
- pyzmq for ZeroMQ server
- aiohttp for ComfyUI API calls
- msgpack-python for serialization
- Job queue with priority support

**Related Issues**: PIXELS-030, PIXELS-031, PIXELS-032, PIXELS-033

**Estimated LOC**: 800-1000 (Python)

---

### WS-11: Sixel Image Preview

**Owner**: Interface Orchestrator
**Agent Type**: `rust-pro`
**Duration**: 3-4 days
**Priority**: P1
**Dependencies**: WS-08 (TUI), WS-10 (Backend)

**Objective**: Implement Sixel protocol for in-terminal image preview.

**Deliverables**:
1. Sixel renderer: `rust/src/image_preview.rs`
2. Image scaling and optimization
3. Terminal compatibility detection
4. Fallback for non-Sixel terminals (ASCII art)
5. Preview performance benchmarks

**Acceptance Criteria**:
- ✅ Displays 1024×1024 images in terminal
- ✅ <100ms render time for Sixel output
- ✅ Automatic terminal capability detection
- ✅ Zero-copy image access (unified memory advantage)
- ✅ Works in: iTerm2, WezTerm, Alacritty (with Sixel support)

**Technical Requirements**:
- image crate for image manipulation
- Sixel encoding library or custom implementation
- Terminal capability detection (query TERM)
- Efficient image downscaling (nearest-neighbor for pixel art)

**Related Issues**: PIXELS-034, PIXELS-035, PIXELS-036

**Estimated LOC**: 400-600 (Rust)

---

### WS-12: Side-by-Side Model Comparison

**Owner**: Interface Orchestrator
**Agent Type**: `rust-pro` + `python-pro`
**Duration**: 4-5 days
**Priority**: P1
**Dependencies**: WS-10, WS-11

**Objective**: Enable simultaneous generation with multiple models for quality comparison.

**Deliverables**:
1. Multi-model generation UI: `rust/src/ui/comparison.rs`
2. Parallel job submission: `python/workers/parallel_generation.py`
3. Side-by-side preview layout
4. User preference tracking
5. Comparison mode documentation

**Acceptance Criteria**:
- ✅ Generate with 2-4 models simultaneously
- ✅ Display results side-by-side with labels
- ✅ User can vote on best result
- ✅ Preferences saved to JSON for analysis
- ✅ Total generation time ≤ 1.5× single model time

**Technical Requirements**:
- Load multiple models in 128GB unified memory
- Parallel ComfyUI workflow execution
- TUI layout with multiple preview panes
- Preference tracking: model_id, prompt, user_vote
- Compare: pre-trained vs custom LoRA

**Related Issues**: PIXELS-037, PIXELS-038, PIXELS-039

**Estimated LOC**: 700-900 (Rust + Python)

---

## Phase 3: Integration & Production (Weeks 7-12)

**Orchestrator**: Integration Orchestrator
**Goal**: Integrate with Bevy, deploy production infrastructure
**Timeline**: 6 weeks (WS-13/14/15 sequential, WS-16/17/18 parallel)
**Dependencies**: WS-10 (Backend working)

### WS-13: FastMCP Server

**Owner**: Integration Orchestrator
**Agent Type**: `backend-architect`
**Duration**: 5-6 days
**Priority**: P0 (blocks WS-14, WS-15)
**Dependencies**: WS-10

**Objective**: Build MCP server for Bevy game engine integration.

**Deliverables**:
1. FastMCP server: `python/mcp_server/server.py`
2. MCP tools: `generate_sprite`, `generate_batch`, `deploy_to_bevy`
3. Server configuration: `config/mcp_config.yaml`
4. API documentation
5. MCP client testing tools

**Acceptance Criteria**:
- ✅ MCP server responds to tool calls
- ✅ Integrates with Python backend worker (WS-10)
- ✅ Supports stdio and SSE transports
- ✅ Tool schemas validated against MCP spec
- ✅ <200ms response time for tool invocation

**Technical Requirements**:
- fastmcp library (Python MCP framework)
- Async integration with backend worker
- Tool parameters: prompt, style, resolution, output_path
- Error handling with MCP error format
- Logging and debugging support

**Related Issues**: PIXELS-040, PIXELS-041, PIXELS-042

**Estimated LOC**: 500-700 (Python)

---

### WS-14: Bevy Plugin Integration

**Owner**: Integration Orchestrator
**Agent Type**: `rust-pro`
**Duration**: 6-7 days
**Priority**: P0 (blocks WS-15)
**Dependencies**: WS-13

**Objective**: Create Bevy plugin example with MCP integration.

**Deliverables**:
1. Example Bevy project: `examples/bevy_integration/`
2. MCP client integration using `bevy_brp_mcp`
3. Asset hot-reload support
4. Example game with AI-generated sprites
5. Integration guide

**Acceptance Criteria**:
- ✅ Bevy app connects to MCP server
- ✅ Can invoke generate_sprite from Bevy
- ✅ Generated assets load automatically
- ✅ Hot-reload updates sprites in running game
- ✅ Example game showcases workflow

**Technical Requirements**:
- Bevy 0.13+
- bevy_brp_mcp plugin
- Asset pipeline integration
- Example game: simple platformer or top-down
- Rust tests for integration

**Related Issues**: PIXELS-043, PIXELS-044, PIXELS-045

**Estimated LOC**: 800-1000 (Rust + Bevy)

---

### WS-15: Asset Deployment Pipeline

**Owner**: Integration Orchestrator
**Agent Type**: `devops-automator`
**Duration**: 4-5 days
**Priority**: P1
**Dependencies**: WS-13, WS-14

**Objective**: Automate asset generation and deployment to Bevy projects.

**Deliverables**:
1. Deployment script: `scripts/deploy_assets.sh`
2. Asset naming conventions and directory structure
3. Bevy asset manifest generation
4. Validation pipeline (checks resolution, format, naming)
5. Deployment documentation

**Acceptance Criteria**:
- ✅ Generates assets in Bevy-compatible format (PNG, 32-bit)
- ✅ Places assets in correct `assets/` subdirectory
- ✅ Updates Bevy asset manifest automatically
- ✅ Validates all assets before deployment
- ✅ Deployment completes in <1 second

**Technical Requirements**:
- Asset output: PNG with transparency
- Bevy directory structure: `assets/sprites/`, `assets/tiles/`, etc.
- Manifest format: JSON or RON
- Validation: resolution, file size, format
- Git integration (optional): auto-commit generated assets

**Related Issues**: PIXELS-046, PIXELS-047

**Estimated LOC**: 400-500 (Bash + Python)

---

### WS-16: DCGM Metrics & Observability

**Owner**: Integration Orchestrator
**Agent Type**: `devops-automator` + `infrastructure-maintainer`
**Duration**: 5-6 days
**Priority**: P1 (can run parallel with WS-13/14/15)
**Dependencies**: WS-05 (inference working)

**Objective**: Implement comprehensive GPU metrics and observability.

**Deliverables**:
1. DCGM exporter configuration
2. Prometheus metrics collection
3. Grafana dashboards: performance, quality, system health
4. Alerting rules (VRAM, temperature, errors)
5. Observability documentation

**Acceptance Criteria**:
- ✅ DCGM exports: GPU utilization, VRAM, power, temperature
- ✅ Prometheus scrapes metrics every 15 seconds
- ✅ Grafana dashboards visualize real-time performance
- ✅ Alerts trigger on: VRAM >95%, temp >85°C, errors
- ✅ Metrics retained for 30 days

**Technical Requirements**:
- DCGM installed and running
- Prometheus + Grafana containers
- Custom metrics: img/s, latency, quality scores
- Dashboard templates in `deploy/grafana/`
- Alert rules in `deploy/prometheus/`

**Related Issues**: PIXELS-048, PIXELS-049, PIXELS-050

**Estimated LOC**: 600-800 (configs + dashboards + docs)

---

### WS-17: Docker Compose Deployment

**Owner**: Integration Orchestrator
**Agent Type**: `devops-automator`
**Duration**: 4-5 days
**Priority**: P1 (can run parallel after WS-10, WS-16)
**Dependencies**: WS-10 (Backend), WS-16 (Metrics)

**Objective**: Package entire stack in Docker Compose for easy deployment.

**Deliverables**:
1. `docker-compose.yml` - Complete stack definition
2. Dockerfiles for all services
3. Environment configuration: `.env.example`
4. Setup script: `scripts/setup_docker.sh`
5. Deployment documentation

**Acceptance Criteria**:
- ✅ `docker-compose up` starts entire stack
- ✅ Services: ComfyUI, Python backend, Prometheus, Grafana
- ✅ Persists data: models, outputs, metrics
- ✅ GPU passthrough working (NVIDIA runtime)
- ✅ Stack starts in <60 seconds

**Technical Requirements**:
- Docker Compose v2+
- NVIDIA Container Toolkit
- Volume mounts for: models/, outputs/, configs/
- Health checks for all services
- Non-root containers (security)

**Related Issues**: PIXELS-051, PIXELS-052, PIXELS-053

**Estimated LOC**: 500-600 (Dockerfiles + compose + docs)

---

### WS-18: CI/CD Pipeline

**Owner**: Integration Orchestrator
**Agent Type**: `devops-automator`
**Duration**: 6-8 days
**Priority**: P2 (nice-to-have for MVP)
**Dependencies**: WS-17

**Objective**: Automate testing, building, and deployment.

**Deliverables**:
1. GitHub Actions workflows: `.github/workflows/`
2. Test automation: unit, integration, performance
3. Docker image building and publishing
4. Automated documentation generation
5. Release automation

**Acceptance Criteria**:
- ✅ Tests run on every PR
- ✅ Docker images built and tagged on merge
- ✅ Performance regression tests run weekly
- ✅ Documentation auto-deployed to GitHub Pages
- ✅ Release process automated (tag → build → publish)

**Technical Requirements**:
- GitHub Actions (ARM runners if available)
- Test matrix: Rust tests, Python tests, integration tests
- Docker buildx for multi-arch (x86 + ARM)
- Performance benchmarks vs baselines
- Semantic versioning

**Related Issues**: PIXELS-054, PIXELS-055, PIXELS-056

**Estimated LOC**: 700-900 (YAML + scripts + docs)

---

## Dependency Graph

```
Phase 1 (Sequential)
  WS-01 (Hardware Baselines)
    ├─> WS-02 (Reproducibility)
    └─> WS-03 (Benchmarks)

Phase 2 (Parallel Execution)
  WS-01 ─┬─> WS-04 (ComfyUI) ─> WS-05 (SDXL Opt) ─┬─> WS-06 (LoRA Training)
         │                                         └─> WS-07 (Dataset Tools)
         │
         └─> WS-08 (Rust TUI) ─> WS-09 (ZeroMQ)
                                      │
                                      └─> WS-10 (Backend) ─┬─> WS-11 (Sixel)
                                          │                 └─> WS-12 (Comparison)
                                          └─> WS-13 (MCP Server)

Phase 3 (Mixed Execution)
  WS-10 ─> WS-13 (MCP) ─> WS-14 (Bevy) ─> WS-15 (Asset Pipeline)

  WS-05 ─> WS-16 (Metrics) ─┐
  WS-10 ─> WS-17 (Docker)   ├─> WS-18 (CI/CD)
           WS-16 ───────────┘
```

---

## Resource Allocation

### Agent Types Required

| Agent Type | Workstreams | Total Days | Notes |
|------------|-------------|------------|-------|
| `devops-automator` | WS-01, WS-02, WS-03, WS-15, WS-16, WS-17, WS-18 | 35-40 | Infrastructure focus |
| `ai-engineer` | WS-04, WS-05, WS-06, WS-07 | 21-28 | ML/AI expertise |
| `rust-pro` | WS-08, WS-09, WS-11, WS-12, WS-14 | 23-30 | Rust TUI + Bevy |
| `python-pro` | WS-10, WS-12 (partial) | 8-10 | Backend worker |
| `backend-architect` | WS-09 (partial), WS-13 | 8-10 | MCP server |

### Parallel Execution Capacity

**Maximum Parallel Workstreams**:
- **Phase 1**: 1-2 (sequential preferred for foundation)
- **Phase 2**: 4-6 (Models + Interface domains independent)
- **Phase 3**: 3-4 (Integration + Metrics parallel)

**Orchestrator Concurrency**:
- Foundation Orchestrator: 1-2 agents (sequential work)
- Model Orchestrator: 2-3 agents (WS-04/05 sequential, then WS-06/07 parallel)
- Interface Orchestrator: 3-4 agents (WS-08 first, then WS-09/10, then WS-11/12)
- Integration Orchestrator: 2-3 agents (WS-13/14/15 sequential, WS-16/17/18 parallel)

---

## Success Metrics

### Overall Project Success

**Timeline**:
- ✅ Complete all 18 workstreams in ≤ 12 weeks (+1 week buffer acceptable)
- ✅ All phase gates passed with acceptance criteria met

**Quality**:
- ✅ All workstreams have ≥ 80% test coverage
- ✅ Performance targets met (docs/metrics.md)
- ✅ Documentation complete for all deliverables

**Coordination**:
- ✅ No workstream blocked for >48 hours
- ✅ Cross-domain conflicts resolved within 24 hours
- ✅ All orchestrators report status regularly

### Per-Workstream Success

Each workstream must meet:
- ✅ All acceptance criteria verified
- ✅ Tests passing (unit + integration)
- ✅ Documentation complete
- ✅ Code reviewed and merged
- ✅ Completion summary created

---

## Risk Management

### High-Risk Workstreams

| Workstream | Risk | Mitigation |
|------------|------|------------|
| **WS-04** | ARM compatibility issues with ComfyUI dependencies | Research ARM packages early, have fallbacks |
| **WS-05** | Performance targets not met on GB10 hardware | Profile early, iterate on optimizations |
| **WS-06** | LoRA training too slow or poor quality | Start with small dataset, tune hyperparameters |
| **WS-09** | ZeroMQ not available for ARM | Alternative IPC (gRPC, Unix sockets) |
| **WS-14** | Bevy MCP integration complex | Start simple, use bevy_brp_mcp examples |

### Dependency Risks

**Critical Path**:
WS-01 → WS-04 → WS-05 → WS-10 → WS-13 → WS-14

Any delay in critical path workstreams adds to timeline.

**Mitigation**:
- Prioritize critical path (P0) workstreams
- Start non-blocking workstreams early (WS-03, WS-08)
- Have contingency plans for blocked workstreams

---

## Next Steps

1. **Review this plan** with user
2. **Create individual workstream specs** in `docs/orchestration/workstreams/ws##-name/README.md`
3. **Create domain orchestrator specs** in `docs/orchestration/orchestrators/`
4. **Generate GitHub issues** from workstream specs
5. **Initialize Meta Orchestrator** and spawn Foundation Orchestrator

**Ready to proceed?** See `docs/orchestration/meta-orchestrator.md` for orchestration details.
