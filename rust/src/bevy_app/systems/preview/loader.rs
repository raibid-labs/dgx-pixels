//! # Preview Image Loader System
//!
//! System that loads preview images when jobs complete.
//!
//! ## How It Works
//!
//! 1. Queries for jobs that have changed status
//! 2. For completed jobs, loads the image using Bevy's AssetServer
//! 3. Attaches PreviewImage component with asset handle to the job entity
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::systems::preview::load_preview_images;
//!
//! App::new()
//!     .add_systems(Update, load_preview_images);
//! ```

use bevy::prelude::*;

use crate::bevy_app::components::{Job, JobStatus, PreviewImage};

/// System that loads preview images when jobs complete.
///
/// This system:
/// - Monitors jobs for status changes using `Changed<Job>`
/// - When a job completes, loads the image via `AssetServer`
/// - Attaches a `PreviewImage` component with the asset handle
///
/// ## Change Detection
///
/// Uses Bevy's change detection to only process jobs that have updated.
/// This avoids re-loading images that are already loaded.
///
/// ## Asset Loading
///
/// Images are loaded asynchronously by Bevy's asset system.
/// The handle is immediately available, but the actual image data
/// loads in the background.
pub fn load_preview_images(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    job_query: Query<(Entity, &Job), Changed<Job>>,
) {
    for (entity, job) in job_query.iter() {
        // Only load preview when job completes
        if let JobStatus::Complete { image_path, .. } = &job.status {
            // Load the image asset
            let handle: Handle<Image> = asset_server.load(image_path.clone());

            // Attach PreviewImage component with populated asset handle
            commands.entity(entity).insert(PreviewImage {
                path: image_path.clone(),
                asset_handle: Some(handle),
            });

            info!(
                "Loaded preview image for job {}: {}",
                job.id,
                image_path.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Instant;

    #[test]
    fn test_load_preview_images_system() {
        // Setup test app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_systems(Update, load_preview_images);

        // Spawn a job entity
        let job_entity = app
            .world_mut()
            .spawn(Job {
                id: "test-job-001".to_string(),
                prompt: "test prompt".to_string(),
                status: JobStatus::Pending,
                submitted_at: Instant::now(),
            })
            .id();

        // Run one update - should not load anything yet
        app.update();

        // Verify no preview attached
        let world = app.world();
        assert!(world.get::<PreviewImage>(job_entity).is_none());

        // Update job to complete status
        let mut job = app.world_mut().get_mut::<Job>(job_entity).unwrap();
        job.status = JobStatus::Complete {
            image_path: PathBuf::from("test_image.png"),
            duration_s: 3.5,
        };

        // Run update - should load preview
        app.update();

        // Verify preview was attached
        let world = app.world();
        let preview = world.get::<PreviewImage>(job_entity);
        assert!(preview.is_some());

        let preview = preview.unwrap();
        assert_eq!(preview.path, PathBuf::from("test_image.png"));
        assert!(preview.asset_handle.is_some());
        assert!(preview.is_loaded());
    }

    #[test]
    fn test_load_preview_does_not_trigger_on_other_status() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_systems(Update, load_preview_images);

        // Test with Generating status
        let job_entity = app
            .world_mut()
            .spawn(Job {
                id: "test-job-002".to_string(),
                prompt: "test prompt".to_string(),
                status: JobStatus::Generating {
                    stage: "sampling".to_string(),
                    progress: 0.5,
                    eta_s: 2.0,
                },
                submitted_at: Instant::now(),
            })
            .id();

        app.update();

        // Should not have preview
        let world = app.world();
        assert!(world.get::<PreviewImage>(job_entity).is_none());

        // Test with Failed status
        let mut job = app.world_mut().get_mut::<Job>(job_entity).unwrap();
        job.status = JobStatus::Failed {
            error: "test error".to_string(),
        };

        app.update();

        // Should still not have preview
        let world = app.world();
        assert!(world.get::<PreviewImage>(job_entity).is_none());
    }

    #[test]
    fn test_multiple_jobs_load_independently() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_systems(Update, load_preview_images);

        // Spawn multiple jobs
        let job1 = app
            .world_mut()
            .spawn(Job {
                id: "job-001".to_string(),
                prompt: "prompt 1".to_string(),
                status: JobStatus::Complete {
                    image_path: PathBuf::from("image1.png"),
                    duration_s: 3.0,
                },
                submitted_at: Instant::now(),
            })
            .id();

        let job2 = app
            .world_mut()
            .spawn(Job {
                id: "job-002".to_string(),
                prompt: "prompt 2".to_string(),
                status: JobStatus::Complete {
                    image_path: PathBuf::from("image2.png"),
                    duration_s: 4.0,
                },
                submitted_at: Instant::now(),
            })
            .id();

        app.update();

        // Both should have previews
        let world = app.world();
        let preview1 = world.get::<PreviewImage>(job1).unwrap();
        let preview2 = world.get::<PreviewImage>(job2).unwrap();

        assert_eq!(preview1.path, PathBuf::from("image1.png"));
        assert_eq!(preview2.path, PathBuf::from("image2.png"));
        assert!(preview1.is_loaded());
        assert!(preview2.is_loaded());
    }
}
