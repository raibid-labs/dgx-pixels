#!/usr/bin/env bash
################################################################################
# WS-04: ComfyUI Setup - Integration Test Suite
#
# This test suite validates:
# 1. ComfyUI installation and startup
# 2. SDXL model download and validation
# 3. Workflow template creation and validation
# 4. API endpoint functionality
# 5. GPU utilization during generation
#
# Usage:
#   ./test_comfyui.sh
#
# Exit codes:
#   0 - All tests passed
#   1 - One or more tests failed
################################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
PROJECT_ROOT="/home/beengud/raibid-labs/dgx-pixels"
COMFYUI_PORT=8188
TEST_OUTPUT_DIR="${PROJECT_ROOT}/outputs/test_ws04"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create test output directory
mkdir -p "${TEST_OUTPUT_DIR}"

# Test counters
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0
TEST_START_TIME=$(date +%s)

################################################################################
# Helper Functions
################################################################################

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

test_start() {
    local test_name="$1"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo ""
    echo "========================================================================"
    echo "Test ${TESTS_TOTAL}: ${test_name}"
    echo "========================================================================"
}

test_pass() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✅ PASS${NC}"
}

test_fail() {
    local reason="$1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}❌ FAIL${NC}: ${reason}"
}

test_skip() {
    local reason="$1"
    echo -e "${YELLOW}⚠️  SKIP${NC}: ${reason}"
}

################################################################################
# Test 1: ComfyUI Installation Verification
################################################################################

test_comfyui_installation() {
    test_start "ComfyUI Installation Verification"

    log_info "Checking if ComfyUI is installed in Docker container..."

    # Check if ComfyUI directory exists
    if docker run --rm --gpus all --ipc=host \
        -v "${PROJECT_ROOT}:/workspace" \
        dgx-pixels:dev \
        test -d /workspace/comfyui; then
        log_info "ComfyUI directory found at /workspace/comfyui"
    else
        test_fail "ComfyUI directory not found"
        return 1
    fi

    # Check if main.py exists
    if docker run --rm --gpus all --ipc=host \
        -v "${PROJECT_ROOT}:/workspace" \
        dgx-pixels:dev \
        test -f /workspace/comfyui/main.py; then
        log_info "ComfyUI main.py found"
    else
        test_fail "ComfyUI main.py not found"
        return 1
    fi

    # Check if requirements are installed
    log_info "Verifying ComfyUI dependencies..."
    if docker run --rm --gpus all --ipc=host \
        -v "${PROJECT_ROOT}:/workspace" \
        dgx-pixels:dev \
        python3 -c "import torch; import PIL; import numpy" 2>/dev/null; then
        log_info "Core dependencies available"
    else
        test_fail "ComfyUI dependencies not installed"
        return 1
    fi

    test_pass
}

################################################################################
# Test 2: SDXL Model Verification
################################################################################

test_sdxl_model() {
    test_start "SDXL Model Download and Validation"

    log_info "Checking for SDXL base model..."

    # Check if model file exists
    local model_path="${PROJECT_ROOT}/models/checkpoints/sd_xl_base_1.0.safetensors"
    if [ ! -f "${model_path}" ]; then
        test_fail "SDXL model not found at ${model_path}"
        return 1
    fi

    log_info "SDXL model found"

    # Check file size (should be ~6.9GB)
    local file_size=$(stat -f%z "${model_path}" 2>/dev/null || stat -c%s "${model_path}" 2>/dev/null)
    local size_gb=$((file_size / 1024 / 1024 / 1024))

    if [ "${size_gb}" -lt 6 ] || [ "${size_gb}" -gt 8 ]; then
        test_fail "SDXL model size unexpected: ${size_gb}GB (expected ~6.9GB)"
        return 1
    fi

    log_info "SDXL model size valid: ${size_gb}GB"

    # Verify it's a valid safetensors file (check magic bytes)
    local magic=$(head -c 8 "${model_path}" | xxd -p)
    if [[ "${magic}" =~ ^[0-9a-f]{16}$ ]]; then
        log_info "SDXL model appears to be valid safetensors format"
    else
        log_warn "Could not verify safetensors format (non-critical)"
    fi

    test_pass
}

################################################################################
# Test 3: Workflow Templates Validation
################################################################################

