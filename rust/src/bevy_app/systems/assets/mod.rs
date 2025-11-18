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
//!
//! ## Performance Targets
//!
//! - Image loading: <500ms per image
//! - Cache: LRU eviction to prevent memory leaks
//! - Support: PNG, JPG, WebP formats

pub mod cache;
pub mod loader;
pub mod render;

pub use cache::ImageCache;
pub use loader::load_preview_images;
pub use render::render_image_to_ascii;
