# WS-02: ECS State Migration

**Orchestrator**: Foundation
**Duration**: 4-5 days
**Risk**: Medium
**Dependencies**: WS-01 (Bevy Runtime)

## Summary

Decompose the monolithic `App` struct into Bevy Resources and Components, establishing the ECS-based state management foundation. This workstream migrates all application state from imperative structs to declarative ECS patterns.

## Timeline & Dependencies

### Dependencies
- **Upstream**: WS-01 (Bevy Runtime Setup)
- **Parallel**: None (blocks WS-03, WS-04)
- **Downstream**: WS-03, WS-04, WS-05, WS-07, WS-08

### Timeline
- **Day 1**: Analyze old App struct, design resource decomposition
- **Day 2**: Create resource types (screen, input, gallery, jobs)
- **Day 3**: Create component types, implement state init systems
- **Day 4**: Implement App → Bevy conversion, write tests
- **Day 5**: Testing, documentation, PR submission

## Scope

### Files Created
```
rust/src/bevy_app/resources/
├── mod.rs
├── app_state.rs          # Main app state resource
├── screen_state.rs       # Current screen tracking
├── input_state.rs        # Input buffer, cursor
├── job_state.rs          # Active jobs tracking
└── gallery_state.rs      # Gallery images

rust/src/bevy_app/components/
├── mod.rs
├── job.rs                # Job component
└── preview.rs            # Preview component

rust/src/bevy_app/systems/
├── mod.rs
└── state_init.rs         # State initialization system
```

### Files Modified
```
rust/src/app.rs                    # Add `impl From<App> for BevyResources`
rust/src/bevy_app/mod.rs           # Export resources, components
rust/src/bevy_app/plugins.rs       # Register state resources + init systems
```

### Files Removed
None (old App struct kept for fallback)

## Implementation Details

### Step 1: Analyze Old State

**Review `rust/src/app.rs`** (existing structure):
```rust
pub struct App {
    // Screen management
    current_screen: Screen,

    // Input handling
    input_buffer: String,
    cursor_pos: usize,

    // Job tracking
    active_jobs: Vec<ActiveJob>,
    completed_jobs: Vec<CompletedJob>,

    // Gallery
    gallery_images: Vec<PathBuf>,
    selected_image: usize,

    // Models
    available_models: Vec<ModelInfo>,
    selected_model: String,

    // Queue
    queue_jobs: Vec<QueuedJob>,

    // Monitor
    gpu_stats: GpuStats,
    system_stats: SystemStats,

    // Settings
    settings: AppSettings,

    // ... more fields
}
```

**Decomposition strategy**:
- Global singletons → Resources
- Per-job/per-image data → Components on entities
- Stateless logic → Systems

### Step 2: Create Resources

**Create `rust/src/bevy_app/resources/mod.rs`**:
```rust
//! Application state resources.

pub mod app_state;
pub mod screen_state;
pub mod input_state;
pub mod job_state;
pub mod gallery_state;

pub use app_state::AppState;
pub use screen_state::CurrentScreen;
pub use input_state::InputBuffer;
pub use job_state::JobTracker;
pub use gallery_state::GalleryState;
```

**Create `rust/src/bevy_app/resources/screen_state.rs`**:
```rust
use bevy::prelude::*;

/// Current active screen.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentScreen(pub Screen);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Generation,
    Gallery,
    Comparison,
    Models,
    Queue,
    Monitor,
    Settings,
    Help,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        Self(Screen::Generation)
    }
}

impl Screen {
    /// Navigate to next screen (Tab key).
    pub fn next(self) -> Self {
        use Screen::*;
        match self {
            Generation => Gallery,
            Gallery => Comparison,
            Comparison => Models,
            Models => Queue,
            Queue => Monitor,
            Monitor => Settings,
            Settings => Help,
            Help => Generation,
        }
    }

    /// Navigate to previous screen (Shift+Tab).
    pub fn previous(self) -> Self {
        use Screen::*;
        match self {
            Generation => Help,
            Gallery => Generation,
            Comparison => Gallery,
            Models => Comparison,
            Queue => Models,
            Monitor => Queue,
            Settings => Monitor,
            Help => Settings,
        }
    }
}
```

**Create `rust/src/bevy_app/resources/input_state.rs`**:
```rust
use bevy::prelude::*;

/// Input buffer state.
#[derive(Resource, Debug, Clone)]
pub struct InputBuffer {
    /// Current text input
    pub text: String,
    /// Cursor position (character index)
    pub cursor: usize,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }
}

impl InputBuffer {
    /// Insert character at cursor position.
    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Delete character before cursor.
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }

    /// Clear buffer and reset cursor.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }
}
```

