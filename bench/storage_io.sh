#!/bin/bash
################################################################################
# Storage I/O Benchmark for DGX-Spark
#
# Measures storage I/O performance including:
# - Sequential read/write throughput
# - Random read IOPS
# - Block size impact
#
# Expected Performance:
# - Sequential Read/Write: ≥8 GB/s (from WS-01 baseline)
#
# Usage:
#   bash bench/storage_io.sh
#
# Output:
#   bench/baselines/storage_baseline.json
################################################################################

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Output file
OUTPUT_FILE="/home/beengud/raibid-labs/dgx-pixels/bench/baselines/storage_baseline.json"
TEST_DIR="/home/beengud/raibid-labs/dgx-pixels/bench/.io_test"
TEST_FILE="$TEST_DIR/testfile"

# Test parameters
FILE_SIZE="2G"       # 2 GB test file
BLOCK_SIZE="1M"      # 1 MB block size
IOPS_SIZE="4K"       # 4 KB for IOPS test
IOPS_RUNTIME="30"    # 30 seconds for IOPS test

echo "======================================================================"
echo "Storage I/O Benchmark - DGX-Spark"
echo "======================================================================"
echo ""

# Create test directory
mkdir -p "$TEST_DIR"

echo "Test Parameters:"
echo "  Test Directory: $TEST_DIR"
echo "  File Size: $FILE_SIZE"
echo "  Block Size (throughput): $BLOCK_SIZE"
echo "  Block Size (IOPS): $IOPS_SIZE"
echo ""

# Get filesystem info
echo "Filesystem Information:"
df -h "$TEST_DIR" | tail -1
echo ""

################################################################################
# Sequential Write Test
################################################################################

echo "Running Sequential Write Test..."
echo "  (Writing $FILE_SIZE with $BLOCK_SIZE blocks)"

# Clear caches
sync
echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null 2>&1 || true

# Sequential write with dd
WRITE_OUTPUT=$(dd if=/dev/zero of="$TEST_FILE" bs="$BLOCK_SIZE" count=2048 conv=fdatasync 2>&1)
WRITE_SPEED=$(echo "$WRITE_OUTPUT" | grep -oP '\d+\.?\d* [MG]B/s' | tail -1)
WRITE_SPEED_VALUE=$(echo "$WRITE_SPEED" | grep -oP '\d+\.?\d*')
WRITE_UNIT=$(echo "$WRITE_SPEED" | grep -oP '[MG]B/s')

# Convert to GB/s if in MB/s
if [[ "$WRITE_UNIT" == "MB/s" ]]; then
    WRITE_SPEED_GBS=$(echo "scale=2; $WRITE_SPEED_VALUE / 1024" | bc)
else
    WRITE_SPEED_GBS=$WRITE_SPEED_VALUE
fi

echo "  Result: $WRITE_SPEED_GBS GB/s"
echo ""

################################################################################
# Sequential Read Test
################################################################################

echo "Running Sequential Read Test..."
echo "  (Reading $FILE_SIZE with $BLOCK_SIZE blocks)"

# Clear caches to force disk read
sync
echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null 2>&1 || true

# Sequential read with dd
READ_OUTPUT=$(dd if="$TEST_FILE" of=/dev/null bs="$BLOCK_SIZE" 2>&1)
READ_SPEED=$(echo "$READ_OUTPUT" | grep -oP '\d+\.?\d* [MG]B/s' | tail -1)
READ_SPEED_VALUE=$(echo "$READ_SPEED" | grep -oP '\d+\.?\d*')
READ_UNIT=$(echo "$READ_SPEED" | grep -oP '[MG]B/s')

# Convert to GB/s if in MB/s
if [[ "$READ_UNIT" == "MB/s" ]]; then
    READ_SPEED_GBS=$(echo "scale=2; $READ_SPEED_VALUE / 1024" | bc)
else
    READ_SPEED_GBS=$READ_SPEED_VALUE
fi

echo "  Result: $READ_SPEED_GBS GB/s"
echo ""

################################################################################
# Random Read IOPS Test (using fio if available, fallback to dd)
################################################################################

echo "Running Random Read IOPS Test..."

