//! # Preview Manager System (T10)
//!
//! Automatic preview image loading and management for the gallery screen.
//!
//! ## Architecture
//!
//! This system periodically scans the gallery directory for new images,
//! loads them via Bevy's AssetServer, and manages the preview cache.
//!
//! ## Components
//!
//! - `scan_gallery_directory`: Periodic scan (every 2s) for new images
//! - `check_preview_loading`: Monitor asset loading status
//! - Integration with `ImageCache` for LRU eviction
//!
//! ## Flow
//!
//! 1. Scan `../outputs` directory for PNG/JPG files
//! 2. Compare with existing `GalleryState.images`
//! 3. For new images: spawn entities with `PreviewImage` components
//! 4. `AssetServer` loads images asynchronously in background
//! 5. `check_preview_loading` monitors load state
//! 6. `ImageCache` evicts old entries after 5 minutes

use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

use crate::bevy_app::components::PreviewImage;
use crate::bevy_app::resources::GalleryState;
use crate::bevy_app::systems::assets::ImageCache;

/// Default gallery directory (relative to workspace root).
pub const DEFAULT_GALLERY_DIR: &str = "../outputs";

/// Scan interval for directory watching (2 seconds).
pub const SCAN_INTERVAL_SECS: u64 = 2;

/// Resource to track last directory scan time.
#[derive(Resource, Debug)]
pub struct GalleryScanState {
    pub last_scan: SystemTime,
    pub gallery_dir: PathBuf,
}

impl Default for GalleryScanState {
    fn default() -> Self {
        Self {
            last_scan: SystemTime::UNIX_EPOCH,
            gallery_dir: PathBuf::from(DEFAULT_GALLERY_DIR),
        }
    }
}

impl GalleryScanState {
    /// Create with custom gallery directory.
    pub fn with_directory(mut self, dir: PathBuf) -> Self {
        self.gallery_dir = dir;
        self
    }

    /// Check if it's time to scan again.
    pub fn should_scan(&self) -> bool {
        if let Ok(elapsed) = SystemTime::now().duration_since(self.last_scan) {
            elapsed.ge(&Duration::from_secs(SCAN_INTERVAL_SECS))
        } else {
            true // Clock went backwards, scan anyway
        }
    }

    /// Mark scan as completed.
    pub fn mark_scanned(&mut self) {
        self.last_scan = SystemTime::now();
    }
}

/// System to periodically scan gallery directory for new images.
///
/// Runs every 2 seconds to detect new images added by completed jobs.
/// Updates GalleryState with newly discovered images.
pub fn scan_gallery_directory(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut gallery: ResMut<GalleryState>,
    mut scan_state: ResMut<GalleryScanState>,
    existing_query: Query<&PreviewImage>,
    mut cache: ResMut<ImageCache>,
) {
    // Check if it's time to scan
    if !scan_state.should_scan() {
        return;
    }

    // Convert relative path to absolute for scanning
    let abs_gallery_dir = match fs::canonicalize(&scan_state.gallery_dir) {
        Ok(abs_path) => abs_path,
        Err(e) => {
            debug!(
                "Gallery directory does not exist or cannot be canonicalized: {:?} (error: {})",
                scan_state.gallery_dir, e
            );
            scan_state.mark_scanned();
            return;
        }
    };

    debug!("Scanning gallery directory: {:?}", abs_gallery_dir);

    // Get project root for converting paths to relative
    let project_root = std::env::current_dir()
        .ok()
        .and_then(|cwd| cwd.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from(".."));

    // Scan directory
    match scan_image_directory(&abs_gallery_dir) {
        Ok(discovered_images) => {
            let mut new_images = 0;

            for abs_image_path in discovered_images {
                // Convert absolute path to relative path for AssetServer
                // AssetPlugin is configured with file_path="../" so we need "outputs/job-xxx.png"
                let image_path = abs_image_path
                    .strip_prefix(&project_root)
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|_| {
                        // Fallback: extract filename and prepend "outputs/"
                        if let Some(filename) = abs_image_path.file_name() {
                            std::path::PathBuf::from("outputs").join(filename)
                        } else {
                            abs_image_path.clone()
                        }
                    });

                // Skip if already in gallery
                if gallery.images.contains(&image_path) {
                    continue;
                }

                // Skip if already loaded as preview
                let already_loaded = existing_query
                    .iter()
                    .any(|preview| preview.path == image_path);

                if already_loaded {
                    continue;
                }

                debug!("New image discovered: {:?} (asset path: {:?})", abs_image_path, image_path);

                // Add to gallery state (using relative path for AssetServer)
                gallery.add_image(image_path.clone());

                // Load image via AssetServer (relative to project root)
                let handle: Handle<Image> = asset_server.load(image_path.clone());

                // Add to cache
                cache.insert(image_path.clone(), handle.clone());

                // Spawn entity with PreviewImage component
                commands.spawn(PreviewImage {
                    path: image_path.clone(),
                    asset_handle: Some(handle),
                });

                new_images += 1;
            }

            if new_images > 0 {
                info!("Loaded {} new images from gallery", new_images);
            }
        }
        Err(e) => {
            warn!("Failed to scan gallery directory: {}", e);
        }
    }

    scan_state.mark_scanned();
}

