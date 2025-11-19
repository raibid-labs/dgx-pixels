# TUI Testing Strategy for DGX-Pixels

## Executive Summary

This document outlines a comprehensive testing strategy for the DGX-Pixels ratatui TUI, identifies current implementation issues, and provides actionable fixes for sixel preview and gallery functionality.

**Status**: After deep research and code analysis, we've identified 6 critical issues affecting sixel preview and gallery functionality, and designed a 3-tier testing strategy to prevent future regressions.

---

## Identified Issues

### 1. **img2sixel Dependency Not Validated** ⚠️ CRITICAL

**Location**: `rust/src/sixel/image_renderer.rs:170`

**Problem**:
- The `ImageRenderer::render_to_buffer()` uses `Command::new("img2sixel")` without verifying it's installed
- If `img2sixel` is missing, sixel rendering fails silently
- Users get "Loading..." screens that never resolve

**Impact**: Sixel preview completely non-functional without `img2sixel`

**Fix**:
```rust
// Add to ImageRenderer::new()
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
                "img2sixel not found. Install with: apt install libsixel-bin"
            ));
        }
    }

    Ok(Self { _config: Config::default() })
}
```

**Alternative**: Fall back to `viuer` crate's built-in sixel support if `img2sixel` is unavailable.

---

### 2. **SixelImage Widget Bypasses Ratatui Buffer** ⚠️ HIGH

**Location**: `rust/src/ui/widgets/sixel_image.rs:24-41`

**Problem**:
- `SixelImage::render()` writes directly to `io::stdout()`
- This bypasses ratatui's `Buffer` system completely
- Causes:
  - Race conditions with ratatui's frame rendering
  - Incorrect positioning if terminal scrolls
  - Makes testing with `TestBackend` impossible (stdout isn't captured)
  - Flickering/artifacts when navigating between screens

**Current Implementation**:
```rust
impl<'a> Widget for SixelImage<'a> {
    fn render(self, area: Rect, _buf: &mut Buffer) {
        // ❌ Direct stdout write - bypasses ratatui!
        let mut stdout = io::stdout();
        let _ = write!(stdout, "\x1b[{};{}H{}", row, col, self.sixel_data);
        let _ = stdout.flush();
    }
}
```

**Why This Fails in TestBackend**:
- `TestBackend` doesn't render to a real terminal
- It renders to an in-memory `Buffer`
- Direct stdout writes are invisible to test assertions
- Integration tests can't verify sixel rendering

**Recommended Fix**: Use `ratatui-image` crate pattern

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub struct SixelImage<'a> {
    sixel_data: &'a str,
    placeholder: String, // For TestBackend
}

impl<'a> Widget for SixelImage<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Check if we're in a test environment
        if cfg!(test) || std::env::var("RATATUI_TEST_MODE").is_ok() {
            // Render placeholder for testing
            let placeholder = format!("[SIXEL {}x{}]", area.width, area.height);
            buf.set_string(
                area.x,
                area.y,
                &placeholder,
                ratatui::style::Style::default()
            );
            return;
        }

        // Production: Write sixel to alternate buffer
        // This requires coordination with Terminal::draw()
        // Store sixel data in a global queue to be flushed after draw()
        SIXEL_QUEUE.lock().unwrap().push(SixelDrawCommand {
            area,
            data: self.sixel_data.to_string(),
        });
    }
}
```

**Better Alternative**: Use `ratatui-image` crate

The `ratatui-image` crate (https://github.com/benjajaja/ratatui-image) already solves this problem:

```rust
use ratatui_image::{StatefulImage, protocol::StatefulProtocol};

// Use StatefulImage widget instead
let mut picker = Picker::from_termios().unwrap();
let dyn_img = image::open(path).unwrap();
let image = picker.new_resize_protocol(dyn_img);

// In render:
f.render_stateful_widget(image, area, &mut state);
```

**Benefits**:
- Works with TestBackend (renders placeholder)
- Handles terminal capability detection
- Supports multiple protocols (Sixel, Kitty, iTerm2)
- Tested in production

---

### 3. **No Error Feedback in Gallery Screen** ⚠️ MEDIUM

**Location**: `rust/src/ui/screens/gallery.rs:97`

**Problem**:
- When preview loading fails, user sees "Loading preview..." forever
- No timeout mechanism
- No error display if `img2sixel` fails or image is corrupt

**Current Flow**:
```rust
if let Some(preview_entry) = app.preview_manager.get_preview(selected_path) {
    render_sixel_large_preview(f, inner, &preview_entry.sixel_data, selected_path);
} else {
    // Request preview
    let _ = app.preview_manager.request_preview(selected_path.clone(), options);

    // Show loading - NEVER TIMES OUT! ❌
    render_loading(f, inner);
}
```

**Fix**:
```rust
use std::time::{Duration, Instant};