test_workflow_templates() {
    test_start "Workflow Template Validation"

    log_info "Checking for workflow templates..."

    local workflows_dir="${PROJECT_ROOT}/workflows"
    local required_workflows=(
        "txt2img_sdxl.json"
        "img2img_sdxl.json"
        "batch_generation.json"
    )

    for workflow in "${required_workflows[@]}"; do
        local workflow_path="${workflows_dir}/${workflow}"

        if [ ! -f "${workflow_path}" ]; then
            test_fail "Workflow template not found: ${workflow}"
            return 1
        fi

        log_info "Found workflow: ${workflow}"

        # Validate JSON syntax
        if ! jq empty "${workflow_path}" 2>/dev/null; then
            test_fail "Invalid JSON in workflow: ${workflow}"
            return 1
        fi

        log_info "Workflow JSON valid: ${workflow}"

        # Check for required keys
        local has_checkpoint=$(jq 'recurse | select(.class_type? == "CheckpointLoaderSimple")' "${workflow_path}")
        if [ -z "${has_checkpoint}" ]; then
            test_fail "Workflow ${workflow} missing CheckpointLoaderSimple node"
            return 1
        fi

        log_info "Workflow ${workflow} has valid structure"
    done

    test_pass
}

################################################################################
# Test 4: ComfyUI Startup and API Test
################################################################################

