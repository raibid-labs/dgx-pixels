# Side-by-Side Model Comparison Guide

## Overview

The **Side-by-Side Model Comparison** feature is the unique differentiator of DGX-Pixels. It allows you to compare multiple models (base SDXL vs custom LoRA) simultaneously to validate that your training has improved quality.

This feature is essential for:
- **Training Validation**: Verify that your custom LoRA actually improves quality
- **Model Selection**: Compare different LoRA models to choose the best one
- **A/B Testing**: Make data-driven decisions about model configurations
- **Quality Assurance**: Track preference rates across multiple comparisons

## Quick Start

### 1. Navigate to Comparison Screen

Press `2` or navigate to the Comparison screen from the main menu.

### 2. Configure Comparison

1. **Enter Prompt**: Type your prompt (same prompt will be used for all models)
2. **Select Models**:
   - Press `M` to open model selector
   - Use arrow keys to navigate
   - Press `Enter` to select
   - Repeat for slots 2 and 3 (minimum 2 models required)
3. **Configure Parameters**:
   - Ensure "Use same seed" is checked (fair comparison)
   - Adjust steps, CFG scale, size as needed

### 3. Run Comparison

1. Press `Enter` to start comparison
2. Wait for all models to finish generating (10-15 seconds)
3. Review side-by-side results

### 4. Vote for Best Result

1. Press `1`, `2`, or `3` to vote for the best model
2. Optionally add notes about why you preferred it
3. Results are saved for analysis

### 5. Export Results

1. Press `E` to export comparison results
2. Choose format: CSV or JSON
3. Results include prompts, models, votes, and timing data

## Workflow: Training Validation

This is the primary use case for the comparison feature.

### Step 1: Prepare Models

You need at least two models:
- **Model 1**: Base SDXL (pre-trained, no LoRA)
- **Model 2**: SDXL + Your Custom LoRA

```
Model 1 Config:
  Name: "SDXL Base 1.0"
  Base: sd_xl_base_1.0.safetensors
  LoRA: None

Model 2 Config:
  Name: "SDXL + Pixel Art LoRA"
  Base: sd_xl_base_1.0.safetensors
  LoRA: pixel_art_v1.safetensors
  LoRA Strength: 0.8
```

### Step 2: Create Test Prompts

Prepare 10-20 prompts that represent your target use case:

```
Test Prompts:
1. "16-bit knight sprite, pixel art, game asset"
2. "8-bit wizard character, retro game style"
3. "pixel art goblin enemy, side view"
4. "medieval castle tileset, top-down view"
5. "32x32 health potion icon, pixel art"
...
```

### Step 3: Run Comparisons

For each test prompt:
1. Enter prompt in comparison screen
2. Select both models
3. Use same seed (checked by default)
4. Run comparison
5. Vote for which result is better

### Step 4: Analyze Results

After running all comparisons:
1. Export results to CSV: `comparison_results.csv`
2. Open in spreadsheet or analysis tool
3. Calculate LoRA win rate:

```python
import pandas as pd

df = pd.read_csv('comparison_results.csv')

# Count wins
base_wins = len(df[df['winner'] == 0])
lora_wins = len(df[df['winner'] == 1])
total = len(df)

lora_win_rate = (lora_wins / total) * 100
print(f"LoRA Win Rate: {lora_win_rate:.1f}%")
```

### Step 5: Interpret Results

**LoRA Win Rate > 60%**: Training significantly improved quality - use this model!

**LoRA Win Rate 40-60%**: Training showed some improvement - may need more data or tuning

**LoRA Win Rate < 40%**: Training may need adjustment - review training parameters

## Features

### Same Seed Enforcement

When "Use same seed" is enabled (default), all models use identical:
- Seed value
- Prompt
- Generation parameters (steps, CFG, size)

This ensures a **fair comparison** - the only difference is the model/LoRA.

### Progress Tracking

