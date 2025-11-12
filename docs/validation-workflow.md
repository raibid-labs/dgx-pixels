# Training Validation Workflow

## Overview

This document describes how to use the Side-by-Side Model Comparison feature to validate that LoRA training has improved model quality.

## The Problem: Blind Training

Traditional LoRA training workflow:
1. Collect training data
2. Train LoRA for N epochs
3. **Generate samples and eyeball them**
4. Repeat if results "look bad"

Problems:
- Subjective evaluation ("looks good to me")
- No baseline comparison
- No quantitative metrics
- Hard to track improvement over time
- Difficult to justify training decisions

## The Solution: Data-Driven Validation

DGX-Pixels comparison workflow:
1. Collect training data
2. Train LoRA for N epochs
3. **Run side-by-side comparison with baseline**
4. **Calculate win rate and statistics**
5. **Make data-driven training decisions**

Benefits:
- Objective evaluation (user votes for best)
- Direct baseline comparison
- Quantitative metrics (win rate %)
- Track improvement over training runs
- Justify training decisions with data

## Validation Workflow

### Phase 1: Establish Baseline (Week 1)

**Goal**: Generate baseline samples with pre-trained SDXL

**Steps**:
1. Create test prompt set (10-20 prompts)
2. Generate samples with base SDXL
3. Save samples as baseline reference

**Test Prompt Set Example**:
```
test_prompts.txt:
16-bit knight sprite, pixel art, game asset, white background
8-bit wizard character, retro game style, purple robe
pixel art goblin enemy, side view, green skin
medieval castle tileset, top-down view, 32x32
health potion icon, pixel art, red liquid
...
```

**Baseline Generation**:
```bash
# Generate baseline samples
for prompt in $(cat test_prompts.txt); do
  dgx-pixels generate "$prompt" \
    --model sd_xl_base_1.0.safetensors \
    --seed 42 \
    --output baseline/
done
```

**Deliverable**: 10-20 baseline images in `baseline/` directory

### Phase 2: Train Initial LoRA (Weeks 2-3)

**Goal**: Train first LoRA and validate improvement

**Steps**:
1. Collect training dataset (50-100 images)
2. Train LoRA (2-4 hours on DGX-Spark)
3. Run comparison with baseline
4. Calculate win rate

**Training**:
```bash
python python/training/train_lora.py \
  --name pixel_art_v1 \
  --dataset datasets/pixel_art/ \
  --base_model models/sd_xl_base_1.0.safetensors \
  --steps 3000 \
  --learning_rate 1e-4 \
  --batch_size 4 \
  --output models/loras/pixel_art_v1.safetensors
```

**Comparison**:
1. Open Comparison screen (press `2`)
2. For each test prompt:
   - Slot 1: Base SDXL (no LoRA)
   - Slot 2: SDXL + pixel_art_v1.safetensors
   - Same seed: Enabled
   - Run comparison
   - Vote for best result
3. Export results: `comparison_v1.csv`

**Analysis**:
```python
import pandas as pd

df = pd.read_csv('comparison_v1.csv')

# Calculate win rate
base_wins = len(df[df['winner'] == 0])
lora_wins = len(df[df['winner'] == 1])
total = len(df)

win_rate = (lora_wins / total) * 100
print(f"LoRA v1 Win Rate: {win_rate:.1f}%")
```

**Expected Result**: 50-60% win rate (modest improvement)

**Decision Tree**:
- **Win rate > 60%**: Good progress, continue to Phase 3
- **Win rate 40-60%**: Some improvement, may need more training data
- **Win rate < 40%**: Training may be ineffective, review setup

### Phase 3: Iterate and Improve (Weeks 4-8)

**Goal**: Improve dataset and training to achieve >70% win rate

**Improvement Strategies**:

1. **Expand Dataset**:
   - Add 50-100 more training images
   - Ensure diversity (different characters, objects, scenes)
   - Tag images accurately

2. **Adjust Training Parameters**:
   - Increase training steps (3000 → 5000)
   - Adjust learning rate (try 5e-5 or 2e-4)
   - Increase batch size (if memory allows)