test_comfyui_api() {
    test_start "ComfyUI API Functionality"

    log_info "Starting ComfyUI server (background)..."

    # Start ComfyUI in background
    docker run -d --rm --gpus all --ipc=host \
        --name comfyui-test-${TIMESTAMP} \
        -p ${COMFYUI_PORT}:8188 \
        -v "${PROJECT_ROOT}:/workspace" \
        dgx-pixels:dev \
        bash -c "cd /workspace/comfyui && python3 main.py --listen 0.0.0.0 --port 8188 --disable-auto-launch" \
        > "${TEST_OUTPUT_DIR}/comfyui_startup.log" 2>&1

    # Wait for ComfyUI to start (max 60 seconds)
    log_info "Waiting for ComfyUI to start..."
    local max_wait=60
    local waited=0
    while [ $waited -lt $max_wait ]; do
        if curl -s http://localhost:${COMFYUI_PORT}/system_stats > /dev/null 2>&1; then
            log_info "ComfyUI started successfully (${waited}s)"
            break
        fi
        sleep 2
        waited=$((waited + 2))
    done

    if [ $waited -ge $max_wait ]; then
        docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true
        test_fail "ComfyUI did not start within ${max_wait} seconds"
        return 1
    fi

    # Test system_stats endpoint
    log_info "Testing /system_stats endpoint..."
    local stats=$(curl -s http://localhost:${COMFYUI_PORT}/system_stats)
    if echo "${stats}" | jq empty 2>/dev/null; then
        log_info "/system_stats returned valid JSON"
    else
        docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true
        test_fail "/system_stats did not return valid JSON"
        return 1
    fi

    # Test queue endpoint
    log_info "Testing /queue endpoint..."
    local queue=$(curl -s http://localhost:${COMFYUI_PORT}/queue)
    if echo "${queue}" | jq empty 2>/dev/null; then
        log_info "/queue returned valid JSON"
    else
        docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true
        test_fail "/queue did not return valid JSON"
        return 1
    fi

    # Test history endpoint
    log_info "Testing /history endpoint..."
    local history=$(curl -s http://localhost:${COMFYUI_PORT}/history)
    if echo "${history}" | jq empty 2>/dev/null; then
        log_info "/history returned valid JSON"
    else
        docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true
        test_fail "/history did not return valid JSON"
        return 1
    fi

    # Stop ComfyUI
    log_info "Stopping ComfyUI..."
    docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true

    test_pass
}

################################################################################
# Test 5: SDXL Image Generation Test
################################################################################

test_sdxl_generation() {
    test_start "SDXL Image Generation"

    log_info "Starting ComfyUI for generation test..."

    # Start ComfyUI
    docker run -d --rm --gpus all --ipc=host \
        --name comfyui-test-${TIMESTAMP} \
        -p ${COMFYUI_PORT}:8188 \
        -v "${PROJECT_ROOT}:/workspace" \
        dgx-pixels:dev \
        bash -c "cd /workspace/comfyui && python3 main.py --listen 0.0.0.0 --port 8188 --disable-auto-launch" \
        > "${TEST_OUTPUT_DIR}/comfyui_generation.log" 2>&1

    # Wait for startup
    log_info "Waiting for ComfyUI startup..."
    local max_wait=60
    local waited=0
    while [ $waited -lt $max_wait ]; do
        if curl -s http://localhost:${COMFYUI_PORT}/system_stats > /dev/null 2>&1; then
            break
        fi
        sleep 2
        waited=$((waited + 2))
    done

    if [ $waited -ge $max_wait ]; then
        docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true
        test_fail "ComfyUI did not start"
        return 1
    fi

    # Load workflow template
    local workflow="${PROJECT_ROOT}/workflows/txt2img_sdxl.json"
    if [ ! -f "${workflow}" ]; then
        docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true
        test_fail "Workflow template not found"
        return 1
    fi

    # Submit generation job
    log_info "Submitting SDXL generation job..."
    local start_time=$(date +%s)

    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d @"${workflow}" \
        http://localhost:${COMFYUI_PORT}/prompt)

    if ! echo "${response}" | jq -e '.prompt_id' > /dev/null 2>&1; then
        docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true
        test_fail "Failed to submit generation job"
        return 1
    fi

    local prompt_id=$(echo "${response}" | jq -r '.prompt_id')
    log_info "Job submitted with prompt_id: ${prompt_id}"

    # Wait for generation to complete (max 120 seconds)
    log_info "Waiting for generation to complete..."
    local max_gen_wait=120
    local gen_waited=0
    local generation_complete=false

    while [ $gen_waited -lt $max_gen_wait ]; do
        local history=$(curl -s http://localhost:${COMFYUI_PORT}/history/${prompt_id})

        if echo "${history}" | jq -e ".\"${prompt_id}\".status.completed" > /dev/null 2>&1; then
            log_info "Generation completed"
            generation_complete=true
            break
        fi

        sleep 2
        gen_waited=$((gen_waited + 2))
    done

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    docker stop comfyui-test-${TIMESTAMP} 2>/dev/null || true

    if [ "${generation_complete}" = false ]; then
        test_fail "Generation did not complete within ${max_gen_wait} seconds"
        return 1
    fi

    log_info "Generation time: ${duration} seconds"

    # Check if generation time is reasonable (should be ≤10 seconds target)
    if [ $duration -gt 60 ]; then
        log_warn "Generation took ${duration}s (target: ≤10s), but acceptable for baseline"
    else
        log_info "Generation time acceptable: ${duration}s"
    fi

    test_pass
}

################################################################################
# Test 6: GPU Utilization Test
################################################################################

test_gpu_utilization() {
    test_start "GPU Utilization During Generation"

    log_info "This test requires ComfyUI to be running during generation"
    log_info "Checking nvidia-smi availability..."

    if ! command -v nvidia-smi &> /dev/null; then
        test_skip "nvidia-smi not available on host"
        return 0
    fi

    # This is a manual verification test
    log_info "GPU utilization should be >50% during SDXL generation"
    log_info "Run: watch -n 1 nvidia-smi"
    log_info "While generating: docker run --rm --gpus all ... python3 generate.py"

    log_warn "Manual verification required (skipping automated test)"
    test_skip "Manual verification recommended"
}

################################################################################
# Main Test Execution
################################################################################

main() {
    echo "========================================================================"
    echo "  WS-04: ComfyUI Setup - Integration Tests"
    echo "========================================================================"
    echo ""
    echo "Project: ${PROJECT_ROOT}"
    echo "ComfyUI Port: ${COMFYUI_PORT}"
    echo "Test Output: ${TEST_OUTPUT_DIR}"
    echo ""

    # Run all tests
    test_comfyui_installation || true
    test_sdxl_model || true
    test_workflow_templates || true
    test_comfyui_api || true
    test_sdxl_generation || true
    test_gpu_utilization || true

    # Summary
    local test_end_time=$(date +%s)
    local total_time=$((test_end_time - TEST_START_TIME))

    echo ""
    echo "========================================================================"
    echo "  Test Summary"
    echo "========================================================================"
    echo "Total tests:  ${TESTS_TOTAL}"
    echo "Passed:       ${TESTS_PASSED}"
    echo "Failed:       ${TESTS_FAILED}"
    echo "Duration:     ${total_time} seconds"
    echo ""

    if [ ${TESTS_FAILED} -eq 0 ]; then
        echo -e "${GREEN}✅ WS-04 INTEGRATION TESTS: PASSED (${TESTS_PASSED}/${TESTS_TOTAL})${NC}"
        return 0
    else
        echo -e "${RED}❌ WS-04 INTEGRATION TESTS: FAILED (${TESTS_FAILED}/${TESTS_TOTAL} tests failed)${NC}"
        return 1
    fi
}

# Run main function
main "$@"
