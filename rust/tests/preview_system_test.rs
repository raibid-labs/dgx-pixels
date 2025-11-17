//! Test for preview system integration

use bevy::{
    app::{App, Update},
    prelude::*,
};
use dgx_pixels_tui::bevy_app::{
    resources::{AppState, PreviewManagerResource},
    systems::{poll_preview_results, request_preview_rendering},
};
use std::path::PathBuf;

#[test]
fn test_preview_manager_resource() {
    let mut app = App::new();
    app.insert_resource(PreviewManagerResource::default());
    app.insert_resource(AppState::default());

    // Verify resource is registered
    assert!(app.world().contains_resource::<PreviewManagerResource>());
}

#[test]
fn test_preview_systems_registered() {
    let mut app = App::new();
    app.insert_resource(PreviewManagerResource::default());
    app.insert_resource(AppState::default());

    // Add preview systems
    app.add_systems(Update, (request_preview_rendering, poll_preview_results).chain());

    // Run one update cycle
    app.update();

    // Systems should complete without panic
}

#[test]
fn test_preview_path_setting() {
    let mut app_state = AppState::default();
    assert!(app_state.current_preview.is_none());

    // Set a preview path
    let test_path = PathBuf::from("outputs/test_sprite.png");
    app_state.current_preview = Some(test_path.clone());

    assert!(app_state.current_preview.is_some());
    assert_eq!(app_state.current_preview.unwrap(), test_path);
}

#[test]
fn test_preview_manager_creation() {
    let manager = PreviewManagerResource::new();
    let stats = manager.manager.cache_stats();

    // Cache should start empty
    assert_eq!(stats.entries, 0);
    assert_eq!(stats.size_bytes, 0);
}
