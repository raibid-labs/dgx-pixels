use bevy::prelude::*;

/// Enemy component with health tracking
#[derive(Component)]
pub struct Enemy {
    #[allow(dead_code)]
    pub health: i32,
    #[allow(dead_code)]
    pub enemy_type: EnemyType,
}

/// Different enemy types that can be generated
#[derive(Debug, Clone, Copy)]
pub enum EnemyType {
    Goblin,
    Skeleton,
    #[allow(dead_code)]
    Dragon,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType) -> Self {
        let health = match enemy_type {
            EnemyType::Goblin => 50,
            EnemyType::Skeleton => 75,
            EnemyType::Dragon => 200,
        };

        Self {
            health,
            enemy_type,
        }
    }
}

/// Example of batch enemy generation via MCP
///
/// This shows how to generate multiple enemy sprites
/// in one batch request for better performance
#[allow(dead_code)]
pub fn generate_enemy_batch(
    commands: &mut Commands,
    asset_server: &AssetServer,
    mcp_client: &crate::mcp_client::McpClient,
) {
    let enemy_prompts = vec![
        crate::mcp_client::GenerateSpriteParams {
            prompt: "pixel art goblin enemy sprite, 32x32, green skin, carrying club".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
        crate::mcp_client::GenerateSpriteParams {
            prompt: "pixel art skeleton warrior sprite, 32x32, white bones, sword".to_string(),
            style: "pixel_art".to_string(),
            resolution: "1024x1024".to_string(),
        },
    ];

    match mcp_client.generate_batch_sync(enemy_prompts) {
        Ok(paths) => {
            info!("Generated {} enemy sprites", paths.len());

            // Spawn enemies with generated sprites
            for (i, path) in paths.iter().enumerate() {
                let texture = asset_server.load(path.clone());
                let enemy_type = if i == 0 { EnemyType::Goblin } else { EnemyType::Skeleton };

                commands.spawn((
                    SpriteBundle {
                        texture,
                        transform: Transform::from_xyz(
                            (i as f32) * 100.0 - 200.0,
                            -200.0,
                            0.0,
                        ).with_scale(Vec3::splat(2.0)),
                        ..default()
                    },
                    Enemy::new(enemy_type),
                ));
            }
        }
        Err(e) => {
            error!("Failed to generate enemy batch: {}", e);
        }
    }
}
