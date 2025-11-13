use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Asset metadata from manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default = "default_frames")]
    pub frames: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size_kb: Option<f32>,
}

fn default_frames() -> usize {
    1
}

/// Sprite manifest structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteManifest {
    #[serde(default)]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub sprite_count: usize,
    pub sprites: Vec<AssetMetadata>,
}

/// Resource for accessing manifest data
#[derive(Resource)]
pub struct ManifestData {
    pub manifest: SpriteManifest,
    pub sprites_by_name: HashMap<String, AssetMetadata>,
    pub sprites_by_category: HashMap<String, Vec<AssetMetadata>>,
}

impl ManifestData {
    /// Load manifest from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let manifest: SpriteManifest = serde_json::from_str(&contents)?;

        // Build lookup maps
        let mut sprites_by_name = HashMap::new();
        let mut sprites_by_category: HashMap<String, Vec<AssetMetadata>> = HashMap::new();

        for sprite in &manifest.sprites {
            sprites_by_name.insert(sprite.name.clone(), sprite.clone());

            if let Some(category) = &sprite.category {
                sprites_by_category
                    .entry(category.clone())
                    .or_insert_with(Vec::new)
                    .push(sprite.clone());
            }
        }

        Ok(Self {
            manifest,
            sprites_by_name,
            sprites_by_category,
        })
    }

    /// Get sprite metadata by name
    pub fn get_sprite(&self, name: &str) -> Option<&AssetMetadata> {
        self.sprites_by_name.get(name)
    }

    /// Get all sprites in a category
    pub fn get_sprites_by_category(&self, category: &str) -> Option<&Vec<AssetMetadata>> {
        self.sprites_by_category.get(category)
    }

    /// Get all sprite names
    pub fn get_all_names(&self) -> Vec<String> {
        self.sprites_by_name.keys().cloned().collect()
    }

    /// Check if sprite exists
    pub fn has_sprite(&self, name: &str) -> bool {
        self.sprites_by_name.contains_key(name)
    }
}

/// Load sprites from manifest and prepare handles
pub fn load_sprites_from_manifest(
    manifest: &ManifestData,
    asset_server: &AssetServer,
) -> HashMap<String, Handle<Image>> {
    let mut handles = HashMap::new();

    for (name, metadata) in &manifest.sprites_by_name {
        let handle: Handle<Image> = asset_server.load(&metadata.path);
        handles.insert(name.clone(), handle);
    }

    info!(
        "Loaded {} sprites from manifest",
        manifest.manifest.sprite_count
    );

    handles
}

/// Load specific sprites by category
pub fn load_sprites_by_category(
    manifest: &ManifestData,
    category: &str,
    asset_server: &AssetServer,
) -> HashMap<String, Handle<Image>> {
    let mut handles = HashMap::new();

    if let Some(sprites) = manifest.get_sprites_by_category(category) {
        for sprite in sprites {
            let handle: Handle<Image> = asset_server.load(&sprite.path);
            handles.insert(sprite.name.clone(), handle);
        }

        info!(
            "Loaded {} sprites from category '{}'",
            sprites.len(),
            category
        );
    } else {
        warn!("Category '{}' not found in manifest", category);
    }

    handles
}

/// Plugin for manifest loading
pub struct ManifestLoaderPlugin {
    pub manifest_path: String,
}

impl Default for ManifestLoaderPlugin {
    fn default() -> Self {
        Self {
            manifest_path: "assets/sprites_manifest.json".to_string(),
        }
    }
}

impl Plugin for ManifestLoaderPlugin {
    fn build(&self, app: &mut App) {
        // Try to load manifest
        match ManifestData::load_from_file(&self.manifest_path) {
            Ok(manifest_data) => {
                info!(
                    "Loaded sprite manifest with {} sprites",
                    manifest_data.manifest.sprite_count
                );
                app.insert_resource(manifest_data);
            }
            Err(e) => {
                warn!(
                    "Failed to load sprite manifest from '{}': {}",
                    self.manifest_path, e
                );
                warn!("Sprite manifest features will be unavailable");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let json = r#"{
            "version": "1.0",
            "sprite_count": 2,
            "sprites": [
                {
                    "name": "character_knight_idle",
                    "path": "sprites/character_knight_idle_0001.png",
                    "category": "character",
                    "frames": 4,
                    "resolution": [64, 64]
                },
                {
                    "name": "enemy_goblin_attack",
                    "path": "sprites/enemy_goblin_attack_0001.png",
                    "category": "enemy",
                    "frames": 6,
                    "resolution": [32, 32]
                }
            ]
        }"#;

        let manifest: SpriteManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.sprite_count, 2);
        assert_eq!(manifest.sprites.len(), 2);
        assert_eq!(manifest.sprites[0].name, "character_knight_idle");
        assert_eq!(manifest.sprites[0].frames, 4);
    }

    #[test]
    fn test_manifest_data_lookup() {
        let manifest = SpriteManifest {
            version: "1.0".to_string(),
            generated_at: None,
            sprite_count: 1,
            sprites: vec![AssetMetadata {
                name: "test_sprite".to_string(),
                path: "sprites/test.png".to_string(),
                category: Some("character".to_string()),
                variant: None,
                frames: 1,
                resolution: Some([64, 64]),
                generated_at: None,
                prompt: None,
                workflow: None,
                file_size_kb: None,
            }],
        };

        let mut sprites_by_name = HashMap::new();
        sprites_by_name.insert("test_sprite".to_string(), manifest.sprites[0].clone());

        let mut sprites_by_category = HashMap::new();
        sprites_by_category
            .entry("character".to_string())
            .or_insert_with(Vec::new)
            .push(manifest.sprites[0].clone());

        let manifest_data = ManifestData {
            manifest,
            sprites_by_name,
            sprites_by_category,
        };

        assert!(manifest_data.has_sprite("test_sprite"));
        assert!(!manifest_data.has_sprite("nonexistent"));

        let sprite = manifest_data.get_sprite("test_sprite").unwrap();
        assert_eq!(sprite.name, "test_sprite");
        assert_eq!(sprite.category, Some("character".to_string()));

        let characters = manifest_data.get_sprites_by_category("character").unwrap();
        assert_eq!(characters.len(), 1);
    }
}
