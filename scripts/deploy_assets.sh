#!/usr/bin/env bash
#
# Asset Deployment Pipeline for DGX-Pixels
#
# Deploys AI-generated sprites from ComfyUI outputs to Bevy game projects
# with validation, optional post-processing, and manifest generation.
#
# Usage:
#   ./scripts/deploy_assets.sh <source_dir> <target_bevy_project> [options]
#
# Examples:
#   ./scripts/deploy_assets.sh outputs/ ../my-game/
#   ./scripts/deploy_assets.sh outputs/ ../my-game/ --no-validate
#   ./scripts/deploy_assets.sh outputs/ ../my-game/ --preset retro

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default options
VALIDATE=true
POST_PROCESS=false
GENERATE_MANIFEST=true
BACKUP=true
PRESET="pixel_art"
DRY_RUN=false

# Print colored message
print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Print usage
usage() {
    cat << EOF
Asset Deployment Pipeline for DGX-Pixels

USAGE:
    $0 <source_dir> <target_bevy_project> [options]

ARGUMENTS:
    source_dir              Directory containing generated sprites (e.g., outputs/)
    target_bevy_project     Path to Bevy game project root

OPTIONS:
    --no-validate          Skip validation checks
    --post-process         Enable post-processing
    --preset <name>        Post-processing preset (pixel_art, retro, modern, minimal)
    --no-manifest          Skip manifest generation
    --no-backup            Skip backup creation
    --dry-run              Show what would be deployed without deploying
    -h, --help             Show this help message

EXAMPLES:
    # Deploy sprites with validation and manifest
    $0 outputs/ ../my-game/

    # Deploy with post-processing (retro preset)
    $0 outputs/ ../my-game/ --post-process --preset retro

    # Dry run to preview deployment
    $0 outputs/ ../my-game/ --dry-run

NOTES:
    - Source directory should contain PNG files
    - Target must be a valid Bevy project (has assets/ directory)
    - Validation checks format, resolution, naming convention
    - Post-processing includes color quantization, cropping, optimization
    - Manifest is generated at assets/sprites_manifest.json

EOF
}

# Parse arguments
if [ $# -lt 2 ]; then
    usage
    exit 1
fi

SOURCE_DIR="$1"
TARGET_PROJECT="$2"
shift 2

# Parse options
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-validate)
            VALIDATE=false
            shift
            ;;
        --post-process)
            POST_PROCESS=true
            shift
            ;;
        --preset)
            PRESET="$2"
            shift 2
            ;;
        --no-manifest)
            GENERATE_MANIFEST=false
            shift
            ;;
        --no-backup)
            BACKUP=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
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

# Validate arguments
if [ ! -d "$SOURCE_DIR" ]; then
    print_color "$RED" "Error: Source directory not found: $SOURCE_DIR"
    exit 1
fi

if [ ! -d "$TARGET_PROJECT" ]; then
    print_color "$RED" "Error: Target project not found: $TARGET_PROJECT"
    exit 1
fi

if [ ! -d "$TARGET_PROJECT/assets" ]; then
    print_color "$RED" "Error: Target is not a Bevy project (no assets/ directory): $TARGET_PROJECT"
    exit 1
fi

# Count PNG files in source
PNG_COUNT=$(find "$SOURCE_DIR" -name "*.png" -type f | wc -l)

if [ "$PNG_COUNT" -eq 0 ]; then
    print_color "$YELLOW" "Warning: No PNG files found in $SOURCE_DIR"
    exit 0
fi

# Print deployment summary
print_color "$BLUE" "=========================================="
print_color "$BLUE" "DGX-Pixels Asset Deployment Pipeline"
print_color "$BLUE" "=========================================="
echo "Source:      $SOURCE_DIR"
echo "Target:      $TARGET_PROJECT"
echo "Assets:      $PNG_COUNT PNG files"
echo "Validate:    $VALIDATE"
echo "Process:     $POST_PROCESS"
if [ "$POST_PROCESS" = true ]; then
    echo "Preset:      $PRESET"
fi
echo "Manifest:    $GENERATE_MANIFEST"
echo "Backup:      $BACKUP"
echo "Dry Run:     $DRY_RUN"
print_color "$BLUE" "=========================================="
echo

# Step 1: Validation
if [ "$VALIDATE" = true ]; then
    print_color "$BLUE" "[1/4] Validating assets..."

    if ! python3 -m python.deployment.validator "$SOURCE_DIR"; then
        print_color "$RED" "✗ Validation failed"
        exit 1
    fi

    print_color "$GREEN" "✓ Validation passed"
    echo
fi

