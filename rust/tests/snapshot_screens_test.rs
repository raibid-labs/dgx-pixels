//! Snapshot tests for all UI screens
//!
//! These tests use the insta crate to verify UI layout doesn't break.
//! Run `cargo insta review` to review and accept snapshot changes.

use dgx_pixels_tui::app::{App, JobStatus, Screen};
use dgx_pixels_tui::ui::screens::{generation, help, queue, settings};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

mod helpers;
use helpers::*;

// ========================================
// Generation Screen Tests
// ========================================

/// Test generation screen initial state
#[tokio::test]
async fn test_generation_initial_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let app = App::new();

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_initial", format!("{:?}", buffer));
}

/// Test generation screen with prompt input
#[tokio::test]
async fn test_generation_with_prompt_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Add prompt text
    for c in "pixel art wizard character, 32x32".chars() {
        app.input_char(c);
    }

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_with_prompt", format!("{:?}", buffer));
}

/// Test generation screen with active job (queued)
#[tokio::test]
async fn test_generation_with_queued_job_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    app.add_job(
        "job-001".to_string(),
        "pixel art castle sprite".to_string(),
    );

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_queued_job", format!("{:?}", buffer));
}

/// Test generation screen with running job
#[tokio::test]
async fn test_generation_with_running_job_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    app.add_job("job-002".to_string(), "forest background tile".to_string());
    app.update_job_status(
        "job-002",
        JobStatus::Running {
            stage: "sampling".to_string(),
            progress: 0.45,
            eta_s: 3.2,
        },
    );

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_running_job", format!("{:?}", buffer));
}

/// Test generation screen with completed job
#[tokio::test]
async fn test_generation_with_completed_job_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    app.add_job("job-003".to_string(), "pixel art sword weapon".to_string());
    app.update_job_status(
        "job-003",
        JobStatus::Complete {
            image_path: PathBuf::from("/output/sprite_001.png"),
            duration_s: 4.2,
        },
    );

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_completed_job", format!("{:?}", buffer));
}

/// Test generation screen with failed job
#[tokio::test]
async fn test_generation_with_failed_job_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    app.add_job("job-004".to_string(), "invalid prompt".to_string());
    app.update_job_status(
        "job-004",
        JobStatus::Failed {
            error: "Model not found: sdxl_custom.safetensors".to_string(),
        },
    );

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_failed_job", format!("{:?}", buffer));
}

/// Test generation screen with recent generations
#[tokio::test]
async fn test_generation_with_recent_generations_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Add some gallery images
    app.add_to_gallery(PathBuf::from("/output/sprite_001.png"));
    app.add_to_gallery(PathBuf::from("/output/sprite_002.png"));
    app.add_to_gallery(PathBuf::from("/output/sprite_003.png"));

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_with_recent", format!("{:?}", buffer));
}

/// Test generation screen with preview
#[tokio::test]
async fn test_generation_with_preview_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Set current preview
    app.current_preview = Some(PathBuf::from("/output/preview.png"));

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_with_preview", format!("{:?}", buffer));
}

/// Test generation screen in debug mode (logs tab)
#[tokio::test]
async fn test_generation_debug_mode_logs_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Enable debug mode
    app.debug_mode = true;
    app.preview_tab = 1; // Switch to Logs tab

    // Add some backend logs
    app.backend_logs
        .push("[INFO] Backend started on tcp://127.0.0.1:5555".to_string());
    app.backend_logs
        .push("[INFO] ComfyUI health check: OK".to_string());
    app.backend_logs
        .push("[INFO] Model loaded: sdxl_base_1.0.safetensors".to_string());
    app.backend_logs
        .push("[WARN] Cache size exceeds 80%".to_string());

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_debug_logs", format!("{:?}", buffer));
}

