#!/usr/bin/env nu
################################################################################
# Script Name: dgx.nu
# Description: DGX-Spark specific hardware utilities for DGX-Pixels project
# Author: dgx-pixels project
# Created: 2025-11-10
# Modified: 2025-11-10
#
# Usage: use scripts/nu/modules/dgx.nu *
#
# Provides:
#   - dgx-gpu-stats: Get GPU stats (memory, utilization, temp)
#   - dgx-validate-hardware: Validate GB10, unified memory, ARM CPU
#   - dgx-benchmark-memory: Test unified memory bandwidth
#   - dgx-export-topology: Export nvidia-smi topology
#   - dgx-get-cpu-info: Get ARM CPU details
#   - dgx-check-tensor-cores: Verify Tensor Core availability
#
# Dependencies:
#   - nushell >= 0.96
#   - nvidia-smi (NVIDIA drivers)
#   - Optional: nvcc (CUDA toolkit)
################################################################################

use ../config.nu [COLORS, log-success, log-error, log-warning, log-info, command-exists]

# Get comprehensive GPU statistics
#
# Queries nvidia-smi for current GPU status including memory, utilization, temperature
#
# Returns: table - GPU statistics with formatted columns
#
# Example:
#   dgx-gpu-stats
#   # Displays table with GPU index, name, memory, utilization, temperature
export def dgx-gpu-stats [] {
    if not (command-exists "nvidia-smi") {
        log-error "nvidia-smi not found - NVIDIA drivers may not be installed"
        return []
    }

    try {
        let stats = (
            nvidia-smi --query-gpu=index,name,memory.total,memory.used,memory.free,utilization.gpu,utilization.memory,temperature.gpu,power.draw,power.limit
            --format=csv,noheader,nounits
            | from csv --noheaders
            | rename gpu_id name memory_total_mb memory_used_mb memory_free_mb gpu_util_pct memory_util_pct temp_c power_draw_w power_limit_w
            | update gpu_id { into int }
            | update memory_total_mb { into int }
            | update memory_used_mb { into int }
            | update memory_free_mb { into int }
            | update gpu_util_pct { into int }
            | update memory_util_pct { into int }
            | update temp_c { into int }
            | update power_draw_w { into float }
            | update power_limit_w { into float }
        )

        log-success $"Retrieved stats for ($stats | length) GPU(s)"
        return $stats
    } catch {|err|
        log-error $"Failed to get GPU stats: ($err.msg)"
        return []
    }
}

# Get GPU statistics in JSON format
#
# Returns GPU stats as structured records for programmatic use
#
# Returns: list<record> - List of GPU stat records
#
# Example:
#   let gpus = (dgx-gpu-stats-json)
#   $gpus | each {|gpu| print $"GPU ($gpu.gpu_id): ($gpu.memory_used_mb)MB / ($gpu.memory_total_mb)MB"}
export def dgx-gpu-stats-json [] {
    dgx-gpu-stats
}

