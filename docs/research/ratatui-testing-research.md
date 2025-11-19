# Ratatui TUI Testing Research

**Research Date:** 2025-11-19
**Purpose:** Identify best practices, tools, and techniques for testing ratatui TUI applications in Rust
**Project:** dgx-pixels (Rust TUI + Python Backend)

---

## Executive Summary

Ratatui provides comprehensive testing capabilities through its `TestBackend` and integrates well with the Rust testing ecosystem. The project maintains 90% test coverage and has established patterns for unit testing, snapshot testing, and integration testing.

**Key Findings:**
- **TestBackend**: Built-in mock backend for headless testing (no terminal required)
- **Snapshot Testing**: Official support via `insta` crate with cargo-insta workflow
- **Event Testing**: Separate event handlers from rendering for easy keyboard/mouse testing
- **Coverage**: cargo-llvm-cov integration with codecov for CI/CD
- **Limitation**: Color assertion not yet fully supported (PR #2099 in progress)

---

## 1. Official Ratatui Testing Documentation

### Primary Resources
- **Official Snapshot Testing Guide**: https://ratatui.rs/recipes/testing/snapshots/
- **Contributing Guide**: https://github.com/ratatui/ratatui/blob/main/CONTRIBUTING.md
- **Example Tests**: `ratatui/tests/widgets_block.rs` (canonical TestBackend example)
- **Counter App Tutorial**: https://ratatui.rs/tutorials/counter-app/basic-app/ (complete testing examples)

### Testing Philosophy
From the CONTRIBUTING.md:
- "Write unit tests for all new or modified code"
- "Generally prefer to write unit tests and doc tests directly in the code file being tested rather than integration tests in the `tests/` folder"
- "One of the most valuable test you can write for Ratatui is a test against the TestBackend"
- Test coverage tracked via codecov (90% coverage as of 2024)

---

## 2. TestBackend - Core Testing Infrastructure

### What is TestBackend?

TestBackend is a Backend implementation specifically designed for testing that renders to a memory buffer instead of an actual terminal. This enables headless testing in CI/CD pipelines.

### Basic Usage Pattern

```rust
use ratatui::{backend::TestBackend, Terminal, Frame};

#[test]
fn test_widget_rendering() {
    let backend = TestBackend::new(80, 20);  // width, height
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        frame.render_widget(&app, frame.area());
    }).unwrap();

    // Assert the buffer contents
    let expected = Buffer::with_lines(vec![
        "┌Title─────┐",
        "│          │",
        "└──────────┘",
    ]);

    terminal.backend().assert_buffer(&expected);
}
```

### Key Features

1. **Headless Rendering**: No terminal required, runs in CI/CD
2. **Memory Buffer**: Captures all output for assertion
3. **Deterministic**: Same input always produces same output
4. **Buffer Comparison**: Multiple assertion methods available

### Assertion Methods

```rust
// Method 1: Direct buffer comparison
assert_eq!(terminal.backend().buffer(), &expected);

// Method 2: assert_buffer (recommended)
terminal.backend().assert_buffer(&expected);

// Method 3: assert_buffer_lines (line-by-line comparison)
terminal.backend().assert_buffer_lines(vec!["line1", "line2"]);
```

### Complete Example from widgets_block.rs

```rust
#[test]
fn widgets_block_renders() {
    let backend = TestBackend::new(10, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let block = Block::bordered()
        .title(Span::styled("Title", Style::default().fg(Color::LightBlue)));

    terminal
        .draw(|frame| frame.render_widget(block, Rect::new(0, 0, 8, 8)))
        .unwrap();

    let mut expected = Buffer::with_lines([
        "┌Title─┐ ",
        "│      │ ",
        "│      │ ",
        "│      │ ",
        "│      │ ",
        "│      │ ",
        "│      │ ",
        "└──────┘ ",
        "         ",
        "         ",
    ]);

    // Set styles for specific regions
    for x in 1..=5 {
        expected[(x, 0)].set_fg(Color::LightBlue);
    }

    terminal.backend().assert_buffer(&expected);
}
```

### Recent Changes (v0.28.0+)

- **Error Handling**: Now uses `core::convert::Infallible` instead of `std::io::Error`
- **Removed Fields**: `height` and `width` fields removed (use buffer dimensions instead)
- **Buffer Access**: Access via `terminal.backend().buffer()`

---

## 3. Snapshot Testing with Insta

### Why Snapshot Testing?

Snapshot testing eliminates tedious manual assertions by capturing reference outputs once, then automatically comparing future test runs against these snapshots. Perfect for TUIs where visual layout matters.

### Setup

```bash
# Install CLI tool
cargo install cargo-insta

# Add as dev dependency
cargo add insta --dev
```

### Implementation

```rust
#[cfg(test)]
mod tests {
    use super::App;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_render_app() {
        let app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }
}
```

### Snapshot Management Workflow

```bash
# Run tests (creates/compares snapshots)
cargo test

# Review and approve changes
cargo insta review

# Accept all pending snapshots
cargo insta accept

# Reject all pending snapshots
cargo insta reject
```

### Snapshot Files

Snapshots are stored in `snapshots/` directory:

```
snapshots/
├── module_name__test_name.snap
└── module_name__test_name_2.snap
```

Each snapshot file contains:
```
---
source: tests/my_test.rs
expression: terminal.backend()
---
┌Title──────────┐
│               │
└───────────────┘
```

### Best Practices

1. **Consistent Dimensions**: Use fixed terminal size (e.g., 80x20) for reproducibility
2. **Review Changes**: Always review snapshots after UI modifications
3. **Commit Snapshots**: Include snapshot files in version control
4. **CI Integration**: Snapshots fail in CI if changes detected

### Limitations

**Color assertions not supported** (as of November 2024)
- Display implementation doesn't capture style information
- Related issue: https://github.com/ratatui/ratatui/issues/1402
- Workaround: Test colors separately with buffer assertions
- PR #2099 provides implementation for color testing

---

## 4. Testing Keyboard and Mouse Events

### Architecture for Testable Event Handling

Separate event handling logic from terminal I/O by creating dedicated handler methods:

```rust
impl App {
    // Separate handler method - easy to test!
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Right => self.counter += 1,
            KeyCode::Left => self.counter = self.counter.saturating_sub(1),
            _ => {}
        }
    }
}
```

### Unit Test for Event Handling

```rust
#[test]
fn handle_key_event() -> io::Result<()> {
    let mut app = App::default();

    // Test increment
    app.handle_key_event(KeyCode::Right.into());
    assert_eq!(app.counter, 1);

    // Test decrement
    app.handle_key_event(KeyCode::Left.into());
    assert_eq!(app.counter, 0);

    // Test quit
    let mut app = App::default();
    app.handle_key_event(KeyCode::Char('q').into());
    assert!(app.exit);

    Ok(())
}
```

### Key Insight from Tutorial

"Splitting the keyboard event handling out to a separate function makes it easy to test the application without having to emulate the terminal."

### Crossterm Event Testing

For more complex event scenarios:

```rust
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_ctrl_c_quit() {
    let mut app = App::default();
    let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

    app.handle_key_event(event);
    assert!(app.should_quit);
}
```

### Mouse Event Testing

```rust
use crossterm::event::{MouseEvent, MouseEventKind};

#[test]
fn test_mouse_click() {
    let mut app = App::default();
    let event = MouseEvent {
        kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: 10,
        row: 5,
        modifiers: KeyModifiers::empty(),
    };

    app.handle_mouse_event(event);
    assert_eq!(app.selected_position, (10, 5));
}
```

### Async Event Testing

For async applications using tokio:

```rust
#[tokio::test]
async fn test_async_event_handling() {
    let mut app = App::default();
    let event = KeyCode::Enter.into();

    app.handle_key_event_async(event).await;
    assert!(app.data_loaded);
}
```

---

## 5. Integration Testing Approaches

### Complete App Testing

```rust
#[test]
fn integration_test_full_app() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::default();

    // Simulate user interaction
    app.handle_key_event(KeyCode::Right.into());
    app.handle_key_event(KeyCode::Right.into());

    // Render
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

    // Assert state
    assert_eq!(app.counter, 2);

    // Assert rendering
    let expected = Buffer::with_lines(vec![
        "┏━━━━━━━━━━━━━ Counter App ━━━━━━━━━━━━━┓",
        "┃                Value: 2                ┃",
        "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
    ]);
    terminal.backend().assert_buffer(&expected);
}
```

### Multi-Screen Navigation Testing

```rust
#[test]
fn test_screen_navigation() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    // Start at home screen
    assert_eq!(app.current_screen, Screen::Home);

    // Navigate to settings
    app.handle_key_event(KeyCode::Char('s').into());
    assert_eq!(app.current_screen, Screen::Settings);

    // Render and verify
    terminal.draw(|frame| app.render(frame, frame.area())).unwrap();
    terminal.backend().assert_buffer_lines(vec![
        "Settings Screen",
        // ... expected output
    ]);
}
```

---

## 6. Testing Sixel Image Rendering

### ratatui-image Library Testing

The `ratatui-image` crate (https://github.com/benjajaja/ratatui-image) provides image rendering support and demonstrates advanced testing techniques for terminal graphics.

### Screenshot Testing Infrastructure

The project uses automated screenshot testing across multiple terminal emulators:

**Supported Terminals:**
- xterm (with `-ti 340` for sixel support)
- foot
- kitty
- wezterm
- ghostty
- rio
- mlterm

### Testing Approach

1. **Flake VM Tests**: Run demo and capture screenshots posted as PR comments
2. **CI Screenshot Tests**: `cargo make screenshot-xvfb && cargo make screenshot-diff`
3. **QA Matrix**: Marks terminals with verified working implementations (✔️) vs untested (❌)

### Known Issues Tracked

- "Sixel image rendered on the last line of terminal causes a scroll"
- Terminal-specific configuration requirements (e.g., xterm sixel enable)

### Verification Strategy

From the documentation:
- "Handling the encoding result" suggests verifying both visual output AND protocol execution
- Tiered testing: confirmed working vs. untested vs. incompatible terminals
- Some platforms not possible to screenshot-test, but expectations documented

### Practical Testing Pattern

```rust
#[test]
fn test_sixel_rendering() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    // Note: TestBackend may not fully support sixel rendering verification
    // This tests the widget logic, not the actual sixel encoding

    let image = load_test_image();
    let widget = ImageWidget::new(image);

    terminal.draw(|frame| {
        frame.render_widget(widget, frame.area());
    }).unwrap();

    // Verify the widget rendered without error
    // Full sixel verification requires actual terminal or screenshot tests
}
```

### Recommendation for dgx-pixels

For sixel preview testing in the TUI:
1. **Unit Tests**: Test widget logic with TestBackend
2. **Screenshot Tests**: Use xterm on Xvfb for visual verification
3. **Integration Tests**: Test protocol selection and fallback logic
4. **Manual QA**: Final verification on actual DGX-Spark terminal

---

## 7. Property-Based Testing

### Overview

While no specific ratatui+proptest examples were found, property-based testing can be valuable for TUI applications.

### Available Tools

**proptest** (recommended)
- Hypothesis-like property testing for Rust
- State machine testing capabilities
- Shrinking on failure

**quickcheck**
- Type-based generation and shrinking
- Simpler than proptest for basic cases

### Example: Testing Widget Properties

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn text_widget_never_panics(text: String) {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let widget = Paragraph::new(text);

        // Should never panic regardless of input
        let result = terminal.draw(|frame| {
            frame.render_widget(widget, frame.area());
        });

        assert!(result.is_ok());
    }

    #[test]
    fn layout_constraints_are_satisfied(
        width in 10u16..100u16,
        height in 10u16..100u16,
    ) {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(frame.area());

            // Properties: chunks should sum to total area
            assert_eq!(chunks[0].height + chunks[1].height, height);
        }).unwrap();
    }
}
```

### Potential Use Cases

1. **Fuzz Testing**: Verify widgets handle arbitrary input without panicking
2. **Layout Properties**: Ensure constraints are satisfied for any terminal size
3. **State Transitions**: Verify state machine properties hold
4. **Invariants**: Test that certain conditions always hold

### Recommendation

Consider proptest for:
- Testing layout calculations across many terminal sizes
- Verifying text handling with unicode edge cases
- State machine testing for complex navigation flows

---

## 8. Parameterized Testing with rstest

### What is rstest?

Fixture-based test framework with pytest-like syntax for parameterized tests.

### Example from widgets_block.rs

```rust
use rstest::rstest;

