//! # Preview Systems
//!
//! Systems for managing image preview loading and tracking.
//!
//! ## Systems
//!
//! - [`load_preview_images`]: Loads images when jobs complete
//! - [`track_loading_status`]: Tracks asset loading status
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::systems::preview::{load_preview_images, track_loading_status};
//!
//! App::new()
//!     .add_systems(Update, (load_preview_images, track_loading_status));
//! ```

pub mod loader;
pub mod tracker;

pub use loader::load_preview_images;
pub use tracker::track_loading_status;