if command -v fio &> /dev/null; then
    echo "  (Using fio for $IOPS_RUNTIME seconds)"

    # Run fio random read test
    FIO_OUTPUT=$(fio --name=random_read_iops \
        --filename="$TEST_FILE" \
        --ioengine=libaio \
        --direct=1 \
        --bs="$IOPS_SIZE" \
        --iodepth=64 \
        --rw=randread \
        --runtime="$IOPS_RUNTIME" \
        --time_based \
        --group_reporting \
        --output-format=normal 2>&1)

    # Extract IOPS
    RANDOM_READ_IOPS=$(echo "$FIO_OUTPUT" | grep -oP 'IOPS=\K\d+\.?\d*k?' | head -1)

    # Convert k to thousands if present
    if [[ "$RANDOM_READ_IOPS" == *"k"* ]]; then
        IOPS_VALUE=$(echo "$RANDOM_READ_IOPS" | tr -d 'k')
        RANDOM_READ_IOPS=$(echo "scale=0; $IOPS_VALUE * 1000" | bc)
    fi

    echo "  Result: $RANDOM_READ_IOPS IOPS"
else
    echo "  (fio not available, estimating from dd)"

    # Fallback: use dd for random-ish reads
    RANDOM_OUTPUT=$(dd if="$TEST_FILE" of=/dev/null bs="$IOPS_SIZE" count=10000 iflag=direct 2>&1)
    RANDOM_TIME=$(echo "$RANDOM_OUTPUT" | grep -oP '\d+\.?\d+' | tail -1)
    RANDOM_READ_IOPS=$(echo "scale=0; 10000 / $RANDOM_TIME" | bc)

    echo "  Result: ~$RANDOM_READ_IOPS IOPS (estimated)"
fi

echo ""

################################################################################
# Cleanup
################################################################################

echo "Cleaning up test files..."
rm -f "$TEST_FILE"
rmdir "$TEST_DIR" 2>/dev/null || true
echo ""

################################################################################
# Generate JSON Output
################################################################################

# Get timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Get filesystem info for JSON
FS_INFO=$(df -h "$TEST_DIR" 2>/dev/null || df -h / | tail -1)
FS_MOUNT=$(echo "$FS_INFO" | awk '{print $6}')
FS_SIZE=$(echo "$FS_INFO" | awk '{print $2}')
FS_USED=$(echo "$FS_INFO" | awk '{print $3}')
FS_AVAIL=$(echo "$FS_INFO" | awk '{print $4}')

# Create JSON output
cat > "$OUTPUT_FILE" <<EOF
{
  "version": "1.0",
  "timestamp": "$TIMESTAMP",
  "filesystem": {
    "mount": "$FS_MOUNT",
    "size": "$FS_SIZE",
    "used": "$FS_USED",
    "available": "$FS_AVAIL"
  },
  "sequential_read_gbs": $READ_SPEED_GBS,
  "sequential_write_gbs": $WRITE_SPEED_GBS,
  "random_read_iops": $RANDOM_READ_IOPS,
  "benchmark_params": {
    "file_size": "$FILE_SIZE",
    "block_size_throughput": "$BLOCK_SIZE",
    "block_size_iops": "$IOPS_SIZE",
    "iops_runtime_seconds": $IOPS_RUNTIME
  }
}
EOF

echo "======================================================================"
echo "✓ Storage I/O baseline saved to: $OUTPUT_FILE"
echo "======================================================================"
echo ""

# Summary
echo "Summary:"
echo "  Sequential Read:  $READ_SPEED_GBS GB/s"
echo "  Sequential Write: $WRITE_SPEED_GBS GB/s"
echo "  Random Read IOPS: $RANDOM_READ_IOPS IOPS"
echo ""

# Validation
TARGET_THROUGHPUT=8.0
if (( $(echo "$READ_SPEED_GBS < $TARGET_THROUGHPUT" | bc -l) )); then
    echo -e "${YELLOW}⚠ WARNING: Read throughput below target ($READ_SPEED_GBS < $TARGET_THROUGHPUT GB/s)${NC}"
else
    echo -e "${GREEN}✓ Read throughput meets target (≥$TARGET_THROUGHPUT GB/s)${NC}"
fi

if (( $(echo "$WRITE_SPEED_GBS < $TARGET_THROUGHPUT" | bc -l) )); then
    echo -e "${YELLOW}⚠ WARNING: Write throughput below target ($WRITE_SPEED_GBS < $TARGET_THROUGHPUT GB/s)${NC}"
else
    echo -e "${GREEN}✓ Write throughput meets target (≥$TARGET_THROUGHPUT GB/s)${NC}"
fi

echo ""

exit 0
