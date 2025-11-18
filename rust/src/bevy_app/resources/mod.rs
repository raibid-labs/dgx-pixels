//! Application state resources.
//!
//! This module contains Bevy Resources that replace the monolithic `App` struct
//! from the classic ratatui implementation. Each resource manages a specific
//! aspect of application state, enabling better parallelization and clearer
//! separation of concerns.

pub mod app_state;
pub mod comparison_state;
pub mod gallery_state;
pub mod help_state;
pub mod input_state;
pub mod job_state;
pub mod models;
pub mod screen_state;
pub mod settings;
pub mod theme;

pub use app_state::AppState;
pub use comparison_state::ComparisonState;
pub use gallery_state::GalleryState;
pub use help_state::HelpState;
pub use input_state::InputBuffer;
pub use job_state::JobTracker;
pub use models::ModelsState;
pub use screen_state::{CurrentScreen, Screen};
pub use settings::SettingsState;
pub use theme::AppTheme;
