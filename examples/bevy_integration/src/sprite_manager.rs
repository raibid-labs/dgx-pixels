use bevy::prelude::*;
use std::collections::HashMap;

/// Manages sprite assets and provides caching layer
///
/// This resource keeps track of loaded sprites to avoid
/// loading the same asset multiple times. It integrates
/// with Bevy's asset system and supports hot-reloading.
#[derive(Resource)]
pub struct SpriteManager {
    #[allow(dead_code)]
    pub sprites: HashMap<String, Handle<Image>>,
}

impl Default for SpriteManager {
    fn default() -> Self {
        Self {
            sprites: HashMap::new(),
        }
    }
}

impl SpriteManager {
    /// Load a sprite or return existing handle if already loaded
    ///
    /// # Arguments
    /// * `name` - Unique identifier for this sprite
    /// * `path` - Asset path relative to assets/ directory
    /// * `asset_server` - Bevy's AssetServer resource
    ///
    /// # Example
    /// ```
    /// let handle = sprite_manager.load_or_get(
    ///     "player_idle",
    ///     "sprites/player_idle.png",
    ///     &asset_server
    /// );
    /// ```
    #[allow(dead_code)]
    pub fn load_or_get(
        &mut self,
        name: impl Into<String>,
        path: impl Into<String>,
        asset_server: &AssetServer,
    ) -> Handle<Image> {
        let name = name.into();
        if let Some(handle) = self.sprites.get(&name) {
            debug!("Sprite '{}' already loaded, returning cached handle", name);
            handle.clone()
        } else {
            let path = path.into();
            info!("Loading new sprite '{}' from '{}'", name, path);
            let handle: Handle<Image> = asset_server.load(path);
            self.sprites.insert(name.clone(), handle.clone());
            handle
        }
    }

    /// Preload multiple sprites at once
    #[allow(dead_code)]
    pub fn preload_batch(
        &mut self,
        sprites: Vec<(String, String)>,
        asset_server: &AssetServer,
    ) {
        info!("Preloading {} sprites", sprites.len());
        for (name, path) in sprites {
            self.load_or_get(name, path, asset_server);
        }
    }

    /// Check if a sprite is loaded
    #[allow(dead_code)]
    pub fn has_sprite(&self, name: &str) -> bool {
        self.sprites.contains_key(name)
    }

    /// Get sprite handle if it exists
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<Handle<Image>> {
        self.sprites.get(name).cloned()
    }
}

pub struct SpriteManagerPlugin;

impl Plugin for SpriteManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteManager>();
        info!("Sprite Manager Plugin initialized");
    }
}
