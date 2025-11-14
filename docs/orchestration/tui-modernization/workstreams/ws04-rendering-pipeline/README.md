# WS-04: Rendering Pipeline

**Orchestrator**: Foundation
**Duration**: 4-5 days
**Risk**: High
**Dependencies**: WS-02 (ECS State)

## Summary

Implement Bevy rendering systems using `RatatuiContext::draw()`. Create screen routing dispatch system and port existing ratatui widget code to Bevy systems while maintaining 60 FPS performance.

## Files Created
```
rust/src/bevy_app/systems/render/
├── mod.rs
├── dispatch.rs          # Screen routing
├── layout.rs            # Layout computation
└── widgets.rs           # Reusable widget helpers
```

## Key Implementation

```rust
fn render_dispatch(
    mut ratatui_ctx: ResMut<RatatuiContext>,
    current_screen: Res<CurrentScreen>,
) -> Result<()> {
    ratatui_ctx.draw(|frame| {
        match current_screen.0 {
            Screen::Generation => render_placeholder(frame, "Generation"),
            Screen::Gallery => render_placeholder(frame, "Gallery"),
            // ... other screens (migrated in WS-09 through WS-16)
            _ => render_placeholder(frame, "Screen"),
        }
    })?;
    Ok(())
}
```

## Acceptance Criteria
- [ ] All screens render placeholders in Bevy mode
- [ ] 60 FPS maintained (frame time <16ms)
- [ ] No flickering or tearing
- [ ] Status bar renders correctly

**Branch**: `tui-modernization/ws04-rendering-pipeline`
