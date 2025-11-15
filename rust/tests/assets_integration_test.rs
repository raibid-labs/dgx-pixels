//! Integration tests for the image asset loading system (WS-06A).
//!
//! These tests verify that the asset loading infrastructure works correctly
//! with Bevy's asset system.

#![cfg(feature = "bevy_migration_foundation")]

use dgx_pixels_tui::bevy_app::assets::{
    validate_image_path, AssetCache, CacheConfig, ImageAssetPlugin, ImageLoadError,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_validate_supported_formats() {
    let temp_dir = TempDir::new().unwrap();

    // Test all supported formats
    for ext in &["png", "jpg", "jpeg", "webp"] {
        let path = temp_dir.path().join(format!("test.{}", ext));
        fs::write(&path, b"fake image data").unwrap();

        let result = validate_image_path(&path);
        assert!(
            result.is_ok(),
            "Format {} should be supported, but got error: {:?}",
            ext,
            result.err()
        );
    }
}

#[test]
fn test_validate_unsupported_formats() {
    let temp_dir = TempDir::new().unwrap();

    // Test unsupported formats
    for ext in &["bmp", "gif", "tiff", "svg"] {
        let path = temp_dir.path().join(format!("test.{}", ext));
        fs::write(&path, b"fake image data").unwrap();

        let result = validate_image_path(&path);
        assert!(
            matches!(result, Err(ImageLoadError::UnsupportedFormat(_))),
            "Format {} should be unsupported",
            ext
        );
    }
}

#[test]
fn test_validate_missing_file() {
    let result = validate_image_path("/nonexistent/path/image.png");
    assert!(matches!(result, Err(ImageLoadError::NotFound(_))));
}

#[test]
fn test_cache_basic_operations() {
    use bevy::prelude::*;

    let mut cache = AssetCache::default_config();

    // Initially empty
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());

    // Insert an entry
    let path = std::path::PathBuf::from("/test/image.png");
    let handle: Handle<Image> = Handle::default();
    cache.insert(path.clone(), handle.clone(), 1024 * 1024); // 1MB

    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());
    assert_eq!(cache.stats().current_entries, 1);
    assert_eq!(cache.stats().current_memory_bytes, 1024 * 1024);

    // Retrieve the entry (cache hit)
    let retrieved = cache.get(&path);
    assert!(retrieved.is_some());
    assert_eq!(cache.stats().hits, 1);
    assert_eq!(cache.stats().misses, 0);

    // Try to get non-existent entry (cache miss)
    let missing = cache.get(&std::path::PathBuf::from("/nonexistent.png"));
    assert!(missing.is_none());
    assert_eq!(cache.stats().misses, 1);
}

#[test]
fn test_cache_lru_eviction() {
    use bevy::prelude::*;

    let config = CacheConfig {
        max_memory_mb: 1024,
        max_entries: 3, // Limit to 3 entries
    };
    let mut cache = AssetCache::new(config);

    // Insert 3 entries
    for i in 0..3 {
        let path = std::path::PathBuf::from(format!("/test/image{}.png", i));
        cache.insert(path, Handle::default(), 100 * 1024); // 100KB each
    }

    assert_eq!(cache.len(), 3);
    assert_eq!(cache.stats().evictions, 0);

    // Insert a 4th entry - should evict the oldest (image0)
    let path3 = std::path::PathBuf::from("/test/image3.png");
    cache.insert(path3.clone(), Handle::default(), 100 * 1024);

    assert_eq!(cache.len(), 3);
    assert_eq!(cache.stats().evictions, 1);

    // Verify image0 was evicted
    let evicted_path = std::path::PathBuf::from("/test/image0.png");
    assert!(cache.get(&evicted_path).is_none());
}

#[test]
fn test_cache_memory_limit() {
    use bevy::prelude::*;

    let config = CacheConfig {
        max_memory_mb: 1, // 1MB limit
        max_entries: 100,
    };
    let mut cache = AssetCache::new(config);

    // Insert images totaling > 1MB
    for i in 0..5 {
        let path = std::path::PathBuf::from(format!("/test/large{}.png", i));
        cache.insert(path, Handle::default(), 512 * 1024); // 512KB each
    }

    // Should have evicted to stay under 1MB limit
    let stats = cache.stats();
    assert!(
        stats.current_memory_bytes <= 1024 * 1024,
        "Cache exceeded memory limit: {} bytes",
        stats.current_memory_bytes
    );
    assert!(
        stats.evictions > 0,
        "Expected evictions due to memory pressure"
    );
}

#[test]
fn test_cache_stats_hit_rate() {
    use bevy::prelude::*;

    let mut cache = AssetCache::default_config();
    let path = std::path::PathBuf::from("/test/image.png");

    // Insert and access multiple times
    cache.insert(path.clone(), Handle::default(), 1024);

    for _ in 0..9 {
        cache.get(&path); // Hit
    }
    cache.get(&std::path::PathBuf::from("/miss.png")); // Miss

    let stats = cache.stats();
    assert_eq!(stats.hits, 9);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hit_rate(), 0.9); // 90% hit rate
}

#[test]
fn test_cache_clear() {
    use bevy::prelude::*;

    let mut cache = AssetCache::default_config();

    // Insert multiple entries
    for i in 0..5 {
        let path = std::path::PathBuf::from(format!("/test/image{}.png", i));
        cache.insert(path, Handle::default(), 1024);
    }

    assert_eq!(cache.len(), 5);
    assert!(cache.stats().current_memory_bytes > 0);

    // Clear cache
    cache.clear();

    assert_eq!(cache.len(), 0);
    assert_eq!(cache.stats().current_memory_bytes, 0);
    assert!(cache.is_empty());
}

#[test]
fn test_image_asset_plugin_registers() {
    use bevy::prelude::*;

    // Verify plugin can be registered without errors
    let mut app = App::new();
    app.add_plugins(ImageAssetPlugin);

    // Plugin should have registered the ImageLoader resource
    // (This will be properly testable once we integrate with full Bevy app)
}