#[rstest]
#[case::left_with_all_borders(Alignment::Left, Borders::ALL, [
    " ┌Title──────┐ ",
    " │           │ ",
    " └───────────┘ ",
])]
#[case::center_with_all_borders(Alignment::Center, Borders::ALL, [
    " ┌───Title───┐ ",
    " │           │ ",
    " └───────────┘ ",
])]
#[case::right_with_all_borders(Alignment::Right, Borders::ALL, [
    " ┌──────Title┐ ",
    " │           │ ",
    " └───────────┘ ",
])]
fn widgets_block_title_alignment_top<'line, Lines>(
    #[case] alignment: Alignment,
    #[case] borders: Borders,
    #[case] expected: Lines,
) where
    Lines: IntoIterator,
    Lines::Item: Into<ratatui::text::Line<'line>>,
{
    let backend = TestBackend::new(15, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let block = Block::new()
        .borders(borders)
        .title(Line::from("Title").alignment(alignment));

    let area = Rect::new(1, 0, 13, 3);
    let expected = Buffer::with_lines(expected);

    terminal
        .draw(|frame| frame.render_widget(block, area))
        .unwrap();

    terminal.backend().assert_buffer(&expected);
}
```

### Benefits

1. **Readable**: Self-documenting test cases
2. **DRY**: Avoid test duplication
3. **Fixtures**: Reusable test setup
4. **Named Cases**: `#[case::descriptive_name]` for clarity

### Use Cases for dgx-pixels TUI

```rust
#[rstest]
#[case::small_terminal(40, 10, LayoutMode::Compact)]
#[case::medium_terminal(80, 24, LayoutMode::Normal)]
#[case::large_terminal(120, 40, LayoutMode::Expanded)]
fn test_responsive_layout(
    #[case] width: u16,
    #[case] height: u16,
    #[case] expected_mode: LayoutMode,
) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new();

    terminal.draw(|frame| app.render(frame, frame.area())).unwrap();

    assert_eq!(app.layout_mode, expected_mode);
}
```

---

## 9. Mock/Stub Strategies

### TestBackend as Mock

TestBackend IS the mock - it implements the Backend trait but writes to memory instead of a terminal.

### Mocking External Dependencies

For the dgx-pixels architecture (Rust TUI + ZeroMQ + Python backend):

```rust
// Mock ZMQ client for testing
pub struct MockZmqClient {
    responses: Vec<GenerationResponse>,
}

impl MockZmqClient {
    pub fn with_responses(responses: Vec<GenerationResponse>) -> Self {
        Self { responses }
    }
}

impl ZmqClientTrait for MockZmqClient {
    fn send_request(&mut self, request: GenerationRequest) -> Result<GenerationResponse> {
        Ok(self.responses.remove(0))
    }
}

#[test]
fn test_generation_workflow() {
    let mock_client = MockZmqClient::with_responses(vec![
        GenerationResponse::JobCreated { job_id: "123" },
        GenerationResponse::Completed { image_path: "/tmp/test.png" },
    ]);

    let mut app = App::new(Box::new(mock_client));

    // Test the workflow without actual ZMQ
    app.start_generation("test prompt");
    assert_eq!(app.current_job, Some("123"));
}
```

### Trait-Based Dependency Injection

```rust
pub trait Backend {
    fn send_job(&self, params: JobParams) -> Result<JobId>;
    fn poll_status(&self, job_id: JobId) -> Result<JobStatus>;
}

pub struct RealBackend {
    zmq_client: ZmqClient,
}

pub struct MockBackend {
    job_queue: Vec<Job>,
}

// App is generic over backend
pub struct App<B: Backend> {
    backend: B,
}

#[test]
fn test_with_mock() {
    let mock = MockBackend::new();
    let app = App::new(mock);
    // Test without real ZMQ
}
```

---

## 10. Code Coverage and CI Integration

### Tools

**cargo-llvm-cov** (recommended)
```bash
# Install
cargo install cargo-llvm-cov

# Run with coverage
cargo llvm-cov

# Generate lcov report
cargo llvm-cov --lcov --output-path lcov.info

# HTML report
cargo llvm-cov --html

# With nextest
cargo llvm-cov nextest
```

### Codecov Integration

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info

      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: true
```

### Bacon Integration

Bacon is a background task runner for Rust that can run tests with coverage:

```toml
# bacon.toml
[jobs.coverage]
command = ["cargo", "llvm-cov", "--open"]
need_stdout = false
```

Then run: `bacon coverage`

### Ratatui's CI Setup

From research, ratatui uses:
- GitHub Actions for CI (2,466+ workflow runs)
- 90% test coverage target
- Automated testing on PRs
- Benchmark tracking
- Dependabot for dependency updates

---

## 11. Async Testing

### With Tokio

```rust
#[tokio::test]
async fn test_async_event_handler() {
    let mut app = App::new();

    // Simulate async event processing
    app.handle_async_event(Event::LoadData).await;

    assert!(app.data.is_some());
}
```

### Async Event Stream Testing

From the ratatui async tutorial:

```rust
use crossterm::event::{EventStream, Event};
use tokio::select;

#[tokio::test]
async fn test_event_stream() {
    let mut event_stream = EventStream::new();

    select! {
        Some(Ok(event)) = event_stream.next() => {
            // Handle event
        }
        _ = tokio::time::sleep(Duration::from_millis(100)) => {
            // Timeout
        }
    }
}
```

### Testing Tokio Select Loops

```rust
#[tokio::test]
async fn test_main_loop() {
    let mut app = App::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(100));

    // Test one iteration
    select! {
        _ = ticker.tick() => {
            app.on_tick();
        }
    }

    assert_eq!(app.tick_count, 1);
}
```

---

## 12. Real-World Testing Examples

### Projects with Good Testing Practices

1. **ratatui-image** (https://github.com/benjajaja/ratatui-image)
   - Screenshot testing across multiple terminals
   - CI integration with Xvfb
   - QA matrix for terminal compatibility

2. **Maelstrom** - Fast test runner with ratatui TUI
   - Tests TUI in containerized environment
   - Production-ready testing infrastructure

3. **ratatui core** (https://github.com/ratatui/ratatui)
   - 90% test coverage
   - Extensive widget tests in `tests/widgets_*.rs`
   - Snapshot testing examples
   - Parameterized tests with rstest

4. **RustLab 2024 Workshop** (https://github.com/orhun/rustlab2024-ratatui-workshop)
   - Complete chat application with tests
   - Event handling examples
   - Integration testing patterns

### Lessons from Real Projects

1. **Separate Concerns**: Event handling separate from rendering
2. **Use TestBackend**: Core testing primitive for all UI tests
3. **Snapshot Testing**: Essential for catching visual regressions
4. **Parameterized Tests**: rstest for testing multiple scenarios
5. **CI Integration**: Automated testing with coverage tracking

---

## 13. Testing Recommendations for dgx-pixels

### Testing Strategy

Based on research, recommended testing approach for dgx-pixels Rust TUI:

#### Unit Tests (70% of tests)

```rust
// src/ui/components/job_list.rs
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_job_list_renders_empty() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let job_list = JobList::new(vec![]);

        terminal.draw(|frame| {
            frame.render_widget(&job_list, frame.area());
        }).unwrap();

        terminal.backend().assert_buffer_lines(vec![
            "┌Jobs──────────────────────────────┐",
            "│ No jobs                          │",
            "└──────────────────────────────────┘",
        ]);
    }

    #[test]
    fn test_job_selection() {
        let mut job_list = JobList::new(vec![
            Job { id: "1", status: Status::Running },
            Job { id: "2", status: Status::Pending },
        ]);

        assert_eq!(job_list.selected(), 0);
        job_list.next();
        assert_eq!(job_list.selected(), 1);
    }
}
```

#### Snapshot Tests (20% of tests)

```rust
// src/ui/screens/main_screen.rs
#[test]
fn test_main_screen_layout() {
    let app = App::with_mock_backend();
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();

    terminal.draw(|frame| {
        app.render_main_screen(frame, frame.area());
    }).unwrap();

    insta::assert_snapshot!(terminal.backend());
}
```

#### Integration Tests (10% of tests)

```rust
// tests/integration/workflow.rs
#[test]
fn test_generation_workflow() {
    let mock_zmq = MockZmqClient::new();
    let mut app = App::new(mock_zmq);
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();

    // Start generation
    app.handle_key_event(KeyCode::Char('g').into());
    assert_eq!(app.state, AppState::GenerationForm);

    // Enter prompt
    app.input.value = "pixel art character";
    app.handle_key_event(KeyCode::Enter.into());

    // Verify job created
    assert_eq!(app.jobs.len(), 1);

    // Render and verify UI
    terminal.draw(|frame| app.render(frame, frame.area())).unwrap();
    assert!(terminal.backend().buffer().content().contains("Running"));
}
```

### Test Organization

```
rust/
├── src/
│   ├── ui/
│   │   ├── components/
│   │   │   ├── job_list.rs          # Unit tests inline
│   │   │   ├── model_selector.rs    # Unit tests inline
│   │   │   └── preview.rs           # Unit tests inline
│   │   └── screens/
│   │       └── main_screen.rs       # Snapshot tests inline
│   ├── zmq_client.rs                # Unit tests inline
│   └── app.rs                       # Unit + integration tests
├── tests/
│   ├── integration/
│   │   ├── workflow.rs              # End-to-end workflows
│   │   └── navigation.rs            # Screen navigation
│   └── snapshots/                   # Insta snapshots
└── Cargo.toml
```

### Required Dependencies

```toml
[dev-dependencies]
insta = "1.40"
rstest = "0.18"
proptest = "1.4"
tokio-test = "0.4"
pretty_assertions = "1.4"
```

### CI Pipeline

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Install cargo-insta
        uses: taiki-e/install-action@cargo-insta

      - name: Run tests
        run: cargo test

      - name: Check snapshots
        run: cargo insta test --check

      - name: Generate coverage
        run: cargo llvm-cov --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
```

