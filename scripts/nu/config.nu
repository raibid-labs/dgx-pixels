#!/usr/bin/env nu
################################################################################
# Script Name: config.nu
# Description: DGX-Pixels project configuration and shared utilities
# Author: dgx-pixels project
# Created: 2025-11-10
# Modified: 2025-11-10
#
# Usage: source scripts/nu/config.nu
#
# Provides:
#   - Color constants for terminal output
#   - Logging functions (success, error, warning, info)
#   - File system utilities
#   - Command existence checks
#   - Project paths and configuration
#
# Dependencies:
#   - nushell >= 0.96
################################################################################

# === Color Scheme ===

export const COLORS = {
    success: (ansi green)
    error: (ansi red)
    warning: (ansi yellow)
    info: (ansi blue)
    debug: (ansi cyan)
    header: (ansi green_bold)
    reset: (ansi reset)
}

# === Logging Functions ===

# Log success message
export def log-success [message: string] {
    print $"($COLORS.success)✓($COLORS.reset) ($message)"
}

# Log error message
export def log-error [message: string] {
    print $"($COLORS.error)✗($COLORS.reset) ($message)"
}

# Log warning message
export def log-warning [message: string] {
    print $"($COLORS.warning)⚠($COLORS.reset) ($message)"
}

# Log info message
export def log-info [message: string] {
    print $"($COLORS.info)ℹ($COLORS.reset) ($message)"
}

# Log debug message (only if DEBUG env var is set)
export def log-debug [message: string] {
    if "DEBUG" in $env {
        print $"($COLORS.debug)🐛($COLORS.reset) ($message)"
    }
}

# Log section header
export def log-header [message: string] {
    print $"\n($COLORS.header)━━━ ($message) ━━━($COLORS.reset)"
}

# === Command Utilities ===

# Check if a command exists in PATH
export def command-exists [cmd: string] {
    (which $cmd | length) > 0
}

# Require a command to exist (exit if not found)
export def require-command [cmd: string, install_hint?: string] {
    if not (command-exists $cmd) {
        log-error $"Required command not found: ($cmd)"
        if ($install_hint != null) {
            log-info $"Install with: ($install_hint)"
        }
        exit 1
    }
}

# === Project Paths ===

# Get project root directory
export def project-root [] {
    # Navigate up from scripts/nu/ to project root
    $env.PWD | path dirname | path dirname | path dirname
}

# Get docs directory
export def docs-dir [] {
    (project-root) | path join "docs"
}

# Get models directory
export def models-dir [] {
    (project-root) | path join "models"
}

# Get checkpoints directory
export def checkpoints-dir [] {
    (models-dir) | path join "checkpoints"
}

# Get loras directory
export def loras-dir [] {
    (models-dir) | path join "loras"
}

# Get workflows directory
export def workflows-dir [] {
    (project-root) | path join "workflows"
}

# === File System Utilities ===

# Ensure directory exists (create if missing)
export def ensure-dir [dir: path] {
    if not ($dir | path exists) {
        mkdir $dir
        log-info $"Created directory: ($dir)"
    }
}

# Check if path is inside project root
export def is-in-project [file: path] {
    let root = (project-root)
    ($file | path expand | str starts-with ($root | path expand))
}

# === Git Utilities ===

# Get current git branch
export def current-branch [] {
    (git rev-parse --abbrev-ref HEAD | str trim)
}

# Check if working directory is clean
export def is-git-clean [] {
    (git status --porcelain | str trim | is-empty)
}

# Get current commit SHA (short)
export def current-commit [] {
    (git rev-parse --short HEAD | str trim)
}

# === Hardware Detection ===

# Check if NVIDIA GPU is available
export def has-nvidia-gpu [] {
    (command-exists "nvidia-smi") and ((nvidia-smi --query-gpu=name --format=csv,noheader | length) > 0)
}

# Get GPU model name
export def gpu-model [] {
    if (has-nvidia-gpu) {
        nvidia-smi --query-gpu=name --format=csv,noheader | str trim
    } else {
        "No NVIDIA GPU detected"
    }
}

# Get CUDA version
export def cuda-version [] {
    if (command-exists "nvcc") {
        nvcc --version | lines | get 3 | parse "release {version}," | get version.0
    } else {
        "CUDA not found"
    }
}

# === Environment Checks ===

# Verify DGX-Spark prerequisites
export def check-dgx-prerequisites [] {
    log-header "Checking DGX-Spark Prerequisites"

    # Check GPU
    if (has-nvidia-gpu) {
        log-success $"GPU detected: (gpu-model)"
    } else {
        log-error "No NVIDIA GPU detected"
        exit 1
    }

    # Check CUDA
    if (command-exists "nvcc") {
        log-success $"CUDA version: (cuda-version)"
    } else {
        log-warning "CUDA compiler (nvcc) not found"
    }

    # Check Docker
    if (command-exists "docker") {
        log-success "Docker installed"

        # Check NVIDIA Container Toolkit
        try {
            docker run --rm --gpus all nvidia/cuda:13.0-base-ubuntu22.04 nvidia-smi
            log-success "NVIDIA Container Toolkit working"
        } catch {
            log-warning "NVIDIA Container Toolkit may not be configured"
        }
    } else {
        log-warning "Docker not found"
    }

    # Check Python
    if (command-exists "python3") {
        let py_version = (python3 --version | parse "Python {version}" | get version.0)
        log-success $"Python version: ($py_version)"
    } else {
        log-error "Python 3 not found"
        exit 1
    }

    # Check Rust
    if (command-exists "cargo") {
        let rust_version = (cargo --version | parse "cargo {version}" | get version.0)
        log-success $"Rust (cargo) version: ($rust_version)"
    } else {
        log-warning "Rust (cargo) not found - needed for TUI development"
    }
}

# === Configuration Management ===

# Load project configuration from TOML
export def load-config [] {
    let config_file = (project-root) | path join "dgx-pixels.toml"

    if ($config_file | path exists) {
        open $config_file
    } else {
        log-warning "Config file not found, using defaults"
        {
            api_port: 8000
            zmq_port: 5555
            comfyui_url: "http://localhost:8188"
            models_dir: "models"
        }
    }
}

# === Exit Code Constants ===

export const EXIT_SUCCESS = 0
export const EXIT_ERROR = 1
export const EXIT_INVALID_ARGS = 2
export const EXIT_MISSING_DEPS = 3
export const EXIT_GPU_NOT_FOUND = 4

# === Banner ===

export def show-banner [] {
    print $"
($COLORS.header)╔═══════════════════════════════════════════════════════╗
║                    DGX-Pixels                        ║
║         AI Pixel Art Generation for Bevy Games      ║
╚═══════════════════════════════════════════════════════╝($COLORS.reset)
"
}

# === Initialization ===

# Run on module load
# show-banner
