# Testing Implementation - Next Steps

**Date**: 2025-11-19
**Status**: Testing Infrastructure Complete - Ready for Implementation
**Estimated Time to Working Tests**: 1-2 hours
**Estimated Time to Fix Sixel Preview**: 2-3 days

---

## 🎯 Immediate Actions (Today - 1-2 hours)

### Step 1: Fix Gallery State Tests (30 minutes)

The tests are written but need async annotations since `App::new()` spawns tokio tasks.

**File**: `rust/tests/gallery_state_test.rs`

**Change Required**:
```rust
// Change all tests from this:
#[test]
fn test_gallery_empty_state() {
    let app = App::new();
    // ...
}

// To this:
#[tokio::test]
async fn test_gallery_empty_state() {
    let app = App::new();
    // ...
}
```

**Apply to all 23 tests in the file**. Use search and replace:
- Find: `#[test]\nfn test_`
- Replace: `#[tokio::test]\nasync fn test_`

**Verify**:
```bash
cargo test --test gallery_state_test
# Should see: test result: ok. 23 passed
```

### Step 2: Verify Test Infrastructure (10 minutes)

Check that all test helpers compile:

```bash
# Test the helper modules
cargo test --lib helpers

# Run all tests
cargo test

# Should see helpers tests passing:
# - test_mock_zmq_client
# - test_sent_requests_tracking
# - test_create_test_terminal
# - test_test_mode_guard
```

### Step 3: Install img2sixel (5 minutes)

On DGX-Spark or your development machine:

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y libsixel-bin

# Verify installation
which img2sixel
img2sixel --version
```

This will immediately fix silent failures in sixel rendering.

---

## 🔧 Critical Fixes (This Week - 2-3 days)

### Day 1: Fix img2sixel Validation & Error Handling

#### Task 1.1: Validate img2sixel Installation (1 hour)

**File**: `rust/src/sixel/image_renderer.rs`

**Current Code** (line 42):
```rust
pub fn new() -> Self {
    let config = Config::default();
    Self { _config: config }
}
```

**New Code**:
```rust
pub fn new() -> Result<Self> {
    // Validate img2sixel is available
    let check = Command::new("img2sixel")
        .arg("--version")
        .output();

    match check {
        Ok(output) if output.status.success() => {
            debug!("img2sixel found: {:?}",
                String::from_utf8_lossy(&output.stdout));
        }
        _ => {
            return Err(anyhow::anyhow!(
                "img2sixel not found in PATH. Install with: apt install libsixel-bin"
            ));
        }
    }

    Ok(Self { _config: Config::default() })
}
```

**Also Update** (line 80 in `preview_manager.rs`):
```rust
// Change from:
let renderer = Arc::new(ImageRenderer::new());

// To:
let renderer = Arc::new(ImageRenderer::new()
    .context("Failed to initialize image renderer")?);
```

**Update All Call Sites**:
- `preview_manager.rs:84`
- Any other places that call `ImageRenderer::new()`

**Test**:
```bash
# Without img2sixel installed
cargo run
# Should show clear error: "img2sixel not found in PATH..."

# With img2sixel installed
cargo run
# Should start normally
```

#### Task 1.2: Add Gallery Error Handling (2 hours)

**File**: `rust/src/app.rs`

**Add to App struct** (around line 100):
```rust
/// Preview request timestamps for timeout detection
pub preview_requests: std::collections::HashMap<PathBuf, Instant>,

