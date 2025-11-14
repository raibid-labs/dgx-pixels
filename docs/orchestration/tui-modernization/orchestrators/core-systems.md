# Core Systems Orchestrator

**Timeline**: Weeks 2-3
**Workstreams**: 4 (WS-05 through WS-08)
**Risk Level**: Medium (system integration)
**Dependencies**: Foundation Orchestrator (WS-01 through WS-04)

## Mission

Migrate critical subsystems (ZeroMQ, image assets, theme, event bus) from imperative to Bevy ECS patterns. This orchestrator builds on the Foundation layer to create a complete Bevy-native application architecture.

## Workstreams

### WS-05: ZeroMQ Integration
- **Duration**: 3-4 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws05-zeromq-integration`
- **Spec**: [../workstreams/ws05-zeromq-integration/README.md](../workstreams/ws05-zeromq-integration/README.md)
- **Depends on**: WS-02 (ECS State)

### WS-06: Image Asset System
- **Duration**: 4-5 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws06-image-assets`
- **Spec**: [../workstreams/ws06-image-assets/README.md](../workstreams/ws06-image-assets/README.md)
- **Depends on**: WS-02 (ECS State), WS-04 (Rendering Pipeline)

### WS-07: Theme & Styling
- **Duration**: 2-3 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws07-theme-styling`
- **Spec**: [../workstreams/ws07-theme-styling/README.md](../workstreams/ws07-theme-styling/README.md)
- **Depends on**: WS-04 (Rendering Pipeline)

### WS-08: Event Bus
- **Duration**: 2-3 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws08-event-bus`
- **Spec**: [../workstreams/ws08-event-bus/README.md](../workstreams/ws08-event-bus/README.md)
- **Depends on**: WS-02 (ECS State), WS-03 (Input System)

## Execution Strategy

### Parallel Execution Tracks

```
Week 2: (After Foundation WS-04 completes)
┌─────────────────────────────────────┐
│ Track A: WS-05 (ZeroMQ)            │ 3-4 days
│ Track B: WS-07 (Theme)             │ 2-3 days
│ Track C: WS-08 (Event Bus)         │ 2-3 days
└─────────────────────────────────────┘
                ↓
Week 3:
┌─────────────────────────────────────┐
│ Track A: WS-06 (Image Assets)      │ 4-5 days (High Priority)
└─────────────────────────────────────┘
```

**Week 2 Parallel Tracks**:
- **Track A**: WS-05 (ZeroMQ) - Depends on WS-02, no rendering conflicts
- **Track B**: WS-07 (Theme) - Isolated to styling resources, no conflicts
- **Track C**: WS-08 (Event Bus) - Depends on WS-02 + WS-03, no rendering conflicts

**Week 3 Sequential**:
- **WS-06 (Image Assets)**: High-risk workstream, runs solo to ensure stability
  - Replaces entire Sixel preview system
  - Touches rendering pipeline (WS-04 dependency)
  - Requires focused testing and performance validation

### Parallelization Safety

**Safe to Run Simultaneously** (Week 2):
```rust
// WS-05: Touches only ZMQ-specific files
rust/src/bevy_app/systems/zmq/*
rust/src/zmq_client.rs

// WS-07: Touches only theme resources
rust/src/bevy_app/resources/theme.rs

// WS-08: Touches only event definitions
rust/src/bevy_app/events/*
```

**No file overlap**, all workstreams can execute in parallel.

## File Ownership Map

| Files/Directories | Primary Owner | Shared? | Notes |
|-------------------|---------------|---------|-------|
| `rust/src/bevy_app/systems/zmq/*` | WS-05 | ✅ Exclusive | ZeroMQ polling/handlers |
| `rust/src/zmq_client.rs` | WS-05 | ⚠️  Modified (thread-safety) | Make Arc/Mutex |
| `rust/src/bevy_app/assets/*` | WS-06 | ✅ Exclusive | New image asset system |
| `rust/src/bevy_app/systems/preview/*` | WS-06 | ✅ Exclusive | Preview loading/rendering |
| `rust/src/bevy_app/components/preview_image.rs` | WS-06 | ✅ Exclusive | Preview component |
| `rust/src/sixel/*` | WS-06 | 🗑️  To be REMOVED | After migration complete |
| `rust/src/bevy_app/resources/theme.rs` | WS-07 | ✅ Exclusive | Theme resource |
| `rust/src/ui/theme.rs` | WS-07 | 📖 Reference | Old theme (keep for fallback) |
| `rust/src/bevy_app/events/*` | WS-08 | ✅ Exclusive | Custom event types |
| `rust/src/bevy_app/plugins.rs` | ALL | ⚠️  Shared | Each WS adds systems in dedicated section |