---

## 14. Testing Checklist for Implementation

### Phase 1: Foundation (WS-06)

- [ ] Set up TestBackend for basic widget tests
- [ ] Add insta + cargo-insta for snapshot testing
- [ ] Create test helpers module
- [ ] Test basic widget rendering (empty states)
- [ ] Test keyboard navigation logic

### Phase 2: Core Features (WS-13)

- [ ] Test job list component (empty, populated, selected)
- [ ] Test model selector component
- [ ] Test prompt input component
- [ ] Test status display component
- [ ] Snapshot tests for each screen layout
- [ ] Test screen transitions

### Phase 3: Integration (WS-13)

- [ ] Mock ZMQ client implementation
- [ ] Test generation workflow end-to-end
- [ ] Test error handling and recovery
- [ ] Test async event processing
- [ ] Test image preview logic (without actual sixel)

### Phase 4: Advanced (WS-18)

- [ ] Property-based tests for layout calculations
- [ ] Screenshot tests for sixel preview (manual/CI)
- [ ] Performance tests (render time < 16ms for 60fps)
- [ ] Stress tests (many jobs, large lists)
- [ ] CI integration with coverage reporting

---

## 15. Known Limitations and Workarounds

### Color Assertion Limitation

**Issue**: TestBackend's Display implementation doesn't capture style/color information
**Tracking**: https://github.com/ratatui/ratatui/issues/1402
**Status**: PR #2099 provides implementation