/// Test generation screen on small terminal
#[tokio::test]
async fn test_generation_small_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_small_terminal();
    let mut app = App::new();

    for c in "small terminal test".chars() {
        app.input_char(c);
    }

    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("generation_small_terminal", format!("{:?}", buffer));
}

// ========================================
// Queue Screen Tests
// ========================================

/// Test queue screen initial state
#[tokio::test]
async fn test_queue_initial_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let app = App::new();

    terminal
        .draw(|f| queue::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("queue_initial", format!("{:?}", buffer));
}

/// Test queue screen on small terminal
#[tokio::test]
async fn test_queue_small_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_small_terminal();
    let app = App::new();

    terminal
        .draw(|f| queue::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("queue_small_terminal", format!("{:?}", buffer));
}

/// Test queue screen on large terminal
#[tokio::test]
async fn test_queue_large_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_large_terminal();
    let app = App::new();

    terminal
        .draw(|f| queue::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("queue_large_terminal", format!("{:?}", buffer));
}

// ========================================
// Help Screen Tests
// ========================================

/// Test help screen
#[tokio::test]
async fn test_help_screen_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let app = App::new();

    terminal
        .draw(|f| help::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("help_screen", format!("{:?}", buffer));
}

/// Test help screen on small terminal
#[tokio::test]
async fn test_help_small_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_small_terminal();
    let app = App::new();

    terminal
        .draw(|f| help::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("help_small_terminal", format!("{:?}", buffer));
}

/// Test help screen on large terminal
#[tokio::test]
async fn test_help_large_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_large_terminal();
    let app = App::new();

    terminal
        .draw(|f| help::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("help_large_terminal", format!("{:?}", buffer));
}

// ========================================
// Settings Screen Tests
// ========================================

/// Test settings screen
#[tokio::test]
async fn test_settings_screen_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let app = App::new();

    terminal
        .draw(|f| settings::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("settings_screen", format!("{:?}", buffer));
}

/// Test settings screen on small terminal
#[tokio::test]
async fn test_settings_small_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_small_terminal();
    let app = App::new();

    terminal
        .draw(|f| settings::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("settings_small_terminal", format!("{:?}", buffer));
}

/// Test settings screen on large terminal
#[tokio::test]
async fn test_settings_large_terminal_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_large_terminal();
    let app = App::new();

    terminal
        .draw(|f| settings::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("settings_large_terminal", format!("{:?}", buffer));
}

// ========================================
// Navigation State Tests
// ========================================

/// Test app state with different screens navigated
#[tokio::test]
async fn test_navigation_state_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();

    // Navigate through screens
    app.navigate_to(Screen::Queue);
    app.navigate_to(Screen::Gallery);
    app.navigate_to(Screen::Help);

    // Back to Gallery
    app.navigate_back();
    assert_eq!(app.current_screen, Screen::Gallery);

    // Add some gallery images
    app.add_to_gallery(PathBuf::from("/test/sprite_001.png"));
    app.add_to_gallery(PathBuf::from("/test/sprite_002.png"));

    terminal
        .draw(|f| dgx_pixels_tui::ui::screens::gallery::render(f, &app))
        .expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("navigation_state_gallery", format!("{:?}", buffer));
}

/// Test empty state consistency across screens
#[tokio::test]
async fn test_empty_states_consistency() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let app = App::new();

    // Test generation screen empty state
    terminal
        .draw(|f| generation::render(f, &app))
        .expect("Failed to draw");
    let gen_buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("empty_state_generation", format!("{:?}", gen_buffer));

    // Test queue screen empty state
    terminal
        .draw(|f| queue::render(f, &app))
        .expect("Failed to draw");
    let queue_buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("empty_state_queue", format!("{:?}", queue_buffer));

    // Test gallery screen empty state
    terminal
        .draw(|f| dgx_pixels_tui::ui::screens::gallery::render(f, &app))
        .expect("Failed to draw");
    let gallery_buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!("empty_state_gallery", format!("{:?}", gallery_buffer));
}