// Add to App state:
pub struct App {
    // ...
    preview_requests: HashMap<PathBuf, Instant>,
}

// In gallery render:
if let Some(preview_entry) = app.preview_manager.get_preview(selected_path) {
    render_sixel_large_preview(f, inner, &preview_entry.sixel_data, selected_path);
} else {
    let now = Instant::now();

    // Check if request timed out (5s timeout)
    if let Some(&request_time) = app.preview_requests.get(selected_path) {
        if now.duration_since(request_time) > Duration::from_secs(5) {
            render_preview_error(f, inner, "Preview timed out");
            return;
        }
    } else {
        // First request
        app.preview_requests.insert(selected_path.clone(), now);
        let _ = app.preview_manager.request_preview(selected_path.clone(), options);
    }

    render_loading(f, inner);
}

// In event loop, handle errors:
while let Some(preview_result) = app.preview_manager.try_recv_result() {
    if let Some(error) = preview_result.error {
        warn!("Preview failed: {}", error);
        // Store error in app state for display
        app.preview_errors.insert(preview_result.path, error);
    }
    app.preview_requests.remove(&preview_result.path);
    app.needs_redraw = true;
}
```

---

### 4. **TestBackend Incompatibility** ⚠️ HIGH

**Location**: All test files

**Problem**:
- Current tests use `TestBackend::new(80, 24)` which doesn't capture sixel output
- Sixel previews write to stdout, which is invisible to TestBackend
- No way to assert sixel rendering worked correctly

**Current Test**:
```rust
#[tokio::test]
async fn test_gallery_with_images() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.add_to_gallery(PathBuf::from("/test/img1.png"));

    // ❌ This renders, but sixel output is lost!
    let result = terminal.draw(|f| render(f, &app));
    assert!(result.is_ok());

    // ❌ Can't assert sixel was rendered!
    // terminal.backend().assert_buffer(...) only sees text cells
}
```

**Fix**: Multi-layered testing approach

```rust
// Layer 1: Unit tests for logic (no rendering)
#[test]
fn test_gallery_state_management() {
    let mut app = App::new();
    assert_eq!(app.gallery_images.len(), 0);

    app.add_to_gallery(PathBuf::from("/test/img1.png"));
    assert_eq!(app.gallery_images.len(), 1);
    assert_eq!(app.selected_gallery_index, 0);

    app.add_to_gallery(PathBuf::from("/test/img2.png"));
    app.next_gallery_image();
    assert_eq!(app.selected_gallery_index, 1);
}

// Layer 2: Render tests with placeholder (TestBackend)
#[test]
fn test_gallery_layout() {
    std::env::set_var("RATATUI_TEST_MODE", "1");

    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let mut app = App::new();
    app.add_to_gallery(PathBuf::from("/test/img1.png"));

    terminal.draw(|f| render(f, &app)).unwrap();

    // Assert layout structure
    let buffer = terminal.backend().buffer();
    assert!(buffer.content.iter().any(|cell|
        cell.symbol().contains("[SIXEL")
    ));
}

// Layer 3: Screenshot tests (real terminal)
#[test]
#[ignore] // Only run with --include-ignored on CI
fn test_gallery_sixel_rendering() {
    // Requires: xvfb-run on CI, kitty terminal installed
    let output = Command::new("xvfb-run")
        .args(&["-a", "kitty", "--", "cargo", "run", "--", "--test-mode"])
        .output()
        .unwrap();

    // Capture screenshot using kitty icat
    // Compare against golden image
    assert_eq!(output.status.code(), Some(0));
}
```

---

### 5. **No Cursor Positioning Validation** ⚠️ MEDIUM

**Location**: `rust/src/ui/widgets/sixel_image.rs:28`

**Problem**:
- ANSI cursor positioning assumes 1-indexed coordinates
- Calculation: `row = area.y + 1`, `col = area.x + 1`
- Doesn't account for:
  - Alternate screen mode offsets
  - Terminal scroll regions
  - Unicode wide characters in ratatui buffer

**Impact**: Sixel images appear at wrong positions or off-screen

**Fix**: Use crossterm's cursor positioning

```rust
use crossterm::cursor::{MoveTo, SavePosition, RestorePosition};
use crossterm::execute;