/// System to check preview image loading status.
///
/// Monitors async asset loading and logs status changes.
/// Runs every frame in Update schedule.
pub fn check_preview_loading(
    asset_server: Res<AssetServer>,
    images: Option<Res<Assets<Image>>>,
    preview_query: Query<(Entity, &PreviewImage), Changed<PreviewImage>>,
) {
    for (entity, preview) in preview_query.iter() {
        if let Some(handle) = &preview.asset_handle {
            match asset_server.load_state(handle) {
                bevy::asset::LoadState::Failed(err) => {
                    warn!(
                        "Failed to load preview image {:?}: {:?}",
                        preview.path, err
                    );
                }
                bevy::asset::LoadState::Loaded => {
                    // Verify image is in Assets storage (if available)
                    if let Some(images) = images.as_ref() {
                        if images.get(handle).is_some() {
                            debug!("Preview image loaded: {:?}", preview.path);
                        } else {
                            warn!("Preview handle loaded but image not in storage: {:?}", preview.path);
                        }
                    }
                }
                bevy::asset::LoadState::Loading => {
                    debug!("Loading preview image: {:?}", preview.path);
                }
                bevy::asset::LoadState::NotLoaded => {
                    debug!("Preview image not yet started loading: {:?}", preview.path);
                }
            }
        } else {
            warn!(
                "PreviewImage entity {:?} has no asset handle for path: {:?}",
                entity, preview.path
            );
        }
    }
}

/// Scan a directory for image files (PNG, JPG, JPEG, WebP).
///
/// Returns a sorted list of image paths (newest first).
fn scan_image_directory(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut images = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Check if file has image extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if matches!(ext_str.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                images.push(path);
            }
        }
    }

    // Sort by filename (descending - newest first)
    // Filenames contain timestamps, so this sorts by creation time
    images.sort_by(|a, b| b.cmp(a));

    Ok(images)
}

/// Helper function to preload all images in a directory.
///
/// Useful for initialization or bulk loading scenarios.
pub fn preload_gallery_directory(
    commands: &mut Commands,
    asset_server: &AssetServer,
    gallery: &mut GalleryState,
    cache: &mut ImageCache,
    dir: &Path,
) -> Result<usize, std::io::Error> {
    // Convert to absolute path for scanning
    let abs_dir = fs::canonicalize(dir)?;
    let images = scan_image_directory(&abs_dir)?;
    let count = images.len();

    // Get project root for converting paths to relative
    let project_root = std::env::current_dir()
        .ok()
        .and_then(|cwd| cwd.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from(".."));

    for abs_image_path in images {
        // Convert absolute path to relative path for AssetServer
        let image_path = abs_image_path
            .strip_prefix(&project_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| {
                // Fallback: extract filename and prepend "outputs/"
                if let Some(filename) = abs_image_path.file_name() {
                    std::path::PathBuf::from("outputs").join(filename)
                } else {
                    abs_image_path.clone()
                }
            });

        // Add to gallery (using relative path)
        gallery.add_image(image_path.clone());

        // Load image (relative to project root)
        let handle: Handle<Image> = asset_server.load(image_path.clone());

        // Add to cache
        cache.insert(image_path.clone(), handle.clone());

        // Spawn preview entity
        commands.spawn(PreviewImage {
            path: image_path,
            asset_handle: Some(handle),
        });
    }

    info!("Preloaded {} images from directory: {:?}", count, abs_dir);
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_state_default() {
        let state = GalleryScanState::default();
        assert!(state.should_scan()); // First scan should always run
        assert_eq!(state.gallery_dir, PathBuf::from(DEFAULT_GALLERY_DIR));
    }

    #[test]
    fn test_scan_state_custom_directory() {
        let state = GalleryScanState::default().with_directory(PathBuf::from("/custom/path"));
        assert_eq!(state.gallery_dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_scan_state_interval() {
        let mut state = GalleryScanState::default();
        state.mark_scanned();

        // Should not scan immediately
        assert!(!state.should_scan());

        // Wait and check again (this test might be flaky in CI)
        std::thread::sleep(Duration::from_millis(100));
        // Still shouldn't scan (interval is 2 seconds)
        assert!(!state.should_scan());
    }

    #[test]
    fn test_scan_image_directory_empty() {
        // Test with non-existent directory
        let result = scan_image_directory(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_image_directory_filters_extensions() {
        // This test would need a temp directory with test files
        // Skipping for now - would be tested in integration tests
    }
}
