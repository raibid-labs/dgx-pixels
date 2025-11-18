//! # Sixel Preview System
//!
//! Bevy ECS-based preview system with Sixel caching.
//! Replaces the old PreviewManager with a Bevy resource-based approach.

use anyhow::Result;
use bevy::prelude::*;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

use super::sixel_renderer::{render_image_sixel, SixelRenderOptions};

/// Maximum preview cache size in MB
pub const MAX_CACHE_SIZE_MB: usize = 50;

/// Sixel preview cache entry
#[derive(Debug, Clone)]
pub struct SixelCacheEntry {
    /// Path to the image file
    pub path: PathBuf,
    /// Rendered Sixel string
    pub sixel_data: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// When it was last accessed
    pub last_access: Instant,
    /// Image dimensions
    pub dimensions: (u32, u32),
}

/// Sixel preview cache resource
///
/// Stores pre-rendered Sixel strings to avoid re-encoding every frame.
/// Uses DashMap for concurrent access from multiple systems.
#[derive(Resource)]
pub struct SixelPreviewCache {
    /// Cache entries (path -> SixelCacheEntry)
    cache: Arc<DashMap<PathBuf, SixelCacheEntry>>,
    /// Maximum cache size in bytes
    max_size_bytes: usize,
    /// Current cache size in bytes
    current_size: Arc<parking_lot::RwLock<usize>>,
}

impl SixelPreviewCache {
    /// Create a new Sixel preview cache
    pub fn new() -> Self {
        info!(
            "Initializing Sixel preview cache with {}MB limit",
            MAX_CACHE_SIZE_MB
        );

        Self {
            cache: Arc::new(DashMap::new()),
            max_size_bytes: MAX_CACHE_SIZE_MB * 1024 * 1024,
            current_size: Arc::new(parking_lot::RwLock::new(0)),
        }
    }

    /// Get a cached Sixel entry
    pub fn get(&self, path: &Path) -> Option<SixelCacheEntry> {
        self.cache.get(path).map(|entry| {
            debug!("Sixel cache hit: {:?}", path);
            let mut entry = entry.clone();
            entry.last_access = Instant::now();
            entry
        })
    }

    /// Insert a Sixel entry into the cache
    pub fn insert(&self, entry: SixelCacheEntry) {
        let size = entry.size_bytes;
        let path = entry.path.clone();

        // Evict if needed
        self.evict_if_needed(size);

        // Insert entry
        self.cache.insert(path.clone(), entry);
        *self.current_size.write() += size;

        debug!("Cached Sixel: {:?} ({} bytes)", path, size);
    }

    /// Check if entry exists in cache
    pub fn contains(&self, path: &Path) -> bool {
        self.cache.contains_key(path)
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        info!("Clearing Sixel preview cache");
        self.cache.clear();
        *self.current_size.write() = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> SixelCacheStats {
        SixelCacheStats {
            entries: self.cache.len(),
            size_bytes: *self.current_size.read(),
            max_size_bytes: self.max_size_bytes,
        }
    }

    /// Evict LRU entries if needed
    fn evict_if_needed(&self, required_space: usize) {
        let current = *self.current_size.read();

        if current + required_space <= self.max_size_bytes {
            return; // No eviction needed
        }

        debug!(
            "Evicting LRU entries (need {} bytes, have {} bytes)",
            required_space, current
        );

        // Collect entries sorted by access time
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().last_access,
                    entry.value().size_bytes,
                )
            })
            .collect();

        entries.sort_by_key(|(_, access, _)| *access);

        // Evict oldest until we have space
        let target_size = self.max_size_bytes.saturating_sub(required_space);
        let mut current_size = current;

        for (path, _, size) in entries {
            if current_size <= target_size {
                break;
            }

            self.cache.remove(&path);
            current_size = current_size.saturating_sub(size);
            debug!("Evicted: {:?} ({} bytes)", path, size);
        }

        *self.current_size.write() = current_size;
    }
}

impl Default for SixelPreviewCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct SixelCacheStats {
    pub entries: usize,
    pub size_bytes: usize,
    pub max_size_bytes: usize,
}

