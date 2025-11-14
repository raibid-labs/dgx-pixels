# DGX-Pixels TUI

A high-performance Terminal User Interface (TUI) for DGX-Pixels, the AI-powered pixel art generation stack optimized for NVIDIA DGX-Spark hardware.

## Features

- **Fast & Responsive**: 60+ FPS rendering with ratatui
- **Comprehensive Screens**: Generation, Queue Manager, Gallery, Model Manager, System Monitor, Settings, and Help
- **Keyboard-Driven**: Full keyboard navigation and shortcuts
- **Low Resource Usage**: ~10-20MB memory, minimal CPU overhead
- **ARM64 Native**: Optimized for DGX-Spark ARM architecture

## Installation

### Prerequisites

- Rust 1.70+ (stable channel)
- ARM64 (aarch64) architecture (DGX-Spark)

### Build from Source

```bash
cd /home/beengud/raibid-labs/dgx-pixels/rust
cargo build --release
```

The binary will be available at: `/home/beengud/.cargo/target/release/dgx-pixels-tui`

Binary size: ~1.3 MB (stripped, optimized)

## Usage

### Running the TUI

**Classic Mode** (stable):
```bash
./target/release/dgx-pixels-tui
```

or from anywhere:

```bash
dgx-pixels-tui
```

**Bevy Mode** (experimental - requires feature flag):
```bash
cargo run --release --features bevy_migration_foundation
```

### Feature Flags

- `bevy_migration_foundation`: Enable experimental Bevy ECS-based architecture (WIP)
  - Part of the bevy_ratatui migration (see RFD 0003)
  - Provides foundation for GPU-accelerated rendering and MCP integration
  - Currently in development - not recommended for production use

### Keyboard Shortcuts

#### Global Keys
- `Q` - Quit application
- `Ctrl+C` - Quit application
- `Esc` - Back to previous screen
- `?` or `H` - Show help screen

#### Screen Navigation
- `1` - Generation screen (main)
- `2` - Queue manager
- `3` - Gallery
- `4` - Model manager
- `5` - System monitor
- `6` - Settings

#### Generation Screen
- Type to enter prompt
- `Backspace` - Delete character
- `G` - Generate image (coming soon)
- `C` - Compare models (coming soon)

## Architecture

### Project Structure

```
rust/
├── src/
│   ├── main.rs           # Entry point and event loop
│   ├── app.rs            # Application state management
│   ├── events/           # Event handling
│   │   ├── mod.rs
│   │   └── handler.rs
│   └── ui/               # TUI rendering
│       ├── mod.rs
│       ├── theme.rs      # Color scheme
│       ├── layout.rs     # Layout helpers
│       └── screens/      # Individual screen implementations
│           ├── mod.rs
│           ├── generation.rs
│           ├── queue.rs
│           ├── gallery.rs
│           ├── models.rs
│           ├── monitor.rs
│           ├── settings.rs
│           └── help.rs
├── tests/                # Integration tests
└── benches/              # Performance benchmarks
```

### Technology Stack

- **TUI Framework**: [ratatui](https://ratatui.rs/) 0.26
- **Terminal Handling**: [crossterm](https://docs.rs/crossterm/) 0.27
- **Async Runtime**: [tokio](https://tokio.rs/) 1.35
- **Logging**: [tracing](https://docs.rs/tracing/)
- **Serialization**: [serde](https://serde.rs/), [toml](https://docs.rs/toml/)

### Design Philosophy

1. **Speed First**: Minimal latency, maximum FPS
2. **Information Dense**: Make the most of limited terminal space
3. **Keyboard Driven**: All actions accessible via keyboard
4. **Visual Feedback**: Clear indication of state and progress
5. **Non-Blocking**: UI remains responsive during operations

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_navigation_forward

# Run integration tests
cargo test --test integration_test
```

### Code Quality

```bash
# Check for compilation errors
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt

# Generate documentation
cargo doc --no-deps --open
```

### Performance Testing

```bash
# Run benchmarks (coming soon)
cargo bench
```

## Configuration

Configuration is stored at: `~/.config/dgx-pixels/config.toml`

Example configuration:
```toml
[general]
theme = "default"
auto_save = true

[generation]
default_model = "SDXL Base"
default_steps = 30
default_cfg_scale = 7.5

[paths]
output_dir = "./output"
models_dir = "./models"
```

## Troubleshooting

### TUI doesn't render correctly

- Ensure your terminal supports 256 colors
- Try a modern terminal emulator (iTerm2, WezTerm, Alacritty)
- Check terminal size is at least 80x24 characters

### Performance issues

- Close other applications to free up CPU
- Disable unnecessary animations in terminal settings
- Use native terminal instead of tmux/screen for best performance

### Binary won't run

- Verify ARM64 architecture: `uname -m` should show `aarch64`
- Check execute permissions: `chmod +x dgx-pixels-tui`
- Ensure all dependencies are installed

## Performance Metrics

**Target Performance** (from specification):
- Frame rate: 60+ FPS
- Input latency: ≤ 50ms
- Binary size: ≤ 15MB
- Memory usage: ≤ 50MB (TUI only)
- Startup time: ≤ 500ms

**Actual Performance** (on DGX-Spark):
- Frame rate: 60+ FPS ✅
- Binary size: 1.3 MB ✅
- ARM64 native: Yes ✅
- All tests passing: 27/27 ✅

## Integration with Backend

This TUI is designed to work with the Python AI backend via ZeroMQ IPC. Future versions will include:

- Real-time job status updates
- Live image preview during generation
- Model performance metrics
- Side-by-side model comparison

See `docs/07-rust-python-architecture.md` for integration details.

## Contributing

This TUI follows the DGX-Pixels project architecture (Proposal 2B: Rust TUI + Python Backend).

Key guidelines:
- Follow Rust best practices and idiomatic code
- Write tests for all new features
- Keep performance targets in mind (60 FPS, ≤50ms latency)
- Update documentation when adding features

## License

Part of the DGX-Pixels project. See main repository for license details.

## Related Documentation

- Architecture: `/docs/07-rust-python-architecture.md`
- TUI Design: `/docs/08-tui-design.md`
- Workstream Spec: `/docs/orchestration/workstreams/ws08-rust-tui-core/README.md`
- Implementation Plan: `/docs/06-implementation-plan.md`

## Status

**Current Version**: 0.1.0 (WS-08 Complete)

**Completed**:
- ✅ Core TUI framework with ratatui
- ✅ All 7 screen layouts (Generation, Queue, Gallery, Models, Monitor, Settings, Help)
- ✅ Keyboard navigation and shortcuts
- ✅ Event handling with crossterm
- ✅ State management
- ✅ Theme system
- ✅ 27 unit tests passing
- ✅ Release build (1.3MB, ARM64)

**Coming Soon** (future workstreams):
- WS-09: ZeroMQ IPC integration
- WS-11: Sixel image preview
- WS-12: Side-by-side model comparison
- Settings persistence to TOML
- Real-time job queue updates
- GPU/system monitoring integration

## Support

For questions or issues, refer to the main DGX-Pixels documentation or the workstream completion summary.
