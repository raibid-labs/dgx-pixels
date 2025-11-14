# Screen Migration Orchestrator

**Timeline**: Weeks 3-5
**Workstreams**: 8 (WS-09 through WS-16)
**Risk Level**: Low (highly parallel, isolated)
**Dependencies**: Foundation + Core Systems (WS-01 through WS-08)

## Mission

Migrate all 8 UI screens from old ratatui rendering to Bevy ECS systems. Each screen is isolated to its own workstream, enabling **maximum parallelization** with zero merge conflicts.

## Workstreams

### WS-09: Generation Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws09-generation-screen`
- **Spec**: [../workstreams/ws09-generation-screen/README.md](../workstreams/ws09-generation-screen/README.md)
- **File**: `screens/generation.rs`

### WS-10: Gallery Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws10-gallery-screen`
- **Spec**: [../workstreams/ws10-gallery-screen/README.md](../workstreams/ws10-gallery-screen/README.md)
- **File**: `screens/gallery.rs`

### WS-11: Comparison Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws11-comparison-screen`
- **Spec**: [../workstreams/ws11-comparison-screen/README.md](../workstreams/ws11-comparison-screen/README.md)
- **File**: `screens/comparison.rs`

### WS-12: Models Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws12-models-screen`
- **Spec**: [../workstreams/ws12-models-screen/README.md](../workstreams/ws12-models-screen/README.md)
- **File**: `screens/models.rs`

### WS-13: Queue Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws13-queue-screen`
- **Spec**: [../workstreams/ws13-queue-screen/README.md](../workstreams/ws13-queue-screen/README.md)
- **File**: `screens/queue.rs`

### WS-14: Monitor Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws14-monitor-screen`
- **Spec**: [../workstreams/ws14-monitor-screen/README.md](../workstreams/ws14-monitor-screen/README.md)
- **File**: `screens/monitor.rs`

### WS-15: Settings Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws15-settings-screen`
- **Spec**: [../workstreams/ws15-settings-screen/README.md](../workstreams/ws15-settings-screen/README.md)
- **File**: `screens/settings.rs`

### WS-16: Help Screen
- **Duration**: 1-2 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws16-help-screen`
- **Spec**: [../workstreams/ws16-help-screen/README.md](../workstreams/ws16-help-screen/README.md)
- **File**: `screens/help.rs`

## Execution Strategy

### Massive Parallelization

**All 8 screens can migrate simultaneously** with **ZERO merge conflicts**:

```
Week 3-5: (After Core Systems WS-08 completes)
┌───────────────────────────────────────────────────────────┐
│ Track A: WS-09 (Generation) + WS-10 (Gallery)           │
│ Track B: WS-11 (Comparison) + WS-12 (Models)            │
│ Track C: WS-13 (Queue) + WS-14 (Monitor)                │
│ Track D: WS-15 (Settings) + WS-16 (Help)                │
└───────────────────────────────────────────────────────────┘
        All tracks run simultaneously (4 parallel pairs)
```

**Why zero conflicts?**
- Each screen has dedicated file: `rust/src/bevy_app/systems/render/screens/{name}.rs`
- Each screen has dedicated input handler: `rust/src/bevy_app/systems/input/screens/{name}.rs`
- Shared file (`plugins.rs`) has screen-specific sections (commented)
- No overlap in resources, components, or events

### Migration Pattern (Per Screen)

Every screen follows identical 4-step pattern:

```
Step 1: Create Bevy render system
  └── rust/src/bevy_app/systems/render/screens/generation.rs

Step 2: Create Bevy input handler
  └── rust/src/bevy_app/systems/input/screens/generation.rs

Step 3: Register systems in plugins.rs
  └── Add to dispatch + input routing

Step 4: Test + verify parity
  └── Visual comparison + interaction tests
