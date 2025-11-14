# WS-03: Input System

**Orchestrator**: Foundation
**Duration**: 3-4 days
**Risk**: Medium
**Dependencies**: WS-02 (ECS State)

## Summary

Replace crossterm event polling with Bevy message-based input handling. Migrate all keyboard and resize event logic from imperative loops to declarative Bevy systems.

## Timeline

- **Day 1**: Create input system structure, keyboard message handling
- **Day 2**: Implement navigation and text entry systems
- **Day 3**: Testing, edge cases (special keys, modifiers)
- **Day 4**: Documentation, PR submission

## Scope

### Files Created
```
rust/src/bevy_app/systems/input/
├── mod.rs
├── keyboard.rs          # Keyboard input system
├── navigation.rs        # Screen navigation logic
└── text_entry.rs        # Text input handling
```

### Files Modified
```
rust/src/bevy_app/plugins.rs    # Register input systems in PreUpdate
```

## Implementation Highlights

**Keyboard input system** (`keyboard.rs`):
```rust
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use crate::bevy_app::resources::*;

pub fn handle_keyboard_input(
    mut messages: MessageReader<KeyMessage>,
    mut current_screen: ResMut<CurrentScreen>,
    mut input_buffer: ResMut<InputBuffer>,
    mut exit: EventWriter<AppExit>,
) {
    for message in messages.read() {
        match message.code {
            KeyCode::Char('q') if current_screen.0 != Screen::Generation => {
                exit.send(AppExit);
            }
            KeyCode::Tab => {
                current_screen.0 = current_screen.0.next();
            }
            KeyCode::BackTab => {
                current_screen.0 = current_screen.0.previous();
            }
            _ => {}
        }
    }
}
```

**Text entry system** (`text_entry.rs`):
```rust
pub fn handle_text_input(
    mut messages: MessageReader<KeyMessage>,
    current_screen: Res<CurrentScreen>,
    mut input_buffer: ResMut<InputBuffer>,
) {
    // Only process text input on Generation screen
    if current_screen.0 != Screen::Generation {
        return;
    }

    for message in messages.read() {
        match message.code {
            KeyCode::Char(c) => input_buffer.insert(c),
            KeyCode::Backspace => input_buffer.backspace(),
            KeyCode::Left => input_buffer.move_left(),
            KeyCode::Right => input_buffer.move_right(),
            KeyCode::Enter => {
                // Submit job (handled by WS-08 events)
            }
            _ => {}
        }
    }
}
```

## Acceptance Criteria

- [ ] All keyboard shortcuts functional in Bevy mode
- [ ] Text input works in Generation screen
- [ ] Screen navigation (Tab/Shift+Tab) works
- [ ] Quit on 'q' works (except Generation screen)
- [ ] Resize events handled
- [ ] Input latency <16ms (60 FPS)

## Testing

```rust
#[test]
fn test_keyboard_navigation() {
    let mut app = create_test_app();

    app.world.send_message(KeyMessage { 
        code: KeyCode::Tab, 
        .. 
    });
    app.update();

    let screen = app.world.resource::<CurrentScreen>();
    assert_eq!(screen.0, Screen::Gallery);
}
```

---

**Status**: Ready for Implementation
**Branch**: `tui-modernization/ws03-input-system`