# Step 2: Post-processing (optional)
if [ "$POST_PROCESS" = true ]; then
    print_color "$BLUE" "[2/4] Post-processing assets..."

    PROCESSED_DIR="$SOURCE_DIR/.processed"
    mkdir -p "$PROCESSED_DIR"

    # Process each PNG file
    find "$SOURCE_DIR" -maxdepth 1 -name "*.png" -type f | while read -r file; do
        filename=$(basename "$file")
        output_file="$PROCESSED_DIR/$filename"

        if [ "$DRY_RUN" = true ]; then
            echo "  Would process: $filename (preset: $PRESET)"
        else
            if python3 -m python.deployment.post_processor "$file" "$output_file" "$PRESET"; then
                print_color "$GREEN" "  ✓ Processed: $filename"
            else
                print_color "$YELLOW" "  ⚠ Failed to process: $filename (using original)"
                cp "$file" "$output_file"
            fi
        fi
    done

    # Use processed directory as source
    SOURCE_DIR="$PROCESSED_DIR"

    print_color "$GREEN" "✓ Post-processing complete"
    echo
else
    print_color "$BLUE" "[2/4] Skipping post-processing"
    echo
fi

# Step 3: Deploy assets
print_color "$BLUE" "[3/4] Deploying assets..."

TARGET_SPRITES_DIR="$TARGET_PROJECT/assets/sprites"
mkdir -p "$TARGET_SPRITES_DIR"

# Create backup if enabled
if [ "$BACKUP" = true ] && [ "$DRY_RUN" = false ]; then
    BACKUP_DIR="$TARGET_PROJECT/assets/.backup/$(date +%Y%m%d_%H%M%S)"
    if [ "$(ls -A "$TARGET_SPRITES_DIR" 2>/dev/null)" ]; then
        mkdir -p "$BACKUP_DIR"
        cp -r "$TARGET_SPRITES_DIR"/* "$BACKUP_DIR/" 2>/dev/null || true
        print_color "$BLUE" "  Backup created: $BACKUP_DIR"
    fi
fi

# Copy assets
DEPLOYED_COUNT=0
find "$SOURCE_DIR" -maxdepth 1 -name "*.png" -type f | while read -r file; do
    filename=$(basename "$file")
    target_file="$TARGET_SPRITES_DIR/$filename"

    if [ "$DRY_RUN" = true ]; then
        echo "  Would deploy: $filename"
    else
        cp "$file" "$target_file"
        print_color "$GREEN" "  ✓ Deployed: $filename"
    fi

    DEPLOYED_COUNT=$((DEPLOYED_COUNT + 1))
done

if [ "$DRY_RUN" = false ]; then
    # Wait for filesystem to settle (for hot reload compatibility)
    sleep 0.1

    print_color "$GREEN" "✓ Deployed $PNG_COUNT assets"
else
    print_color "$BLUE" "✓ Would deploy $PNG_COUNT assets"
fi
echo

# Step 4: Generate manifest
if [ "$GENERATE_MANIFEST" = true ]; then
    print_color "$BLUE" "[4/4] Generating asset manifest..."

    MANIFEST_FILE="$TARGET_PROJECT/assets/sprites_manifest.json"

    if [ "$DRY_RUN" = true ]; then
        print_color "$BLUE" "  Would generate manifest: $MANIFEST_FILE"
    else
        if python3 -m python.deployment.manifest_generator "$TARGET_PROJECT/assets" "$MANIFEST_FILE" "json"; then
            print_color "$GREEN" "✓ Manifest generated: $MANIFEST_FILE"
        else
            print_color "$YELLOW" "⚠ Failed to generate manifest"
        fi
    fi
    echo
else
    print_color "$BLUE" "[4/4] Skipping manifest generation"
    echo
fi

# Summary
print_color "$BLUE" "=========================================="
if [ "$DRY_RUN" = true ]; then
    print_color "$BLUE" "Dry Run Complete"
else
    print_color "$GREEN" "Deployment Complete"
fi
print_color "$BLUE" "=========================================="
echo "Deployed:    $PNG_COUNT assets"
echo "Location:    $TARGET_SPRITES_DIR"
if [ "$GENERATE_MANIFEST" = true ]; then
    echo "Manifest:    $TARGET_PROJECT/assets/sprites_manifest.json"
fi
print_color "$BLUE" "=========================================="
echo

if [ "$DRY_RUN" = false ]; then
    print_color "$GREEN" "✓ Assets ready for use in Bevy project"
    print_color "$BLUE" "  Load with: asset_server.load(\"sprites/your_sprite.png\")"
fi