impl SixelCacheStats {
    /// Get cache usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.max_size_bytes == 0 {
            0.0
        } else {
            (self.size_bytes as f32 / self.max_size_bytes as f32) * 100.0
        }
    }

    /// Get size in MB
    pub fn size_mb(&self) -> f32 {
        self.size_bytes as f32 / (1024.0 * 1024.0)
    }
}

/// Bevy system to render images to Sixel and cache them
///
/// This system runs whenever images are loaded and need to be displayed.
/// It converts Bevy Image assets to Sixel format and caches the results.
pub fn cache_image_as_sixel(
    images: &Assets<Image>,
    cache: &SixelPreviewCache,
    handle: Handle<Image>,
    path: PathBuf,
    options: SixelRenderOptions,
) -> Result<SixelCacheEntry> {
    // Check cache first
    if let Some(entry) = cache.get(&path) {
        return Ok(entry);
    }

    // Get image from asset server
    let image = images
        .get(&handle)
        .ok_or_else(|| anyhow::anyhow!("Image asset not loaded: {:?}", path))?;

    // Render to Sixel
    let sixel_data = render_image_sixel(image, &options)?;

    // Create cache entry
    let entry = SixelCacheEntry {
        path: path.clone(),
        sixel_data: sixel_data.clone(),
        size_bytes: sixel_data.len(),
        last_access: Instant::now(),
        dimensions: (image.width(), image.height()),
    };

    // Cache it
    cache.insert(entry.clone());

    Ok(entry)
}

/// Helper function to get or render Sixel preview
///
/// This is a convenience function that can be called from render systems.
pub fn get_or_render_sixel(
    path: &Path,
    handle: Handle<Image>,
    images: &Assets<Image>,
    cache: &SixelPreviewCache,
    options: &SixelRenderOptions,
) -> Result<String> {
    // Check cache
    if let Some(entry) = cache.get(path) {
        return Ok(entry.sixel_data);
    }

    // Render and cache
    let entry = cache_image_as_sixel(
        images,
        cache,
        handle,
        path.to_path_buf(),
        options.clone(),
    )?;

    Ok(entry.sixel_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = SixelPreviewCache::new();
        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.size_bytes, 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = SixelPreviewCache::new();
        let path = PathBuf::from("/test/image.png");

        let entry = SixelCacheEntry {
            path: path.clone(),
            sixel_data: "test_sixel_data".to_string(),
            size_bytes: 100,
            last_access: Instant::now(),
            dimensions: (64, 64),
        };

        cache.insert(entry.clone());

        let retrieved = cache.get(&path);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().sixel_data, "test_sixel_data");

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.size_bytes, 100);
    }

    #[test]
    fn test_cache_contains() {
        let cache = SixelPreviewCache::new();
        let path = PathBuf::from("/test/image.png");

        assert!(!cache.contains(&path));

        let entry = SixelCacheEntry {
            path: path.clone(),
            sixel_data: "data".to_string(),
            size_bytes: 10,
            last_access: Instant::now(),
            dimensions: (32, 32),
        };

        cache.insert(entry);
        assert!(cache.contains(&path));
    }

    #[test]
    fn test_cache_stats() {
        let cache = SixelPreviewCache::new();
        let stats = cache.stats();

        assert_eq!(stats.usage_percent(), 0.0);
        assert_eq!(stats.size_mb(), 0.0);

        // Insert 1MB entry
        let entry = SixelCacheEntry {
            path: PathBuf::from("/test.png"),
            sixel_data: "x".repeat(1024 * 1024),
            size_bytes: 1024 * 1024,
            last_access: Instant::now(),
            dimensions: (128, 128),
        };

        cache.insert(entry);
        let stats = cache.stats();

        assert!((stats.size_mb() - 1.0).abs() < 0.1);
        assert!(stats.usage_percent() > 0.0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = SixelPreviewCache::new();
        let entry = SixelCacheEntry {
            path: PathBuf::from("/test.png"),
            sixel_data: "data".to_string(),
            size_bytes: 100,
            last_access: Instant::now(),
            dimensions: (64, 64),
        };

        cache.insert(entry);
        assert_eq!(cache.stats().entries, 1);

        cache.clear();
        assert_eq!(cache.stats().entries, 0);
        assert_eq!(cache.stats().size_bytes, 0);
    }
}
