# WS-13: Queue Screen

**Orchestrator**: Screen Migration
**Duration**: 1-2 days
**Risk**: Low
**Dependencies**: WS-04 (Rendering), WS-03 (Input)

## Summary

Migrate Queue screen from old ratatui rendering to Bevy ECS systems. Create render system and input handler following the established pattern from WS-09.

## Files Created
```
rust/src/bevy_app/systems/render/screens/queue.rs
rust/src/bevy_app/systems/input/screens/queue.rs
```

## Files Modified
```
rust/src/bevy_app/plugins.rs    # Register screen systems
```

## Implementation Pattern

**Render system** (`render/screens/queue.rs`):
```rust
use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;

pub fn render_queue_screen(
    mut ratatui_ctx: ResMut<RatatuiContext>,
    current_screen: Res<CurrentScreen>,
    // ... screen-specific resources
) {
    if current_screen.0 != Screen::Queue {
        return;
    }

    ratatui_ctx.draw(|frame| {
        // Port widget code from ui/screens/queue.rs
    }).ok();
}
```

**Input handler** (`input/screens/queue.rs`):
```rust
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;

pub fn handle_queue_input(
    mut messages: MessageReader<KeyMessage>,
    current_screen: Res<CurrentScreen>,
    // ... screen-specific resources
) {
    if current_screen.0 != Screen::Queue {
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
fn test_queue_renders() {
    let mut app = create_test_app();
    app.world.resource_mut::<CurrentScreen>().0 = Screen::Queue;
    
    app.update();
    
    let stats = app.world.resource::<RenderStats>();
    assert_eq!(stats.last_rendered_screen, Screen::Queue);
}
```

**Complexity**: Medium (job list)

**Branch**: `tui-modernization/ws13-queue-screen`