**Workaround 1**: Manual buffer cell inspection
```rust
let mut expected = Buffer::with_lines(vec!["Hello"]);
expected[(0, 0)].set_fg(Color::Red);
assert_eq!(terminal.backend().buffer(), &expected);
```

**Workaround 2**: Test colors separately
```rust
#[test]
fn test_content() {
    // Test layout without colors
    terminal.backend().assert_buffer_lines(vec!["Hello"]);
}

#[test]
fn test_colors() {
    // Test colors separately
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(0, 0)].fg, Color::Red);
}
```

### Sixel Testing Limitation

**Issue**: TestBackend can't verify actual sixel encoding
**Workaround**: Multi-level testing
1. Unit tests: Test widget logic with TestBackend
2. Integration tests: Test protocol selection
3. Screenshot tests: Visual verification on real terminal
4. Manual QA: Final verification on target hardware

### Async Testing Complexity

**Issue**: Testing tokio::select! loops is complex
**Workaround**: Extract logic into testable functions
```rust
// Hard to test
async fn main_loop() {
    loop {
        select! {
            event = events.next() => { /* complex logic */ }
            _ = ticker.tick() => { /* complex logic */ }
        }
    }
}

// Easier to test
async fn handle_event(&mut self, event: Event) { /* logic */ }
async fn handle_tick(&mut self) { /* logic */ }

#[tokio::test]
async fn test_event_handling() {
    let mut app = App::new();
    app.handle_event(Event::Key(...)).await;
    assert_eq!(app.state, Expected);
}
```

