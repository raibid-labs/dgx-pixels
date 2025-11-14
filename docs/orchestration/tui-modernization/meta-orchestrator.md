# Meta-Orchestrator: TUI Modernization (bevy_ratatui Migration)

**Status**: Active
**Timeline**: 6 weeks (Weeks 1-6 of project)
**Owner**: Lead Architect
**Related RFD**: [RFD 0003](../../rfds/0003-bevy-ratatui-migration.md)

## Mission

Migrate the DGX-Pixels TUI from pure ratatui to bevy_ratatui, implementing a Bevy ECS-based architecture that enables GPU-accelerated rendering, MCP integration, and dual-mode (terminal/window) operation.

## Strategic Goals

1. **Bevy Integration**: Align with project's target audience (Bevy game developers)
2. **GPU Acceleration**: Enable GPU-rendered image previews in terminal
3. **MCP Foundation**: Set up architecture for seamless Bevy asset hot-reloading
4. **Maintainability**: Replace 600+ lines of custom code with battle-tested libraries
5. **Future-Proof**: Enable advanced features (3D previews, animations, collaborative editing)

## Orchestration Architecture

```
Meta-Orchestrator (TUI Modernization)
│
├── Foundation Orchestrator (Weeks 1-2)
│   ├── WS-01: Bevy Runtime Setup
│   ├── WS-02: ECS State Migration
│   ├── WS-03: Input System
│   └── WS-04: Rendering Pipeline
│
├── Core Systems Orchestrator (Weeks 2-3)
│   ├── WS-05: ZeroMQ Integration
│   ├── WS-06: Image Asset System
│   ├── WS-07: Theme & Styling
│   └── WS-08: Event Bus
│
├── Screen Migration Orchestrator (Weeks 3-5)
│   ├── WS-09: Generation Screen
│   ├── WS-10: Gallery Screen
│   ├── WS-11: Comparison Screen
│   ├── WS-12: Models Screen
│   ├── WS-13: Queue Screen
│   ├── WS-14: Monitor Screen
│   ├── WS-15: Settings Screen
│   └── WS-16: Help Screen
│
└── Integration Orchestrator (Weeks 5-6)
    ├── WS-17: MCP Integration
    └── WS-18: Dual-Mode Rendering
```

## Orchestrator Specifications

### Foundation Orchestrator

**Goal**: Establish Bevy runtime and core ECS architecture

**Workstreams**: 4 (WS-01 through WS-04)
**Timeline**: Weeks 1-2
**Dependencies**: None (foundation layer)
**Risk**: Medium-High (architectural change)

**Success Criteria**:
- ✅ Bevy app runs with 60 FPS terminal rendering
- ✅ All app state migrated to ECS resources
- ✅ Input handling message-based (Bevy events)
- ✅ Rendering uses `RatatuiContext::draw()`
- ✅ Old ratatui mode still functional (feature-flagged)

**Specification**: [orchestrators/foundation.md](orchestrators/foundation.md)

---

### Core Systems Orchestrator

**Goal**: Migrate core subsystems to Bevy architecture

**Workstreams**: 4 (WS-05 through WS-08)
**Timeline**: Weeks 2-3
**Dependencies**: Foundation complete (WS-01 to WS-04)
**Risk**: Medium

**Success Criteria**:
- ✅ ZeroMQ integrated with Bevy async runtime
- ✅ Image rendering via Bevy assets (GPU-accelerated)
- ✅ Theme system as Bevy resource
- ✅ Event-driven architecture operational
- ✅ Sixel system deprecated (but still available as fallback)

**Specification**: [orchestrators/core-systems.md](orchestrators/core-systems.md)

---

### Screen Migration Orchestrator

**Goal**: Migrate all UI screens to Bevy render systems

**Workstreams**: 8 (WS-09 through WS-16)
**Timeline**: Weeks 3-5
**Dependencies**: Foundation + Core Systems
**Risk**: Low (isolated per screen)

**Success Criteria**:
- ✅ All 7 screens render in Bevy mode
- ✅ Visual parity with old ratatui implementation
- ✅ All interactions functional
- ✅ 60 FPS maintained across all screens
- ✅ Feature parity verified per screen

**Specification**: [orchestrators/screen-migration.md](orchestrators/screen-migration.md)

---

### Integration Orchestrator

**Goal**: Complete Bevy integration with MCP and dual-mode rendering

**Workstreams**: 2 (WS-17 through WS-18)
**Timeline**: Weeks 5-6
**Dependencies**: All screens migrated
**Risk**: Medium

**Success Criteria**:
- ✅ MCP server operational (asset hot-reloading)
- ✅ Terminal and window modes both functional
- ✅ Seamless mode switching (Ctrl+W)
- ✅ Old ratatui code removed
- ✅ Documentation updated
- ✅ Performance targets met

**Specification**: [orchestrators/integration.md](orchestrators/integration.md)

---

## Coordination Strategy

### Daily Standup (Async)

Each orchestrator reports:
1. **Completed**: Workstreams merged yesterday
2. **In Progress**: Active workstreams today
3. **Blocked**: Dependencies or issues
4. **Metrics**: Test coverage, performance benchmarks

