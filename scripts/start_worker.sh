#!/bin/bash
# Start the DGX-Pixels backend worker
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "DGX-Pixels Backend Worker Startup"
echo "=========================================="

# Determine project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${GREEN}Project root: $PROJECT_ROOT${NC}"

# Check if ComfyUI is running
echo -n "Checking ComfyUI status... "
if curl -s http://localhost:8188/system_stats > /dev/null 2>&1; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo -e "${YELLOW}Warning: ComfyUI is not running at http://localhost:8188${NC}"
    echo "Please start ComfyUI first:"
    echo "  cd /workspace/ComfyUI"
    echo "  python main.py"
    exit 1
fi

# Check Python environment
echo -n "Checking Python environment... "
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version)
    echo -e "${GREEN}$PYTHON_VERSION${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "Python 3 is required but not found"
    exit 1
fi

# Install dependencies if needed
REQUIREMENTS_FILE="$PROJECT_ROOT/python/requirements-worker.txt"
if [ -f "$REQUIREMENTS_FILE" ]; then
    echo -n "Checking dependencies... "

    # Check if key packages are installed
    if python3 -c "import zmq, msgpack, requests" 2>/dev/null; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${YELLOW}Installing dependencies${NC}"
        pip install -r "$REQUIREMENTS_FILE"
    fi
fi

# Check if output directory exists
OUTPUT_DIR="$PROJECT_ROOT/outputs"
if [ ! -d "$OUTPUT_DIR" ]; then
    echo "Creating output directory: $OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR"
fi

# Parse command line arguments
REQ_ADDR="tcp://127.0.0.1:5555"
PUB_ADDR="tcp://127.0.0.1:5556"
COMFYUI_URL="http://localhost:8188"

while [[ $# -gt 0 ]]; do
    case $1 in
        --req-addr)
            REQ_ADDR="$2"
            shift 2
            ;;
        --pub-addr)
            PUB_ADDR="$2"
            shift 2
            ;;
        --comfyui-url)
            COMFYUI_URL="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--req-addr ADDR] [--pub-addr ADDR] [--comfyui-url URL]"
            exit 1
            ;;
    esac
done

# Display configuration
echo ""
echo "Configuration:"
echo "  REQ-REP endpoint: $REQ_ADDR"
echo "  PUB-SUB endpoint: $PUB_ADDR"
echo "  ComfyUI URL: $COMFYUI_URL"
echo "  Workflow dir: $PROJECT_ROOT/workflows"
echo "  Output dir: $OUTPUT_DIR"
echo ""

# Start the worker
echo -e "${GREEN}Starting worker...${NC}"
echo ""

cd "$PROJECT_ROOT/python/workers"
python3 generation_worker.py \
    --req-addr "$REQ_ADDR" \
    --pub-addr "$PUB_ADDR" \
    --comfyui-url "$COMFYUI_URL" \
    --workflow-dir "$PROJECT_ROOT/workflows" \
    --output-dir "$OUTPUT_DIR"
