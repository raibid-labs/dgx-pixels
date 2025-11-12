#!/bin/bash
################################################################################
# DCGM Metrics Collection for DGX-Spark GB10
#
# Collects GPU metrics using NVIDIA DCGM (Data Center GPU Manager).
# Falls back to nvidia-smi if DCGM is not available on ARM architecture.
#
# Metrics Collected:
# - GPU Utilization (%)
# - Memory Utilization (%)
# - Power Consumption (W)
# - Temperature (°C)
# - Clock Speeds (MHz)
#
# Usage:
#   bash bench/dcgm_metrics.sh
#
# Output:
#   bench/baselines/dcgm_baseline.json
################################################################################

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Output file
OUTPUT_FILE="/home/beengud/raibid-labs/dgx-pixels/bench/baselines/dcgm_baseline.json"

echo "======================================================================"
echo "DCGM Metrics Collection - DGX-Spark GB10"
echo "======================================================================"
echo ""

################################################################################
# Helper function to sanitize nvidia-smi values
################################################################################

sanitize_value() {
    local value=$1
    local default=${2:-0}

    # Replace [N/A], N/A, or empty values with default
    if [[ "$value" == "[N/A]" ]] || [[ "$value" == "N/A" ]] || [[ -z "$value" ]]; then
        echo "$default"
    else
        echo "$value"
    fi
}

################################################################################
# Check DCGM Availability
################################################################################

USE_DCGM=false

if command -v dcgmi &> /dev/null; then
    echo "✓ DCGM found: $(dcgmi --version 2>&1 | head -1)"
    USE_DCGM=true
else
    echo "⚠ DCGM not found on system"
    echo "  Falling back to nvidia-smi"
fi

echo ""

################################################################################
# Collect Metrics with DCGM (if available)
################################################################################

if [ "$USE_DCGM" = true ]; then
    echo "Collecting metrics with DCGM..."
    echo ""

    # Start DCGM daemon if not running
    dcgmi discovery -l > /dev/null 2>&1 || {
        echo "Starting DCGM daemon..."
        nv-hostengine &
        sleep 2
    }

    # Get GPU metrics
    DCGM_OUTPUT=$(dcgmi dmon -e 155,156,203,204,251 -c 1 2>/dev/null || echo "")

    if [ -n "$DCGM_OUTPUT" ]; then
        # Parse DCGM output
        # Fields: GPU, Power (W), Temp (°C), SM Clock (MHz), Mem Clock (MHz), Util (%)
        METRICS=$(echo "$DCGM_OUTPUT" | grep -E "^[0-9]" | head -1)

        GPU_ID=$(echo "$METRICS" | awk '{print $1}')
        POWER=$(echo "$METRICS" | awk '{print $2}')
        TEMP=$(echo "$METRICS" | awk '{print $3}')
        SM_CLOCK=$(echo "$METRICS" | awk '{print $4}')
        MEM_CLOCK=$(echo "$METRICS" | awk '{print $5}')
        GPU_UTIL=$(echo "$METRICS" | awk '{print $6}')

        echo "DCGM Metrics (GPU $GPU_ID):"
        echo "  Power: $POWER W"
        echo "  Temperature: $TEMP °C"
        echo "  SM Clock: $SM_CLOCK MHz"
        echo "  Memory Clock: $MEM_CLOCK MHz"
        echo "  GPU Utilization: $GPU_UTIL %"

        # Get memory utilization separately
        MEM_UTIL=$(nvidia-smi --query-gpu=utilization.memory --format=csv,noheader,nounits -i 0 2>/dev/null || echo "0")
        MEM_UTIL=$(sanitize_value "$MEM_UTIL" "0")

        SOURCE="dcgm"
    else
        echo -e "${YELLOW}⚠ DCGM failed to collect metrics, falling back to nvidia-smi${NC}"
        USE_DCGM=false
    fi
fi

################################################################################
# Collect Metrics with nvidia-smi (fallback or if DCGM unavailable)
################################################################################

