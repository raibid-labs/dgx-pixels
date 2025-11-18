//! # Image Asset Loader
//!
//! System to load images from the filesystem as Bevy assets and attach
//! them to entities with PreviewImage components.

use bevy::prelude::*;
use std::path::PathBuf;
use tracing::{debug, warn};

use crate::bevy_app::components::{Job, JobStatus, PreviewImage};
use crate::bevy_app::resources::GalleryState;

/// System to load preview images for completed jobs.
///
/// Listens for jobs with Complete status and loads their images
/// as Bevy assets, attaching PreviewImage components.
pub fn load_preview_images(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    job_query: Query<(Entity, &Job), Changed<Job>>,
) {
    for (entity, job) in job_query.iter() {
        if let JobStatus::Complete { image_path, .. } = &job.status {
            debug!("Loading preview image for job {:?}: {:?}", entity, image_path);

            // Load image as Bevy asset
            let handle: Handle<Image> = asset_server.load(image_path.clone());

            // Create or update PreviewImage component
            let preview = PreviewImage {
                path: image_path.clone(),
                asset_handle: Some(handle),
            };

            commands.entity(entity).insert(preview);
        }
    }
}

/// System to load gallery images on demand.
///
/// Loads images from GalleryState into Bevy's asset system for rendering.
/// Creates temporary entities to hold the image handles.
pub fn load_gallery_images(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gallery: Res<GalleryState>,
    existing_query: Query<&PreviewImage>,
) {
    // Only load if gallery changed
    if !gallery.is_changed() {
        return;
    }

    // Load all gallery images
    for path in &gallery.images {
        // Check if already loaded
        let already_loaded = existing_query
            .iter()
            .any(|preview| preview.path == *path);

        if !already_loaded {
            debug!("Loading gallery image: {:?}", path);

            // Verify file exists before loading
            if !path.exists() {
                warn!("Gallery image does not exist: {:?}", path);
                continue;
            }

            let handle: Handle<Image> = asset_server.load(path.clone());

            let preview = PreviewImage {
                path: path.clone(),
                asset_handle: Some(handle),
            };

            // Spawn entity with preview component
            commands.spawn(preview);
        }
    }
}

/// System to check asset loading status and log warnings for failures.
pub fn check_asset_loading(
    asset_server: Res<AssetServer>,
    preview_query: Query<&PreviewImage>,
) {
    for preview in preview_query.iter() {
        if let Some(handle) = &preview.asset_handle {
            match asset_server.load_state(handle) {
                bevy::asset::LoadState::Failed(err) => {
                    warn!("Failed to load image {:?}: {:?}", preview.path, err);
                }
                bevy::asset::LoadState::Loaded => {
                    debug!("Successfully loaded image: {:?}", preview.path);
                }
                _ => {
                    // Still loading or not started
                }
            }
        }
    }
}

/// Helper function to preload a single image path.
///
/// Returns the asset handle immediately. The image will load asynchronously.
pub fn preload_image(asset_server: &AssetServer, path: &PathBuf) -> Handle<Image> {
    debug!("Preloading image: {:?}", path);
    asset_server.load(path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_image_component() {
        let path = PathBuf::from("/test/image.png");
        let preview = PreviewImage::new(path.clone());

        assert_eq!(preview.path, path);
        assert!(!preview.is_loaded());
    }

    #[test]
    fn test_preview_image_with_handle() {
        let mut app = App::new();
        app.add_plugins(AssetPlugin::default());

        let asset_server = app.world().resource::<AssetServer>();
        let handle = asset_server.load("test.png");

        let preview = PreviewImage {
            path: PathBuf::from("test.png"),
            asset_handle: Some(handle),
        };

        assert!(preview.is_loaded());
    }
}
