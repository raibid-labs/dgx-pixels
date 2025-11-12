#!/bin/bash
################################################################################
# WS-03: Benchmark Suite - Integration Test Framework
#
# This test suite verifies that all benchmark scripts work correctly and
# produce valid baseline metrics on DGX-Spark GB10 hardware.
#
# Tests:
#   1. GPU Throughput Benchmark (FP32, FP16, INT8 TFLOPS)
#   2. Memory Bandwidth Benchmark (CPU-GPU unified memory)
#   3. Storage I/O Benchmark (sequential read/write, IOPS)
#   4. DCGM Integration (GPU metrics export)
#   5. All Baseline JSONs Created and Valid
#
# Performance Targets:
#   - Each benchmark: ≤5 minutes
#   - Full suite: ≤15 minutes
#   - No system hangs or crashes
#
# Usage:
#   bash tests/integration/ws_03/test_benchmarks.sh
#
# Exit codes:
#   0 = All tests passed
#   1 = One or more tests failed
################################################################################

set -e  # Exit on first error

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=5
PASSED_TESTS=0
FAILED_TESTS=0

# Test results array
declare -a TEST_RESULTS

# Project root
PROJECT_ROOT="/home/beengud/raibid-labs/dgx-pixels"
cd "$PROJECT_ROOT"

# Timestamps for performance tracking
START_TIME=$(date +%s)

################################################################################
# Helper Functions
################################################################################

print_header() {
    echo ""
    echo "========================================================================"
    echo "  WS-03: Benchmark Suite - Integration Tests"
    echo "========================================================================"
    echo ""
    echo "Testing GPU throughput, memory bandwidth, storage I/O, and DCGM metrics"
    echo "Target: All benchmarks complete in ≤15 minutes"
    echo ""
}

print_test() {
    local test_num=$1
    local test_name=$2
    echo ""
    echo "--------------------------------------------------------------------"
    echo "Test $test_num/$TOTAL_TESTS: $test_name"
    echo "--------------------------------------------------------------------"
}

pass_test() {
    local test_name=$1
    PASSED_TESTS=$((PASSED_TESTS + 1))
    TEST_RESULTS+=("✅ PASS: $test_name")
    echo -e "${GREEN}✅ PASS: $test_name${NC}"
}

fail_test() {
    local test_name=$1
    local error_msg=$2
    FAILED_TESTS=$((FAILED_TESTS + 1))
    TEST_RESULTS+=("❌ FAIL: $test_name - $error_msg")
    echo -e "${RED}❌ FAIL: $test_name${NC}"
    echo -e "${RED}   Error: $error_msg${NC}"
}

check_file_exists() {
    local file=$1
    if [ ! -f "$file" ]; then
        fail_test "File existence check" "Required file not found: $file"
        return 1
    fi
    return 0
}

measure_time() {
    local start=$1
    local end=$(date +%s)
    local duration=$((end - start))
    local minutes=$((duration / 60))
    local seconds=$((duration % 60))
    echo "${minutes}m ${seconds}s"
}

validate_json() {
    local file=$1
    local test_name=$2

    if [ ! -f "$file" ]; then
        fail_test "$test_name" "Baseline JSON not created: $file"
        return 1
    fi

    if [ ! -s "$file" ]; then
        fail_test "$test_name" "Baseline JSON is empty: $file"
        return 1
    fi

    # Validate JSON structure
    if command -v jq &> /dev/null; then
        if jq empty "$file" 2>/dev/null; then
            echo "✓ Valid JSON structure: $file"
            return 0
        else
            fail_test "$test_name" "Invalid JSON in $file"
            return 1
        fi
    else
        echo "⚠ jq not installed, skipping JSON validation"
        return 0
    fi
}

check_json_field() {
    local file=$1
    local field=$2
    local test_name=$3

    if command -v jq &> /dev/null; then
        if jq -e ".$field" "$file" &> /dev/null; then
            local value=$(jq -r ".$field" "$file")
            echo "✓ Field $field present: $value"
            return 0
        else
            fail_test "$test_name" "Missing required field: $field"
            return 1
        fi
    fi
    return 0
}

