# DGX-Pixels Bevy Integration Example

This example demonstrates how to integrate AI sprite generation from DGX-Pixels into a Bevy game using the Model Context Protocol (MCP).

## Prerequisites

- Rust 1.75+
- Bevy 0.13
- DGX-Pixels MCP server running (WS-13)
- Python 3.10+ with DGX-Pixels backend

## Quick Start

```bash
# Build and run
cd examples/bevy_integration
cargo run --release

# Controls
WASD - Move player
ESC - Quit
```

## Architecture

```
Bevy Game
    ↓ (MCP Client)
FastMCP Server (WS-13)
    ↓ (ZeroMQ)
Python Backend (WS-10)
    ↓ (HTTP)
ComfyUI + SDXL
```

## Features

- **MCP Integration**: Connect to DGX-Pixels MCP server for sprite generation
- **Sprite Management**: Caching layer for efficient asset loading
- **Hot Reload**: Automatic asset reloading when sprites regenerate
- **Example Game**: Simple top-down game demonstrating integration
- **Batch Generation**: Generate multiple sprites efficiently
- **Placeholder Graphics**: Works out-of-box with placeholder sprites

## Project Structure

```
examples/bevy_integration/
├── Cargo.toml              # Bevy dependencies
├── src/
│   ├── main.rs             # Main game entry point
│   ├── mcp_client.rs       # MCP client integration
│   ├── sprite_manager.rs   # Sprite loading and management
│   └── game/
│       ├── mod.rs
│       ├── player.rs       # Player with AI-generated sprites
│       ├── enemies.rs      # Enemies with AI-generated sprites
│       └── level.rs        # Level with AI-generated tiles
├── assets/
│   ├── sprites/            # Generated sprites directory
│   ├── tiles/              # Generated tiles directory
│   └── placeholder/        # Placeholder graphics
├── prompts/
│   └── sprite_prompts.yaml # Sprite generation prompts
├── README.md               # This file
└── .mcp_config             # MCP connection configuration
```

## Generating Sprites

### Option 1: Via YAML Configuration

Edit `prompts/sprite_prompts.yaml`:

```yaml
player:
  prompt: "pixel art medieval knight character sprite, 32x32, front view, blue armor"
  style: "pixel_art"
  resolution: "1024x1024"
```

Then generate (requires implementation):

```bash
cargo run --release -- --generate-all
```

### Option 2: Programmatically

```rust
use bevy::prelude::*;
use mcp_client::{McpClient, GenerateSpriteParams};

fn generate_sprite_system(mcp: Res<McpClient>) {
    let params = GenerateSpriteParams {
        prompt: "pixel art warrior".to_string(),
        style: "pixel_art".to_string(),
        resolution: "1024x1024".to_string(),
    };

    match mcp.generate_sprite_sync(params) {
        Ok(path) => info!("Generated: {}", path),
        Err(e) => error!("Generation failed: {}", e),
    }
}
```

## Integration Guide

### Step 1: Add MCP Client to Your Bevy App

```rust
use bevy::prelude::*;
use mcp_client::McpPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(McpPlugin)  // Add MCP support
        .run();
}
```

### Step 2: Request Sprite Generation

```rust
fn setup_player(
    mut commands: Commands,
    mcp: Res<McpClient>,
    asset_server: Res<AssetServer>,
) {
    // Request sprite generation via MCP
    let sprite_path = mcp.generate_sprite_sync(GenerateSpriteParams {
        prompt: "pixel art warrior".to_string(),
        style: "pixel_art".to_string(),
        resolution: "1024x1024".to_string(),
    }).expect("Failed to generate sprite");

    // Load generated sprite
    let texture = asset_server.load(sprite_path);

    // Spawn entity with sprite
    commands.spawn(SpriteBundle {
        texture,
        ..default()
    });
}
```

### Step 3: Enable Hot Reload

```rust
App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes_override: Some(true),  // Enable hot reload
        ..default()
    }))
```

### Step 4: Manage Sprites with Caching

```rust
use sprite_manager::SpriteManager;

fn load_sprite(
    mut sprite_manager: ResMut<SpriteManager>,
    asset_server: Res<AssetServer>,
) {
    // Load or get cached sprite
    let handle = sprite_manager.load_or_get(
        "player_idle",                  // Unique name
        "sprites/player_idle.png",      // Path
        &asset_server
    );
}
```

