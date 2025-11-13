#!/usr/bin/env bash
#
# Manifest Generation Script for DGX-Pixels
#
# Generates or updates asset manifest for Bevy projects.
#
# Usage:
#   ./scripts/generate_manifest.sh <bevy_project> [options]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default options
FORMAT="json"
OUTPUT_FILE=""

print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

usage() {
    cat << EOF
Manifest Generation Script for DGX-Pixels

USAGE:
    $0 <bevy_project> [options]

ARGUMENTS:
    bevy_project           Path to Bevy game project root

OPTIONS:
    --format <fmt>         Output format: json (default) or toml
    --output <file>        Output file path (default: assets/sprites_manifest.json)
    -h, --help             Show this help message

EXAMPLES:
    # Generate JSON manifest
    $0 ../my-game/

    # Generate TOML manifest
    $0 ../my-game/ --format toml --output assets/sprites.toml

    # Update existing manifest
    $0 examples/bevy_integration/

EOF
}

# Parse arguments
if [ $# -lt 1 ]; then
    usage
    exit 1
fi

BEVY_PROJECT="$1"
shift

# Parse options
while [[ $# -gt 0 ]]; do
    case $1 in
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_color "$RED" "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate Bevy project
if [ ! -d "$BEVY_PROJECT" ]; then
    print_color "$RED" "Error: Project not found: $BEVY_PROJECT"
    exit 1
fi

if [ ! -d "$BEVY_PROJECT/assets" ]; then
    print_color "$RED" "Error: Not a Bevy project (no assets/ directory): $BEVY_PROJECT"
    exit 1
fi

# Set default output file if not specified
if [ -z "$OUTPUT_FILE" ]; then
    if [ "$FORMAT" = "toml" ]; then
        OUTPUT_FILE="$BEVY_PROJECT/assets/sprites_manifest.toml"
    else
        OUTPUT_FILE="$BEVY_PROJECT/assets/sprites_manifest.json"
    fi
fi

# Print header
print_color "$BLUE" "=========================================="
print_color "$BLUE" "DGX-Pixels Manifest Generation"
print_color "$BLUE" "=========================================="
echo "Project:     $BEVY_PROJECT"
echo "Format:      $FORMAT"
echo "Output:      $OUTPUT_FILE"
print_color "$BLUE" "=========================================="
echo

# Run Python manifest generator
cd "$PROJECT_ROOT"

if python3 -m python.deployment.manifest_generator "$BEVY_PROJECT/assets" "$OUTPUT_FILE" "$FORMAT"; then
    print_color "$GREEN" "✓ Manifest generated successfully"

    # Show summary
    if [ -f "$OUTPUT_FILE" ]; then
        FILE_SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)
        echo
        echo "File:        $OUTPUT_FILE"
        echo "Size:        $FILE_SIZE"

        # Count sprites in manifest
        if [ "$FORMAT" = "json" ]; then
            SPRITE_COUNT=$(python3 -c "import json; f = open('$OUTPUT_FILE'); d = json.load(f); print(d.get('sprite_count', 0))")
            echo "Sprites:     $SPRITE_COUNT"
        fi
    fi

    exit 0
else
    print_color "$RED" "✗ Manifest generation failed"
    exit 1
fi
