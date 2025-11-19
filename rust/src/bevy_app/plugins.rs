use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
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
        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(config.update_rate)));

        // Add logging plugin
        app.add_plugins(LogPlugin::default());

        // Add asset plugin for image loading (WS-06)
        app.add_plugins(bevy::asset::AssetPlugin::default());

        // Initialize Image asset type (required for AssetServer::load)
        app.init_asset::<Image>();

        // Ratatui terminal rendering
        app.add_plugins(RatatuiPlugins::default());

        // WS-02: State initialization
        app.add_systems(Startup, systems::init_app_state);

        // WS-06: Image asset cache
        app.insert_resource(systems::assets::ImageCache::default());

        // T10: Gallery scan state for preview manager
        app.insert_resource(systems::assets::GalleryScanState::default());

        // WS-07: Theme resource
        app.insert_resource(super::resources::AppTheme::default());

        // WS-11: Comparison state resource
        app.insert_resource(super::resources::ComparisonState::default());

        // WS-12: Models state resource
        app.insert_resource(super::resources::ModelsState::default());

        // T3: Settings state resource (needed by gallery screen)
        app.insert_resource(super::resources::SettingsState::default());

        // T8: ZeroMQ client for backend communication (optional - graceful degradation if backend offline)
        match crate::zmq_client::ZmqClient::new_default() {
            Ok(client) => {
                info!("ZMQ client connected to backend");
                app.insert_resource(systems::zmq::ZmqClientResource::new(client));
            }
            Err(e) => {
                warn!("Failed to connect to backend - jobs will be created but not processed: {}", e);
            }
        }

        // WS-03: Global input systems (run in PreUpdate schedule)
        // These systems handle cross-screen functionality like quit, help, and navigation
        app.add_systems(
            PreUpdate,
            (
                systems::input::handle_keyboard_input,   // Global keys: q, ?, h
                systems::input::handle_navigation,       // Tab, numbers 1-8, Esc
                systems::input::handle_text_input,       // Text entry on Generation screen
            ),
        );

        // WS-03: Screen-specific input handlers (run in PreUpdate after global handlers)
        // Each handler checks CurrentScreen and only processes events for its screen
        app.add_systems(
            PreUpdate,
            (
                systems::input::screens::handle_generation_input,  // Enter, G, C, P, L
                systems::input::screens::handle_gallery_input,     // Arrow keys, d, Home/End
                systems::input::screens::handle_comparison_input,  // Arrow keys, a, d, Enter
                systems::input::screens::handle_models_input,      // Arrow keys, Enter, d, i
                systems::input::screens::handle_queue_input,       // (Future: job navigation)
                systems::input::screens::handle_monitor_input,     // r, p (refresh/pause)
                systems::input::screens::settings::handle_settings_input, // Settings toggles
                systems::input::screens::handle_help_input,        // Read-only screen
            ),
        );

        // WS-05: ZeroMQ polling (run in PreUpdate before other systems)
        app.add_systems(PreUpdate, systems::zmq::poll_zmq);

        // WS-06: Image asset loading systems
        app.add_systems(
            Update,
            (
                systems::assets::load_preview_images,
                systems::assets::loader::load_gallery_images,
                systems::assets::loader::check_asset_loading,
            ),
        );

        // T10: Preview manager - periodic gallery scan (every 2 seconds)
        app.add_systems(
            Update,
            systems::assets::scan_gallery_directory
                .run_if(on_timer(std::time::Duration::from_secs(
                    systems::assets::SCAN_INTERVAL_SECS,
                ))),
        );

        // T10: Preview loading status checker (runs every frame)
        app.add_systems(Update, systems::assets::check_preview_loading);

        // WS-06: Periodic cache eviction (run every 60 seconds)
        app.add_systems(
            Update,
            systems::assets::cache::evict_old_cache_entries
                .run_if(on_timer(std::time::Duration::from_secs(60))),
        );

        // WS-04: Rendering dispatch (tracks frame state, runs independently)
        app.add_systems(Update, systems::render::render_dispatch);

        // Screen Rendering: All render systems chained sequentially
        // This is necessary because all screens need ResMut<RatatuiContext> exclusively
        // Only run when RatatuiContext is available
        app.add_systems(
            Update,
            (
                systems::render::screens::render_generation_screen,
                systems::render::screens::render_gallery_screen,
                systems::render::screens::render_comparison_screen,
                systems::render::screens::render_models_screen,
                systems::render::screens::render_queue_screen,
                systems::render::screens::render_monitor_screen,
                systems::render::screens::settings::render_settings_screen,
                systems::render::screens::render_help_screen,
            ).chain()
             .run_if(bevy::prelude::resource_exists::<bevy_ratatui::terminal::RatatuiContext>),
        );

        // WS-08: Event bus registration
        app.add_event::<super::events::NavigateToScreen>();
        app.add_event::<super::events::NavigateBack>();
        app.add_event::<super::events::SubmitGenerationJob>();
        app.add_event::<super::events::GenerationComplete>();
        app.add_event::<super::events::CancelJob>();
        app.add_event::<super::events::SelectNextImage>();
        app.add_event::<super::events::SelectPreviousImage>();
        app.add_event::<super::events::DeleteImage>();

        // Event handlers (run in Update after input processing)
        app.add_systems(
            Update,
            (
                super::events::handle_navigation_events,
                super::events::handle_generation_events,
                super::events::handle_gallery_events,
                systems::zmq::handle_zmq_responses,
            ),
        );

        info!("DgxPixelsPlugin initialized with T10 Preview Manager and all 8 screens");
    }
}