if [ "$USE_DCGM" = false ]; then
    echo "Collecting metrics with nvidia-smi..."
    echo ""

    # Check nvidia-smi availability
    if ! command -v nvidia-smi &> /dev/null; then
        echo -e "${RED}ERROR: Neither DCGM nor nvidia-smi available${NC}"
        exit 1
    fi

    # Collect metrics from nvidia-smi (with sanitization)
    GPU_UTIL=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits -i 0 2>/dev/null || echo "0")
    GPU_UTIL=$(sanitize_value "$GPU_UTIL" "0")

    MEM_UTIL=$(nvidia-smi --query-gpu=utilization.memory --format=csv,noheader,nounits -i 0 2>/dev/null || echo "0")
    MEM_UTIL=$(sanitize_value "$MEM_UTIL" "0")

    POWER=$(nvidia-smi --query-gpu=power.draw --format=csv,noheader,nounits -i 0 2>/dev/null || echo "0")
    POWER=$(sanitize_value "$POWER" "0")

    TEMP=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits -i 0 2>/dev/null || echo "0")
    TEMP=$(sanitize_value "$TEMP" "0")

    SM_CLOCK=$(nvidia-smi --query-gpu=clocks.sm --format=csv,noheader,nounits -i 0 2>/dev/null || echo "0")
    SM_CLOCK=$(sanitize_value "$SM_CLOCK" "0")

    MEM_CLOCK=$(nvidia-smi --query-gpu=clocks.mem --format=csv,noheader,nounits -i 0 2>/dev/null || echo "0")
    MEM_CLOCK=$(sanitize_value "$MEM_CLOCK" "0")

    echo "nvidia-smi Metrics:"
    echo "  GPU Utilization: $GPU_UTIL %"
    echo "  Memory Utilization: $MEM_UTIL %"
    echo "  Power: $POWER W"
    echo "  Temperature: $TEMP °C"
    echo "  SM Clock: $SM_CLOCK MHz"
    echo "  Memory Clock: $MEM_CLOCK MHz"

    SOURCE="nvidia-smi"
fi

echo ""

################################################################################
# Get GPU Information
################################################################################

GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader -i 0 2>/dev/null || echo "Unknown")
DRIVER_VERSION=$(nvidia-smi --query-gpu=driver_version --format=csv,noheader -i 0 2>/dev/null || echo "Unknown")
CUDA_VERSION=$(nvidia-smi | grep -oP "CUDA Version: \K\d+\.\d+" || echo "Unknown")

################################################################################
# Generate JSON Output
################################################################################

# Get timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Create JSON output
cat > "$OUTPUT_FILE" <<EOF
{
  "version": "1.0",
  "timestamp": "$TIMESTAMP",
  "source": "$SOURCE",
  "gpu": {
    "name": "$GPU_NAME",
    "driver_version": "$DRIVER_VERSION",
    "cuda_version": "$CUDA_VERSION"
  },
  "metrics": {
    "gpu_utilization_percent": $GPU_UTIL,
    "memory_utilization_percent": $MEM_UTIL,
    "power_watts": $POWER,
    "temperature_celsius": $TEMP,
    "sm_clock_mhz": $SM_CLOCK,
    "memory_clock_mhz": $MEM_CLOCK
  },
  "gpu_utilization_percent": $GPU_UTIL,
  "power_watts": $POWER,
  "temperature_celsius": $TEMP
}
EOF

echo "======================================================================"
echo "✓ DCGM metrics baseline saved to: $OUTPUT_FILE"
echo "======================================================================"
echo ""

# Summary
echo "Summary:"
echo "  Source: $SOURCE"
echo "  GPU: $GPU_NAME"
echo "  Driver: $DRIVER_VERSION"
echo "  CUDA: $CUDA_VERSION"
echo ""
echo "Metrics:"
echo "  GPU Utilization: $GPU_UTIL%"
echo "  Memory Utilization: $MEM_UTIL%"
echo "  Power: $POWER W"
echo "  Temperature: $TEMP °C"
echo "  SM Clock: $SM_CLOCK MHz"
echo "  Memory Clock: $MEM_CLOCK MHz"
echo ""

# Validation
if [ "$TEMP" -gt 85 ]; then
    echo -e "${YELLOW}⚠ WARNING: GPU temperature high (${TEMP}°C > 85°C)${NC}"
elif [ "$TEMP" -lt 20 ]; then
    echo -e "${YELLOW}⚠ NOTE: GPU temperature very low (${TEMP}°C), possibly idle${NC}"
else
    echo -e "${GREEN}✓ GPU temperature normal (${TEMP}°C)${NC}"
fi

if [ "$POWER" -gt 300 ]; then
    echo -e "${YELLOW}⚠ WARNING: GPU power consumption high (${POWER}W > 300W)${NC}"
else
    echo -e "${GREEN}✓ GPU power consumption normal (${POWER}W)${NC}"
fi

echo ""

# Note about DCGM on ARM
if [ "$SOURCE" = "nvidia-smi" ]; then
    echo "Note: DCGM may have limited support on ARM architecture (DGX-Spark)."
    echo "      nvidia-smi fallback provides sufficient metrics for baseline."
    echo "      For production monitoring, consider DCGM alternatives or wait for ARM support."
    echo ""
fi

exit 0
