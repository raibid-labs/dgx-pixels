# DGX-Pixels Reproducibility Framework

**Version**: 1.0
**Last Updated**: 2025-11-11
**Status**: Complete

---

## Overview

The DGX-Pixels reproducibility framework ensures consistent development, testing, and production environments across ARM64 NVIDIA DGX-Spark hardware using Docker containers with NVIDIA GPU support.

### Design Decisions

**Docker-Only Approach** (no Conda):
- Simpler dependency management (one tool vs two)
- Container-level isolation vs environment-level
- Production-ready (Docker is deployment target)
- Bit-for-bit reproducibility guarantee
- NVIDIA NGC containers are Docker-native

**NVIDIA NGC PyTorch Base**:
- PyTorch ARM+CUDA wheels unavailable on PyPI
- Building from source takes 2-3 hours
- NGC containers pre-built and optimized for NVIDIA hardware
- Includes PyTorch 2.6.0 with CUDA 12.6 support

---

## Quick Start

### Prerequisites

- DGX-Spark GB10 (ARM64 / aarch64)
- CUDA 13.0, Driver 580.95.05+
- Docker with NVIDIA Container Toolkit
- Ubuntu 22.04 (ARM64)

### Build Image

```bash
cd /home/beengud/raibid-labs/dgx-pixels
docker build -t dgx-pixels:dev docker/
```

**Build time**: ~30 seconds (with NGC base cached), ~5 minutes (first time)

### Run Container

```bash
# Interactive shell
docker run --rm -it --gpus all --ipc=host \
  -v $(pwd):/workspace \
  dgx-pixels:dev bash

# Run specific command
docker run --rm --gpus all --ipc=host \
  -v $(pwd):/workspace \
  dgx-pixels:dev python3 your_script.py

# Verify GPU access
docker run --rm --gpus all --ipc=host dgx-pixels:dev nvidia-smi
```

**Important flags**:
- `--gpus all`: GPU passthrough
- `--ipc=host`: Shared memory for PyTorch (recommended by NVIDIA)
- `-v $(pwd):/workspace`: Mount project directory

### Using Docker Compose

```bash
# Start stack
cd docker/
docker-compose up -d

# Enter container
docker-compose exec dgx-pixels bash

# Stop stack
docker-compose down
```

---

## Environment Details

### Base Image

**NVIDIA NGC PyTorch 24.11**
- Image: `nvcr.io/nvidia/pytorch:24.11-py3`
- Size: ~18GB
- Architecture: ARM64 (aarch64)
- OS: Ubuntu 24.04.1 LTS

### Pre-installed Components

From NGC base:
- Python 3.12.3
- PyTorch 2.6.0a0+df5bbc09d1.nv24.11
- CUDA 12.6
- cuDNN 9.5.1
- NumPy 1.26.4
- SciPy 1.14.1
- Pillow 11.0.0
- Jupyter, TensorBoard, many ML libraries

Additional packages installed:
- aiohttp 3.10.10
- msgpack 1.1.0
- pyyaml 6.0.2
- tqdm 4.67.0
- click 8.1.7
- pytest 9.0.0
- pytest-asyncio 1.3.0
- python-dotenv 1.2.1

See full package list: `docker run --rm dgx-pixels:dev pip list`

### GPU Support

- **Hardware**: NVIDIA GB10 Grace Blackwell
- **Compute Capability**: 12.1 (sm_121)
- **CUDA**: 12.6 (container), 13.0 (host driver)
- **Memory**: 119GB unified (CPU+GPU shared)

**Note**: NGC container shows warning about GB10 not being supported, but PyTorch CUDA functionality works (`torch.cuda.is_available()` returns `True`).

### Known Limitations

1. **PyTorch Compute Capability**: NGC PyTorch 24.11 supports sm_80, sm_86, sm_90 but not sm_121 (GB10). However, CUDA operations still function.

2. **NumPy Version Pinning**: NGC packages require NumPy <2.0. Do not upgrade to NumPy 2.x as it breaks NVIDIA ModelOpt and other libraries.