During generation:
- Each model shows individual progress bar
- Stage information (Initializing, Sampling, Decoding)
- ETA for each model
- Side-by-side progress visualization

### Side-by-Side Preview

Results are displayed side-by-side:
- Model 1 | Model 2 | Model 3
- Image previews (Sixel if supported, placeholder otherwise)
- Generation time for each
- Model configuration (base, LoRA, strength)

### Preference Tracking

Vote for the best result:
- Press `1` for Model 1
- Press `2` for Model 2
- Press `3` for Model 3 (if used)
- Add optional notes
- Results saved to comparison history

### Report Export

Export formats:

**CSV Format:**
```csv
comparison_id,prompt,seed,model_name,base_model,lora,lora_strength,generation_time_s,winner,notes,completed_at
cmp-001,"knight sprite",42,"Base SDXL","sdxl_base.safetensors","",1.0,3.5,0,"base looks better","2025-11-11T10:30:00Z"
cmp-001,"knight sprite",42,"Pixel LoRA","sdxl_base.safetensors","pixel_art.safetensors",0.8,3.8,0,"base looks better","2025-11-11T10:30:00Z"
```

**JSON Format:**
```json
{
  "metadata": {
    "generated_at": "2025-11-11T10:30:00Z",
    "version": "1.0.0",
    "total_comparisons": 10
  },
  "comparisons": [
    {
      "comparison_id": "cmp-001",
      "prompt": "knight sprite",
      "seed": 42,
      "models": [
        {
          "name": "Base SDXL",
          "base_model": "sdxl_base.safetensors",
          "lora": null,
          "generation_time_s": 3.5
        },
        {
          "name": "Pixel LoRA",
          "base_model": "sdxl_base.safetensors",
          "lora": "pixel_art.safetensors",
          "generation_time_s": 3.8
        }
      ],
      "winner": "Base SDXL",
      "notes": "base looks better"
    }
  ],
  "statistics": {
    "total_comparisons": 10,
    "comparisons_with_preference": 10,
    "preference_rate": 100.0,
    "model_wins": [
      {"model_name": "Base SDXL", "wins": 3, "win_rate": 30.0},
      {"model_name": "Pixel LoRA", "wins": 7, "win_rate": 70.0}
    ]
  }
}
```

## Advanced Use Cases

### Comparing Multiple LoRA Models

Compare 3 different LoRA models at once:

```
Model 1: Base SDXL (control)
Model 2: Pixel Art LoRA v1
Model 3: Pixel Art LoRA v2
```

This helps you decide which LoRA version is best.

### LoRA Strength Comparison

Compare same LoRA at different strengths:

```
Model 1: SDXL + LoRA (strength 0.6)
Model 2: SDXL + LoRA (strength 0.8)
Model 3: SDXL + LoRA (strength 1.0)
```

This helps you find optimal LoRA strength.

### Training Progress Tracking

Compare LoRA at different training checkpoints:

```
Model 1: pixel_art_v1_epoch_1.safetensors
Model 2: pixel_art_v1_epoch_2.safetensors
Model 3: pixel_art_v1_epoch_3.safetensors
```

This helps you identify when to stop training.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `2` | Navigate to Comparison screen |
| `M` | Open model selector for current slot |
| `↑`/`↓` | Navigate model list |
| `Enter` | Select model / Start comparison |
| `1`-`3` | Vote for model 1, 2, or 3 |
| `R` | Run comparison again (same settings) |
| `E` | Export results |
| `S` | Toggle "Use same seed" |
| `ESC` | Back / Cancel |
| `Q` | Quit |

## Technical Details

### Multi-Job Orchestration

Comparisons use parallel generation:
1. TUI submits N generation jobs (one per model) via ZeroMQ
2. Backend queues all jobs
3. Backend processes jobs in parallel (if GPU memory allows)
4. TUI tracks progress for each job separately
5. Results displayed when all jobs complete

### Progress Updates

