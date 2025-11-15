//! # Preview Image Component
//!
//! Represents a preview image that can be attached to job entities.
//!
//! ## Lifecycle
//!
//! 1. PreviewImage is created with `asset_handle: None`
//! 2. The `load_preview_images` system populates the `asset_handle` when a job completes
//! 3. Bevy's asset system loads the image asynchronously
//! 4. The `track_loading_status` system monitors loading progress
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::components::PreviewImage;
//! use std::path::PathBuf;
//!
//! // PreviewImage is typically created by the load_preview_images system,
//! // but can be created manually:
//! fn attach_preview(mut commands: Commands, job_entity: Entity) {
//!     let preview = PreviewImage::new(PathBuf::from("/output/image.png"));
//!     commands.entity(job_entity).insert(preview);
//! }
//! ```

use bevy::prelude::*;
use std::path::PathBuf;

/// Preview image component (attached to job entities).
///
/// The asset_handle field is populated by the `load_preview_images` system
/// when a job completes. Rendering systems should check `is_loaded()` before
/// attempting to render the image.
#[derive(Component, Debug, Clone)]
pub struct PreviewImage {
    /// Path to image file
    pub path: PathBuf,
    /// Bevy asset handle (populated by `load_preview_images` system in WS-06B)
    pub asset_handle: Option<Handle<Image>>,
}

impl PreviewImage {
    /// Create a new preview image component.
    ///
    /// The `asset_handle` field starts as `None` and is populated
    /// by the `load_preview_images` system when the job completes.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            asset_handle: None,
        }
    }

    /// Check if the asset handle has been set.
    ///
    /// Note: This only checks if the handle exists, not if the actual
    /// image data has finished loading. Use the `Assets<Image>` resource
    /// to check if the image data is available.
    pub fn is_loaded(&self) -> bool {
        self.asset_handle.is_some()
    }

    /// Get a reference to the asset handle, if available.
    pub fn handle(&self) -> Option<&Handle<Image>> {
        self.asset_handle.as_ref()
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
        assert!(preview.handle().is_none());
    }

    #[test]
    fn test_preview_with_handle() {
        let handle: Handle<Image> = Handle::default();
        let preview = PreviewImage {
            path: PathBuf::from("/test.png"),
            asset_handle: Some(handle.clone()),
        };

        assert!(preview.is_loaded());
        assert!(preview.handle().is_some());
        assert_eq!(preview.handle().unwrap(), &handle);
    }

    #[test]
    fn test_preview_handle_getter() {
        let mut preview = PreviewImage::new(PathBuf::from("/test.png"));

        // Initially no handle
        assert!(preview.handle().is_none());

        // Add handle
        let handle: Handle<Image> = Handle::default();
        preview.asset_handle = Some(handle.clone());

        // Handle should now be available
        assert!(preview.handle().is_some());
        assert_eq!(preview.handle().unwrap(), &handle);
    }
}
