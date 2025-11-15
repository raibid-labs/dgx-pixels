use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;

use crate::bevy_app::resources::{CurrentScreen, Screen};

/// Handle input for Help screen
///
/// The Help screen is informational only - all navigation is handled by
/// the main keyboard handler. This handler exists for consistency with
/// other screens.
pub fn handle_help_input(mut events: EventReader<KeyEvent>, current_screen: Res<CurrentScreen>) {
    if current_screen.0 != Screen::Help {
        return;
    }

    for event in events.read() {
        match event.code {
            // Help screen is read-only
            // All navigation (Tab/Shift+Tab/q) is handled by main keyboard handler
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_help_input_compiles() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.insert_resource(CurrentScreen(Screen::Help));
        app.add_systems(Update, handle_help_input);
    }
}