Each model gets real-time progress updates via PUB-SUB:
```
Job 1: 45% (stage: Sampling, ETA: 2.1s)
Job 2: 43% (stage: Sampling, ETA: 2.3s)
Job 3: 44% (stage: Sampling, ETA: 2.2s)
```

### Memory Management

The DGX-Spark has 119GB unified memory, allowing:
- Load 2-3 SDXL models simultaneously (~6.5GB each)
- Generate with multiple models in parallel
- No model swapping overhead

### Fair Comparison Guarantees

When "Use same seed" is enabled:
- All jobs use identical seed value
- Same RNG state for noise generation
- Same prompt and parameters
- Only model/LoRA differs

This ensures any quality difference is due to the model, not randomness.

## Troubleshooting

### "Need 2+ models to compare"

**Problem**: Comparison requires at least 2 models selected.

**Solution**: Press `M` to select models for slots 1 and 2.

### Progress bars stuck

**Problem**: Generation not progressing.

**Solution**:
1. Check backend is running
2. Check ZeroMQ connection
3. Check backend logs for errors
4. Press `C` to cancel and retry

### Results look identical

**Problem**: Both models producing same output.

**Solution**:
1. Verify different models selected
2. Check LoRA actually loaded (see model metadata)
3. Increase LoRA strength
4. Try different prompts

### Export fails

**Problem**: Cannot export results.

**Solution**:
1. Ensure write permissions in export directory
2. Check disk space
3. Try different export format (CSV vs JSON)

## Best Practices

### 1. Use Diverse Test Prompts

Don't just test one prompt - use 10-20 diverse prompts covering your use cases.

### 2. Always Enable "Use Same Seed"

This ensures fair comparison. Disable only if you want to test randomness/variety.

### 3. Vote Honestly

Don't bias toward custom LoRA just because you trained it. Vote objectively.

### 4. Track Trends Over Time

Export results after each training run to track improvement over time.

### 5. Document Your Findings

Add notes to each comparison explaining why you preferred one model.

### 6. Compare Incrementally

Start with base vs LoRA v1, then v1 vs v2, etc. This helps isolate improvements.

## Example Workflow

Here's a complete training validation workflow:

```bash
# 1. Train LoRA
python train_lora.py \
  --dataset pixel_art_dataset \
  --epochs 3 \
  --output pixel_art_v1.safetensors

# 2. Run comparison (in TUI)
# - Press 2 to open Comparison screen
# - Select "Base SDXL" for slot 1
# - Select "SDXL + Pixel Art v1" for slot 2
# - Enter 10 test prompts
# - Vote for best results
# - Export to comparison_v1.csv

# 3. Analyze results
python analyze_comparison.py comparison_v1.csv
# Output: LoRA Win Rate: 72% - Training successful!

# 4. If win rate low, adjust training and repeat
python train_lora.py \
  --dataset pixel_art_dataset_v2 \  # More data
  --epochs 5 \                       # More training
  --learning_rate 1e-4 \             # Adjust LR
  --output pixel_art_v2.safetensors

# 5. Compare v1 vs v2
# (repeat comparison process)
```

## Integration with Training Pipeline

The comparison feature integrates with the 12-week training roadmap (see `05-training-roadmap.md`):

**Weeks 1-2**: Generate baseline with pre-trained SDXL

**Weeks 3-4**: Train first LoRA, compare with baseline
- Expected win rate: 50-60%

**Weeks 5-8**: Improve dataset and training
- Expected win rate: 60-75%

**Weeks 9-12**: Fine-tune and optimize
- Expected win rate: 75-85%+

Use comparison results to guide training adjustments!

## Summary

The Side-by-Side Model Comparison feature is **the killer feature** of DGX-Pixels. It provides:

- **Objective validation** of training improvements
- **Data-driven decisions** about model selection
- **Quantitative metrics** (win rates, preferences)
- **Export capabilities** for analysis
- **Fair comparison** through same-seed enforcement

This feature transforms training from guesswork into a data-driven process. Use it to ensure your custom LoRAs actually improve quality!
