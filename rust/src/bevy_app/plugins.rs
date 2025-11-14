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

        // Future: Add input systems (WS-03)
        // Future: Add rendering systems (WS-04)

        info!("DgxPixelsPlugin initialized with state resources");
    }
}