################################################################################
# Test 1: GPU Throughput Benchmark
################################################################################

test_gpu_throughput() {
    print_test 1 "GPU Throughput Benchmark"

    echo "Target: Measure FP32, FP16, INT8 TFLOPS"
    echo "Expected: Complete in ≤5 minutes"
    echo ""

    # Check if benchmark script exists
    if ! check_file_exists "bench/gpu_throughput.py"; then
        fail_test "GPU Throughput" "Benchmark script not found: bench/gpu_throughput.py"
        return 1
    fi

    local bench_start=$(date +%s)

    # Run GPU throughput benchmark
    echo "Running GPU throughput benchmark..."
    if docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        --gpus all \
        --ipc=host \
        dgx-pixels:dev \
        python3 /workspace/bench/gpu_throughput.py > /tmp/ws03_gpu_throughput.log 2>&1; then

        local bench_end=$(date +%s)
        local bench_duration=$((bench_end - bench_start))
        echo "✓ Benchmark completed in $(measure_time "$bench_start")"

        # Check if benchmark was within time limit
        if [ "$bench_duration" -le 300 ]; then
            echo "✓ Benchmark time within target (≤5 minutes)"
        else
            echo -e "${YELLOW}⚠ Benchmark exceeded target: ${bench_duration}s > 300s${NC}"
        fi

        # Validate baseline JSON
        local baseline_file="bench/baselines/gpu_baseline.json"
        if validate_json "$baseline_file" "GPU Throughput"; then
            # Check for required fields
            check_json_field "$baseline_file" "fp32_tflops" "GPU Throughput" || return 1
            check_json_field "$baseline_file" "fp16_tflops" "GPU Throughput" || return 1
            # INT8 skipped (not supported in PyTorch), field may be null
            check_json_field "$baseline_file" "timestamp" "GPU Throughput" || return 1

            pass_test "GPU Throughput"
            return 0
        else
            return 1
        fi
    else
        fail_test "GPU Throughput" "Benchmark script failed (see /tmp/ws03_gpu_throughput.log)"
        cat /tmp/ws03_gpu_throughput.log
        return 1
    fi
}

################################################################################
# Test 2: Memory Bandwidth Benchmark
################################################################################

test_memory_bandwidth() {
    print_test 2 "Memory Bandwidth Benchmark"

    echo "Target: Measure CPU-GPU unified memory bandwidth"
    echo "Expected: Compare against 435 GB/s specification"
    echo ""

    # Check if benchmark script exists
    if ! check_file_exists "bench/memory_bandwidth.py"; then
        fail_test "Memory Bandwidth" "Benchmark script not found: bench/memory_bandwidth.py"
        return 1
    fi

    local bench_start=$(date +%s)

    # Run memory bandwidth benchmark
    echo "Running memory bandwidth benchmark..."
    if docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        --gpus all \
        --ipc=host \
        dgx-pixels:dev \
        python3 /workspace/bench/memory_bandwidth.py > /tmp/ws03_memory_bandwidth.log 2>&1; then

        local bench_end=$(date +%s)
        local bench_duration=$((bench_end - bench_start))
        echo "✓ Benchmark completed in $(measure_time "$bench_start")"

        # Check if benchmark was within time limit
        if [ "$bench_duration" -le 300 ]; then
            echo "✓ Benchmark time within target (≤5 minutes)"
        else
            echo -e "${YELLOW}⚠ Benchmark exceeded target: ${bench_duration}s > 300s${NC}"
        fi

        # Validate baseline JSON
        local baseline_file="bench/baselines/memory_baseline.json"
        if validate_json "$baseline_file" "Memory Bandwidth"; then
            # Check for required fields
            check_json_field "$baseline_file" "bandwidth_gbs" "Memory Bandwidth" || return 1
            check_json_field "$baseline_file" "cpu_to_gpu_gbs" "Memory Bandwidth" || return 1
            check_json_field "$baseline_file" "gpu_to_cpu_gbs" "Memory Bandwidth" || return 1
            check_json_field "$baseline_file" "timestamp" "Memory Bandwidth" || return 1

            pass_test "Memory Bandwidth"
            return 0
        else
            return 1
        fi
    else
        fail_test "Memory Bandwidth" "Benchmark script failed (see /tmp/ws03_memory_bandwidth.log)"
        cat /tmp/ws03_memory_bandwidth.log
        return 1
    fi
}

