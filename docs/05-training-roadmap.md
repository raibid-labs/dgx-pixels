# Training Roadmap

## Overview

This roadmap outlines the strategy for training custom models to improve pixel art generation quality and consistency for your game's specific art style. Training custom models via LoRA fine-tuning is **strongly recommended** for production projects.

## Table of Contents
- [Why Train Custom Models](#why-train-custom-models)
- [Training Phases](#training-phases)
- [Dataset Creation](#dataset-creation)
- [Training Strategy](#training-strategy)
- [Evaluation and Iteration](#evaluation-and-iteration)
- [Deployment](#deployment)

---

## Why Train Custom Models

### Benefits

1. **Style Consistency**: Ensures all generated assets match your game's art direction
2. **Better Prompt Adherence**: Model learns your specific terminology and requirements
3. **Reduced Post-Processing**: Generates closer to final assets
4. **Unique Aesthetic**: Creates distinctive look not available in pre-trained models
5. **Character Consistency**: Maintains character appearance across different poses
6. **Technical Accuracy**: Learns your specific sprite dimensions, palette constraints

### Pre-trained vs Custom Trained

| Aspect | Pre-trained Models | Custom LoRA Models |
|--------|-------------------|-------------------|
| **Setup Time** | Immediate | 1-2 days |
| **Training Time** | None | 2-4 hours per LoRA |
| **Style Consistency** | Variable | Excellent |
| **Character Consistency** | Poor | Good |
| **Prompt Understanding** | Generic | Project-specific |
| **Asset Quality** | Good | Excellent |
| **Post-processing Needed** | High | Low |
| **Cost** | Free | GPU time (~$5-10/model) |

**Recommendation**: Start with pre-trained models for prototyping, then train custom LoRAs before production asset generation.

---

## Training Phases

### Phase 1: Foundation (Week 1-2)

**Goal**: Establish baseline with pre-trained models

**Tasks**:
1. Test multiple pre-trained pixel art models
2. Generate sample assets for your game
3. Identify quality issues and gaps
4. Document desired vs actual results
5. Collect reference art (existing or curated examples)

**Deliverables**:
- Evaluation report of 3-5 pre-trained models
- Collection of 50-100 reference images
- List of quality issues to address

### Phase 2: Style Training (Week 3-4)

**Goal**: Train a LoRA that captures your game's overall art style

**Dataset**: 50-100 images representing your art style

**Model**: General style LoRA

**Training Time**: 3-4 hours on DGX-Spark

**Example Images**:
- Various character types in your style
- Environment art
- Items and props
- UI elements
- Different color palettes used

**Success Metrics**:
- Generated assets match style guide
- Color palette consistency
- Line weight and shading match references

### Phase 3: Specialized Models (Week 5-8)

**Goal**: Train specialized LoRAs for specific asset categories

**Models to Train**:

1. **Character LoRA**
   - Dataset: 30-50 character sprites
   - Focus: Body proportions, animation poses
   - Use cases: Heroes, NPCs, enemies

2. **Environment LoRA**
   - Dataset: 40-60 tile and background images
   - Focus: Perspective, tileable edges
   - Use cases: Levels, dungeons, outdoor areas

3. **Items LoRA**
   - Dataset: 30-50 item sprites
   - Focus: Iconic silhouettes, consistent size
   - Use cases: Weapons, potions, collectibles

4. **Effects LoRA** (optional)
   - Dataset: 30-40 effect animations
   - Focus: Particle shapes, motion blur
   - Use cases: Magic, explosions, UI feedback

**Training Time**: 2-3 hours each on DGX-Spark

### Phase 4: Character Consistency (Week 9-10)

**Goal**: Maintain specific character appearance across different poses

**Approach**: DreamBooth or character-specific LoRA

**Dataset**: 20-30 images of same character in different poses/angles

**Training**:
- Use trigger word (e.g., "herochar")
- Train for character identity
- Test with various pose prompts

**Example**:
```bash
# Training
dgx-pixels train dreambooth \
  --instance-prompt "herochar knight" \
  --dataset ./datasets/hero_knight/ \
  --output ./models/hero_knight_db

# Usage
dgx-pixels generate "herochar knight, walking pose, side view"
```

### Phase 5: Refinement (Week 11-12)

**Goal**: Fine-tune based on production feedback

**Activities**:
1. Generate production assets
2. Collect artist feedback
3. Identify remaining issues
4. Create augmented datasets addressing issues
5. Retrain with improved data
6. A/B test old vs new models

**Continuous Improvement**:
- Add new training examples from approved generated assets
- Update models monthly with curated new data
- Track quality metrics over time

---

## Dataset Creation

### Sourcing Training Data

**Option 1: Use Existing Art**
- Sprites from previous projects
- Concept art and mockups
- Asset store purchases (check licensing!)
- Open-source game assets

**Option 2: Commission Reference Art**
- Hire pixel artist for 50-100 reference sprites
- Specify variety: poses, angles, types
- Ensure consistent style
- One-time investment: $500-2000

**Option 3: Curate from Open Sources**
- OpenGameArt.org
- itch.io assets
- Game dev communities
- Filter for consistent style

**Option 4: Synthetic Data Augmentation**
- Generate with pre-trained models
- Manually curate best results
- Use approved outputs as training data
- Bootstrap from small initial dataset

### Dataset Preparation

**1. Image Curation**
```python
# Quality checklist
criteria = [
    "Consistent art style",
    "Clear, sharp pixels",
    "Appropriate resolution",
    "Clean background or transparent",
    "Good color palette",
    "Representative of target style"
]
```

**2. Image Processing**
```python
from PIL import Image
import os

def prepare_training_image(input_path, output_path, target_size=1024):
    """Prepare image for SDXL training."""
    img = Image.open(input_path)

    # Convert to RGBA
    if img.mode != 'RGBA':
        img = img.convert('RGBA')

    # Pad to square
    max_dim = max(img.size)
    canvas = Image.new('RGBA', (max_dim, max_dim), (0, 0, 0, 0))
    offset = ((max_dim - img.size[0]) // 2, (max_dim - img.size[1]) // 2)
    canvas.paste(img, offset)

    # Resize to target
    canvas = canvas.resize((target_size, target_size), Image.LANCZOS)

    # Save
    canvas.save(output_path, 'PNG')

# Process dataset
for img_file in os.listdir('raw_dataset/'):
    prepare_training_image(
        f'raw_dataset/{img_file}',
        f'prepared_dataset/{img_file}'
    )
```

**3. Auto-Captioning**
```python
from transformers import BlipProcessor, BlipForConditionalGeneration
from PIL import Image

processor = BlipProcessor.from_pretrained("Salesforce/blip-image-captioning-base")
model = BlipForConditionalGeneration.from_pretrained("Salesforce/blip-image-captioning-base")

def generate_caption(image_path):
    """Generate caption for training image."""
    image = Image.open(image_path)
    inputs = processor(image, return_tensors="pt")
    out = model.generate(**inputs, max_length=50)
    caption = processor.decode(out[0], skip_special_tokens=True)

    # Add pixel art context
    caption = f"pixel art, {caption}, game sprite, 16-bit style"

    return caption

# Generate captions
for img_file in os.listdir('prepared_dataset/'):
    if img_file.endswith('.png'):
        caption = generate_caption(f'prepared_dataset/{img_file}')

        # Save caption file
        caption_file = img_file.replace('.png', '.txt')
        with open(f'prepared_dataset/{caption_file}', 'w') as f:
            f.write(caption)
```

**4. Manual Caption Refinement**

Review and improve auto-generated captions:
```
# Before (auto-generated)
pixel art, a person holding a sword, game sprite, 16-bit style

# After (manually refined)
pixel art, medieval knight character, holding sword, standing pose,
front view, full body, game sprite, 16-bit rpg style, clean background
```

### Dataset Organization

```
training_datasets/
├── style_general/
│   ├── 001_knight.png
│   ├── 001_knight.txt
│   ├── 002_mage.png
│   ├── 002_mage.txt
│   └── ...
├── characters/
│   ├── heroes/
│   ├── enemies/
│   └── npcs/
├── environments/
│   ├── dungeons/
│   ├── forests/
│   └── towns/
└── items/
    ├── weapons/
    ├── potions/
    └── treasures/
```

---

## Training Strategy

### Hyperparameter Selection

**Starting Configuration** (works for most cases):
```yaml
# config.yaml
model:
  base: "stabilityai/stable-diffusion-xl-base-1.0"
  type: "lora"

lora:
  rank: 64
  alpha: 64
  dropout: 0.1

training:
  learning_rate: 1e-4
  batch_size: 4
  gradient_accumulation_steps: 2  # effective batch = 8
  max_train_steps: 3000
  save_every_n_steps: 500

optimization:
  optimizer: "AdamW8bit"
  lr_scheduler: "cosine"
  warmup_steps: 100
  max_grad_norm: 1.0

regularization:
  min_snr_gamma: 5
  noise_offset: 0.05

hardware:
  mixed_precision: "fp16"
  gradient_checkpointing: true
  xformers: true
```

### Training Script

```python
#!/usr/bin/env python3
# train_lora.py

import argparse
from pathlib import Path
import accelerate
from diffusers import DiffusionPipeline, DDPMScheduler
from peft import LoraConfig, get_peft_model

def main(args):
    # Load base model
    pipe = DiffusionPipeline.from_pretrained(
        args.model_name,
        torch_dtype=torch.float16
    )

    # Configure LoRA
    lora_config = LoraConfig(
        r=args.lora_rank,
        lora_alpha=args.lora_alpha,
        target_modules=["to_k", "to_q", "to_v", "to_out.0"],
        lora_dropout=args.lora_dropout,
    )

    # Wrap model with LoRA
    model = get_peft_model(pipe.unet, lora_config)

    # Training loop
    # ... (detailed implementation)

    # Save trained LoRA
    model.save_pretrained(args.output_dir)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--model_name", default="stabilityai/stable-diffusion-xl-base-1.0")
    parser.add_argument("--dataset", required=True)
    parser.add_argument("--output_dir", required=True)
    parser.add_argument("--lora_rank", type=int, default=64)
    # ... more arguments

    args = parser.parse_args()
    main(args)
```

**Run Training:**
```bash
python train_lora.py \
  --dataset ./training_datasets/style_general/ \
  --output_dir ./models/my_game_style_v1 \
  --config config.yaml
```

### Training Monitoring

**1. Loss Curves**
- Should decrease steadily
- Plateaus indicate convergence
- Sharp spikes indicate learning rate too high

**2. Sample Generation**
- Generate samples every 500 steps
- Use consistent test prompts
- Visual quality should improve over time

**3. Validation Metrics**
```python
validation_prompts = [
    "pixel art knight character, standing",
    "16bit potion item, red, game sprite",
    "dungeon floor tile, stone, top-down view",
    "pixel art mage casting spell"
]

# Generate and save every N steps
for epoch, step in training_loop:
    if step % 500 == 0:
        for prompt in validation_prompts:
            image = generate(prompt)
            image.save(f"samples/step_{step}_{hash(prompt)}.png")
```

**4. Overfitting Check**
- Generate from prompts NOT in training set
- Should still produce good results
- If only training prompts work well → overfit

---

## Evaluation and Iteration

### Evaluation Framework

**Automated Metrics**:
```python
from pytorch_fid import fid_score
from clip_score import compute_clip_score

def evaluate_model(model_path, test_prompts, reference_images):
    """Evaluate trained model."""

    # 1. Generate test images
    generated = generate_batch(model_path, test_prompts)

    # 2. FID Score (distribution similarity)
    fid = fid_score.calculate_fid_given_paths(
        [reference_images, generated],
        batch_size=50,
        device='cuda',
        dims=2048
    )

    # 3. CLIP Score (text-image alignment)
    clip_scores = compute_clip_score(generated, test_prompts)

    # 4. Style consistency (custom metric)
    consistency = measure_style_consistency(generated)

    return {
        "fid": fid,
        "clip_score": clip_scores.mean(),
        "consistency": consistency
    }
```

**Human Evaluation**:
```yaml
# evaluation_rubric.yaml
categories:
  - name: "Style Match"
    weight: 0.3
    scale: 1-5
    description: "Matches target art style"

  - name: "Prompt Accuracy"
    weight: 0.25
    scale: 1-5
    description: "Generates requested content"

  - name: "Technical Quality"
    weight: 0.25
    scale: 1-5
    description: "Clean pixels, good composition"

  - name: "Usability"
    weight: 0.2
    scale: 1-5
    description: "Ready to use in game"
```

### A/B Testing

```python
def ab_test_models(model_a, model_b, test_prompts, evaluators):
    """Compare two model versions."""

    results = {"model_a": [], "model_b": []}

    for prompt in test_prompts:
        img_a = generate(model_a, prompt)
        img_b = generate(model_b, prompt)

        # Blind test (randomize order)
        if random.random() < 0.5:
            shown = [(img_a, "a"), (img_b, "b")]
        else:
            shown = [(img_b, "b"), (img_a, "a")]

        # Collect votes
        for evaluator in evaluators:
            vote = evaluator.choose(shown[0][0], shown[1][0])
            results[shown[vote][1]].append(1)

    # Statistical analysis
    from scipy import stats
    t_stat, p_value = stats.ttest_ind(results["model_a"], results["model_b"])

    return {
        "model_a_wins": sum(results["model_a"]),
        "model_b_wins": sum(results["model_b"]),
        "p_value": p_value,
        "significant": p_value < 0.05
    }
```

### Iteration Cycle

1. **Train** → 2-4 hours
2. **Generate test set** → 30 minutes
3. **Evaluate** → 1-2 hours (with humans)
4. **Identify issues** → 1 hour
5. **Improve dataset** → Variable
6. **Repeat**

**Typical Issues and Solutions**:

| Issue | Solution |
|-------|----------|
| Blurry outputs | Add more high-quality sharp examples |
| Wrong colors | Curate color palette, add palette-correct images |
| Inconsistent style | Remove outliers from training set |
| Poor composition | Add well-composed examples |
| Ignores prompts | Improve captions, increase training steps |
| Character inconsistency | Use DreamBooth or character-specific LoRA |

---

## Deployment

### Model Registry

```
models/
├── production/
│   ├── style_v1.2.safetensors      # Current production
│   ├── characters_v1.0.safetensors
│   └── environments_v1.1.safetensors
├── staging/
│   └── style_v1.3_rc1.safetensors  # Testing
└── archive/
    └── style_v1.0.safetensors      # Old versions
```

### Metadata Tracking

```json
{
  "model_name": "style_v1.2",
  "base_model": "SDXL 1.0",
  "type": "LoRA",
  "training_date": "2025-01-15",
  "dataset": {
    "name": "game_style_general",
    "size": 75,
    "version": "1.2"
  },
  "hyperparameters": {
    "learning_rate": 1e-4,
    "steps": 3000,
    "lora_rank": 64
  },
  "metrics": {
    "fid": 23.4,
    "clip_score": 0.82,
    "human_rating": 4.3
  },
  "trigger_words": ["gamestyle", "16bit rpg"],
  "recommended_strength": 0.8,
  "status": "production"
}
```

### Gradual Rollout

```python
class ModelSelector:
    """Gradually transition to new model."""

    def __init__(self, old_model, new_model, rollout_percentage=0.0):
        self.old_model = old_model
        self.new_model = new_model
        self.rollout = rollout_percentage

    def select_model(self, user_id):
        """Select model based on rollout percentage."""
        if hash(user_id) % 100 < self.rollout:
            return self.new_model
        return self.old_model

# Usage
selector = ModelSelector(
    old_model="models/style_v1.1",
    new_model="models/style_v1.2",
    rollout_percentage=10  # 10% of users
)

model = selector.select_model(request.user_id)
```

### Rollback Strategy

```python
def rollback_model(from_version, to_version):
    """Rollback to previous model version."""

    # Update symlink
    os.symlink(
        f"models/production/style_{to_version}.safetensors",
        "models/current/style.safetensors"
    )

    # Log rollback
    log_event({
        "event": "model_rollback",
        "from": from_version,
        "to": to_version,
        "timestamp": datetime.now(),
        "reason": "Quality degradation detected"
    })

    # Notify team
    send_alert(f"Model rolled back from {from_version} to {to_version}")
```

---

## Timeline Summary

| Week | Phase | Activities | Deliverables |
|------|-------|-----------|--------------|
| 1-2 | Foundation | Test pre-trained models, collect references | 50-100 reference images, evaluation report |
| 3-4 | Style Training | Train general style LoRA | Production style model v1.0 |
| 5-6 | Character Models | Train character-specific LoRA | Character model v1.0 |
| 7-8 | Environment Models | Train environment LoRA | Environment model v1.0 |
| 9-10 | Character Consistency | DreamBooth for key characters | 3-5 character-specific models |
| 11-12 | Refinement | Iterate based on feedback | Improved models v1.1 |

**Total Time**: 12 weeks from start to production-ready custom models

**Ongoing**: Monthly retraining with new curated data

---

## Cost Estimate

### One-time Costs
- Reference art commission: $500-2000
- Initial setup and testing: 40 hours (developer time)
- Training compute (DGX-Spark already owned): $0

### Ongoing Costs
- Dataset curation: 4 hours/month
- Model retraining: 8 hours GPU time/month
- Evaluation: 4 hours/month
- Total: ~10-12 hours/month

**ROI**: Custom models reduce post-processing time by 60-80%, making them worthwhile after generating 100+ assets.

---

## Success Criteria

**Phase 2 (Style Training) Success**:
- ✓ Generated assets match style guide 90%+ of time
- ✓ Consistent color palette across generations
- ✓ Minimal post-processing needed
- ✓ Artists approve quality for production use

**Phase 3 (Specialized Models) Success**:
- ✓ Character proportions consistent
- ✓ Environment tiles are seamless
- ✓ Items have recognizable silhouettes
- ✓ Each model outperforms general model in its category

**Phase 4 (Character Consistency) Success**:
- ✓ Same character recognizable across all poses
- ✓ Clothing and colors remain consistent
- ✓ Distinctive features preserved

**Overall Success**:
- ✓ 80%+ of generated assets used in game with minimal edits
- ✓ Asset generation time reduced by 70%+
- ✓ Consistent visual style across all generated content
- ✓ Team prefers AI-generated + edited over from-scratch

---

## Next Steps

1. **Decide on training timeline** based on project needs
2. **Allocate budget** for reference art if needed
3. **Assign team member** to manage training process
4. **Set up training environment** (see `06-implementation-plan.md`)
5. **Begin Phase 1**: Test pre-trained models and collect references

For technical implementation details, see `03-technology-deep-dive.md`.
For getting started, see `06-implementation-plan.md`.
