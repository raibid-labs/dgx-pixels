//! Screen-specific input handlers.

pub mod comparison;
pub mod gallery;
pub mod generation;
pub mod help;
pub mod models;
pub mod queue;
pub mod settings;

pub use comparison::handle_comparison_input;
pub use gallery::handle_gallery_input;
pub use generation::handle_generation_input;
pub use help::handle_help_input;
pub use models::handle_models_input;
pub use queue::handle_queue_input;
pub use settings::handle_settings_input;
