# Sixel Image Preview Guide

## Overview

The DGX-Pixels TUI now supports real-time image preview using the Sixel graphics protocol. This allows you to see generated images directly in your terminal without switching to an external viewer.

## Terminal Requirements

### Supported Terminals (Full Sixel Support)

1. **kitty** (Recommended)
   ```bash
   # Install on Ubuntu/Debian
   sudo apt install kitty

   # Launch
   kitty
   ```

2. **WezTerm**
   ```bash
   # Install from https://wezfurlong.org/wezterm/

   # Launch
   wezterm
   ```

3. **iTerm2** (macOS only)
   - Download from https://iterm2.com/
   - Sixel support in version 3.5+

4. **xterm with Sixel**
   ```bash
   # Install xterm with Sixel support
   sudo apt install xterm

   # Launch with Sixel enabled
   xterm -ti vt340
   ```

### Fallback Mode (Text-Only Terminals)

If your terminal doesn't support Sixel, the TUI automatically falls back to text-only mode showing:
- Image filename
- Dimensions
- File size
- Helpful message about Sixel support

No functionality is lost - you just won't see inline previews.

## Terminal Detection

The TUI automatically detects Sixel support by checking:

1. `$TERM` environment variable
2. `$TERM_PROGRAM` environment variable
3. `$KITTY_WINDOW_ID` for kitty terminal

You can verify detection in the TUI logs:
```bash
RUST_LOG=info ./target/release/dgx-pixels-tui
```

Look for lines like:
```
INFO dgx_pixels_tui::sixel::terminal_detection: Terminal supports Sixel (kitty)
```

## Usage

### Generation Screen (Key: 1)

The generation screen shows:
- **Left Pane:** Generation controls and active job progress
- **Right Pane:** Real-time preview of generated image

When a generation completes:
1. Preview automatically requests rendering
2. Loading indicator appears while decoding
3. Image displays in preview pane (Sixel terminals)
4. Preview is cached for instant re-display

**Controls:**
- Type your prompt in the input field
- Press `[G]` to generate (when connected to backend)
- Watch progress in the left pane
- Preview appears in the right pane

### Gallery Screen (Key: 3)

The gallery screen shows:
- **Left Pane (70%):** Full-size preview of selected image
- **Right Pane (30%):** List of all generated images

**Controls:**
- `Left Arrow` - Previous image
- `Right Arrow` - Next image
- `Esc` - Return to generation screen

**Features:**
- Thumbnails auto-load for visible items
- Main preview updates instantly when navigating
- Cache prevents re-loading same images
- Status bar shows total images and storage used

### Cache Management

The preview manager maintains a 50MB LRU cache:

- **Cache Hit:** <1ms to display cached preview
- **Cache Miss:** <100ms to decode and cache new preview
- **Automatic Eviction:** Oldest previews removed when cache full

**View Cache Stats:**
Check the status bar on any screen:
```
GPU: Ready | Memory: 104GB free | Cache: 12.3MB (8 previews)
```

**Clear Cache:**
Cache automatically clears when TUI exits. To manually clear during development:
```rust
app.preview_manager.clear_cache();
```

## Performance Characteristics

### Sixel Mode (Supported Terminals)

- **First Preview:** 50-100ms (decode + render)
- **Cached Preview:** <1ms (instant)
- **Gallery Navigation:** <16ms (60 FPS)
- **TUI Responsiveness:** 60+ FPS maintained

### Text-Only Mode (Fallback)

- **First Preview:** <1ms (no rendering)
- **Cached Preview:** <1ms
- **Gallery Navigation:** <16ms
- **TUI Responsiveness:** 60+ FPS maintained

## Advanced Configuration

### Render Quality (Code-Level)

```rust
use crate::sixel::RenderOptions;

// High quality (generation screen)
let options = RenderOptions {
    width: 40,
    height: 20,
    preserve_aspect: true,
    high_quality: true,  // Lanczos3 filtering
};

// Fast thumbnails (gallery)
let options = RenderOptions {
    width: 10,
    height: 10,
    preserve_aspect: true,
    high_quality: false,  // Triangle filtering
};
```

### Cache Size Limit

Adjust in `rust/src/sixel/mod.rs`:
```rust
/// Maximum preview cache size in MB
pub const MAX_CACHE_SIZE_MB: usize = 50;  // Change as needed
```

### Sixel Color Depth

Adjust in `rust/src/sixel/mod.rs`:
```rust
/// Maximum colors for Sixel (256 for best compatibility)
pub const MAX_SIXEL_COLORS: usize = 256;  // 256 recommended
```