### Conflict Prevention Rules

1. **plugins.rs System Registration**:
   - Each workstream adds systems in commented sections:
   ```rust
   // WS-05: ZeroMQ systems
   .add_systems(PreUpdate, (poll_zmq_responses, handle_zmq_updates))

   // WS-06: Image asset systems
   .add_systems(Update, load_preview_images)
   .add_systems(PostUpdate, render_preview_images.after(render_dispatch))

   // WS-07: Theme initialization
   .insert_resource(AppTheme::default())

   // WS-08: Event registration
   .add_event::<NavigateToScreen>()
   .add_event::<SubmitGenerationJob>()
   .add_systems(Update, (handle_navigation_events, handle_generation_events))
   ```

2. **Cargo.toml Dependency Coordination**:
   - WS-05: Adds `zeromq` async compatibility deps
   - WS-06: Adds `image` crate, Bevy image loading features
   - WS-07: No new dependencies (uses existing ratatui styles)
   - WS-08: No new dependencies (uses Bevy events)

3. **ZeroMQ Client Thread-Safety** (WS-05):
   - Must coordinate with existing `zmq_client.rs` usage
   - Wrap in `Arc<Mutex<>>` for Bevy resource sharing
   - Ensure no breaking changes to existing API

## Integration Testing

### WS-05 Integration Test
```rust
#[tokio::test]
async fn test_zmq_bevy_integration() {
    let mut app = create_test_bevy_app();

    // Mock ZMQ server
    let mock_server = spawn_mock_zmq_server().await;

    // Submit job via ZMQ
    app.world.resource::<ZmqClientResource>()
        .0.submit_job(JobRequest {
            prompt: "test".into(),
            ..default()
        })
        .await
        .unwrap();

    // Simulate response
    mock_server.send_response(Response::JobAccepted {
        job_id: "test-123".into(),
    });

    // Run polling system
    app.update();

    // Verify job entity created
    let jobs: Vec<&Job> = app.world.query::<&Job>()
        .iter(&app.world)
        .collect();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "test-123");
}
```

### WS-06 Integration Test
```rust
#[test]
fn test_image_asset_loading() {
    let mut app = create_test_bevy_app();

    // Create test image
    let test_image_path = create_test_image();

    // Spawn job with completed status
    app.world.spawn(Job {
        id: "test-job".into(),
        status: JobStatus::Complete {
            image_path: test_image_path.clone(),
            duration_s: 3.5,
        },
        ..default()
    });

    // Run asset loading system
    app.update();

    // Verify PreviewImage component added
    let preview_query = app.world.query::<&PreviewImage>();
    assert_eq!(preview_query.iter(&app.world).count(), 1);

    // Verify asset loaded
    let assets = app.world.resource::<Assets<Image>>();
    assert!(assets.len() > 0);
}
```

### WS-07 Integration Test
```rust
#[test]
fn test_theme_resource() {
    let mut app = create_test_bevy_app();

    // Verify theme resource exists
    assert!(app.world.contains_resource::<AppTheme>());

    let theme = app.world.resource::<AppTheme>();

    // Verify color definitions
    assert_eq!(theme.colors.text, Color::Rgb(255, 255, 255));
    assert_eq!(theme.colors.highlight, Color::Rgb(0, 255, 255));

    // Verify style methods
    let text_style = theme.text();
    assert_eq!(text_style.fg, Some(Color::Rgb(255, 255, 255)));
}
```

### WS-08 Integration Test
```rust
#[test]
fn test_event_bus() {
    let mut app = create_test_bevy_app();

    // Send navigation event
    app.world.send_event(NavigateToScreen(Screen::Gallery));

    // Run event handler systems
    app.update();

    // Verify screen changed
    let screen = app.world.resource::<CurrentScreen>();
    assert_eq!(screen.0, Screen::Gallery);
}

#[test]
fn test_generation_event() {
    let mut app = create_test_bevy_app();

    // Send generation request event
    app.world.send_event(SubmitGenerationJob {
        prompt: "pixel art wizard".into(),
        params: GenerationParams::default(),
    });

    // Run event handler
    app.update();

    // Verify ZMQ client received request
    let zmq_resource = app.world.resource::<ZmqClientResource>();
    assert_eq!(zmq_resource.0.pending_requests(), 1);
}
```

## Success Criteria

### Milestone 2: Core Systems Complete

**Technical**:
- ✅ ZeroMQ integrated with Bevy async runtime
- ✅ Job state updates flow through ECS
- ✅ Image assets replace Sixel system
- ✅ GPU-accelerated image rendering working
- ✅ Theme resource used consistently
- ✅ Event-driven architecture operational

