# Foundation Orchestrator

**Timeline**: Weeks 1-2
**Workstreams**: 4 (WS-01 through WS-04)
**Risk Level**: High (architectural foundation)
**Dependencies**: None (first layer)

## Mission

Establish the Bevy ECS runtime and migrate core architecture from imperative ratatui to declarative Bevy systems. This orchestrator lays the foundation that all other workstreams build upon.

## Workstreams

### WS-01: Bevy Runtime Setup
- **Duration**: 3-4 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws01-bevy-runtime`
- **Spec**: [../workstreams/ws01-bevy-runtime/README.md](../workstreams/ws01-bevy-runtime/README.md)

### WS-02: ECS State Migration
- **Duration**: 4-5 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws02-ecs-state`
- **Spec**: [../workstreams/ws02-ecs-state/README.md](../workstreams/ws02-ecs-state/README.md)
- **Depends on**: WS-01

### WS-03: Input System
- **Duration**: 3-4 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws03-input-system`
- **Spec**: [../workstreams/ws03-input-system/README.md](../workstreams/ws03-input-system/README.md)
- **Depends on**: WS-02

### WS-04: Rendering Pipeline
- **Duration**: 4-5 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws04-rendering-pipeline`
- **Spec**: [../workstreams/ws04-rendering-pipeline/README.md](../workstreams/ws04-rendering-pipeline/README.md)
- **Depends on**: WS-02

## Execution Strategy

### Sequential Path
```
WS-01 (Bevy Runtime)
  ↓
WS-02 (ECS State) ← Foundation
  ↓ ↘
WS-03 (Input)  WS-04 (Rendering) ← Can run in parallel
```

**Week 1**:
- Days 1-3: WS-01 (Bevy Runtime Setup)
- Days 4-6: WS-02 (ECS State Migration) - START
- Day 7: WS-02 completion + Review

**Week 2**:
- Days 1-4: WS-03 + WS-04 (Parallel execution)
- Days 5-6: Testing, integration, demo prep
- Day 7: Milestone 1 Demo

### Parallel Opportunities

**After WS-02 completes**, run in parallel:
- **Track A**: WS-03 (Input System) - Touches `bevy_app/systems/input/*`
- **Track B**: WS-04 (Rendering) - Touches `bevy_app/systems/render/*`

No file overlap, safe to execute simultaneously.

## File Ownership Map

