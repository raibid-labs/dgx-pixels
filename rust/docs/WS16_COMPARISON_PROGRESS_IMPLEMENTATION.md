# WS-16: Real Progress Tracking in Comparison Screen

**Issue**: GitHub #16
**Branch**: `ws16-comparison-progress`
**Status**: Implementation Plan

## Overview

Replace hardcoded 45.0% progress with real-time progress tracking from backend workers in the side-by-side comparison screen.

## Requirements

1. Remove hardcoded `45.0%` progress value (line 465 in `comparison.rs`)
2. Track actual generation progress for each model in comparison
3. Display real-time progress updates from backend ZMQ messages
4. Show different progress for left vs right model
5. Update comparison state to store progress per model

## Implementation Plan

### Step 1: Extend `ComparisonState` Struct

Add field to track job IDs for each model slot:

```rust
pub struct ComparisonState {
    // ... existing fields ...

    /// Job IDs for each model slot (for progress tracking)
    pub model_job_ids: Vec<Option<String>>,
}
```

Update `ComparisonState::new()`:

```rust
pub fn new() -> Self {
    Self {
        // ... existing fields ...
        model_job_ids: vec![None, None, None], // Job IDs for each slot
    }
}
```

Add helper methods:

```rust
impl ComparisonState {
    /// Get job ID for a model slot
    pub fn get_job_id(&self, slot_index: usize) -> Option<&String> {
        self.model_job_ids.get(slot_index).and_then(|opt| opt.as_ref())
    }

    /// Set job ID for a model slot
    pub fn set_job_id(&mut self, slot_index: usize, job_id: String) {
        if slot_index < self.model_job_ids.len() {
            self.model_job_ids[slot_index] = Some(job_id);
        }
    }

    /// Clear all job IDs
    pub fn clear_job_ids(&mut self) {
        self.model_job_ids = vec![None, None, None];
    }
}
```

### Step 2: Create Progress Info Struct

Add internal struct to hold progress information:

```rust
/// Progress info for a model
#[derive(Debug, Clone)]
struct ModelProgress {
    stage: String,
    progress: f32,
    eta_s: f32,
    is_complete: bool,
    is_failed: bool,
    error_msg: Option<String>,
}

impl Default for ModelProgress {
    fn default() -> Self {
        Self {
            stage: "Queued".to_string(),
            progress: 0.0,
            eta_s: 0.0,
            is_complete: false,
            is_failed: false,
            error_msg: None,
        }
    }
}
```

### Step 3: Implement Progress Lookup Function

Add function to get progress from `app.active_jobs`:

```rust
use crate::app::{App, JobStatus};

/// Get progress for a model slot from active jobs
fn get_model_progress(app: &App, state: &ComparisonState, slot_index: usize) -> ModelProgress {
    // Get job ID for this slot
    let job_id = match state.get_job_id(slot_index) {
        Some(id) => id,
        None => return ModelProgress::default(),
    };

    // Find job in active jobs
    let job = app.active_jobs.iter().find(|j| &j.job_id == job_id);

    match job {
        Some(job) => match &job.status {
            JobStatus::Queued => ModelProgress {
                stage: "Queued".to_string(),
                progress: 0.0,
                eta_s: 0.0,
                is_complete: false,
                is_failed: false,
                error_msg: None,
            },
            JobStatus::Running {
                stage,
                progress,
                eta_s,
            } => ModelProgress {
                stage: stage.clone(),
                progress: *progress,
                eta_s: *eta_s,
                is_complete: false,
                is_failed: false,
                error_msg: None,
            },
            JobStatus::Complete { .. } => ModelProgress {
                stage: "Complete".to_string(),
                progress: 100.0,
                eta_s: 0.0,
                is_complete: true,
                is_failed: false,
                error_msg: None,
            },
            JobStatus::Failed { error } => ModelProgress {
                stage: "Failed".to_string(),
                progress: 0.0,
                eta_s: 0.0,
                is_complete: false,
                is_failed: true,
                error_msg: Some(error.clone()),
            },
        },
        None => ModelProgress::default(),
    }
}
```

### Step 4: Update `render_running_mode`

Pass `app` parameter to `render_progress_grid`:

```rust
fn render_running_mode(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App,  // ADD THIS
    state: &ComparisonState,
    comparison_id: &str,
) {
    // ... existing code ...

    // Progress for each model
    render_progress_grid(f, body_chunks[1], app, state, comparison_id); // Pass app
}
```

And update the main `render` function call:

```rust
pub fn render(f: &mut Frame, app: &App, state: &ComparisonState) {
    // ... existing code ...

    match &state.mode {
        // ... other cases ...
        ComparisonMode::Running { comparison_id } => {
            render_running_mode(f, chunks[1], app, state, comparison_id);  // Pass app
        }
        // ... other cases ...
    }
}
```

### Step 5: Update `render_progress_grid`

Add `app` parameter and use real progress:

```rust
fn render_progress_grid(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App,  // ADD THIS
    state: &ComparisonState,
    _comparison_id: &str,
) {
    // ... existing layout code ...

    for (i, model_opt) in state.selected_models.iter().enumerate() {
        if let Some(model) = model_opt {
            let progress_info = get_model_progress(app, state, i); // Get real progress
            render_model_progress(f, progress_chunks[i], model, &progress_info);  // Pass struct
        }
    }
}
```

### Step 6: Update `render_model_progress`

Change signature to accept `ModelProgress` struct instead of `f32`:

