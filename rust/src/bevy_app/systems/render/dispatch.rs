//! # Rendering Dispatch System
//!
//! Coordinates frame updates and state tracking for screen-specific renderers.
//! Each screen has its own dedicated render system that handles full frame rendering.

use bevy::prelude::*;

use crate::bevy_app::resources::*;

/// Main rendering dispatch system.
///
/// This system runs in Update schedule and coordinates frame state.
/// Actual rendering is delegated to screen-specific render systems:
/// - render_generation_screen (WS-09)
/// - render_gallery_screen (WS-10)
/// - render_comparison_screen (WS-11)
/// - render_models_screen (WS-12)
/// - render_queue_screen (WS-13)
/// - render_monitor_screen (WS-14)
/// - render_settings_screen (WS-15)
/// - render_help_screen (WS-16)
///
/// Each screen-specific system checks CurrentScreen and only renders when active.
pub fn render_dispatch(
    current_screen: Res<CurrentScreen>,
    mut app_state: ResMut<AppState>,
) {
    // Track frame rendering
    // Screen-specific systems handle actual ratatui drawing

    // Update frame state
    app_state.mark_rendered();

    // Log screen changes in debug mode
    if app_state.debug_mode {
        trace!("Frame {} - Screen: {:?}", app_state.frame_count, current_screen.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_render_dispatch_compiles() {
        // Basic compilation test
        let mut app = App::new();
        app.add_systems(Update, render_dispatch);
    }

    #[test]
    fn test_render_dispatch_updates_frame_count() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Generation));
        app.insert_resource(AppState::default());

        app.add_systems(Update, render_dispatch);

        let initial_frame = app.world().resource::<AppState>().frame_count;
        app.update();
        let after_frame = app.world().resource::<AppState>().frame_count;

        assert_eq!(after_frame, initial_frame + 1, "Frame count should increment");
    }
}
