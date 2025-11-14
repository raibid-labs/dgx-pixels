# WS-08: Event Bus

**Orchestrator**: Core Systems
**Duration**: 2-3 days
**Risk**: Low
**Dependencies**: WS-02, WS-03

## Summary

Create custom Bevy events for app-specific actions. Implement event-driven architecture for cross-system communication. Add event logging/debugging.

## Files Created
```
rust/src/bevy_app/events/
├── mod.rs
├── navigation.rs        # Screen navigation events
├── generation.rs        # Generation request events
└── gallery.rs           # Gallery actions
```

## Key Implementation

```rust
#[derive(Event)]
pub struct NavigateToScreen(pub Screen);

#[derive(Event)]
pub struct SubmitGenerationJob {
    pub prompt: String,
    pub params: GenerationParams,
}

fn handle_navigation_events(
    mut events: EventReader<NavigateToScreen>,
    mut current_screen: ResMut<CurrentScreen>,
) {
    for event in events.read() {
        current_screen.0 = event.0;
    }
}
```

## Acceptance Criteria
- [ ] Events registered with Bevy
- [ ] Event-driven navigation works
- [ ] Cross-system communication via events
- [ ] Event debugging logs available

**Branch**: `tui-modernization/ws08-event-bus`