# Validate DGX-Spark hardware configuration
#
# Checks for:
#   - NVIDIA Grace Blackwell GPU (GB10)
#   - Unified memory architecture
#   - ARM-based CPU
#   - Expected memory capacity (128GB)
#
# Returns: record - Validation results with boolean flags and details
#
# Example:
#   let validation = (dgx-validate-hardware)
#   if $validation.is_dgx_spark {
#       print "Running on DGX-Spark!"
#   }
export def dgx-validate-hardware [] {
    log-info "Validating DGX-Spark hardware configuration..."

    # Check for NVIDIA GPU
    if not (command-exists "nvidia-smi") {
        log-error "nvidia-smi not found - cannot validate GPU"
        return {
            is_dgx_spark: false
            has_nvidia_gpu: false
            is_grace_blackwell: false
            has_unified_memory: false
            is_arm_cpu: false
            memory_gb: 0
            gpu_count: 0
            warnings: ["nvidia-smi not found"]
        }
    }

    # Query GPU information
    let gpu_result = (do {
        try {
            let gpu_info = (
                nvidia-smi --query-gpu=name,memory.total,count
                --format=csv,noheader
                | lines
                | first
                | split column ", "
                | rename name memory_total count
            )

            let gpu_name = ($gpu_info | get name.0)
            let memory_total = ($gpu_info | get memory_total.0 | str trim)

            let is_grace_blackwell = (
                ($gpu_name | str contains -i "grace") or
                ($gpu_name | str contains -i "blackwell") or
                ($gpu_name | str contains -i "gb10")
            )

            if $is_grace_blackwell {
                log-success $"Detected Grace Blackwell GPU: ($gpu_name)"
            } else {
                log-warning $"GPU detected but not Grace Blackwell: ($gpu_name)"
            }

            let memory_gb = (
                $memory_total
                | str replace " MiB" ""
                | into float
                | $in / 1024
                | math round
            )

            if $memory_gb >= 120 {
                log-success $"Memory capacity: ($memory_gb)GB"
            } else {
                log-warning $"Memory capacity ($memory_gb)GB is less than expected 128GB"
            }

            let gpu_count = (nvidia-smi --query-gpu=count --format=csv,noheader | lines | length)

            {
                success: true
                is_grace_blackwell: $is_grace_blackwell
                memory_gb: $memory_gb
                gpu_count: $gpu_count
                gpu_name: $gpu_name
            }
        } catch {|err|
            log-error $"Failed to query GPU info: ($err.msg)"
            {
                success: false
                is_grace_blackwell: false
                memory_gb: 0
                gpu_count: 0
                gpu_name: ""
            }
        }
    })

    # Check CPU architecture
    let cpu_result = (do {
        try {
            let cpu_arch = (^uname -m | str trim)
            let is_arm = (($cpu_arch | str starts-with "aarch64") or ($cpu_arch | str starts-with "arm"))

            if $is_arm {
                log-success $"ARM CPU detected: ($cpu_arch)"
            } else {
                log-warning $"CPU architecture is not ARM: ($cpu_arch)"
            }

            { success: true, is_arm: $is_arm }
        } catch {
            log-warning "Failed to detect CPU architecture"
            { success: false, is_arm: false }
        }
    })

    # Check for unified memory
    let has_unified_memory = (do {
        try {
            let topology = (nvidia-smi topo -m | complete)
            let has_unified = (($topology.stdout | str contains -i "nvlink") or ($topology.stdout | str contains -i "unified"))

            if $has_unified {
                log-success "Unified memory architecture detected"
            } else {
                log-info "Could not confirm unified memory from topology"
            }

            $has_unified
        } catch {
            log-info "Could not query memory topology"
            false
        }
    })

    # Build result
    let result = {
        is_dgx_spark: (
            $gpu_result.success and
            $gpu_result.is_grace_blackwell and
            $cpu_result.is_arm and
            $gpu_result.memory_gb >= 120
        )
        has_nvidia_gpu: true
        is_grace_blackwell: $gpu_result.is_grace_blackwell
        has_unified_memory: $has_unified_memory
        is_arm_cpu: $cpu_result.is_arm
        memory_gb: $gpu_result.memory_gb
        gpu_count: $gpu_result.gpu_count
        warnings: []
    }

    if $result.is_dgx_spark {
        log-success "✓ Validated DGX-Spark hardware configuration"
    } else {
        log-warning "Hardware does not match DGX-Spark specification"
    }

    return $result
}

