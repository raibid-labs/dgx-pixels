//! Screen-specific input handlers.

pub mod comparison;
pub mod generation;
pub mod queue;

pub use comparison::handle_comparison_input;
pub use generation::handle_generation_input;
pub use queue::handle_queue_input;
