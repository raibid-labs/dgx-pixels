#!/usr/bin/env python3
"""Create placeholder sprite images for Bevy integration example."""

from PIL import Image
import os

# Ensure assets directory exists
os.makedirs("assets/placeholder", exist_ok=True)

# Create placeholder images
placeholders = {
    "player.png": (32, 32, (50, 100, 255)),      # Blue player
    "enemy.png": (32, 32, (200, 50, 50)),         # Red enemy
    "ground.png": (32, 32, (100, 200, 100)),      # Green ground
}

for filename, (width, height, color) in placeholders.items():
    img = Image.new('RGB', (width, height), color)
    path = os.path.join("assets", "placeholder", filename)
    img.save(path)
    print(f"Created: {path}")

print("\nPlaceholder sprites created successfully!")
