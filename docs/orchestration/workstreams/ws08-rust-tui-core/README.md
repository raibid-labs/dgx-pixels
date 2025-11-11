# WS-08: Rust TUI Core

**ID**: WS-08
**Orchestrator**: Interface
**Milestone**: M2
**Duration**: 6-8 days
**Priority**: P0 (CRITICAL PATH)
**Dependencies**: WS-01 (Hardware Baselines)
**Agent Type**: `rust-pro`
**Status**: Not Started

---

## Objective

Build the core Rust TUI application using ratatui framework to provide a fast, responsive terminal-based interface for DGX-Pixels. This workstream creates the foundation for all user interaction, including generation requests, gallery viewing, settings management, and image previews. The TUI leverages the DGX-Spark's unified memory architecture for zero-copy image access and provides 60+ FPS rendering for a fluid user experience.

**Importance**: This is the primary user interface for DGX-Pixels. It blocks ZeroMQ IPC (WS-09) and all downstream interface workstreams. The Rust TUI is a key differentiator from Python-only approaches, providing superior performance and responsiveness.

---

## Deliverables

1. **Rust Project Structure** (`/home/beengud/raibid-labs/dgx-pixels/rust/`)
   - Cargo.toml with all dependencies
   - Standard Rust project layout (src/, tests/, benches/)
   - Binary target: `dgx-pixels-tui`

2. **TUI Framework** (`rust/src/ui/`)
   - `rust/src/ui/mod.rs` - UI module entry point
   - `rust/src/ui/app.rs` - Application state management
   - `rust/src/ui/layout.rs` - Responsive layout engine
   - `rust/src/ui/theme.rs` - Color scheme and styling
   - `rust/src/ui/widgets/` - Custom reusable widgets

3. **Screen Layouts** (`rust/src/ui/screens/`)
   - `rust/src/ui/screens/generation.rs` - Prompt input and generation controls
   - `rust/src/ui/screens/gallery.rs` - Generated image browser
   - `rust/src/ui/screens/settings.rs` - Configuration management
   - `rust/src/ui/screens/help.rs` - Keyboard shortcuts and help
   - `rust/src/ui/screens/comparison.rs` - Side-by-side model comparison (placeholder for WS-12)

4. **Event Handling** (`rust/src/events.rs`)
   - Keyboard input handling
   - Mouse event support
   - Async event loop with tokio
   - Input validation and sanitization

5. **State Management** (`rust/src/state.rs`)
   - Application state machine
   - Navigation stack
   - User preferences persistence (TOML config)

6. **Binary & Build System**
   - Release-optimized binary (size ≤ 15MB)
   - Cross-compilation support (ARM64 primary)
   - Installation script

7. **Tests & Benchmarks**
   - Unit tests for all UI components
   - Integration tests for screen navigation
   - Performance benchmarks for rendering

8. **Documentation**
   - `rust/README.md` - Setup and usage guide
   - `docs/tui-architecture.md` - Technical architecture
   - Inline code documentation (rustdoc)

---

## Acceptance Criteria

**Functional**:
- ✅ TUI renders at 60+ FPS on DGX-Spark terminal
- ✅ Responsive keyboard navigation (arrow keys, vim keys, tab/shift-tab)
- ✅ Mouse support for clicking buttons and scrolling (optional, keyboard-first)
- ✅ All screen layouts implemented: generation, gallery, settings, help
- ✅ Navigation between screens with breadcrumb trail
- ✅ Settings persist to `~/.config/dgx-pixels/config.toml`
- ✅ Graceful terminal resize handling (no crash or layout break)
- ✅ Clean exit on Ctrl-C or 'q' key (proper cleanup)

**Performance**:
- ✅ Frame time ≤ 16.6ms (60 FPS) on DGX-Spark terminal
- ✅ Input latency ≤ 50ms (key press to screen update)
- ✅ Binary size ≤ 15MB (release build with strip)
- ✅ Memory usage ≤ 50MB (TUI only, no backend)
- ✅ Startup time ≤ 500ms (from invocation to first render)

**Quality**:
- ✅ Test coverage ≥ 80% (cargo tarpaulin)
- ✅ All clippy lints passing (no warnings)
- ✅ Rustfmt formatted (standard style)
- ✅ Documentation complete (all public APIs documented)
- ✅ No unsafe code (unless explicitly justified)

---

## Technical Requirements