**Performance**:
- ✅ ZMQ polling latency <10ms
- ✅ Image preview loading <1 second
- ✅ No memory leaks (verified with heaptrack)
- ✅ Frame rate maintained at 60 FPS
- ✅ Image asset caching prevents OOM

**Quality**:
- ✅ Test coverage >75% for all workstreams
- ✅ All CI checks passing
- ✅ Benchmarks meet targets
- ✅ Documentation complete

**Process**:
- ✅ All 4 workstreams merged
- ✅ Old Sixel code removed (WS-06 cleanup)
- ✅ Demo prepared for stakeholder review

### Demo Checklist

**Prepare for Week 3 milestone demo**:

1. **Setup Demo**:
   - Test image generation end-to-end
   - Multiple images in gallery
   - Both modes (old Sixel vs new Bevy assets) ready for comparison

2. **Demo Script**:
   ```bash
   # Show ZeroMQ integration
   cargo run --features bevy_migration_core
   # Submit generation job
   # Show real-time status updates in Queue screen

   # Show image asset system
   # Navigate to Gallery screen
   # Display GPU-rendered previews

   # Show theme consistency
   # Navigate all screens, verify colors match

   # Show event-driven architecture
   # Use keyboard shortcuts, show event flow in logs
   ```

3. **Talking Points**:
   - ZeroMQ async integration ✅
   - GPU image rendering (replacing Sixel) ✅
   - Consistent theming via resources ✅
   - Event-driven cross-system communication ✅
   - Foundation for MCP integration ready

## Risk Management

### High-Risk Items

1. **WS-06: Image Asset System Migration**
   - **Risk**: Broken previews, performance degradation, memory leaks
   - **Detection**:
     - Benchmark preview rendering vs Sixel baseline
     - Memory profiling with heaptrack
     - Visual comparison tests
   - **Mitigation**:
     - Feature-flagged rollout (Sixel fallback available)
     - Extensive testing across image formats (PNG, JPG, WebP)
     - Cache size limits + LRU eviction
     - Keep Sixel code for 1 release cycle

2. **WS-05: ZeroMQ Thread-Safety**
   - **Risk**: Deadlocks, race conditions with Arc/Mutex wrapping
   - **Detection**:
     - Integration tests with concurrent requests
     - Stress testing (100+ rapid job submissions)
   - **Mitigation**:
     - Use `tokio::sync::Mutex` (async-aware)
     - Minimize lock contention (short critical sections)
     - Deadlock detection in CI tests

3. **Performance Regression**
   - **Risk**: Bevy overhead + image assets slow rendering
   - **Detection**: Continuous benchmarking vs baseline
   - **Mitigation**:
     - Profile with `cargo flamegraph`
     - Lazy image loading (load on-demand)
     - Asset unloading for off-screen images
     - Use Bevy's built-in asset caching

### Rollback Plan

**If Core Systems fail after merge**:
1. **WS-05 rollback**: Revert to old ZMQ client (non-Bevy)
2. **WS-06 rollback**: Re-enable Sixel feature flag, disable asset system
3. **WS-07 rollback**: Use old `ui/theme.rs` module
4. **WS-08 rollback**: Remove events, use direct resource mutation

**Feature flags per workstream**:
```toml
[features]
bevy_migration_core = ["bevy_migration_foundation"]
zmq_bevy = ["bevy_migration_core"]
image_assets = ["bevy_migration_core"]
theme_resource = ["bevy_migration_core"]
event_bus = ["bevy_migration_core"]
```

## Coordination with Other Orchestrators

### Handoff from Foundation Orchestrator

**Prerequisites verified**:
- ✅ WS-04 (Rendering Pipeline) merged and stable
- ✅ ECS resources accessible (WS-02)
- ✅ Input systems functional (WS-03)
- ✅ 60 FPS baseline established

**Foundation provides**:
- Bevy app structure
- Resource access patterns
- Rendering dispatch system
- Input message handling

### Handoff to Screen Migration Orchestrator

**After Core Systems completes**:
- ✅ All subsystems Bevy-native
- ✅ Image preview system operational
- ✅ Theme resource available
- ✅ Event bus ready for screen-specific events

**Screen Migration can now**:
- Migrate all 8 screens in parallel (WS-09 through WS-16)
- Use Bevy image assets for previews
- Apply consistent theming
- Emit screen-specific events

**Communication**:
- Tag Screen Migration orchestrator on WS-06 (Image Assets) PR
- Document preview rendering API
- Provide example screen using all Core Systems features

### Parallel with Screen Migration (Week 3 overlap)