**Create `rust/src/bevy_app/resources/gallery_state.rs`**:
```rust
use bevy::prelude::*;
use std::path::PathBuf;

/// Gallery state resource.
#[derive(Resource, Debug, Clone)]
pub struct GalleryState {
    /// All images in gallery
    pub images: Vec<PathBuf>,
    /// Currently selected image index
    pub selected: usize,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            selected: 0,
        }
    }
}

impl GalleryState {
    /// Add image to gallery.
    pub fn add_image(&mut self, path: PathBuf) {
        self.images.push(path);
    }

    /// Select next image.
    pub fn select_next(&mut self) {
        if !self.images.is_empty() {
            self.selected = (self.selected + 1) % self.images.len();
        }
    }

    /// Select previous image.
    pub fn select_previous(&mut self) {
        if !self.images.is_empty() {
            self.selected = if self.selected == 0 {
                self.images.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Get currently selected image path.
    pub fn current_image(&self) -> Option<&PathBuf> {
        self.images.get(self.selected)
    }
}
```

**Create `rust/src/bevy_app/resources/job_state.rs`**:
```rust
use bevy::prelude::*;

/// Job tracking resource.
#[derive(Resource, Debug, Clone, Default)]
pub struct JobTracker {
    /// Total jobs submitted this session
    pub total_submitted: usize,
    /// Total jobs completed
    pub total_completed: usize,
    /// Total jobs failed
    pub total_failed: usize,
}

impl JobTracker {
    pub fn submit_job(&mut self) {
        self.total_submitted += 1;
    }

    pub fn complete_job(&mut self) {
        self.total_completed += 1;
    }

    pub fn fail_job(&mut self) {
        self.total_failed += 1;
    }

    pub fn active_jobs(&self) -> usize {
        self.total_submitted - self.total_completed - self.total_failed
    }
}
```

### Step 3: Create Components

**Create `rust/src/bevy_app/components/job.rs`**:
```rust
use bevy::prelude::*;
use std::path::PathBuf;

/// Job entity component.
#[derive(Component, Debug, Clone)]
pub struct Job {
    /// Unique job ID
    pub id: String,
    /// Generation prompt
    pub prompt: String,
    /// Job status
    pub status: JobStatus,
    /// Submission timestamp
    pub submitted_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum JobStatus {
    /// Job submitted, waiting for acceptance
    Pending,
    /// Job accepted by backend, queued
    Queued,
    /// Job currently generating
    Generating {
        progress: f32,  // 0.0 - 1.0
    },
    /// Job completed successfully
    Complete {
        image_path: PathBuf,
        duration_s: f32,
    },
    /// Job failed
    Failed {
        error: String,
    },
}

impl Job {
    pub fn new(id: String, prompt: String) -> Self {
        Self {
            id,
            prompt,
            status: JobStatus::Pending,
            submitted_at: std::time::Instant::now(),
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, JobStatus::Complete { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, JobStatus::Failed { .. })
    }
}
```

**Create `rust/src/bevy_app/components/preview.rs`**:
```rust
use bevy::prelude::*;
use std::path::PathBuf;

/// Preview image component (attached to job entities).
#[derive(Component, Debug, Clone)]
pub struct PreviewImage {
    /// Path to image file
    pub path: PathBuf,
    /// Bevy asset handle (populated by WS-06)
    pub asset_handle: Option<Handle<Image>>,
}

impl PreviewImage {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            asset_handle: None,
        }
    }
}
```

### Step 4: State Initialization System

**Create `rust/src/bevy_app/systems/state_init.rs`**:
```rust
use bevy::prelude::*;

use crate::bevy_app::resources::*;

/// Initialize all application state resources.
pub fn init_app_state(mut commands: Commands) {
    info!("Initializing application state resources");

    commands.insert_resource(CurrentScreen::default());
    commands.insert_resource(InputBuffer::default());
    commands.insert_resource(GalleryState::default());
    commands.insert_resource(JobTracker::default());

    // Future: Add more resources as needed
}
```

### Step 5: Register Systems in Plugin

**Update `rust/src/bevy_app/plugins.rs`**:
```rust
use bevy::prelude::*;
use bevy_ratatui::RatatuiPlugins;

use super::systems::state_init;

pub struct DgxPixelsPlugin;

impl Plugin for DgxPixelsPlugin {
    fn build(&self, app: &mut App) {
        // ... existing plugin setup

        // WS-02: State initialization
        app.add_systems(Startup, state_init::init_app_state);

        info!("DgxPixelsPlugin initialized with state resources");
    }
}
```

### Step 6: Conversion from Old App

**Update `rust/src/app.rs`**:
```rust
#[cfg(feature = "bevy_migration_foundation")]
impl App {
    /// Convert old App to Bevy resources.
    pub fn into_bevy_resources(self, world: &mut bevy::prelude::World) {
        use crate::bevy_app::resources::*;

        // Screen state
        world.insert_resource(CurrentScreen(self.current_screen));

        // Input state
        world.insert_resource(InputBuffer {
            text: self.input_buffer,
            cursor: self.cursor_pos,
        });

        // Gallery state
        world.insert_resource(GalleryState {
            images: self.gallery_images,
            selected: self.selected_image,
        });

        // Job tracker
        world.insert_resource(JobTracker {
            total_submitted: self.active_jobs.len() + self.completed_jobs.len(),
            total_completed: self.completed_jobs.len(),
            total_failed: 0,  // Not tracked in old app
        });

        // Convert active jobs to entities
        for job in self.active_jobs {
            world.spawn(components::Job {
                id: job.id,
                prompt: job.prompt,
                status: job.status.into(),  // Convert old status to new
                submitted_at: job.submitted_at,
            });
        }
    }
}
```

