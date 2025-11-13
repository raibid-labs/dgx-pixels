# Asset Deployment Pipeline Guide

This guide explains how to deploy AI-generated sprites from DGX-Pixels to Bevy game projects using the automated deployment pipeline.

## Overview

The deployment pipeline automates the process of:

1. **Validating** generated sprites against quality standards
2. **Post-processing** sprites for game use (optional)
3. **Deploying** assets to Bevy project directories
4. **Generating** asset manifests for programmatic loading

## Quick Start

### Basic Deployment

Deploy sprites from ComfyUI output to a Bevy project:

```bash
./scripts/deploy_assets.sh outputs/ ../my-game/
```

This will:
- Validate all PNG files in `outputs/`
- Copy them to `../my-game/assets/sprites/`
- Generate manifest at `../my-game/assets/sprites_manifest.json`
- Create backups of existing sprites

### With Post-Processing

Apply post-processing (color quantization, cropping, optimization):

```bash
./scripts/deploy_assets.sh outputs/ ../my-game/ --post-process --preset pixel_art
```

### Dry Run

Preview what would be deployed without making changes:

```bash
./scripts/deploy_assets.sh outputs/ ../my-game/ --dry-run
```

## Deployment Pipeline Components

### 1. Validation (`validate_assets.sh`)

Validates sprites against deployment requirements:

```bash
./scripts/validate_assets.sh outputs/
```

**Validation Checks**:
- **Format**: Must be PNG
- **Color Mode**: RGB, RGBA, or grayscale
- **Resolution**: Power-of-2 recommended (128, 256, 512, 1024, 2048)
- **File Size**: <10MB per file
- **Naming Convention**: `category_name_variant_frame.png`

**Exit Codes**:
- `0` - All files valid
- `1` - Validation errors found

**Options**:
```bash
# Skip naming convention enforcement
./scripts/validate_assets.sh outputs/ --no-enforce-naming

# Require power-of-2 resolutions (error instead of warning)
./scripts/validate_assets.sh outputs/ --enforce-power-of-2

# Fail on warnings (for CI/CD)
./scripts/validate_assets.sh outputs/ --fail-on-warnings
```

### 2. Post-Processing (`post_processor.py`)

Transforms sprites for optimal game use:

**Available Presets**:

- **`pixel_art`** (default): 64-color palette, auto-crop, aggressive compression
- **`retro`**: 16-color palette, auto-crop, aggressive compression
- **`modern`**: No quantization, auto-crop, moderate compression
- **`minimal`**: No processing, moderate compression only

**Usage**:
```bash
# Process single file
python -m python.deployment.post_processor input.png output.png pixel_art

# Process as part of deployment
./scripts/deploy_assets.sh outputs/ ../my-game/ --post-process --preset retro
```

**Processing Options**:
- Color quantization (reduce to N-color palette)
- Auto-cropping (remove empty borders)
- Scaling (nearest-neighbor for pixel-perfect)
- Background removal/replacement
- PNG optimization

### 3. Manifest Generation (`generate_manifest.sh`)

Generates asset manifests for programmatic loading:

```bash
# Generate JSON manifest
./scripts/generate_manifest.sh ../my-game/

# Generate TOML manifest
./scripts/generate_manifest.sh ../my-game/ --format toml --output assets/sprites.toml
```

**Manifest Format** (JSON):
```json
{
  "version": "1.0",
  "generated_at": "2025-11-13T10:30:00Z",
  "sprite_count": 2,
  "sprites": [
    {
      "name": "character_knight_idle",
      "path": "sprites/character_knight_idle_0001.png",
      "category": "character",
      "variant": "idle",
      "frames": 4,
      "resolution": [64, 64],
      "generated_at": "2025-11-13T10:30:00Z",
      "file_size_kb": 12.5
    }
  ]
}
```

## Naming Convention

Follow this naming convention for automatic metadata extraction:

### Pattern

```
{category}_{name}_{variant}_{frame}.png
```

### Components