### Environment
- **Hardware**: DGX-Spark GB10 (ARM64)
- **OS**: Ubuntu 22.04 (ARM64)
- **Rust**: 1.70+ (stable channel)
- **Terminal**: xterm-256color compatible (iTerm2, WezTerm, Alacritty recommended)

### Dependencies

**Rust Crates** (Cargo.toml):
```toml
[package]
name = "dgx-pixels-tui"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"

[dependencies]
# TUI framework
ratatui = "0.26"
crossterm = "0.27"

# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"

# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
dirs = "5.0"
chrono = "0.4"

[dev-dependencies]
# Testing
criterion = "0.5"
proptest = "1.4"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

**System Dependencies**:
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup target add aarch64-unknown-linux-gnu

# Build tools (already available on DGX-Spark)
sudo apt install -y build-essential pkg-config
```

### Technical Constraints
- Must compile for ARM64 (aarch64-unknown-linux-gnu target)
- Must work in tmux/screen sessions (no terminal-specific features)
- Must handle terminals without true color (fallback to 256-color)
- Must not use GPU (TUI is CPU-only)
- Must be fully keyboard-navigable (accessibility requirement)
- Must not block on I/O (async event handling)

### Known Rust on ARM Issues
- Ensure all dependencies support ARM64 (check Cargo.lock)
- crossterm confirmed working on ARM Linux
- ratatui is architecture-independent
- tokio has excellent ARM support

---

## Implementation Plan

### Phase 1: Project Foundation (Days 1-2)
**Goal**: Set up Rust project structure and basic TUI rendering

**Tasks**:
1. Create Rust project: `cargo new --bin dgx-pixels-tui`
2. Add dependencies to Cargo.toml
3. Set up project structure: src/ui/, src/events.rs, src/state.rs
4. Implement basic event loop with crossterm
5. Create main.rs with terminal setup/teardown
6. Implement simple "Hello, DGX-Pixels!" screen
7. Test terminal resize handling
8. Set up logging with tracing
9. Write basic integration test

**Output**: Minimal TUI that initializes, renders, and exits cleanly

**Verification**:
```bash
cd /home/beengud/raibid-labs/dgx-pixels/rust
cargo build --release
./target/release/dgx-pixels-tui
# Press 'q' to exit - should exit cleanly
```

### Phase 2: Core UI Components (Days 3-5)
**Goal**: Implement all screen layouts and navigation

**Tasks**:
1. Create UI module structure: ui/app.rs, ui/layout.rs, ui/theme.rs
2. Implement generation screen (prompt input, settings, generate button)
3. Implement gallery screen (grid view, navigation, selection)
4. Implement settings screen (config options, save/load)
5. Implement help screen (keyboard shortcuts, usage guide)
6. Create navigation system (screen stack, breadcrumbs)
7. Add keyboard shortcuts (tab, arrow keys, vim keys, 'q', 'h')
8. Implement responsive layout (handle terminal resize)
9. Add color theme (use ratatui color palette)
10. Write unit tests for each screen

**Output**: Fully functional TUI with all screens navigable

**Verification**:
```bash
# Run TUI and navigate between screens
./target/release/dgx-pixels-tui

# Test keyboard navigation:
# - Press '1' to go to generation screen
# - Press '2' to go to gallery screen
# - Press '3' to go to settings screen
# - Press '?' or 'h' to show help
# - Press 'q' to quit
```

### Phase 3: State Management & Persistence (Days 6-7)
**Goal**: Add configuration persistence and state management

**Tasks**:
1. Implement state.rs with application state machine
2. Create config file structure (~/.config/dgx-pixels/config.toml)
3. Implement settings save/load with serde + toml
4. Add user preferences: theme, default settings, history
5. Implement navigation history (back button)
6. Add input validation for prompt fields
7. Add status bar with state indicators
8. Write integration tests for state transitions
9. Add benchmarks for rendering performance

**Output**: TUI with persistent settings and state management

**Verification**:
```bash
# Test settings persistence
./target/release/dgx-pixels-tui
# Change some settings, quit
# Restart - settings should be preserved
cat ~/.config/dgx-pixels/config.toml
```

### Phase 4: Testing, Optimization & Documentation (Day 8)
**Goal**: Comprehensive testing, performance optimization, documentation

