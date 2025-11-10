# Technology Deep Dive

## Table of Contents
- [Stable Diffusion XL](#stable-diffusion-xl)
- [LoRA Fine-Tuning](#lora-fine-tuning)
- [ComfyUI](#comfyui)
- [NVIDIA DGX-Spark Optimizations](#nvidia-dgx-spark-optimizations)
- [FastAPI and MCP](#fastapi-and-mcp)
- [Sprite Processing](#sprite-processing)

---

## Stable Diffusion XL

### Overview

Stable Diffusion XL (SDXL) is a latent diffusion model for text-to-image generation. It represents a significant improvement over SD 1.5 with:

- **3x larger UNet**: More capacity for learning complex patterns
- **Dual text encoders**: OpenCLIP ViT-bigG/14 + original CLIP
- **Higher resolution**: Native 1024x1024 (vs 512x512 for SD 1.5)
- **Better composition**: Improved understanding of spatial relationships
- **Refined details**: Superior fine detail generation

### Architecture

```
Text Prompt
    │
    ▼
┌───────────────┐
│ Text Encoders │
│ - CLIP ViT-L  │
│ - OpenCLIP    │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ UNet          │
│ (Denoising)   │
│               │
│ Latent Space  │
│ (64x64 for    │
│  1024x1024)   │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ VAE Decoder   │
│ (Upscale 8x)  │
└───────┬───────┘
        │
        ▼
   Final Image
  (1024x1024)
```

### Why SDXL for Pixel Art?

1. **Higher resolution** allows for more detailed sprites
2. **Better prompt understanding** = more accurate generation
3. **Fine-tuning friendly** via LoRA
4. **Large community** with pre-trained pixel art checkpoints
5. **NVIDIA optimization** with Tensor Core support

### Model Sizes and VRAM

| Model | Parameters | VRAM (FP16) | VRAM (FP4) | Inference Time |
|-------|-----------|-------------|------------|----------------|
| SD 1.5 | 860M | ~4GB | ~2GB | ~2s |
| SDXL Base | 2.6B | ~8GB | ~4GB | ~5s |
| SDXL + Refiner | 2.6B + 3.5B | ~12GB | ~6GB | ~8s |

**DGX-Spark Advantage**: 128GB unified memory means we can load multiple models simultaneously and use larger batch sizes.

### Pixel Art Specific Considerations

**Challenges:**
- Diffusion models tend to produce "smooth" images
- Pixel art requires sharp edges and limited palettes
- Small sprite sizes can be problematic

**Solutions:**
- Fine-tune with LoRA on pixel art dataset
- Use "pixel art" trigger words in prompts
- Post-process with color quantization
- Generate at higher resolution, then downscale

### Sample Prompts

```
# Character sprite
"pixelsprite, fantasy knight character,
standing pose, side view, 32x32,
clean background, 16-bit style"

# Environment tile
"16bitscene, dungeon stone floor tile,
top-down view, seamless, dark fantasy"

# Item
"pixel art, health potion, red liquid,
glass bottle, isometric view, game item"
```

---

## LoRA Fine-Tuning

### What is LoRA?

LoRA (Low-Rank Adaptation) is an efficient fine-tuning technique that:

1. **Freezes base model weights** (saves memory)
2. **Trains small adapter matrices** (fast training)
3. **Produces tiny files** (100-500MB vs. 2-6GB for full model)
4. **Combines with base model** at inference time
5. **Multiple LoRAs** can be stacked

### How LoRA Works

```
Original Weight Matrix (W)
        ┌─────────────────┐
        │   Frozen Base   │
        │   Model (SDXL)  │
        └─────────────────┘
                +
        ┌──────────┐
        │ LoRA     │
        │ Adapter  │
        │ A × B    │  ← Only these are trained
        └──────────┘
                ║
                ▼
        Updated Behavior
```

**Math:**
- Original: `W ∈ ℝ^(d×k)`
- LoRA: `ΔW = A × B` where `A ∈ ℝ^(d×r)` and `B ∈ ℝ^(r×k)`
- Rank `r << min(d, k)` (typically 32-128)
- New forward pass: `Y = (W + α·ΔW)X`

### Training Requirements

**Dataset:**
- **Minimum**: 20 images
- **Recommended**: 50-100 images for style training
- **Quality over quantity**: Well-curated > large noisy dataset

**Image Requirements:**
- Resolution: 1024x1024 (for SDXL)
- Format: PNG or JPG
- Total size: < 5GB
- Consistency: Similar style, quality

**Captions:**
Each image needs a text description. Options:
1. **Manual**: Write descriptions
2. **Automated**: Use BLIP, CLIP Interrogator
3. **Hybrid**: Auto-generate, then manually refine

**Example Dataset Structure:**
```
training_data/
  001_knight_walk.png
  001_knight_walk.txt  # "medieval knight character walking animation"
  002_knight_idle.png
  002_knight_idle.txt  # "medieval knight character standing idle"
  ...
```

### Training Hyperparameters

**Recommended Settings:**
```python
# LoRA specific
lora_rank = 64              # 32-128, higher = more capacity
lora_alpha = 64             # Usually same as rank
lora_dropout = 0.1          # Regularization

# Training
learning_rate = 1e-4        # 1e-5 to 5e-4
batch_size = 4              # Adjust based on VRAM
num_epochs = 10             # Or use steps
gradient_accumulation = 2   # Effective batch = 4*2=8

# Optimization
optimizer = "AdamW8bit"     # Memory efficient
lr_scheduler = "cosine"     # Smooth decay
warmup_steps = 100          # Gradual start

# Regularization
max_grad_norm = 1.0         # Gradient clipping
min_snr_gamma = 5           # Noise schedule
```

### Training on DGX-Spark

**Kohya_ss Setup:**
```bash
# Install
git clone https://github.com/bmaltais/kohya_ss.git
cd kohya_ss
./setup.sh

# Configure for DGX-Spark
export CUDA_VISIBLE_DEVICES=0
export PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True
```

**Training Command:**
```bash
accelerate launch --num_cpu_threads_per_process=2 train_network.py \
  --pretrained_model_name_or_path="stabilityai/stable-diffusion-xl-base-1.0" \
  --train_data_dir="./training_data" \
  --output_dir="./output/my_style_lora" \
  --resolution="1024,1024" \
  --train_batch_size=4 \
  --learning_rate=1e-4 \
  --max_train_epochs=10 \
  --save_every_n_epochs=2 \
  --network_module=networks.lora \
  --network_dim=64 \
  --network_alpha=64 \
  --enable_bucket \
  --mixed_precision="fp16" \
  --xformers \
  --gradient_checkpointing
```

**Expected Training Time:**
- 50 images, 3000 steps: ~2-3 hours
- 100 images, 5000 steps: ~4-5 hours
- VRAM usage: ~16-20GB

### Using LoRAs

**In ComfyUI:**
1. Place `.safetensors` file in `models/loras/`
2. Add "Load LoRA" node to workflow
3. Set strength (0.5-1.0 typically)
4. Can stack multiple LoRAs

**In Code (Diffusers):**
```python
from diffusers import DiffusionPipeline

pipe = DiffusionPipeline.from_pretrained(
    "stabilityai/stable-diffusion-xl-base-1.0",
    torch_dtype=torch.float16
)
pipe.to("cuda")

# Load LoRA
pipe.load_lora_weights("./my_style_lora.safetensors")

# Generate
image = pipe(
    "pixelsprite, fantasy warrior",
    num_inference_steps=30,
    guidance_scale=7.5
).images[0]
```

### LoRA Management Strategy

**Organizational Structure:**
```
models/lora/
  styles/
    16bit_retro.safetensors
    32bit_modern.safetensors
    8bit_nes.safetensors
  characters/
    fantasy_heroes.safetensors
    sci_fi_soldiers.safetensors
  environments/
    dungeon_tiles.safetensors
    forest_tiles.safetensors
  items/
    weapons_medieval.safetensors
    potions_vials.safetensors
```

**Metadata File:**
```json
{
  "name": "16bit_retro",
  "version": "1.0",
  "base_model": "SDXL 1.0",
  "trained_on": "2025-01-15",
  "dataset_size": 75,
  "steps": 3000,
  "trigger_words": ["16bitscene", "retro pixel"],
  "recommended_strength": 0.8,
  "description": "Retro 16-bit RPG style",
  "sample_images": ["sample1.png", "sample2.png"]
}
```

---

## ComfyUI

### Architecture

ComfyUI uses a node-based workflow system where each node performs a specific operation:

```
┌────────────┐    ┌─────────────┐    ┌──────────────┐
│ Load       │───▶│ CLIP Text   │───▶│ KSampler     │
│ Checkpoint │    │ Encode      │    │ (Denoising)  │
└────────────┘    └─────────────┘    └──────┬───────┘
                                            │
                  ┌─────────────┐           │
                  │ Load LoRA   │───────────┘
                  └─────────────┘           │
                                            ▼
                  ┌─────────────┐    ┌──────────────┐
                  │ VAE Decode  │◀───│ VAE Encode   │
                  └──────┬──────┘    └──────────────┘
                         │
                         ▼
                  ┌─────────────┐
                  │ Save Image  │
                  └─────────────┘
```

### Why ComfyUI?

**Advantages over A1111:**
1. **Speed**: 2x faster in benchmarks
2. **Memory efficiency**: Better VRAM management
3. **Flexibility**: Node-based allows complex pipelines
4. **Workflows**: Save and share complete pipelines
5. **Custom nodes**: Easy to extend
6. **API**: JSON-based workflow submission

**Performance Comparison:**
| Task | A1111 | ComfyUI | Speedup |
|------|-------|---------|---------|
| 20x 512x512 (SD 1.5) | 2:23 | 1:07 | 2.1x |
| 10x 1024x1024 (SDXL) | 5:45 | 2:50 | 2.0x |

### Workflow JSON Format

Example workflow for sprite generation:

```json
{
  "1": {
    "class_type": "CheckpointLoaderSimple",
    "inputs": {
      "ckpt_name": "pixel_art_xl.safetensors"
    }
  },
  "2": {
    "class_type": "CLIPTextEncode",
    "inputs": {
      "text": "pixelsprite, knight character",
      "clip": ["1", 1]
    }
  },
  "3": {
    "class_type": "KSampler",
    "inputs": {
      "seed": 42,
      "steps": 30,
      "cfg": 7.5,
      "sampler_name": "euler_ancestral",
      "scheduler": "normal",
      "model": ["1", 0],
      "positive": ["2", 0],
      "negative": ["4", 0],
      "latent_image": ["5", 0]
    }
  }
}
```

### Custom Nodes for Sprites

**Sprite Sheet Assembler Node:**
```python
class SpriteSheetAssembler:
    @classmethod
    def INPUT_TYPES(cls):
        return {
            "required": {
                "images": ("IMAGE",),
                "columns": ("INT", {"default": 4, "min": 1, "max": 16}),
                "spacing": ("INT", {"default": 0, "min": 0, "max": 10}),
            }
        }

    RETURN_TYPES = ("IMAGE",)
    FUNCTION = "assemble"
    CATEGORY = "dgx-pixels"

    def assemble(self, images, columns, spacing):
        # Combine multiple images into sprite sheet
        # Implementation details...
        return (assembled_image,)
```

### API Usage

**Submit Workflow:**
```python
import requests
import json

workflow = {
    # ... workflow definition
}

response = requests.post(
    "http://localhost:8188/prompt",
    json={"prompt": workflow}
)

prompt_id = response.json()["prompt_id"]
```

**Check Status:**
```python
response = requests.get(
    f"http://localhost:8188/history/{prompt_id}"
)
result = response.json()
```

**Get Image:**
```python
# Images are saved to output folder
# Or fetch via websocket during generation
```

### Best Practices

1. **Save workflows as templates**: Reuse common patterns
2. **Use queue system**: Don't block on single generations
3. **Monitor VRAM**: Check GPU usage with `nvidia-smi`
4. **Version workflows**: Track changes to generation pipelines
5. **Batch processing**: Use batch nodes for efficiency

---

## NVIDIA DGX-Spark Optimizations

### Hardware Utilization

**Tensor Cores:**
- 5th generation Blackwell Tensor Cores
- Optimized for matrix multiplication
- FP16, TF32, FP8, FP4 support

**Memory Architecture:**
- 128GB unified memory (CPU + GPU)
- LPDDR5x at 273 GB/s
- No PCIe bottleneck for CPU-GPU transfers

### PyTorch Optimization

**1. Mixed Precision Training**
```python
from torch.cuda.amp import autocast, GradScaler

scaler = GradScaler()

for batch in dataloader:
    optimizer.zero_grad()

    with autocast():  # Automatic mixed precision
        output = model(batch)
        loss = criterion(output, target)

    scaler.scale(loss).backward()
    scaler.step(optimizer)
    scaler.update()
```

**2. Torch.compile (PyTorch 2.0+)**
```python
model = torch.compile(model, mode="reduce-overhead")
# ~20% speedup for inference
```

**3. Channels Last Memory Format**
```python
model = model.to(memory_format=torch.channels_last)
input = input.to(memory_format=torch.channels_last)
# Better cache locality for convolutions
```

**4. CUDA Graphs**
```python
# For static models, record execution graph
g = torch.cuda.CUDAGraph()
with torch.cuda.graph(g):
    output = model(static_input)

# Replay is much faster
g.replay()
```

### Diffusers Optimization

**Enable xFormers:**
```python
pipe.enable_xformers_memory_efficient_attention()
# Reduces VRAM and increases speed
```

**Model CPU Offload:**
```python
pipe.enable_model_cpu_offload()
# For very large models that don't fit in VRAM
# (Not usually needed on DGX-Spark)
```

**Attention Slicing:**
```python
pipe.enable_attention_slicing()
# Trade speed for memory (if needed)
```

### Batch Processing

**Optimal Batch Sizes:**
```python
# For SDXL inference on DGX-Spark
batch_sizes = {
    "1024x1024": 8,   # Can fit 8 concurrent
    "512x512": 16,    # Smaller images = larger batches
}

# For training
training_batch_sizes = {
    "lora_sdxl": 4,   # With gradient accumulation
    "full_sdxl": 1,   # Full fine-tuning needs more VRAM
}
```

### Monitoring and Profiling

**GPU Monitoring:**
```bash
# Watch GPU usage
watch -n 1 nvidia-smi

# Detailed stats
nvitop  # Install: pip install nvitop
```

**PyTorch Profiler:**
```python
from torch.profiler import profile, ProfilerActivity

with profile(activities=[ProfilerActivity.CPU, ProfilerActivity.CUDA]) as prof:
    model(input)

prof.export_chrome_trace("trace.json")
# View in chrome://tracing
```

---

## FastAPI and MCP

### FastAPI Structure

**Project Layout:**
```
api/
  __init__.py
  main.py                  # FastAPI app
  config.py                # Configuration
  dependencies.py          # Shared dependencies

  models/
    schemas.py             # Pydantic models
    database.py            # DB models (if using DB)

  routers/
    generate.py            # Generation endpoints
    models.py              # Model management
    jobs.py                # Job status

  services/
    comfyui_client.py      # ComfyUI integration
    storage.py             # File storage
    processing.py          # Post-processing

  mcp/
    server.py              # MCP server implementation
```

**Main Application:**
```python
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI(title="DGX-Pixels API", version="1.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/")
def root():
    return {"message": "DGX-Pixels API"}

# Include routers
from .routers import generate, models, jobs
app.include_router(generate.router, prefix="/api/v1")
app.include_router(models.router, prefix="/api/v1")
app.include_router(jobs.router, prefix="/api/v1")
```

**Generation Endpoint:**
```python
from pydantic import BaseModel
from fastapi import APIRouter, BackgroundTasks

router = APIRouter()

class GenerateRequest(BaseModel):
    prompt: str
    negative_prompt: str = ""
    width: int = 1024
    height: int = 1024
    steps: int = 30
    guidance_scale: float = 7.5
    lora: str | None = None
    lora_strength: float = 1.0

class GenerateResponse(BaseModel):
    job_id: str
    status: str

@router.post("/generate", response_model=GenerateResponse)
async def generate_sprite(
    request: GenerateRequest,
    background_tasks: BackgroundTasks
):
    job_id = create_job(request)
    background_tasks.add_task(process_generation, job_id, request)
    return GenerateResponse(job_id=job_id, status="queued")

@router.get("/jobs/{job_id}")
async def get_job_status(job_id: str):
    job = get_job(job_id)
    return {
        "status": job.status,
        "progress": job.progress,
        "result_url": job.result_url if job.status == "completed" else None
    }
```

### MCP Integration

**FastMCP Implementation:**
```python
from fastapi import FastAPI
from fastmcp import FastMCP

app = FastAPI()
mcp = FastMCP.from_fastapi(
    app=app,
    name="DGX-Pixels MCP",
    description="AI Pixel Art Generation"
)

@mcp.tool()
async def generate_sprite(
    prompt: str,
    style: str = "16bit",
    size: int = 32
) -> dict:
    """Generate a pixel art sprite.

    Args:
        prompt: Description of the sprite
        style: Art style (16bit, 32bit, 8bit)
        size: Sprite size in pixels

    Returns:
        dict with asset_path and metadata
    """
    # Implementation
    result = await comfyui_client.generate(prompt, style, size)
    return {
        "asset_path": result.path,
        "url": result.url,
        "metadata": result.metadata
    }

@mcp.tool()
async def deploy_to_bevy(
    asset_path: str,
    bevy_project: str,
    category: str = "sprites"
) -> dict:
    """Deploy generated asset to Bevy project.

    Args:
        asset_path: Path to generated asset
        bevy_project: Path to Bevy project root
        category: Asset category (sprites, items, etc.)

    Returns:
        dict with deployment status
    """
    # Copy file to Bevy assets directory
    dest = f"{bevy_project}/assets/{category}/{Path(asset_path).name}"
    shutil.copy(asset_path, dest)

    return {
        "status": "deployed",
        "bevy_path": f"assets/{category}/{Path(asset_path).name}"
    }

# Mount MCP server
mcp_app = mcp.http_app(path="/mcp")
app.mount("/mcp", mcp_app)
```

**Usage from Claude or other MCP clients:**
```
User: "Generate a pixel art knight sprite and add it to my Bevy game"

Claude calls:
1. generate_sprite(prompt="medieval knight character", style="16bit", size=32)
2. deploy_to_bevy(asset_path="/output/knight.png", bevy_project="./my_game", category="characters")
```

---

## Sprite Processing

### Color Quantization

**Reduce to Optimal Palette:**
```python
from PIL import Image
import numpy as np
from sklearn.cluster import KMeans

def quantize_colors(image: Image.Image, num_colors: int = 16):
    """Reduce image to n-color palette using k-means."""
    # Convert to numpy
    img_array = np.array(image)
    pixels = img_array.reshape(-1, 3)

    # K-means clustering
    kmeans = KMeans(n_clusters=num_colors, random_state=42)
    kmeans.fit(pixels)

    # Replace pixels with cluster centers
    quantized = kmeans.cluster_centers_[kmeans.labels_]
    quantized = quantized.reshape(img_array.shape).astype(np.uint8)

    return Image.fromarray(quantized)
```

### Pixel Perfect Scaling

```python
def pixel_perfect_scale(image: Image.Image, scale: int):
    """Scale without blur using nearest neighbor."""
    new_size = (image.width * scale, image.height * scale)
    return image.resize(new_size, Image.NEAREST)
```

### Sprite Sheet Assembly

```python
def assemble_sprite_sheet(
    images: list[Image.Image],
    cols: int,
    spacing: int = 0,
    background: tuple[int, int, int, int] = (0, 0, 0, 0)
):
    """Combine multiple sprites into texture atlas."""
    if not images:
        return None

    rows = (len(images) + cols - 1) // cols
    sprite_w, sprite_h = images[0].size

    sheet_w = cols * sprite_w + (cols - 1) * spacing
    sheet_h = rows * sprite_h + (rows - 1) * spacing

    sheet = Image.new("RGBA", (sheet_w, sheet_h), background)

    for idx, img in enumerate(images):
        row = idx // cols
        col = idx % cols
        x = col * (sprite_w + spacing)
        y = row * (sprite_h + spacing)
        sheet.paste(img, (x, y))

    return sheet
```

### Background Removal

```python
from rembg import remove

def remove_background(image: Image.Image):
    """Remove background using ML model."""
    return remove(image)
```

### Complete Processing Pipeline

```python
def process_generated_sprite(
    input_path: str,
    output_path: str,
    target_size: int = 32,
    num_colors: int = 16,
    scale: int = 1
):
    """Full post-processing pipeline."""
    # Load
    image = Image.open(input_path)

    # Remove background
    image = remove_background(image)

    # Resize to target
    image = image.resize((target_size, target_size), Image.LANCZOS)

    # Quantize colors
    image = quantize_colors(image, num_colors)

    # Scale up for viewing
    if scale > 1:
        image = pixel_perfect_scale(image, scale)

    # Save
    image.save(output_path, "PNG", optimize=True)

    return output_path
```

---

## Summary

This technology stack provides:

1. **State-of-the-art generation**: SDXL with pixel art LoRAs
2. **Fast iteration**: ComfyUI's speed and flexibility
3. **Custom styles**: LoRA training on DGX-Spark
4. **Hardware optimization**: Full utilization of Tensor Cores
5. **Easy integration**: MCP protocol for Bevy
6. **Production ready**: FastAPI for scalable API

Next: See `04-bevy-integration.md` for detailed Bevy integration guide.