- **category**: Asset type (character, enemy, npc, item, tile, ui, effect, projectile, environment, background)
- **name**: Specific name (knight, goblin, grass, button)
- **variant**: State or style (idle, walk, attack, hover)
- **frame**: Frame number for animations (0001, 0002, ..., 9999)

### Examples

**Characters**:
```
character_knight_idle_0001.png
character_knight_walk_0001.png
character_wizard_attack_0003.png
```

**Enemies**:
```
enemy_goblin_idle_0001.png
enemy_skeleton_attack_0004.png
```

**Tiles**:
```
tile_grass_001.png
tile_stone_wall_002.png
```

**UI Elements**:
```
ui_button_normal.png
ui_button_hover.png
ui_health_bar_empty.png
```

**Single-Frame Assets**:
```
item_potion_health.png
projectile_arrow.png
```

## Directory Structure

### Source (ComfyUI Output)

```
outputs/
├── character_knight_idle_0001.png
├── character_knight_walk_0001.png
├── enemy_goblin_attack_0001.png
└── tile_grass_001.png
```

### Target (Bevy Project)

```
my-game/
├── assets/
│   ├── sprites/
│   │   ├── character_knight_idle_0001.png
│   │   ├── character_knight_walk_0001.png
│   │   ├── enemy_goblin_attack_0001.png
│   │   └── tile_grass_001.png
│   ├── sprites_manifest.json
│   └── .backup/
│       └── 20251113_103000/
│           └── [old sprites]
├── src/
└── Cargo.toml
```

## Integration with Bevy

### Loading Sprites Directly

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture: Handle<Image> = asset_server.load("sprites/character_knight_idle_0001.png");

    commands.spawn(SpriteBundle {
        texture,
        ..default()
    });
}
```

### Loading from Manifest

```rust
use bevy::prelude::*;
use manifest_loader::ManifestData;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    manifest: Res<ManifestData>,
) {
    // Get sprite metadata
    if let Some(sprite) = manifest.get_sprite("character_knight_idle") {
        let texture: Handle<Image> = asset_server.load(&sprite.path);

        info!("Loading {} ({}x{}, {} frames)",
            sprite.name,
            sprite.resolution.unwrap_or([0, 0])[0],
            sprite.resolution.unwrap_or([0, 0])[1],
            sprite.frames
        );

        commands.spawn(SpriteBundle {
            texture,
            ..default()
        });
    }
}
```

### Loading by Category

```rust
use bevy::prelude::*;
use manifest_loader::{ManifestData, load_sprites_by_category};

fn load_all_characters(
    asset_server: Res<AssetServer>,
    manifest: Res<ManifestData>,
) {
    // Load all sprites in "character" category
    let handles = load_sprites_by_category(&manifest, "character", &asset_server);

    info!("Loaded {} character sprites", handles.len());

    for (name, handle) in handles {
        // Use handles...
    }
}
```

## Hot Reload Compatibility

The deployment pipeline is compatible with Bevy's hot reload feature:

### Enable Hot Reload in Bevy

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),  // Enable hot reload
            ..default()
        }))
        .run();
}
```

### Deploy While Running

1. Start your Bevy game
2. Generate new sprites via MCP or ComfyUI
3. Deploy sprites: `./scripts/deploy_assets.sh outputs/ ../my-game/`
4. Sprites automatically reload in game (no restart needed)

**Note**: Manifest updates require manual reload by calling `ManifestData::load_from_file()` again.

## Configuration

Customize deployment behavior in `config/deployment_config.yaml`:

```yaml
# Naming convention
naming:
  enforce: true  # Require naming convention compliance
  categories:
    - character
    - enemy
    # ... add custom categories

# Validation rules
validation:
  resolution:
    enforce_power_of_2: false  # false = warning only
    allow_non_square: true
  max_file_size_mb: 10

# Post-processing presets
post_processing:
  default_preset: "pixel_art"
  presets:
    pixel_art:
      quantize_colors: true
      max_colors: 64

# Deployment
deployment:
  bevy:
    validate_structure: true  # Check for assets/ directory
    sprite_subdir: "sprites"  # Where to deploy sprites
  backup:
    enabled: true
    keep_last: 5  # Keep 5 most recent backups

# Manifest
manifest:
  enabled: true
  format: "json"  # or "toml"
```