**Tasks**:
1. Achieve ≥80% test coverage (cargo tarpaulin)
2. Run clippy and fix all warnings
3. Format with rustfmt
4. Run benchmarks and verify performance targets
5. Optimize binary size (strip, LTO)
6. Write rust/README.md (setup, usage, architecture)
7. Write docs/tui-architecture.md (technical deep-dive)
8. Generate rustdoc documentation
9. Create completion summary

**Output**: Production-ready TUI with full documentation

**Verification**:
```bash
# Run test suite
cargo test

# Check coverage
cargo tarpaulin --out Html

# Run clippy
cargo clippy -- -D warnings

# Run benchmarks
cargo bench

# Check binary size
ls -lh target/release/dgx-pixels-tui
# Should be ≤ 15MB

# Check performance
cargo run --release -- --benchmark
# Should report 60+ FPS
```

---

## Test-Driven Development (TDD)

### Test Requirements

**Unit Tests** (`rust/src/ui/*/tests.rs`):
- `test_generation_screen_render`: Verify generation screen renders without panic
- `test_gallery_screen_render`: Verify gallery screen renders empty and with items
- `test_settings_screen_render`: Verify settings screen renders and updates
- `test_help_screen_render`: Verify help screen renders keyboard shortcuts
- `test_navigation_forward`: Test navigation from screen A to screen B
- `test_navigation_back`: Test back button returns to previous screen
- `test_keyboard_input`: Test all keyboard shortcuts trigger correct actions
- `test_terminal_resize`: Test layout adapts to different terminal sizes
- `test_theme_application`: Test color theme applied correctly
- `test_input_validation`: Test prompt input validation (max length, special chars)

**Integration Tests** (`rust/tests/integration_test.rs`):
- `test_full_navigation_cycle`: Navigate through all screens and back
- `test_config_persistence`: Set config, restart app, verify config loaded
- `test_event_loop`: Test async event loop handles input correctly
- `test_graceful_shutdown`: Test Ctrl-C and 'q' exit cleanly

**Performance Tests** (`rust/benches/rendering_bench.rs`):
- `bench_generation_screen_render`: Measure render time for generation screen
- `bench_gallery_screen_render`: Measure render time for gallery with 100 items
- `bench_navigation_latency`: Measure screen transition time
- `bench_input_latency`: Measure key press to screen update time
- `bench_full_frame_time`: Measure complete render cycle (target: ≤ 16.6ms)

### Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_generation_screen_render

# Run integration tests only
cargo test --test integration_test

# Run with coverage
cargo tarpaulin --out Html
# Open tarpaulin-report.html

# Run benchmarks
cargo bench

# Run clippy (linting)
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Build and run
cargo run --release
```

### Expected Test Output
```
running 14 tests
test ui::screens::generation::tests::test_generation_screen_render ... ok
test ui::screens::gallery::tests::test_gallery_screen_render ... ok
test ui::screens::settings::tests::test_settings_screen_render ... ok
test ui::screens::help::tests::test_help_screen_render ... ok
test ui::app::tests::test_navigation_forward ... ok
test ui::app::tests::test_navigation_back ... ok
test events::tests::test_keyboard_input ... ok
test ui::layout::tests::test_terminal_resize ... ok
test ui::theme::tests::test_theme_application ... ok
test state::tests::test_input_validation ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured

running 4 integration tests
test test_full_navigation_cycle ... ok
test test_config_persistence ... ok
test test_event_loop ... ok
test test_graceful_shutdown ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured

Benchmarks:
bench_generation_screen_render: 8.2 ms (target: ≤ 16.6 ms) ✅
bench_gallery_screen_render: 12.4 ms (target: ≤ 16.6 ms) ✅
bench_navigation_latency: 3.1 ms (target: ≤ 50 ms) ✅
bench_input_latency: 18.3 ms (target: ≤ 50 ms) ✅
bench_full_frame_time: 14.7 ms (60 FPS) ✅

Test Coverage: 87% ✅
Clippy: 0 warnings ✅
Binary Size: 12.4 MB ✅