```

### File Creation Template

**Each workstream creates 2 new files**:

```
rust/src/bevy_app/systems/
├── render/screens/{screen_name}.rs       # Rendering logic
└── input/screens/{screen_name}.rs        # Input handling
```

**And modifies 1 shared file** (in dedicated section):
```
rust/src/bevy_app/plugins.rs             # System registration
```

## File Ownership Matrix

**Zero overlap = Zero conflicts**

| Screen | Render File | Input File | Old File (Reference) |
|--------|-------------|------------|----------------------|
| WS-09: Generation | `render/screens/generation.rs` | `input/screens/generation.rs` | `ui/screens/generation.rs` |
| WS-10: Gallery | `render/screens/gallery.rs` | `input/screens/gallery.rs` | `ui/screens/gallery.rs` |
| WS-11: Comparison | `render/screens/comparison.rs` | `input/screens/comparison.rs` | `ui/screens/comparison.rs` |
| WS-12: Models | `render/screens/models.rs` | `input/screens/models.rs` | `ui/screens/models.rs` |
| WS-13: Queue | `render/screens/queue.rs` | `input/screens/queue.rs` | `ui/screens/queue.rs` |
| WS-14: Monitor | `render/screens/monitor.rs` | `input/screens/monitor.rs` | `ui/screens/monitor.rs` |
| WS-15: Settings | `render/screens/settings.rs` | `input/screens/settings.rs` | `ui/screens/settings.rs` |
| WS-16: Help | `render/screens/help.rs` | `input/screens/help.rs` | `ui/screens/help.rs` |

**Shared File Sections** (plugins.rs):
```rust
// WS-09: Generation Screen
.add_systems(PostUpdate, render::screens::generation::render_generation_screen)
.add_systems(PreUpdate, input::screens::generation::handle_generation_input)

// WS-10: Gallery Screen
.add_systems(PostUpdate, render::screens::gallery::render_gallery_screen)
.add_systems(PreUpdate, input::screens::gallery::handle_gallery_input)

// ... and so on for WS-11 through WS-16
```

Each section is isolated by screen name - no conflicts.

## Conflict Prevention Rules

### 1. plugins.rs Modification Protocol

**Before modifying plugins.rs**:
1. Pull latest main: `git pull origin main`
2. Add systems in alphabetical screen order
3. Use full module paths (no wildcard imports)
4. Comment with workstream ID

**Template for each screen**:
```rust
// WS-XX: [Screen Name] Screen
.add_systems(
    PostUpdate,
    render::screens::{screen_name}::render_{screen_name}_screen
        .after(render_dispatch)
)
.add_systems(
    PreUpdate,
    input::screens::{screen_name}::handle_{screen_name}_input
)
```

### 2. Merge Order

**Recommended merge sequence** (minimize conflicts):
1. WS-09 (Generation) - First screen, establishes pattern
2. WS-16 (Help) - Simplest screen, validates pattern
3. WS-10, WS-11, WS-12, WS-13, WS-14, WS-15 - Parallel (any order)

**Alternative: Merge as ready** (with conflict resolution):
- Each PR rebases on main before merge
- Conflicts only in plugins.rs (easy to resolve)
- CI enforces no conflicts in dedicated screen files

### 3. API Compatibility

**All screens use same APIs** (established by Core Systems):
- Resources: `CurrentScreen`, `InputBuffer`, `GalleryState`, etc.
- Components: `Job`, `PreviewImage`
- Events: `NavigateToScreen`, `SubmitGenerationJob`, etc.
- Theme: `AppTheme` resource

**No API changes during screen migration** - all screens conform to existing APIs.

## Integration Testing (Per Screen)

### Test Template

Every screen workstream runs identical test suite:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_{screen_name}_renders() {
        let mut app = create_test_bevy_app();
        app.world.resource_mut::<CurrentScreen>().0 = Screen::{ScreenName};

        // Run one frame
        app.update();

        // Verify render system executed
        let stats = app.world.resource::<RenderStats>();
        assert_eq!(stats.last_rendered_screen, Screen::{ScreenName});
        assert!(stats.last_frame_time < Duration::from_millis(16));
    }

    #[test]
    fn test_{screen_name}_input() {
        let mut app = create_test_bevy_app();
        app.world.resource_mut::<CurrentScreen>().0 = Screen::{ScreenName};

        // Send test input
        app.world.send_message(KeyMessage {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        });

        // Run input system
        app.update();

        // Verify state changed (screen-specific assertion)
        // Example: assert!(some_state_changed);
    }

    #[test]
    fn test_{screen_name}_parity() {
        // Visual regression test
        // Compare old ratatui rendering vs new Bevy rendering
        // Generate snapshot, compare against baseline
    }
}
```

