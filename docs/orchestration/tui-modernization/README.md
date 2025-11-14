# TUI Modernization: bevy_ratatui Migration

**Status**: Planning Complete → Ready for Execution
**Timeline**: 6 weeks (18 workstreams)
**Strategy**: Incremental coexistence (strangler fig pattern)

## Quick Links

- **Strategy**: [RFD 0003 - bevy_ratatui Migration](../../rfds/0003-bevy-ratatui-migration.md)
- **Coordination**: [Meta-Orchestrator](meta-orchestrator.md)
- **Orchestrators**:
  - [Foundation](orchestrators/foundation.md) (Weeks 1-2)
  - [Core Systems](orchestrators/core-systems.md) (Weeks 2-3)
  - [Screen Migration](orchestrators/screen-migration.md) (Weeks 3-5)
  - [Integration](orchestrators/integration.md) (Weeks 5-6)

## Executive Summary

### What We're Doing

Migrating the DGX-Pixels TUI from **pure ratatui** (manual event loop, imperative architecture) to **bevy_ratatui** (ECS-based, GPU-accelerated, declarative systems).

### Why Now

1. **Broken Sixel encoding** - Current image preview is incomplete (placeholder)
2. **Limited terminal compatibility** - Only works in Sixel terminals
3. **Strategic alignment** - Project targets Bevy developers
4. **Foundation for MCP** - Bevy asset system enables hot-reloading
5. **Future features** - GPU rendering enables 3D previews, animations

### How (Strategy)

**Incremental Coexistence**: New Bevy code grows alongside old ratatui code, feature-flagged. Old code removed after all features migrated and validated.

### Timeline

```
Week 1-2: Foundation (Bevy runtime, ECS state, input, rendering)
Week 2-3: Core Systems (ZeroMQ, image assets, theme, events)
Week 3-5: Screen Migration (7 screens, fully parallel)
Week 5-6: Integration (MCP, dual-mode, cleanup)
```

### Risk Level

**High** (major architectural change) - Mitigated by:
- Incremental rollout (feature flags)
- Comprehensive testing (integration tests per workstream)
- Rollback capability (old code preserved)
- Isolated workstreams (minimal merge conflicts)

## Architecture Overview

### Current (Pure ratatui)

```rust
// main.rs - Manual event loop
async fn run_app(terminal: &mut Terminal, app: &mut App) -> Result<()> {
    loop {
        // 1. Poll ZeroMQ manually
        // 2. Poll preview manager manually
        // 3. Render directly: ui::render(terminal, app)?
        // 4. Poll input: event::poll()
        // 5. Mutate app state directly
    }
}

// Monolithic state
struct App {
    current_screen: Screen,
    input_buffer: String,
    active_jobs: Vec<ActiveJob>,
    gallery_images: Vec<PathBuf>,
    // ... 20+ more fields
}
```

**Pain Points**:
- ❌ 300-line manual event loop
- ❌ Monolithic state management
- ❌ Imperative event handling
- ❌ Manual async coordination
- ❌ Incomplete Sixel implementation
- ❌ No GPU acceleration

### Target (bevy_ratatui)

```rust
// main.rs - Bevy ECS
fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(1.0/60.0))),
            RatatuiPlugins::default(),
            DgxPixelsPlugin,
        ))
        .add_systems(PreUpdate, (handle_input, poll_zmq))
        .add_systems(Update, (update_jobs, update_gallery))
        .add_systems(PostUpdate, render_all_screens)
        .run();
}

// Decomposed state
#[derive(Resource)] struct CurrentScreen(Screen);
#[derive(Resource)] struct InputBuffer { text: String, cursor: usize }
#[derive(Resource)] struct GalleryState { images: Vec<PathBuf>, selected: usize }

#[derive(Component)] struct Job { id: String, prompt: String, status: JobStatus }
```

