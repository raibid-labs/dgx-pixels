# Asset Naming and Format Conventions

This document defines the naming conventions and format standards for DGX-Pixels generated sprites.

## Naming Convention

### Overview

All assets follow a structured naming convention that enables:
- Automatic metadata extraction
- Organized asset management
- Easy searching and filtering
- Animation sequence grouping

### Pattern

```
{category}_{name}_{variant}_{frame}.{extension}
```

### Components

#### 1. Category (Required)

The asset type. Valid categories:

| Category | Description | Examples |
|----------|-------------|----------|
| `character` | Player characters, NPCs | knight, wizard, archer |
| `enemy` | Enemy sprites | goblin, skeleton, dragon |
| `npc` | Non-player characters | merchant, villager, guard |
| `item` | Collectible items | potion, coin, key |
| `tile` | Environment tiles | grass, stone, water |
| `ui` | User interface elements | button, bar, icon |
| `effect` | Visual effects | explosion, sparkle, smoke |
| `projectile` | Projectiles and bullets | arrow, fireball, bullet |
| `environment` | Background objects | tree, rock, cloud |
| `background` | Background layers | sky, mountains, forest |

**Rules**:
- Lowercase only
- Single word
- Must be from valid categories list
- Can be extended via config

#### 2. Name (Required)

Specific identifier for the asset:

**Rules**:
- Lowercase only
- Alphanumeric characters
- Use letters and numbers only (no special characters)
- Be descriptive but concise

**Examples**:
- `knight`, `wizard`, `archer`
- `goblin`, `skeleton`, `dragon`
- `grass`, `stone`, `water`
- `button`, `bar`, `icon`

#### 3. Variant (Optional)

State, style, or variation:

**Common Variants**:

**Animation States**:
- `idle`, `walk`, `run`, `jump`, `fall`
- `attack`, `defend`, `hit`, `death`
- `cast`, `shoot`, `crouch`, `climb`

**UI States**:
- `normal`, `hover`, `pressed`, `disabled`
- `empty`, `half`, `full`

**Styles**:
- `red`, `blue`, `green`
- `small`, `medium`, `large`
- `left`, `right`, `up`, `down`

**Rules**:
- Lowercase only
- Alphanumeric characters
- Optional (can be omitted for single-state assets)

#### 4. Frame (Optional)

Frame number for animations:

**Rules**:
- 3-4 digits
- Zero-padded
- Sequential numbering
- Starts at 0001 (not 0000)

**Formats**:
- `0001`, `0002`, ..., `0099` (3-digit for ≤99 frames)
- `0001`, `0002`, ..., `9999` (4-digit for >99 frames)

**Examples**:
- Single frame: omit frame number
- 4-frame animation: `0001`, `0002`, `0003`, `0004`
- 12-frame animation: `0001` through `0012`

#### 5. Extension (Required)

File format extension:

**Rules**:
- Lowercase only
- Must be `.png`

## Complete Examples

### Character Sprites

```
character_knight_idle_0001.png
character_knight_idle_0002.png
character_knight_idle_0003.png
character_knight_idle_0004.png

character_knight_walk_0001.png
character_knight_walk_0002.png
character_knight_walk_0003.png
character_knight_walk_0004.png

character_knight_attack_0001.png
character_knight_attack_0002.png
character_knight_attack_0003.png

character_wizard_idle_0001.png
character_wizard_cast_0001.png
```

### Enemy Sprites

```
enemy_goblin_idle_0001.png
enemy_goblin_walk_0001.png
enemy_goblin_attack_0001.png

enemy_skeleton_idle_0001.png
enemy_skeleton_attack_0001.png

enemy_dragon_idle_0001.png
enemy_dragon_fly_0001.png
enemy_dragon_breath_0001.png
```

### Tiles

```
tile_grass_001.png
tile_grass_002.png

tile_stone_wall_001.png
tile_stone_floor_001.png

tile_water_001.png
tile_water_002.png
tile_water_003.png
tile_water_004.png
```

### UI Elements

```
ui_button_normal.png
ui_button_hover.png
ui_button_pressed.png
ui_button_disabled.png

ui_health_bar_empty.png
ui_health_bar_half.png
ui_health_bar_full.png

ui_icon_sword.png
ui_icon_shield.png
ui_icon_potion.png
```

### Items

```
item_potion_health.png
item_potion_mana.png

item_coin_gold.png
item_coin_silver.png

item_key_red.png
item_key_blue.png
```

### Effects

```
effect_explosion_0001.png
effect_explosion_0002.png
effect_explosion_0003.png

effect_sparkle_0001.png
effect_smoke_0001.png
```