# Benchmark unified memory bandwidth
#
# Tests memory transfer speeds between CPU and GPU to validate unified memory performance
# Requires CUDA samples or custom benchmark tool
#
# Parameters:
#   size_mb?: int - Test data size in MB (default: 1024)
#
# Returns: record - Benchmark results with bandwidth measurements
#
# Example:
#   let benchmark = (dgx-benchmark-memory 512)
#   print $"Memory bandwidth: ($benchmark.bandwidth_gbps) GB/s"
export def dgx-benchmark-memory [
    size_mb: int = 1024
] {
    log-info $"Benchmarking unified memory bandwidth with ($size_mb)MB test size..."

    # Check if bandwidthTest is available (from CUDA samples)
    if (command-exists "bandwidthTest") {
        let result = (do {
            try {
                log-info "Running CUDA bandwidthTest..."

                let output = (^bandwidthTest --htod --dtoh --memory=pinned --mode=quick | complete)

                if $output.exit_code == 0 {
                    # Parse bandwidth from output
                    let bandwidth_line = (
                        $output.stdout
                        | lines
                        | where {|line| $line | str contains "GB/s"}
                        | first
                    )

                    let bandwidth = (
                        $bandwidth_line
                        | parse "{desc}: {bandwidth} GB/s"
                        | get bandwidth
                        | first
                        | into float
                    )

                    log-success $"Memory bandwidth: ($bandwidth) GB/s"

                    {
                        success: true
                        bandwidth_gbps: $bandwidth
                        latency_us: 0.0
                        method: "cuda-bandwidth-test"
                        error: null
                    }
                } else {
                    {
                        success: false
                        bandwidth_gbps: 0.0
                        latency_us: 0.0
                        method: "none"
                        error: "bandwidthTest exited with error"
                    }
                }
            } catch {|err|
                log-warning $"bandwidthTest failed: ($err.msg)"
                {
                    success: false
                    bandwidth_gbps: 0.0
                    latency_us: 0.0
                    method: "none"
                    error: $err.msg
                }
            }
        })

        return $result
    }

    log-warning "bandwidthTest not found - cannot benchmark memory"
    log-info "Install CUDA samples for accurate benchmarking:"
    log-info "  git clone https://github.com/NVIDIA/cuda-samples.git"
    log-info "  cd cuda-samples/Samples/1_Utilities/bandwidthTest && make"

    return {
        success: false
        bandwidth_gbps: 0.0
        latency_us: 0.0
        method: "none"
        error: "bandwidthTest not available"
    }
}

# Export nvidia-smi topology information
#
# Exports GPU topology matrix showing interconnects (NVLink, PCIe, etc.)
#
# Parameters:
#   output_file?: string - Optional file to save topology (default: print to stdout)
#
# Returns: string - Topology matrix as text
#
# Example:
#   dgx-export-topology "topology.txt"
export def dgx-export-topology [
    output_file?: string
] {
    if not (command-exists "nvidia-smi") {
        log-error "nvidia-smi not found"
        return ""
    }

    try {
        let topology = (nvidia-smi topo -m)

        if ($output_file != null) {
            $topology | save -f $output_file
            log-success $"Topology exported to ($output_file)"
        } else {
            print $topology
        }

        return $topology
    } catch {|err|
        log-error $"Failed to export topology: ($err.msg)"
        return ""
    }
}

# Get detailed CPU information
#
# Returns ARM CPU details including model, cores, frequency
#
# Returns: record - CPU information
#
# Example:
#   let cpu = (dgx-get-cpu-info)
#   print $"CPU: ($cpu.model), Cores: ($cpu.cores)"
export def dgx-get-cpu-info [] {
    let arch = (do { try { ^uname -m | str trim } catch { "unknown" } })

    let lscpu_info = (do {
        if (command-exists "lscpu") {
            try {
                let lscpu_output = (^lscpu | lines)

                let model_line = ($lscpu_output | where {|line| $line | str starts-with "Model name:"} | first)
                let model = if ($model_line != null) {
                    ($model_line | str replace "Model name:" "" | str trim)
                } else {
                    "unknown"
                }

                let cores_line = ($lscpu_output | where {|line| $line | str starts-with "CPU(s):"} | first)
                let threads = if ($cores_line != null) {
                    ($cores_line | str replace "CPU(s):" "" | str trim | into int)
                } else {
                    0
                }

                let freq_line = ($lscpu_output | where {|line| $line | str starts-with "CPU max MHz:"} | first)
                let max_freq = if ($freq_line != null) {
                    ($freq_line | str replace "CPU max MHz:" "" | str trim | into float | math round)
                } else {
                    0
                }

                { model: $model, threads: $threads, max_freq_mhz: $max_freq }
            } catch {
                { model: "unknown", threads: 0, max_freq_mhz: 0 }
            }
        } else {
            { model: "unknown", threads: 0, max_freq_mhz: 0 }
        }
    })

    let cores = if $lscpu_info.threads == 0 {
        (do { try { ^cat /proc/cpuinfo | ^grep "processor" | lines | length } catch { 0 } })
    } else {
        $lscpu_info.threads
    }

    let cpu_info = {
        architecture: $arch
        model: $lscpu_info.model
        cores: $cores
        threads: $lscpu_info.threads
        max_freq_mhz: $lscpu_info.max_freq_mhz
        features: []
    }

    log-success $"CPU: ($cpu_info.model) ($cpu_info.architecture)"

    return $cpu_info
}

