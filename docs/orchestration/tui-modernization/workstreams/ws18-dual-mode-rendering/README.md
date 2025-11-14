# WS-18: Dual-Mode Rendering

**Orchestrator**: Integration
**Duration**: 3-4 days
**Risk**: Medium
**Dependencies**: All screens (WS-09 through WS-16)

## Summary

Enable both terminal and native window rendering. Add toggle between modes (Ctrl+W). Optimize rendering for each mode. Support running both modes simultaneously.

## Files Created
```
rust/src/bevy_app/rendering/
├── mod.rs
├── terminal_mode.rs     # Terminal rendering
├── window_mode.rs       # Native window rendering
└── mode_switcher.rs     # Toggle system
```

## Key Implementation

```rust
#[derive(Resource)]
enum RenderMode {
    Terminal,
    Window,
    Both,
}

fn toggle_render_mode(
    mut messages: MessageReader<KeyMessage>,
    mut mode: ResMut<RenderMode>,
    mut windows: Query<&mut Window>,
) {
    for message in messages.read() {
        if message.code == KeyCode::Char('w') && message.modifiers.contains(CONTROL) {
            *mode = match *mode {
                RenderMode::Terminal => RenderMode::Window,
                RenderMode::Window => RenderMode::Both,
                RenderMode::Both => RenderMode::Terminal,
            };
        }
    }
}
```

## Acceptance Criteria
- [ ] Terminal mode works (default)
- [ ] Window mode works (GPU-accelerated UI)
- [ ] Both modes simultaneously (split view)
- [ ] Smooth toggling with Ctrl+W
- [ ] Image quality better in window mode
- [ ] Terminal mode degrades gracefully (ASCII fallback)

## Performance Targets
- Terminal mode: 60 FPS
- Window mode: 144 FPS (on capable hardware)
- Mode switching: <1 second

## Testing

Test on various environments:
- [ ] Local terminal (kitty, alacritty, etc.)
- [ ] SSH session
- [ ] Headless server (window mode should fail gracefully)
- [ ] Containers

**Branch**: `tui-modernization/ws18-dual-mode-rendering`