WS-08: ALL TESTS PASSING ✅
```

---

## Dependencies

### Blocked By
- **WS-01 (Hardware Baselines)**: Need hardware validation before testing on DGX-Spark

### Blocks
- **WS-09 (ZeroMQ IPC Layer)**: Needs TUI structure to integrate communication
- **WS-11 (Sixel Image Preview)**: Needs TUI framework to render images
- **WS-12 (Side-by-Side Model Comparison)**: Needs TUI screens for comparison UI

### Soft Dependencies
- **WS-04 (ComfyUI Setup)**: Helpful to have backend working for end-to-end testing, but not required for TUI core

---

## Known Issues & Risks

### Issue 1: Terminal Compatibility
**Problem**: Different terminals have varying support for true color, mouse events, Sixel
**Impact**: Medium (UI experience degradation)
**Mitigation**:
- Detect terminal capabilities at startup (query TERM environment variable)
- Fallback to 256-color if true color unavailable
- Gracefully disable mouse support if terminal doesn't support it
- Use terminfo database for capability detection
**Fallback**: Keyboard-only mode with 256-color palette (works on all terminals)
**Status**: Low risk - ratatui handles most compatibility issues

### Issue 2: Performance in tmux/screen
**Problem**: tmux/screen can add latency and reduce frame rate
**Impact**: Low (still usable, but not 60 FPS)
**Mitigation**:
- Document performance difference in tmux vs native terminal
- Recommend native terminal for best experience
- Test in tmux and ensure ≥ 30 FPS minimum
**Fallback**: Still functional, just not as smooth
**Status**: Acceptable - document limitation

### Issue 3: ARM Build Time
**Problem**: Rust compilation can be slow on ARM, especially for release builds
**Impact**: Low (development experience)
**Mitigation**:
- Use `cargo check` for fast iteration during development
- Use `cargo build` (debug) for testing, not release
- Only build release for benchmarks and final testing
- Cache dependencies in CI/CD
**Fallback**: Remote compilation on faster x86 machine with cross-compilation
**Status**: Manageable with good development practices

### Issue 4: Unicode Rendering
**Problem**: Some terminals may not render box-drawing characters correctly
**Impact**: Low (cosmetic)
**Mitigation**:
- Use ratatui's built-in border styles (safe across terminals)
- Test with common terminals: xterm, gnome-terminal, iTerm2, WezTerm
- Provide ASCII fallback for unsupported characters
**Fallback**: ASCII-only borders (less pretty but functional)
**Status**: Low risk - ratatui handles this well

---

## Integration Points

### With Other Workstreams
- **WS-09 (ZeroMQ IPC)**: TUI sends generation requests via ZeroMQ client
- **WS-10 (Python Backend)**: TUI receives status updates and results from backend
- **WS-11 (Sixel Preview)**: TUI embeds Sixel images in gallery screen
- **WS-12 (Model Comparison)**: TUI provides comparison screen layout

### With External Systems
- **Terminal Emulator**: Renders TUI using crossterm + ratatui
- **Config File**: Reads/writes `~/.config/dgx-pixels/config.toml`
- **Log Files**: Writes to `~/.local/share/dgx-pixels/logs/` (via tracing)

---

## Verification & Validation

### Verification Steps (Agent Self-Check)

```bash
# Step 1: Verify project structure exists
test -f /home/beengud/raibid-labs/dgx-pixels/rust/Cargo.toml && echo "✅ Cargo.toml exists"

# Step 2: Verify build succeeds
cd /home/beengud/raibid-labs/dgx-pixels/rust
cargo build --release && echo "✅ Release build succeeds"

# Step 3: Verify binary size
SIZE=$(stat -c%s target/release/dgx-pixels-tui)
[ $SIZE -le 15728640 ] && echo "✅ Binary size ≤ 15MB ($SIZE bytes)"

# Step 4: Verify tests pass
cargo test && echo "✅ All tests passing"

# Step 5: Verify test coverage
cargo tarpaulin --out Stdout | grep "^Coverage" | grep -E "(8[0-9]|9[0-9]|100)\." && echo "✅ Coverage ≥ 80%"

# Step 6: Verify clippy clean
cargo clippy -- -D warnings && echo "✅ Clippy passing"

# Step 7: Verify benchmarks meet targets
cargo bench | tee /tmp/bench_results.txt
grep "time:.*ms" /tmp/bench_results.txt && echo "✅ Benchmarks complete"

# Step 8: Verify documentation
cargo doc --no-deps && echo "✅ Documentation generated"

# Step 9: Manual TUI test
echo "⚠️  Manual test required: Run './target/release/dgx-pixels-tui' and verify all screens work"
```

### Acceptance Verification (Orchestrator)

```bash
# Run complete verification script
/home/beengud/raibid-labs/dgx-pixels/scripts/verify_ws_08.sh