/// Preview errors for display
pub preview_errors: std::collections::HashMap<PathBuf, String>,
```

**Update App::new()** (around line 140):
```rust
preview_requests: std::collections::HashMap::new(),
preview_errors: std::collections::HashMap::new(),
```

**File**: `rust/src/ui/screens/gallery.rs`

**Update render_main_preview()** (line 66):
```rust
fn render_main_preview(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = create_block(" Preview ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    debug!("render_main_preview called.");
    debug!("Selected gallery image: {:?}", app.selected_gallery_image());
    debug!("Terminal capability: {:?}", app.terminal_capability);

    if let Some(selected_path) = app.selected_gallery_image() {
        // Check if there's an error for this path
        if let Some(error) = app.preview_errors.get(selected_path) {
            render_preview_error(f, inner, error);
            return;
        }

        match app.terminal_capability {
            TerminalCapability::Sixel => {
                debug!("Terminal capability is Sixel.");

                if let Some(preview_entry) = app.preview_manager.get_preview(selected_path) {
                    debug!("Preview found in cache for path: {:?}", selected_path);
                    render_sixel_large_preview(f, inner, &preview_entry.sixel_data, selected_path);
                } else {
                    // Check if request timed out (5 seconds)
                    let now = Instant::now();
                    let timed_out = app.preview_requests
                        .get(selected_path)
                        .map(|&request_time| now.duration_since(request_time) > Duration::from_secs(5))
                        .unwrap_or(false);

                    if timed_out {
                        render_preview_error(f, inner, "Preview timed out (5s)");
                        return;
                    }

                    // Request preview if not already requested
                    if !app.preview_requests.contains_key(selected_path) {
                        debug!("First request for preview: {:?}", selected_path);
                        let options = RenderOptions {
                            width: inner.width.saturating_sub(4),
                            height: inner.height.saturating_sub(4),
                            preserve_aspect: true,
                            high_quality: true,
                        };

                        let _ = app.preview_manager.request_preview(selected_path.clone(), options);
                    }

                    // Show loading
                    render_loading(f, inner);
                }
            }
            TerminalCapability::TextOnly => {
                debug!("Terminal capability is TextOnly.");
                render_text_only_info(f, inner, selected_path);
            }
        }
    } else {
        debug!("No image selected in gallery.");
        render_no_selection(f, inner);
    }
}

// Add new function at end of file:
fn render_preview_error(f: &mut Frame, area: ratatui::layout::Rect, error: &str) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("❌ Preview Failed", Theme::error())),
        Line::from(""),
        Line::from(Span::styled(error, Theme::muted())),
        Line::from(""),
        Line::from("Press 'r' to retry"),
    ];

    let paragraph = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(paragraph, area);
}
```

**File**: `rust/src/lib.rs` (event loop)

**Update preview polling** (around line 154):
```rust
// Process preview results from async worker
while let Some(preview_result) = app.preview_manager.try_recv_result() {
    // Remove from pending requests
    app.preview_requests.remove(&preview_result.path);

    if let Some(error) = preview_result.error {
        warn!("Preview failed for {:?}: {}", preview_result.path, error);
        app.preview_errors.insert(preview_result.path.clone(), error);
        app.needs_redraw = true;
    } else if preview_result.entry.is_some() {
        info!("Preview ready: {:?}", preview_result.path);
        // Clear any previous error
        app.preview_errors.remove(&preview_result.path);
        app.needs_redraw = true;
    }
}

// In gallery screen input handler, add retry on 'r' key
// File: rust/src/bevy_app/systems/input/screens/gallery.rs
// Add case for 'r' to clear error and retry preview request
```

**Track Preview Requests**:

Add to event handler when preview is requested:
```rust
app.preview_requests.insert(path.clone(), Instant::now());
```

**Test**:
```bash
# Test with corrupt image
cargo run
# Navigate to gallery
# Select corrupt image
# Should show error after timeout
# Press 'r' to retry
```

---

### Day 2: Migrate to ratatui-image (RECOMMENDED)

This is the **best long-term solution** - it fixes multiple issues simultaneously.

#### Task 2.1: Add Dependency (5 minutes)

```bash
cargo add ratatui-image
```

Or manually add to `Cargo.toml`:
```toml
[dependencies]
ratatui-image = "1.0"
```

#### Task 2.2: Update SixelImage Widget (1 hour)

**File**: `rust/src/ui/widgets/sixel_image.rs`

**Replace entire file**:
```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Image};
use std::path::Path;
use tracing::debug;

/// Sixel image widget using ratatui-image
pub struct SixelImage {
    path: std::path::PathBuf,
}

impl SixelImage {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn from_path(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}

impl Widget for SixelImage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // In test mode, render placeholder
        if cfg!(test) || std::env::var("RATATUI_TEST_MODE").is_ok() {
            let placeholder = format!("[SIXEL {}x{}]", area.width, area.height);
            buf.set_string(
                area.x,
                area.y,
                &placeholder,
                ratatui::style::Style::default(),
            );
            return;
        }