## Batch Generation

Generate multiple sprites efficiently:

```rust
fn generate_enemy_sprites(mcp: Res<McpClient>) {
    let prompts = vec![
        GenerateSpriteParams {
            prompt: "pixel art goblin".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
        GenerateSpriteParams {
            prompt: "pixel art skeleton".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
    ];

    match mcp.generate_batch_sync(prompts) {
        Ok(paths) => {
            info!("Generated {} sprites", paths.len());
            for path in paths {
                info!("  - {}", path);
            }
        }
        Err(e) => error!("Batch generation failed: {}", e),
    }
}
```

## Sprite Generation Workflow

```
1. Game requests sprite via MCP client
        ↓
2. MCP server receives request
        ↓
3. Python backend queues job via ZeroMQ
        ↓
4. ComfyUI generates sprite with SDXL + LoRA
        ↓
5. MCP server returns sprite path
        ↓
6. Bevy loads sprite via AssetServer
        ↓
7. Hot reload detects changes (if enabled)
        ↓
8. Sprite appears in game
```

## Configuration

### MCP Server Configuration

Edit `.mcp_config`:

```json
{
  "server": {
    "command": "python",
    "args": ["-m", "python.mcp_server.server"],
    "cwd": "../.."
  },
  "tools": {
    "generate_sprite": {
      "timeout": 60000
    }
  }
}
```

### Asset Directory Structure

Ensure your Bevy project follows this structure:

```
your_game/
├── assets/
│   ├── sprites/        # AI-generated sprites go here
│   ├── tiles/          # AI-generated tiles go here
│   └── sounds/         # Other assets
```

**Important**: Use relative paths in Bevy code:

```rust
// ✅ Correct - relative to assets/
asset_server.load("sprites/player.png")

// ❌ Incorrect - absolute paths don't work
asset_server.load("/home/user/project/assets/sprites/player.png")
```

## Troubleshooting

### MCP Server Not Connecting

**Symptom**: Sprite generation requests fail immediately

**Solution**:
```bash
# Verify MCP server is running
cd /home/beengud/raibid-labs/dgx-pixels
python -m python.mcp_server.server

# Check logs
tail -f logs/mcp_server.log
```

### Sprites Not Loading

**Symptom**: Black squares instead of sprites

**Solution**:
1. Verify sprites exist in `assets/sprites/` directory
2. Check file paths are relative: `"sprites/player.png"` not `"/full/path/player.png"`
3. Enable hot reload: `watch_for_changes_override: Some(true)`
4. Check Bevy logs for asset loading errors

### Hot Reload Not Working

**Symptom**: Updated sprites don't appear until restart

**Solution**:
```rust
App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes_override: Some(true),  // Must be enabled
        ..default()
    }))
```

### Generation Timeout

**Symptom**: Sprite generation times out

**Solution**:
1. Increase timeout in `.mcp_config`: `"timeout": 120000`
2. Verify ComfyUI is running and responsive
3. Check Python backend worker is processing jobs
4. Monitor GPU memory usage on DGX-Spark

### Missing Placeholder Graphics

**Symptom**: Error loading `placeholder/player.png`

**Solution**:
```bash
# Create placeholder graphics
cd assets/placeholder
# Add simple 32x32 PNG files for player, enemy, ground
```

## Performance Tips

1. **Batch Generation**: Generate multiple sprites in one request
2. **Sprite Caching**: Use SpriteManager to avoid reloading
3. **Async Loading**: Use Bevy's async asset loading for large batches
4. **Resolution**: Start with 512x512 for faster generation, upscale later
5. **LoRA Models**: Use trained LoRA models for consistent style

## Example Prompts

### Character Sprites
```
"pixel art medieval knight character sprite, 32x32, front view, blue armor"
"pixel art wizard character sprite, 32x32, purple robes, staff"
"pixel art rogue character sprite, 32x32, dark leather, daggers"
```

### Enemy Sprites
```
"pixel art goblin enemy sprite, 32x32, green skin, carrying club"
"pixel art skeleton warrior sprite, 32x32, white bones, sword"
"pixel art dragon sprite, 64x64, red scales, breathing fire"
```

### Tile Sprites
```
"pixel art grass ground tile, 32x32, seamless, top-down view"
"pixel art stone wall tile, 32x32, seamless, top-down view"
"pixel art water tile, 32x32, seamless, animated, top-down view"
```