---

## 16. Key Crates and Tools

### Core Testing

| Crate | Purpose | Version | Priority |
|-------|---------|---------|----------|
| `ratatui` | TUI framework with TestBackend | 0.28.0+ | Required |
| `insta` | Snapshot testing | 1.40+ | Required |
| `cargo-insta` | Snapshot review CLI | 1.40+ | Required |

### Test Frameworks

| Crate | Purpose | Version | Priority |
|-------|---------|---------|----------|
| `rstest` | Parameterized tests | 0.18+ | Recommended |
| `proptest` | Property-based testing | 1.4+ | Optional |
| `tokio-test` | Async test utilities | 0.4+ | Required (async) |
| `pretty_assertions` | Better diff output | 1.4+ | Recommended |

### Coverage and CI

| Tool | Purpose | Priority |
|------|---------|----------|
| `cargo-llvm-cov` | Code coverage | Required |
| `cargo-nextest` | Faster test runner | Optional |
| `bacon` | Background test runner | Optional |
| `codecov` | Coverage reporting | Recommended |

### Development

| Tool | Purpose | Priority |
|------|---------|----------|
| `cargo-watch` | Auto-run tests on changes | Recommended |
| `cargo-expand` | Debug macros | Optional |

---

## 17. Additional Resources

### Official Documentation