**Safe to start WS-06 while screens migrate**:
- Image asset system is independent of screen implementation
- Screens can continue using placeholder rendering
- Migrate preview rendering after WS-06 completes

**Coordination needed**:
- Screen workstreams must wait for WS-06 before adding image rendering
- Use feature flag to gate preview rendering until WS-06 merged

## Daily Standup Format

**Each workstream owner reports** (async, GitHub Discussions):

```markdown
## WS-XX: [Workstream Name] - Day N

### Progress
- Completed: [specific tasks done today]
- In Progress: [current focus]
- Next: [planned for tomorrow]

### Metrics
- Tests: X passing / Y total
- Coverage: X%
- Performance: [benchmark results]
- Memory: [heaptrack results]

### Blockers
- [Dependencies waiting on other workstreams]
- [Technical issues requiring help]

### Integration Notes
- [Changes affecting other workstreams]
- [API additions/modifications]
```

**Core Systems Orchestrator synthesizes** into weekly report.

## Tools & Infrastructure

### Required Tools
- `cargo flamegraph` - Performance profiling
- `heaptrack` - Memory leak detection
- `cargo bench` - Benchmarking
- `cargo tarpaulin` - Test coverage
- `tokio-console` - Async task debugging (WS-05)
- Image comparison tools (WS-06)

### CI Pipeline
```yaml
# .github/workflows/core-systems.yml
name: Core Systems Tests

on:
  pull_request:
    branches:
      - tui-modernization/ws05-*
      - tui-modernization/ws06-*
      - tui-modernization/ws07-*
      - tui-modernization/ws08-*

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Test Core Systems
        run: |
          cargo test --features bevy_migration_core

      - name: Benchmark ZeroMQ
        if: contains(github.head_ref, 'ws05')
        run: cargo bench --bench zmq_integration

      - name: Benchmark Image Assets
        if: contains(github.head_ref, 'ws06')
        run: cargo bench --bench image_loading

      - name: Memory Leak Check
        run: |
          heaptrack target/debug/dgx-pixels --features bevy_migration_core
          heaptrack_print heaptrack.dgx-pixels.*.gz | grep -i leak

      - name: Coverage
        run: |
          cargo tarpaulin --features bevy_migration_core --out Xml
          bash <(curl -s https://codecov.io/bash)
```

### WS-06 Specific: Image Comparison Tests
```bash
# Visual regression testing for image previews
cargo test --features bevy_migration_core test_preview_rendering
# Generates snapshots: tests/snapshots/preview_*.png
# Manual review required for visual changes
```

## Documentation Requirements

Each workstream must update:
1. **RFD 0003**: Mark workstream complete, document decisions
2. **CHANGELOG.md**: Document API changes
3. **API Docs**: Rustdoc for public APIs (resources, components, systems)
4. **Architecture Diagram**: Update `docs/architecture.md` with new subsystems
5. **Migration Guide**: Document Sixel → Image Assets migration (WS-06)

### WS-06 Specific Documentation
```markdown
# Migration Guide: Sixel to Bevy Image Assets

## Breaking Changes
- `sixel::preview()` removed
- `PreviewManager` removed
- Replaced with Bevy `AssetServer` + `PreviewImage` component

## Migration Steps
1. Remove Sixel dependencies from Cargo.toml
2. Use `AssetServer.load(path)` for image loading
3. Add `PreviewImage` component to entities
4. Render using `bevy_ratatui` image widgets

## Rollback
If issues arise, re-enable Sixel:
```toml
[features]
default = ["sixel_fallback"]
sixel_fallback = []
```
```

## Conclusion

The Core Systems Orchestrator completes the **subsystem migration** to Bevy ECS patterns. Success requires:

- **Careful sequencing**: WS-06 (Image Assets) runs solo due to high risk
- **Performance focus**: No regression in rendering or memory usage
- **Thorough testing**: Integration tests for all subsystems
- **Clear rollback**: Feature flags for each workstream

**Critical Success Factors**:
1. **WS-06 (Image Assets)** must succeed - it gates Screen Migration
2. **ZeroMQ integration** must be thread-safe and performant
3. **Event bus** must support screen-specific event patterns
4. **Theme resource** must provide consistent styling

**Next Steps**:
1. Verify Foundation complete (WS-01 through WS-04 merged)
2. Assign Core Systems workstream owners
3. Create feature branches
4. Kickoff WS-05, WS-07, WS-08 in parallel (Week 2)
5. Kickoff WS-06 after testing complete (Week 3)

---

**Status**: ✅ **Ready for Execution**
**Owner**: Core Systems Orchestrator
**Last Updated**: 2025-11-14
