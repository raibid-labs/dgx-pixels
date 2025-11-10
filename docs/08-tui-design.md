# TUI Design and Workflows

## Overview

This document details the Terminal User Interface (TUI) design for DGX-Pixels, including screen layouts, interaction patterns, and workflows. The TUI is built with **ratatui** in Rust, providing a fast, responsive interface for pixel art generation.

## Table of Contents
- [Design Philosophy](#design-philosophy)
- [Screen Layouts](#screen-layouts)
- [Key Features](#key-features)
- [Interaction Patterns](#interaction-patterns)
- [Side-by-Side Model Comparison](#side-by-side-model-comparison)
- [Image Preview](#image-preview)

---

## Design Philosophy

### Principles

1. **Speed First**: 60+ FPS rendering, <50ms input latency
2. **Information Dense**: Max info in limited terminal space
3. **Keyboard Driven**: All actions accessible via keyboard
4. **Visual Feedback**: Clear indication of state and progress
5. **Non-Blocking**: UI remains responsive during generation

### Color Scheme

```
Primary:   Cyan    (#00FFFF) - Active elements, highlights
Secondary: Yellow  (#FFFF00) - Warnings, notifications
Success:   Green   (#00FF00) - Completed jobs, success states
Error:     Red     (#FF0000) - Errors, failures
Muted:     Gray    (#808080) - Inactive, disabled elements
Background: Black  (#000000) - Terminal background
```

### Typography

- **Headers**: Bold, uppercase
- **Values**: Regular weight
- **Hotkeys**: Square brackets `[K]`
- **Status**: Colored indicators `●`

---

## Screen Layouts

### 1. Generation Screen (Main)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ DGX-Pixels v0.1.0                          [Q]uit [Tab] Switch Panel    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│ ┌─ Generate ─────────────────────────────────────────────────────────┐ │
│ │                                                                      │ │
│ │ Prompt:                                                              │ │
│ │ ┌──────────────────────────────────────────────────────────────────┐│ │
│ │ │medieval knight character, standing pose, side view, pixel art__  ││ │
│ │ └──────────────────────────────────────────────────────────────────┘│ │
│ │                                                                      │ │
│ │ Model: [SDXL Pixel Art v2 ▼]  LoRA: [16bit_rpg ▼]  Size: [32x32]  │ │
│ │                                                                      │ │
│ │ ┌─ Options ─────────────────────┬─ Preview ───────────────────────┐│ │
│ │ │ Steps:         [30        ]    │                                 ││ │
│ │ │ CFG Scale:     [7.5       ]    │    ░░▓▓▓▓██████▓▓░░             ││ │
│ │ │ Seed:          [Random    ]    │    ░░██████████████░░           ││ │
│ │ │ Batch Size:    [1         ]    │    ▓▓██████████████▓▓           ││ │
│ │ │ Palette:       [16 colors ]    │    ▓▓██████████████▓▓           ││ │
│ │ │                                 │    ▓▓██████████████▓▓           ││ │
│ │ │ [G]enerate  [C]ompare Models   │    ██████████████████           ││ │
│ │ └─────────────────────────────────┴─────────────────────────────────┘│ │
│ │                                                                      │ │
│ │ ┌─ Recent Generations ──────────────────────────────────────────────┐│ │
│ │ │ ● job_001  knight      SDXL+16bit   12.3s  [View] [Deploy]       ││ │
│ │ │ ● job_002  mage        SDXL+16bit   13.1s  [View] [Deploy]       ││ │
│ │ │ ○ job_003  warrior     Generating... 45%   [Cancel]              ││ │
│ │ └──────────────────────────────────────────────────────────────────┘│ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
├─────────────────────────────────────────────────────────────────────────┤
│ GPU: 87% (78°C) │ Mem: 24.2/128GB │ Queue: 1 job │ [1]Gen [2]Queue     │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Features**:
- Large prompt input area
- Model/LoRA selection dropdowns
- Real-time preview (updated as generation progresses)
- Options panel with common settings
- Recent generations list with quick actions
- Status bar with GPU/memory metrics

### 2. Compare Models Screen

```
┌─────────────────────────────────────────────────────────────────────────┐
│ DGX-Pixels - Model Comparison                        [Esc] Back         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│ Prompt: "medieval knight character, standing pose"                       │
│                                                                           │
│ ┌─ Model A: SDXL Base ───────────────┬─ Model B: SDXL + 16bit_rpg ────┐│
│ │                                     │                                 ││
│ │   ░░▓▓▓▓██████▓▓░░                  │   ░░▓▓▓▓██████▓▓░░             ││
│ │   ░░██████████████░░                │   ░░██████████████░░           ││
│ │   ▓▓██████████████▓▓                │   ▓▓██████████████▓▓           ││
│ │   ▓▓██████████████▓▓                │   ▓▓██████████████▓▓           ││
│ │   ▓▓██████████████▓▓                │   ▓▓██████████████▓▓           ││
│ │   ██████████████████                │   ██████████████████           ││
│ │                                     │                                 ││
│ │ Time: 14.2s                         │ Time: 12.8s                     ││
│ │ Steps: 30                           │ Steps: 30                       ││
│ │ CFG: 7.5                            │ CFG: 7.5                        ││
│ │                                     │                                 ││
│ │ Style: Generic                      │ Style: 16-bit RPG ✓             ││
│ │ Colors: 256 (smooth)                │ Colors: 16 (quantized) ✓       ││
│ │ Detail: High                        │ Detail: Pixel-perfect ✓        ││
│ │                                     │                                 ││
│ │ [1] Select Model A                  │ [2] Select Model B              ││
│ ├─────────────────────────────────────┴─────────────────────────────────┤│
│ │ ┌─ Model C: Custom Trained ──────────────────────────────────────────┐││
│ │ │   ░░▓▓▓▓██████▓▓░░     Time: 11.9s  Style: Custom Fantasy ✓       │││
│ │ │   ░░██████████████░░   Steps: 30    Colors: 16 ✓                  │││
│ │ │   (similar preview)    CFG: 8.0     Detail: High ✓                │││
│ │ │                                                                     │││
│ │ │ [3] Select Model C                                                 │││
│ │ └─────────────────────────────────────────────────────────────────────┘││
│ └───────────────────────────────────────────────────────────────────────┘│
│                                                                           │
│ [G]enerate All  [S]ave Comparison  [V]ote (Mark Best)                   │
│                                                                           │
├─────────────────────────────────────────────────────────────────────────┤
│ Generating with 3 models... │ ETA: 24s │ [Esc] Cancel                   │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Features**:
- Side-by-side comparison of 2-4 models
- Same prompt for all models (A/B testing)
- Metrics comparison (time, style matching, color accuracy)
- Visual checkmarks for desired attributes
- Quick selection for further use
- Batch generation across models

### 3. Queue Manager Screen

```
┌─────────────────────────────────────────────────────────────────────────┐
│ DGX-Pixels - Job Queue                      [Esc] Back [R]efresh        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│ ┌─ Active Jobs (2) ──────────────────────────────────────────────────┐ │
│ │                                                                      │ │
│ │ ● job_005  RUNNING   "dragon breathing fire"      [████████░░] 85%  │ │
│ │   Model: SDXL+fantasy  Step 26/30  ETA: 2s         [P]ause [X]Kill  │ │
│ │   Preview: ░░▓▓██████▓▓░░                                           │ │
│ │                                                                      │ │
│ │ ○ job_006  QUEUED    "wizard casting spell"        [░░░░░░░░░░] 0%  │ │
│ │   Model: SDXL+16bit   Waiting...   ETA: 15s        [U]p [D]own      │ │
│ │                                                                      │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌─ Completed Jobs (15) ──────────────────────────────────────────────┐ │
│ │ ✓ job_004  "knight"       SDXL+16bit    12.3s  output/knight.png   │ │
│ │ ✓ job_003  "mage"         SDXL+16bit    13.1s  output/mage.png     │ │
│ │ ✓ job_002  "warrior"      SDXL+custom   11.8s  output/warrior.png  │ │
│ │ ✗ job_001  "invalid"      FAILED        Error: Invalid prompt      │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌─ Queue Stats ──────────────────────────────────────────────────────┐ │
│ │ Total Jobs: 21       Completed: 15      Failed: 1      Queued: 2   │ │
│ │ Avg Time: 12.8s      Success Rate: 93%  GPU Util: 87%              │ │
│ │                                                                      │ │
│ │ Throughput: 4.2 jobs/min                                            │ │
│ │ Est. Queue Clear: 25 seconds                                        │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ [N]ew Job  [C]lear Completed  [F]ilter  [E]xport Queue                  │
│                                                                           │
├─────────────────────────────────────────────────────────────────────────┤
│ Active: 2 │ Queued: 2 │ Completed: 15 │ Failed: 1                       │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Features**:
- Real-time job status with progress bars
- Live preview thumbnails for running jobs
- Queue reordering (up/down)
- Pause/resume/cancel controls
- Completed job history with quick actions
- Queue statistics and throughput metrics

### 4. GPU Monitor Screen

```
┌─────────────────────────────────────────────────────────────────────────┐
│ DGX-Pixels - System Monitor                 [Esc] Back [R]efresh       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│ ┌─ GPU Metrics ──────────────────────────────────────────────────────┐ │
│ │                                                                      │ │
│ │ Utilization:  [████████████████████████████████████████████░░] 87%  │ │
│ │                                                                      │ │
│ │ Temperature:  78°C  [████████████████░░░░░░░░░░░░░░░░░░░░░░]       │ │
│ │               ▲ 72  73  74  75  76  77  78  78  78  78  (last 10s)  │ │
│ │                                                                      │ │
│ │ Power:        240W / 350W  [████████████████████████░░░░░░░░] 68%   │ │
│ │                                                                      │ │
│ │ Memory:       24.2 GB / 128 GB  [███████░░░░░░░░░░░░░░░░░░░] 19%   │ │
│ │               ▼ Breakdown:                                           │ │
│ │                 - SDXL Model:     8.2 GB                             │ │
│ │                 - LoRA Adapters:  0.4 GB                             │ │
│ │                 - Active Batch:   2.1 GB                             │ │
│ │                 - ComfyUI:        1.2 GB                             │ │
│ │                 - Python Worker:  0.3 GB                             │ │
│ │                 - Available:     104 GB                              │ │
│ │                                                                      │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌─ Performance History (Last 5 min) ─────────────────────────────────┐ │
│ │                                                                      │ │
│ │ Inference Time:                                                      │ │
│ │  14s │                              ██                               │ │
│ │  12s │                 ██      ██   ██                               │ │
│ │  10s │     ██   ██     ██      ██   ██      ██                       │ │
│ │   8s │     ██   ██     ██      ██   ██      ██                       │ │
│ │      └─────────────────────────────────────────────                 │ │
│ │       :00  :30  1:00  1:30  2:00  2:30  3:00  3:30  4:00  4:30  5:00│ │
│ │                                                                      │ │
│ │ Avg: 12.3s   Min: 10.8s   Max: 14.2s   Std Dev: 1.2s                │ │
│ │                                                                      │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌─ System Resources ─────────────────────────────────────────────────┐ │
│ │ CPU:  [████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 23%  (16 of 20 cores)│ │
│ │ RAM:  [████████████████████████░░░░░░░░░░░░] 64GB / 128GB (50%)   │ │
│ │ Disk: [████████████████████████████████████░░] 892GB / 1TB (89%)   │ │
│ │ Net:  ↓ 2.3 MB/s  ↑ 0.8 MB/s                                        │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ [S]napshot  [E]xport Metrics  [A]lerts  [L]ogs                          │
│                                                                           │
├─────────────────────────────────────────────────────────────────────────┤
│ GPU: 87% (78°C) │ Mem: 24.2/128GB │ Jobs: 2 active, 2 queued            │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Features**:
- Real-time GPU metrics (util, temp, power, memory)
- Memory breakdown by component
- Performance history graphs
- System resource monitoring
- Alerts for threshold violations

### 5. Model Manager Screen

```
┌─────────────────────────────────────────────────────────────────────────┐
│ DGX-Pixels - Model Manager                   [Esc] Back [R]efresh       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│ ┌─ Base Models ──────────────────────────────────────────────────────┐ │
│ │ ✓ stabilityai/stable-diffusion-xl-base-1.0          8.2 GB  LOADED │ │
│ │   Default model for generation                                      │ │
│ │   [U]nload  [I]nfo  [B]enchmark                                     │ │
│ │                                                                      │ │
│ │ ○ stable-diffusion-v1-5                             3.8 GB          │ │
│ │   Smaller, faster model for quick iteration                         │ │
│ │   [L]oad  [I]nfo  [D]ownload                                        │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌─ LoRA Adapters ────────────────────────────────────────────────────┐ │
│ │ ✓ 16bit_rpg_v1              420 MB  LOADED   ⭐⭐⭐⭐⭐ (23 uses)      │ │
│ │   16-bit RPG style, trained on 75 images                            │ │
│ │   Trained: 2025-01-15  Steps: 3000  Rank: 64                        │ │
│ │   [U]nload  [I]nfo  [R]etrain  [C]ompare                            │ │
│ │                                                                      │ │
│ │ ○ fantasy_characters_v2     385 MB           ⭐⭐⭐⭐ (12 uses)        │ │
│ │   Character sprites, fantasy theme                                  │ │
│ │   [L]oad  [I]nfo  [T]est                                            │ │
│ │                                                                      │ │
│ │ ○ 32bit_modern_v1           410 MB           ⭐⭐⭐ (5 uses)          │ │
│ │   32-bit modern pixel art style                                     │ │
│ │   [L]oad  [I]nfo  [D]elete                                          │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌─ Custom Training ──────────────────────────────────────────────────┐ │
│ │ Dataset: my_game_style/         75 images  22.4 MB                  │ │
│ │ Status:  Ready to train                                             │ │
│ │                                                                      │ │
│ │ [T]rain New LoRA  [V]alidate Dataset  [P]review Samples            │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌─ Model Comparison History ─────────────────────────────────────────┐ │
│ │ 2025-01-15  "knight sprite"  Winner: 16bit_rpg (4/5 votes)         │ │
│ │ 2025-01-14  "mage sprite"    Winner: fantasy_char (3/5 votes)      │ │
│ │ [V]iew Details  [E]xport Report                                     │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ [D]ownload Model  [T]rain LoRA  [C]ompare Models  [S]ettings            │
│                                                                           │
├─────────────────────────────────────────────────────────────────────────┤
│ Loaded: 1 base, 1 LoRA │ Memory: 8.6/128 GB │ [5] Models                │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Features**:
- Base model management (load/unload/download)
- LoRA adapter library with ratings
- Model comparison history
- Training interface for custom LoRAs
- Memory usage tracking
- Quick model switching

### 6. Gallery Browser

```
┌─────────────────────────────────────────────────────────────────────────┐
│ DGX-Pixels - Gallery                        [Esc] Back [←→] Navigate    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│ Filter: [All ▼]  Sort: [Recent ▼]  Search: [________________] [Enter]  │
│                                                                           │
│ ┌─────────────────┬─────────────────┬─────────────────┬─────────────────┐│
│ │ knight_001.png  │ mage_002.png    │ warrior_003.png │ dragon_004.png  ││
│ │                 │                 │                 │                 ││
│ │   ▓▓██████▓▓    │   ░░████████    │   ██████████    │   ▓▓▓▓████      ││
│ │   ██████████    │   ▓▓████████    │   ██████████    │   ████████▓▓    ││
│ │   ██████████    │   ██████████    │   ██████████    │   ████████░░    ││
│ │                 │                 │                 │                 ││
│ │ 32x32  12.3s    │ 32x32  13.1s    │ 32x32  11.8s    │ 64x64  18.2s    ││
│ │ 16bit_rpg ⭐⭐⭐   │ 16bit_rpg ⭐⭐    │ custom_v2 ⭐⭐⭐   │ fantasy ⭐⭐⭐⭐    ││
│ ├─────────────────┼─────────────────┼─────────────────┼─────────────────┤│
│ │ (4 more rows...)                                                     ││
│ └──────────────────────────────────────────────────────────────────────┘│
│                                                                           │
│ ┌─ Selected: knight_001.png ─────────────────────────────────────────┐ │
│ │                                                                      │ │
│ │     ░░▓▓▓▓██████▓▓░░           Prompt: "medieval knight character,  │ │
│ │     ░░██████████████░░                  standing pose, side view"   │ │
│ │     ▓▓██████████████▓▓         Model: SDXL + 16bit_rpg_v1          │ │
│ │     ▓▓██████████████▓▓         Size: 32x32                          │ │
│ │     ▓▓██████████████▓▓         Time: 12.3s                          │ │
│ │     ██████████████████         Seed: 42                             │ │
│ │                                 Rating: ⭐⭐⭐                          │ │
│ │                                                                      │ │
│ │ [V]iew Full  [D]eploy to Bevy  [E]dit  [Del]ete  [S]hare           │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ [F]ilter  [S]ort  [T]ag  [B]atch Actions  [E]xport                      │
│                                                                           │
├─────────────────────────────────────────────────────────────────────────┤
│ Total: 247 images │ Selected: 1 │ Space: 892 MB │ [6] Gallery          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Features**:
- Grid view of generated images
- Thumbnail previews with metadata
- Filter and search
- Detailed view with full metadata
- Quick actions (deploy, edit, delete)
- Batch operations

---

## Key Features

### 1. Live Image Preview

**Implementation**: ratatui-image supports multiple protocols:
- **Sixels**: High-quality image rendering (if terminal supports)
- **Unicode halfblocks**: Fallback for wider compatibility
- **ASCII art**: Ultimate fallback

```rust
use ratatui_image::{Image, protocol::StatefulProtocol};

// In render function
let img = image::open("preview.png")?;
let image_widget = Image::new(&img);
f.render_widget(image_widget, preview_area);
```

**Preview Modes**:
1. **Live Preview**: Updates every 500ms during generation (from Python pub-sub)
2. **Final Preview**: Full resolution after completion
3. **Thumbnail Grid**: Multiple images in gallery view

### 2. Model Comparison Workflow

**User Flow**:
```
1. Enter prompt
2. Press [C] to compare models
3. Select 2-4 models/LoRAs
4. TUI generates with all models in parallel
5. Results displayed side-by-side
6. User votes/selects best
7. Winner noted for future reference
```

**Data Tracking**:
```rust
struct ModelComparison {
    prompt: String,
    models: Vec<ModelConfig>,
    results: Vec<GenerationResult>,
    winner: Option<usize>,  // Index of winning model
    votes: Vec<Vote>,
    timestamp: SystemTime,
}

// Store in SQLite
fn save_comparison(comp: &ModelComparison) {
    db.execute(
        "INSERT INTO comparisons (prompt, winner, votes, timestamp) VALUES (?, ?, ?, ?)",
        // ...
    );
}
```

### 3. Real-Time Progress

**Progress Sources**:
- ComfyUI WebSocket (step-by-step)
- Python worker pub-sub (high-level status)
- Preview images (partial results)

**Rendering**:
```rust
// Progress bar
let gauge = Gauge::default()
    .block(Block::default().title("Generating"))
    .gauge_style(Style::default().fg(Color::Cyan))
    .percent(progress as u16);

// With ETA
let label = format!("Step {}/{} - ETA: {}s", step, total, eta);
let gauge = gauge.label(label);
```

### 4. Batch Operations

**Queue Multiple Jobs**:
```rust
impl App {
    fn batch_generate(&mut self, prompts: Vec<String>, config: GenerateConfig) {
        for prompt in prompts {
            let job_id = self.zmq_client.generate(&prompt, config.clone())?;
            self.job_queue.push(job_id);
        }
    }
}
```

**Example**: Generate 20 variations from prompt file:
```
[B] Batch Generate → Select file → Configure options → Submit all
```

### 5. Keyboard Shortcuts

**Global**:
- `Tab` / `Shift+Tab` - Switch between panels
- `Q` - Quit
- `?` - Help overlay
- `Esc` - Back/Cancel

**Generation Screen**:
- `G` - Generate
- `C` - Compare models
- `B` - Batch generate
- `S` - Settings

**Queue Screen**:
- `P` - Pause selected job
- `X` - Cancel selected job
- `U` / `D` - Move job up/down in queue
- `R` - Retry failed job

**Gallery Screen**:
- `←` / `→` - Navigate images
- `Space` - Select/deselect
- `V` - View full size
- `D` - Deploy to Bevy
- `Del` - Delete

---

## Interaction Patterns

### Dropdown Menus

```
Model: [SDXL Pixel Art v2 ▼]  <- Click or press Enter
       ┌────────────────────────────┐
       │ SDXL Base                  │
       │ > SDXL Pixel Art v2        │  <- Selected
       │ SDXL + 16bit_rpg           │
       │ SDXL + fantasy_char        │
       │ Custom Trained v1          │
       └────────────────────────────┘
       Press ↑↓ to navigate, Enter to select
```

### Modal Dialogs

```
┌───────────────────────────────────┐
│ Deploy to Bevy Project            │
├───────────────────────────────────┤
│                                   │
│ Project: /home/user/my_game       │
│ Category: sprites/characters      │
│ Filename: knight.png              │
│                                   │
│ [Deploy]  [Cancel]                │
└───────────────────────────────────┘
```

### Notifications

```
┌────────────────────────────────────────┐
│ ✓ Generation complete!                 │
│   knight_001.png ready                 │
└────────────────────────────────────────┘
  (Auto-dismiss in 3s)

┌────────────────────────────────────────┐
│ ⚠ GPU temperature high (82°C)          │
│   Consider reducing batch size         │
└────────────────────────────────────────┘
  [Dismiss]
```

---

## Side-by-Side Model Comparison

### Workflow Detailed

**Step 1: Initiate Comparison**
```
User is on Generation screen
Enters prompt: "fantasy mage character"
Presses [C] to compare models
```

**Step 2: Model Selection**
```
┌─ Select Models to Compare ──────────────┐
│ □ SDXL Base                              │
│ ☑ SDXL + 16bit_rpg                       │
│ ☑ SDXL + fantasy_char                    │
│ ☑ Custom Trained v2                      │
│                                          │
│ Selected: 3 models                       │
│ Est. Time: 36-42s (3 × 12-14s)          │
│                                          │
│ [Generate]  [Cancel]                     │
└──────────────────────────────────────────┘
```

**Step 3: Parallel Generation**
```
TUI sends 3 requests to Python worker:
- Job 1: SDXL + 16bit_rpg
- Job 2: SDXL + fantasy_char
- Job 3: Custom Trained v2

Python worker queues all 3 (processes sequentially)
TUI subscribes to progress for all 3 jobs
```

**Step 4: Live Updates**
```
┌─ Generating with 3 models... ──────────┐
│                                         │
│ [████████░░] SDXL + 16bit_rpg     82%   │
│ [███░░░░░░░] SDXL + fantasy_char  30%   │
│ [░░░░░░░░░░] Custom Trained v2    0%    │
│                                         │
│ Overall: [████░░░░░░░] 37%              │
│ ETA: 24 seconds                         │
└─────────────────────────────────────────┘
```

**Step 5: Results Display**
```
(Shows comparison screen from Layout #2)
- All 3 results side-by-side
- Metrics comparison
- Visual differences highlighted
```

**Step 6: User Votes**
```
User reviews all 3
Presses [2] to select Model B (fantasy_char)
System records:
- Comparison ID
- Prompt
- Models tested
- Winner: fantasy_char
- User preference data
```

**Step 7: Use Winner**
```
Selected model becomes default for next generation
Or user can immediately generate more with winner:
[G] Generate More with Selected Model
```

### Comparison Data Structure

```rust
#[derive(Debug, Serialize, Deserialize)]
struct ComparisonSession {
    id: Uuid,
    prompt: String,
    timestamp: SystemTime,
    models: Vec<ModelConfig>,
    results: Vec<ComparisonResult>,
    winner: Option<usize>,
    user_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComparisonResult {
    model_name: String,
    generation_time: Duration,
    image_path: PathBuf,
    metrics: GenerationMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerationMetrics {
    color_count: usize,
    style_score: f32,      // How well it matches target style
    prompt_adherence: f32,  // How well it matches prompt
    technical_quality: f32, // Sharpness, artifacts, etc.
}
```

### Comparison History

```
Stored in SQLite:

CREATE TABLE comparisons (
    id TEXT PRIMARY KEY,
    prompt TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    winner_model TEXT,
    models_tested TEXT,  -- JSON array
    user_notes TEXT
);

CREATE TABLE comparison_results (
    comparison_id TEXT,
    model_name TEXT,
    generation_time_ms INTEGER,
    image_path TEXT,
    metrics TEXT,  -- JSON
    FOREIGN KEY (comparison_id) REFERENCES comparisons(id)
);
```

**Analytics Query**:
```sql
-- Which model wins most often?
SELECT winner_model, COUNT(*) as wins
FROM comparisons
WHERE winner_model IS NOT NULL
GROUP BY winner_model
ORDER BY wins DESC;

-- Average generation time by model
SELECT model_name, AVG(generation_time_ms) as avg_time
FROM comparison_results
GROUP BY model_name;
```

---

## Image Preview

### Sixel Support

**Check Terminal Capability**:
```rust
use ratatui_image::protocol::StatefulProtocol;

fn detect_best_protocol() -> Box<dyn StatefulProtocol> {
    // Try in order of quality
    if sixel_supported() {
        Box::new(SixelProtocol::new())
    } else if kitty_supported() {
        Box::new(KittyProtocol::new())
    } else {
        Box::new(HalfblockProtocol::new())
    }
}
```

**Rendering**:
```rust
use image::DynamicImage;
use ratatui_image::Image;

fn render_preview(f: &mut Frame, area: Rect, img_path: &Path) {
    let img = image::open(img_path).unwrap();

    // Resize to fit terminal area
    let (width, height) = calculate_size(area, img.dimensions());
    let resized = img.resize_exact(width, height, FilterType::Nearest);

    let image_widget = Image::new(&resized);
    f.render_widget(image_widget, area);
}
```

### Live Preview Updates

```rust
impl App {
    fn subscribe_to_preview(&mut self, job_id: &str) {
        let topic = format!("preview.{}", job_id);

        loop {
            if let Some(update) = self.zmq_client.poll_updates(100)? {
                if let Some(preview_bytes) = update.preview {
                    // Save to temp file
                    let path = format!("/tmp/preview_{}.png", job_id);
                    std::fs::write(&path, preview_bytes)?;

                    // Update TUI
                    self.current_preview = Some(path);
                    self.needs_redraw = true;
                }
            }
        }
    }
}
```

---

## Performance Considerations

### Rendering Optimization

```rust
// Only redraw when needed
impl App {
    fn should_redraw(&self) -> bool {
        self.needs_redraw ||
        self.has_active_jobs() ||
        self.animation_frame_changed()
    }

    fn run(&mut self) {
        loop {
            if self.should_redraw() {
                self.render(&mut terminal)?;
                self.needs_redraw = false;
            }

            // Poll at 60 FPS
            thread::sleep(Duration::from_millis(16));
        }
    }
}
```

### Async Preview Loading

```rust
use tokio::fs;

async fn load_preview_async(path: PathBuf) -> Result<DynamicImage> {
    let bytes = fs::read(path).await?;
    let img = tokio::task::spawn_blocking(move || {
        image::load_from_memory(&bytes)
    }).await??;

    Ok(img)
}
```

---

## Next Steps

1. See `docs/09-rust-project-structure.md` for codebase organization
2. Review `docs/07-rust-python-architecture.md` for communication patterns
3. Check `docs/06-implementation-plan.md` for implementation guide
4. Reference ratatui examples: https://github.com/ratatui/ratatui/tree/main/examples