        // Production: Use ratatui-image
        // Note: This requires the image to be loaded and protocol selected
        // For best integration, use StatefulWidget pattern with protocol state
        debug!("SixelImage widget rendering for: {:?}", self.path);

        // TODO: Integrate with PreviewManager to get cached protocol state
        // For now, just render placeholder in production too
        let placeholder = format!("[Image: {:?}]", self.path.file_name().unwrap_or_default());
        buf.set_string(
            area.x,
            area.y,
            &placeholder,
            ratatui::style::Style::default(),
        );
    }
}

// Better approach: Use StatefulWidget
pub struct StatefulSixelImage;

pub struct SixelImageState {
    pub protocol: Box<dyn StatefulProtocol>,
}

impl StatefulWidget for StatefulSixelImage {
    type State = SixelImageState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.protocol.render(area, buf);
    }
}
```

**Better Integration** (full implementation):

Create `rust/src/ui/widgets/image_preview.rs`:
```rust
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::path::Path;

pub struct ImagePreview {
    picker: Picker,
}

impl ImagePreview {
    pub fn new() -> Result<Self> {
        let picker = Picker::from_termios()?;
        Ok(Self { picker })
    }

    pub fn load_image(&mut self, path: impl AsRef<Path>) -> Result<Box<dyn StatefulProtocol>> {
        let dyn_img = image::open(path.as_ref())?;
        let protocol = self.picker.new_resize_protocol(dyn_img);
        Ok(protocol)
    }
}
```

**Update PreviewManager to use ratatui-image**:
- Store `Box<dyn StatefulProtocol>` instead of sixel string
- Use `Picker` to select best protocol (Sixel, Kitty, iTerm2, etc.)
- Render with `StatefulWidget` pattern

#### Task 2.3: Update Gallery Screen (30 minutes)

**File**: `rust/src/ui/screens/gallery.rs`

Replace sixel rendering with ratatui-image protocol rendering.

#### Task 2.4: Test (30 minutes)

```bash
# Run in different terminals
kitty -- cargo run    # Should use Kitty protocol
wezterm -- cargo run  # Should use Sixel
xterm -- cargo run    # Should use Sixel
alacritty -- cargo run # Should fall back to ASCII/blocks

# Run tests
cargo test
# TestBackend should render placeholders
```

---

### Day 3: Testing & Verification

#### Task 3.1: Write Preview Manager Tests (2 hours)

**File**: `rust/tests/preview_manager_test.rs`

```rust
use dgx_pixels_tui::sixel::{PreviewManager, RenderOptions};
use std::path::PathBuf;
use tempfile::tempdir;

mod helpers;
use helpers::*;

#[tokio::test]
async fn test_preview_manager_creation() {
    let manager = PreviewManager::new();
    let stats = manager.cache_stats();
    assert_eq!(stats.entries, 0);
    assert_eq!(stats.size_bytes, 0);
}

#[tokio::test]
async fn test_request_preview() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions {
        width: 40,
        height: 20,
        preserve_aspect: true,
        high_quality: false,
    };

    // Request preview
    let result = manager.request_preview(paths[0].clone(), options);
    assert!(result.is_ok());

    // Wait for worker to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Should have result
    let preview_result = manager.try_recv_result();
    assert!(preview_result.is_some());
}

// Add 10-15 more tests covering:
// - Cache hits/misses
// - LRU eviction
// - Concurrent requests
// - Error handling
// - Cache statistics
```

#### Task 3.2: Add Snapshot Tests (1 hour)

**File**: `rust/tests/snapshot_gallery_test.rs`

```rust
use dgx_pixels_tui::ui::screens::gallery;
use dgx_pixels_tui::app::App;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

mod helpers;
use helpers::*;

#[tokio::test]
async fn test_gallery_empty_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let app = App::new();

    terminal.draw(|f| gallery::render(f, &app)).unwrap();

    insta::assert_snapshot!(terminal.backend().buffer());
}