Format: GitHub Discussions thread or Slack channel

### Weekly Sync

**Attendees**: Meta-orchestrator + All sub-orchestrators
**Agenda**:
1. Review dependency graph status
2. Resolve cross-orchestrator conflicts
3. Adjust timeline if needed
4. Celebrate wins

**Format**: 30-min video call or detailed written update

### Merge Policy

**Branch Strategy**:
```
main
├── tui-modernization/foundation     (Foundation orchestrator umbrella)
│   ├── ws01-bevy-runtime
│   ├── ws02-ecs-state
│   ├── ws03-input-system
│   └── ws04-rendering-pipeline
│
├── tui-modernization/core-systems   (Core Systems orchestrator umbrella)
│   ├── ws05-zmq-integration
│   ├── ws06-image-assets
│   ├── ws07-theme
│   └── ws08-event-bus
│
├── tui-modernization/screens        (Screen Migration orchestrator umbrella)
│   ├── ws09-generation-screen
│   ├── ws10-gallery-screen
│   ├── ws11-comparison-screen
│   ├── ws12-models-screen
│   ├── ws13-queue-screen
│   ├── ws14-monitor-screen
│   ├── ws15-settings-screen
│   └── ws16-help-screen
│
└── tui-modernization/integration    (Integration orchestrator umbrella)
    ├── ws17-mcp-integration
    └── ws18-dual-mode
```

**Merge Rules**:
1. ✅ All tests passing (CI green)
2. ✅ Code review approved (at least 1 reviewer)
3. ✅ Performance benchmarks meet targets
4. ✅ Documentation updated
5. ✅ Feature flag properly configured
6. ✅ No merge conflicts with main (rebase required)

**Merge Order**:
- Foundation workstreams: **Sequential** (WS-01 → WS-02 → WS-03 → WS-04)
- Core Systems workstreams: **Parallel after Foundation**
- Screen Migration workstreams: **Fully parallel**
- Integration workstreams: **Sequential after Screens**

### Conflict Resolution

**File Ownership Matrix**:

| File/Module | Owner Orchestrator | Shared? |
|-------------|-------------------|---------|
| `rust/Cargo.toml` | Foundation | ⚠️  Yes - coordinate additions |
| `rust/src/main.rs` | Foundation | ⚠️  Yes - entry point changes |
| `rust/src/bevy_app/*` | All | ⚠️  Yes - module isolation required |
| `rust/src/app.rs` | Foundation | ❌ No - locked during WS-02 |
| `rust/src/sixel/*` | Core Systems | ❌ No - deprecated after WS-06 |
| `rust/src/ui/screens/*.rs` | Screen Migration | ✅ No overlap (1 screen per workstream) |

**Conflict Protocol**:
1. **Prevention**: Daily `git pull --rebase origin main` per workstream
2. **Detection**: CI fails on overlapping changes
3. **Resolution**: Meta-orchestrator mediates, enforces file ownership
4. **Escalation**: If unresolved in 24h, meta-orchestrator makes final decision

### Progress Tracking

**GitHub Project Board**:
```
Columns:
- Backlog (Not Started)
- In Progress (Active workstreams)
- Review (PR open, awaiting review)
- Testing (Merged to main, validation ongoing)
- Done (Complete, verified)
```

**Metrics Dashboard**:
- **Velocity**: Workstreams completed per week
- **Quality**: Test coverage % (target: >80%)
- **Performance**: Frame time (target: <16ms)
- **Stability**: CI pass rate (target: >95%)

**Weekly Report Template**:
```markdown
## Week X Progress Report

### Completed Workstreams
- [x] WS-XX: [Name] - [Link to PR]

### In Progress (with % complete)
- [ ] WS-YY: [Name] - 60% (ETA: +2 days)

### Blocked
- [ ] WS-ZZ: [Name] - Waiting on WS-YY completion

### Metrics
- Test Coverage: 82% (+3% from last week)
- Frame Time: 14.2ms (target: <16ms) ✅
- CI Pass Rate: 96% ✅

### Risks
- [Risk description and mitigation plan]

### Next Week Goals
- Complete WS-XX, WS-YY
- Begin WS-ZZ, WS-AA
```

## Risk Management

### High-Risk Workstreams

1. **WS-04: Rendering Pipeline**
   - **Risk**: Performance regression, visual glitches
   - **Owner**: Foundation Orchestrator
   - **Mitigation**:
     - Benchmark every commit
     - Visual regression tests (snapshot testing)
     - Keep old renderer as fallback

2. **WS-06: Image Asset System**
   - **Risk**: Memory leaks, broken preview
   - **Owner**: Core Systems Orchestrator
   - **Mitigation**:
     - Memory profiling with heaptrack
     - Feature flag for gradual rollout
     - Sixel fallback for 2 releases

3. **Merge Conflicts (cross-cutting)**
   - **Risk**: Parallel workstreams collide
   - **Owner**: Meta-orchestrator
   - **Mitigation**:
     - Strict file ownership enforcement
     - Daily rebases
     - CI prevents overlapping changes