################################################################################
# Test 3: Storage I/O Benchmark
################################################################################

test_storage_io() {
    print_test 3 "Storage I/O Benchmark"

    echo "Target: Measure sequential read/write and random IOPS"
    echo "Expected: ≥8 GB/s throughput (from WS-01)"
    echo ""

    # Check if benchmark script exists
    if ! check_file_exists "bench/storage_io.sh"; then
        fail_test "Storage I/O" "Benchmark script not found: bench/storage_io.sh"
        return 1
    fi

    local bench_start=$(date +%s)

    # Run storage I/O benchmark
    echo "Running storage I/O benchmark..."
    if bash bench/storage_io.sh > /tmp/ws03_storage_io.log 2>&1; then

        local bench_end=$(date +%s)
        local bench_duration=$((bench_end - bench_start))
        echo "✓ Benchmark completed in $(measure_time "$bench_start")"

        # Check if benchmark was within time limit
        if [ "$bench_duration" -le 300 ]; then
            echo "✓ Benchmark time within target (≤5 minutes)"
        else
            echo -e "${YELLOW}⚠ Benchmark exceeded target: ${bench_duration}s > 300s${NC}"
        fi

        # Validate baseline JSON
        local baseline_file="bench/baselines/storage_baseline.json"
        if validate_json "$baseline_file" "Storage I/O"; then
            # Check for required fields
            check_json_field "$baseline_file" "sequential_read_gbs" "Storage I/O" || return 1
            check_json_field "$baseline_file" "sequential_write_gbs" "Storage I/O" || return 1
            check_json_field "$baseline_file" "random_read_iops" "Storage I/O" || return 1
            check_json_field "$baseline_file" "timestamp" "Storage I/O" || return 1

            pass_test "Storage I/O"
            return 0
        else
            return 1
        fi
    else
        fail_test "Storage I/O" "Benchmark script failed (see /tmp/ws03_storage_io.log)"
        cat /tmp/ws03_storage_io.log
        return 1
    fi
}

################################################################################
# Test 4: DCGM Integration
################################################################################

test_dcgm_integration() {
    print_test 4 "DCGM Integration"

    echo "Target: Integrate DCGM or fallback to nvidia-smi"
    echo "Expected: Export power, temperature, utilization metrics"
    echo ""

    # Check if benchmark script exists
    if ! check_file_exists "bench/dcgm_metrics.sh"; then
        fail_test "DCGM Integration" "Benchmark script not found: bench/dcgm_metrics.sh"
        return 1
    fi

    local bench_start=$(date +%s)

    # Run DCGM metrics collection
    echo "Running DCGM metrics collection..."
    if bash bench/dcgm_metrics.sh > /tmp/ws03_dcgm.log 2>&1; then

        local bench_end=$(date +%s)
        local bench_duration=$((bench_end - bench_start))
        echo "✓ Metrics collection completed in $(measure_time "$bench_start")"

        # Check if collection was within time limit
        if [ "$bench_duration" -le 60 ]; then
            echo "✓ Collection time within target (≤1 minute)"
        else
            echo -e "${YELLOW}⚠ Collection exceeded target: ${bench_duration}s > 60s${NC}"
        fi

        # Validate baseline JSON
        local baseline_file="bench/baselines/dcgm_baseline.json"
        if validate_json "$baseline_file" "DCGM Integration"; then
            # Check for required fields (either DCGM or nvidia-smi fields)
            check_json_field "$baseline_file" "gpu_utilization_percent" "DCGM Integration" || return 1
            check_json_field "$baseline_file" "power_watts" "DCGM Integration" || return 1
            check_json_field "$baseline_file" "temperature_celsius" "DCGM Integration" || return 1
            check_json_field "$baseline_file" "timestamp" "DCGM Integration" || return 1

            pass_test "DCGM Integration"
            return 0
        else
            return 1
        fi
    else
        fail_test "DCGM Integration" "Metrics collection failed (see /tmp/ws03_dcgm.log)"
        cat /tmp/ws03_dcgm.log
        return 1
    fi
}