## Acceptance Criteria

### Functional Requirements
- [ ] All old `App` fields mapped to resources or components
- [ ] State initialization system populates resources
- [ ] Conversion function `App → Bevy resources` works
- [ ] All resources accessible in Bevy systems
- [ ] No runtime panics accessing resources

### Code Quality
- [ ] All resources derive `Resource`
- [ ] All components derive `Component`
- [ ] Default implementations provided
- [ ] Rustdoc on all public types

### Testing
- [ ] Unit tests for all resource methods
- [ ] Integration test for state initialization
- [ ] Conversion test (old App → Bevy)

## Testing Strategy

### Unit Tests

**In each resource file**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_navigation() {
        let mut screen = CurrentScreen::default();
        assert_eq!(screen.0, Screen::Generation);

        screen.0 = screen.0.next();
        assert_eq!(screen.0, Screen::Gallery);

        screen.0 = screen.0.previous();
        assert_eq!(screen.0, Screen::Generation);
    }

    #[test]
    fn test_input_buffer() {
        let mut buffer = InputBuffer::default();

        buffer.insert('a');
        buffer.insert('b');
        assert_eq!(buffer.text, "ab");
        assert_eq!(buffer.cursor, 2);

        buffer.backspace();
        assert_eq!(buffer.text, "a");
        assert_eq!(buffer.cursor, 1);
    }

    #[test]
    fn test_gallery_state() {
        let mut gallery = GalleryState::default();

        gallery.add_image("image1.png".into());
        gallery.add_image("image2.png".into());

        assert_eq!(gallery.images.len(), 2);
        assert_eq!(gallery.selected, 0);

        gallery.select_next();
        assert_eq!(gallery.selected, 1);

        gallery.select_next();
        assert_eq!(gallery.selected, 0);  // Wraps around
    }
}
```

### Integration Tests

**Create `rust/tests/state_migration.rs`**:
```rust
#![cfg(feature = "bevy_migration_foundation")]

use bevy::prelude::*;
use dgx_pixels::bevy_app::{DgxPixelsPlugin, resources::*};

#[test]
fn test_state_initialization() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);

    // Verify all resources initialized
    assert!(app.world.contains_resource::<CurrentScreen>());
    assert!(app.world.contains_resource::<InputBuffer>());
    assert!(app.world.contains_resource::<GalleryState>());
    assert!(app.world.contains_resource::<JobTracker>());
}

#[test]
fn test_app_conversion() {
    use dgx_pixels::app::App as OldApp;

    let old_app = OldApp::new();

    let mut bevy_app = App::new();
    old_app.into_bevy_resources(&mut bevy_app.world);

    // Verify resources populated
    let screen = bevy_app.world.resource::<CurrentScreen>();
    assert_eq!(screen.0, Screen::Generation);

    let buffer = bevy_app.world.resource::<InputBuffer>();
    assert_eq!(buffer.text, "");
}
```

## Integration Points

### Upstream Dependencies
- **WS-01**: Bevy runtime must be operational

### Downstream Consumers
- **WS-03**: Input systems will mutate resources
- **WS-04**: Rendering systems will read resources
- **WS-05**: ZeroMQ systems will update job components
- **WS-07**: Theme resource will be added
- **WS-08**: Event systems will use resources

### API Contract

**Guaranteed exports**:
```rust
// Resources
pub use resources::{
    CurrentScreen,
    InputBuffer,
    GalleryState,
    JobTracker,
};

// Components
pub use components::{
    Job,
    PreviewImage,
};
```

**Stability promise**: Resource types won't change during migration

## Documentation

### Code Documentation

**Add to each resource file**:
```rust
//! # [Resource Name]
//!
//! Description of what this resource tracks.
//!
//! ## Example
//!
//! ```rust
//! use dgx_pixels::bevy_app::resources::CurrentScreen;
//!
//! fn my_system(screen: Res<CurrentScreen>) {
//!     println!("Current screen: {:?}", screen.0);
//! }
//! ```
```

### RFD Update

**Mark in RFD 0003**:
```markdown
## WS-02: ECS State Migration

**Status**: ✅ Complete
**Completed**: [DATE]
**PR**: #[PR_NUMBER]

### Outcomes
- Monolithic App decomposed into X resources, Y components
- State initialization system operational
- Conversion function tested and working
- Foundation ready for input/rendering systems
```

## Notes

### Design Decisions

1. **Why separate resources per concern?**
   - Better system parallelization (Bevy scheduler)
   - Clearer ownership and responsibilities
   - Easier to test in isolation

2. **Why components for jobs?**
   - Jobs are entities (can query/filter)
   - Enables per-job lifecycle management
   - Natural fit for Bevy's ECS model

3. **Why keep old App struct?**
   - Fallback during migration
   - Conversion function needs old data
   - Delete after migration complete

---

**Status**: Ready for Implementation
**Assigned**: TBD
**Branch**: `tui-modernization/ws02-ecs-state`
