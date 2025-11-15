//! Screen-specific input handlers.

pub mod comparison;
pub mod queue;
pub mod settings;

pub use comparison::handle_comparison_input;
pub use queue::handle_queue_input;
pub use settings::handle_settings_input;
