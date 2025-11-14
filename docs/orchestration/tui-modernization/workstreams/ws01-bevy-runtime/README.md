# WS-01: Bevy Runtime Setup

**Orchestrator**: Foundation
**Duration**: 3-4 days
**Risk**: Low
**Dependencies**: None (foundation workstream)

## Summary

Establish the Bevy ECS runtime foundation by adding `bevy` and `bevy_ratatui` dependencies, creating the Bevy app structure, and implementing a feature-flagged dual-mode entry point that preserves the old ratatui app while enabling the new Bevy architecture.

## Timeline & Dependencies

### Dependencies
- **Upstream**: None (first workstream)
- **Parallel**: None
- **Downstream**: All other workstreams depend on this

### Timeline
- **Day 1**: Add dependencies, create `bevy_app` module structure
- **Day 2**: Implement Bevy app entry point, configure plugins
- **Day 3**: Create dual-mode main.rs, test both modes
- **Day 4**: Testing, documentation, PR submission

## Scope

### Files Created
```
rust/src/bevy_app/
├── mod.rs              # Module entry point, re-exports
├── plugins.rs          # DgxPixelsPlugin definition
└── config.rs           # Bevy configuration (update rate, plugins)
```

### Files Modified
```
rust/Cargo.toml         # Add bevy, bevy_ratatui dependencies
rust/src/main.rs        # Add feature-gated dual-mode entry
rust/src/lib.rs         # Export bevy_app module (if exists)
```

### Files Removed
None (no deletions in foundation workstream)

## Implementation Details

### Step 1: Add Dependencies

**Edit `rust/Cargo.toml`**:
```toml
[dependencies]
# Existing dependencies...
ratatui = "0.26"
crossterm = "0.27"

# NEW: Bevy dependencies
bevy = { version = "0.16", default-features = false, features = [
    "bevy_asset",
    "bevy_core_pipeline",
    "bevy_render",
    "multi_threaded",
] }
bevy_ratatui = "0.7"

[features]
default = []
bevy_migration_foundation = []  # Feature flag for migration
```

**Rationale**:
- `bevy` with minimal features (no windowing yet)
- `bevy_ratatui` for terminal rendering
- Feature flag enables gradual rollout

### Step 2: Create Bevy App Module

**Create `rust/src/bevy_app/mod.rs`**:
```rust
//! Bevy ECS-based application architecture.
//!
//! This module contains the Bevy app implementation that will replace
//! the old imperative ratatui event loop.

pub mod config;
pub mod plugins;

pub use config::BevyAppConfig;
pub use plugins::DgxPixelsPlugin;
```

**Create `rust/src/bevy_app/config.rs`**:
```rust
use bevy::prelude::*;
use std::time::Duration;

/// Configuration for the Bevy app runtime.
#[derive(Debug, Clone)]
pub struct BevyAppConfig {
    /// Update rate (60 FPS = 16.67ms per frame)
    pub update_rate: Duration,
}

impl Default for BevyAppConfig {
    fn default() -> Self {
        Self {
            update_rate: Duration::from_secs_f32(1.0 / 60.0), // 60 FPS
        }
    }
}

impl BevyAppConfig {
    /// Create config with custom update rate.
    pub fn with_update_rate(mut self, fps: u32) -> Self {
        self.update_rate = Duration::from_secs_f32(1.0 / fps as f32);
        self
    }
}
```

**Create `rust/src/bevy_app/plugins.rs`**:
```rust
use bevy::prelude::*;
use bevy_ratatui::RatatuiPlugins;

use super::BevyAppConfig;

/// Main plugin for DGX-Pixels Bevy app.
pub struct DgxPixelsPlugin;

impl Plugin for DgxPixelsPlugin {
    fn build(&self, app: &mut App) {
        // Configuration
        let config = BevyAppConfig::default();

        // Bevy minimal plugins (no windowing)
        app.add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(config.update_rate))
        );

        // Ratatui terminal rendering
        app.add_plugins(RatatuiPlugins::default());

        // Future: Add state resources (WS-02)
        // Future: Add input systems (WS-03)
        // Future: Add rendering systems (WS-04)

        info!("DgxPixelsPlugin initialized");
    }
}
```