| Files/Directories | Primary Owner | Shared? |
|-------------------|---------------|---------|
| `rust/Cargo.toml` | WS-01 | ⚠️  Yes - coordinate all dependency additions |
| `rust/src/main.rs` | WS-01 | ⚠️  Yes - WS-02 adds dual-mode entry |
| `rust/src/bevy_app/mod.rs` | WS-01 | ✅ Framework only |
| `rust/src/bevy_app/plugins.rs` | WS-01 | ⚠️  Yes - all workstreams register systems here |
| `rust/src/bevy_app/resources/*` | WS-02 | ✅ Exclusive |
| `rust/src/bevy_app/components/*` | WS-02 | ✅ Exclusive |
| `rust/src/bevy_app/systems/state_init.rs` | WS-02 | ✅ Exclusive |
| `rust/src/bevy_app/systems/input/*` | WS-03 | ✅ Exclusive |
| `rust/src/bevy_app/systems/render/*` | WS-04 | ✅ Exclusive |
| `rust/src/app.rs` | WS-02 | ⚠️  Locked during migration |
| `rust/src/ui/*` | None | 📖 Reference only (don't modify) |

### Conflict Prevention Rules

1. **Cargo.toml Changes**:
   - All workstreams add dependencies with PR description listing additions
   - Meta-orchestrator reviews for conflicts before merge
   - Use feature flags for conditional dependencies

2. **plugins.rs Modifications**:
   - Each workstream adds its systems in dedicated section:
   ```rust
   // WS-01: Core Bevy plugins
   .add_plugins(RatatuiPlugins::default())

   // WS-02: State initialization
   .add_systems(Startup, init_resources)

   // WS-03: Input systems
   .add_systems(PreUpdate, (handle_keyboard, handle_resize))

   // WS-04: Rendering systems
   .add_systems(PostUpdate, render_dispatch)
   ```

3. **main.rs Entry Point**:
   - WS-01 creates feature-gated entry
   - WS-02 completes dual-mode switching
   - No further modifications by WS-03, WS-04

## Integration Testing

Each workstream must pass integration test before merge:

### WS-01 Integration Test
```bash
# Old mode still works
cargo test
cargo run

# New mode launches
cargo test --features bevy_migration_foundation
cargo run --features bevy_migration_foundation

# Verify 60 FPS
cargo run --features bevy_migration_foundation --release
# Should show ~16ms frame time in logs
```

### WS-02 Integration Test
```rust
#[test]
fn test_state_migration() {
    // Create old App
    let app = App::new();

    // Convert to Bevy resources
    let bevy_app = app.into_bevy_app();

    // Verify all resources exist
    assert!(bevy_app.world.contains_resource::<CurrentScreen>());
    assert!(bevy_app.world.contains_resource::<InputBuffer>());
    assert!(bevy_app.world.contains_resource::<GalleryState>());
    // ... all resources
}
```

### WS-03 Integration Test
```rust
#[test]
fn test_input_handling() {
    let mut app = create_test_bevy_app();

    // Simulate Tab key press
    app.world.send_message(KeyMessage {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::empty(),
    });

    app.update(); // Run one frame

    // Verify screen changed
    let screen = app.world.resource::<CurrentScreen>();
    assert_eq!(screen.0, Screen::Comparison);
}
```

### WS-04 Integration Test
```rust
#[test]
fn test_rendering_pipeline() {
    let mut app = create_test_bevy_app();

    // Run one frame
    app.update();

    // Verify render system executed
    let stats = app.world.resource::<RenderStats>();
    assert_eq!(stats.frames_rendered, 1);
    assert!(stats.last_frame_time < Duration::from_millis(16));
}
```

## Success Criteria

### Milestone 1: Foundation Complete

**Technical**:
- ✅ Bevy app runs at 60 FPS
- ✅ All app state in ECS resources/components
- ✅ Input handling message-based
- ✅ Rendering uses `RatatuiContext::draw()`
- ✅ Old ratatui mode functional (fallback)

**Quality**:
- ✅ Test coverage >75%
- ✅ All CI checks passing
- ✅ No memory leaks (heaptrack verified)
- ✅ Frame time <16ms (benchmarked)

**Process**:
- ✅ All 4 workstreams merged
- ✅ Documentation updated
- ✅ Demo prepared

### Demo Checklist

**Prepare for Week 2 milestone demo**:

1. **Setup Demo**:
   - Clean repository state
   - Both modes runnable
   - Screen recording ready

2. **Demo Script**:
   ```bash
   # Show old mode (baseline)
   cargo run
   # Navigate screens, show responsiveness

   # Show new Bevy mode
   cargo run --features bevy_migration_foundation
   # Same navigation, show ECS in action

   # Show performance
   cargo run --features bevy_migration_foundation --release
   # Display frame time stats
   ```

3. **Talking Points**:
   - Bevy runtime operational ✅
   - ECS state management ✅
   - Message-based input ✅
   - Foundation for GPU rendering ✅
   - Zero user-visible changes (perfect migration)

## Risk Management

### High-Risk Items

1. **Performance Regression**
   - **Risk**: Bevy overhead slows rendering below 60 FPS
   - **Detection**: Benchmark every PR against baseline
   - **Mitigation**:
     - Profile with `cargo flamegraph`
     - Optimize hot paths (caching, batching)
     - Use `MinimalPlugins` (not `DefaultPlugins`)

2. **State Migration Bugs**
   - **Risk**: ECS resources miss fields from old `App` struct
   - **Detection**: Comprehensive conversion tests
   - **Mitigation**:
     - Checklist of all `App` fields
     - Manual verification of conversion
     - Property tests for round-trip conversion

3. **Feature Flag Complexity**
   - **Risk**: Dual-mode codebase becomes unmaintainable
   - **Detection**: Code review for excessive `#[cfg]`
   - **Mitigation**:
     - Centralize feature-gated code in `main.rs`
     - Abstract differences behind traits
     - Plan to remove old code after 2 releases

### Rollback Plan

**If Foundation fails after merge**:
1. Disable feature flag: `bevy_migration_foundation = []` (empty by default)
2. Revert to old ratatui path (always functional)
3. Fix issues in hotfix branch
4. Re-enable feature flag in next release

## Coordination with Other Orchestrators

### Handoff to Core Systems Orchestrator

**After WS-04 completes**:
- ✅ Bevy runtime stable
- ✅ ECS resources defined
- ✅ Input systems operational
- ✅ Rendering pipeline works

**Core Systems can now**:
- Integrate ZeroMQ with Bevy async runtime (WS-05)
- Implement Bevy image assets (WS-06)
- Port theme to resource (WS-07)
- Add custom events (WS-08)

**Communication**:
- Tag Core Systems orchestrator on WS-04 PR
- Document ECS resource APIs
- Provide example systems for reference

### Parallel with Core Systems (Week 2-3 overlap)

**Safe to start during Week 2**:
- WS-05 (ZeroMQ) - Depends on WS-02 (state), not WS-03/WS-04
- WS-07 (Theme) - Isolated to styling, no conflicts
- WS-08 (Event Bus) - Depends on WS-02, can overlap with WS-03/WS-04

**Must wait for Foundation complete**:
- WS-06 (Image Assets) - Needs WS-04 (rendering pipeline) done

## Daily Standup Format

**Each workstream owner reports** (async, GitHub Discussions):

```markdown
## WS-XX: [Workstream Name] - Day N

### Progress
- Completed: [specific accomplishments]
- In Progress: [current task]
- Next: [upcoming tasks]

### Blockers
- [Any blockers or dependencies waiting]

### Metrics
- Tests: X passing, Y total
- Frame Time: Xms (target <16ms)
- Coverage: X%

### Questions
- [Any questions for orchestrator or other workstreams]
```

**Foundation Orchestrator synthesizes** into weekly report.

## Tools & Infrastructure

### Required Tools
- `cargo flamegraph` - Performance profiling
- `heaptrack` - Memory leak detection
- `cargo tarpaulin` - Test coverage
- `cargo bench` - Benchmarking

### CI Pipeline
```yaml
# .github/workflows/foundation.yml
name: Foundation Tests

on:
  pull_request:
    branches:
      - tui-modernization/foundation

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Test old mode
        run: cargo test
      - name: Test new mode
        run: cargo test --features bevy_migration_foundation
      - name: Benchmark
        run: cargo bench --features bevy_migration_foundation
      - name: Coverage
        run: cargo tarpaulin --features bevy_migration_foundation
```

## Documentation Requirements

Each workstream must update:
1. **RFD 0003**: Mark workstream as complete
2. **CHANGELOG.md**: Document changes
3. **API Docs**: Rustdoc for public APIs
4. **Architecture Diagram**: Update `docs/architecture.md`

## Conclusion

The Foundation Orchestrator is the **most critical** phase of the migration. All subsequent work depends on a solid Bevy ECS foundation. Success requires:

- **Rigorous testing**: Every workstream fully tested
- **Performance focus**: Maintain 60 FPS throughout
- **Clear communication**: Daily updates, weekly demos
- **Risk management**: Rollback plan always available

**Next Steps**:
1. Assign workstream owners (agents or humans)
2. Create feature branches
3. Kickoff WS-01 (Bevy Runtime Setup)
4. Daily standups via GitHub Discussions

---

**Status**: ✅ **Ready for Execution**
**Owner**: Foundation Orchestrator
**Last Updated**: 2025-11-14
