# WS-07: Theme & Styling

**Orchestrator**: Core Systems
**Duration**: 2-3 days
**Risk**: Low
**Dependencies**: WS-04 (Rendering Pipeline)

## Summary

Port `ui/theme.rs` to Bevy resource. Ensure consistent styling across old and new systems. Foundation for future theme switching (dark/light modes).

## Files Created
```
rust/src/bevy_app/resources/theme.rs
```

## Key Implementation

```rust
#[derive(Resource, Clone)]
pub struct AppTheme {
    pub colors: ThemeColors,
}

pub struct ThemeColors {
    pub text: Color,
    pub background: Color,
    pub highlight: Color,
    pub border: Color,
    pub success: Color,
    pub error: Color,
}

impl AppTheme {
    pub fn text(&self) -> Style {
        Style::default().fg(self.colors.text)
    }
    
    pub fn highlight(&self) -> Style {
        Style::default()
            .fg(self.colors.highlight)
            .add_modifier(Modifier::BOLD)
    }
}
```

## Acceptance Criteria
- [ ] Visual consistency between old and new modes
- [ ] All colors match design system
- [ ] Theme resource accessible in all render systems

**Branch**: `tui-modernization/ws07-theme-styling`
