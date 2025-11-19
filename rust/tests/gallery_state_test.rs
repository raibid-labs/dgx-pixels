//! Unit tests for gallery state management

use dgx_pixels_tui::app::{App, Screen};
use rstest::*;
use std::path::PathBuf;

mod helpers;
use helpers::*;

#[test]
fn test_gallery_empty_state() {
    let app = App::new();

    assert!(app.gallery_images.is_empty());
    assert_eq!(app.selected_gallery_index, 0);
    assert!(app.selected_gallery_image().is_none());
}

#[test]
fn test_add_single_image_to_gallery() {
    let mut app = App::new();
    let path = PathBuf::from("/test/sprite_001.png");

    app.add_to_gallery(path.clone());

    assert_eq!(app.gallery_images.len(), 1);
    assert_eq!(app.selected_gallery_index, 0);
    assert_eq!(app.selected_gallery_image(), Some(&path));
}

#[test]
fn test_add_multiple_images_to_gallery() {
    let mut app = App::new();

    let paths: Vec<PathBuf> = (1..=5)
        .map(|i| PathBuf::from(format!("/test/sprite_{:03}.png", i)))
        .collect();

    for path in &paths {
        app.add_to_gallery(path.clone());
    }

    assert_eq!(app.gallery_images.len(), 5);
    assert_eq!(app.gallery_images, paths);
}

#[test]
fn test_add_duplicate_image_to_gallery() {
    let mut app = App::new();
    let path = PathBuf::from("/test/sprite_001.png");

    app.add_to_gallery(path.clone());
    app.add_to_gallery(path.clone());

    // Duplicates should NOT be added (add_to_gallery checks contains)
    assert_eq!(app.gallery_images.len(), 1);
}

#[rstest]
#[case(0, 0, 0)] // Empty gallery - can't navigate
#[case(1, 0, 0)] // Single image - stays at 0
#[case(3, 0, 1)] // Navigate forward from first
#[case(3, 1, 2)] // Navigate forward from middle
#[case(3, 2, 0)] // Navigate forward from last (wraps to beginning)
fn test_gallery_next_navigation(
    #[case] num_images: usize,
    #[case] start_index: usize,
    #[case] expected_index: usize,
) {
    let mut app = App::new();

    // Add images
    for i in 0..num_images {
        app.add_to_gallery(PathBuf::from(format!("/test/img{}.png", i)));
    }

    app.selected_gallery_index = start_index;
    app.gallery_next();

    assert_eq!(app.selected_gallery_index, expected_index);
}

#[rstest]
#[case(0, 0, 0)] // Empty gallery - can't navigate
#[case(1, 0, 0)] // Single image - stays at 0
#[case(3, 2, 1)] // Navigate backward from last
#[case(3, 1, 0)] // Navigate backward from middle
#[case(3, 0, 2)] // Navigate backward from first (wraps to end)
fn test_gallery_previous_navigation(
    #[case] num_images: usize,
    #[case] start_index: usize,
    #[case] expected_index: usize,
) {
    let mut app = App::new();

    // Add images
    for i in 0..num_images {
        app.add_to_gallery(PathBuf::from(format!("/test/img{}.png", i)));
    }

    app.selected_gallery_index = start_index;
    app.gallery_prev();

    assert_eq!(app.selected_gallery_index, expected_index);
}

#[test]
fn test_selected_gallery_image() {
    let mut app = App::new();

    let path1 = PathBuf::from("/test/img1.png");
    let path2 = PathBuf::from("/test/img2.png");
    let path3 = PathBuf::from("/test/img3.png");

    app.add_to_gallery(path1.clone());
    app.add_to_gallery(path2.clone());
    app.add_to_gallery(path3.clone());

    // Initially at index 0
    assert_eq!(app.selected_gallery_image(), Some(&path1));

    // Navigate to index 1
    app.gallery_next();
    assert_eq!(app.selected_gallery_image(), Some(&path2));

    // Navigate to index 2
    app.gallery_next();
    assert_eq!(app.selected_gallery_image(), Some(&path3));
}

#[test]
fn test_load_gallery_from_outputs() {
    let (_temp_dir, test_paths) = create_test_gallery();
    let mut app = App::new();

    // Load from the temp directory
    let dir_str = _temp_dir.path().to_str().unwrap();
    app.load_gallery_from_outputs(dir_str);

    // Should have loaded all test images
    assert_eq!(app.gallery_images.len(), test_paths.len());

    // All paths should exist in gallery
    for path in &test_paths {
        assert!(
            app.gallery_images.contains(path),
            "Gallery should contain {:?}",
            path
        );
    }
}

#[test]
fn test_load_gallery_from_nonexistent_directory() {
    let mut app = App::new();

    // Try to load from nonexistent directory - should not crash
    app.load_gallery_from_outputs("/nonexistent/directory/xyz");

    // Gallery should remain empty
    assert!(app.gallery_images.is_empty());
}

#[test]
fn test_navigate_to_gallery_screen() {
    let mut app = App::new();

    // Start at Generation screen
    assert_eq!(app.current_screen, Screen::Generation);

    // Navigate to Gallery
    app.navigate_to(Screen::Gallery);

    assert_eq!(app.current_screen, Screen::Gallery);
    assert_eq!(app.screen_history.len(), 1);
    assert_eq!(app.screen_history[0], Screen::Generation);
}

#[test]
fn test_navigate_back_from_gallery() {
    let mut app = App::new();

    // Navigate Generation -> Gallery
    app.navigate_to(Screen::Gallery);
    assert_eq!(app.current_screen, Screen::Gallery);

    // Navigate back
    app.navigate_back();

    assert_eq!(app.current_screen, Screen::Generation);
    assert!(app.screen_history.is_empty());
}

#[test]
fn test_gallery_bounds_checking() {
    let mut app = App::new();

    // Add 3 images
    for i in 0..3 {
        app.add_to_gallery(PathBuf::from(format!("/test/img{}.png", i)));
    }

    // Set index beyond bounds
    app.selected_gallery_index = 10;

    // selected_gallery_image should return None (out of bounds)
    assert!(app.selected_gallery_image().is_none());
}

#[test]
fn test_gallery_selection_persistence() {
    let mut app = App::new();

    // Add images
    for i in 0..5 {
        app.add_to_gallery(PathBuf::from(format!("/test/img{}.png", i)));
    }

    // Navigate to index 3
    app.selected_gallery_index = 3;

    // Navigate away from gallery
    app.navigate_to(Screen::Settings);

    // Navigate back to gallery
    app.navigate_to(Screen::Gallery);

    // Selection should be preserved
    assert_eq!(app.selected_gallery_index, 3);
}

#[test]
fn test_current_preview_not_set_by_gallery_add() {
    let mut app = App::new();

    let path = PathBuf::from("/test/new_sprite.png");
    app.add_to_gallery(path.clone());

    // add_to_gallery doesn't set current_preview
    // (it's set when job completes in the event loop)
    assert_eq!(app.current_preview, None);
}

#[test]
fn test_needs_redraw_on_gallery_changes() {
    let mut app = App::new();
    app.needs_redraw = false;

    // Adding image should trigger redraw
    app.add_to_gallery(PathBuf::from("/test/img.png"));
    assert!(app.needs_redraw);

    app.needs_redraw = false;

    // Navigating should trigger redraw
    app.gallery_next();
    assert!(app.needs_redraw);
}
