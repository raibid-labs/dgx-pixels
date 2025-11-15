//! # Image Asset System
//!
//! This module provides image asset loading and caching infrastructure for the
//! Bevy-based TUI. It replaces the Sixel preview system with GPU-accelerated
//! image rendering using Bevy's asset system.
//!
//! ## Architecture
//!
//! - **Image Loader**: Async image loading using Bevy's `AssetServer`
//! - **Cache Manager**: LRU cache with configurable memory limits
//! - **Format Support**: PNG, JPG, WebP formats
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::assets::{ImageAssetPlugin, load_image};
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     let handle = load_image(&asset_server, "output/sprite.png");
//!     // Use handle in components...
//! }
//! ```
//!
//! ## Memory Management
//!
//! The cache system automatically evicts least-recently-used assets when the
//! configured memory limit is reached. Default limit: 512MB.

pub mod cache;
pub mod image_loader;

pub use cache::{AssetCache, CacheConfig, CacheStats};
pub use image_loader::{load_image, ImageAssetPlugin, ImageLoadError, ImageLoader};
