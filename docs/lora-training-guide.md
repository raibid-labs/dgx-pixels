# LoRA Training Guide for DGX-Pixels

Complete guide to training custom LoRA models for pixel art generation on NVIDIA DGX-Spark GB10.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Dataset Preparation](#dataset-preparation)
- [Training Configuration](#training-configuration)
- [Running Training](#running-training)
- [Validation and Testing](#validation-and-testing)
- [Integration with ComfyUI](#integration-with-comfyui)
- [Troubleshooting](#troubleshooting)
- [Advanced Topics](#advanced-topics)

---

## Overview

LoRA (Low-Rank Adaptation) fine-tuning allows you to customize Stable Diffusion XL for your specific pixel art style with:

- **Minimal training data**: 50-100 images
- **Fast training**: 2-4 hours on GB10
- **Small model size**: 50-200 MB LoRA weights
- **Style consistency**: Match your game's art direction
- **Easy deployment**: Load LoRA on top of base SDXL

### What You'll Need

- Training dataset: 50-100 pixel art images
- Captions: Text descriptions for each image
- Hardware: NVIDIA DGX-Spark GB10 (or compatible GPU)
- Time: 2-4 hours for training
- Disk space: ~10 GB for models and outputs

---

## Quick Start

### 1. Install Dependencies

```bash
cd /home/beengud/raibid-labs/dgx-pixels
pip install -r python/training/requirements.txt
```

### 2. Prepare Your Dataset

```bash
# Prepare images and create captions
python3 python/training/dataset_prep.py \
  --input ./my_raw_images \
  --output ./datasets/my_style \
  --create-captions

# Validate dataset
python3 python/training/dataset_prep.py \
  --input ./datasets/my_style \
  --validate-only
```

### 3. Train LoRA

```bash
# Train with default settings (3000 steps, ~3 hours)
python3 python/training/lora_trainer.py \
  --dataset ./datasets/my_style \
  --output ./outputs/my_lora \
  --rank 32 \
  --steps 3000
```

### 4. Validate Results

```bash
# Generate validation samples
python3 python/training/validation/validate_lora.py \
  --lora ./outputs/my_lora/checkpoint_final \
  --output ./outputs/validation \
  --compare-base
```

---

## Dataset Preparation

### Dataset Requirements

**Minimum Requirements:**
- **Size**: 50 images minimum, 100+ recommended
- **Resolution**: 512×512 or higher (will be resized to 1024×1024)
- **Format**: PNG, JPG, or JPEG
- **Quality**: Clean, sharp pixels (no blur or compression artifacts)
- **Style**: Consistent art style across all images
- **Captions**: Text description for each image

**Dataset Structure:**
```
datasets/my_style/
├── sprite_001.png
├── sprite_001.txt
├── sprite_002.png
├── sprite_002.txt
└── ...
```

### Collecting Training Data

**Option 1: Use Existing Art**
```bash
# Copy your existing pixel art to a directory
mkdir -p datasets/my_game_art
cp ~/my_game/assets/sprites/*.png datasets/my_game_art/
```

**Option 2: Curate from Open Sources**
- OpenGameArt.org
- itch.io game assets
- Pixel art communities
- Asset packs (check licensing)

**Option 3: Commission Reference Art**
- Hire pixel artist for 50-100 reference sprites
- Specify variety: characters, items, environments
- Cost: $500-2000 one-time investment

### Preprocessing Images

The dataset preparator handles:
- Resizing to 1024×1024 (SDXL requirement)
- Padding rectangular images to square
- Format conversion to PNG
- Quality validation

**Prepare Dataset:**
```bash
python3 python/training/dataset_prep.py \
  --input ./raw_images \
  --output ./datasets/prepared \
  --resolution 1024 \
  --create-captions
```

**Options:**
- `--resolution 1024`: Target resolution (default: 1024 for SDXL)
- `--create-captions`: Create default captions for images without captions
- `--no-pad`: Don't pad to square (stretches instead)
- `--validate-only`: Validate without processing

### Creating Captions

Good captions improve training quality. Each caption should describe:
- **Subject**: What is in the image
- **Style**: "pixel art", "16-bit", "game sprite"
- **Details**: Pose, angle, colors, features

**Example Captions:**
```
# Good captions
pixel art knight character, standing pose, front view, full body, medieval armor, 16-bit rpg style

pixel art potion item, red health potion, glass bottle, game inventory item, 16-bit style

pixel art dungeon floor tile, stone texture, top-down view, seamless, 16-bit rpg

# Avoid generic captions
pixel art game sprite

# Avoid over-detailed captions
pixel art knight character with shiny silver armor holding a golden sword with ruby in the hilt standing on stone floor with grass in background in medieval fantasy setting with detailed shading
```

### Auto-Captioning

Use BLIP-2 for automatic caption generation:

```bash
# Auto-caption all images in dataset
python3 python/training/captioning.py \
  --dataset ./datasets/my_style \
  --mode caption \
  --prefix "pixel art, " \
  --suffix ", game sprite, 16-bit style"

# Validate captions
python3 python/training/captioning.py \
  --dataset ./datasets/my_style \
  --mode validate

# Manual editing mode
python3 python/training/captioning.py \
  --dataset ./datasets/my_style \
  --mode edit
```

**Auto-captioning will:**
1. Analyze each image with BLIP-2
2. Generate base caption
3. Add pixel art context (prefix/suffix)
4. Save to `.txt` file next to image

**Note**: Auto-generated captions should be reviewed and refined manually for best results.

---

## Training Configuration

Three preset configurations are provided:

### 1. Fast Training (1-2 hours)

```yaml
# configs/training/fast_training.yaml
lora:
  rank: 16  # Lower rank = faster
  alpha: 16

training:
  max_train_steps: 1500
  learning_rate: 2e-4
```

**Use for:**
- Quick experiments
- Testing dataset quality
- Rapid iterations

### 2. Balanced Training (2-4 hours)

```yaml
# configs/training/pixel_art_base.yaml
lora:
  rank: 32  # Balanced capacity
  alpha: 32

training:
  max_train_steps: 3000
  learning_rate: 1e-4
```

**Use for:**
- Production models
- General style training
- Recommended starting point

### 3. Quality Training (4-6 hours)

```yaml
# configs/training/quality_training.yaml
lora:
  rank: 64  # Higher capacity
  alpha: 64

training:
  max_train_steps: 5000
  learning_rate: 5e-5
```

**Use for:**
- Final production models
- Complex styles
- Maximum quality

### Key Parameters

**LoRA Parameters:**
- `rank` (8-64): LoRA rank, higher = more capacity but slower training
- `alpha` (8-64): Scaling factor, typically same as rank
- `dropout` (0.0-0.2): Regularization, 0.1 recommended

**Training Parameters:**
- `learning_rate` (1e-5 to 5e-4): Learning rate, lower = more stable
- `batch_size` (1-4): Batch size, limited by memory
- `max_train_steps` (1000-10000): Total training steps
- `gradient_accumulation_steps` (1-8): Effective batch size multiplier

**Optimization:**
- `mixed_precision`: "fp16" (use Tensor Cores)
- `gradient_checkpointing`: true (save memory)
- `optimizer`: "adamw_8bit" (memory efficient)

---

## Running Training

### Basic Training Command

```bash
python3 python/training/lora_trainer.py \
  --dataset ./datasets/my_style \
  --output ./outputs/my_lora \
  --rank 32 \
  --alpha 32 \
  --lr 1e-4 \
  --steps 3000 \
  --batch-size 2
```

### Training Arguments

```
Required:
  --dataset PATH          Path to training dataset

Optional:
  --output PATH          Output directory (default: ./outputs/lora_training)
  --model NAME           Base model (default: sd_xl_base_1.0)
  --rank INT             LoRA rank (default: 32)
  --alpha INT            LoRA alpha (default: 32)
  --lr FLOAT             Learning rate (default: 1e-4)
  --steps INT            Training steps (default: 3000)
  --batch-size INT       Batch size (default: 2)
```

### Monitoring Training

Training progress is logged to console:
```
Training: 100%|██████████| 3000/3000 [2:45:33<00:00, 3.31s/it]
loss: 0.1234, lr: 1.00e-04, step_time: 3.31s
```

**Key Metrics:**
- `loss`: Should decrease over time (0.5 → 0.1)
- `lr`: Learning rate (follows schedule)
- `step_time`: Time per training step (2-4s on GB10)
- `memory`: GPU memory usage (<80GB)

### Checkpoints

Checkpoints are saved every 500 steps:
```
outputs/my_lora/
├── checkpoint_step_500/
├── checkpoint_step_1000/
├── checkpoint_step_1500/
├── checkpoint_step_2000/
├── checkpoint_step_2500/
├── checkpoint_step_3000/
└── checkpoint_final/
```

### Training Metrics

Training metrics are saved to JSON:
```bash
cat outputs/my_lora/training_metrics.json
```

### Resuming Training

To resume from checkpoint (future feature):
```bash
python3 python/training/lora_trainer.py \
  --dataset ./datasets/my_style \
  --output ./outputs/my_lora \
  --resume checkpoint_step_1500
```

---

## Validation and Testing

### Generate Validation Samples

```bash
python3 python/training/validation/validate_lora.py \
  --lora ./outputs/my_lora/checkpoint_final \
  --output ./outputs/validation \
  --compare-base \
  --steps 25 \
  --guidance 8.0
```

This generates images with:
- Your trained LoRA
- Base SDXL model (for comparison)
- Same prompts and seeds (fair comparison)

### Validation Prompts

Create a prompts file:
```bash
cat > validation_prompts.txt << EOF
pixel art knight character, standing pose, front view
pixel art mage character, casting spell, side view
16bit potion item, red, game sprite
pixel art sword weapon, medieval, game item
pixel art dungeon floor tile, stone, top-down
EOF
```

Use prompts file:
```bash
python3 python/training/validation/validate_lora.py \
  --lora ./outputs/my_lora/checkpoint_final \
  --prompts-file validation_prompts.txt \
  --output ./outputs/validation
```

### Comparing Multiple LoRAs

Compare different training runs:
```bash
python3 python/training/validation/validate_lora.py \
  --compare-models \
  --loras ./outputs/lora_v1 ./outputs/lora_v2 ./outputs/lora_v3 \
  --names "Version 1" "Version 2" "Version 3" \
  --output ./outputs/comparison
```

### Quality Assessment

**Visual Inspection:**
1. Compare LoRA vs base model outputs
2. Check style consistency across prompts
3. Verify prompt adherence
4. Assess pixel art quality (no blur/smooth edges)

**Key Questions:**
- Does the LoRA match your training data style?
- Are generated sprites usable in your game?
- Is quality better than base model?
- Does the LoRA overfit (only works on training prompts)?

---

## Integration with ComfyUI

### Loading LoRA in ComfyUI

1. Copy trained LoRA to ComfyUI directory:
```bash
cp ./outputs/my_lora/checkpoint_final/adapter_model.safetensors \
   ./comfyui/models/loras/my_pixel_art_style.safetensors
```

2. In ComfyUI workflow, add "Load LoRA" node
3. Select your LoRA from dropdown
4. Set LoRA strength (0.6-1.0 typical)

### ComfyUI Workflow Example

```json
{
  "nodes": [
    {
      "type": "CheckpointLoaderSimple",
      "model": "sd_xl_base_1.0.safetensors"
    },
    {
      "type": "LoraLoader",
      "lora_name": "my_pixel_art_style.safetensors",
      "strength_model": 0.8,
      "strength_clip": 0.8
    },
    {
      "type": "CLIPTextEncode",
      "text": "pixel art knight character, standing pose"
    },
    {
      "type": "KSampler",
      "steps": 20,
      "cfg": 8.0
    }
  ]
}
```

### LoRA Strength

- `0.5-0.7`: Subtle style influence
- `0.7-0.9`: Balanced (recommended)
- `0.9-1.0`: Strong style enforcement
- `>1.0`: Can cause artifacts

Test different strengths to find optimal balance.

---

## Troubleshooting

### Training Issues

**Loss Not Decreasing:**
- Check dataset quality (consistent style, good captions)
- Reduce learning rate: `--lr 5e-5`
- Increase training steps: `--steps 5000`
- Check for duplicate/corrupted images

**Out of Memory (OOM):**
- Reduce batch size: `--batch-size 1`
- Enable gradient checkpointing (already default)
- Close other GPU processes
- Use smaller LoRA rank: `--rank 16`

**Training Too Slow:**
- Increase batch size if memory allows: `--batch-size 4`
- Reduce LoRA rank: `--rank 16`
- Use fast training config
- Check GPU utilization: `nvidia-smi`

**Poor Quality Results:**
- Increase training steps: `--steps 5000`
- Use higher LoRA rank: `--rank 64`
- Improve dataset (more images, better captions)
- Check for overfitting (test with new prompts)

### Dataset Issues

**Not Enough Images:**
- Minimum 50 images recommended
- Can work with 30-40 but quality suffers
- Consider data augmentation (flips, rotations)
- Mix with similar style images

**Inconsistent Style:**
- Remove outliers from training set
- Focus on single art style
- Separate different styles into different LoRAs

**Caption Quality:**
- Review auto-generated captions
- Add specific style keywords
- Be consistent with terminology
- Not too short (<20 chars) or too long (>200 chars)

### Validation Issues

**LoRA Not Loading:**
- Check file path
- Verify safetensors format
- Ensure compatible with base model version

**Generated Images Look Like Base Model:**
- Increase LoRA strength in ComfyUI
- Train longer (more steps)
- Use higher LoRA rank
- Check if LoRA weights are too small

---

## Advanced Topics

### Training Strategy

**Phase 1: Quick Test (1 hour)**
- Use fast config
- 10-20 images
- Validate approach

**Phase 2: Full Training (3 hours)**
- Use balanced config
- 50-100 images
- Production quality

**Phase 3: Refinement (1-2 hours)**
- Fine-tune based on feedback
- Adjust learning rate
- Add difficult examples

### Hyperparameter Tuning

**Learning Rate:**
- Too high: Loss oscillates, poor convergence
- Too low: Slow training, may not converge
- Sweet spot: 1e-4 to 5e-5 for SDXL LoRA

**LoRA Rank:**
- Rank 8-16: Fast, small, limited capacity
- Rank 32: Balanced (recommended)
- Rank 64-128: High capacity, slower, larger files

**Training Steps:**
- Dataset size × 50-100 = rough estimate
- 50 images × 60 = 3000 steps (good starting point)
- Monitor loss curve to determine when to stop

### Advanced Techniques

**Min-SNR Weighting:**
Improves training stability by weighting loss based on noise level.
```yaml
regularization:
  min_snr_gamma: 5.0  # Recommended value
```

**Noise Offset:**
Helps model learn brightness variations.
```yaml
regularization:
  noise_offset: 0.05  # Small offset
```

**Gradient Accumulation:**
Simulate larger batch size without more memory.
```yaml
training:
  batch_size: 2
  gradient_accumulation_steps: 4  # Effective batch = 8
```

### Multiple LoRAs

Train specialized LoRAs for different categories:

1. **Style LoRA**: General art style (train first)
2. **Character LoRA**: Character-specific features
3. **Environment LoRA**: Backgrounds and tiles
4. **Items LoRA**: Weapons, potions, collectibles

Use multiple LoRAs together in ComfyUI by chaining LoRA loaders.

---

## Performance Targets (GB10)

**Training:**
- Single epoch: 5-10 minutes for 50 images
- Full training (3000 steps): 2-3 hours
- Memory usage: 40-60 GB
- Batch size: 2-4

**Validation:**
- Single image: 3-5 seconds
- Batch of 10 images: 30-50 seconds

---

## Best Practices

1. **Start Small**: Test with fast config before full training
2. **Quality Over Quantity**: 50 good images > 200 mixed quality
3. **Consistent Captions**: Use consistent terminology
4. **Monitor Loss**: Loss should decrease steadily
5. **Validate Often**: Check samples every 500-1000 steps
6. **Compare Models**: Always compare LoRA vs base model
7. **Iterate**: Train → Evaluate → Improve dataset → Repeat

---

## Next Steps

1. **Prepare your dataset** following the guidelines above
2. **Run fast training** to validate approach
3. **Evaluate results** with validation tool
4. **Iterate** on dataset and hyperparameters
5. **Full training** with balanced/quality config
6. **Deploy** to ComfyUI for production use

For detailed technical information, see:
- `docs/05-training-roadmap.md`: 12-week training strategy
- `docs/03-technology-deep-dive.md`: LoRA technical details
- `docs/troubleshooting.md`: Common issues and solutions

---

**Training Time Estimate:**
- Dataset preparation: 2-4 hours
- Fast training: 1-2 hours
- Validation: 30 minutes
- Full training: 2-4 hours
- Total: 1-2 days for first LoRA

**Recommended Workflow:**
1. Day 1: Prepare dataset, fast training, evaluation
2. Day 2: Refine dataset, full training, validation
3. Day 3+: Integration, testing, refinement
