//! # Preview Image Component
//!
//! Represents a preview image that can be attached to job entities.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::components::PreviewImage;
//! use std::path::PathBuf;
//!
//! fn attach_preview(mut commands: Commands, job_entity: Entity) {
//!     let preview = PreviewImage::new(PathBuf::from("/output/image.png"));
//!     commands.entity(job_entity).insert(preview);
//! }
//! ```

use bevy::prelude::*;
use std::path::PathBuf;

/// Preview image component (attached to job entities).
#[derive(Component, Debug, Clone)]
pub struct PreviewImage {
    /// Path to image file
    pub path: PathBuf,
    /// Bevy asset handle (populated by WS-06 Image Asset System)
    pub asset_handle: Option<Handle<Image>>,
}

impl PreviewImage {
    /// Create a new preview image component.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            asset_handle: None,
        }
    }

    /// Check if the asset has been loaded.
    pub fn is_loaded(&self) -> bool {
        self.asset_handle.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_preview() {
        let preview = PreviewImage::new(PathBuf::from("/test.png"));
        assert_eq!(preview.path, PathBuf::from("/test.png"));
        assert!(preview.asset_handle.is_none());
        assert!(!preview.is_loaded());
    }
}
