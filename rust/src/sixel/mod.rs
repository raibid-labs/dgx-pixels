//! Sixel image rendering module
//!
//! Provides real-time image preview using Sixel graphics protocol
//! for terminals that support it (kitty, WezTerm, iTerm2, xterm).

mod image_renderer;
mod preview_manager;
mod terminal_detection;

pub use image_renderer::RenderOptions;
pub use preview_manager::PreviewManager;
pub use terminal_detection::{detect_sixel_support, TerminalCapability};

/// Maximum preview cache size in MB
pub const MAX_CACHE_SIZE_MB: usize = 50;

/// Default preview quality (1-100)
#[allow(dead_code)]
pub const DEFAULT_PREVIEW_QUALITY: u8 = 85;

/// Maximum colors for Sixel (256 for best terminal compatibility)
#[allow(dead_code)]
pub const MAX_SIXEL_COLORS: usize = 256;