### Visual Regression Testing

**Each screen must pass visual comparison**:

```bash
# Generate baseline (old ratatui)
cargo run > /dev/null
# Manual: Navigate to screen, screenshot terminal

# Generate new rendering (Bevy)
cargo run --features bevy_migration_screens
# Manual: Navigate to screen, screenshot terminal

# Compare
compare baseline.png current.png diff.png
# Verify diff.png shows no significant changes
```

**Automated visual tests** (optional):
- Use `insta` crate for snapshot testing
- Capture terminal output as text
- Compare text snapshots (character-level diff)

## Success Criteria

### Per-Screen Acceptance

**Each screen workstream merges when**:
- ✅ Visual parity verified (rendering matches old version)
- ✅ All interactions functional (input handling correct)
- ✅ Tests passing (unit + integration)
- ✅ Performance maintained (<16ms frame time)
- ✅ No regressions in other screens (CI validates)

### Milestone 3: All Screens Migrated

**Technical**:
- ✅ All 8 screens render via Bevy systems
- ✅ All input handlers using Bevy message readers
- ✅ Old `ui/screens/*` code deleted
- ✅ Rendering dispatch routes to all screens
- ✅ 60 FPS maintained across all screens

**Quality**:
- ✅ Test coverage >80% for screen logic
- ✅ All visual regression tests passing
- ✅ No memory leaks (heaptrack verified)
- ✅ Accessibility maintained (keyboard navigation)

**Process**:
- ✅ All 8 workstreams merged
- ✅ Old ratatui screen code removed
- ✅ Documentation updated (screen-specific APIs)
- ✅ Demo prepared for stakeholder review

### Demo Checklist

**Prepare for Week 5 milestone demo**:

1. **Setup Demo**:
   - Build with all screens migrated
   - Prepare test data (jobs, images, models)
   - Both modes ready (old vs new for comparison)

2. **Demo Script**:
   ```bash
   # Launch Bevy-migrated app
   cargo run --features bevy_migration_screens

   # Navigate all screens (show smooth transitions)
   Tab → Generation Screen
   Tab → Gallery Screen
   Tab → Comparison Screen
   Tab → Models Screen
   Tab → Queue Screen
   Tab → Monitor Screen
   Tab → Settings Screen
   Tab → Help Screen

   # Show interactions work
   # - Submit generation job (Generation screen)
   # - Select image (Gallery screen)
   # - Compare models (Comparison screen)
   # - View queue (Queue screen)
   # - Monitor GPU (Monitor screen)

   # Show performance
   # - 60 FPS across all screens
   # - Smooth image preview rendering
   ```

3. **Talking Points**:
   - All 8 screens Bevy-native ✅
   - Zero merge conflicts (isolation strategy worked)
   - Visual parity perfect (no user-visible changes)
   - Foundation for GPU-accelerated features
   - Ready for MCP integration

## Risk Management

### Low-Risk Assessment

**Why screen migration is low-risk**:
1. **Isolated files**: No cross-screen dependencies
2. **Established pattern**: First screen (WS-09) validates approach
3. **Visual verification**: Easy to spot regressions
4. **Rollback simple**: Keep old screen code until all migrated
5. **Parallel execution**: Failures isolated to single screen

### Potential Issues

1. **Visual Regression**
   - **Risk**: New rendering differs from old
   - **Detection**: Visual comparison tests
   - **Mitigation**: Pixel-perfect widget porting, reference screenshots