### Step 3: Dual-Mode Main Entry

**Edit `rust/src/main.rs`**:
```rust
use anyhow::Result;

#[cfg(feature = "bevy_migration_foundation")]
use bevy::prelude::*;

#[cfg(feature = "bevy_migration_foundation")]
use dgx_pixels::bevy_app::{DgxPixelsPlugin};

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    #[cfg(feature = "bevy_migration_foundation")]
    {
        info!("Starting Bevy-based DGX-Pixels TUI");
        run_bevy_app()
    }

    #[cfg(not(feature = "bevy_migration_foundation"))]
    {
        info!("Starting classic ratatui DGX-Pixels TUI");
        dgx_pixels::run_classic_app()
    }
}

#[cfg(feature = "bevy_migration_foundation")]
fn run_bevy_app() -> Result<()> {
    App::new()
        .add_plugins(DgxPixelsPlugin)
        .run();

    Ok(())
}
```

**Note**: Assumes old app code moved to `run_classic_app()` in lib.rs.

### Step 4: Verify Functionality

**Test old mode**:
```bash
cargo run
# Should launch classic ratatui app (unchanged)
```

**Test new Bevy mode**:
```bash
cargo run --features bevy_migration_foundation
# Should launch Bevy app with blank terminal (no rendering yet)
```

**Expected behavior**:
- Terminal enters raw mode
- Blank screen displayed
- Quits cleanly on Ctrl+C
- 60 FPS maintained (check logs for frame time)

## Acceptance Criteria

### Functional Requirements
- [ ] `cargo run` launches old ratatui app (no changes)
- [ ] `cargo run --features bevy_migration_foundation` launches Bevy app
- [ ] Terminal enters raw mode in both modes
- [ ] Clean shutdown on Ctrl+C in both modes
- [ ] No panics or errors in either mode

### Performance Requirements
- [ ] Bevy mode maintains 60 FPS (frame time <16ms)
- [ ] No CPU spikes or memory leaks
- [ ] Binary size increase acceptable (<5MB)

### Code Quality
- [ ] All new code documented with rustdoc
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] Feature flag properly gates Bevy code

## Testing Strategy

### Unit Tests

**Add to `rust/src/bevy_app/config.rs`**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BevyAppConfig::default();
        assert_eq!(config.update_rate, Duration::from_secs_f32(1.0 / 60.0));
    }

    #[test]
    fn test_custom_fps() {
        let config = BevyAppConfig::default().with_update_rate(30);
        assert_eq!(config.update_rate, Duration::from_secs_f32(1.0 / 30.0));
    }
}
```

### Integration Tests

**Create `rust/tests/bevy_runtime.rs`**:
```rust
#![cfg(feature = "bevy_migration_foundation")]

use bevy::prelude::*;
use dgx_pixels::bevy_app::DgxPixelsPlugin;

#[test]
fn test_bevy_app_builds() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);

    // Verify app built successfully
    assert!(app.world.resource::<bevy::app::ScheduleRunnerPlugin>().is_ok());
}

#[test]
fn test_bevy_app_updates() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);

    // Run one frame
    app.update();

    // Verify no panics, app still functional
    app.update();
}
```

### Manual Testing

**Checklist**:
```bash
# 1. Old mode works
cargo run
# Navigate UI, verify unchanged

# 2. New mode launches
cargo run --features bevy_migration_foundation
# Verify terminal clears, raw mode active

# 3. Clean shutdown
# Press Ctrl+C in both modes
# Verify terminal restored properly

