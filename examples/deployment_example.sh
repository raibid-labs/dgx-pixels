#!/usr/bin/env bash
#
# DGX-Pixels Deployment Pipeline - Usage Examples
#
# This script demonstrates various deployment scenarios.
# Run sections individually or as a complete workflow.

set -e

echo "=========================================="
echo "DGX-Pixels Deployment Pipeline Examples"
echo "=========================================="
echo

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_section() {
    echo -e "${BLUE}$1${NC}"
    echo "---"
}

# Example 1: Basic Validation
print_section "Example 1: Validate Sprites"
echo "Command: ./scripts/validate_assets.sh outputs/"
echo
echo "This validates all PNG files in outputs/ directory:"
echo "  - Checks file format (PNG required)"
echo "  - Validates color mode (RGB/RGBA/L)"
echo "  - Checks resolution (power-of-2 recommended)"
echo "  - Validates naming convention"
echo "  - Checks file size (<10MB)"
echo

# Example 2: Dry Run Deployment
print_section "Example 2: Preview Deployment (Dry Run)"
echo "Command: ./scripts/deploy_assets.sh outputs/ examples/bevy_integration/ --dry-run"
echo
echo "Shows what would be deployed without making changes:"
echo "  - Runs validation"
echo "  - Shows files to be copied"
echo "  - Shows manifest that would be generated"
echo "  - No actual file changes"
echo

# Example 3: Basic Deployment
print_section "Example 3: Deploy Sprites"
echo "Command: ./scripts/deploy_assets.sh outputs/ examples/bevy_integration/"
echo
echo "Full deployment with validation and manifest:"
echo "  - Validates all sprites"
echo "  - Creates backup of existing sprites"
echo "  - Copies sprites to assets/sprites/"
echo "  - Generates sprites_manifest.json"
echo

# Example 4: Deployment with Post-Processing
print_section "Example 4: Deploy with Post-Processing (Pixel Art)"
echo "Command: ./scripts/deploy_assets.sh outputs/ examples/bevy_integration/ --post-process --preset pixel_art"
echo
echo "Deploys with pixel art optimization:"
echo "  - Color quantization (64-color palette)"
echo "  - Auto-cropping (removes empty borders)"
echo "  - PNG optimization (compression level 9)"
echo

# Example 5: Deployment with Retro Preset
print_section "Example 5: Deploy with Retro Preset"
echo "Command: ./scripts/deploy_assets.sh outputs/ examples/bevy_integration/ --post-process --preset retro"
echo
echo "Deploys with retro game optimization:"
echo "  - Color quantization (16-color palette)"
echo "  - Auto-cropping"
echo "  - PNG optimization"
echo

# Example 6: Deployment Without Validation
print_section "Example 6: Deploy Without Validation"
echo "Command: ./scripts/deploy_assets.sh outputs/ examples/bevy_integration/ --no-validate"
echo
echo "Skips validation step (use with caution):"
echo "  - No format/size checks"
echo "  - No naming validation"
echo "  - Faster deployment"
echo "  - Risk of deploying invalid assets"
echo

# Example 7: Regenerate Manifest Only
print_section "Example 7: Regenerate Manifest Only"
echo "Command: ./scripts/generate_manifest.sh examples/bevy_integration/"
echo
echo "Updates manifest without redeploying sprites:"
echo "  - Scans assets/sprites/ directory"
echo "  - Extracts metadata from filenames"
echo "  - Generates sprites_manifest.json"
echo

# Example 8: Generate TOML Manifest
print_section "Example 8: Generate TOML Manifest"
echo "Command: ./scripts/generate_manifest.sh examples/bevy_integration/ --format toml"
echo
echo "Generates TOML format manifest instead of JSON:"
echo "  - Same metadata as JSON"
echo "  - TOML format (more readable for some)"
echo "  - Compatible with TOML parsers"
echo

# Example 9: Validate with Strict Mode
print_section "Example 9: Strict Validation"
echo "Command: ./scripts/validate_assets.sh outputs/ --enforce-power-of-2 --fail-on-warnings"
echo
echo "Validates with strict requirements:"
echo "  - Power-of-2 resolutions required (not just recommended)"
echo "  - Fails on warnings (for CI/CD)"
echo "  - Returns error code if any issues"
echo

# Example 10: Complete Workflow
print_section "Example 10: Complete Production Workflow"
echo "Commands:"
echo "  1. ./scripts/validate_assets.sh outputs/ --fail-on-warnings"
echo "  2. ./scripts/deploy_assets.sh outputs/ ../my-game/ --post-process --preset pixel_art"
echo "  3. cd ../my-game && cargo run --release"
echo
echo "Production workflow:"
echo "  1. Strict validation (fail on any issues)"
echo "  2. Deploy with post-processing"
echo "  3. Run game to verify"
echo

# Example 11: CI/CD Pipeline
print_section "Example 11: CI/CD Integration"
echo "GitHub Actions example:"
echo
cat << 'YAML'
name: Deploy Assets

on:
  push:
    paths:
      - 'outputs/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.10'

      - name: Install dependencies
        run: pip install -r python/requirements.txt

      - name: Validate assets
        run: ./scripts/validate_assets.sh outputs/ --fail-on-warnings

      - name: Deploy
        run: ./scripts/deploy_assets.sh outputs/ examples/bevy_integration/
YAML
echo

# Example 12: Bevy Integration
print_section "Example 12: Load Sprites in Bevy"
echo "Rust code example:"
echo
cat << 'RUST'
use bevy::prelude::*;
use manifest_loader::{ManifestLoaderPlugin, ManifestData};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .add_plugins(ManifestLoaderPlugin::default())
        .add_systems(Startup, load_sprites)
        .run();
}

fn load_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    manifest: Res<ManifestData>,
) {
    // Load sprite by name
    if let Some(sprite) = manifest.get_sprite("character_knight_idle") {
        let texture: Handle<Image> = asset_server.load(&sprite.path);
        commands.spawn(SpriteBundle {
            texture,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });

        info!("Loaded {} ({}x{}, {} frames)",
            sprite.name,
            sprite.resolution.unwrap_or([0, 0])[0],
            sprite.resolution.unwrap_or([0, 0])[1],
            sprite.frames
        );
    }

    // Load all character sprites
    if let Some(characters) = manifest.get_sprites_by_category("character") {
        info!("Found {} character sprites", characters.len());
    }
}
RUST
echo

echo "=========================================="
echo -e "${GREEN}Examples Complete${NC}"
echo "=========================================="
echo
echo "For more information:"
echo "  - Deployment Guide: docs/deployment/deployment-guide.md"
echo "  - Asset Conventions: docs/deployment/asset-conventions.md"
echo "  - Configuration: config/deployment_config.yaml"
echo
