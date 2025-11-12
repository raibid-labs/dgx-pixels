#!/bin/bash
# Health check for DGX-Pixels backend worker
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "DGX-Pixels Worker Health Check"
echo "=========================================="

# Determine project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Parse arguments
REQ_ADDR="tcp://127.0.0.1:5555"
COMFYUI_URL="http://localhost:8188"
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --req-addr)
            REQ_ADDR="$2"
            shift 2
            ;;
        --comfyui-url)
            COMFYUI_URL="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check 1: ComfyUI
echo -n "ComfyUI ($COMFYUI_URL)... "
if curl -s "$COMFYUI_URL/system_stats" > /dev/null 2>&1; then
    echo -e "${GREEN}OK${NC}"

    if [ "$VERBOSE" = true ]; then
        STATS=$(curl -s "$COMFYUI_URL/system_stats")
        echo "  Stats: $STATS"
    fi
else
    echo -e "${RED}FAILED${NC}"
    EXIT_CODE=1
fi

# Check 2: Backend worker (attempt to connect)
echo -n "Backend Worker ($REQ_ADDR)... "

# Create test script to ping worker
TEST_SCRIPT="$PROJECT_ROOT/python/workers/test_ping.py"
cat > "$TEST_SCRIPT" << 'EOF'
import sys
import zmq
import msgpack

try:
    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    socket.setsockopt(zmq.RCVTIMEO, 2000)  # 2 second timeout
    socket.connect(sys.argv[1])

    # Send ping
    ping_msg = {"type": "ping"}
    socket.send(msgpack.packb(ping_msg, use_bin_type=True))

    # Wait for pong
    response = socket.recv()
    msg = msgpack.unpackb(response, raw=False)

    if msg.get("type") == "pong":
        print("OK")
        sys.exit(0)
    else:
        print(f"UNEXPECTED: {msg}")
        sys.exit(1)

except zmq.error.Again:
    print("TIMEOUT")
    sys.exit(1)
except Exception as e:
    print(f"ERROR: {e}")
    sys.exit(1)
finally:
    socket.close()
    context.term()
EOF

RESULT=$(python3 "$TEST_SCRIPT" "$REQ_ADDR" 2>&1 || true)
rm -f "$TEST_SCRIPT"

if [ "$RESULT" = "OK" ]; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}$RESULT${NC}"
    EXIT_CODE=1
fi

# Check 3: Workflow files
echo -n "Workflow files... "
WORKFLOW_DIR="$PROJECT_ROOT/workflows"
if [ -d "$WORKFLOW_DIR" ]; then
    WORKFLOW_COUNT=$(find "$WORKFLOW_DIR" -name "*.json" | wc -l)
    echo -e "${GREEN}$WORKFLOW_COUNT found${NC}"

    if [ "$VERBOSE" = true ]; then
        find "$WORKFLOW_DIR" -name "*.json" -exec basename {} \; | sed 's/^/  - /'
    fi
else
    echo -e "${YELLOW}Directory not found${NC}"
fi

# Check 4: Output directory
echo -n "Output directory... "
OUTPUT_DIR="$PROJECT_ROOT/outputs"
if [ -d "$OUTPUT_DIR" ]; then
    OUTPUT_COUNT=$(find "$OUTPUT_DIR" -name "*.png" 2>/dev/null | wc -l)
    echo -e "${GREEN}$OUTPUT_COUNT images${NC}"

    if [ "$VERBOSE" = true ]; then
        du -sh "$OUTPUT_DIR" | awk '{print "  Size: " $1}'
    fi
else
    echo -e "${YELLOW}Directory not found${NC}"
fi

# Check 5: Python dependencies
echo -n "Python dependencies... "
if python3 -c "import zmq, msgpack, requests, yaml" 2>/dev/null; then
    echo -e "${GREEN}OK${NC}"

    if [ "$VERBOSE" = true ]; then
        python3 -c "import zmq; print(f'  pyzmq: {zmq.zmq_version()}')"
        python3 -c "import msgpack; print(f'  msgpack: {msgpack.version[0]}')"
        python3 -c "import requests; print(f'  requests: {requests.__version__}')"
    fi
else
    echo -e "${RED}MISSING${NC}"
    echo "  Install with: pip install -r python/requirements-worker.txt"
    EXIT_CODE=1
fi

echo ""
echo "=========================================="
if [ "${EXIT_CODE:-0}" -eq 0 ]; then
    echo -e "${GREEN}All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}Some checks failed${NC}"
    exit 1
fi
