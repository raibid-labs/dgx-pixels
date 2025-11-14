# WS-14: Monitor Screen

**Orchestrator**: Screen Migration
**Duration**: 1-2 days
**Risk**: Low
**Dependencies**: WS-04 (Rendering), WS-03 (Input)

## Summary

Migrate Monitor screen from old ratatui rendering to Bevy ECS systems. Create render system and input handler following the established pattern from WS-09.

## Files Created
```
rust/src/bevy_app/systems/render/screens/monitor.rs
rust/src/bevy_app/systems/input/screens/monitor.rs
```

## Files Modified
```
rust/src/bevy_app/plugins.rs    # Register screen systems
```

## Implementation Pattern

**Render system** (`render/screens/monitor.rs`):
```rust
use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;

pub fn render_monitor_screen(
    mut ratatui_ctx: ResMut<RatatuiContext>,
    current_screen: Res<CurrentScreen>,
    // ... screen-specific resources
) {
    if current_screen.0 != Screen::Monitor {
        return;
    }

    ratatui_ctx.draw(|frame| {
        // Port widget code from ui/screens/monitor.rs
    }).ok();
}
```

**Input handler** (`input/screens/monitor.rs`):
```rust
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;

pub fn handle_monitor_input(
    mut messages: MessageReader<KeyMessage>,
    current_screen: Res<CurrentScreen>,
    // ... screen-specific resources
) {
    if current_screen.0 != Screen::Monitor {
        return;
    }

    for message in messages.read() {
        match message.code {
            // Handle screen-specific input
            _ => {}
        }
    }
}
```

## Acceptance Criteria
- [ ] Visual parity with old screen
- [ ] All interactions functional
- [ ] Performance maintained (60 FPS)
- [ ] Tests passing
- [ ] No regressions in other screens

## Testing

```rust
#[test]
fn test_monitor_renders() {
    let mut app = create_test_app();
    app.world.resource_mut::<CurrentScreen>().0 = Screen::Monitor;
    
    app.update();
    
    let stats = app.world.resource::<RenderStats>();
    assert_eq!(stats.last_rendered_screen, Screen::Monitor);
}
```

**Complexity**: Medium (real-time stats)

**Branch**: `tui-modernization/ws14-monitor-screen`