#[tokio::test]
async fn test_gallery_with_images_snapshot() {
    let _guard = TestModeGuard::new();
    let mut terminal = create_standard_terminal();
    let mut app = App::new();
    let (_dir, paths) = create_test_gallery();

    for path in &paths[0..3] {
        app.add_to_gallery(path.clone());
    }

    terminal.draw(|f| gallery::render(f, &app)).unwrap();

    insta::assert_snapshot!(terminal.backend().buffer());
}

// Add 5-10 more snapshot tests
```

**Review Snapshots**:
```bash
cargo test
cargo insta review
# Review each snapshot, accept or reject
```

#### Task 3.3: Run Full Test Suite (30 minutes)

```bash
# Run all tests
cargo test

# Generate coverage report
cargo install cargo-llvm-cov
cargo llvm-cov --html --open

# Check coverage is > 70%
# Identify gaps and add tests
```

---

## 📅 Week 2: Integration & CI (Optional but Recommended)

### Task 4.1: Integration Tests with Mock ZMQ (4 hours)

Create end-to-end workflow tests:
- Submit generation request
- Receive progress updates
- Handle job completion
- Add to gallery
- Display preview

### Task 4.2: CI Pipeline (2 hours)

**File**: `.github/workflows/test.yml`

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libsixel-bin
          cargo install cargo-llvm-cov cargo-insta

      - name: Run tests
        run: cargo test --all

      - name: Verify snapshots
        run: cargo insta test --check

      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./lcov.info
```

---

## 🎯 Success Criteria

### Immediate (End of Today)
- [ ] All gallery state tests passing (23/23)
- [ ] Helper modules compile and test
- [ ] img2sixel installed and verified

### End of Week
- [ ] img2sixel validation prevents silent failures
- [ ] Gallery shows error messages on preview failure
- [ ] Preview requests timeout after 5 seconds
- [ ] Either: ratatui-image integrated OR direct sixel rendering working in TestBackend
- [ ] Preview manager tests written (85% coverage)
- [ ] Snapshot tests created for main screens
- [ ] Overall test coverage > 70%

### Quality Gates
- [ ] `cargo test` passes all tests
- [ ] `cargo clippy` shows no warnings
- [ ] `cargo fmt --check` passes
- [ ] `cargo llvm-cov` shows > 70% coverage
- [ ] `cargo insta test --check` passes (no pending snapshots)

---

## 🐛 Troubleshooting

### Tests Fail with "no reactor running"
**Solution**: Add `#[tokio::test]` and `async` to test functions

### img2sixel not found
**Solution**:
```bash
sudo apt-get install libsixel-bin
# or
brew install libsixel  # macOS
```

### Snapshots don't match
**Solution**:
```bash
cargo insta review
# Review and accept/reject each change
```

### Coverage too low
**Solution**: Identify untested code with:
```bash
cargo llvm-cov --html --open
# Browser opens showing coverage heatmap
```

### SixelImage still flickers
**Solution**: Migrate to ratatui-image (Day 2 task)

### Gallery still shows "Loading..." forever
**Solution**: Implement timeout (Day 1, Task 1.2)

---

## 📚 Reference Documents

- **Testing Strategy**: `/docs/testing/tui-testing-strategy.md` (comprehensive 3500+ line guide)
- **Research Findings**: `/docs/research/ratatui-testing-research.md` (2000+ line research doc)
- **Summary**: `/docs/testing/TESTING_SUMMARY.md` (executive summary)
- **This File**: `/docs/testing/NEXT_STEPS.md` (step-by-step implementation)

---

## 🚀 Quick Commands

```bash
# Fix tests (add tokio::test)
# Edit rust/tests/gallery_state_test.rs manually

# Run tests
cargo test --test gallery_state_test

# Install img2sixel
sudo apt-get install -y libsixel-bin

# Validate installation
which img2sixel

# Run full test suite
cargo test

# Generate coverage
cargo install cargo-llvm-cov
cargo llvm-cov --html --open

# Review snapshots (after adding insta tests)
cargo insta review

# Format code
cargo fmt

# Check for issues
cargo clippy

# Build
cargo build --release

# Run TUI
cargo run
```

---

**Last Updated**: 2025-11-19
**Next Review**: After completing Day 1 tasks
**Questions**: See `/docs/testing/tui-testing-strategy.md` for detailed explanations