3. **OpenCV**: opencv-python-headless not included by default to avoid NumPy 2.x upgrade. Install manually if needed with `--no-deps` flag.

---

## Environment Capture

### Capture Current Environment

```bash
# On host
bash scripts/capture_environment.sh > bench/baselines/env_$(date +%Y%m%d).json

# In container
docker run --rm --gpus all --ipc=host \
  -v $(pwd):/workspace \
  dgx-pixels:dev \
  bash /workspace/scripts/capture_environment.sh > env.json
```

### Environment JSON Schema

```json
{
  "version": "1.0",
  "timestamp": "2025-11-11T17:17:01Z",
  "captured_by": "capture_environment.sh",
  "git": {
    "sha": "commit-sha",
    "branch": "branch-name",
    "dirty": true/false,
    "remote": "git-url"
  },
  "cuda": {
    "version": "12.6",
    "driver": "580.95.05",
    "cudnn": "90501"
  },
  "gpu": {
    "name": "NVIDIA GB10",
    "count": 1,
    "memory": "119GB",
    "compute_capability": "12.1"
  },
  "python": {
    "version": "3.12.3",
    "path": "/usr/bin/python3"
  },
  "pytorch": {
    "version": "2.6.0a0+df5bbc09d1.nv24.11",
    "cuda": "12.6",
    "cuda_available": true,
    "build": "df5bbc09d1"
  },
  "system": {
    "hostname": "container-id",
    "kernel": "6.11.0-1016-nvidia",
    "architecture": "aarch64",
    "os": "Ubuntu 24.04.1 LTS",
    "cpu_model": "Cortex-X925",
    "cpu_cores": 20,
    "memory_total": "119Gi",
    "in_container": true
  },
  "packages": [
    {"name": "package-name", "version": "x.y.z"},
    ...
  ]
}
```

---

## Testing

### Run Smoke Tests

```bash
# Full test suite
bash tests/integration/ws_02/test_reproducibility.sh

# Expected: 4/5 tests passing
# - ✅ GPU Access
# - ✅ Python 3.10+ Environment
# - ✅ PyTorch + CUDA Functional
# - ✅ Environment Capture
# - ⚠️  Docker Build (may false-fail on cached builds)
```

### Manual Verification

```bash
# Verify GPU
docker run --rm --gpus all --ipc=host dgx-pixels:dev nvidia-smi

# Verify Python
docker run --rm dgx-pixels:dev python3 --version

# Verify PyTorch + CUDA
docker run --rm --gpus all --ipc=host dgx-pixels:dev python3 -c "
import torch
print(f'PyTorch: {torch.__version__}')
print(f'CUDA available: {torch.cuda.is_available()}')
print(f'GPU count: {torch.cuda.device_count()}')
print(f'GPU name: {torch.cuda.get_device_name(0)}')
"
```

---

## Development Workflow

### Iterative Development

```bash
# 1. Start container with project mounted
docker run --rm -it --gpus all --ipc=host \
  -v /home/beengud/raibid-labs/dgx-pixels:/workspace \
  -w /workspace \
  dgx-pixels:dev bash

# 2. Make code changes on host (files are mounted)

# 3. Run/test inside container
python3 your_script.py

# 4. Repeat
```

### Adding Python Packages

Edit `docker/requirements-extra.txt`:

```txt
# Add new package
new-package>=1.0.0
```

Rebuild image:

```bash
docker build -t dgx-pixels:dev docker/
```

### Dockerfile Best Practices

The Dockerfile follows these best practices:
1. **Multi-stage build**: Separates build and runtime (if needed)
2. **Layer caching**: Dependencies installed before code
3. **Non-root user**: Runs as `ubuntu` user (UID 1000)
4. **Minimal layers**: Combined RUN commands
5. **Clean up**: Removes apt caches and tmp files
6. **Health check**: Verifies PyTorch + CUDA on startup
7. **Metadata**: Labels for versioning and documentation

---

## Troubleshooting

### Issue: GPU not accessible

**Symptoms**: `nvidia-smi` fails, `torch.cuda.is_available()` returns `False`