### Projectiles

```
projectile_arrow_0001.png
projectile_fireball_0001.png
projectile_bullet_0001.png
```

### Single-Frame Assets

For non-animated assets, omit variant and frame:

```
character_knight.png
enemy_goblin.png
tile_grass.png
item_potion.png
ui_button.png
```

Or include variant but omit frame:

```
ui_button_normal.png
ui_button_hover.png
item_potion_health.png
```

## Format Standards

### File Format

**Required**: PNG (Portable Network Graphics)

**Why PNG**:
- Lossless compression
- Alpha channel support (transparency)
- Wide compatibility
- Industry standard for game sprites

**Not Allowed**:
- JPEG (lossy, no transparency)
- BMP (uncompressed, large files)
- GIF (limited colors, animation format)
- WebP (limited game engine support)

### Color Mode

**Allowed**:
- **RGBA** (RGB + Alpha): Sprites with transparency (recommended)
- **RGB**: Opaque sprites (no transparency)
- **L** (Grayscale): Masks or single-channel data

**Not Allowed**:
- CMYK (print color space)
- Indexed color (limited palette, use post-processing instead)

### Resolution

**Recommended**: Power-of-2 dimensions

**Common Sizes**:
- **16x16**: Tiny items, UI icons
- **32x32**: Small sprites, tiles
- **64x64**: Standard character sprites
- **128x128**: Large character sprites
- **256x256**: Boss sprites, large objects
- **512x512**: Detailed sprites, background elements
- **1024x1024**: High-detail sprites (SDXL generation size)
- **2048x2048**: Maximum recommended size

**Rules**:
- Minimum: 16x16 pixels
- Maximum: 4096x4096 pixels
- Power-of-2 recommended but not required
- Non-square allowed (e.g., 64x32 for wide sprites)

**Why Power-of-2**:
- Better GPU texture cache utilization
- Mipmapping support
- Atlas packing efficiency
- Industry best practice

### File Size

**Maximum**: 10 MB per file

**Typical Sizes**:
- 32x32 sprite: ~5-20 KB
- 64x64 sprite: ~10-40 KB
- 128x128 sprite: ~20-80 KB
- 1024x1024 sprite: ~100-500 KB

**Optimization**:
- Use PNG compression (level 6-9)
- Remove unnecessary metadata
- Apply color quantization if appropriate
- Consider post-processing pipeline

### Bit Depth

**Recommended**:
- **8-bit per channel** (24-bit RGB or 32-bit RGBA)
- Standard for game sprites
- Good balance of quality and size

**Allowed**:
- 16-bit per channel (HDR sprites, special cases)

**Not Recommended**:
- 1-bit (black and white only)
- 4-bit (limited colors)

## Metadata Standards

### Image Metadata

**Remove** before deployment:
- Camera metadata (EXIF)
- Author information
- GPS data
- Timestamps
- Software tags

**Keep**:
- Color profile (sRGB recommended)
- Pixel density (72 or 96 DPI standard)

### Manifest Metadata

Automatically extracted from filename and file:

```json
{
  "name": "character_knight_idle",
  "path": "sprites/character_knight_idle_0001.png",
  "category": "character",
  "variant": "idle",
  "frames": 4,
  "resolution": [64, 64],
  "file_size_kb": 12.5
}
```

## Validation

The deployment pipeline validates all conventions:

### Automatic Validation

Run validation before deployment:

```bash
./scripts/validate_assets.sh outputs/
```

### Validation Checks

1. **File Format**: Must be PNG
2. **Color Mode**: RGB, RGBA, or L
3. **Naming Convention**: Matches pattern
4. **Resolution**: Within min/max bounds
5. **File Size**: <10 MB
6. **Power-of-2**: Warning if not power-of-2 (not error)

### Enforcement Levels

**Errors** (block deployment):
- Wrong file format
- Invalid color mode
- File too large
- Invalid naming (if enforcement enabled)

**Warnings** (allow deployment):
- Non-power-of-2 resolution
- Non-square dimensions
- Missing metadata

## Custom Categories

Add custom categories in `config/deployment_config.yaml`:

```yaml
naming:
  categories:
    - character
    - enemy
    - vehicle      # Custom category
    - weapon       # Custom category
    - particle     # Custom category
```

Example usage:

```
vehicle_car_idle_0001.png
weapon_sword_swing_0001.png
particle_dust_0001.png
```

## Animation Naming

### Frame Sequences

Group animation frames by base name:

```
character_knight_walk_0001.png
character_knight_walk_0002.png
character_knight_walk_0003.png
character_knight_walk_0004.png
```