impl<'a> Widget for SixelImage<'a> {
    fn render(self, area: Rect, _buf: &mut Buffer) {
        let mut stdout = io::stdout();

        // Save current cursor position
        let _ = execute!(stdout, SavePosition);

        // Move to target position (crossterm handles coordinate conversion)
        let _ = execute!(stdout, MoveTo(area.x, area.y));

        // Clear area first
        for line in 0..area.height {
            let _ = execute!(stdout, MoveTo(area.x, area.y + line));
            let _ = write!(stdout, "{}", " ".repeat(area.width as usize));
        }

        // Write sixel data
        let _ = execute!(stdout, MoveTo(area.x, area.y));
        let _ = write!(stdout, "{}", self.sixel_data);

        // Restore cursor position
        let _ = execute!(stdout, RestorePosition);
        let _ = stdout.flush();
    }
}
```

---

### 6. **Missing Sixel Capability Detection** ⚠️ LOW

**Location**: `rust/src/sixel/terminal_detection.rs`

**Problem**:
- Terminal capability detection might not work in all environments
- No fallback strategy if detection fails

**Current Implementation**: Check this file

**Recommended Enhancement**:
```rust
pub fn detect_sixel_support() -> TerminalCapability {
    // 1. Check environment variables
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") || term.contains("wezterm") {
            return TerminalCapability::Sixel;
        }
    }

    // 2. Check terminal program
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        if term_program == "iTerm.app" {
            return TerminalCapability::Sixel;
        }
    }

    // 3. Query terminal using device attributes (DECRPM)
    // Send: CSI ? 80 ; 2 $ p
    // Expect: CSI ? 80 ; {0|1|2|3|4} $ y
    // where 1 = supported

    // 4. Test actual rendering (safest but slowest)
    // Try rendering a 1x1 pixel sixel and check for errors

    // 5. Default to TextOnly if all detection fails
    TerminalCapability::TextOnly
}
```

---

## Comprehensive Testing Strategy

### 3-Tier Testing Pyramid

```
                    ▲
                   / \
                  /   \
                 /     \        Layer 3: E2E Screenshot Tests (5%)
                /_______\       - Real terminals (kitty, wezterm)
               /         \      - Xvfb on CI
              /           \     - Visual regression
             /             \
            /               \   Layer 2: Integration Tests (25%)
           /_________________\  - TestBackend with placeholders
          /                   \ - Workflow testing
         /                     \- Mock ZMQ client
        /                       \
       /                         \ Layer 1: Unit Tests (70%)
      /___________________________\- Pure logic
                                   - No rendering
                                   - Fast, deterministic
```

### Test Distribution

1. **Unit Tests (70%)**: Fast, no rendering
   - State management (gallery navigation, job tracking)
   - Input handling (keyboard, text entry)
   - Data transformations (image paths, job status)
   - PreviewManager cache logic
   - RenderOptions calculations

2. **Integration Tests (25%)**: TestBackend with mocks
   - Screen rendering layouts
   - Navigation flows
   - Mock ZMQ responses
   - Preview request/response cycle
   - Snapshot testing with `insta`

3. **E2E Screenshot Tests (5%)**: Real terminals
   - Actual sixel rendering
   - Terminal compatibility matrix
   - Visual regression detection
   - Only on CI or with `--include-ignored`

---

## Recommended Testing Tools & Dependencies

### Add to `Cargo.toml`

```toml
[dev-dependencies]
# Snapshot testing (official ratatui support)
insta = { version = "1.40", features = ["redactions", "yaml"] }
cargo-insta = "1.40"

# Parameterized tests
rstest = "0.18"

# Property-based testing
proptest = "1.4"

# Async testing utilities
tokio-test = "0.4"

# Better assertions
pretty_assertions = "1.4"

# Mock building
mockall = "0.11"

# Coverage reporting
# cargo install cargo-llvm-cov
```

### CI Tools

```bash
# Install on CI
cargo install cargo-insta
cargo install cargo-llvm-cov

