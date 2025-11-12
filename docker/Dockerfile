################################################################################
# DGX-Pixels Reproducibility Framework Dockerfile
#
# Target: NVIDIA DGX-Spark GB10 (ARM64 / aarch64)
# Base: NVIDIA NGC PyTorch 24.11 (includes PyTorch 2.6+ with CUDA 12.6+)
# Python: 3.12 (from NGC base)
# PyTorch: 2.6.0a0 Pre-installed with CUDA support from NGC
#
# Build:
#   docker build -t dgx-pixels:dev docker/
#
# Run:
#   docker run --rm --gpus all --ipc=host dgx-pixels:dev nvidia-smi
#   docker run --rm --gpus all --ipc=host dgx-pixels:dev python3 -c "import torch; print(torch.cuda.is_available())"
#
# Why NGC base:
#   - PyTorch wheels for ARM+CUDA are not available on PyPI
#   - Building from source takes 2-3 hours
#   - NGC containers are pre-built and optimized for NVIDIA hardware
#   - Includes PyTorch 2.6 with CUDA support
#
# Note: Using 24.11-py3 (18GB, includes PyTorch 2.6.0 + CUDA)
# Note: NGC base already has 'ubuntu' user (UID 1000), we use that
#
# WS-04 Update: Added ComfyUI dependencies and setup
################################################################################

FROM nvcr.io/nvidia/pytorch:24.11-py3

# Metadata
LABEL maintainer="DGX-Pixels Team"
LABEL description="Reproducible development environment for DGX-Pixels on DGX-Spark ARM"
LABEL version="1.1-comfyui"
LABEL architecture="aarch64"
LABEL base="NGC PyTorch 24.11"
LABEL pytorch_version="2.6.0a0"
LABEL cuda_version="12.6"

# Environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1

# Update system and install additional packages
# Note: libgl1-mesa-glx is obsolete in Ubuntu 24.04, using libgl1 instead
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Utilities
    vim \
    htop \
    tmux \
    jq \
    wget \
    curl \
    git \
    # Image processing libraries (updated for Ubuntu 24.04)
    libgl1 \
    libglib2.0-0 \
    # Clean up
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Switch to root to install system-wide Python packages
USER root

# Install additional Python dependencies (beyond NGC base)
# NOTE: Do NOT install torch/torchvision - use what NGC provides
COPY requirements-extra.txt /tmp/requirements-extra.txt
COPY requirements-comfyui.txt /tmp/requirements-comfyui.txt

RUN pip3 install --no-cache-dir -r /tmp/requirements-extra.txt && \
    pip3 install --no-cache-dir -r /tmp/requirements-comfyui.txt && \
    rm /tmp/requirements-extra.txt /tmp/requirements-comfyui.txt

# Create workspace directories and set permissions for ubuntu user (UID 1000)
# NGC base already has 'ubuntu' user
RUN mkdir -p /workspace /models /outputs /config && \
    chown -R ubuntu:ubuntu /workspace /models /outputs /config

# Set working directory
WORKDIR /workspace

# Switch to non-root user (use existing ubuntu user from NGC base)
USER ubuntu

# Health check - verify CUDA is accessible
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python3 -c "import torch; assert torch.cuda.is_available()" || exit 1

# Default command: show environment info
CMD ["python3", "-c", "import sys; import torch; print(f'Python {sys.version}'); print(f'PyTorch {torch.__version__}'); print(f'CUDA available: {torch.cuda.is_available()}'); print(f'GPU count: {torch.cuda.device_count()}')"]