**Benefits**:
- ✅ Bevy ECS handles scheduling & parallelism
- ✅ State decomposed into resources/components
- ✅ Message-based input
- ✅ GPU-accelerated image rendering
- ✅ Natural async integration
- ✅ Foundation for MCP & dual-mode

## Workstream Breakdown

### Phase 1: Foundation (Weeks 1-2)

| WS | Name | Duration | Risk | Dependencies |
|----|------|----------|------|--------------|
| WS-01 | Bevy Runtime Setup | 3-4 days | Low | None |
| WS-02 | ECS State Migration | 4-5 days | Medium | WS-01 |
| WS-03 | Input System | 3-4 days | Medium | WS-02 |
| WS-04 | Rendering Pipeline | 4-5 days | High | WS-02 |

**Goal**: Bevy app runs at 60 FPS, all core systems migrated

### Phase 2: Core Systems (Weeks 2-3)

| WS | Name | Duration | Risk | Dependencies |
|----|------|----------|------|--------------|
| WS-05 | ZeroMQ Integration | 3-4 days | Medium | WS-02 |
| WS-06 | Image Asset System | 4-5 days | High | WS-04 |
| WS-07 | Theme & Styling | 2-3 days | Low | WS-04 |
| WS-08 | Event Bus | 2-3 days | Low | WS-02, WS-03 |

**Goal**: All subsystems integrated with Bevy

### Phase 3: Screen Migration (Weeks 3-5)

| WS | Name | Duration | Risk | Dependencies |
|----|------|----------|------|--------------|
| WS-09 | Generation Screen | 1-2 days | Low | WS-04, WS-03 |
| WS-10 | Gallery Screen | 1-2 days | Low | WS-04, WS-03, WS-06 |
| WS-11 | Comparison Screen | 1-2 days | Low | WS-04, WS-03, WS-06 |
| WS-12 | Models Screen | 1-2 days | Low | WS-04, WS-03 |
| WS-13 | Queue Screen | 1-2 days | Low | WS-04, WS-03, WS-05 |
| WS-14 | Monitor Screen | 1-2 days | Low | WS-04, WS-03, WS-05 |
| WS-15 | Settings Screen | 1-2 days | Low | WS-04, WS-03 |
| WS-16 | Help Screen | 1-2 days | Low | WS-04, WS-03 |

**Goal**: All UI screens functional in Bevy mode
**Parallelization**: All 8 screens can be migrated simultaneously (no file overlap)

### Phase 4: Integration (Weeks 5-6)

| WS | Name | Duration | Risk | Dependencies |
|----|------|----------|------|--------------|
| WS-17 | MCP Integration | 4-5 days | Medium | WS-06, All screens |
| WS-18 | Dual-Mode Rendering | 3-4 days | Low | All screens |

**Goal**: MCP hot-reload working, terminal + window modes operational

## Workstream Isolation Strategy

### File Ownership Matrix

| Area | Workstreams | Conflict Risk |
|------|-------------|---------------|
| `rust/Cargo.toml` | All (coordinate) | ⚠️  High |
| `rust/src/main.rs` | WS-01, WS-02 | ⚠️  Medium |
| `rust/src/bevy_app/mod.rs` | WS-01 | ✅ None |
| `rust/src/bevy_app/plugins.rs` | All (sections) | ⚠️  Low (coordinated) |
| `rust/src/bevy_app/resources/*` | WS-02 | ✅ None |
| `rust/src/bevy_app/systems/input/*` | WS-03 | ✅ None |
| `rust/src/bevy_app/systems/render/*` | WS-04, WS-09-16 | ✅ Low (per-screen files) |
| `rust/src/bevy_app/systems/zmq/*` | WS-05 | ✅ None |
| `rust/src/bevy_app/assets/*` | WS-06 | ✅ None |
| `rust/src/sixel/*` | None (deprecated) | ✅ None (delete after WS-06) |
| `rust/src/ui/screens/*.rs` | None (reference) | ✅ None (read-only) |

