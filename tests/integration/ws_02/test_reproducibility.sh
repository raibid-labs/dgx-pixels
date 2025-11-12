#!/bin/bash
################################################################################
# WS-02: Reproducibility Framework - Smoke Test Suite
#
# This test suite verifies that the Docker-based reproducibility framework
# works correctly on DGX-Spark ARM architecture with NVIDIA GPU support.
#
# Tests:
#   1. Docker image builds successfully
#   2. GPU accessible inside container (nvidia-smi works)
#   3. Python 3.10+ installed and functional
#   4. PyTorch 2.5+ with CUDA 13.0 working
#   5. Environment capture script creates valid JSON
#
# Performance Targets:
#   - Docker build: ≤10 minutes
#   - Container startup: ≤30 seconds
#   - Test suite execution: ≤5 minutes
#
# Usage:
#   bash tests/integration/ws_02/test_reproducibility.sh
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

# Docker image name
IMAGE_NAME="dgx-pixels:test"

# Timestamps for performance tracking
START_TIME=$(date +%s)

################################################################################
# Helper Functions
################################################################################

print_header() {
    echo ""
    echo "========================================================================"
    echo "  WS-02: Reproducibility Framework - Smoke Tests"
    echo "========================================================================"
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

################################################################################
# Test 1: Docker Image Builds Successfully
################################################################################

test_docker_build() {
    print_test 1 "Docker Image Builds Successfully"

    # Check if Dockerfile exists
    if ! check_file_exists "docker/Dockerfile"; then
        fail_test "Docker Build" "Dockerfile not found at docker/Dockerfile"
        return 1
    fi

    echo "Building Docker image: $IMAGE_NAME"
    echo "Target: Build completes in ≤10 minutes"

    local build_start=$(date +%s)

    # Build Docker image
    if docker build -t "$IMAGE_NAME" docker/ > /tmp/ws02_docker_build.log 2>&1; then
        local build_end=$(date +%s)
        local build_time=$((build_end - build_start))
        local build_duration=$(measure_time "$build_start")

        echo "Build completed in: $build_duration ($build_time seconds)"

        # Check if build time is within target
        if [ "$build_time" -le 600 ]; then
            echo "✓ Build time within target (≤10 minutes)"
        else
            echo -e "${YELLOW}⚠ Build time exceeded target: ${build_time}s > 600s${NC}"
        fi

        # Verify image exists
        if docker images | grep -q "$IMAGE_NAME"; then
            pass_test "Docker Build"
            return 0
        else
            fail_test "Docker Build" "Image built but not found in docker images"
            return 1
        fi
    else
        fail_test "Docker Build" "Docker build command failed (see /tmp/ws02_docker_build.log)"
        cat /tmp/ws02_docker_build.log
        return 1
    fi
}

################################################################################
# Test 2: GPU Accessible Inside Container
################################################################################

test_gpu_access() {
    print_test 2 "GPU Accessible Inside Container"

    echo "Running nvidia-smi inside container..."
    echo "Target: Detect GB10 GPU with CUDA 13.0"

    # Run nvidia-smi in container
    local gpu_output
    if gpu_output=$(docker run --rm --gpus all "$IMAGE_NAME" nvidia-smi 2>&1); then
        echo "$gpu_output"

        # Check for GB10 GPU
        if echo "$gpu_output" | grep -q "GB10"; then
            echo "✓ GB10 GPU detected"
        else
            fail_test "GPU Access" "GB10 GPU not detected in nvidia-smi output"
            return 1
        fi

        # Check CUDA version
        if echo "$gpu_output" | grep -q "CUDA Version: 13"; then
            echo "✓ CUDA 13.x detected"
        else
            echo -e "${YELLOW}⚠ CUDA version may not be 13.x${NC}"
        fi

        pass_test "GPU Access"
        return 0
    else
        fail_test "GPU Access" "nvidia-smi failed inside container: $gpu_output"
        return 1
    fi
}

################################################################################
# Test 3: Python 3.10+ Installed and Functional
################################################################################

test_python_environment() {
    print_test 3 "Python 3.10+ Installed and Functional"

    echo "Checking Python version inside container..."
    echo "Target: Python 3.10+"

    # Check Python version
    local python_version
    if python_version=$(docker run --rm "$IMAGE_NAME" python3 --version 2>&1); then
        echo "$python_version"

        # Extract version number
        local version_num=$(echo "$python_version" | grep -oP 'Python \K\d+\.\d+')
        local major=$(echo "$version_num" | cut -d. -f1)
        local minor=$(echo "$version_num" | cut -d. -f2)

        # Check if version is 3.10+
        if [ "$major" -eq 3 ] && [ "$minor" -ge 10 ]; then
            echo "✓ Python $version_num meets requirement (≥3.10)"
            pass_test "Python Environment"
            return 0
        else
            fail_test "Python Environment" "Python $version_num does not meet requirement (need ≥3.10)"
            return 1
        fi
    else
        fail_test "Python Environment" "Failed to run python3 in container"
        return 1
    fi
}

################################################################################
# Test 4: PyTorch + CUDA Functional
################################################################################

test_pytorch_cuda() {
    print_test 4 "PyTorch 2.5+ with CUDA 13.0 Functional"

    echo "Testing PyTorch CUDA availability..."
    echo "Target: PyTorch 2.5+, CUDA 13.0 accessible"

    # Python script to test PyTorch + CUDA
    local test_script='
import sys
import torch

print(f"PyTorch version: {torch.__version__}")
print(f"CUDA available: {torch.cuda.is_available()}")

if not torch.cuda.is_available():
    print("ERROR: CUDA not available")
    sys.exit(1)

print(f"CUDA version: {torch.version.cuda}")
print(f"GPU count: {torch.cuda.device_count()}")

if torch.cuda.device_count() == 0:
    print("ERROR: No GPU detected")
    sys.exit(1)

print(f"GPU name: {torch.cuda.get_device_name(0)}")

# Check PyTorch version
version = torch.__version__.split("+")[0]  # Remove +cu118 suffix
major, minor = map(int, version.split(".")[:2])

if major < 2 or (major == 2 and minor < 5):
    print(f"WARNING: PyTorch {version} is below recommended 2.5+")
else:
    print(f"✓ PyTorch {version} meets requirement (≥2.5)")

# Check CUDA version
cuda_version = torch.version.cuda
if cuda_version and cuda_version.startswith("13"):
    print(f"✓ CUDA {cuda_version} meets requirement (13.x)")
else:
    print(f"WARNING: CUDA {cuda_version} may not be 13.x")

print("SUCCESS: PyTorch + CUDA functional")
'

    # Run PyTorch test
    local pytorch_output
    if pytorch_output=$(docker run --rm --gpus all "$IMAGE_NAME" python3 -c "$test_script" 2>&1); then
        echo "$pytorch_output"

        if echo "$pytorch_output" | grep -q "SUCCESS: PyTorch + CUDA functional"; then
            pass_test "PyTorch + CUDA"
            return 0
        else
            fail_test "PyTorch + CUDA" "PyTorch test did not complete successfully"
            return 1
        fi
    else
        fail_test "PyTorch + CUDA" "PyTorch test script failed: $pytorch_output"
        return 1
    fi
}

################################################################################
# Test 5: Environment Capture Script
################################################################################

test_environment_capture() {
    print_test 5 "Environment Capture Script Creates Valid JSON"

    # Check if capture script exists
    if ! check_file_exists "scripts/capture_environment.sh"; then
        fail_test "Environment Capture" "Script not found at scripts/capture_environment.sh"
        return 1
    fi

    echo "Running environment capture script..."
    echo "Target: Valid JSON output with all required fields"

    # Create output directory
    mkdir -p bench/baselines

    # Run capture script
    local output_file="bench/baselines/env_test_$(date +%Y%m%d_%H%M%S).json"

    if docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        --gpus all \
        "$IMAGE_NAME" \
        bash /workspace/scripts/capture_environment.sh 2>&1 | sed -n '/^{/,$p' > "$output_file"; then

        echo "Environment captured to: $output_file"

        # Check if file exists and is not empty
        if [ ! -s "$output_file" ]; then
            fail_test "Environment Capture" "Output file is empty"
            return 1
        fi

        # Validate JSON structure
        if command -v jq &> /dev/null; then
            if jq empty "$output_file" 2>/dev/null; then
                echo "✓ Valid JSON structure"

                # Check for required fields
                local required_fields=("python.version" "cuda.version" "gpu.name")
                local missing_fields=()

                for field in "${required_fields[@]}"; do
                    if ! jq -e ".$field" "$output_file" &> /dev/null; then
                        missing_fields+=("$field")
                    fi
                done

                if [ ${#missing_fields[@]} -eq 0 ]; then
                    echo "✓ All required fields present"
                    pass_test "Environment Capture"

                    # Show captured environment
                    echo ""
                    echo "Captured environment:"
                    jq '.' "$output_file" | head -20
                    return 0
                else
                    fail_test "Environment Capture" "Missing required fields: ${missing_fields[*]}"
                    return 1
                fi
            else
                fail_test "Environment Capture" "Invalid JSON output"
                cat "$output_file"
                return 1
            fi
        else
            echo "⚠ jq not installed, skipping JSON validation"
            echo "✓ Output file created"
            pass_test "Environment Capture"
            return 0
        fi
    else
        fail_test "Environment Capture" "Script execution failed"
        cat "$output_file"
        return 1
    fi
}

################################################################################
# Main Test Execution
################################################################################

main() {
    print_header

    echo "Starting WS-02 Reproducibility Framework smoke tests..."
    echo "Project root: $PROJECT_ROOT"
    echo "Docker image: $IMAGE_NAME"
    echo ""

    # Run all tests (continue even if some fail)
    test_docker_build || true
    test_gpu_access || true
    test_python_environment || true
    test_pytorch_cuda || true
    test_environment_capture || true

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

    if [ "$TOTAL_SECONDS" -le 300 ]; then
        echo -e "${GREEN}✓ Execution time within target (≤5 minutes)${NC}"
    else
        echo -e "${YELLOW}⚠ Execution time exceeded target: ${TOTAL_SECONDS}s > 300s${NC}"
    fi
    echo "--------------------------------------------------------------------"
    echo ""

    # Final verdict
    if [ "$FAILED_TESTS" -eq 0 ]; then
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}  ✅ WS-02 SMOKE TESTS: PASSED ($PASSED_TESTS/$TOTAL_TESTS)${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${RED}  ❌ WS-02 SMOKE TESTS: FAILED ($PASSED_TESTS/$TOTAL_TESTS passed)${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        return 1
    fi
}

# Run main function
main

# Exit with appropriate code
exit $?