### Rollback Plan

**If major issue discovered post-merge**:
1. **Immediate**: Disable feature flag (revert to old behavior)
2. **Short-term**: Fix in hotfix branch, fast-track review
3. **Long-term**: If unfixable, revert PR and redesign

**Feature Flag Strategy**:
```rust
// Each workstream has toggle
#[cfg(feature = "bevy_migration_foundation")]
#[cfg(feature = "bevy_migration_core_systems")]
#[cfg(feature = "bevy_migration_screens")]
#[cfg(feature = "bevy_migration_integration")]

// Master toggle
#[cfg(feature = "bevy_migration")]  // Enables all
```

Gradual rollout:
- **Week 2**: Foundation only
- **Week 3**: +Core Systems
- **Week 5**: +Screens
- **Week 6**: +Integration, remove old code

## Communication Plan

### Stakeholders

- **Lead Architect**: Overall vision, final decisions
- **Sub-Orchestrators**: Day-to-day coordination (4 roles)
- **Implementers**: Workstream owners (18 agents/devs)
- **Reviewers**: Code review, quality assurance
- **Users**: Beta testers (after Week 4)

### Communication Channels

- **GitHub Issues**: Workstream tracking, bug reports
- **GitHub Discussions**: Async standups, design discussions
- **Slack/Discord**: Real-time coordination (optional)
- **Documentation**: Living RFD, updated weekly

### Cadence

- **Daily**: Orchestrator check-ins (async, 5 min)
- **Weekly**: Progress reports (written)
- **Bi-weekly**: Full team sync (video call, 30 min)
- **Milestones**: Demos (Foundation, Core Systems, Screens, Integration)

## Success Metrics

### Technical Metrics

| Metric | Baseline (Old) | Target (New) | Measurement |
|--------|----------------|--------------|-------------|
| Frame Time | 15ms | <16ms | `cargo flamegraph` |
| Input Latency | 50ms | <16ms | Manual testing |
| Preview Load | 1000ms | <500ms | Benchmark suite |
| Memory Usage | 50MB | <100MB | `heaptrack` |
| Test Coverage | 65% | >80% | `cargo tarpaulin` |
| Binary Size | 15MB | <40MB | `ls -lh` |

### Process Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Velocity | 3-4 workstreams/week | GitHub Project |
| CI Pass Rate | >95% | GitHub Actions |
| PR Cycle Time | <2 days | GitHub Insights |
| Merge Conflicts | <2 per week | Git logs |
| Test Failures | <5% | CI dashboard |

### User Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Beta Users | 5+ | Signup form |
| Bug Reports | <10 critical | GitHub Issues |
| User Satisfaction | >4/5 stars | Survey |
| Feature Parity | 100% | Checklist |

## Timeline & Milestones

### Milestone 1: Foundation Complete (End of Week 2)
**Demo**: Bevy app runs, input works, simple screen renders
**Deliverables**:
- ✅ Bevy runtime operational
- ✅ ECS state migrated
- ✅ Input system functional
- ✅ Rendering pipeline working

### Milestone 2: Core Systems Complete (End of Week 3)
**Demo**: Images render via Bevy assets, ZMQ works
**Deliverables**:
- ✅ ZeroMQ integrated
- ✅ GPU image rendering
- ✅ Theme system ported
- ✅ Event bus operational

### Milestone 3: Screens Complete (End of Week 5)
**Demo**: All UI screens functional in Bevy mode
**Deliverables**:
- ✅ 7 screens migrated
- ✅ Visual parity verified
- ✅ All interactions working
- ✅ Performance targets met

### Milestone 4: Integration Complete (End of Week 6)
**Demo**: MCP hot-reload working, dual-mode operational
**Deliverables**:
- ✅ MCP integration live
- ✅ Terminal + window modes
- ✅ Old code removed
- ✅ Documentation complete
- ✅ **v0.2.0 released**

## Post-Migration Plan

### Week 7: Stabilization
- Bug fixes from user feedback
- Performance optimizations
- Documentation polish

### Week 8+: New Features
- 3D model previews (leveraging Bevy 3D)
- Animation timeline viewer
- Collaborative editing (multiple users)
- Plugin system for custom generators

## Conclusion

This meta-orchestrator provides the coordination framework for a complex, 6-week migration from pure ratatui to bevy_ratatui. Success depends on:

1. **Clear ownership**: Each orchestrator owns specific workstreams
2. **Strict isolation**: File-level boundaries prevent conflicts
3. **Incremental delivery**: Feature flags enable safe rollout
4. **Continuous validation**: Daily testing ensures no regressions
5. **Transparent communication**: Weekly reports keep stakeholders aligned

**Next Steps**:
1. ✅ Approve this meta-orchestrator plan
2. ⏭️  Create 4 sub-orchestrator specifications
3. ⏭️  Generate 18 detailed workstream READMEs
4. ⏭️  Assign orchestrator roles (agents or humans)
5. ⏭️  Kickoff WS-01 (Bevy Runtime Setup)

---

**Status**: ✅ **Ready for Execution**
**Owner**: Lead Architect
**Last Updated**: 2025-11-14
