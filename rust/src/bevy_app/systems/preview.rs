//! # Preview Loading System
//!
//! System that handles async preview loading and updates app state
//! when previews are ready to display.

use bevy::prelude::*;
use std::path::PathBuf;

use crate::bevy_app::resources::{AppState, PreviewManagerResource};
use crate::sixel::RenderOptions;

/// System to request preview rendering when a new preview path is set.
///
/// This system monitors changes to `app_state.current_preview` and requests
/// preview rendering when a new image is set.
pub fn request_preview_rendering(
    mut app_state: ResMut<AppState>,
    preview_manager: Res<PreviewManagerResource>,
) {
    if let Some(preview_path) = &app_state.current_preview {
        // Check if preview is already cached
        if !preview_manager.manager.has_preview(preview_path) {
            // Request async preview rendering
            let options = RenderOptions {
                width: 40,
                height: 20,
                preserve_aspect: true,
                high_quality: true,
            };

            if let Err(e) = preview_manager
                .manager
                .request_preview(preview_path.clone(), options)
            {
                warn!("Failed to request preview for {:?}: {}", preview_path, e);
                // Clear the preview path on error
                app_state.current_preview = None;
            } else {
                debug!("Requested preview rendering for {:?}", preview_path);
            }
        }
    }
}

/// System to poll for completed preview results.
///
/// This system checks for completed async preview rendering results
/// and triggers a redraw when new previews are available.
pub fn poll_preview_results(
    mut app_state: ResMut<AppState>,
    preview_manager: Res<PreviewManagerResource>,
) {
    // Try to receive any completed preview results
    while let Some(result) = preview_manager.manager.try_recv_result() {
        if let Some(entry) = result.entry {
            debug!(
                "Preview ready: {:?} ({}x{}, {} bytes)",
                entry.path, entry.dimensions.0, entry.dimensions.1, entry.size_bytes
            );
            // Request redraw to show the new preview
            app_state.needs_redraw = true;
        } else if let Some(error) = result.error {
            warn!("Preview rendering failed: {}", error);
        }
    }
}

/// Helper system to set a test preview (for development/testing).
///
/// This can be called manually or from a keybinding to load a test image.
#[allow(dead_code)]
pub fn load_test_preview(mut app_state: ResMut<AppState>) {
    let test_path = PathBuf::from("outputs/test_sprite.png");

    if test_path.exists() {
        info!("Loading test preview: {:?}", test_path);
        app_state.current_preview = Some(test_path);
        app_state.needs_redraw = true;
    } else {
        warn!("Test preview not found: {:?}", test_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_systems_compile() {
        // Basic compilation test
        let mut app = bevy::app::App::new();
        app.add_systems(Update, (request_preview_rendering, poll_preview_results));
    }
}
