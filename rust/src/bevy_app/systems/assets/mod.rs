//! # Asset Loading Systems
//!
//! WS-06: Image Asset System
//! Replaces Sixel preview with Bevy AssetServer-based image loading.
//!
//! ## Architecture
//!
//! - **loader.rs**: System to load images from filesystem as Bevy assets
//! - **cache.rs**: LRU cache management for loaded images
//! - **render.rs**: Image rendering utilities for ratatui integration
//! - **sixel_renderer.rs**: Sixel protocol encoding (T9)
//! - **preview.rs**: Sixel preview caching system (T9)
//! - **preview_loader.rs**: (T10) Automatic gallery directory scanning and preview management
//!
//! ## Performance Targets
//!
//! - Image loading: <500ms per image
//! - Cache: LRU eviction to prevent memory leaks
//! - Support: PNG, JPG, WebP formats
//! - Sixel rendering: <100ms per image (cached)
//! - Directory scan: Every 2 seconds for new images

pub mod cache;
pub mod loader;
pub mod preview;
pub mod preview_loader;
pub mod render;
pub mod sixel_renderer;

pub use cache::ImageCache;
pub use loader::load_preview_images;
pub use preview::{SixelCacheEntry, SixelCacheStats, SixelPreviewCache};
pub use preview_loader::{
    check_preview_loading, preload_gallery_directory, scan_gallery_directory, GalleryScanState,
    DEFAULT_GALLERY_DIR, SCAN_INTERVAL_SECS,
};
pub use render::render_image_to_ascii;
pub use sixel_renderer::{render_image_sixel, supports_sixel, SixelRenderOptions};
