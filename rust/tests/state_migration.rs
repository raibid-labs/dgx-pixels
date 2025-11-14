#![cfg(feature = "bevy_migration_foundation")]

use bevy::prelude::*;
use dgx_pixels_tui::bevy_app::{
    components::{Job, JobStatus},
    resources::*,
    DgxPixelsPlugin,
};
use std::path::PathBuf;

#[test]
fn test_state_initialization() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);

    // Run startup systems
    app.update();

    // Verify all resources initialized
    assert!(app.world().contains_resource::<AppState>());
    assert!(app.world().contains_resource::<CurrentScreen>());
    assert!(app.world().contains_resource::<InputBuffer>());
    assert!(app.world().contains_resource::<GalleryState>());
    assert!(app.world().contains_resource::<JobTracker>());

    // Verify default values
    let screen = app.world().resource::<CurrentScreen>();
    assert_eq!(screen.0, Screen::Generation);

    let input = app.world().resource::<InputBuffer>();
    assert_eq!(input.text, "");
    assert_eq!(input.cursor, 0);

    let gallery = app.world().resource::<GalleryState>();
    assert_eq!(gallery.len(), 0);

    let tracker = app.world().resource::<JobTracker>();
    assert_eq!(tracker.total_submitted, 0);
}

#[test]
fn test_resource_mutation() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);
    app.update();

    // Mutate resources
    {
        let mut screen = app.world_mut().resource_mut::<CurrentScreen>();
        screen.0 = Screen::Gallery;
    }

    {
        let mut input = app.world_mut().resource_mut::<InputBuffer>();
        input.insert('h');
        input.insert('i');
    }

    {
        let mut gallery = app.world_mut().resource_mut::<GalleryState>();
        gallery.add_image(PathBuf::from("/test.png"));
    }

    // Verify mutations
    let screen = app.world().resource::<CurrentScreen>();
    assert_eq!(screen.0, Screen::Gallery);

    let input = app.world().resource::<InputBuffer>();
    assert_eq!(input.text, "hi");

    let gallery = app.world().resource::<GalleryState>();
    assert_eq!(gallery.len(), 1);
}

#[test]
fn test_job_entity_spawning() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);
    app.update();

    // Spawn job entities
    app.world_mut().spawn(Job::new(
        "job-001".to_string(),
        "test prompt 1".to_string(),
    ));
    app.world_mut().spawn(Job::new(
        "job-002".to_string(),
        "test prompt 2".to_string(),
    ));

    app.update();

    // Query for jobs
    let mut query = app.world_mut().query::<&Job>();
    let jobs: Vec<&Job> = query.iter(app.world()).collect();

    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].id, "job-001");
    assert_eq!(jobs[1].id, "job-002");
}

#[test]
fn test_job_status_updates() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);
    app.update();

    // Spawn a job
    let entity = app
        .world_mut()
        .spawn(Job::new("job-001".to_string(), "test".to_string()))
        .id();

    app.update();

    // Update job status
    {
        let mut job = app.world_mut().get_mut::<Job>(entity).unwrap();
        job.status = JobStatus::Generating {
            stage: "sampling".to_string(),
            progress: 0.5,
            eta_s: 2.0,
        };
    }

    // Verify update
    let job = app.world().get::<Job>(entity).unwrap();
    assert!(matches!(job.status, JobStatus::Generating { .. }));
}

#[test]
fn test_job_tracker_statistics() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);
    app.update();

    // Submit and complete jobs
    {
        let mut tracker = app.world_mut().resource_mut::<JobTracker>();
        tracker.submit_job();
        tracker.submit_job();
        tracker.submit_job();
        tracker.complete_job();
        tracker.complete_job();
        tracker.fail_job();
    }

    // Verify statistics
    let tracker = app.world().resource::<JobTracker>();
    assert_eq!(tracker.total_submitted, 3);
    assert_eq!(tracker.total_completed, 2);
    assert_eq!(tracker.total_failed, 1);
    assert_eq!(tracker.active_jobs(), 0); // All jobs accounted for
}

#[test]
fn test_gallery_navigation() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);
    app.update();

    // Add images and navigate
    {
        let mut gallery = app.world_mut().resource_mut::<GalleryState>();
        gallery.add_image(PathBuf::from("/img1.png"));
        gallery.add_image(PathBuf::from("/img2.png"));
        gallery.add_image(PathBuf::from("/img3.png"));

        gallery.select_next();
        assert_eq!(gallery.selected, 1);

        gallery.select_next();
        assert_eq!(gallery.selected, 2);

        gallery.select_next(); // Wrap around
        assert_eq!(gallery.selected, 0);

        gallery.select_previous(); // Wrap backward
        assert_eq!(gallery.selected, 2);
    }
}

#[test]
fn test_screen_navigation() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);
    app.update();

    // Navigate screens
    {
        let mut screen = app.world_mut().resource_mut::<CurrentScreen>();
        let mut current = screen.0;

        current = current.next();
        assert_eq!(current, Screen::Comparison);

        current = current.next();
        assert_eq!(current, Screen::Queue);

        current = current.previous();
        assert_eq!(current, Screen::Comparison);
    }
}

#[test]
fn test_app_state_quit_flag() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);
    app.update();

    // Test quit flag
    {
        let mut state = app.world_mut().resource_mut::<AppState>();
        assert!(!state.should_quit);
        state.quit();
        assert!(state.should_quit);
    }
}
