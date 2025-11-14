//! Application state resources.
//!
//! This module contains Bevy Resources that replace the monolithic `App` struct
//! from the classic ratatui implementation. Each resource manages a specific
//! aspect of application state, enabling better parallelization and clearer
//! separation of concerns.

pub mod app_state;
pub mod gallery_state;
pub mod input_state;
pub mod job_state;
pub mod screen_state;

pub use app_state::AppState;
pub use gallery_state::GalleryState;
pub use input_state::InputBuffer;
pub use job_state::JobTracker;
pub use screen_state::{CurrentScreen, Screen};
