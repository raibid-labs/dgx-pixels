//! Input handling systems.
//!
//! These systems process keyboard and resize events using bevy_ratatui's
//! message-based event system, replacing the imperative crossterm event loop.

pub mod keyboard;
pub mod navigation;
pub mod text_entry;

pub mod screens;

pub use keyboard::handle_keyboard_input;
pub use navigation::handle_navigation;
pub use screens::*;
pub use text_entry::handle_text_input;
