# WS-09: Generation Screen

**Orchestrator**: Screen Migration
**Duration**: 1-2 days
**Risk**: Low
**Dependencies**: WS-04 (Rendering), WS-03 (Input)

## Summary

Migrate Generation screen from old ratatui rendering to Bevy ECS systems. Create render system and input handler following the established pattern from WS-09.

## Files Created
```
rust/src/bevy_app/systems/render/screens/generation.rs
rust/src/bevy_app/systems/input/screens/generation.rs
```

## Files Modified
```
rust/src/bevy_app/plugins.rs    # Register screen systems
```

## Implementation Pattern

**Render system** (`render/screens/generation.rs`):
```rust
use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;

pub fn render_generation_screen(
    mut ratatui_ctx: ResMut<RatatuiContext>,
    current_screen: Res<CurrentScreen>,
    // ... screen-specific resources
) {
    if current_screen.0 != Screen::Generation {
        return;
    }

    ratatui_ctx.draw(|frame| {
        // Port widget code from ui/screens/generation.rs
    }).ok();
}
```

**Input handler** (`input/screens/generation.rs`):
```rust
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;

pub fn handle_generation_input(
    mut messages: MessageReader<KeyMessage>,
    current_screen: Res<CurrentScreen>,
    // ... screen-specific resources
) {
    if current_screen.0 != Screen::Generation {
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
fn test_generation_renders() {
    let mut app = create_test_app();
    app.world.resource_mut::<CurrentScreen>().0 = Screen::Generation;
    
    app.update();
    
    let stats = app.world.resource::<RenderStats>();
    assert_eq!(stats.last_rendered_screen, Screen::Generation);
}
```

**Complexity**: High (most interactions)

**Branch**: `tui-modernization/ws09-generation-screen`