3. **Improve Data Quality**:
   - Remove low-quality images
   - Add more examples of underrepresented styles
   - Balance dataset (equal number of each category)

**Iterative Comparison**:

Train multiple versions and compare:

```
Comparison 1: Base vs v1 (initial)
Comparison 2: Base vs v2 (more data)
Comparison 3: Base vs v3 (tuned parameters)
Comparison 4: v2 vs v3 (which approach better?)
```

**Track Progress**:
```python
import matplotlib.pyplot as plt

versions = ['v1', 'v2', 'v3']
win_rates = [55, 68, 73]  # From comparison results

plt.plot(versions, win_rates, marker='o')
plt.ylabel('Win Rate (%)')
plt.xlabel('LoRA Version')
plt.title('Training Progress')
plt.axhline(y=60, color='r', linestyle='--', label='Target')
plt.legend()
plt.savefig('training_progress.png')
```

**Expected Result**: 70-80% win rate by end of Phase 3

### Phase 4: Fine-Tuning (Weeks 9-12)

**Goal**: Optimize for specific use cases, achieve >80% win rate

**Advanced Comparisons**:

1. **LoRA Strength Comparison**:
   ```
   Model 1: SDXL + LoRA (strength 0.6)
   Model 2: SDXL + LoRA (strength 0.8)
   Model 3: SDXL + LoRA (strength 1.0)
   ```
   Find optimal strength for your use case.

2. **Multi-LoRA Comparison**:
   ```
   Model 1: SDXL + pixel_art_v3
   Model 2: SDXL + pixel_art_v3 + color_palette
   Model 3: SDXL + pixel_art_v3 + upscaler
   ```
   Test LoRA combinations.

3. **Category-Specific Validation**:
   Run separate comparisons for:
   - Character sprites
   - Environment tiles
   - UI icons
   - Weapons/items

   Track win rate per category to identify weak areas.

**Expected Result**: 80-90%+ win rate in target categories

### Phase 5: Production Deployment

**Goal**: Validate production readiness

**Final Validation Checklist**:

- [ ] Overall win rate >80%
- [ ] Win rate >70% in all categories
- [ ] Consistent results across diverse prompts
- [ ] Generation time acceptable (<5s per image)
- [ ] No artifacts or quality issues
- [ ] User feedback positive

**Production Comparison**:
```
Model 1: Previous production LoRA
Model 2: New trained LoRA
```

Only deploy if win rate >60% vs current production model.

## Quantitative Metrics

### Primary Metric: Win Rate

```
Win Rate = (LoRA Wins / Total Comparisons) * 100%
```

**Interpretation**:
- **50%**: LoRA same quality as baseline (no improvement)
- **60%**: Modest improvement
- **70%**: Good improvement
- **80%+**: Significant improvement

### Secondary Metrics

**1. Category Win Rate**:
Track win rate per category (characters, tiles, etc.)

**2. Consistency Score**:
```
Consistency = (Comparisons with clear winner / Total) * 100%
```
Low consistency means results are close/ambiguous.

**3. Generation Time**:
Track average generation time. Ensure LoRA doesn't slow inference.

**4. User Notes Analysis**:
Parse user notes for common themes:
- "Better colors" → Color palette working
- "More detail" → LoRA adding useful detail
- "Too blurry" → Training issue

## Example Analysis Script

