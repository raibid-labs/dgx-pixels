//! Screen-specific rendering modules.

pub mod comparison;
pub mod queue;

pub use comparison::render_comparison_screen;
pub use queue::render_queue_screen;
