# DGX-Pixels - AI Pixel Art Generation for Bevy Games
# Task automation for development and operations
#
# Usage: just [RECIPE]
# Example: just init
#
# See available recipes: just --list

# Default recipe - show available commands
default:
    @just --list

# === Project Initialization ===

# Initialize project (first-time setup)
init:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🎨 Initializing DGX-Pixels..."

    # Create directory structure
    mkdir -p rust/src/{ui,zmq_client}
    mkdir -p python/workers
    mkdir -p workflows
    mkdir -p models/{checkpoints,loras,configs}
    mkdir -p examples/bevy
    mkdir -p bench/baselines
    mkdir -p repro
    mkdir -p scripts/{nu/modules,bash}

    # Create Python virtual environment
    if [ ! -d "venv" ]; then
        python3 -m venv venv
        echo "✓ Created Python virtual environment"
    fi

    # Install Python dependencies (when requirements.txt exists)
    if [ -f "requirements.txt" ]; then
        source venv/bin/activate
        pip install -r requirements.txt
        echo "✓ Installed Python dependencies"
    fi

    # Initialize Rust workspace (when Cargo.toml exists)
    if [ -f "Cargo.toml" ]; then
        cargo fetch
        echo "✓ Fetched Rust dependencies"
    fi

    echo "✅ Project initialized!"
    echo ""
    echo "Next steps:"
    echo "  1. Run 'just validate-gpu' to check GPU"
    echo "  2. Run 'just test' to verify setup"
    echo "  3. See 'just --list' for all commands"

# Validate DGX-Spark hardware prerequisites
validate-gpu:
    #!/usr/bin/env nu
    use scripts/nu/config.nu *
    check-dgx-prerequisites

# Show hardware information
hw-info:
    #!/usr/bin/env nu
    use scripts/nu/config.nu *
    use scripts/nu/modules/dgx.nu *

    log-header "DGX-Spark Hardware Information"
    dgx-gpu-stats
    dgx-validate-hardware

# === Build Commands ===

# Build Rust TUI (debug mode)
build:
    cargo build --workspace

# Build Rust TUI (release mode, optimized)
build-release:
    cargo build --workspace --release

# Clean build artifacts
clean:
    cargo clean
    rm -rf python/__pycache__
    rm -rf target/
    find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true

# === Development Commands ===

# Run Rust TUI in debug mode
tui:
    cargo run --package dgx-pixels-tui

# Run Rust TUI in release mode
tui-release:
    cargo run --package dgx-pixels-tui --release

# Start Python backend worker
backend PORT="5555":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -d "venv" ]; then
        echo "❌ Virtual environment not found. Run 'just init' first."
        exit 1
    fi
    source venv/bin/activate
    python python/workers/generation_worker.py --port {{PORT}}

# Start ComfyUI server (assumes installed)
comfyui PORT="8188":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -d "$HOME/ComfyUI" ]; then
        cd $HOME/ComfyUI
        python main.py --port {{PORT}}
    else
        echo "❌ ComfyUI not found. Install it first."
        echo "   See: docs/orchestration/workstreams/ws04-comfyui-setup/"
        exit 1
    fi

# Check ComfyUI health
check-comfyui:
    #!/usr/bin/env nu
    use scripts/nu/modules/comfyui.nu *
    comfyui-health-check

# === Testing ===

# Run all tests
test:
    @echo "Running Rust tests..."
    cargo test --workspace
    @echo ""
    @echo "Running Python tests..."
    @if [ -d "venv" ] && [ -d "python/tests" ]; then \
        source venv/bin/activate && pytest python/tests/ -v; \
    else \
        echo "⚠️  Python tests not yet set up"; \
    fi

# Run tests with coverage
test-coverage:
    cargo tarpaulin --workspace --out Html --output-dir coverage
    @echo "Coverage report: coverage/index.html"

# Run integration tests
test-integration:
    cargo test --workspace --test '*' -- --test-threads=1

# === Benchmarking ===

# Run performance benchmarks
bench:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🏃 Running benchmarks..."

    # Rust benchmarks
    if [ -d "benches" ]; then
        cargo bench --workspace
    fi

    # Hardware benchmarks
    if [ -f "bench/throughput.py" ]; then
        source venv/bin/activate
        python bench/throughput.py
    fi

# Baseline hardware performance
bench-baseline:
    #!/usr/bin/env nu
    use scripts/nu/modules/dgx.nu *

    log-header "Running Baseline Benchmarks"

    # GPU stats
    dgx-gpu-stats | save -f bench/baselines/gpu_baseline.json

    # Topology
    dgx-export-topology | save -f bench/baselines/topology.txt

    log-success "Baselines saved to bench/baselines/"

# === Model Management ===

# List available models
models-list:
    #!/usr/bin/env bash
    echo "📦 Models:"
    echo ""
    echo "Checkpoints:"
    ls -lh models/checkpoints/ 2>/dev/null || echo "  (none)"
    echo ""
    echo "LoRAs:"
    ls -lh models/loras/ 2>/dev/null || echo "  (none)"

# Download base SDXL model
download-model:
    #!/usr/bin/env bash
    echo "⬇️  Downloading SDXL 1.0 base model..."
    echo "⚠️  This is a large file (~6.5 GB)"
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi

    mkdir -p models/checkpoints
    cd models/checkpoints

    # Download from Hugging Face
    wget -c https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/resolve/main/sd_xl_base_1.0.safetensors

    echo "✅ Model downloaded to models/checkpoints/"

