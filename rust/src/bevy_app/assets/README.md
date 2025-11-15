# Image Asset Loading System (WS-06A)

This module implements the image asset loading infrastructure for the Bevy-based TUI system.

## Overview

The asset system provides:
- **Async image loading** using Bevy's `AssetServer`
- **LRU cache management** with configurable memory limits
- **Multi-format support** for PNG, JPG/JPEG, and WebP

## Architecture

### Components

1. **ImageLoader** (`image_loader.rs`)
   - Async image loading via Bevy's asset system
   - Load state tracking and error logging
   - Format validation (PNG, JPG, WebP)
   - Path validation utilities

2. **AssetCache** (`cache.rs`)
   - LRU (Least Recently Used) eviction policy
   - Memory-based eviction (default 512MB limit)
   - Entry count limits (default 100 images)
   - Hit/miss statistics tracking

## Usage

### Loading Images

```rust
use bevy::prelude::*;
use dgx_pixels_tui::bevy_app::assets::load_image;

fn my_system(asset_server: Res<AssetServer>) {
    // Load an image (async)
    let handle = load_image(&asset_server, "output/sprite.png");

    // Use handle with Assets<Image> to access loaded image
}
```

### Using the Cache

```rust
use bevy::prelude::*;
use dgx_pixels_tui::bevy_app::assets::{AssetCache, CacheConfig};

fn setup(mut commands: Commands) {
    // Configure cache limits
    let config = CacheConfig {
        max_memory_mb: 512,  // 512MB max memory
        max_entries: 100,    // Max 100 cached images
    };

    commands.insert_resource(AssetCache::new(config));
}

fn use_cache(mut cache: ResMut<AssetCache>, asset_server: Res<AssetServer>) {
    let path = PathBuf::from("output/image.png");

    // Check cache first
    if let Some(handle) = cache.get(&path) {
        // Cache hit - use existing handle
        println!("Cache hit!");
    } else {
        // Cache miss - load and cache
        let handle = asset_server.load(&path);
        cache.insert(path, handle, 1024 * 1024); // Estimate 1MB
    }

    // View stats
    let stats = cache.stats();
    println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);
    println!("Memory: {:.1} MB", stats.memory_mb());
}
```

### Plugin Registration

```rust
use bevy::prelude::*;
use dgx_pixels_tui::bevy_app::assets::ImageAssetPlugin;

fn main() {
    App::new()
        .add_plugins(ImageAssetPlugin)
        .run();
}
```

## Supported Formats

- **PNG** - Lossless raster format (recommended for pixel art)
- **JPG/JPEG** - Lossy compressed format
- **WebP** - Modern format with good compression

Unsupported formats (BMP, GIF, TIFF, SVG, etc.) will be rejected with `ImageLoadError::UnsupportedFormat`.

## Memory Management

### Cache Eviction

The cache automatically evicts entries when limits are exceeded:

1. **Memory Limit**: When total cached memory exceeds `max_memory_mb`
2. **Entry Limit**: When number of entries exceeds `max_entries`

Eviction uses LRU policy - least recently accessed entries are removed first.

### Size Estimation

When inserting into the cache, provide an estimated size in bytes:

```rust
// Estimate based on image dimensions and format
let width = 1024;
let height = 1024;
let bytes_per_pixel = 4; // RGBA
let estimated_bytes = width * height * bytes_per_pixel;

cache.insert(path, handle, estimated_bytes);
```

For loaded images, you can get the actual size:

```rust
if let Some(image) = images.get(&handle) {
    let actual_bytes = image.data.len();
    cache.insert(path, handle, actual_bytes);
}
```

## Monitoring

### Cache Statistics

```rust
let stats = cache.stats();

println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);
println!("Hits: {}, Misses: {}", stats.hits, stats.misses);
println!("Evictions: {}", stats.evictions);
println!("Entries: {}", stats.current_entries);
println!("Memory: {:.1} MB", stats.memory_mb());
```

### Load State Tracking

The `ImageAssetPlugin` automatically tracks loading state and logs errors:

```
[DEBUG] Image loaded successfully: /output/sprite.png
[ERROR] Failed to load image: /output/missing.png
```

## Testing

All tests are located in:
- `src/bevy_app/assets/image_loader.rs` - Unit tests for image loading
- `src/bevy_app/assets/cache.rs` - Unit tests for cache management
- `tests/assets_integration_test.rs` - Integration tests

Run tests:
```bash
cargo test --features bevy_migration_foundation --lib assets
cargo test --features bevy_migration_foundation --test assets_integration_test
```

## Error Handling

### ImageLoadError Variants

- `NotFound(PathBuf)` - Image file does not exist
- `UnsupportedFormat(String)` - File extension not supported
- `InvalidPath(String)` - Path is not a valid file
- `CorruptedImage(String)` - Image data is corrupted (future use)

### Validation

Use `validate_image_path()` before loading:

```rust
use dgx_pixels_tui::bevy_app::assets::validate_image_path;

match validate_image_path("/output/sprite.png") {
    Ok(path) => {
        let handle = asset_server.load(path);
    }
    Err(e) => {
        error!("Invalid image path: {}", e);
    }
}
```

## Performance Characteristics

- **Load Time**: Async, non-blocking (handled by Bevy's asset system)
- **Cache Lookup**: O(1) average case (HashMap)
- **Cache Insertion**: O(1) amortized
- **Eviction**: O(n) worst case, where n = number of entries to evict

## Future Enhancements (Beyond WS-06A)

1. **Smart Size Estimation**: Calculate actual image size after loading
2. **Preloading**: Predictive loading based on usage patterns
3. **Compression**: In-memory compression for large images
4. **Disk Cache**: Persistent cache across restarts
5. **Hot Reloading**: Watch filesystem for image changes

## Integration with Preview System

The asset system is designed to replace Sixel previews (see WS-06 full spec).
Future work will integrate this with:
- `rust/src/bevy_app/systems/preview/` - Preview loading and rendering systems
- `rust/src/bevy_app/components/preview_image.rs` - Preview components with asset handles

## References

- **WS-06 Spec**: `docs/rfds/0003-bevy-ratatui-migration.md` lines 543-644
- **Bevy Asset System**: https://bevyengine.org/learn/book/asset-loading/
- **LRU Cache**: https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_recently_used_(LRU)