# For screenshot tests
apt-get install -y xvfb libsixel-bin kitty wezterm
```

---

## Implementation Plan

### Phase 1: Fix Critical Issues (2-3 days)

**Priority Order**:

1. **Fix img2sixel dependency** (rust/src/sixel/image_renderer.rs)
   - Add validation in `ImageRenderer::new()`
   - Provide clear error message if missing
   - Add fallback to viuer or ratatui-image

2. **Migrate to ratatui-image** (rust/src/ui/widgets/sixel_image.rs)
   - Replace custom `SixelImage` with `ratatui-image`
   - Removes stdout bypass issue
   - Adds TestBackend compatibility
   - PR: https://github.com/benjajaja/ratatui-image

3. **Add error handling to gallery** (rust/src/ui/screens/gallery.rs)
   - Add preview timeout (5s)
   - Display error messages
   - Track request times

### Phase 2: Add Unit Tests (3-4 days)

**Test Coverage Targets**:
- `src/app.rs`: State management → 90% coverage
- `src/sixel/preview_manager.rs`: Cache logic → 85% coverage
- `src/ui/screens/gallery.rs`: Navigation logic → 80% coverage

**Example Unit Tests**:

```rust
// tests/unit/gallery_state_test.rs
use dgx_pixels_tui::app::App;
use std::path::PathBuf;

#[test]
fn test_gallery_empty_state() {
    let app = App::new();
    assert!(app.gallery_images.is_empty());
    assert_eq!(app.selected_gallery_index, 0);
    assert!(app.selected_gallery_image().is_none());
}

#[test]
fn test_add_to_gallery() {
    let mut app = App::new();
    let path1 = PathBuf::from("/test/image1.png");
    let path2 = PathBuf::from("/test/image2.png");

    app.add_to_gallery(path1.clone());
    assert_eq!(app.gallery_images.len(), 1);
    assert_eq!(app.selected_gallery_image(), Some(&path1));

    app.add_to_gallery(path2.clone());
    assert_eq!(app.gallery_images.len(), 2);
}

#[rstest]
#[case(0, 0, 0)] // Empty gallery
#[case(1, 0, 0)] // Single image
#[case(5, 0, 1)] // Navigate forward
#[case(5, 4, 4)] // Navigate at end (wraps or stops?)
fn test_gallery_navigation(
    #[case] num_images: usize,
    #[case] start_index: usize,
    #[case] expected_after_next: usize,
) {
    let mut app = App::new();

    for i in 0..num_images {
        app.add_to_gallery(PathBuf::from(format!("/test/img{}.png", i)));
    }

    app.selected_gallery_index = start_index;
    app.next_gallery_image();

    assert_eq!(app.selected_gallery_index, expected_after_next);
}
```

### Phase 3: Add Integration Tests (2-3 days)

**Snapshot Testing Example**:

```rust
// tests/integration/gallery_rendering_test.rs
use dgx_pixels_tui::ui::screens::gallery;
use dgx_pixels_tui::app::App;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

#[test]
fn test_gallery_empty_layout() {
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let app = App::new();

    terminal.draw(|f| gallery::render(f, &app)).unwrap();

    // Snapshot the terminal output
    insta::assert_snapshot!(terminal.backend().buffer());
}

#[test]
fn test_gallery_with_images_layout() {
    std::env::set_var("RATATUI_TEST_MODE", "1");

    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let mut app = App::new();

    app.add_to_gallery(PathBuf::from("/test/sprite_001.png"));
    app.add_to_gallery(PathBuf::from("/test/sprite_002.png"));
    app.add_to_gallery(PathBuf::from("/test/sprite_003.png"));

    terminal.draw(|f| gallery::render(f, &app)).unwrap();

    insta::assert_snapshot!(terminal.backend().buffer());
}
```

**Mock ZMQ Client**:

```rust
// tests/helpers/mock_zmq.rs
use dgx_pixels_tui::messages::{Request, Response, ProgressUpdate};
use dgx_pixels_tui::zmq_client::ZmqClient;
use std::collections::VecDeque;

pub struct MockZmqClient {
    responses: VecDeque<Response>,
    updates: VecDeque<ProgressUpdate>,
}

impl MockZmqClient {
    pub fn new() -> Self {
        Self {
            responses: VecDeque::new(),
            updates: VecDeque::new(),
        }
    }

    pub fn queue_response(&mut self, response: Response) {
        self.responses.push_back(response);
    }

    pub fn queue_update(&mut self, update: ProgressUpdate) {
        self.updates.push_back(update);
    }
}

// Use trait objects for dependency injection
pub trait ZmqClientTrait {
    fn send_request(&mut self, request: Request) -> anyhow::Result<()>;
    fn try_recv_response(&self) -> Option<Response>;
    fn try_recv_update(&self) -> Option<ProgressUpdate>;
}

// Implement for both real and mock clients
impl ZmqClientTrait for MockZmqClient {
    fn send_request(&mut self, _request: Request) -> anyhow::Result<()> {
        Ok(())
    }

