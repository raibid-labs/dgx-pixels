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
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::DgxPixelsPlugin;
//!
//! App::new()
//!     .add_plugins(DgxPixelsPlugin)
//!     .run();
//! ```

pub mod config;
pub mod plugins;

pub use config::BevyAppConfig;
pub use plugins::DgxPixelsPlugin;
