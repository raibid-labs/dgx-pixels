use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::resources::{CurrentScreen, Screen};

/// Handle input for the Monitor screen
pub fn handle_monitor_input(mut events: EventReader<KeyEvent>, current_screen: Res<CurrentScreen>) {
    if current_screen.0 != Screen::Monitor {
        return;
    }

    for event in events.read() {
        match event.code {
            // 'r' to refresh (future: trigger manual refresh)
            KeyCode::Char('r') | KeyCode::Char('R') => {
                debug!("Monitor: Refresh requested (not yet implemented)");
            }

            // 'p' to pause auto-refresh (future feature)
            KeyCode::Char('p') | KeyCode::Char('P') => {
                debug!("Monitor: Pause auto-refresh (not yet implemented)");
            }

            // Ignore other keys (let main keyboard handler process them)
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_monitor_input_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Monitor));
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_monitor_input);
    }

    #[test]
    fn test_only_handles_monitor_screen() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Generation)); // Different screen
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_monitor_input);

        // Send 'r' key
        let key_event = KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        // Should not crash or cause issues (just ignored)
    }
}