# Check for Tensor Core availability and generation
#
# Verifies that GPU has Tensor Cores and identifies the generation
#
# Returns: record - Tensor Core information
#
# Example:
#   let tensor_cores = (dgx-check-tensor-cores)
#   if $tensor_cores.available {
#       print $"Tensor Cores: Gen ($tensor_cores.generation)"
#   }
export def dgx-check-tensor-cores [] {
    if not (command-exists "nvidia-smi") {
        log-error "nvidia-smi not found"
        return {
            available: false
            generation: 0
            compute_capability: "unknown"
            supports_fp16: false
            supports_int8: false
            supports_fp8: false
        }
    }

    let result = (do {
        try {
            # Get compute capability
            let compute_cap = (
                nvidia-smi --query-gpu=compute_cap
                --format=csv,noheader
                | str trim
            )

            # Parse compute capability to determine Tensor Core generation
            let major_version = ($compute_cap | split row "." | first | into int)

            if $major_version >= 7 {
                let generation = if $major_version >= 10 {
                    5
                } else if $major_version >= 9 {
                    4
                } else if $major_version >= 8 {
                    3
                } else {
                    1
                }

                let supports_fp8 = ($major_version >= 9)

                log-success $"Tensor Cores: Generation ($generation) (Compute ($compute_cap))"
                log-info $"Supports: FP16=true, INT8=true, FP8=($supports_fp8)"

                {
                    available: true
                    generation: $generation
                    compute_capability: $compute_cap
                    supports_fp16: true
                    supports_int8: true
                    supports_fp8: $supports_fp8
                }
            } else {
                log-warning $"GPU compute capability ($compute_cap) does not support Tensor Cores (requires 7.0+)"

                {
                    available: false
                    generation: 0
                    compute_capability: $compute_cap
                    supports_fp16: false
                    supports_int8: false
                    supports_fp8: false
                }
            }
        } catch {|err|
            log-error $"Failed to check Tensor Cores: ($err.msg)"
            {
                available: false
                generation: 0
                compute_capability: "unknown"
                supports_fp16: false
                supports_int8: false
                supports_fp8: false
            }
        }
    })

    return $result
}

# Get CUDA version information
#
# Returns CUDA driver and runtime versions
#
# Returns: record - CUDA version details
#
# Example:
#   let cuda = (dgx-get-cuda-version)
#   print $"CUDA Driver: ($cuda.driver_version)"
export def dgx-get-cuda-version [] {
    # Get driver version from nvidia-smi
    let driver_version = if (command-exists "nvidia-smi") {
        (do {
            try {
                nvidia-smi --query-gpu=driver_version
                --format=csv,noheader
                | lines
                | first
                | str trim
            } catch {
                log-warning "Could not get CUDA driver version"
                "unknown"
            }
        })
    } else {
        "unknown"
    }

    # Get runtime version from nvcc
    let runtime_info = if (command-exists "nvcc") {
        (do {
            try {
                let nvcc_output = (^nvcc --version | complete)
                let version_line = ($nvcc_output.stdout | lines | where {|line| $line | str contains "release"} | first)

                if ($version_line != null) {
                    let runtime_version = ($version_line | parse "release {version}," | get version.0)
                    { runtime_version: $runtime_version, nvcc_available: true }
                } else {
                    { runtime_version: "unknown", nvcc_available: false }
                }
            } catch {
                log-info "nvcc not available (CUDA toolkit not installed)"
                { runtime_version: "unknown", nvcc_available: false }
            }
        })
    } else {
        { runtime_version: "unknown", nvcc_available: false }
    }

    return {
        driver_version: $driver_version
        runtime_version: $runtime_info.runtime_version
        nvcc_available: $runtime_info.nvcc_available
    }
}