# Train LoRA on custom dataset
train-lora DATASET:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -d "{{DATASET}}" ]; then
        echo "❌ Dataset not found: {{DATASET}}"
        exit 1
    fi

    echo "🎓 Training LoRA on dataset: {{DATASET}}"
    source venv/bin/activate
    python python/training/lora_trainer.py --dataset "{{DATASET}}"

# === Code Quality ===

# Format Rust code
fmt:
    cargo fmt --all

# Lint Rust code
lint:
    cargo clippy --workspace -- -D warnings

# Format Python code
fmt-python:
    #!/usr/bin/env bash
    if [ -d "venv" ]; then
        source venv/bin/activate
        ruff format python/ || black python/ || echo "⚠️  No Python formatter installed"
    fi

# Run all quality checks (CI)
ci: fmt lint test
    @echo "✅ All CI checks passed!"

# === Documentation ===

# Generate Rust API docs
docs:
    cargo doc --workspace --no-deps --open

# Serve documentation locally
docs-serve PORT="8000":
    python3 -m http.server {{PORT}} --directory docs/

# === Monitoring & Operations ===

# Monitor GPU usage (live)
gpu-watch:
    watch -n 1 nvidia-smi

# Show GPU status (one-time)
gpu-status:
    nvidia-smi --query-gpu=name,memory.used,memory.total,temperature.gpu,utilization.gpu,power.draw --format=csv

# Show detailed GPU stats
gpu-stats:
    #!/usr/bin/env nu
    use scripts/nu/modules/dgx.nu *
    dgx-gpu-stats

# View backend logs (if running as service)
logs:
    journalctl -u dgx-pixels-backend -f

# === Git Commands ===

# Show current branch and status
status:
    @git branch --show-current
    @git status --short

# Create feature branch for workstream
branch WS_ID:
    #!/usr/bin/env nu
    use scripts/nu/modules/github.nu *
    gh-create-branch "{{WS_ID}}"

# Create PR for current branch
pr TITLE:
    #!/usr/bin/env nu
    use scripts/nu/modules/github.nu *
    gh-create-pr "{{TITLE}}"

# Rebase current branch onto main
rebase:
    #!/usr/bin/env nu
    use scripts/nu/modules/github.nu *
    gh-rebase-main

# Run pre-commit checks
pre-commit: fmt lint test
    @echo "✅ Pre-commit checks passed! Ready to commit."

# === Orchestration Commands ===

# Start Foundation Orchestrator (M0)
orch-foundation:
    @echo "🚀 Starting Foundation Orchestrator..."
    @echo "This will execute WS-01, WS-02, WS-03 sequentially."
    @echo ""
    @echo "See: docs/orchestration/orchestrators/foundation.md"

# Start Model Orchestrator (M1, M3)
orch-model:
    @echo "🚀 Starting Model Orchestrator..."
    @echo "This will execute WS-04, WS-05, WS-06, WS-07."
    @echo ""
    @echo "See: docs/orchestration/orchestrators/model.md"

# Start Interface Orchestrator (M2)
orch-interface:
    @echo "🚀 Starting Interface Orchestrator..."
    @echo "This will execute WS-08, WS-09, WS-10, WS-11, WS-12."
    @echo ""
    @echo "See: docs/orchestration/orchestrators/interface.md"

# Start Integration Orchestrator (M4, M5)
orch-integration:
    @echo "🚀 Starting Integration Orchestrator..."
    @echo "This will execute WS-13, WS-14, WS-15, WS-16, WS-17, WS-18."
    @echo ""
    @echo "See: docs/orchestration/orchestrators/integration.md"

# === Docker Commands ===

# Build Docker image
docker-build TAG="latest":
    docker build -t dgx-pixels:{{TAG}} .

# Run Docker container with GPU support
docker-run TAG="latest":
    docker run --gpus all -it --rm dgx-pixels:{{TAG}}

# Start full stack with Docker Compose
up:
    docker-compose up -d

# Stop full stack
down:
    docker-compose down

# View Docker logs
docker-logs:
    docker-compose logs -f

# === Maintenance ===

# Update dependencies
update:
    cargo update
    @if [ -d "venv" ]; then \
        source venv/bin/activate && pip list --outdated; \
    fi

# Check for security vulnerabilities
audit:
    cargo audit

# Check project health
health:
    @echo "🏥 DGX-Pixels Health Check"
    @echo ""
    @just validate-gpu
    @echo ""
    @just status
    @echo ""
    @just test

# === Miscellaneous ===

# Show project information
info:
    @echo "DGX-Pixels Project Information"
    @echo "=============================="
    @echo ""
    @echo "Project: AI Pixel Art Generation"
    @echo "Hardware: DGX-Spark (GB10 Grace Blackwell)"
    @echo "Target: Bevy Game Engine"
    @echo ""
    @echo "Milestones:"
    @echo "  M0: Foundation & Baselines (Weeks 1-2)"
    @echo "  M1: Core Inference (Weeks 3-4)"
    @echo "  M2: Interactive TUI (Weeks 5-6)"
    @echo "  M3: LoRA Training (Weeks 5-6)"
    @echo "  M4: Bevy Integration (Weeks 7-9)"
    @echo "  M5: Production Ready (Weeks 10-12)"
    @echo ""
    @echo "Documentation: docs/"
    @echo "Orchestration: docs/orchestration/"

# Clean everything (including models - be careful!)
clean-all:
    @echo "⚠️  This will delete ALL build artifacts and downloaded models!"
    @read -p "Are you sure? (y/N) " -n 1 -r; \
    echo; \
    if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
        just clean; \
        rm -rf models/checkpoints/*.safetensors; \
        rm -rf models/loras/*.safetensors; \
        echo "✅ Cleaned everything"; \
    fi