Base name: `character_knight_walk`
Frames: 4

### Multiple Animations

Different animations for same sprite:

```
character_knight_idle_0001.png to 0004.png
character_knight_walk_0001.png to 0008.png
character_knight_attack_0001.png to 0006.png
```

### Frame Rate Convention

Filename doesn't encode frame rate. Specify in game code:

```rust
// Bevy animation example
AnimationTimer::from_seconds(0.1, TimerMode::Repeating)  // 10 FPS
AnimationTimer::from_seconds(0.05, TimerMode::Repeating) // 20 FPS
```

## Tile Naming

### Tileset Elements

```
tile_grass_001.png
tile_grass_002.png
tile_grass_003.png
```

### Tileset Variations

```
tile_ground_grass.png
tile_ground_dirt.png
tile_ground_stone.png

tile_wall_brick.png
tile_wall_stone.png
tile_wall_wood.png
```

### Tile States

```
tile_door_closed.png
tile_door_open.png

tile_chest_closed.png
tile_chest_open.png
```

## UI Naming

### Button States

```
ui_button_normal.png
ui_button_hover.png
ui_button_pressed.png
ui_button_disabled.png
```

### Progress Elements

```
ui_health_bar_empty.png
ui_health_bar_quarter.png
ui_health_bar_half.png
ui_health_bar_threequarter.png
ui_health_bar_full.png
```

### Icons

```
ui_icon_health.png
ui_icon_mana.png
ui_icon_stamina.png

ui_icon_sword.png
ui_icon_shield.png
ui_icon_bow.png
```

## Common Patterns

### Directional Sprites

Include direction in variant:

```
character_knight_walk_left_0001.png
character_knight_walk_right_0001.png
character_knight_walk_up_0001.png
character_knight_walk_down_0001.png
```

Or as separate base names:

```
character_knight_walkleft_0001.png
character_knight_walkright_0001.png
character_knight_walkup_0001.png
character_knight_walkdown_0001.png
```

### Color Variations

```
enemy_slime_red_idle_0001.png
enemy_slime_blue_idle_0001.png
enemy_slime_green_idle_0001.png
```

### Size Variations

```
item_potion_small.png
item_potion_medium.png
item_potion_large.png
```

## Anti-Patterns

### Don't Use

**Spaces**:
```
❌ character knight idle.png
✅ character_knight_idle.png
```

**Capital Letters**:
```
❌ Character_Knight_Idle.png
✅ character_knight_idle.png
```

**Special Characters**:
```
❌ character-knight-idle.png
❌ character.knight.idle.png
✅ character_knight_idle.png
```

**Inconsistent Numbering**:
```
❌ sprite_1.png, sprite_2.png, sprite_10.png
✅ sprite_0001.png, sprite_0002.png, sprite_0010.png
```

**Vague Names**:
```
❌ sprite1.png, asset2.png, image3.png
✅ character_knight_idle_0001.png
```

## Migration Guide

### Converting Existing Assets

If you have existing assets with different naming:

1. **Create mapping file**:
```json
{
  "old_name.png": "character_knight_idle_0001.png",
  "sprite2.png": "enemy_goblin_walk_0001.png"
}
```

2. **Rename script**:
```python
import json
import shutil
from pathlib import Path

with open("mapping.json") as f:
    mapping = json.load(f)

for old_name, new_name in mapping.items():
    shutil.move(old_name, new_name)
```

3. **Update game references**:
- Update asset paths in code
- Regenerate manifest
- Test loading

## FAQ

**Q: Can I use uppercase letters?**
A: No, lowercase only for consistency.

**Q: Can I use dashes or dots?**
A: No, use underscores only.

**Q: What if I have more than 9999 frames?**
A: Use 5-digit frame numbers: `00001`, `00002`, etc.

**Q: Can I omit the category?**
A: No, category is required for organization.

**Q: Can I add custom categories?**
A: Yes, add to `config/deployment_config.yaml`.

**Q: Do I need frame numbers for single-frame sprites?**
A: No, omit frame numbers for non-animated assets.

**Q: Can I use non-power-of-2 resolutions?**
A: Yes, but power-of-2 is recommended.

**Q: What resolution should I use?**
A: 64x64 for characters, 32x32 for tiles, 1024x1024 for generation (downscale after).

**Q: Can I use JPEG instead of PNG?**
A: No, PNG required for transparency and quality.

## Resources

- **Validation**: `./scripts/validate_assets.sh`
- **Configuration**: `config/deployment_config.yaml`
- **Deployment Guide**: `docs/deployment/deployment-guide.md`
- **Examples**: `examples/bevy_integration/assets/sprites/`
