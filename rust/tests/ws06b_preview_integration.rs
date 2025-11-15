//! Integration tests for WS-06B: Preview Component & Systems
//!
//! These tests verify that the preview loading and tracking systems
//! work correctly in isolation from the rest of the application.

#![cfg(feature = "bevy_migration_foundation")]

use bevy::prelude::*;
use dgx_pixels_tui::bevy_app::components::{Job, JobStatus, PreviewImage};
use dgx_pixels_tui::bevy_app::systems::preview::{load_preview_images, track_loading_status};
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn test_preview_system_integration() {
    // Setup test app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_systems(Update, load_preview_images);
    app.add_systems(Update, track_loading_status);

    // Spawn a job entity in pending state
    let job_entity = app
        .world_mut()
        .spawn(Job {
            id: "integration-test-001".to_string(),
            prompt: "pixel art character".to_string(),
            status: JobStatus::Pending,
            submitted_at: Instant::now(),
        })
        .id();

    // Run one update - should not load anything yet
    app.update();

    // Verify no preview attached
    {
        let world = app.world();
        assert!(
            world.get::<PreviewImage>(job_entity).is_none(),
            "Preview should not exist for pending job"
        );
    }

    // Update job to complete status
    {
        let mut job = app.world_mut().get_mut::<Job>(job_entity).unwrap();
        job.status = JobStatus::Complete {
            image_path: PathBuf::from("output/test_image.png"),
            duration_s: 3.5,
        };
    }

    // Run update - should load preview
    app.update();

    // Verify preview was attached with asset handle
    {
        let world = app.world();
        let preview = world
            .get::<PreviewImage>(job_entity)
            .expect("Preview should be attached after job completes");

        assert_eq!(
            preview.path,
            PathBuf::from("output/test_image.png"),
            "Preview path should match job output path"
        );
        assert!(
            preview.asset_handle.is_some(),
            "Asset handle should be populated"
        );
        assert!(preview.is_loaded(), "Preview should report as loaded");
    }
}

#[test]
fn test_multiple_jobs_with_previews() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_systems(Update, load_preview_images);
    app.add_systems(Update, track_loading_status);

    // Spawn multiple completed jobs
    let job1 = app
        .world_mut()
        .spawn(Job {
            id: "job-001".to_string(),
            prompt: "prompt 1".to_string(),
            status: JobStatus::Complete {
                image_path: PathBuf::from("output/image1.png"),
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
                image_path: PathBuf::from("output/image2.png"),
                duration_s: 4.0,
            },
            submitted_at: Instant::now(),
        })
        .id();

    let job3 = app
        .world_mut()
        .spawn(Job {
            id: "job-003".to_string(),
            prompt: "prompt 3".to_string(),
            status: JobStatus::Generating {
                stage: "sampling".to_string(),
                progress: 0.5,
                eta_s: 2.0,
            },
            submitted_at: Instant::now(),
        })
        .id();

    app.update();

    // Both completed jobs should have previews
    {
        let world = app.world();

        let preview1 = world
            .get::<PreviewImage>(job1)
            .expect("Job 1 should have preview");
        let preview2 = world
            .get::<PreviewImage>(job2)
            .expect("Job 2 should have preview");

        assert_eq!(preview1.path, PathBuf::from("output/image1.png"));
        assert_eq!(preview2.path, PathBuf::from("output/image2.png"));
        assert!(preview1.is_loaded());
        assert!(preview2.is_loaded());

        // Generating job should not have preview
        assert!(
            world.get::<PreviewImage>(job3).is_none(),
            "Generating job should not have preview"
        );
    }
}

#[test]
fn test_preview_component_helper_methods() {
    // Test PreviewImage::new()
    let preview = PreviewImage::new(PathBuf::from("test.png"));
    assert_eq!(preview.path, PathBuf::from("test.png"));
    assert!(!preview.is_loaded());
    assert!(preview.handle().is_none());

    // Test with handle
    let handle: Handle<Image> = Handle::default();
    let preview_with_handle = PreviewImage {
        path: PathBuf::from("test.png"),
        asset_handle: Some(handle.clone()),
    };

    assert!(preview_with_handle.is_loaded());
    assert_eq!(preview_with_handle.handle().unwrap(), &handle);
}
