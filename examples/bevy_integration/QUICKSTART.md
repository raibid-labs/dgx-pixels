# DGX-Pixels Bevy Integration - Quick Start

**Get started in 5 minutes!**

## Prerequisites

- Rust 1.75+ (`rustc --version`)
- Cargo (comes with Rust)
- X11 display or Xvfb for headless systems

## Installation

```bash
# 1. Navigate to example
cd /home/beengud/raibid-labs/dgx-pixels/examples/bevy_integration

# 2. Build project
cargo build --release

# 3. Run example
cargo run --release
```

## Controls

- **WASD** - Move player
- **ESC** - Quit game

## What You'll See

- Blue square player sprite (placeholder)
- Player moves with WASD controls
- Smooth frame-rate independent movement

## Next Steps

### 1. Create Placeholder Graphics

The example uses placeholder sprites. Create simple 32x32 PNGs:

**Option A: Manual (Recommended)**
1. Open any image editor (GIMP, Photoshop, Paint)
2. Create 32x32 pixel image
3. Fill with color (blue for player)
4. Save as `assets/placeholder/player.png`

**Option B: Python Script**
```bash
pip install pillow
python3 create_placeholders.py
```

**Option C: ImageMagick**
```bash
convert -size 32x32 xc:blue PNG32:assets/placeholder/player.png
```

### 2. Understand MCP Integration

The example shows how to integrate with DGX-Pixels MCP server:

```rust
// Request sprite generation
let sprite_path = mcp_client.generate_sprite_sync(GenerateSpriteParams {
    prompt: "pixel art knight".to_string(),
    style: "pixel_art".to_string(),
    resolution: "1024x1024".to_string(),
}).expect("Failed to generate sprite");

// Load generated sprite
let texture = asset_server.load(sprite_path);

// Use in game
commands.spawn(SpriteBundle { texture, ..default() });
```

### 3. Generate Your First Sprite

Edit `prompts/sprite_prompts.yaml` to customize sprite prompts:

```yaml
player:
  prompt: "YOUR CUSTOM PROMPT HERE"
  style: "pixel_art"
  resolution: "1024x1024"
```

(Note: Full MCP integration requires WS-15 implementation)

### 4. Read Full Documentation

See `README.md` for:
- Complete integration guide
- Batch generation examples
- Troubleshooting solutions
- Advanced usage patterns

## Project Structure

```
examples/bevy_integration/
├── src/
│   ├── main.rs              # Game entry point
│   ├── mcp_client.rs        # MCP integration (start here!)
│   ├── sprite_manager.rs    # Asset caching
│   └── game/
│       ├── player.rs        # Player controls
│       ├── enemies.rs       # Enemy generation
│       └── level.rs         # Tileset generation
├── assets/
│   ├── sprites/             # Generated sprites go here
│   ├── tiles/               # Generated tiles go here
│   └── placeholder/         # Temporary placeholders
├── prompts/
│   └── sprite_prompts.yaml  # Sprite generation prompts
└── README.md                # Full documentation

```

## Common Issues

**Build fails with ALSA error**
→ Already fixed! We use minimal Bevy features without audio.

**"Error loading placeholder/player.png"**
→ Create placeholder graphics (see step 1 above).

**Game doesn't run on headless system**
→ Use `xvfb-run cargo run` or focus on API integration.

**MCP generation returns placeholder path**
→ Expected! Real MCP integration coming in WS-15.

## Integration Examples

### Example 1: Basic Sprite Generation

```rust
fn generate_player_sprite(
    mcp: Res<McpClient>,
    asset_server: Res<AssetServer>,
) {
    let sprite_path = mcp.generate_sprite_sync(GenerateSpriteParams {
        prompt: "pixel art warrior, 32x32, blue armor".to_string(),
        style: "pixel_art".to_string(),
        resolution: "1024x1024".to_string(),
    }).expect("Failed to generate");

    let texture = asset_server.load(sprite_path);
    // Use texture...
}
```

### Example 2: Batch Enemy Generation

```rust
fn generate_enemy_batch(mcp: Res<McpClient>) {
    let prompts = vec![
        GenerateSpriteParams {
            prompt: "pixel art goblin".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
        // ... more enemies
    ];

    let sprite_paths = mcp.generate_batch_sync(prompts)
        .expect("Failed to generate batch");

    // Load and spawn enemies...
}
```

### Example 3: Cached Sprite Loading

```rust
fn load_sprite(
    mut sprite_manager: ResMut<SpriteManager>,
    asset_server: Res<AssetServer>,
) {
    let handle = sprite_manager.load_or_get(
        "player_idle",
        "sprites/player_idle.png",
        &asset_server
    );

    // Handle is cached - second call returns same handle instantly
}
```

## Performance Tips

1. **Batch generation**: Generate multiple sprites in one request (faster)
2. **Use sprite manager**: Prevents redundant asset loads
3. **Pre-generate**: Generate sprites at startup or in loading screens
4. **Resolution**: Start with 512x512, upscale to 1024x1024 if needed

## Architecture

```
Bevy Game (Rust)
    ↓
McpClient (this example)
    ↓
FastMCP Server (WS-13)
    ↓
Python Worker (WS-10)
    ↓
ComfyUI + SDXL
```

## Resources

- **Full Guide**: `README.md` (350+ lines)
- **MCP Client**: `src/mcp_client.rs` (85 lines)
- **Sprite Manager**: `src/sprite_manager.rs` (90 lines)
- **Player Example**: `src/game/player.rs` (120 lines)

## Help

**Documentation**: See `README.md`
**Troubleshooting**: See `README.md` § Troubleshooting
**Issues**: See main project `CONTRIBUTING.md`

---

**Ready to build AI-powered game sprites!** 🎮