### Conflict Prevention

1. **Cargo.toml**: All dependency additions listed in PR description, meta-orchestrator reviews
2. **plugins.rs**: Dedicated section per workstream with comments
3. **Parallel screens**: Each screen = separate file, zero overlap
4. **Daily rebases**: `git pull --rebase origin main` before daily work
5. **CI enforcement**: Build fails on file overlap between open PRs

## Progress Tracking

### GitHub Project Board

```
Kanban Columns:
📋 Backlog (Not Started)
🏗️  In Progress (Active workstreams)
👀 Review (PR open, awaiting review)
🧪 Testing (Merged, validation ongoing)
✅ Done (Complete, verified)
```

### Metrics Dashboard

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Workstreams Complete | 0/18 | 18/18 | 🔴 |
| Test Coverage | 65% | >80% | 🔴 |
| Frame Time | 15ms | <16ms | 🟢 |
| CI Pass Rate | 98% | >95% | 🟢 |

### Weekly Reporting

**Every Friday**: Meta-orchestrator publishes progress report
- Workstreams completed this week
- Workstreams in progress (% complete)
- Blockers and resolutions
- Metrics snapshot
- Risks and mitigations
- Next week goals

**Format**: GitHub Discussions thread + visual dashboard (GitHub Projects)

## Testing Strategy

### Per-Workstream Testing

Every workstream must pass before merge:

1. **Unit Tests**: Core logic tested in isolation
2. **Integration Tests**: End-to-end scenarios (input → state → render)
3. **Performance Tests**: Frame time benchmarked
4. **Regression Tests**: Old features still work

**Example (WS-03 Input System)**:
```rust
#[test]
fn test_screen_navigation() {
    let mut app = create_test_bevy_app();

    // Simulate Tab key
    app.world.send_message(KeyMessage { code: KeyCode::Tab, .. });
    app.update();

    let screen = app.world.resource::<CurrentScreen>();
    assert_eq!(screen.0, Screen::Comparison);
}
```

### Continuous Integration

```yaml
# .github/workflows/tui-modernization.yml
on:
  pull_request:
    branches:
      - 'tui-modernization/**'

jobs:
  test-dual-mode:
    steps:
      - name: Test old mode
        run: cargo test
      - name: Test new mode
        run: cargo test --features bevy_migration
      - name: Benchmark
        run: cargo bench --features bevy_migration
      - name: Coverage
        run: cargo tarpaulin --features bevy_migration --out Lcov
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Validation Checklist (Pre-Merge)

- [ ] All tests passing (unit, integration, performance)
- [ ] Test coverage >75% (workstream-specific code)
- [ ] Frame time <16ms (60 FPS maintained)
- [ ] No memory leaks (heaptrack verified)
- [ ] Code review approved (1+ reviewer)
- [ ] Documentation updated (rustdoc, RFD, CHANGELOG)
- [ ] Feature flag properly configured
- [ ] CI green (all checks passing)
- [ ] No merge conflicts with main

## Risk Management

### Top 5 Risks

1. **Performance Regression (WS-04, WS-06)**
   - **Impact**: Users experience lag, < 60 FPS
   - **Probability**: Medium
   - **Mitigation**: Benchmark every commit, profile hot paths, keep old renderer as fallback

2. **Broken Image Preview (WS-06)**
   - **Impact**: Core feature non-functional
   - **Probability**: Medium
   - **Mitigation**: Feature-flagged rollout, Sixel fallback for 2 releases, extensive testing

3. **Merge Conflicts (Cross-cutting)**
   - **Impact**: Development velocity slows
   - **Probability**: Medium-High
   - **Mitigation**: Strict file ownership, daily rebases, CI enforcement, coordinated `Cargo.toml` changes

4. **Scope Creep (All workstreams)**
   - **Impact**: Timeline extends beyond 6 weeks
   - **Probability**: Medium
   - **Mitigation**: Strict workstream boundaries, "implementation only" rule (no new features), regular scope reviews

5. **Bevy API Changes (External)**
   - **Impact**: Dependency breaks with Bevy updates
   - **Probability**: Low
   - **Mitigation**: Pin Bevy version (0.16.x), upgrade in separate PR post-migration

### Rollback Plan

**Per-Workstream Rollback**:
1. Disable feature flag for that workstream
2. Revert to old code path (always functional)
3. Fix issues in hotfix branch
4. Re-enable feature flag

**Full Migration Rollback** (if catastrophic failure):
1. Disable master feature flag: `bevy_migration`
2. All old code paths still functional
3. Reassess strategy, potentially switch to Phase 1 (ratatui-image) from RFD 0002

## Feature Flag Strategy

### Granular Flags

```toml
# Cargo.toml
[features]
bevy_migration = [
    "bevy_migration_foundation",
    "bevy_migration_core_systems",
    "bevy_migration_screens",
    "bevy_migration_integration",
]

