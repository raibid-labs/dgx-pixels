//! Screen-specific rendering modules.

pub mod comparison;
pub mod gallery;
pub mod generation;
pub mod queue;

pub use comparison::render_comparison_screen;
pub use gallery::render_gallery_screen;
pub use generation::render_generation_screen;
pub use queue::render_queue_screen;
