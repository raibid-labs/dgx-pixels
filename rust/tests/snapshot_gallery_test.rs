//! Snapshot tests for Gallery screen
//!
//! These tests use the insta crate to verify UI layout doesn't break.
//! Run `cargo insta review` to review and accept snapshot changes.

use dgx_pixels_tui::app::{App, Screen};
use dgx_pixels_tui::ui::screens::gallery;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

mod helpers;
use helpers::*;

/// Test gallery screen with no images (empty state)
#[tokio::test]
async fn test_gallery_empty_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let app = App::new();

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_empty", format!("{:?}", buffer));
}

/// Test gallery screen with 3 images
#[tokio::test]
async fn test_gallery_with_images_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();
    let (_dir, paths) = create_test_gallery();

    // Add first 3 images to gallery
    for path in &paths[0..3] {
        app.add_to_gallery(path.clone());
    }

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_with_images", format!("{:?}", buffer));
}

/// Test gallery screen with selection on first image
#[tokio::test]
async fn test_gallery_first_selected_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();
    let (_dir, paths) = create_test_gallery();

    for path in &paths {
        app.add_to_gallery(path.clone());
    }

    // First image is selected by default (index 0)
    assert_eq!(app.selected_gallery_index, 0);

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_first_selected", format!("{:?}", buffer));
}

/// Test gallery screen with selection on middle image
#[tokio::test]
async fn test_gallery_middle_selected_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();
    let (_dir, paths) = create_test_gallery();

    for path in &paths {
        app.add_to_gallery(path.clone());
    }

    // Navigate to middle image
    app.gallery_next();
    app.gallery_next();
    assert_eq!(app.selected_gallery_index, 2);

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_middle_selected", format!("{:?}", buffer));
}

/// Test gallery screen with last image selected
#[tokio::test]
async fn test_gallery_last_selected_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();
    let (_dir, paths) = create_test_gallery();

    for path in &paths {
        app.add_to_gallery(path.clone());
    }

    // Navigate to last image
    for _ in 0..paths.len() - 1 {
        app.gallery_next();
    }
    assert_eq!(app.selected_gallery_index, paths.len() - 1);

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_last_selected", format!("{:?}", buffer));
}

/// Test gallery screen with single image
#[tokio::test]
async fn test_gallery_single_image_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    app.add_to_gallery(PathBuf::from("/test/single.png"));

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_single_image", format!("{:?}", buffer));
}

/// Test gallery screen with many images (more than viewport)
#[tokio::test]
async fn test_gallery_many_images_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Add 15 images
    for i in 0..15 {
        app.add_to_gallery(PathBuf::from(format!("/test/sprite_{:03}.png", i)));
    }

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_many_images", format!("{:?}", buffer));
}

/// Test gallery screen with many images, middle selected
#[tokio::test]
async fn test_gallery_many_images_middle_selected_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Add 15 images
    for i in 0..15 {
        app.add_to_gallery(PathBuf::from(format!("/test/sprite_{:03}.png", i)));
    }

    // Navigate to middle (index 7)
    for _ in 0..7 {
        app.gallery_next();
    }
    assert_eq!(app.selected_gallery_index, 7);

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_many_images_middle_selected", format!("{:?}", buffer));
}

/// Test gallery screen on small terminal (80x24)
#[tokio::test]
async fn test_gallery_small_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_small_terminal();
    let mut app = App::new();
    let (_dir, paths) = create_test_gallery();

    for path in &paths[0..3] {
        app.add_to_gallery(path.clone());
    }

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_small_terminal", format!("{:?}", buffer));
}

/// Test gallery screen on large terminal (200x60)
#[tokio::test]
async fn test_gallery_large_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_large_terminal();
    let mut app = App::new();
    let (_dir, paths) = create_test_gallery();

    for path in &paths {
        app.add_to_gallery(path.clone());
    }

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_large_terminal", format!("{:?}", buffer));
}

/// Test gallery screen with text-only terminal capability
#[tokio::test]
async fn test_gallery_text_only_capability_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Force text-only mode
    app.terminal_capability = dgx_pixels_tui::sixel::TerminalCapability::TextOnly;

    let (_dir, paths) = create_test_gallery();
    for path in &paths[0..2] {
        app.add_to_gallery(path.clone());
    }

    terminal
        .draw(|f| gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("gallery_text_only_capability", format!("{:?}", buffer));
}
