use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::resources::{AppState, CurrentScreen, Screen};

/// Handle input for Settings screen
///
/// Currently read-only - settings are loaded from config file.
/// Future enhancement: Add interactive editing with s/S to save.
pub fn handle_settings_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut app_state: ResMut<AppState>,
) {
    if current_screen.0 != Screen::Settings {
        return;
    }

    for event in events.read() {
        match event.code {
            // Settings screen is currently read-only
            // Future: Add navigation (Up/Down), editing (Enter), save (s/S)
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_settings_input_compiles() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.insert_resource(CurrentScreen(Screen::Settings));
        app.insert_resource(AppState::default());
        app.add_systems(Update, handle_settings_input);
    }
}
