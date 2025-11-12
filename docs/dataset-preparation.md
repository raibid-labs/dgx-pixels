# Dataset Preparation Guide

Complete guide to preparing high-quality training datasets for LoRA fine-tuning.

## Overview

Quality datasets are essential for training effective LoRA models. This guide covers:
- Dataset requirements and best practices
- Image collection and curation
- Preprocessing and quality control
- Caption creation and refinement
- Validation and troubleshooting

---

## Dataset Requirements

### Minimum Requirements

| Requirement | Value | Notes |
|-------------|-------|-------|
| **Images** | 50+ | Minimum for style training, 100+ recommended |
| **Resolution** | 512×512+ | Will be resized to 1024×1024 for SDXL |
| **Format** | PNG, JPG | PNG preferred for pixel art |
| **Quality** | Clean pixels | No blur, compression artifacts, or dithering |
| **Style** | Consistent | All images should match target style |
| **Captions** | One per image | Text description for each image |

### Ideal Dataset

- **Size**: 100-200 images
- **Variety**: Different subjects (characters, items, environments)
- **Consistency**: Same art style, color palette, pixel density
- **Quality**: Sharp, clean, professional pixel art
- **Captions**: Detailed, accurate descriptions

---

## Dataset Structure

```
datasets/my_game_style/
├── character_knight_001.png
├── character_knight_001.txt
├── character_knight_002.png
├── character_knight_002.txt
├── item_potion_red.png
├── item_potion_red.txt
├── environment_dungeon_floor.png
├── environment_dungeon_floor.txt
└── ...
```

**Naming Convention:**
- Descriptive filenames (not `img001.png`)
- Match image and caption files (same name, different extension)
- Use lowercase with underscores
- Group by category (character_, item_, environment_)

---

## Collecting Training Data

### Option 1: Use Existing Game Art

If you have existing pixel art:

```bash
# Create dataset directory
mkdir -p datasets/my_game

# Copy existing sprites
cp ~/game_project/assets/sprites/*.png datasets/my_game/

# Copy existing captions if available
cp ~/game_project/assets/sprites/*.txt datasets/my_game/
```

**Pros:**
- Already matches your style
- No cost
- Immediate start

**Cons:**
- May have limited variety
- Existing art may need cleanup
- May not have captions

### Option 2: Curate from Open Sources

Public domain and CC0 pixel art sources:

**Websites:**
- [OpenGameArt.org](https://opengameart.org/) - Vast collection, check licenses
- [itch.io](https://itch.io/game-assets/free) - Free game assets
- [Kenney.nl](https://kenney.nl/) - High-quality CC0 assets
- [Pixabay](https://pixabay.com/) - Some pixel art available

**Search Terms:**
- "pixel art sprites"
- "16-bit game assets"
- "retro game sprites"
- "pixel art tileset"

**Filtering Criteria:**
1. Consistent art style
2. Similar pixel density (16×16, 32×32, 64×64)
3. Matching color palette
4. Clean pixels (no anti-aliasing)
5. Compatible license

**Pros:**
- Quick to start
- Large variety available
- Often pre-categorized

**Cons:**
- Mixed styles
- Variable quality
- License restrictions
- Time to curate

### Option 3: Commission Reference Art

Hire a pixel artist to create reference sprites:

**Scope of Work:**
- 50-100 pixel art sprites
- Consistent style and palette
- Variety of subjects:
  - 20-30 characters (poses, angles)
  - 20-30 items (weapons, potions)
  - 20-30 environment elements (tiles, objects)

**Budget:**
- $5-20 per sprite
- Total: $500-2000

**Timeline:**
- 2-4 weeks

**Pros:**
- Perfect style match
- Designed for your needs
- Consistent quality
- Variety as needed

**Cons:**
- Cost
- Time to commission
- Requires clear art direction

### Option 4: Generate Initial Dataset

Use pre-trained models to generate starting dataset:

```bash
# Generate base sprites with existing SDXL model
python3 python/workers/generation_worker.py \
  --prompt "pixel art knight, 16-bit rpg style" \
  --batch 50

# Manually curate best results
# Use curated images as training data
```

**Pros:**
- Fast generation
- Can create variety
- Good for bootstrapping

**Cons:**
- May lack consistency
- Requires manual curation
- Quality depends on prompts
- Circular (AI training AI)

**Recommendation**: Combine approaches - use commissioned art as foundation (30-50 images) and supplement with curated open source assets (20-50 images).

---

## Image Preprocessing

### Automated Preprocessing

Use the dataset preparator:

```bash
python3 python/training/dataset_prep.py \
  --input ./raw_images \
  --output ./datasets/prepared \
  --resolution 1024 \
  --create-captions
```

**What it does:**
1. Finds all images in input directory
2. Validates image quality
3. Resizes to 1024×1024 (SDXL requirement)
4. Pads rectangular images to square
5. Converts to PNG format
6. Creates default captions (if requested)
7. Generates dataset statistics

### Manual Preprocessing Steps

If preprocessing manually:

**1. Clean Up Images**
```python
from PIL import Image

# Remove background
img = Image.open("sprite.png").convert("RGBA")

# Check for transparency
has_alpha = img.mode == "RGBA"
```

**2. Resize to Target Resolution**
```python
# For SDXL, use 1024×1024
target_size = (1024, 1024)

# Pad to square first
max_dim = max(img.size)
square_img = Image.new('RGB', (max_dim, max_dim), (255, 255, 255))
offset = ((max_dim - img.size[0]) // 2, (max_dim - img.size[1]) // 2)
square_img.paste(img, offset)

# Resize
resized_img = square_img.resize(target_size, Image.LANCZOS)
resized_img.save("prepared/sprite.png")
```

**3. Quality Check**
- No blur or anti-aliasing
- Sharp pixel edges
- Consistent pixel density
- No compression artifacts
- Proper alpha channel handling

### Dataset Validation

Validate prepared dataset:

```bash
python3 python/training/dataset_prep.py \
  --input ./datasets/prepared \
  --validate-only
```

**Checks:**
- All images have captions
- Images meet resolution threshold
- No corrupted files
- Captions are not empty
- Caption length is reasonable (20-200 chars)

---

## Caption Creation

### What Makes a Good Caption

**Components:**
1. **Medium**: "pixel art"
2. **Subject**: "knight character", "health potion", "dungeon floor"
3. **Details**: "standing pose", "red liquid", "stone texture"
4. **Angle/View**: "front view", "side view", "top-down"
5. **Style**: "16-bit", "game sprite", "rpg style"

**Formula:**
```
pixel art, [subject], [details], [angle], [style]
```

**Examples:**

**Character:**
```
pixel art knight character, standing pose, front view, full body, medieval armor, holding sword, 16-bit rpg style
```

**Item:**
```
pixel art health potion, red liquid, glass bottle, glowing effect, game inventory item, 16-bit style
```

**Environment:**
```
pixel art dungeon floor tile, stone texture, cracks and moss, top-down view, seamless tileable, 16-bit rpg
```

**Animation:**
```
pixel art wizard character, casting spell animation, magical particles, side view, frame 1 of 4, 16-bit style
```

### Auto-Captioning

Use BLIP-2 for automatic caption generation:

```bash
# Generate captions for all images
python3 python/training/captioning.py \
  --dataset ./datasets/my_style \
  --mode caption \
  --prefix "pixel art, " \
  --suffix ", game sprite, 16-bit style"
```

**Auto-captioning process:**
1. Loads BLIP-2 image captioning model
2. Analyzes each image
3. Generates base description
4. Adds pixel art context (prefix/suffix)
5. Saves to `.txt` file

**Example output:**
```
Input: knight_sprite.png
Auto-caption: "a person holding a sword"
Final: "pixel art, a person holding a sword, game sprite, 16-bit style"
```

### Manual Caption Refinement

Auto-generated captions should be reviewed:

```bash
# Interactive caption editing
python3 python/training/captioning.py \
  --dataset ./datasets/my_style \
  --mode edit
```

**Refinement checklist:**
- [ ] Subject accurately described
- [ ] Important details mentioned
- [ ] Style keywords included
- [ ] Angle/pose specified (if relevant)
- [ ] Consistent terminology across dataset
- [ ] Length: 20-200 characters
- [ ] No typos or grammatical errors

### Caption Best Practices

**DO:**
- Be specific: "red health potion" not "potion"
- Include style: "pixel art", "16-bit", "game sprite"
- Describe pose/angle: "front view", "standing pose"
- Use consistent terms: Always "character" or always "sprite"
- Mention important features: colors, equipment, effects

**DON'T:**
- Be too generic: "pixel art sprite" (too vague)
- Over-describe: Long, rambling sentences
- Include subjective opinions: "beautiful", "amazing"
- Use ambiguous terms: "thing", "stuff"
- Duplicate exact same caption multiple times

### Validation

Validate caption quality:

```bash
python3 python/training/captioning.py \
  --dataset ./datasets/my_style \
  --mode validate
```

**Checks:**
- All images have captions
- No empty captions
- Caption length in range (20-200 chars)
- No special characters or formatting

---

## Dataset Organization Strategies

### Strategy 1: Single Unified Dataset

All training images in one directory:

```
datasets/game_style/
├── knight_001.png
├── knight_002.png
├── potion_red.png
├── dungeon_floor.png
└── ...
```

**Pros:**
- Simple structure
- Easy to manage
- Good for general style training

**Cons:**
- Hard to track categories
- Can't train specialized models easily

**Use for**: General style LoRA

### Strategy 2: Categorized Dataset

Organize by subject type:

```
datasets/
├── characters/
│   ├── knight_*.png
│   └── mage_*.png
├── items/
│   ├── potion_*.png
│   └── sword_*.png
└── environments/
    ├── dungeon_*.png
    └── forest_*.png
```

**Pros:**
- Easy to train specialized LoRAs
- Clear organization
- Can mix categories as needed

**Cons:**
- More complex to manage
- Need to flatten for training

**Use for**: Multiple specialized LoRAs

### Strategy 3: Hierarchical Dataset

Organize by style → category → variant:

```
datasets/
├── 16bit_rpg/
│   ├── characters/
│   └── items/
└── 8bit_retro/
    ├── characters/
    └── items/
```

**Pros:**
- Supports multiple styles
- Very organized
- Scalable

**Cons:**
- Complex structure
- Requires careful management

**Use for**: Large, multi-style projects

---

## Quality Control

### Image Quality Checklist

For each image in your dataset:

- [ ] **Resolution**: At least 512×512 pixels
- [ ] **Sharpness**: Clean pixel edges, no blur
- [ ] **Style**: Matches target art style
- [ ] **Color**: Consistent palette
- [ ] **Background**: Clean or transparent
- [ ] **Artifacts**: No compression artifacts, dithering
- [ ] **Format**: PNG or high-quality JPG

### Caption Quality Checklist

For each caption:

- [ ] **Accuracy**: Correctly describes image
- [ ] **Specificity**: Detailed enough to be useful
- [ ] **Consistency**: Uses same terminology as other captions
- [ ] **Style keywords**: Includes "pixel art", style terms
- [ ] **Length**: 20-200 characters
- [ ] **Grammar**: No typos or errors

### Dataset Balance

Check dataset composition:

```bash
# Get dataset statistics
python3 python/training/dataset_prep.py \
  --input ./datasets/my_style \
  --validate-only
```

**Ideal balance** (for general style LoRA):
- 30-40% characters
- 30-40% items/objects
- 20-30% environments/tiles
- Variety of angles, poses, colors

**Warning signs:**
- >70% one category (may bias model)
- All images similar (lack variety)
- Mixed art styles (inconsistent)
- Low image count (<30 images)

---

## Common Issues and Solutions

### Issue: Dataset Too Small

**Problem**: Only 20-30 images available

**Solutions:**
1. **Generate synthetic data**: Use pre-trained models to create more examples
2. **Data augmentation**: Flip images horizontally (if symmetric)
3. **Lower expectations**: Results may be less consistent
4. **Commission more art**: Hire artist for 20-30 more sprites

### Issue: Inconsistent Style

**Problem**: Mixed art styles in dataset

**Solutions:**
1. **Remove outliers**: Curate dataset to single style
2. **Separate datasets**: Train different LoRAs for different styles
3. **Style guide**: Create strict style criteria for inclusion
4. **Post-process**: Edit images to match style

### Issue: Poor Caption Quality

**Problem**: Generic or inaccurate captions

**Solutions:**
1. **Manual review**: Edit auto-generated captions
2. **Template captions**: Use consistent format
3. **Reference good examples**: Copy structure from well-captioned datasets
4. **Iterate**: Improve captions based on training results

### Issue: Low Resolution Source Images

**Problem**: Images smaller than 512×512

**Solutions:**
1. **Upscale with nearest-neighbor**: Preserves pixel art look
```python
img_upscaled = img.resize((1024, 1024), Image.NEAREST)
```
2. **Accept quality trade-off**: May work but less ideal
3. **Commission higher-res versions**: If budget allows
4. **Remove from dataset**: If quality too poor

---

## Dataset Statistics

After preparation, check statistics:

```json
{
  "total_images": 75,
  "images_with_captions": 75,
  "avg_caption_length": 68.5,
  "min_resolution": [512, 512],
  "max_resolution": [2048, 2048],
  "total_size_mb": 45.2,
  "formats": {
    "PNG": 70,
    "JPG": 5
  }
}
```

**Interpretation:**
- **Total images**: 75 is good (50+ required)
- **All captioned**: Perfect
- **Avg caption length**: 68 chars is good (target 50-100)
- **Resolution range**: Good variety
- **Mostly PNG**: Ideal for pixel art

---

## Best Practices Summary

1. **Quality over quantity**: 50 good images > 200 mixed quality
2. **Consistent style**: All images should match target aesthetic
3. **Detailed captions**: Specific descriptions train better models
4. **Clean preprocessing**: Sharp pixels, proper resizing
5. **Validate early**: Check dataset quality before training
6. **Iterate**: Refine dataset based on training results
7. **Document**: Keep notes on dataset composition and decisions

---

## Next Steps

After dataset preparation:

1. **Validate dataset**: Run quality checks
2. **Review samples**: Manually inspect random images
3. **Check captions**: Read through caption files
4. **Start training**: Begin with fast config for testing
5. **Evaluate results**: Assess if dataset needs improvement
6. **Iterate**: Refine dataset and retrain

For training instructions, see `docs/lora-training-guide.md`.

---

**Time Estimate:**
- Dataset collection: 2-8 hours (depending on source)
- Image preprocessing: 1-2 hours
- Caption creation: 2-4 hours
- Quality control: 1-2 hours
- Total: 1-2 days for first dataset

**Recommended**: Start with smaller dataset (30-50 images), train and evaluate, then expand based on results.