- Ratatui Docs: https://ratatui.rs/
- Testing Recipe: https://ratatui.rs/recipes/testing/snapshots/
- API Docs: https://docs.rs/ratatui/latest/ratatui/
- Contributing Guide: https://github.com/ratatui/ratatui/blob/main/CONTRIBUTING.md

### Tutorials and Examples

- Counter App Tutorial: https://ratatui.rs/tutorials/counter-app/
- Async Tutorial: https://ratatui.rs/tutorials/counter-async-app/
- RustLab 2024 Workshop: https://github.com/orhun/rustlab2024-ratatui-workshop
- Official Examples: https://github.com/ratatui/ratatui/tree/main/examples

### Community Projects

- awesome-ratatui: https://github.com/ratatui/awesome-ratatui
- ratatui-image: https://github.com/benjajaja/ratatui-image
- tui-realm: https://github.com/veeso/tui-realm

### Related Tools

- cargo-insta docs: https://insta.rs/docs/
- rstest docs: https://docs.rs/rstest/
- proptest book: https://proptest-rs.github.io/proptest/

---

## 18. Conclusion

### Key Takeaways

1. **TestBackend is Essential**: Core primitive for all TUI testing
2. **Snapshot Testing Works**: Insta integration is mature and recommended
3. **Separate Concerns**: Event handlers separate from rendering enables easy testing
4. **Good Ecosystem**: Mature testing tools available (rstest, proptest, cargo-llvm-cov)
5. **90% Coverage Achievable**: Ratatui itself demonstrates high coverage is practical

### Recommended Approach for dgx-pixels

1. **Start Simple**: Begin with TestBackend unit tests
2. **Add Snapshots**: Use insta for visual regression testing
3. **Mock Dependencies**: Use trait-based DI for ZMQ client
4. **Parameterize**: Use rstest for testing multiple scenarios
5. **Measure Coverage**: Integrate cargo-llvm-cov from day one
6. **CI First**: Set up GitHub Actions early

### Success Metrics

- **Unit Test Coverage**: >80% for UI components
- **Integration Test Coverage**: >60% for workflows
- **Snapshot Tests**: All screens have snapshot tests
- **CI Pipeline**: All tests pass, coverage reported
- **Performance**: Tests run in <5s for fast feedback

### Next Steps

1. Set up test infrastructure in WS-06 (TUI Foundation)
2. Implement helper utilities for common test patterns
3. Create snapshot baselines for initial UI
4. Integrate CI pipeline with coverage reporting
5. Document testing patterns in team wiki

---

**Research completed by:** Claude (Sonnet 4.5)
**Date:** 2025-11-19
**Project:** dgx-pixels (raibid-labs)
**Location:** `/home/beengud/raibid-labs/dgx-pixels/docs/research/ratatui-testing-research.md`
