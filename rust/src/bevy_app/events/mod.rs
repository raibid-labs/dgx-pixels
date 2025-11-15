//! # Custom Bevy Events
//!
//! App-specific events for cross-system communication via Bevy's event bus.

pub mod gallery;
pub mod generation;
pub mod navigation;

// Re-export events
pub use gallery::*;
pub use generation::*;
pub use navigation::*;
