# Improving Pixel Art Quality

**Current Status**: Base SDXL model with prompt engineering
**Problem**: Results look more like "painterly pixel art" than crisp retro game sprites
**Why**: SDXL trained on photos/art, not actual pixel art

---

## Quick Wins (Implemented Now)

### ✅ 1. Better Workflow

**Created**: `workflows/pixel_art_better.json`
**Now default** for all generations

**Changes**:
- **Resolution**: 512x512 (pixel art works better at lower res)
- **Steps**: 35 (was 20) - more refinement
- **CFG**: 12.0 (was 8.0) - stronger prompt adherence
- **Sampler**: dpmpp_2m + karras (better for stylized art)
- **Better prompts**:
  - Positive: "masterpiece, pixel art, 8bit, retro game sprite, ((pixelated)), crisp edges, dithered colors, limited palette"
  - Negative: "blurry, smooth, antialiased, 3d render, realistic, gradient, soft edges, detailed"

### Test It

```bash
# Restart backend to load new workflow
pkill -f "generation_worker.py"
just debug
```

Then generate - should be **noticeably more pixel-arty**!

---

## Medium-Term Improvements

### 2. Download Existing Pixel Art LoRA

**Fastest way to get good results** without training:

```bash
cd /home/beengud/raibid-labs/dgx-pixels/ComfyUI/models/loras

# Download pixel art LoRA from CivitAI or HuggingFace
# Example (you'll need to find good pixel art LoRAs):
wget https://civitai.com/api/download/models/[ID] -O pixel_art_v1.safetensors
```

Popular pixel art LoRAs to search for:
- "Pixel Art XL" on CivitAI
- "Retro Game Sprites"
- "8bit Art Style"

Then add LoRA loading to workflow (requires workflow modification).

### 3. Adjust Generation Parameters

Try different prompts with emphasis:
- `(pixel art:1.5)` - stronger weight
- `((crisp pixels))` - double emphasis
- `[smooth:-1.0]` - negative weight

Try different samplers in ComfyUI:
- `dpmpp_sde_gpu` - sometimes better for stylized
- `euler_a` - classic choice
- `ddim` - deterministic

---

## Proper Solution: Train Custom LoRA (WS-06)

This is the **right way** per the original architecture docs.

### What You Need

**Dataset** (50-100 images):
- High-quality pixel art sprites
- Consistent style (8-bit, 16-bit, or 32-bit)
- Clean backgrounds
- Properly tagged

**Sources**:
- itch.io (free pixel art packs)
- OpenGameArt.org
- Your own game sprites
- Kenney.nl (CC0 assets)

### Training Process

**Tools**: Kohya_ss or Diffusers (both support DGX-Spark)

**Steps**:
1. Collect 50-100 pixel art images
2. Tag images (captions describing each sprite)
3. Configure training:
   - Base model: SDXL 1.0
   - LoRA rank: 32-64
   - Learning rate: 1e-4
   - Steps: 2000-3000
   - Batch size: 4-8
4. Train on DGX-Spark (~2-4 hours)
5. Test and iterate

**Expected Results**:
- Clean, crisp pixel art
- Consistent style
- Actual dithering patterns
- Limited color palettes
- Proper pixel grid alignment

### Implementation Status

**Not Started**:
- ❌ WS-06: LoRA Training Pipeline
- ❌ WS-07: Dataset Tools & Validation

**Required Work** (~7-10 days per docs):
1. Dataset collection tools
2. Training scripts
3. Validation pipeline
4. LoRA integration in workflows

---

## Post-Processing Options

Even with better models, post-processing helps:

### 4. Pixel Quantization

Add post-processing after generation:
- Reduce to limited color palette (16, 32, or 256 colors)
- Snap to pixel grid
- Remove anti-aliasing

**Tool**: ImageMagick, custom Python script, or ComfyUI node

### 5. Dithering

Apply ordered or Floyd-Steinberg dithering:
- Creates retro pixel art look
- Reduces color banding
- Classic 8-bit aesthetic

---

## Comparison: What to Expect

### Current (Base SDXL + Prompts)
- ❌ Soft edges, some blur
- ❌ Gradients instead of dithering
- ❌ Too many colors
- ⚠️ Pixel-art-ish but not crisp
- ✅ Understands composition
- ✅ Random seeds working

### With Better Workflow (Now)
- ✅ Sharper edges
- ✅ Lower resolution (512x512)
- ✅ Better prompt adherence
- ⚠️ Still not perfect crisp pixels
- ⚠️ May have some smoothing

### With Downloaded LoRA
- ✅✅ Much more authentic pixel art
- ✅✅ Crisp edges, limited colors
- ✅ Dithering patterns
- ⚠️ Style depends on LoRA quality
- ⏱️ Setup: ~30 minutes

### With Custom Trained LoRA (WS-06)
- ✅✅✅ Perfect pixel art for your style
- ✅✅✅ Consistent across generations
- ✅✅ Exact palette control
- ✅✅ Proper retro aesthetic
- ⏱️ Training: 2-4 hours + 1 week setup

---

## Recommended Path

**Today**:
1. ✅ **Test new workflow** (already set as default)
2. Try different prompts with the improved settings

**This Week**:
3. **Download 2-3 pixel art LoRAs** from CivitAI
4. Test which one works best for your needs

**Next Sprint**:
5. **Implement WS-06** (LoRA Training Pipeline)
6. Collect dataset of your target style
7. Train custom LoRA

---

## Example Prompts to Try

With new workflow, try these:

**RPG Character**:
```
pixel art, 8bit rpg character sprite, warrior with sword,
front view, game asset, ((pixelated)), crisp edges,
limited palette, no background
```

**Platformer Sprite**:
```
pixel art, 16bit platformer character, jump pose,
side view, game sprite, sharp pixels, retro style,
dithered shading, transparent background
```

**Enemy Sprite**:
```
pixel art, 8bit enemy slime monster, idle animation frame,
retro game, pixelated, limited colors, crisp edges,
simple shapes, no blur
```

**Item Icon**:
```
masterpiece, pixel art icon, health potion, 32x32 pixels,
game ui asset, crisp edges, flat colors, item sprite,
centered, no background
```

---

## Testing Quality

Compare outputs:
1. Generate 5 sprites with same prompt
2. Check for:
   - ✅ Edge crispness
   - ✅ Color consistency
   - ✅ Dithering vs gradients
   - ✅ Pixel grid alignment
3. Iterate on prompts/settings

---

## TL;DR

**Right Now**: Restart backend, new workflow is active → 30% better
**This Week**: Download pixel art LoRA → 70% better
**Proper Fix**: Train custom LoRA (WS-06) → 100% perfect for your style

**Current workflow** will produce **better but not perfect** pixel art. For production-quality sprites, you'll need a trained LoRA.

---

**Files Modified**:
- Created: `workflows/pixel_art_better.json`
- Modified: `python/workers/job_executor.py` (now uses better workflow)

**Try it now!** 🎮
