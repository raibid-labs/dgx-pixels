#!/usr/bin/env nu
################################################################################
# Test Suite: WS-01 Hardware Detection
# Description: Unit tests for hardware verification functions
# Coverage Target: >= 80%
################################################################################

use ../../../scripts/nu/config.nu [COLORS, log-success, log-error, log-info]
use ../../../scripts/nu/modules/dgx.nu *

# Test result tracking
export def run-all-tests [] {
    log-info "Running WS-01 Hardware Detection Test Suite..."
    print ""

    # Run all test functions and collect results
    let results = [
        (test-gpu-detection)
        (test-cpu-detection)
        (test-memory-detection)
        (test-storage-detection)
        (test-network-detection)
        (test-json-output)
        (test-error-handling)
    ]

    # Summary
    print ""
    log-info "========================================="
    log-info "Test Summary"
    log-info "========================================="

    let passed = ($results | where status == "PASSED" | length)
    let failed = ($results | where status == "FAILED" | length)
    let total = ($results | length)

    $results | each {|test|
        if $test.status == "PASSED" {
            print $"($COLORS.success)✅ ($test.name): PASSED($COLORS.reset)"
        } else {
            print $"($COLORS.error)❌ ($test.name): FAILED - ($test.error)($COLORS.reset)"
        }
    }

    print ""
    if $failed == 0 {
        log-success $"All tests passed! ($passed)/($total)"
        exit 0
    } else {
        log-error $"($failed) tests failed out of ($total)"
        exit 1
    }
}