2. **Input Handling Bugs**
   - **Risk**: Keyboard shortcuts broken
   - **Detection**: Manual testing + automated input tests
   - **Mitigation**: Comprehensive input test coverage per screen

3. **Performance Variance**
   - **Risk**: Some screens render slower
   - **Detection**: Per-screen benchmarks
   - **Mitigation**: Profile slow screens, optimize hot paths

4. **Merge Conflicts in plugins.rs**
   - **Risk**: Simultaneous PRs conflict
   - **Detection**: CI merge checks
   - **Mitigation**: Rebase before merge, clear section comments

### Rollback Plan

**If a screen migration fails**:
1. Disable screen in dispatch: comment out system registration
2. Fallback to old `ui/screens/{name}.rs` rendering
3. Fix issues in hotfix branch
4. Re-enable Bevy rendering when fixed

**Feature flags per screen** (optional):
```toml
[features]
bevy_migration_screens = [
    "screen_generation",
    "screen_gallery",
    "screen_comparison",
    "screen_models",
    "screen_queue",
    "screen_monitor",
    "screen_settings",
    "screen_help",
]
```

## Coordination with Other Orchestrators

### Handoff from Core Systems Orchestrator

**Prerequisites verified**:
- ✅ WS-06 (Image Assets) merged and stable
- ✅ WS-07 (Theme) available as resource
- ✅ WS-08 (Event Bus) operational
- ✅ Rendering dispatch system ready

**Core Systems provides**:
- `RatatuiContext::draw()` API
- `AppTheme` resource for styling
- `PreviewImage` components for gallery
- Event types for interactions

### Handoff to Integration Orchestrator

**After all screens migrated**:
- ✅ All UI rendering Bevy-native
- ✅ Input handling event-driven
- ✅ Asset pipeline operational
- ✅ Theme applied consistently

**Integration can now**:
- Implement MCP server (WS-17)
- Add dual-mode rendering (WS-18)
- Enable hot-reloading
- Add native window support

**Communication**:
- Tag Integration orchestrator when WS-16 (last screen) merges
- Document screen rendering APIs
- Provide end-to-end testing guide

## Parallel Execution Recommendations

### 4 Parallel Tracks (Optimal)

**Assign workstreams to agents/developers**:

**Track A (Most Complex)**:
- WS-09: Generation Screen (most interactions)
- WS-11: Comparison Screen (complex layout)

**Track B (Medium Complexity)**:
- WS-10: Gallery Screen (image grid rendering)
- WS-12: Models Screen (table rendering)

**Track C (Medium Complexity)**:
- WS-13: Queue Screen (job list updates)
- WS-14: Monitor Screen (real-time stats)

**Track D (Simplest)**:
- WS-15: Settings Screen (simple forms)
- WS-16: Help Screen (static content)

**Execution Strategy**:
1. Start all 4 tracks simultaneously
2. Track D finishes first (days 1-2) → validates pattern
3. Tracks B and C finish next (days 2-3)
4. Track A finishes last (days 3-4)
5. Final testing and old code removal (days 4-5)

### 2 Parallel Tracks (Conservative)

**If limited resources**:

**Track A**:
- WS-09, WS-11, WS-13, WS-15 (4 screens sequentially)

**Track B**:
- WS-10, WS-12, WS-14, WS-16 (4 screens sequentially)

**Execution Strategy**:
1. Both tracks start simultaneously
2. Each track completes 1 screen every 1-2 days
3. Total time: 4-8 days (same as 4-track approach)

## Daily Standup Format

**Each screen workstream owner reports** (async, GitHub Discussions):

```markdown
## WS-XX: [Screen Name] Screen - Day N

### Progress
- Completed:
  - [ ] Render system created
  - [ ] Input handler created
  - [ ] Systems registered
  - [ ] Tests written
  - [ ] Visual parity verified
- In Progress: [current step]
- Next: [upcoming step]

### Visual Comparison
- Screenshot: [link to before/after comparison]
- Diff: [any notable differences]

### Blockers
- [None expected - screens isolated]

### Metrics
- Tests: X passing / Y total
- Frame Time: Xms (target <16ms)
- Visual Diff: X pixels changed
```

