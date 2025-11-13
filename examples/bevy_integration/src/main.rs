use bevy::prelude::*;

mod mcp_client;
mod sprite_manager;
mod manifest_loader;
mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .add_plugins(mcp_client::McpPlugin)
        .add_plugins(sprite_manager::SpriteManagerPlugin)
        .add_plugins(manifest_loader::ManifestLoaderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, game::player::move_player)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    manifest: Option<Res<manifest_loader::ManifestData>>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // If manifest is available, log available sprites
    if let Some(manifest) = manifest {
        info!(
            "Sprite manifest loaded with {} sprites",
            manifest.manifest.sprite_count
        );

        // Log available categories
        let categories: std::collections::HashSet<_> = manifest
            .manifest
            .sprites
            .iter()
            .filter_map(|s| s.category.as_ref())
            .collect();

        if !categories.is_empty() {
            info!("Available categories: {:?}", categories);
        }
    } else {
        info!("No sprite manifest found, using direct asset loading");
    }

    // Setup game
    game::player::setup_player(&mut commands, &asset_server);

    info!("DGX-Pixels Bevy Integration Example Started");
    info!("Controls: WASD to move, ESC to quit");
}
