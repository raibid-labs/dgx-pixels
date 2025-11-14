#![cfg(feature = "bevy_migration_foundation")]

use bevy::prelude::*;
use dgx_pixels_tui::bevy_app::DgxPixelsPlugin;

#[test]
fn test_bevy_app_builds() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);

    // Verify app built successfully (shouldn't panic)
    // If we got here without panicking, the test passes
}

#[test]
fn test_bevy_app_updates() {
    let mut app = App::new();
    app.add_plugins(DgxPixelsPlugin);

    // Run one frame - shouldn't panic
    app.update();

    // Verify app still functional - run another frame
    app.update();
}

#[test]
fn test_bevy_app_config() {
    use dgx_pixels_tui::bevy_app::BevyAppConfig;
    use std::time::Duration;

    // Test default config
    let config = BevyAppConfig::default();
    assert_eq!(config.update_rate, Duration::from_secs_f32(1.0 / 60.0));

    // Test custom FPS
    let config_30fps = BevyAppConfig::default().with_update_rate(30);
    assert_eq!(
        config_30fps.update_rate,
        Duration::from_secs_f32(1.0 / 30.0)
    );
}