### Item Sprites
```
"pixel art health potion item, 16x16, red liquid, glass bottle"
"pixel art iron sword item, 16x16, medieval style"
"pixel art treasure chest, 32x32, closed, wooden"
```

## Next Steps

### For Development
1. Implement async sprite generation system
2. Add progress indicators for generation
3. Integrate with bevy_brp_mcp for full MCP support
4. Create generation queue visualization
5. Add batch generation UI

### For Production
1. Pre-generate common sprites during build
2. Implement sprite caching strategy
3. Add fallback sprites for generation failures
4. Monitor generation performance
5. Optimize batch sizes for your game

## Resources

- **DGX-Pixels Documentation**: `../../docs/`
- **Bevy Asset System**: https://bevyengine.org/learn/book/assets/
- **MCP Protocol**: `../../docs/04-bevy-integration.md`
- **Sprite Prompts**: `./prompts/sprite_prompts.yaml`

## Contributing

See `../../CONTRIBUTING.md` for guidelines on:
- Reporting integration issues
- Submitting example improvements
- Adding new integration patterns
- Testing with different Bevy versions

## License

Same as parent DGX-Pixels project (see `../../LICENSE`).

## Manifest-Driven Asset Loading

The deployment pipeline generates a manifest file that lists all deployed sprites with metadata. This enables programmatic asset loading and discovery.

### Loading Manifest on Startup

```rust
use bevy::prelude::*;
use manifest_loader::ManifestLoaderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ManifestLoaderPlugin::default())  // Load manifest
        .run();
}
```

### Accessing Manifest Data

```rust
use bevy::prelude::*;
use manifest_loader::ManifestData;

fn use_manifest(manifest: Res<ManifestData>) {
    // Get sprite metadata
    if let Some(sprite) = manifest.get_sprite("character_knight_idle") {
        info!("Sprite: {} ({}x{}, {} frames)",
            sprite.name,
            sprite.resolution.unwrap_or([0, 0])[0],
            sprite.resolution.unwrap_or([0, 0])[1],
            sprite.frames
        );
    }

    // Get all sprites in a category
    if let Some(characters) = manifest.get_sprites_by_category("character") {
        info!("Found {} character sprites", characters.len());
    }

    // Get all sprite names
    let all_sprites = manifest.get_all_names();
    info!("Total sprites: {}", all_sprites.len());
}
```

### Loading Sprites from Manifest

```rust
use bevy::prelude::*;
use manifest_loader::{ManifestData, load_sprites_from_manifest};

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    manifest: Res<ManifestData>,
) {
    // Load all sprites from manifest
    let handles = load_sprites_from_manifest(&manifest, &asset_server);

    // Use a specific sprite
    if let Some(handle) = handles.get("character_knight_idle") {
        commands.spawn(SpriteBundle {
            texture: handle.clone(),
            ..default()
        });
    }
}
```

### Loading by Category

```rust
use bevy::prelude::*;
use manifest_loader::{ManifestData, load_sprites_by_category};

fn load_characters(
    asset_server: Res<AssetServer>,
    manifest: Res<ManifestData>,
) {
    // Load only character sprites
    let character_handles = load_sprites_by_category(
        &manifest,
        "character",
        &asset_server
    );

    info!("Loaded {} character sprites", character_handles.len());
}
```

### Manifest Structure

The manifest at `assets/sprites_manifest.json` contains:

```json
{
  "version": "1.0",
  "generated_at": "2025-11-13T12:00:00Z",
  "sprite_count": 3,
  "sprites": [
    {
      "name": "character_knight_idle",
      "path": "sprites/character_knight_idle_0001.png",
      "category": "character",
      "variant": "idle",
      "frames": 1,
      "resolution": [64, 64],
      "file_size_kb": 12.5
    }
  ]
}
```

### Deployment Pipeline Integration

The manifest is automatically generated when using the deployment pipeline:

```bash
# Deploy sprites with manifest generation
cd ../../
./scripts/deploy_assets.sh outputs/ examples/bevy_integration/

# Regenerate manifest only
./scripts/generate_manifest.sh examples/bevy_integration/
```

For more information, see:
- **Deployment Guide**: `../../docs/deployment/deployment-guide.md`
- **Asset Conventions**: `../../docs/deployment/asset-conventions.md`