**Screen Migration Orchestrator synthesizes** into weekly report.

## Tools & Infrastructure

### Required Tools
- `insta` - Snapshot testing
- `cargo-nextest` - Fast test runner
- Terminal screenshot tool (e.g., `kitty +kitten icat`)
- Image comparison: `imagemagick compare`

### CI Pipeline
```yaml
# .github/workflows/screen-migration.yml
name: Screen Migration Tests

on:
  pull_request:
    branches:
      - tui-modernization/ws09-*
      - tui-modernization/ws10-*
      - tui-modernization/ws11-*
      - tui-modernization/ws12-*
      - tui-modernization/ws13-*
      - tui-modernization/ws14-*
      - tui-modernization/ws15-*
      - tui-modernization/ws16-*

jobs:
  test-screen:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run Screen Tests
        run: |
          cargo nextest run --features bevy_migration_screens

      - name: Benchmark Rendering
        run: |
          cargo bench --bench screen_rendering

      - name: Visual Regression
        run: |
          cargo test --features bevy_migration_screens test_visual_parity
          # Uploads diff.png as artifact if test fails

      - name: Check Coverage
        run: |
          cargo tarpaulin --features bevy_migration_screens --packages dgx-pixels
```

### Visual Regression Workflow

```bash
# Step 1: Generate baseline (run once, commit to repo)
./scripts/generate_screen_baselines.sh

# Step 2: Test new rendering
cargo test --features bevy_migration_screens test_visual_parity

# Step 3: Review diffs (if any)
open tests/visual_diffs/{screen_name}_diff.png

# Step 4: Accept new baseline (if intentional change)
./scripts/accept_visual_changes.sh {screen_name}
```

## Documentation Requirements

Each screen workstream must update:
1. **RFD 0003**: Mark screen workstream complete
2. **Screen API Docs**: Document render + input APIs
3. **User Guide**: Update screenshots (if UI changed)
4. **Migration Notes**: Document any subtle behavior changes

**Template per screen**:
```markdown
# WS-XX: [Screen Name] Screen Migration

## Summary
Migrated [Screen Name] screen from old ratatui to Bevy ECS rendering.

## Files Created
- `rust/src/bevy_app/systems/render/screens/{name}.rs`
- `rust/src/bevy_app/systems/input/screens/{name}.rs`

## API Changes
- None (uses existing resources and events)

## Visual Changes
- None (pixel-perfect parity verified)

## Testing
- Unit tests: X passing
- Visual regression: ✅ Passed
- Performance: <16ms frame time

## Migration Notes
- [Any subtle differences or quirks discovered]
```

## Conclusion

The Screen Migration Orchestrator demonstrates the power of **workstream isolation**. With zero file overlap, all 8 screens can migrate in parallel, reducing timeline from **16 days sequential** to **4-5 days parallel**.

**Critical Success Factors**:
1. **Follow the pattern**: WS-09 establishes the template, all others copy
2. **Test visually**: Screenshots prevent regressions
3. **Rebase often**: Keep branches fresh to minimize conflicts
4. **Communicate clearly**: Report progress daily per screen

**Strategic Value**:
- Fastest orchestrator (parallel execution)
- Lowest risk (isolated workstreams)
- Highest confidence (visual verification)
- Foundation for advanced features (GPU rendering, MCP)

**Next Steps**:
1. Verify Core Systems complete (WS-05 through WS-08 merged)
2. Assign 8 screen workstreams to owners (agents or humans)
3. Create feature branches
4. Kickoff all 8 workstreams simultaneously
5. Daily visual diff reviews
6. Merge as ready, rebase on conflicts

---

**Status**: ✅ **Ready for Execution**
**Owner**: Screen Migration Orchestrator
**Last Updated**: 2025-11-14