# Test GPU detection functionality
export def test-gpu-detection [] {
    try {
        # Call dgx-gpu-stats to verify GPU detection
        let stats = (dgx-gpu-stats)

        # Verify we got results
        if ($stats | is-empty) {
            return {
                name: "test_gpu_detection"
                status: "FAILED"
                error: "No GPU stats returned"
            }
        }

        # Verify required fields exist
        let first_gpu = ($stats | first)
        let required_fields = ["gpu_id", "name", "memory_total_mb", "temp_c"]

        let missing_fields = ($required_fields | where {|field|
            not ($field in ($first_gpu | columns))
        })

        if ($missing_fields | is-not-empty) {
            return {
                name: "test_gpu_detection"
                status: "FAILED"
                error: $"Missing fields: ($missing_fields | str join ', ')"
            }
        }

        # Verify GPU name contains expected keywords for DGX-Spark
        let gpu_name = ($first_gpu.name | str downcase)
        if not (($gpu_name | str contains "gb") or ($gpu_name | str contains "grace") or ($gpu_name | str contains "blackwell")) {
            return {
                name: "test_gpu_detection"
                status: "FAILED"
                error: $"GPU name '($first_gpu.name)' doesn't match expected GB10/Grace/Blackwell"
            }
        }

        return {
            name: "test_gpu_detection"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_gpu_detection"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test CPU detection functionality
export def test-cpu-detection [] {
    try {
        # Call dgx-get-cpu-info to verify CPU detection
        let cpu_info = (dgx-get-cpu-info)

        # Verify required fields
        let required_fields = ["architecture", "model", "cores"]
        let missing_fields = ($required_fields | where {|field|
            not ($field in ($cpu_info | columns))
        })

        if ($missing_fields | is-not-empty) {
            return {
                name: "test_cpu_detection"
                status: "FAILED"
                error: $"Missing fields: ($missing_fields | str join ', ')"
            }
        }

        # Verify architecture is ARM (aarch64)
        if not ($cpu_info.architecture | str starts-with "aarch64") {
            return {
                name: "test_cpu_detection"
                status: "FAILED"
                error: $"Expected aarch64 architecture, got ($cpu_info.architecture)"
            }
        }

        # Verify core count is reasonable (should be 20 for DGX-Spark)
        if $cpu_info.cores < 1 or $cpu_info.cores > 256 {
            return {
                name: "test_cpu_detection"
                status: "FAILED"
                error: $"Unexpected core count: ($cpu_info.cores)"
            }
        }

        return {
            name: "test_cpu_detection"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_cpu_detection"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test memory detection functionality
export def test-memory-detection [] {
    try {
        # Get memory info using free command
        let mem_output = (^free -g | complete)

        if $mem_output.exit_code != 0 {
            return {
                name: "test_memory_detection"
                status: "FAILED"
                error: "free command failed"
            }
        }

        # Parse memory info
        let mem_lines = ($mem_output.stdout | lines)
        let mem_line = ($mem_lines | where {|line| $line | str starts-with "Mem:"} | first)

        if ($mem_line | is-empty) {
            return {
                name: "test_memory_detection"
                status: "FAILED"
                error: "Could not parse memory line"
            }
        }

        # Verify we can extract total memory
        let mem_parts = ($mem_line | split row --regex '\s+')
        let total_gb = ($mem_parts | get 1 | into int)

        # DGX-Spark should have ~128GB (119-128 GB available)
        if $total_gb < 100 or $total_gb > 200 {
            return {
                name: "test_memory_detection"
                status: "FAILED"
                error: $"Unexpected total memory: ($total_gb)GB (expected ~128GB)"
            }
        }

        return {
            name: "test_memory_detection"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_memory_detection"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test storage detection functionality
export def test-storage-detection [] {
    try {
        # Get storage info using df
        let df_output = (^df -BG / | complete)

        if $df_output.exit_code != 0 {
            return {
                name: "test_storage_detection"
                status: "FAILED"
                error: "df command failed"
            }
        }

        # Parse df output
        let df_lines = ($df_output.stdout | lines | skip 1)

        if ($df_lines | is-empty) {
            return {
                name: "test_storage_detection"
                status: "FAILED"
                error: "No storage info returned"
            }
        }

        let root_line = ($df_lines | first)
        let parts = ($root_line | split row --regex '\s+')

        # Verify we can extract size and available space
        if ($parts | length) < 4 {
            return {
                name: "test_storage_detection"
                status: "FAILED"
                error: "Could not parse df output"
            }
        }

        return {
            name: "test_storage_detection"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_storage_detection"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test network detection functionality
export def test-network-detection [] {
    try {
        # Get network interfaces using ip link
        let ip_output = (^ip -o link show | complete)

        if $ip_output.exit_code != 0 {
            return {
                name: "test_network_detection"
                status: "FAILED"
                error: "ip link command failed"
            }
        }

        # Parse network interfaces
        let interfaces = ($ip_output.stdout | lines)

        if ($interfaces | is-empty) {
            return {
                name: "test_network_detection"
                status: "FAILED"
                error: "No network interfaces found"
            }
        }

        # Verify we can detect at least one interface
        if ($interfaces | length) < 1 {
            return {
                name: "test_network_detection"
                status: "FAILED"
                error: "Expected at least one network interface"
            }
        }

        return {
            name: "test_network_detection"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_network_detection"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test JSON output formatting
export def test-json-output [] {
    try {
        # Create a sample hardware baseline structure
        let baseline = {
            version: "1.0"
            timestamp: (date now | format date "%Y-%m-%dT%H:%M:%SZ")
            gpu: {
                model: "GB10"
                memory_gb: 128
            }
            cpu: {
                architecture: "aarch64"
                cores: 20
            }
        }

        # Convert to JSON and verify it's valid
        let json_str = ($baseline | to json)

        # Try to parse it back
        let parsed = ($json_str | from json)

        # Verify structure is preserved
        if not ("gpu" in ($parsed | columns)) {
            return {
                name: "test_json_output"
                status: "FAILED"
                error: "JSON structure not preserved"
            }
        }

        return {
            name: "test_json_output"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_json_output"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test error handling when hardware is missing/unavailable
export def test-error-handling [] {
    try {
        # Test that validation handles missing nvidia-smi gracefully
        # We can't actually remove nvidia-smi, but we can verify the
        # validation function returns a proper structure

        let validation = (dgx-validate-hardware)

        # Verify the result has required fields
        let required_fields = ["is_dgx_spark", "has_nvidia_gpu", "memory_gb"]
        let missing_fields = ($required_fields | where {|field|
            not ($field in ($validation | columns))
        })

        if ($missing_fields | is-not-empty) {
            return {
                name: "test_error_handling"
                status: "FAILED"
                error: $"Validation missing fields: ($missing_fields | str join ', ')"
            }
        }

        # Verify boolean fields are actually boolean
        if not ($validation.is_dgx_spark in [true, false]) {
            return {
                name: "test_error_handling"
                status: "FAILED"
                error: "is_dgx_spark is not a boolean"
            }
        }

        return {
            name: "test_error_handling"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_error_handling"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Main entry point
def main [] {
    run-all-tests
}
