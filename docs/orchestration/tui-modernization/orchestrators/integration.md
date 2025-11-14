# Integration Orchestrator

**Timeline**: Weeks 5-6
**Workstreams**: 2 (WS-17, WS-18)
**Risk Level**: Medium (external integration, new features)
**Dependencies**: All previous orchestrators (WS-01 through WS-16)

## Mission

Complete the migration by integrating advanced features: **MCP server** for Bevy asset synchronization and **dual-mode rendering** for terminal + native window support. These workstreams unlock the full potential of the Bevy architecture.

## Workstreams

### WS-17: MCP Integration
- **Duration**: 4-5 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws17-mcp-integration`
- **Spec**: [../workstreams/ws17-mcp-integration/README.md](../workstreams/ws17-mcp-integration/README.md)
- **Depends on**: WS-06 (Image Assets), All screen migrations (WS-09 through WS-16)

### WS-18: Dual-Mode Rendering
- **Duration**: 3-4 days
- **Owner**: TBD
- **Branch**: `tui-modernization/ws18-dual-mode-rendering`
- **Spec**: [../workstreams/ws18-dual-mode-rendering/README.md](../workstreams/ws18-dual-mode-rendering/README.md)
- **Depends on**: All screen migrations (WS-09 through WS-16)

## Execution Strategy

### Sequential Execution

```
Week 5: (After Screen Migration WS-16 completes)
┌─────────────────────────────────────┐
│ WS-17: MCP Integration (4-5 days)  │
└─────────────────────────────────────┘
                ↓
Week 6:
┌─────────────────────────────────────┐
│ WS-18: Dual-Mode Rendering (3-4d)  │
└─────────────────────────────────────┘
                ↓
         Final Testing (2-3 days)
         Old Code Removal
         Documentation Update
         Release Preparation
