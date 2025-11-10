# Bevy Integration Guide

## Overview

This guide covers integrating DGX-Pixels AI pixel art generation with Bevy game engine projects. The integration supports both manual workflows and automated MCP-based deployment.

## Table of Contents
- [Bevy Asset System Basics](#bevy-asset-system-basics)
- [Manual Integration](#manual-integration)
- [MCP-Based Integration](#mcp-based-integration)
- [Asset Organization](#asset-organization)
- [Sprite Sheet Usage](#sprite-sheet-usage)
- [Hot Reloading](#hot-reloading)
- [Complete Example](#complete-example)

---

## Bevy Asset System Basics

### Asset Loading

Bevy's `AssetServer` handles async asset loading:

```rust
use bevy::prelude::*;

fn load_sprite(
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    // Load a sprite
    let texture = asset_server.load("sprites/character/knight.png");

    // Spawn entity with sprite
    commands.spawn(SpriteBundle {
        texture,
        ..default()
    });
}
```

### Asset Paths

Assets are loaded relative to the `assets/` directory:

```
my_game/
├── assets/
│   ├── sprites/
│   │   ├── characters/
│   │   │   └── knight.png     # Loaded as "sprites/characters/knight.png"
│   │   └── items/
│   │       └── potion.png     # Loaded as "sprites/items/potion.png"
│   └── textures/
└── src/
    └── main.rs
```

### Asset Handles

Handles are reference-counted smart pointers to assets:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct SpriteAssets {
    knight: Handle<Image>,
    potion: Handle<Image>,
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.insert_resource(SpriteAssets {
        knight: asset_server.load("sprites/characters/knight.png"),
        potion: asset_server.load("sprites/items/potion.png"),
    });
}

fn spawn_knight(
    mut commands: Commands,
    sprites: Res<SpriteAssets>
) {
    commands.spawn(SpriteBundle {
        texture: sprites.knight.clone(),
        ..default()
    });
}
```

### Asset Loading States

Check if assets are loaded:

```rust
use bevy::asset::LoadState;

fn check_assets_loaded(
    asset_server: Res<AssetServer>,
    sprites: Res<SpriteAssets>
) {
    match asset_server.load_state(&sprites.knight) {
        LoadState::Loaded => {
            println!("Knight sprite loaded!");
        }
        LoadState::Failed(error) => {
            eprintln!("Failed to load: {:?}", error);
        }
        _ => {
            // Still loading...
        }
    }
}
```

---

## Manual Integration

### Workflow

1. **Generate sprites** using DGX-Pixels CLI/API
2. **Review** generated images
3. **Copy** to Bevy `assets/` directory
4. **Reference** in Bevy code

### Example: Generating and Using

**Step 1: Generate**
```bash
# Using DGX-Pixels CLI
dgx-pixels generate character "medieval knight, standing pose, side view" \
  --style 16bit \
  --size 32 \
  --output ./generated/
```

**Step 2: Review and Copy**
```bash
# Review
ls generated/

# Copy to Bevy project
cp generated/knight_001.png ~/my_game/assets/sprites/characters/knight.png
```

**Step 3: Use in Bevy**
```rust
// In your Bevy game
fn spawn_knight(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sprites/characters/knight.png"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}
```

### Batch Generation Script

Automate the manual workflow:

```bash
#!/bin/bash
# generate_and_copy.sh

BEVY_PROJECT="$HOME/my_game"
ASSETS_DIR="$BEVY_PROJECT/assets"

# Generate characters
dgx-pixels batch prompts/characters.txt \
  --output ./generated/characters/

# Generate items
dgx-pixels batch prompts/items.txt \
  --output ./generated/items/

# Copy to Bevy project
cp -r ./generated/characters/* "$ASSETS_DIR/sprites/characters/"
cp -r ./generated/items/* "$ASSETS_DIR/sprites/items/"

echo "Assets deployed to $BEVY_PROJECT"
```

---

## MCP-Based Integration

### Setup bevy_brp_mcp

**Add to your Bevy project:**

```toml
# Cargo.toml
[dependencies]
bevy = "0.13"
bevy_brp_mcp = "0.1"
```

**Enable in your game:**

```rust
use bevy::prelude::*;
use bevy_brp_mcp::BrpMcpPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BrpMcpPlugin::default())
        .add_systems(Startup, setup)
        .run();
}
```

### Configure MCP Client

**In DGX-Pixels (FastAPI + FastMCP):**

```python
from fastapi import FastAPI
from fastmcp import FastMCP
import httpx

app = FastAPI()
mcp = FastMCP(app)

# Bevy BRP endpoint (when game is running)
BEVY_BRP_URL = "http://localhost:15702"

@mcp.tool()
async def generate_and_deploy_sprite(
    prompt: str,
    bevy_project_path: str,
    category: str = "sprites",
    size: int = 32,
    style: str = "16bit"
) -> dict:
    """Generate sprite and deploy to running Bevy game.

    Args:
        prompt: Description of sprite to generate
        bevy_project_path: Path to Bevy project root
        category: Asset category (sprites/items/characters)
        size: Sprite size in pixels
        style: Art style (16bit, 32bit, etc.)

    Returns:
        Dict with asset info and deployment status
    """
    # 1. Generate sprite
    result = await comfyui_client.generate(
        prompt=prompt,
        size=(size, size),
        lora=f"{style}_style"
    )

    # 2. Post-process
    processed_path = await process_sprite(result.path)

    # 3. Copy to Bevy assets
    asset_filename = f"{sanitize_filename(prompt)}.png"
    dest_path = f"{bevy_project_path}/assets/{category}/{asset_filename}"

    shutil.copy(processed_path, dest_path)

    # 4. Notify Bevy via BRP (if running)
    try:
        async with httpx.AsyncClient() as client:
            await client.post(
                f"{BEVY_BRP_URL}/notify",
                json={
                    "event": "asset_added",
                    "path": f"{category}/{asset_filename}"
                }
            )
    except Exception as e:
        print(f"Bevy not running: {e}")

    return {
        "status": "deployed",
        "asset_path": dest_path,
        "bevy_path": f"{category}/{asset_filename}"
    }
```

### Usage from AI Assistant

With MCP configured, AI assistants like Claude can:

```
User: "Generate a fire spell icon and add it to my Bevy game"

Claude executes:
generate_and_deploy_sprite(
    prompt="fire spell icon, pixel art, magic effect",
    bevy_project_path="/home/user/my_game",
    category="sprites/ui/spells",
    size=64,
    style="32bit"
)

Result:
✓ Sprite generated
✓ Processed and optimized
✓ Deployed to /home/user/my_game/assets/sprites/ui/spells/fire_spell_icon.png
✓ Bevy notified (if running)
```

### Advanced: Live Asset Creation

Monitor prompts file and auto-generate:

```rust
use bevy::prelude::*;
use notify::{Watcher, RecursiveMode};

// Watch prompts.txt for new requests
fn watch_prompts_file() {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify::watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch("prompts.txt", RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                // Read new prompts
                // Call DGX-Pixels API
                // Assets appear in game
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
}
```

---

## Asset Organization

### Recommended Structure

```
assets/
├── sprites/
│   ├── characters/
│   │   ├── player/
│   │   │   ├── idle.png
│   │   │   ├── walk.png
│   │   │   └── attack.png
│   │   ├── enemies/
│   │   │   ├── goblin.png
│   │   │   └── skeleton.png
│   │   └── npcs/
│   ├── items/
│   │   ├── weapons/
│   │   ├── potions/
│   │   └── tools/
│   ├── environment/
│   │   ├── tiles/
│   │   ├── props/
│   │   └── effects/
│   └── ui/
│       ├── icons/
│       └── buttons/
├── spritesheets/
│   ├── player_animations.png
│   └── tileset_dungeon.png
└── metadata/
    ├── player_animations.ron
    └── tileset_dungeon.ron
```

### Metadata Files

Store generation parameters for reproducibility:

```ron
// assets/metadata/knight_sprite.ron
(
    source: "dgx-pixels",
    prompt: "medieval knight character, standing pose",
    style: "16bit",
    lora: "fantasy_characters_v1",
    generated_date: "2025-01-15",
    size: (32, 32),
    color_palette: 16,
    seed: 42,
)
```

### Asset Registry

Track all AI-generated assets:

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct GeneratedAssetMetadata {
    path: String,
    prompt: String,
    style: String,
    generated_at: String,
    dgx_pixels_version: String,
}

#[derive(Resource, Default)]
struct AssetRegistry {
    assets: Vec<GeneratedAssetMetadata>,
}

impl AssetRegistry {
    fn register(&mut self, metadata: GeneratedAssetMetadata) {
        self.assets.push(metadata);
    }

    fn find_by_prompt(&self, prompt: &str) -> Vec<&GeneratedAssetMetadata> {
        self.assets
            .iter()
            .filter(|a| a.prompt.contains(prompt))
            .collect()
    }
}
```

---

## Sprite Sheet Usage

### Creating Texture Atlas

When DGX-Pixels generates multiple frames:

**Generated Files:**
```
output/
  knight_walk_0.png  # Frame 0
  knight_walk_1.png  # Frame 1
  knight_walk_2.png  # Frame 2
  knight_walk_3.png  # Frame 3
```

**Combine into Sprite Sheet:**
```bash
dgx-pixels assemble-sheet output/knight_walk_*.png \
  --output knight_walk_sheet.png \
  --columns 4
```

**Use in Bevy:**
```rust
use bevy::prelude::*;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>
) {
    // Load sprite sheet
    let texture = asset_server.load("spritesheets/knight_walk_sheet.png");

    // Create texture atlas layout
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),  // Tile size
        4,                    // Columns
        1,                    // Rows
        None,                 // Padding
        None                  // Offset
    );
    let texture_atlas_layout = texture_atlases.add(layout);

    // Spawn animated sprite
    commands.spawn((
        SpriteBundle {
            texture,
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: 0,
        },
        AnimationIndices { first: 0, last: 3 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}
```

### Automated Sheet Generation

**In DGX-Pixels API:**
```python
@app.post("/api/v1/generate/animation")
async def generate_animation(
    prompt: str,
    frames: int = 4,
    frame_prompts: list[str] = None
):
    """Generate animation frames and assemble into sprite sheet.

    Args:
        prompt: Base description
        frames: Number of frames
        frame_prompts: Optional per-frame prompts

    Returns:
        Sprite sheet image and layout data
    """
    if not frame_prompts:
        # Generate prompts for each frame
        frame_prompts = [
            f"{prompt}, frame {i}, animation sequence"
            for i in range(frames)
        ]

    # Generate each frame
    images = []
    for frame_prompt in frame_prompts:
        result = await comfyui_client.generate(frame_prompt)
        images.append(result.image)

    # Assemble sprite sheet
    sheet = assemble_sprite_sheet(images, cols=frames)

    # Generate Bevy metadata
    metadata = {
        "tile_size": {"x": 32, "y": 32},
        "columns": frames,
        "rows": 1,
        "frame_count": frames,
        "recommended_fps": 10
    }

    return {
        "sheet_path": save_sheet(sheet),
        "metadata": metadata
    }
```

---

## Hot Reloading

Bevy automatically reloads assets when files change. Leverage this for rapid iteration:

### Development Workflow

**Terminal 1: Watch for prompts**
```bash
# Watch file for new sprite requests
watch -n 2 "cat prompts.txt | tail -n 1 | xargs -I {} dgx-pixels generate character {} --output assets/sprites/temp/"
```

**Terminal 2: Run Bevy game**
```bash
cargo run
# Assets automatically reload when copied to assets/
```

**Terminal 3: Edit prompts**
```bash
echo "fire mage character" >> prompts.txt
# Watch terminal generates sprite
# Bevy automatically loads new sprite
```

### Asset Change Notification

React to new assets in Bevy:

```rust
use bevy::prelude::*;
use bevy::asset::AssetEvent;

fn handle_new_assets(
    mut events: EventReader<AssetEvent<Image>>,
    asset_server: Res<AssetServer>
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } => {
                println!("New asset loaded: {:?}", id);
                // Could spawn sprite automatically, etc.
            }
            AssetEvent::Modified { id } => {
                println!("Asset modified: {:?}", id);
            }
            _ => {}
        }
    }
}
```

---

## Complete Example

### Full Game Integration

**DGX-Pixels Generation:**
```bash
# Generate character sprites
dgx-pixels generate-set characters.yaml

# characters.yaml:
# - prompt: "knight character, idle pose"
#   name: knight_idle
# - prompt: "knight character, walking"
#   name: knight_walk
#   frames: 4
# - prompt: "mage character, casting spell"
#   name: mage_cast
#   frames: 3
```

**Bevy Game:**
```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            load_generated_assets,
            animate_sprites,
            handle_input
        ))
        .run();
}

#[derive(Resource)]
struct GameAssets {
    knight_idle: Handle<Image>,
    knight_walk_sheet: Handle<Image>,
    mage_cast_sheet: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Load camera
    commands.spawn(Camera2dBundle::default());

    // Load assets
    commands.insert_resource(GameAssets {
        knight_idle: asset_server.load("sprites/characters/knight_idle.png"),
        knight_walk_sheet: asset_server.load("spritesheets/knight_walk.png"),
        mage_cast_sheet: asset_server.load("spritesheets/mage_cast.png"),
    });
}

fn load_generated_assets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut loaded: Local<bool>
) {
    // Only load once all assets are ready
    if *loaded {
        return;
    }

    if asset_server.load_state(&assets.knight_idle) != LoadState::Loaded {
        return; // Still loading
    }

    // Spawn knight
    let walk_layout = texture_atlases.add(
        TextureAtlasLayout::from_grid(UVec2::new(32, 32), 4, 1, None, None)
    );

    commands.spawn((
        SpriteBundle {
            texture: assets.knight_walk_sheet.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        TextureAtlas {
            layout: walk_layout,
            index: 0,
        },
        Player,
        AnimationIndices { first: 0, last: 3 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    *loaded = true;
}

#[derive(Component)]
struct Player;

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>
) {
    for mut transform in &mut player_query {
        if keyboard.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 2.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 2.0;
        }
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index >= indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);
```

### CI/CD Integration

Automatically generate assets in CI:

```yaml
# .github/workflows/generate-assets.yml
name: Generate Game Assets

on:
  push:
    paths:
      - 'asset-requests/**'
  workflow_dispatch:

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install DGX-Pixels CLI
        run: |
          pip install dgx-pixels-cli

      - name: Generate assets
        env:
          DGX_PIXELS_API_KEY: ${{ secrets.DGX_PIXELS_API_KEY }}
        run: |
          dgx-pixels batch asset-requests/batch.yaml \
            --api-url ${{ secrets.DGX_PIXELS_URL }} \
            --output assets/sprites/

      - name: Commit new assets
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add assets/
          git commit -m "Generate new game assets" || exit 0
          git push
```

---

## Best Practices

### 1. Naming Conventions
- Use descriptive names: `knight_walk_side.png` not `sprite_01.png`
- Include variant info: `potion_health_red.png`, `potion_mana_blue.png`
- Use consistent prefixes: `char_`, `item_`, `tile_`

### 2. Organization
- Group by type, then subcategory
- Keep generated assets separate from hand-made ones initially
- Use metadata files to track generation params

### 3. Version Control
- Commit generated assets to Git
- Use Git LFS for large sprite sheets
- Track generation prompts in version control

### 4. Performance
- Use texture atlases for sprites used together
- Load commonly-used assets at startup
- Use AssetServer.load_folder() for bulk loading

### 5. Iteration
- Generate high-res, downscale as needed
- Keep source prompts for regeneration
- Use hot reloading during development

---

## Troubleshooting

### Assets Not Loading

**Check file paths:**
```rust
// Correct: relative to assets/
let texture = asset_server.load("sprites/character.png");

// Wrong: absolute path
let texture = asset_server.load("/home/user/game/assets/sprites/character.png");
```

**Check file permissions:**
```bash
chmod 644 assets/sprites/*.png
```

### Sprites Look Blurry

Set correct image sampler in Bevy:

```rust
use bevy::prelude::*;
use bevy::render::texture::ImageSampler;

fn setup(
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>
) {
    let handle = asset_server.load("sprites/character.png");

    // Set to nearest neighbor sampling
    if let Some(image) = images.get_mut(&handle) {
        image.sampler = ImageSampler::nearest();
    }
}
```

Or set default sampler:

```rust
use bevy::prelude::*;
use bevy::render::texture::ImagePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .run();
}
```

### MCP Connection Issues

1. **Check Bevy BRP is enabled:**
   ```toml
   [dependencies]
   bevy_brp_mcp = "0.1"
   ```

2. **Verify server is running:**
   ```bash
   curl http://localhost:15702/health
   ```

3. **Check firewall rules**

### Hot Reload Not Working

Enable file watcher explicitly:

```rust
use bevy::prelude::*;
use bevy::asset::AssetPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .run();
}
```

---

## Next Steps

- See `05-training-roadmap.md` for customizing models
- See `06-implementation-plan.md` for getting started
- Check out [Bevy Examples](https://github.com/bevyengine/bevy/tree/main/examples) for more sprite techniques
