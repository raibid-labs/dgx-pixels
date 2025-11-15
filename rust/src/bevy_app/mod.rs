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
//! - **Resources**: Global singleton state (screen, input, gallery, etc.)
//! - **Components**: Per-entity data (jobs, previews)
//! - **Systems**: Logic that operates on resources and components
//!
//! ## Feature Flags
//!
//! This module is gated by the `bevy_migration_foundation` feature flag during
//! the migration phase. Once migration completes, this will become the default.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::DgxPixelsPlugin;
//!
//! App::new()
//!     .add_plugins(DgxPixelsPlugin)
//!     .run();
//! ```

pub mod components;
pub mod config;
pub mod events;
pub mod plugins;
pub mod resources;
pub mod systems;

pub use config::BevyAppConfig;
pub use plugins::DgxPixelsPlugin;

// Re-export commonly used types
pub use components::{Job, JobStatus, PreviewImage};
pub use events::{
    CancelJob, DeleteImage, GenerationComplete, NavigateBack, NavigateToScreen,
    SelectNextImage, SelectPreviousImage, SubmitGenerationJob,
};
pub use resources::{AppState, CurrentScreen, GalleryState, InputBuffer, JobTracker, Screen};