# Expected output:
# ✅ Rust project structure exists
# ✅ Cargo.toml has all required dependencies
# ✅ All source files present (app.rs, screens/*.rs, events.rs, state.rs)
# ✅ Release build succeeds
# ✅ Binary size ≤ 15MB (actual: 12.4 MB)
# ✅ Unit tests passing (10/10)
# ✅ Integration tests passing (4/4)
# ✅ Test coverage ≥ 80% (actual: 87%)
# ✅ Clippy passing (0 warnings)
# ✅ Rustfmt formatted
# ✅ Benchmarks meet performance targets (60 FPS)
# ✅ Documentation complete (rustdoc + README)
# ✅ Manual TUI test: All screens navigable ✅
#
# WS-08: READY FOR COMPLETION ✅
```

---

## Success Metrics

**Completion Criteria**:
- All acceptance criteria met (functional, performance, quality)
- All tests passing (≥80% coverage)
- Performance targets achieved (60 FPS, ≤50ms latency)
- Binary size ≤ 15MB
- Documentation complete
- Completion summary created

**Quality Metrics**:
- Test coverage: ≥80% (measured with cargo tarpaulin)
- Clippy warnings: 0
- Rustfmt: Formatted
- Documentation: All public APIs documented (rustdoc)

**Performance Metrics**:
- Frame rate: 60+ FPS
- Input latency: ≤ 50ms
- Binary size: ≤ 15MB
- Memory usage: ≤ 50MB (TUI only)
- Startup time: ≤ 500ms

---

## Completion Checklist

Before marking WS-08 complete:

- [ ] Rust project created (`rust/Cargo.toml`)
- [ ] All dependencies added and building on ARM64
- [ ] UI module structure complete (`rust/src/ui/`)
- [ ] All screen layouts implemented (generation, gallery, settings, help)
- [ ] Event handling working (keyboard, async event loop)
- [ ] State management implemented (persistence to TOML)
- [ ] Navigation system working (screen stack, breadcrumbs)
- [ ] Responsive layout handling (terminal resize)
- [ ] Unit tests written and passing (≥10 tests)
- [ ] Integration tests written and passing (≥4 tests)
- [ ] Performance benchmarks run and passing (60 FPS)
- [ ] Test coverage ≥ 80%
- [ ] Clippy passing (0 warnings)
- [ ] Rustfmt formatted
- [ ] Binary size ≤ 15MB
- [ ] Documentation written (README, rustdoc, architecture doc)
- [ ] Manual testing complete (all screens navigable)
- [ ] Completion summary created (`docs/orchestration/workstreams/ws08-rust-tui-core/COMPLETION_SUMMARY.md`)
- [ ] GitHub issue PIXELS-023 closed with summary link

---

## Example Usage

```bash
# Build TUI
cd /home/beengud/raibid-labs/dgx-pixels/rust
cargo build --release

# Run TUI
./target/release/dgx-pixels-tui

# TUI starts with generation screen
# Keyboard shortcuts:
# - '1' or Tab: Generation screen
# - '2': Gallery screen
# - '3': Settings screen
# - '?' or 'h': Help screen
# - Arrow keys: Navigate within screen
# - Enter: Activate button/submit form
# - Esc or 'b': Back to previous screen
# - 'q' or Ctrl-C: Quit

# Check configuration
cat ~/.config/dgx-pixels/config.toml

# View logs
tail -f ~/.local/share/dgx-pixels/logs/dgx-pixels-tui.log
```

---

## Related Issues

- GitHub Issue: #PIXELS-023 (Rust TUI Core)
- GitHub Issue: #PIXELS-024 (Screen Layouts)
- GitHub Issue: #PIXELS-025 (Event Handling)
- GitHub Issue: #PIXELS-026 (State Management)
- Related Workstreams: WS-09, WS-11, WS-12
- Related Docs: `docs/08-tui-design.md`, `docs/07-rust-python-architecture.md`

---

## References

- Architecture: `docs/07-rust-python-architecture.md` (Rust TUI + Python Backend)
- TUI Design: `docs/08-tui-design.md` (Screen mockups and workflows)
- Roadmap: `docs/ROADMAP.md` (M2 - Interactive TUI)
- ratatui Documentation: https://ratatui.rs/
- crossterm Documentation: https://docs.rs/crossterm/
- Tokio Documentation: https://tokio.rs/

---

**Status**: Ready for agent spawn
**Last Updated**: 2025-11-10
**Estimated LOC**: 1200-1500 (Rust) + 200 (tests) + 100 (docs)
