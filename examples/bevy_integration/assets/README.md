# Assets Directory

This directory contains all game assets for the Bevy integration example.

## Directory Structure

```
assets/
├── sprites/        # AI-generated character and enemy sprites
├── tiles/          # AI-generated tile sprites for levels
└── placeholder/    # Placeholder graphics (temporary until generated)
```

## Creating Placeholder Graphics

Since this example uses placeholder graphics, you need to create simple PNG files:

### Manual Creation (Recommended)
Use any image editor to create:
- `placeholder/player.png` - 32x32 blue square
- `placeholder/enemy.png` - 32x32 red square
- `placeholder/ground.png` - 32x32 green square

### Using GIMP
1. Open GIMP
2. File → New → 32x32 pixels
3. Fill with color (blue for player, red for enemy, green for ground)
4. File → Export As → PNG
5. Save to `assets/placeholder/`

### Using ImageMagick (if installed)
```bash
convert -size 32x32 xc:blue PNG32:assets/placeholder/player.png
convert -size 32x32 xc:red PNG32:assets/placeholder/enemy.png
convert -size 32x32 xc:green PNG32:assets/placeholder/ground.png
```

### Using Python + Pillow (if installed)
```bash
cd examples/bevy_integration
python3 create_placeholders.py
```

## AI-Generated Sprites

Once the MCP server is running, sprites will be generated to:
- `sprites/` - Character and enemy sprites
- `tiles/` - Level tiles

These directories will be populated automatically via MCP sprite generation.