## CI/CD Integration

Use the deployment pipeline in CI/CD workflows:

### GitHub Actions Example

```yaml
name: Validate and Deploy Assets

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
        run: |
          ./scripts/validate_assets.sh outputs/ --fail-on-warnings

      - name: Deploy to game project
        run: |
          ./scripts/deploy_assets.sh outputs/ ../game-project/ --post-process
```

## Troubleshooting

### Validation Fails

**Issue**: Sprites fail validation

**Solutions**:
1. Check error messages for specific issues
2. Verify files are PNG format
3. Check naming convention: `category_name_variant_frame.png`
4. Ensure resolution is reasonable (16-4096 pixels)
5. Check file size (<10MB)

**Bypass** (use with caution):
```bash
./scripts/deploy_assets.sh outputs/ ../my-game/ --no-validate
```

### Post-Processing Artifacts

**Issue**: Post-processed sprites look incorrect

**Solutions**:
1. Try different preset: `--preset modern` (no quantization)
2. Disable post-processing: remove `--post-process` flag
3. Process manually with custom options

### Manifest Not Updating

**Issue**: Manifest doesn't reflect new sprites

**Solutions**:
1. Regenerate manifest: `./scripts/generate_manifest.sh ../my-game/`
2. Check manifest file permissions
3. Verify sprites are in correct directory

### Hot Reload Not Working

**Issue**: New sprites don't appear in running game

**Solutions**:
1. Verify hot reload is enabled: `watch_for_changes_override: Some(true)`
2. Check Bevy asset loading logs for errors
3. Ensure file paths are relative: `"sprites/sprite.png"` not `"/full/path/sprite.png"`
4. Wait a moment for filesystem to settle (automatic 100ms delay built in)

## Advanced Usage

### Custom Validation Rules

Create custom validator:

```python
from python.deployment.validator import AssetValidator

validator = AssetValidator(
    enforce_naming=True,
    enforce_power_of_2=True,  # Strict mode
    allow_non_square=False     # Only square sprites
)

result = validator.validate_file("sprite.png")
if not result.valid:
    for error in result.errors:
        print(f"Error: {error.message}")
```

### Custom Post-Processing

Create custom processing options:

```python
from python.deployment.post_processor import PostProcessor, ProcessingOptions

options = ProcessingOptions(
    quantize_colors=True,
    max_colors=32,
    scale=0.5,  # Half size
    auto_crop=True,
    optimize=True,
)

processor = PostProcessor()
processor.process_image("input.png", "output.png", options)
```

### Batch Processing

Process multiple files:

```bash
# Validate entire directory
./scripts/validate_assets.sh outputs/

# Deploy all sprites
./scripts/deploy_assets.sh outputs/ ../my-game/ --post-process

# Generate manifest
./scripts/generate_manifest.sh ../my-game/
```

## Best Practices

1. **Always Validate**: Run validation before deployment to catch issues early
2. **Use Naming Convention**: Consistent naming enables automatic metadata extraction
3. **Enable Backups**: Keep backups enabled for easy rollback
4. **Test Post-Processing**: Try different presets to find optimal settings
5. **Version Control Manifests**: Commit manifests to track asset changes
6. **Document Custom Categories**: If adding custom categories, document in team guidelines
7. **CI/CD Integration**: Automate validation and deployment in CI/CD pipeline
8. **Hot Reload in Dev**: Enable hot reload during development for faster iteration

## Next Steps

- **MCP Integration**: Use MCP tools to deploy directly from generation
- **Bevy Plugin**: Add manifest loader plugin to your Bevy project
- **Custom Workflows**: Create project-specific deployment workflows
- **Automation**: Integrate with build systems and CI/CD

## Resources

- **Configuration**: `config/deployment_config.yaml`
- **Scripts**: `scripts/deploy_assets.sh`, `validate_assets.sh`, `generate_manifest.sh`
- **Python Modules**: `python/deployment/`
- **Examples**: `examples/bevy_integration/`
- **Naming Convention**: `docs/deployment/asset-conventions.md`
