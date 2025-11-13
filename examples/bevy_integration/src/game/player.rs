use bevy::prelude::*;

/// Player component with movement speed
#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

/// Setup player entity with sprite
///
/// This function creates the player entity with:
/// - Sprite component (loaded from assets)
/// - Transform (position, rotation, scale)
/// - Player component (game logic data)
pub fn setup_player(commands: &mut Commands, asset_server: &AssetServer) {
    // Load player sprite
    // In production, this would be an AI-generated sprite
    // from DGX-Pixels via MCP
    //
    // Note: This example uses a placeholder sprite. Create a simple
    // 32x32 PNG at assets/placeholder/player.png or the sprite will
    // appear as a white square. See assets/README.md for instructions.
    let texture = asset_server.load("placeholder/player.png");

    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::splat(2.0)), // Scale up for visibility
            sprite: Sprite {
                color: Color::rgb(0.2, 0.4, 1.0), // Blue tint as fallback
                ..default()
            },
            ..default()
        },
        Player { speed: 200.0 },
    ));

    info!("Player spawned at origin with 200.0 speed");
    info!("Note: Create assets/placeholder/player.png for custom sprite (see assets/README.md)");
}

/// System to handle player movement with WASD keys
///
/// Movement is normalized to prevent diagonal speed boost
/// and scaled by delta time for frame-rate independence
pub fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    for (player, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Gather input
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // Apply movement
        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * player.speed * time.delta_seconds();
        }
    }
}

/// Example of how to integrate AI sprite generation
///
/// This function shows how to request sprite generation
/// via MCP and load the result. Uncomment and adapt for real use.
#[allow(dead_code)]
fn generate_and_load_player_sprite(
    commands: &mut Commands,
    asset_server: &AssetServer,
    mcp_client: &crate::mcp_client::McpClient,
) {
    // Request AI generation via MCP
    let result = mcp_client.generate_sprite_sync(crate::mcp_client::GenerateSpriteParams {
        prompt: "pixel art medieval knight character sprite, 32x32, front view, blue armor".to_string(),
        style: "pixel_art".to_string(),
        resolution: "1024x1024".to_string(),
    });

    match result {
        Ok(sprite_path) => {
            info!("Generated player sprite at: {}", sprite_path);

            // Load the generated sprite
            let texture = asset_server.load(sprite_path);

            // Spawn player with generated sprite
            commands.spawn((
                SpriteBundle {
                    texture,
                    transform: Transform::from_xyz(0.0, 0.0, 0.0)
                        .with_scale(Vec3::splat(2.0)),
                    ..default()
                },
                Player { speed: 200.0 },
            ));
        }
        Err(e) => {
            error!("Failed to generate player sprite: {}", e);
        }
    }
}