```rust
fn render_model_progress(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    model: &ModelConfig,
    progress_info: &ModelProgress,  // CHANGED from f32
) {
    // ... layout code ...

    // Progress bar with appropriate color
    let (gauge_color, label) = if progress_info.is_failed {
        (Color::Red, format!("Failed: {}", progress_info.error_msg.as_ref().unwrap_or(&"Unknown".to_string())))
    } else if progress_info.is_complete {
        (Color::Green, "Complete 100%".to_string())
    } else {
        (Color::Cyan, format!("{:.1}%", progress_info.progress))
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(gauge_color))
        .percent((progress_info.progress as u16).min(100))
        .label(label);
    f.render_widget(gauge, progress_chunks[0]);

    // Stage and ETA info
    let stage_eta_text = if progress_info.is_failed {
        "Generation failed".to_string()
    } else if progress_info.is_complete {
        "Ready to view".to_string()
    } else if progress_info.eta_s > 0.0 {
        format!("{} - ETA: {:.1}s", progress_info.stage, progress_info.eta_s)
    } else {
        progress_info.stage.clone()
    };

    let stage_para = Paragraph::new(stage_eta_text)
        .style(Style::default().fg(Theme::muted()))
        .alignment(Alignment::Center);
    f.render_widget(stage_para, progress_chunks[1]);

    // Preview placeholder with status
    let preview_text = if progress_info.is_complete {
        "Generation complete!\nView results next..."
    } else if progress_info.is_failed {
        "Generation failed.\nCheck error above."
    } else {
        "[Preview]\n\nGenerating..."
    };

    let preview = Paragraph::new(preview_text)
        .style(if progress_info.is_failed {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Theme::muted())
        })
        .alignment(Alignment::Center);
    f.render_widget(preview, chunks[2]);
}
```

### Step 7: Update Event Handlers

When starting a comparison, store job IDs in the state:

```rust
// In the event handler that starts a comparison:
if let ComparisonMode::Setup = state.mode {
    if state.can_compare() {
        // Create comparison
        let comparison_id = state.comparison_manager.create_comparison(
            state.params.clone(),
            state.selected_models.iter().filter_map(|m| m.clone()).collect(),
        );

        // Submit jobs to backend and store job IDs
        let mut job_ids = Vec::new();
        for (i, model_opt) in state.selected_models.iter().enumerate() {
            if let Some(model) = model_opt {
                // Submit to backend (implementation depends on ZMQ client)
                let job_id = submit_generation_job(model, &state.params);
                state.set_job_id(i, job_id.clone());
                job_ids.push(job_id);
            }
        }

        // Register jobs with comparison manager
        state.comparison_manager.register_jobs(&comparison_id, job_ids);

        // Switch to running mode
        state.mode = ComparisonMode::Running { comparison_id };
    }
}
```

### Step 8: Add Tests

Add tests to verify job tracking:

```rust
#[test]
fn test_job_id_tracking() {
    let mut state = ComparisonState::new();

    // Set job IDs for each slot
    state.set_job_id(0, "job-001".to_string());
    state.set_job_id(1, "job-002".to_string());
    state.set_job_id(2, "job-003".to_string());

    // Verify retrieval
    assert_eq!(state.get_job_id(0), Some(&"job-001".to_string()));
    assert_eq!(state.get_job_id(1), Some(&"job-002".to_string()));
    assert_eq!(state.get_job_id(2), Some(&"job-003".to_string()));

    // Clear job IDs
    state.clear_job_ids();
    assert_eq!(state.get_job_id(0), None);
    assert_eq!(state.get_job_id(1), None);
    assert_eq!(state.get_job_id(2), None);
}

#[test]
fn test_model_progress_default() {
    let progress = ModelProgress::default();
    assert_eq!(progress.stage, "Queued");
    assert_eq!(progress.progress, 0.0);
    assert!(!progress.is_complete);
    assert!(!progress.is_failed);
}
```

## Theme Style Fixes Required

The codebase has inconsistent usage of `Theme` methods. Many places call `Theme::muted()` which returns `Color`, but use it where `Style` is expected. These need to be fixed:

**Pattern to fix**:
```rust
// WRONG:
Span::styled("text", Theme::muted())

// CORRECT:
Span::styled("text", Style::default().fg(Theme::muted()))
```

**Affected lines** in `comparison.rs`:
- Line 217, 245, 264, 299-300, 305, 308, 316, 349-351, 373, 509, 573, 585, 590, 593, 617, 650

These should be updated in a separate commit for code cleanup.

## Files Modified

- `/home/beengud/raibid-labs/dgx-pixels/rust/src/ui/screens/comparison.rs`

## Testing Plan

1. **Unit Tests**: Run new tests for job ID tracking and progress struct
2. **Manual Testing**:
   - Start side-by-side comparison with 2+ models
   - Verify each model shows independent progress
   - Verify progress updates in real-time (not hardcoded)
   - Check that ETA is displayed correctly
   - Verify completion and failure states display correctly
   - Confirm different models can have different progress percentages

## Dependencies

This implementation depends on:
- `app::ActiveJob` with `job_id` and `status` fields
- `app::JobStatus` enum with variants: `Queued`, `Running`, `Complete`, `Failed`
- ZMQ backend sending progress updates that populate `app.active_jobs`

## Notes

- The original hardcoded `45.0%` at line 465 is removed entirely
- Progress is now dynamically looked up from `app.active_jobs` based on stored job IDs
- Each model slot (left/right/center) tracks its own job ID independently
- The UI refreshes automatically as `app.active_jobs` is updated by the ZMQ client
- Failed jobs show red progress bars with error messages
- Completed jobs show green 100% progress bars

## Future Enhancements

- Add preview image updates during generation (using Sixel)
- Show intermediate generation steps (denoising progress)
- Add ability to cancel individual model generations
- Display generation speed (iter/s) alongside ETA
