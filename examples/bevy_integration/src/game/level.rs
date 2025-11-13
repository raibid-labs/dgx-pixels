use bevy::prelude::*;

/// Level configuration
pub struct Level {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            width: 20,
            height: 15,
            tile_size: 32.0,
        }
    }
}

/// Setup level tiles
///
/// This function would generate or load level tiles.
/// In production, tiles would be AI-generated via MCP.
#[allow(dead_code)]
pub fn setup_level(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    let level = Level::default();

    info!(
        "Setting up level: {}x{} tiles of size {}",
        level.width, level.height, level.tile_size
    );

    // Example: Load ground tile
    let ground_texture = asset_server.load("placeholder/ground.png");

    // Spawn tiles in grid
    for y in 0..level.height {
        for x in 0..level.width {
            let pos_x = (x as f32 - level.width as f32 / 2.0) * level.tile_size;
            let pos_y = (y as f32 - level.height as f32 / 2.0) * level.tile_size;

            commands.spawn(SpriteBundle {
                texture: ground_texture.clone(),
                transform: Transform::from_xyz(pos_x, pos_y, -1.0),
                ..default()
            });
        }
    }
}

/// Example of AI-generated tileset via MCP
///
/// This shows how to generate a complete tileset
/// (ground, wall, water, etc.) in one batch request
#[allow(dead_code)]
pub fn generate_tileset(
    mcp_client: &crate::mcp_client::McpClient,
) -> Result<Vec<String>, String> {
    let tileset_prompts = vec![
        crate::mcp_client::GenerateSpriteParams {
            prompt: "pixel art grass ground tile, 32x32, seamless, top-down view".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
        crate::mcp_client::GenerateSpriteParams {
            prompt: "pixel art stone wall tile, 32x32, seamless, top-down view".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
        crate::mcp_client::GenerateSpriteParams {
            prompt: "pixel art water tile, 32x32, seamless, animated, top-down view".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
    ];

    info!("Requesting tileset generation ({} tiles)", tileset_prompts.len());

    mcp_client.generate_batch_sync(tileset_prompts)
}