bevy_migration_foundation = ["dep:bevy", "dep:bevy_ratatui"]
bevy_migration_core_systems = ["bevy_migration_foundation"]
bevy_migration_screens = ["bevy_migration_foundation", "bevy_migration_core_systems"]
bevy_migration_integration = ["bevy_migration_screens"]
```

### Gradual Rollout Plan

| Release | Feature Flags Enabled | Old Code Status |
|---------|----------------------|-----------------|
| v0.1.0 (Current) | None | ✅ Active |
| v0.2.0 (Week 3) | `foundation` + `core_systems` | ✅ Available (fallback) |
| v0.3.0 (Week 6) | All (master `bevy_migration`) | ⚠️  Deprecated |
| v0.4.0 (Week 8) | All (default ON) | ❌ Removed |

## Communication Plan

### Daily (Async)

**Workstream Owners** post standup in GitHub Discussions:
- Progress (what shipped)
- Current task
- Blockers
- Metrics snapshot

### Weekly (Sync)

**All Orchestrators** join 30-min video call:
- Review dependency graph
- Resolve cross-orchestrator issues
- Adjust timeline if needed
- Celebrate wins

**Meta-Orchestrator** publishes written report:
- Workstreams completed
- Metrics dashboard
- Risks and mitigations
- Next week goals

### Milestone Demos

**4 Major Demos** (end of each phase):
1. **Foundation Demo** (Week 2): Bevy app runs, input + rendering working
2. **Core Systems Demo** (Week 3): Images via Bevy assets, ZMQ integrated
3. **Screens Demo** (Week 5): All 7 screens functional in Bevy mode
4. **Integration Demo** (Week 6): MCP hot-reload, dual-mode operational

**Format**: Live demo recording + written summary

## Success Criteria

### Technical Milestones

- ✅ Bevy app runs at 60 FPS (frame time <16ms)
- ✅ All 7 screens functional in Bevy mode
- ✅ GPU-accelerated image rendering working
- ✅ ZeroMQ integrated with Bevy async runtime
- ✅ MCP hot-reload operational
- ✅ Terminal + window modes both functional
- ✅ Test coverage >80%
- ✅ Old ratatui code removed

### User-Visible Benefits

1. **Working Image Preview** - No more broken Sixel placeholder!
2. **Better Terminal Compatibility** - Works in 80%+ of terminals (fallback support)
3. **GPU Rendering** - Smoother, higher-quality image display
4. **MCP Integration** - Live asset sync with Bevy projects
5. **Native Window Mode** - Optional GUI for better image quality
6. **Foundation for 3D** - Can add 3D model previews in future

### Project Outcomes

- ✅ **Alignment with goals**: Perfect fit for Bevy developer audience
- ✅ **Strategic foundation**: MCP + Bevy assets enable future features
- ✅ **Technical debt eliminated**: 600+ lines of custom code → battle-tested libraries
- ✅ **Risk mitigated**: Incomplete Sixel implementation replaced
- ✅ **Competitive advantage**: Unique GPU-rendered TUI for pixel art

## Next Steps (Execution Kickoff)

### Immediate (Today)

1. ✅ [DONE] Create meta-orchestrator plan
2. ✅ [DONE] Write RFD 0003 (migration strategy)
3. ✅ [DONE] Define 18 workstreams
4. ✅ [DONE] Map dependencies and risks

### This Week

5. ⏭️  Assign orchestrator roles (4 sub-orchestrators)
6. ⏭️  Create feature branches (18 branches)
7. ⏭️  Write detailed workstream READMEs (templates provided)
8. ⏭️  Set up GitHub Project board (kanban)

### Next Week (Week 1)

9. ⏭️  **Kickoff WS-01** (Bevy Runtime Setup)
10. ⏭️  Daily standups begin
11. ⏭️  First progress report (Friday)

## Getting Started (For Implementers)

### Claim a Workstream

1. Review [workstream specifications](workstreams/)
2. Check [dependency graph](../../rfds/0003-bevy-ratatui-migration.md#dependency-graph)
3. Comment on GitHub Issue: "I'll take WS-XX"
4. Get assigned by orchestrator

### Begin Implementation

1. Create branch: `tui-modernization/wsXX-name`
2. Read workstream README thoroughly
3. Set up local dev environment
4. Run baseline tests: `cargo test`
5. Implement following acceptance criteria
6. Daily standup updates (GitHub Discussions)
7. Open PR when complete
8. Address review feedback
9. Celebrate merge! 🎉

### Development Loop

```bash
# Daily workflow
git checkout tui-modernization/wsXX-name
git pull --rebase origin main  # Stay synced

