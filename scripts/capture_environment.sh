#!/bin/bash
################################################################################
# DGX-Pixels Environment Capture Script
#
# Captures complete environment information in JSON format for reproducibility.
# This script can run on host or inside Docker container.
#
# Captures:
#   - Git information (SHA, branch, dirty status)
#   - CUDA and driver versions
#   - GPU information
#   - Python and package versions
#   - System information
#
# Usage:
#   bash scripts/capture_environment.sh > bench/baselines/env_$(date +%Y%m%d).json
#
# Output: JSON to stdout
################################################################################

set -e

# Detect if running in container
if [ -f /.dockerenv ]; then
    IN_CONTAINER=true
else
    IN_CONTAINER=false
fi

################################################################################
# Helper Functions
################################################################################

# Get git information (if available)
get_git_info() {
    local git_sha="N/A"
    local git_branch="N/A"
    local git_dirty=false
    local git_remote="N/A"

    if command -v git &> /dev/null && [ -d .git ]; then
        git_sha=$(git rev-parse HEAD 2>/dev/null || echo "N/A")
        git_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A")
        git_remote=$(git config --get remote.origin.url 2>/dev/null || echo "N/A")

        if ! git diff-index --quiet HEAD -- 2>/dev/null; then
            git_dirty=true
        fi
    fi

    cat <<EOF
  "git": {
    "sha": "$git_sha",
    "branch": "$git_branch",
    "dirty": $git_dirty,
    "remote": "$git_remote"
  },
EOF
}

# Get CUDA information
get_cuda_info() {
    local cuda_version="N/A"
    local cuda_driver="N/A"
    local cudnn_version="N/A"

    if command -v nvcc &> /dev/null; then
        cuda_version=$(nvcc --version | grep "release" | sed 's/.*release //' | sed 's/,.*//')
    fi

    if command -v nvidia-smi &> /dev/null; then
        cuda_driver=$(nvidia-smi --query-gpu=driver_version --format=csv,noheader | head -1)
    fi

    # Try to get cuDNN version (from Python if available)
    if command -v python3 &> /dev/null; then
        cudnn_version=$(python3 -c "import torch; print(torch.backends.cudnn.version())" 2>/dev/null || echo "N/A")
    fi

    cat <<EOF
  "cuda": {
    "version": "$cuda_version",
    "driver": "$cuda_driver",
    "cudnn": "$cudnn_version"
  },
EOF
}

# Get GPU information
get_gpu_info() {
    local gpu_name="N/A"
    local gpu_count=0
    local gpu_memory="N/A"
    local gpu_compute_capability="N/A"

    if command -v nvidia-smi &> /dev/null; then
        gpu_name=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
        gpu_count=$(nvidia-smi --query-gpu=name --format=csv,noheader | wc -l)
        gpu_memory=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader | head -1)
    fi

    # Get compute capability from Python if available
    if command -v python3 &> /dev/null; then
        gpu_compute_capability=$(python3 -c "
import torch
if torch.cuda.is_available():
    cap = torch.cuda.get_device_capability(0)
    print(f'{cap[0]}.{cap[1]}')
" 2>/dev/null || echo "N/A")
    fi

    cat <<EOF
  "gpu": {
    "name": "$gpu_name",
    "count": $gpu_count,
    "memory": "$gpu_memory",
    "compute_capability": "$gpu_compute_capability"
  },
EOF
}

# Get Python information
get_python_info() {
    local python_version="N/A"
    local python_path="N/A"

    if command -v python3 &> /dev/null; then
        python_version=$(python3 --version 2>&1 | sed 's/Python //')
        python_path=$(which python3)
    fi

    cat <<EOF
  "python": {
    "version": "$python_version",
    "path": "$python_path"
  },
EOF
}

# Get PyTorch information
get_pytorch_info() {
    local pytorch_version="N/A"
    local pytorch_cuda="N/A"
    local pytorch_cuda_available=false
    local pytorch_build="N/A"

    if command -v python3 &> /dev/null; then
        pytorch_version=$(python3 -c "import torch; print(torch.__version__)" 2>/dev/null || echo "N/A")
        pytorch_cuda=$(python3 -c "import torch; print(torch.version.cuda)" 2>/dev/null || echo "N/A")
        pytorch_cuda_available=$(python3 -c "import torch; print('true' if torch.cuda.is_available() else 'false')" 2>/dev/null || echo "false")
        pytorch_build=$(python3 -c "import torch; print(torch.version.git_version)" 2>/dev/null || echo "N/A")
    fi

    cat <<EOF
  "pytorch": {
    "version": "$pytorch_version",
    "cuda": "$pytorch_cuda",
    "cuda_available": $pytorch_cuda_available,
    "build": "$pytorch_build"
  },
EOF
}

# Get installed Python packages
get_python_packages() {
    if command -v pip3 &> /dev/null; then
        # Get packages as JSON array
        pip3 list --format=json 2>/dev/null | jq -c '.' 2>/dev/null || echo "[]"
    else
        echo "[]"
    fi
}

# Get system information
get_system_info() {
    local hostname=$(hostname)
    local kernel=$(uname -r)
    local arch=$(uname -m)
    local os="N/A"

    if [ -f /etc/os-release ]; then
        os=$(grep PRETTY_NAME /etc/os-release | cut -d'"' -f2)
    fi

    local cpu_model="N/A"
    local cpu_cores=0

    if [ -f /proc/cpuinfo ]; then
        cpu_model=$(grep "model name" /proc/cpuinfo | head -1 | cut -d':' -f2 | xargs)
        cpu_cores=$(grep -c "processor" /proc/cpuinfo)
    fi

    local mem_total="N/A"
    if command -v free &> /dev/null; then
        mem_total=$(free -h | grep Mem | awk '{print $2}')
    fi

    cat <<EOF
  "system": {
    "hostname": "$hostname",
    "kernel": "$kernel",
    "architecture": "$arch",
    "os": "$os",
    "cpu_model": "$cpu_model",
    "cpu_cores": $cpu_cores,
    "memory_total": "$mem_total",
    "in_container": $IN_CONTAINER
  },
EOF
}

################################################################################
# Main Execution
################################################################################

# Generate timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Output JSON
cat <<EOF
{
  "version": "1.0",
  "timestamp": "$TIMESTAMP",
  "captured_by": "capture_environment.sh",
$(get_git_info)
$(get_cuda_info)
$(get_gpu_info)
$(get_python_info)
$(get_pytorch_info)
$(get_system_info)
  "packages": $(get_python_packages)
}
EOF