```python
#!/usr/bin/env python3
"""
Analyze comparison results and generate training validation report
"""
import pandas as pd
import matplotlib.pyplot as plt
from pathlib import Path

def analyze_comparison(csv_path):
    """Analyze comparison CSV and generate report"""
    df = pd.read_csv(csv_path)

    # Overall win rate
    base_wins = len(df[df['winner'] == 0])
    lora_wins = len(df[df['winner'] == 1])
    total = len(df)
    win_rate = (lora_wins / total) * 100

    print("=" * 60)
    print("TRAINING VALIDATION REPORT")
    print("=" * 60)
    print(f"Total Comparisons: {total}")
    print(f"Base SDXL Wins: {base_wins} ({base_wins/total*100:.1f}%)")
    print(f"Custom LoRA Wins: {lora_wins} ({lora_wins/total*100:.1f}%)")
    print(f"Win Rate: {win_rate:.1f}%")
    print()

    # Decision
    if win_rate >= 80:
        print("✓ EXCELLENT - Ready for production")
    elif win_rate >= 70:
        print("✓ GOOD - Training successful, minor improvements possible")
    elif win_rate >= 60:
        print("⚠ MODERATE - Some improvement, consider more training")
    elif win_rate >= 50:
        print("⚠ MINIMAL - Little improvement, review training setup")
    else:
        print("✗ POOR - Training ineffective, major adjustments needed")
    print()

    # Average generation time
    avg_time = df['generation_time_s'].mean()
    print(f"Avg Generation Time: {avg_time:.2f}s")
    print()

    # User notes analysis
    print("Common Themes in Notes:")
    notes = df['notes'].dropna()
    for note in notes.head(5):
        print(f"  - {note}")
    print()

    # Plot
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 4))

    # Win distribution
    ax1.bar(['Base SDXL', 'Custom LoRA'], [base_wins, lora_wins])
    ax1.set_ylabel('Wins')
    ax1.set_title('Win Distribution')

    # Generation time comparison
    base_times = df[df['model_name'].str.contains('Base')]['generation_time_s']
    lora_times = df[df['model_name'].str.contains('LoRA')]['generation_time_s']
    ax2.boxplot([base_times, lora_times], labels=['Base', 'LoRA'])
    ax2.set_ylabel('Time (s)')
    ax2.set_title('Generation Time')

    plt.tight_layout()
    plt.savefig('validation_report.png')
    print("Report saved to validation_report.png")

if __name__ == '__main__':
    import sys
    if len(sys.argv) != 2:
        print("Usage: python analyze_comparison.py <comparison.csv>")
        sys.exit(1)

    analyze_comparison(sys.argv[1])
```

Usage:
```bash
python analyze_comparison.py comparison_v1.csv
```

## Best Practices

### 1. Use Consistent Test Set

Always use the same test prompts across training runs. This allows valid comparison of v1 vs v2 vs v3.

### 2. Blind Testing (Optional)

For unbiased evaluation:
1. Randomize model slot positions
2. Vote without knowing which is which
3. Reveal model names after voting

### 3. Multiple Evaluators

Have 2-3 people vote on same comparisons:
- Reduces personal bias
- Identifies controversial results
- More robust statistics

### 4. Document Everything

For each training run, record:
- Training parameters
- Dataset size and composition
- Comparison win rate
- User notes and observations
- Training time and compute cost

### 5. Track Trends Over Time

Create a training log:
```csv
version,date,dataset_size,steps,win_rate,notes
v1,2025-11-01,50,3000,55%,Initial training
v2,2025-11-03,100,3000,68%,Added more data
v3,2025-11-05,100,5000,73%,Increased training steps
v4,2025-11-08,150,5000,82%,Expanded dataset + tuned LR
```

## Troubleshooting

### Low Win Rate (<60%)

**Possible Causes**:
1. Insufficient training data
2. Poor data quality
3. Training parameters suboptimal
4. LoRA not compatible with base model

**Solutions**:
1. Add 50-100 more high-quality images
2. Review dataset for consistency
3. Try different learning rates (5e-5, 1e-4, 2e-4)
4. Verify base model compatibility

### Inconsistent Results

**Possible Causes**:
1. Test prompts too varied
2. LoRA overfitting to training data
3. Seed variation issues

**Solutions**:
1. Use more focused test prompts
2. Reduce training steps or add regularization
3. Verify "Use same seed" enabled

### LoRA Worse Than Base (<50% win rate)

**Possible Causes**:
1. Training diverged
2. Wrong base model
3. Dataset mismatch with target style

**Solutions**:
1. Lower learning rate
2. Verify base model version
3. Review dataset quality

## Summary

The Side-by-Side Model Comparison feature transforms LoRA training validation from subjective guessing into a **data-driven process**:

**Old Way**:
- Generate samples
- "Looks good to me"
- Ship it

**New Way**:
- Generate comparison with baseline
- Calculate 73% win rate
- Data shows improvement
- Ship with confidence

Use this workflow to ensure every training run makes measurable progress!