# Implement, test locally
cargo test --features bevy_migration_foundation
cargo bench --features bevy_migration_foundation

# Commit, push
git add .
git commit -m "wsXX: implement [feature]"
git push

# Open PR when done
gh pr create --title "WS-XX: [Workstream Name]" \
             --body "$(cat .github/pr-template.md)"
```

## Resources

### Documentation
- [RFD 0002: Image Preview Architecture](../../rfds/0002-image-preview-architecture.md)
- [RFD 0003: bevy_ratatui Migration](../../rfds/0003-bevy-ratatui-migration.md)
- [bevy_ratatui Docs](https://docs.rs/bevy_ratatui/latest/)
- [Bevy ECS Guide](https://bevyengine.org/learn/book/getting-started/ecs/)

### Tools
- `cargo flamegraph` - Performance profiling
- `heaptrack` - Memory leak detection
- `cargo tarpaulin` - Test coverage
- `cargo bench` - Benchmarking

### Templates
- [Workstream README Template](workstreams/_template/README.md)
- [PR Template](.github/pull_request_template.md)
- [Daily Standup Template](templates/daily-standup.md)
- [Weekly Report Template](templates/weekly-report.md)

## FAQ

**Q: Why not just fix the Sixel implementation?**
A: That only solves preview, not strategic alignment with Bevy or GPU rendering. bevy_ratatui gets us both + MCP foundation.

**Q: Why 6 weeks? Can we go faster?**
A: 18 workstreams, some sequential. Could be 4 weeks with more parallelization, but that increases risk.

**Q: What if Bevy updates break compatibility?**
A: We pin Bevy 0.16.x during migration. Upgrade in separate PR post-migration.

**Q: Can I help?**
A: Yes! Claim a workstream, especially screen migrations (WS-09 through WS-16) - they're parallelizable.

**Q: What happens to old ratatui code?**
A: Kept as fallback for 2 releases (v0.2.0, v0.3.0), removed in v0.4.0.

---

**Status**: ✅ **Planning Complete - Ready for Execution**
**Next Action**: Assign orchestrator roles & kickoff WS-01
**Last Updated**: 2025-11-14