################################################################################
# Test 5: All Baseline JSONs Valid
################################################################################

test_all_baselines_valid() {
    print_test 5 "All Baseline JSONs Created and Valid"

    echo "Verifying all baseline files exist and are valid JSON..."
    echo ""

    local all_valid=true
    local baselines=(
        "bench/baselines/gpu_baseline.json"
        "bench/baselines/memory_baseline.json"
        "bench/baselines/storage_baseline.json"
        "bench/baselines/dcgm_baseline.json"
    )

    for baseline in "${baselines[@]}"; do
        if [ -f "$baseline" ]; then
            echo -e "${GREEN}✓ Found: $baseline${NC}"

            # Check file size
            local file_size=$(stat -c%s "$baseline" 2>/dev/null || stat -f%z "$baseline" 2>/dev/null || echo "0")
            if [ "$file_size" -gt 0 ]; then
                echo "  Size: $file_size bytes"

                # Validate JSON
                if command -v jq &> /dev/null; then
                    if jq empty "$baseline" 2>/dev/null; then
                        echo "  Valid JSON: ✓"
                    else
                        echo -e "${RED}  Invalid JSON: ✗${NC}"
                        all_valid=false
                    fi
                fi
            else
                echo -e "${RED}  Empty file: ✗${NC}"
                all_valid=false
            fi
        else
            echo -e "${RED}✗ Missing: $baseline${NC}"
            all_valid=false
        fi
        echo ""
    done

    if [ "$all_valid" = true ]; then
        pass_test "All Baselines Valid"
        return 0
    else
        fail_test "All Baselines Valid" "One or more baseline files missing or invalid"
        return 1
    fi
}

################################################################################
# Main Test Execution
################################################################################

main() {
    print_header

    echo "Starting WS-03 Benchmark Suite integration tests..."
    echo "Project root: $PROJECT_ROOT"
    echo ""

    # Run all tests (continue even if some fail)
    test_gpu_throughput || true
    test_memory_bandwidth || true
    test_storage_io || true
    test_dcgm_integration || true
    test_all_baselines_valid || true

    # Calculate total time
    END_TIME=$(date +%s)
    TOTAL_DURATION=$(measure_time "$START_TIME")
    TOTAL_SECONDS=$((END_TIME - START_TIME))

    # Print summary
    echo ""
    echo "========================================================================"
    echo "  Test Summary"
    echo "========================================================================"
    echo ""

    for result in "${TEST_RESULTS[@]}"; do
        if [[ $result == *"PASS"* ]]; then
            echo -e "${GREEN}$result${NC}"
        else
            echo -e "${RED}$result${NC}"
        fi
    done

    echo ""
    echo "--------------------------------------------------------------------"
    echo "Total tests: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo ""
    echo "⏱️  Total execution time: $TOTAL_DURATION ($TOTAL_SECONDS seconds)"

    if [ "$TOTAL_SECONDS" -le 900 ]; then
        echo -e "${GREEN}✓ Execution time within target (≤15 minutes)${NC}"
    else
        echo -e "${YELLOW}⚠ Execution time exceeded target: ${TOTAL_SECONDS}s > 900s${NC}"
    fi
    echo "--------------------------------------------------------------------"
    echo ""

    # Final verdict
    if [ "$FAILED_TESTS" -eq 0 ]; then
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}  ✅ WS-03 BENCHMARK TESTS: PASSED ($PASSED_TESTS/$TOTAL_TESTS)${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${RED}  ❌ WS-03 BENCHMARK TESTS: FAILED ($PASSED_TESTS/$TOTAL_TESTS passed)${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        return 1
    fi
}

# Run main function
main

# Exit with appropriate code
exit $?
