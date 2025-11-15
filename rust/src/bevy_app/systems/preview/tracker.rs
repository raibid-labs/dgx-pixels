//! # Preview Loading Status Tracker
//!
//! System that tracks the loading status of preview image assets.
//!
//! ## How It Works
//!
//! 1. Queries for entities with PreviewImage components
//! 2. Checks the loading state of each asset handle
//! 3. Logs loading status and errors
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::systems::preview::track_loading_status;
//!
//! App::new()
//!     .add_systems(Update, track_loading_status);
//! ```

use bevy::prelude::*;

use crate::bevy_app::components::PreviewImage;

/// System that tracks asset loading status for preview images.
///
/// This system:
/// - Queries all entities with PreviewImage components
/// - Checks the load state of each asset using AssetServer
/// - Logs when assets finish loading or encounter errors
///
/// ## Asset Loading States
///
/// Bevy assets can be in several states:
/// - `NotLoaded`: Asset hasn't started loading
/// - `Loading`: Asset is currently loading
/// - `Loaded`: Asset loaded successfully
/// - `Failed`: Asset loading failed
///
/// ## Usage
///
/// This system provides observability for the preview loading pipeline.
/// It's useful for debugging and monitoring asset loading progress.
pub fn track_loading_status(
    asset_server: Res<AssetServer>,
    preview_query: Query<(Entity, &PreviewImage), Changed<PreviewImage>>,
) {
    for (entity, preview) in preview_query.iter() {
        if let Some(handle) = &preview.asset_handle {
            match asset_server.get_load_state(handle) {
                Some(bevy::asset::LoadState::Loaded) => {
                    debug!(
                        "Preview image loaded for entity {:?}: {}",
                        entity,
                        preview.path.display()
                    );
                }
                Some(bevy::asset::LoadState::Failed(err)) => {
                    error!(
                        "Failed to load preview image for entity {:?}: {} - Error: {:?}",
                        entity,
                        preview.path.display(),
                        err
                    );
                }
                Some(bevy::asset::LoadState::Loading) => {
                    trace!(
                        "Preview image loading for entity {:?}: {}",
                        entity,
                        preview.path.display()
                    );
                }
                Some(bevy::asset::LoadState::NotLoaded) => {
                    trace!(
                        "Preview image not yet loaded for entity {:?}: {}",
                        entity,
                        preview.path.display()
                    );
                }
                None => {
                    warn!(
                        "Unknown load state for preview image entity {:?}: {}",
                        entity,
                        preview.path.display()
                    );
                }
            }
        }
    }
}

/// Query helper for getting loaded preview images.
///
/// This can be used by rendering systems to only process
/// previews that have finished loading.
///
/// ## Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use dgx_pixels_tui::bevy_app::components::PreviewImage;
/// use dgx_pixels_tui::bevy_app::systems::preview::is_preview_loaded;
///
/// fn render_previews(
///     images: Res<Assets<Image>>,
///     preview_query: Query<&PreviewImage>,
/// ) {
///     for preview in preview_query.iter() {
///         if let Some(handle) = &preview.asset_handle {
///             if let Some(image) = images.get(handle) {
///                 // Render the loaded image
///             }
///         }
///     }
/// }
/// ```
pub fn is_preview_loaded(
    asset_server: &AssetServer,
    preview: &PreviewImage,
) -> bool {
    if let Some(handle) = &preview.asset_handle {
        matches!(
            asset_server.get_load_state(handle),
            Some(bevy::asset::LoadState::Loaded)
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_track_loading_status_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_systems(Update, track_loading_status);

        // Spawn entity with preview (no asset handle)
        let entity = app
            .world_mut()
            .spawn(PreviewImage {
                path: PathBuf::from("test.png"),
                asset_handle: None,
            })
            .id();

        // Should run without errors
        app.update();

        // Add asset handle
        let handle: Handle<Image> = app.world().resource::<AssetServer>().load("test.png");
        let mut preview = app.world_mut().get_mut::<PreviewImage>(entity).unwrap();
        preview.asset_handle = Some(handle);

        // Should run and track the loading state
        app.update();
    }

    #[test]
    fn test_is_preview_loaded_helper() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());

        let asset_server = app.world().resource::<AssetServer>();

        // Preview without handle
        let preview_no_handle = PreviewImage {
            path: PathBuf::from("test.png"),
            asset_handle: None,
        };
        assert!(!is_preview_loaded(asset_server, &preview_no_handle));

        // Preview with handle (will be in loading state)
        let handle: Handle<Image> = asset_server.load("test.png");
        let preview_with_handle = PreviewImage {
            path: PathBuf::from("test.png"),
            asset_handle: Some(handle),
        };

        // Should return false while loading (asset doesn't exist in test)
        assert!(!is_preview_loaded(asset_server, &preview_with_handle));
    }

    #[test]
    fn test_tracker_with_multiple_previews() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_systems(Update, track_loading_status);

        let asset_server = app.world().resource::<AssetServer>();
        let handle1: Handle<Image> = asset_server.load("image1.png");
        let handle2: Handle<Image> = asset_server.load("image2.png");

        // Spawn multiple preview entities
        app.world_mut().spawn(PreviewImage {
            path: PathBuf::from("image1.png"),
            asset_handle: Some(handle1),
        });

        app.world_mut().spawn(PreviewImage {
            path: PathBuf::from("image2.png"),
            asset_handle: Some(handle2),
        });

        app.world_mut().spawn(PreviewImage {
            path: PathBuf::from("image3.png"),
            asset_handle: None,
        });

        // Should track all previews without errors
        app.update();
    }
}