# 4. Performance
cargo run --features bevy_migration_foundation --release
# Check logs for frame time
# Should be consistently <16ms
```

## Integration Points

### Upstream Dependencies
None (foundation workstream)

### Downstream Consumers
- **WS-02 (ECS State)**: Will add resources to `DgxPixelsPlugin`
- **WS-03 (Input)**: Will add input systems to `DgxPixelsPlugin`
- **WS-04 (Rendering)**: Will add rendering systems to `DgxPixelsPlugin`

### API Contract

**Exported Types**:
```rust
pub struct DgxPixelsPlugin;          // Main plugin
pub struct BevyAppConfig;            // Configuration
```

**Guaranteed Stability**:
- `DgxPixelsPlugin::build()` signature won't change
- `BevyAppConfig::default()` maintains 60 FPS default
- Feature flag `bevy_migration_foundation` remains for entire migration

## Rollback Plan

**If critical issues arise**:
1. Disable feature flag by default
2. Document issues in GitHub issue
3. Fix in hotfix branch
4. Re-enable feature flag in next PR

**Rollback command**:
```bash
# Emergency: Disable Bevy mode
cargo run  # Always falls back to old ratatui
```

**Code rollback**:
- Keep all old code intact (no deletions)
- Feature flag ensures old path still works
- Can ship release with feature disabled

## Documentation

### Code Documentation

**Add to `rust/src/bevy_app/mod.rs`**:
```rust
//! # Bevy-based TUI Architecture
//!
//! This module implements a Bevy ECS-based terminal user interface using
//! the `bevy_ratatui` crate. It replaces the old imperative event loop
//! with a declarative system-based architecture.
//!
//! ## Architecture
//!
//! - **Plugins**: [`DgxPixelsPlugin`] is the main plugin that initializes all systems
//! - **Configuration**: [`BevyAppConfig`] controls update rate and runtime settings
//!
//! ## Feature Flags
//!
//! This module is gated by the `bevy_migration_foundation` feature flag during
//! the migration phase. Once migration completes, this will become the default.
//!
//! ## Example
//!
//! ```rust
//! use bevy::prelude::*;
//! use dgx_pixels::bevy_app::DgxPixelsPlugin;
//!
//! App::new()
//!     .add_plugins(DgxPixelsPlugin)
//!     .run();
//! ```
```

### User Documentation

**Update `README.md`** (add to development section):
```markdown
## Development

### Feature Flags

- `bevy_migration_foundation`: Enable experimental Bevy-based architecture (WIP)

### Running

```bash
# Classic ratatui mode (stable)
cargo run

# Bevy mode (experimental)
cargo run --features bevy_migration_foundation
```
```

### RFD Update

**Mark in RFD 0003**:
```markdown
## WS-01: Bevy Runtime Setup

**Status**: ✅ Complete
**Completed**: [DATE]
**PR**: #[PR_NUMBER]

### Outcomes
- Bevy runtime operational at 60 FPS
- Dual-mode entry point functional
- Zero impact on existing ratatui app
- Foundation ready for state migration (WS-02)
```

## Notes

### Design Decisions

1. **Why MinimalPlugins?**
   - No windowing needed yet (terminal-only)
   - Reduces binary size and startup time
   - WS-18 will add DefaultPlugins for window mode

2. **Why feature flag?**
   - Enables gradual rollout
   - Safe fallback during migration
   - Can disable if issues arise

3. **Why 60 FPS?**
   - Smooth terminal rendering
   - Matches industry standard
   - Low enough to not waste CPU

### Common Pitfalls

1. **Bevy version mismatch**
   - Ensure `bevy` and `bevy_ratatui` versions compatible
   - Check bevy_ratatui docs for supported Bevy version

2. **Terminal not restored on panic**
   - Use `panic = "abort"` in Cargo.toml for clean crashes
   - Add panic handler that restores terminal

3. **Feature flag forgotten**
   - Always test both modes before PR
   - CI should test both paths

### Future Enhancements

- **WS-02**: Add state resources
- **WS-03**: Add input systems
- **WS-04**: Add rendering systems
- **WS-18**: Add DefaultPlugins for window mode

---

**Status**: Ready for Implementation
**Assigned**: TBD
**Branch**: `tui-modernization/ws01-bevy-runtime`