## Troubleshooting

### No Preview Appearing

1. **Check Terminal Support:**
   ```bash
   echo $TERM
   echo $TERM_PROGRAM
   ```

2. **Verify Sixel Detection:**
   ```bash
   RUST_LOG=debug ./target/release/dgx-pixels-tui 2>&1 | grep -i sixel
   ```

3. **Test Sixel Directly:**
   ```bash
   # Use viuer to test
   cargo install viuer
   viuer image.png
   ```

### Slow Preview Loading

1. **Check Image Size:**
   - Large images (>4096x4096) take longer
   - Consider resizing source images

2. **Monitor Cache Stats:**
   - Check status bar for cache usage
   - High cache pressure may cause evictions

3. **Check System Load:**
   - Preview rendering is CPU-bound
   - Other processes may slow decoding

### Preview Artifacts

1. **Terminal Color Depth:**
   - Some terminals limit Sixel colors
   - Artifacts may appear with complex images
   - Try a different terminal

2. **Image Format:**
   - PNG recommended for pixel art
   - JPEG may show compression artifacts

### Cache Not Working

1. **Check Logs:**
   ```bash
   RUST_LOG=debug ./target/release/dgx-pixels-tui 2>&1 | grep -i cache
   ```

2. **Verify File Paths:**
   - Previews cached by file path
   - Moving files invalidates cache

3. **Memory Pressure:**
   - LRU eviction may be aggressive
   - Increase MAX_CACHE_SIZE_MB if needed

## Integration with Backend (WS-10)

When the Python backend is running:

1. **Submit Generation:**
   - Enter prompt in generation screen
   - Press `G` to generate
   - Job ID assigned automatically

2. **Progress Updates:**
   - Left pane shows real-time progress
   - Stage, percentage, and ETA displayed
   - Preview pane shows "Loading..." during generation

3. **Completion:**
   - Final image auto-displays in preview
   - Image added to gallery automatically
   - Press `3` to view in gallery

## Example Workflow

### Basic Generation
```
1. Launch TUI in kitty terminal
   kitty -e ./target/release/dgx-pixels-tui

2. Generation screen (default)
   - Type: "16-bit pixel art knight sprite"
   - Press: G (when backend connected)

3. Watch progress
   - Left pane: Progress bar
   - Right pane: Loading indicator

4. View result
   - Preview appears automatically
   - Image cached for instant re-view

5. Browse gallery
   - Press: 3
   - Arrow keys to navigate
   - Esc to return
```

### Side-by-Side Comparison (Future: WS-12)
```
1. Press: C (comparison screen)
2. Select two models (base + LoRA)
3. Enter prompt
4. Press: G (generate with both)
5. View side-by-side previews
6. Pick winner
7. Export comparison report
```

## Best Practices

1. **Use Recommended Terminals:**
   - kitty offers best performance
   - WezTerm is cross-platform
   - Avoid terminals without Sixel

2. **Manage Cache:**
   - Gallery auto-manages cache
   - Don't worry about manual clearing
   - Monitor status bar for pressure

3. **Optimize Workflow:**
   - Use generation screen for active work
   - Use gallery for browsing history
   - Use comparison for A/B testing

4. **Performance Tips:**
   - Keep image sizes reasonable (<2048x2048)
   - Close other resource-intensive apps
   - Use thumbnail mode in gallery for browsing

## Future Enhancements

### Planned Features
- [ ] Custom sixel encoding (replace viuer)
- [ ] GPU-accelerated rendering
- [ ] Video preview (animated sprites)
- [ ] Multi-preview comparison mode
- [ ] Export preview cache to disk
- [ ] Configurable cache policies

### Experimental
- [ ] ASCII art fallback (text terminals)
- [ ] Remote preview (SSH sessions)
- [ ] Web-based preview mirror
- [ ] VR/AR preview mode (future hardware)

## Support

For issues or questions:
1. Check logs: `RUST_LOG=debug ./target/release/dgx-pixels-tui`
2. Verify terminal: Test with `viuer` CLI tool
3. Review documentation: `docs/ws-11-sixel-preview-complete.md`
4. File issue: Include terminal type and logs

## References

- Sixel Graphics: https://en.wikipedia.org/wiki/Sixel
- viuer crate: https://docs.rs/viuer/
- kitty terminal: https://sw.kovidgoyal.net/kitty/
- WezTerm: https://wezfurlong.org/wezterm/
- iTerm2: https://iterm2.com/

## License

Part of the DGX-Pixels project. See main LICENSE file.
