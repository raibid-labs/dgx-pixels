#!/usr/bin/env nu
################################################################################
# Integration Test: WS-01 Full Hardware Scan
# Description: End-to-end test of complete hardware verification
################################################################################

use ../../../scripts/nu/config.nu [COLORS, log-success, log-error, log-info]

# Run all integration tests
export def run-all-tests [] {
    log-info "Running WS-01 Integration Test Suite..."
    print ""

    # Run all test functions and collect results
    let results = [
        (test-full-hardware-scan)
        (test-baseline-generation)
        (test-idempotency)
        (test-baseline-schema)
    ]

    # Summary
    print ""
    log-info "========================================="
    log-info "Integration Test Summary"
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
        log-success $"All integration tests passed! ($passed)/($total)"
        exit 0
    } else {
        log-error $"($failed) integration tests failed out of ($total)"
        exit 1
    }
}

# Test full hardware scan execution
export def test-full-hardware-scan [] {
    try {
        # Check if hardware verification script exists
        let script_path = "repro/hardware_verification.nu"

        if not ($script_path | path exists) {
            return {
                name: "test_full_hardware_scan"
                status: "FAILED"
                error: $"Verification script not found at ($script_path)"
            }
        }

        # Execute the verification script
        let result = (do { nu $script_path } | complete)

        if $result.exit_code != 0 {
            return {
                name: "test_full_hardware_scan"
                status: "FAILED"
                error: $"Script exited with code ($result.exit_code): ($result.stderr)"
            }
        }

        # Verify output is valid JSON
        let baseline = (try { $result.stdout | from json } catch { null })

        if $baseline == null {
            return {
                name: "test_full_hardware_scan"
                status: "FAILED"
                error: "Script output is not valid JSON"
            }
        }

        # Verify required top-level fields
        let required_fields = ["version", "timestamp", "gpu", "cpu", "memory"]
        let missing_fields = ($required_fields | where {|field|
            not ($field in ($baseline | columns))
        })

        if ($missing_fields | is-not-empty) {
            return {
                name: "test_full_hardware_scan"
                status: "FAILED"
                error: $"Missing fields: ($missing_fields | str join ', ')"
            }
        }

        return {
            name: "test_full_hardware_scan"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_full_hardware_scan"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test baseline JSON generation and file saving
export def test-baseline-generation [] {
    try {
        let script_path = "repro/hardware_verification.nu"
        let output_path = "bench/baselines/hardware_baseline.json"

        if not ($script_path | path exists) {
            return {
                name: "test_baseline_generation"
                status: "FAILED"
                error: "Verification script not found"
            }
        }

        # Run script and save output
        let baseline_json = (do { nu $script_path } | complete)

        if $baseline_json.exit_code != 0 {
            return {
                name: "test_baseline_generation"
                status: "FAILED"
                error: "Script execution failed"
            }
        }

        # Save to file
        $baseline_json.stdout | save -f $output_path

        # Verify file was created
        if not ($output_path | path exists) {
            return {
                name: "test_baseline_generation"
                status: "FAILED"
                error: "Baseline file was not created"
            }
        }

        # Verify file contains valid JSON
        let content = (open --raw $output_path)
        let parsed = (try { $content | from json } catch { null })

        if $parsed == null {
            return {
                name: "test_baseline_generation"
                status: "FAILED"
                error: "Saved file is not valid JSON"
            }
        }

        return {
            name: "test_baseline_generation"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_baseline_generation"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test idempotency (running script multiple times produces same results)
export def test-idempotency [] {
    try {
        let script_path = "repro/hardware_verification.nu"

        if not ($script_path | path exists) {
            return {
                name: "test_idempotency"
                status: "FAILED"
                error: "Verification script not found"
            }
        }

        # Run script twice
        let result1 = (do { nu $script_path } | complete)
        let result2 = (do { nu $script_path } | complete)

        if $result1.exit_code != 0 or $result2.exit_code != 0 {
            return {
                name: "test_idempotency"
                status: "FAILED"
                error: "One or both script runs failed"
            }
        }

        # Parse both outputs
        let baseline1 = ($result1.stdout | from json)
        let baseline2 = ($result2.stdout | from json)

        # Compare key fields (timestamp will differ, so exclude it)
        # Compare GPU model
        if $baseline1.gpu.model != $baseline2.gpu.model {
            return {
                name: "test_idempotency"
                status: "FAILED"
                error: "GPU model differs between runs"
            }
        }

        # Compare CPU architecture
        if $baseline1.cpu.architecture != $baseline2.cpu.architecture {
            return {
                name: "test_idempotency"
                status: "FAILED"
                error: "CPU architecture differs between runs"
            }
        }

        # Compare memory total (should be same)
        if $baseline1.memory.total_gb != $baseline2.memory.total_gb {
            return {
                name: "test_idempotency"
                status: "FAILED"
                error: "Memory total differs between runs"
            }
        }

        return {
            name: "test_idempotency"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_idempotency"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Test baseline JSON schema validation
export def test-baseline-schema [] {
    try {
        let output_path = "bench/baselines/hardware_baseline.json"

        if not ($output_path | path exists) {
            return {
                name: "test_baseline_schema"
                status: "FAILED"
                error: "Baseline file not found (run test_baseline_generation first)"
            }
        }

        let baseline = (open $output_path)

        # Define expected schema
        let gpu_fields = ["model", "count", "memory_gb", "compute_capability", "cuda_version", "driver_version"]
        let cpu_fields = ["model", "architecture", "cores"]
        let memory_fields = ["type", "total_gb", "available_gb"]

        # Check GPU fields
        let missing_gpu = ($gpu_fields | where {|field|
            not ($field in ($baseline.gpu | columns))
        })

        if ($missing_gpu | is-not-empty) {
            return {
                name: "test_baseline_schema"
                status: "FAILED"
                error: $"Missing GPU fields: ($missing_gpu | str join ', ')"
            }
        }

        # Check CPU fields
        let missing_cpu = ($cpu_fields | where {|field|
            not ($field in ($baseline.cpu | columns))
        })

        if ($missing_cpu | is-not-empty) {
            return {
                name: "test_baseline_schema"
                status: "FAILED"
                error: $"Missing CPU fields: ($missing_cpu | str join ', ')"
            }
        }

        # Check memory fields
        let missing_memory = ($memory_fields | where {|field|
            not ($field in ($baseline.memory | columns))
        })

        if ($missing_memory | is-not-empty) {
            return {
                name: "test_baseline_schema"
                status: "FAILED"
                error: $"Missing memory fields: ($missing_memory | str join ', ')"
            }
        }

        # Validate specific values for DGX-Spark
        if $baseline.gpu.model != "GB10" {
            return {
                name: "test_baseline_schema"
                status: "FAILED"
                error: $"Expected GPU model 'GB10', got '($baseline.gpu.model)'"
            }
        }

        if $baseline.cpu.architecture != "aarch64" {
            return {
                name: "test_baseline_schema"
                status: "FAILED"
                error: $"Expected CPU architecture 'aarch64', got '($baseline.cpu.architecture)'"
            }
        }

        # DGX-Spark reports 119GB (not full 128GB due to system overhead)
        if $baseline.memory.total_gb < 100 or $baseline.memory.total_gb > 130 {
            return {
                name: "test_baseline_schema"
                status: "FAILED"
                error: $"Expected memory 100-130GB for DGX-Spark, got ($baseline.memory.total_gb)GB"
            }
        }

        return {
            name: "test_baseline_schema"
            status: "PASSED"
            error: null
        }
    } catch {|err|
        return {
            name: "test_baseline_schema"
            status: "FAILED"
            error: $err.msg
        }
    }
}

# Main entry point
def main [] {
    run-all-tests
}