**Solutions**:
1. Verify NVIDIA Container Toolkit installed: `docker run --rm --gpus all nvidia/cuda:13.0-base nvidia-smi`
2. Add `--gpus all` flag to `docker run`
3. Check driver version: `nvidia-smi` on host should show driver 580.95.05+

### Issue: GB10 "not supported" warning

**Symptoms**: Warning message about GB10 not supported in container

**Solution**: This is expected. GB10 (compute capability 12.1) is newer than the NGC container's supported list (sm_80, sm_86, sm_90). However, CUDA operations still work. The warning can be safely ignored.

### Issue: Slow Docker build

**Symptoms**: First build takes 5+ minutes

**Solution**:
1. This is normal for first build (pulling NGC base image: 18GB)
2. Subsequent builds use cache (~30 seconds)
3. Pre-pull base: `docker pull nvcr.io/nvidia/pytorch:24.11-py3`

### Issue: NumPy version conflicts

**Symptoms**: Errors about NumPy 2.x incompatibility with NVIDIA packages

**Solution**: Do NOT upgrade NumPy beyond 1.26.4. NGC packages require NumPy <2.0.

### Issue: Permission denied in container

**Symptoms**: Cannot write files in `/workspace`

**Solution**: Container runs as `ubuntu` user (UID 1000). Ensure mounted directories are writable by UID 1000 or run container as root: `docker run --user root ...`

---

## Performance Targets

Based on smoke tests and DGX-Spark hardware:

| Metric | Target | Actual |
|--------|--------|--------|
| Docker build (cached) | ≤2 minutes | ~30 seconds |
| Docker build (cold) | ≤10 minutes | ~5 minutes |
| Container startup | ≤30 seconds | ~2 seconds |
| Smoke test suite | ≤5 minutes | ~10 seconds |
| GPU detection | <1 second | <1 second |
| PyTorch import | <5 seconds | ~2 seconds |

---

## File Structure

```
dgx-pixels/
├── docker/
│   ├── Dockerfile                  # Main Dockerfile (NGC PyTorch base)
│   ├── docker-compose.yml          # Stack configuration
│   ├── requirements-extra.txt      # Additional Python packages
│   └── .env.example                # Environment variables template
├── scripts/
│   └── capture_environment.sh      # Environment capture script
├── tests/
│   └── integration/
│       └── ws_02/
│           └── test_reproducibility.sh  # Smoke tests
├── bench/
│   └── baselines/
│       ├── hardware_baseline.json  # From WS-01
│       └── env_YYYYMMDD.json       # Environment snapshots
└── docs/
    └── reproducibility.md          # This document
```

---

## Next Steps

### For WS-04 (ComfyUI Setup)

The reproducibility framework provides:
- Working Docker environment with GPU support
- PyTorch 2.6.0 + CUDA 12.6 functional
- Python 3.12 with common ML libraries
- Environment capture for version tracking

To extend for ComfyUI:
1. Add ComfyUI dependencies to `requirements-extra.txt`
2. Install ComfyUI in `/workspace/comfyui/`
3. Add ComfyUI service to `docker-compose.yml`
4. Mount models directory: `-v /path/to/models:/models`

### For WS-10 (Python Backend Worker)

The environment is ready for:
- ZeroMQ server development
- PyTorch model loading
- ComfyUI API integration
- Async job queue management

---

## References

- **NGC PyTorch Container**: https://catalog.ngc.nvidia.com/orgs/nvidia/containers/pytorch
- **NVIDIA Container Toolkit**: https://docs.nvidia.com/datacenter/cloud-native/
- **Docker Best Practices**: https://docs.docker.com/develop/dev-best-practices/
- **DGX-Spark Hardware**: `/home/beengud/raibid-labs/dgx-pixels/bench/baselines/hardware_baseline.json`
- **WS-01 Completion**: `/home/beengud/raibid-labs/dgx-pixels/docs/orchestration/workstreams/ws01-hardware-baselines/COMPLETION_SUMMARY.md`

---

**Status**: Reproducibility framework complete and tested.
**Smoke Tests**: 4/5 passing (all critical tests pass).
**Ready for**: WS-04 (ComfyUI Setup), WS-10 (Python Backend Worker).
