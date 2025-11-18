//! # Image Asset Cache
//!
//! LRU cache management for loaded images to prevent memory exhaustion.

use bevy::prelude::*;
use bevy::utils::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, info};

/// Maximum number of images to keep in cache.
const MAX_CACHE_SIZE: usize = 100;

/// Maximum cache age in seconds before eviction.
const MAX_CACHE_AGE_SECS: u64 = 300; // 5 minutes

/// Image cache entry with access tracking.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub handle: Handle<Image>,
    pub path: PathBuf,
    pub last_access: Instant,
    pub load_time: Instant,
}

/// LRU cache for image assets.
///
/// Prevents memory exhaustion by limiting the number of loaded images
/// and evicting least recently used entries.
#[derive(Resource, Debug)]
pub struct ImageCache {
    /// Map of path to cache entry
    entries: HashMap<PathBuf, CacheEntry>,
    /// Access order (LRU tracking)
    access_queue: VecDeque<PathBuf>,
    /// Maximum cache size
    max_size: usize,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(MAX_CACHE_SIZE)
    }
}

impl ImageCache {
    /// Create a new image cache with specified max size.
    pub fn new(max_size: usize) -> Self {
        info!("Initializing image cache with max size: {}", max_size);
        Self {
            entries: HashMap::with_capacity(max_size),
            access_queue: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Insert an image into the cache.
    pub fn insert(&mut self, path: PathBuf, handle: Handle<Image>) {
        let entry = CacheEntry {
            handle: handle.clone(),
            path: path.clone(),
            last_access: Instant::now(),
            load_time: Instant::now(),
        };

        // Remove from queue if already present
        self.access_queue.retain(|p| p != &path);

        // Add to front of queue (most recent)
        self.access_queue.push_front(path.clone());

        // Insert entry
        self.entries.insert(path, entry);

        // Evict if over capacity
        self.evict_if_needed();
    }

    /// Get an image from cache, updating access time.
    pub fn get(&mut self, path: &PathBuf) -> Option<Handle<Image>> {
        if let Some(entry) = self.entries.get_mut(path) {
            // Update access time
            entry.last_access = Instant::now();

            // Move to front of queue
            self.access_queue.retain(|p| p != path);
            self.access_queue.push_front(path.clone());

            debug!("Cache hit for image: {:?}", path);
            Some(entry.handle.clone())
        } else {
            debug!("Cache miss for image: {:?}", path);
            None
        }
    }

    /// Check if image is in cache.
    pub fn contains(&self, path: &PathBuf) -> bool {
        self.entries.contains_key(path)
    }

    /// Get cache size.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Evict least recently used entries if cache is full.
    fn evict_if_needed(&mut self) {
        while self.entries.len() > self.max_size {
            if let Some(lru_path) = self.access_queue.pop_back() {
                if let Some(removed) = self.entries.remove(&lru_path) {
                    debug!("Evicted LRU image from cache: {:?}", removed.path);
                }
            }
        }
    }

    /// Evict old entries based on age.
    pub fn evict_old(&mut self) {
        let now = Instant::now();
        let max_age = std::time::Duration::from_secs(MAX_CACHE_AGE_SECS);

        let to_remove: Vec<PathBuf> = self
            .entries
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.last_access) > max_age)
            .map(|(path, _)| path.clone())
            .collect();

        for path in to_remove {
            self.entries.remove(&path);
            self.access_queue.retain(|p| p != &path);
            debug!("Evicted old image from cache: {:?}", path);
        }
    }

    /// Clear entire cache.
    pub fn clear(&mut self) {
        info!("Clearing image cache ({} entries)", self.entries.len());
        self.entries.clear();
        self.access_queue.clear();
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
            max_size: self.max_size,
            hit_rate: 0.0, // Would need to track hits/misses
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub max_size: usize,
    pub hit_rate: f32,
}

impl CacheStats {
    /// Get cache usage percentage.
    pub fn usage_percent(&self) -> f32 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.entries as f32 / self.max_size as f32) * 100.0
        }
    }
}

/// System to periodically evict old cache entries.
pub fn evict_old_cache_entries(mut cache: ResMut<ImageCache>) {
    cache.evict_old();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = ImageCache::new(10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.max_size, 10);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut app = App::new();
        app.add_plugins(AssetPlugin::default());

        let asset_server = app.world().resource::<AssetServer>();
        let handle = asset_server.load("test.png");

        let mut cache = ImageCache::new(10);
        let path = PathBuf::from("test.png");

        cache.insert(path.clone(), handle.clone());
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get(&path);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_cache_eviction() {
        let mut app = App::new();
        app.add_plugins(AssetPlugin::default());

        let asset_server = app.world().resource::<AssetServer>();

        let mut cache = ImageCache::new(3);

        // Insert 4 items (should evict 1)
        for i in 0..4 {
            let path = PathBuf::from(format!("test{}.png", i));
            let handle = asset_server.load(path.clone());
            cache.insert(path, handle);
        }

        assert_eq!(cache.len(), 3); // Max size is 3
    }

    #[test]
    fn test_cache_lru_order() {
        let mut app = App::new();
        app.add_plugins(AssetPlugin::default());

        let asset_server = app.world().resource::<AssetServer>();

        let mut cache = ImageCache::new(3);

        let path1 = PathBuf::from("test1.png");
        let path2 = PathBuf::from("test2.png");
        let path3 = PathBuf::from("test3.png");
        let path4 = PathBuf::from("test4.png");

        cache.insert(path1.clone(), asset_server.load(path1.clone()));
        cache.insert(path2.clone(), asset_server.load(path2.clone()));
        cache.insert(path3.clone(), asset_server.load(path3.clone()));

        // Access path1 to make it most recent
        cache.get(&path1);

        // Insert path4, should evict path2 (least recently used)
        cache.insert(path4.clone(), asset_server.load(path4.clone()));

        assert!(cache.contains(&path1)); // Still present (recently accessed)
        assert!(!cache.contains(&path2)); // Evicted
        assert!(cache.contains(&path3)); // Still present
        assert!(cache.contains(&path4)); // Just inserted
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ImageCache::new(10);
        let stats = cache.stats();

        assert_eq!(stats.entries, 0);
        assert_eq!(stats.max_size, 10);
        assert_eq!(stats.usage_percent(), 0.0);
    }
}
