# WS-06: Image Asset System

**Orchestrator**: Core Systems
**Duration**: 4-5 days
**Risk**: High (replaces Sixel)
**Dependencies**: WS-02, WS-04

## Summary

Replace Sixel preview system with Bevy image assets. Implement GPU-accelerated image rendering using Bevy's asset system. This workstream removes the entire `sixel/` module.

## Files Created
```
rust/src/bevy_app/assets/
├── mod.rs
├── image_loader.rs      # Async image loading
└── cache.rs             # Asset cache management

rust/src/bevy_app/systems/preview/
├── mod.rs
├── loader.rs            # Preview loading system
└── renderer.rs          # Image rendering system
```

## Files Removed (after migration complete)
```
rust/src/sixel/*         # Entire Sixel module
```

## Key Implementation

```rust
#[derive(Component)]
struct PreviewImage {
    asset_handle: Handle<Image>,
    path: PathBuf,
}

fn load_preview_images(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    job_query: Query<(Entity, &Job), Changed<Job>>,
) {
    for (entity, job) in job_query.iter() {
        if let JobStatus::Complete { image_path, .. } = &job.status {
            let handle = asset_server.load(image_path);
            commands.entity(entity).insert(PreviewImage {
                asset_handle: handle,
                path: image_path.clone(),
            });
        }
    }
}
```

## Acceptance Criteria
- [ ] Images load from filesystem as Bevy assets
- [ ] GPU-accelerated rendering in terminal
- [ ] Fallback to Unicode/ASCII for unsupported terminals
- [ ] Preview updates <1 second after generation
- [ ] Memory usage reasonable (cache eviction works)

**Branch**: `tui-modernization/ws06-image-assets`