```

**Why sequential?**
- **WS-17 (MCP)** requires stable image asset system (WS-06) and all screens operational
- **WS-18 (Dual-Mode)** can run in parallel with WS-17, but benefits from MCP testing first
- Both workstreams are complex enough to warrant focused attention

**Parallel Option** (if resources available):
- WS-17 and WS-18 can run simultaneously
- Both touch different parts of the system
- WS-17: Asset synchronization
- WS-18: Rendering mode switching

## File Ownership Map

| Files/Directories | Primary Owner | Shared? | Notes |
|-------------------|---------------|---------|-------|
| `rust/src/bevy_app/mcp/*` | WS-17 | ✅ Exclusive | New MCP server module |
| `rust/src/bevy_app/systems/asset_sync.rs` | WS-17 | ✅ Exclusive | Asset synchronization |
| `rust/src/bevy_app/systems/hot_reload.rs` | WS-17 | ✅ Exclusive | Hot reload handlers |
| `rust/src/bevy_app/rendering/*` | WS-18 | ✅ Exclusive | Rendering mode management |
| `rust/src/bevy_app/systems/mode_switcher.rs` | WS-18 | ✅ Exclusive | Mode toggle system |
| `rust/src/bevy_app/plugins.rs` | BOTH | ⚠️  Shared | Each adds systems in dedicated section |
| `rust/Cargo.toml` | BOTH | ⚠️  Shared | Coordinate dependency additions |

### Conflict Prevention Rules

1. **plugins.rs Coordination**:
   ```rust
   // WS-17: MCP Integration
   .insert_resource(McpServer::new(config))
   .add_systems(Update, (
       mcp::poll_requests,
       mcp::handle_asset_sync,
       mcp::handle_hot_reload,
   ))

   // WS-18: Dual-Mode Rendering
   .insert_resource(RenderMode::Terminal)
   .add_systems(PreUpdate, mode_switcher::handle_mode_toggle)
   .add_systems(PostUpdate, (
       rendering::terminal::render_terminal.run_if(in_terminal_mode),
       rendering::window::render_window.run_if(in_window_mode),
   ))
   ```

2. **Cargo.toml Dependencies**:
   - **WS-17 adds**: `fastmcp`, `tokio` (full features), `serde_json`
   - **WS-18 adds**: Bevy `DefaultPlugins` (for windowing), `bevy_egui` (optional)
   - Coordinate to avoid dependency conflicts

## Integration Testing

### WS-17 Integration Test
```rust
#[tokio::test]
async fn test_mcp_server() {
    let mut app = create_test_bevy_app();

    // Start MCP server
    let mcp_port = 6789;
    app.world.resource_mut::<McpServer>()
        .start(mcp_port)
        .await
        .unwrap();

    // Connect test client
    let client = McpClient::connect(format!("127.0.0.1:{}", mcp_port))
        .await
        .unwrap();

    // Request asset load
    client.send_request(McpRequest::LoadAsset {
        path: "test_image.png".into(),
    }).await.unwrap();

    // Run Bevy update (process MCP request)
    app.update();

    // Verify asset loaded
    let assets = app.world.resource::<Assets<Image>>();
    assert!(assets.iter().any(|(_, img)| img.path().ends_with("test_image.png")));
}

#[test]
fn test_hot_reload() {
    let mut app = create_test_bevy_app();

    // Load initial asset
    let asset_path = "test_sprite.png";
    let handle = app.world.resource::<AssetServer>().load(asset_path);

    app.update();

    // Modify asset file
    std::fs::write(asset_path, updated_image_data).unwrap();

    // Trigger hot reload
    app.world.send_event(AssetEvent::Modified { handle: handle.clone() });

    app.update();

    // Verify asset reloaded
    let assets = app.world.resource::<Assets<Image>>();
    let image = assets.get(&handle).unwrap();
    assert_eq!(image.size(), updated_size);
}
```

### WS-18 Integration Test
```rust
#[test]
fn test_mode_switching() {
    let mut app = create_test_bevy_app();

    // Start in terminal mode
    assert_eq!(
        app.world.resource::<RenderMode>().0,
        RenderModeKind::Terminal
    );

    // Send mode toggle event (Ctrl+W)
    app.world.send_message(KeyMessage {
        code: KeyCode::Char('w'),
        modifiers: KeyModifiers::CONTROL,
    });

    app.update();

    // Verify switched to window mode
    assert_eq!(
        app.world.resource::<RenderMode>().0,
        RenderModeKind::Window
    );

    // Verify window spawned
    let windows = app.world.query::<&Window>();
    assert_eq!(windows.iter(&app.world).count(), 1);
}

#[test]
fn test_dual_mode_rendering() {
    let mut app = create_test_bevy_app();

    // Enable both modes
    app.world.resource_mut::<RenderMode>().0 = RenderModeKind::Both;

    app.update();

    // Verify both rendering systems executed
    let stats = app.world.resource::<RenderStats>();
    assert!(stats.terminal_rendered);
    assert!(stats.window_rendered);
}
```

## Success Criteria

### Milestone 4: Integration Complete

**Technical**:
- ✅ MCP server operational alongside TUI
- ✅ Assets hot-reload when updated externally
- ✅ Bevy projects can inject assets via MCP
- ✅ Generation results auto-deploy to connected apps
- ✅ Terminal mode works (default)
- ✅ Window mode works (GPU-accelerated UI)
- ✅ Both modes simultaneously (split view)
- ✅ Smooth mode switching (Ctrl+W)

**Performance**:
- ✅ MCP request latency <100ms
- ✅ Hot reload triggers <500ms
- ✅ Terminal rendering maintains 60 FPS
- ✅ Window rendering maintains 144 FPS (on capable hardware)
- ✅ Mode switching <1 second

**Quality**:
- ✅ Test coverage >75% for new code
- ✅ All CI checks passing
- ✅ Documentation complete (MCP protocol, dual-mode usage)
- ✅ Security review passed (MCP server)

**Process**:
- ✅ Both workstreams merged
- ✅ Old ratatui code fully removed
- ✅ Feature flags cleaned up (migration complete)
- ✅ Release notes prepared

### Demo Checklist

**Prepare for final milestone demo**:

1. **MCP Integration Demo**:
   ```bash
   # Terminal 1: Start TUI with MCP
   cargo run --release -- --mcp-port 6789

   # Terminal 2: Connect Bevy game project
   cd ../my-bevy-game
   cargo run
   # Bevy game connects to TUI via MCP
   # Generate sprite in TUI → Auto-loads in Bevy game

   # Terminal 3: External asset injection
   curl -X POST http://localhost:6789/assets \
     -H "Content-Type: application/json" \
     -d '{"path": "external_sprite.png", "data": "..."}'
   # Asset appears in TUI gallery instantly
   ```

2. **Dual-Mode Demo**:
   ```bash
   # Start in terminal mode
   cargo run --release

   # Press Ctrl+W → Switch to window mode
   # Show GPU-accelerated rendering
   # Better image quality, higher resolution

   # Press Ctrl+W → Enable both modes
   # Terminal and window side-by-side
   # Synchronized state, dual rendering

   # Press Ctrl+W → Back to terminal mode
   # Seamless transitions
   ```

3. **Talking Points**:
   - MCP integration enables Bevy ecosystem interop ✅
   - Hot-reloading foundation for live asset updates ✅
   - Dual-mode flexibility (terminal for SSH, window for local) ✅
   - GPU-accelerated rendering unlocked ✅
   - Migration complete, old code removed ✅

## Risk Management

### Medium-Risk Items

1. **WS-17: MCP Server Security**
   - **Risk**: Network-exposed server vulnerable to attacks
   - **Detection**: Security audit, penetration testing
   - **Mitigation**:
     - Authentication required (API keys)
     - Rate limiting
     - Localhost-only default (opt-in remote access)
     - Input validation on all MCP requests

2. **WS-17: Hot Reload Race Conditions**
   - **Risk**: Asset modified while being read
   - **Detection**: Stress testing (rapid asset modifications)
   - **Mitigation**:
     - File watching debounce (wait for writes to complete)
     - Atomic file replacement (write temp → rename)
     - Error recovery (retry on failed load)

3. **WS-18: Window Mode Compatibility**
   - **Risk**: Window mode fails on headless servers or SSH sessions
   - **Detection**: Test on various environments (local, SSH, containers)
   - **Mitigation**:
     - Graceful degradation (detect headless, disable window mode)
     - Clear error messages
     - Terminal mode always available as fallback

4. **WS-18: Rendering Synchronization**
   - **Risk**: State drift between terminal and window modes
   - **Detection**: Visual comparison tests (both modes render same state)
   - **Mitigation**:
     - Single source of truth (ECS resources)
     - Both renderers read same data
     - State updates trigger both renderers

### Rollback Plan

**If Integration fails after merge**:

**WS-17 Rollback**:
1. Disable MCP server: `cfg(feature = "mcp_integration")` → default OFF
2. Remove MCP systems from plugins.rs
3. Asset loading reverts to file watching only

**WS-18 Rollback**:
1. Disable window mode: `cfg(feature = "dual_mode")` → default OFF
2. Terminal mode always active (no mode switching)
3. Remove windowing systems from plugins.rs

**Feature flags**:
```toml
[features]
default = ["bevy_migration_complete"]
bevy_migration_complete = []
mcp_integration = ["bevy_migration_complete"]
dual_mode_rendering = ["bevy_migration_complete"]
```

## Coordination with Other Orchestrators

### Handoff from Screen Migration Orchestrator

**Prerequisites verified**:
- ✅ All screens migrated (WS-09 through WS-16)
- ✅ Image asset system stable (WS-06)
- ✅ Bevy ECS fully operational
- ✅ All old ratatui code removed (except feature-flagged fallbacks)

**Screen Migration provides**:
- Complete UI rendering via Bevy
- All screens operational and tested
- Asset pipeline proven
- Event-driven architecture validated

### Final Handoff: Production Release

**After Integration completes**:
- ✅ Migration 100% complete
- ✅ All advanced features operational
- ✅ Old code removed
- ✅ Documentation finalized

**Release Preparation**:
1. Version bump: `v0.1.0 → v0.2.0` (major architectural change)
2. Changelog: Document all changes, migration notes
3. Migration guide: Help users upgrade from v0.1.x
4. Blog post: Announce bevy_ratatui migration success
5. GitHub release: Tag `v0.2.0`, binaries for multiple platforms

## Daily Standup Format

**Each workstream owner reports** (async, GitHub Discussions):

```markdown
## WS-XX: [Workstream Name] - Day N

### Progress
- Completed: [specific accomplishments]
- In Progress: [current task]
- Next: [upcoming tasks]

### Metrics
- Tests: X passing / Y total
- Performance: [latency/throughput numbers]
- Coverage: X%

### Integration Notes
- [External dependencies tested]
- [Security considerations addressed]
- [Compatibility verified]

### Blockers
- [Any external dependencies waiting]
- [Security review status]
```

**Integration Orchestrator synthesizes** into weekly report.

## Tools & Infrastructure

### Required Tools
- **MCP Testing**: `curl`, `httpie`, MCP client library
- **Security**: `cargo-audit`, `cargo-deny`
- **Window Testing**: X11/Wayland display server, VNC (for CI)
- **Performance**: `cargo flamegraph`, `bevy_framepace`

### CI Pipeline
```yaml
# .github/workflows/integration.yml
name: Integration Tests

on:
  pull_request:
    branches:
      - tui-modernization/ws17-*
      - tui-modernization/ws18-*

jobs:
  test-mcp:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Test MCP Server
        run: |
          cargo test --features mcp_integration test_mcp_server

      - name: Security Audit
        run: |
          cargo audit
          cargo deny check

      - name: MCP Protocol Compliance
        run: |
          cargo run --features mcp_integration -- --mcp-port 6789 &
          sleep 2
          ./tests/mcp_protocol_tests.sh

  test-dual-mode:
    runs-on: ubuntu-latest
    services:
      xvfb:
        image: selenium/standalone-chrome
    steps:
      - uses: actions/checkout@v3

      - name: Test Terminal Mode
        run: |
          cargo test --features dual_mode_rendering test_terminal_mode

      - name: Test Window Mode
        run: |
          export DISPLAY=:99
          Xvfb :99 -screen 0 1024x768x24 &
          cargo test --features dual_mode_rendering test_window_mode

      - name: Test Mode Switching
        run: |
          cargo test --features dual_mode_rendering test_mode_switching
```

### MCP Protocol Testing

```bash
#!/bin/bash
# tests/mcp_protocol_tests.sh

# Test asset loading
curl -X POST http://localhost:6789/mcp/load \
  -H "Content-Type: application/json" \
  -d '{"path": "test.png"}'

# Test asset update
curl -X PUT http://localhost:6789/mcp/update \
  -H "Content-Type: application/json" \
  -d '{"path": "test.png", "data": "base64..."}'

# Test hot reload trigger
curl -X POST http://localhost:6789/mcp/reload \
  -H "Content-Type: application/json" \
  -d '{"path": "test.png"}'

# Verify responses
# All should return 200 OK with JSON payloads
```

## Documentation Requirements

### WS-17: MCP Integration Docs

**Create**:
1. **MCP Protocol Spec** (`docs/mcp-protocol.md`):
   - API endpoints
   - Request/response formats
   - Authentication
   - Examples

2. **MCP Integration Guide** (`docs/bevy-integration.md`):
   - How to connect Bevy game to TUI
   - Asset synchronization workflow
   - Hot-reloading setup
   - Example Bevy plugin code

3. **Security Guide** (`docs/mcp-security.md`):
   - Authentication setup
   - Network configuration
   - Best practices
   - Threat model

### WS-18: Dual-Mode Docs

**Create**:
1. **Dual-Mode User Guide** (`docs/dual-mode-rendering.md`):
   - How to switch modes (Ctrl+W)
   - Terminal mode use cases
   - Window mode use cases
   - Both modes simultaneously

2. **Configuration Guide** (`docs/rendering-config.md`):
   - Default mode selection
   - Window size/position
   - Terminal emulator compatibility
   - Performance tuning

### Shared Docs

**Update**:
1. **README.md**:
   - Feature list (add MCP + dual-mode)
   - Installation (updated dependencies)
   - Quick start (new features)

2. **CHANGELOG.md**:
   - Version 0.2.0 release notes
   - Breaking changes (old ratatui removed)
   - New features (MCP, dual-mode)
   - Migration guide from v0.1.x

3. **Architecture Diagram**:
   - Add MCP server component
   - Show dual rendering paths
   - Update data flow

## Migration Completion Checklist

**Before declaring migration complete**:

### Code Cleanup
- [ ] Remove all old ratatui code (`rust/src/ui/*`)
- [ ] Remove feature flags for old code paths
- [ ] Remove Sixel preview code (`rust/src/sixel/*`)
- [ ] Remove old event handling (`rust/src/events/*`)
- [ ] Clean up `main.rs` (single entry point)

### Testing
- [ ] All tests passing (unit + integration)
- [ ] E2E tests cover all workflows
- [ ] Performance benchmarks meet targets
- [ ] Security audit passed (MCP)
- [ ] Compatibility tested (multiple terminals, OS)

### Documentation
- [ ] All RFD 0003 workstreams marked complete
- [ ] API documentation complete (Rustdoc)
- [ ] User guides written
- [ ] Migration guide for v0.1.x users
- [ ] Architecture diagrams updated

### Release Preparation
- [ ] Version bumped (v0.2.0)
- [ ] CHANGELOG.md finalized
- [ ] Release notes drafted
- [ ] Binaries built for Linux/macOS/Windows
- [ ] GitHub release created

### Communication
- [ ] Blog post written
- [ ] Social media announcement
- [ ] Community Discord update
- [ ] Documentation published

## Conclusion

The Integration Orchestrator completes the **bevy_ratatui migration** by adding advanced features that leverage the Bevy architecture:

**MCP Integration** (WS-17):
- Enables Bevy ecosystem interoperability
- Foundation for live asset synchronization
- Opens door for external tool integrations

**Dual-Mode Rendering** (WS-18):
- Flexibility for different use cases
- GPU-accelerated UI option
- Terminal mode for SSH/remote access

**Strategic Impact**:
- DGX-Pixels becomes **best-in-class Bevy-integrated tool**
- Migration demonstrates **incremental coexistence pattern** success
- Foundation for future features (3D previews, advanced GPU rendering)

**Critical Success Factors**:
1. **Security first**: MCP server must be secure by default
2. **Graceful degradation**: Window mode failures don't break terminal mode
3. **Complete cleanup**: Remove all old code, simplify codebase
4. **Excellent docs**: Users understand new features and migration path

**Next Steps**:
1. Verify Screen Migration complete (WS-09 through WS-16 merged)
2. Assign Integration workstream owners
3. Create feature branches
4. Kickoff WS-17 (MCP Integration)
5. Kickoff WS-18 (Dual-Mode Rendering) after WS-17 or in parallel
6. Final testing and cleanup
7. Release v0.2.0

---

**Status**: ✅ **Ready for Execution**
**Owner**: Integration Orchestrator
**Last Updated**: 2025-11-14
