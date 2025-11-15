use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_ratatui::RatatuiPlugins;

use super::systems;
use super::BevyAppConfig;

/// Main plugin for DGX-Pixels Bevy app.
pub struct DgxPixelsPlugin;

impl Plugin for DgxPixelsPlugin {
    fn build(&self, app: &mut App) {
        // Configuration
        let config = BevyAppConfig::default();

        // Bevy minimal plugins (no windowing)
        app.add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(config.update_rate)),
        );

        // Add logging plugin
        app.add_plugins(LogPlugin::default());

        // Ratatui terminal rendering
        app.add_plugins(RatatuiPlugins::default());

        // WS-02: State initialization
        app.add_systems(Startup, systems::init_app_state);

        // WS-07: Theme resource
        app.insert_resource(super::resources::AppTheme::default());

        // WS-03: Input systems (run in PreUpdate schedule)
        app.add_systems(
            PreUpdate,
            (
                systems::input::handle_keyboard_input,
                systems::input::handle_navigation,
                systems::input::handle_text_input,
            )
                .chain(), // Run in order
        );

        // WS-04: Rendering system (run in Update schedule)
        app.add_systems(Update, systems::render::render_dispatch);

        // WS-08: Event bus
        app.add_event::<super::events::NavigateToScreen>();
        app.add_event::<super::events::NavigateBack>();
        app.add_event::<super::events::SubmitGenerationJob>();
        app.add_event::<super::events::GenerationComplete>();
        app.add_event::<super::events::CancelJob>();
        app.add_event::<super::events::SelectNextImage>();
        app.add_event::<super::events::SelectPreviousImage>();
        app.add_event::<super::events::DeleteImage>();

        app.add_systems(
            Update,
            (
                super::events::handle_navigation_events,
                super::events::handle_generation_events,
                super::events::handle_gallery_events,
            ),
        );

        info!("DgxPixelsPlugin initialized with state, input, rendering, and event systems");
    }
}