    fn try_recv_response(&self) -> Option<Response> {
        self.responses.pop_front()
    }

    fn try_recv_update(&self) -> Option<ProgressUpdate> {
        self.updates.pop_front()
    }
}
```

### Phase 4: Add E2E Tests (1-2 days)

**Screenshot Test Infrastructure**:

```bash
# scripts/screenshot_test.sh
#!/bin/bash
set -e

export DISPLAY=:99

# Start Xvfb
Xvfb :99 -screen 0 1920x1080x24 &
XVFB_PID=$!

sleep 2

# Run TUI and capture screenshot
kitty --config scripts/kitty.conf --session scripts/gallery_test_session.txt \
    --hold --title "DGX-Pixels Gallery Test" \
    cargo run --release -- --test-mode &
KITTY_PID=$!

sleep 5

# Capture screenshot using ImageMagick
import -window "DGX-Pixels Gallery Test" tests/screenshots/gallery_current.png

# Compare with golden image
compare -metric RMSE \
    tests/screenshots/gallery_golden.png \
    tests/screenshots/gallery_current.png \
    tests/screenshots/gallery_diff.png

# Cleanup
kill $KITTY_PID
kill $XVFB_PID
```

**CI Integration (GitHub Actions)**:

```yaml
# .github/workflows/test.yml
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
          sudo apt-get install -y libsixel-bin xvfb kitty wezterm
          cargo install cargo-llvm-cov cargo-insta

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Verify snapshots
        run: cargo insta test --check

      - name: Run screenshot tests
        run: |
          xvfb-run -a bash scripts/screenshot_test.sh

      - name: Generate coverage
        run: |
          cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./lcov.info
```

---

## Testing Checklist

### Before Committing Code

- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test '*'`
- [ ] Snapshots are reviewed: `cargo insta review`
- [ ] Code coverage > 80%: `cargo llvm-cov --html`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt --check`

### Before Merging PRFunction tests on real hardware (DGX-Spark with sixel terminal)
- [ ] Screenshot tests pass: `bash scripts/screenshot_test.sh`
- [ ] Manual smoke test in kitty terminal
- [ ] Manual smoke test in wezterm terminal
- [ ] Verify error messages are user-friendly

---

## Known Limitations

1. **TestBackend Can't Verify Sixel Encoding**
   - TestBackend only captures text cells, not graphics
   - Solution: Use placeholders for test rendering, screenshot tests for visual validation

2. **No Color Assertions in TestBackend** (ratatui#1402)
   - Cell colors can't be asserted in current TestBackend
   - Workaround: Manual buffer inspection with `cell.fg()`, `cell.bg()`
   - PR #2099 provides fix: https://github.com/ratatui/ratatui/pull/2099

3. **Screenshot Tests Require Real Terminal**
   - E2E tests need kitty/wezterm installed
   - CI must use Xvfb
   - Tests are slow (5-10s per screenshot)
   - Solution: Mark with `#[ignore]`, run with `--include-ignored` on CI only

4. **Async Testing Complexity**
   - PreviewManager uses async workers
   - Tokio runtime required in tests
   - Use `tokio::test` and `tokio-test` utilities

---

## References

- **Ratatui Testing Guide**: https://ratatui.rs/recipes/testing/
- **Ratatui Snapshot Testing**: https://ratatui.rs/recipes/testing/snapshots/
- **ratatui-image crate**: https://github.com/benjajaja/ratatui-image
- **Insta Snapshot Testing**: https://insta.rs/
- **RustLab 2024 Workshop**: https://github.com/ratatui/awesome-ratatui
- **dgx-pixels Research**: `/docs/research/ratatui-testing-research.md`

---

## Next Steps

1. **Immediate (Today)**:
   - [ ] Fix img2sixel validation
   - [ ] Migrate to ratatui-image
   - [ ] Add gallery error handling

2. **This Week**:
   - [ ] Add unit tests (70% coverage)
   - [ ] Set up snapshot testing with insta
   - [ ] Create mock ZMQ client

3. **Next Week**:
   - [ ] Integration tests with TestBackend
   - [ ] Screenshot test infrastructure
   - [ ] CI pipeline with coverage reporting

4. **Following Sprint**:
   - [ ] Property-based testing for edge cases
   - [ ] Performance benchmarks
   - [ ] Accessibility testing (screen readers)

---

**Document Version**: 1.0
**Last Updated**: 2025-11-19
**Author**: Claude Code
**Status**: Ready for Implementation
