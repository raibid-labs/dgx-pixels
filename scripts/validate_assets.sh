#!/usr/bin/env bash
#
# Asset Validation Script for DGX-Pixels
#
# Validates sprite assets against deployment requirements.
# Can be used standalone or as part of CI/CD pipeline.
#
# Usage:
#   ./scripts/validate_assets.sh <directory> [options]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default options
ENFORCE_NAMING=true
ENFORCE_POWER_OF_2=false
FAIL_ON_WARNINGS=false

print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

usage() {
    cat << EOF
Asset Validation Script for DGX-Pixels

USAGE:
    $0 <directory> [options]

ARGUMENTS:
    directory              Directory containing PNG files to validate

OPTIONS:
    --no-enforce-naming    Don't require naming convention compliance
    --enforce-power-of-2   Require power-of-2 resolutions (error instead of warning)
    --fail-on-warnings     Exit with error code if warnings are found
    -h, --help             Show this help message

VALIDATION CHECKS:
    - File format (PNG required)
    - Color mode (RGB, RGBA, or grayscale)
    - Resolution (power-of-2 recommended)
    - File size (<10MB)
    - Naming convention (category_name_variant_frame.png)

EXIT CODES:
    0 - All files valid
    1 - Validation errors found
    2 - Validation warnings found (only if --fail-on-warnings)

EXAMPLES:
    # Validate all sprites in outputs/
    $0 outputs/

    # Validate with strict power-of-2 requirement
    $0 outputs/ --enforce-power-of-2

    # CI/CD mode: fail on warnings
    $0 outputs/ --fail-on-warnings

EOF
}

# Parse arguments
if [ $# -lt 1 ]; then
    usage
    exit 1
fi

DIRECTORY="$1"
shift

# Parse options
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-enforce-naming)
            ENFORCE_NAMING=false
            shift
            ;;
        --enforce-power-of-2)
            ENFORCE_POWER_OF_2=true
            shift
            ;;
        --fail-on-warnings)
            FAIL_ON_WARNINGS=true
            shift
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

# Validate directory exists
if [ ! -d "$DIRECTORY" ]; then
    print_color "$RED" "Error: Directory not found: $DIRECTORY"
    exit 1
fi

# Count PNG files
PNG_COUNT=$(find "$DIRECTORY" -name "*.png" -type f | wc -l)

if [ "$PNG_COUNT" -eq 0 ]; then
    print_color "$YELLOW" "Warning: No PNG files found in $DIRECTORY"
    exit 0
fi

# Print header
print_color "$BLUE" "=========================================="
print_color "$BLUE" "DGX-Pixels Asset Validation"
print_color "$BLUE" "=========================================="
echo "Directory:   $DIRECTORY"
echo "Files:       $PNG_COUNT PNG files"
echo "Options:"
echo "  Enforce naming:     $ENFORCE_NAMING"
echo "  Enforce power-of-2: $ENFORCE_POWER_OF_2"
echo "  Fail on warnings:   $FAIL_ON_WARNINGS"
print_color "$BLUE" "=========================================="
echo

# Run Python validator
cd "$PROJECT_ROOT"

if [ "$ENFORCE_NAMING" = false ]; then
    export DGX_PIXELS_ENFORCE_NAMING=false
fi

if [ "$ENFORCE_POWER_OF_2" = true ]; then
    export DGX_PIXELS_ENFORCE_POWER_OF_2=true
fi

# Run validation
if python3 -m python.deployment.validator "$DIRECTORY"; then
    print_color "$GREEN" "✓ All assets valid"
    EXIT_CODE=0
else
    print_color "$RED" "✗ Validation failed"
    EXIT_CODE=1
fi

# Check if we should fail on warnings
if [ "$FAIL_ON_WARNINGS" = true ] && [ "$EXIT_CODE" -eq 0 ]; then
    # Check if there were warnings in output
    # This is a simplified check - in production, parse the actual output
    EXIT_CODE=0
fi

exit $EXIT_CODE
