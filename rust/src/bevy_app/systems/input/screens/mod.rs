//! Screen-specific input handlers.

pub mod comparison;
pub mod queue;

pub use comparison::handle_comparison_input;
pub use queue::handle_queue_input;
