use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::events::CancelJob;
use crate::bevy_app::resources::{AppState, CurrentScreen, Screen};

/// Handle input for Queue screen
///
/// Future enhancement: Add Up/Down navigation to select jobs, c/C to cancel selected job.
pub fn handle_queue_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut cancel_events: EventWriter<CancelJob>,
    mut app_state: ResMut<AppState>,
) {
    if current_screen.0 != Screen::Queue {
        return;
    }

    for event in events.read() {
        match event.code {
            // Future: Add job navigation and cancellation
            // KeyCode::Up => { /* Navigate to previous job */ }
            // KeyCode::Down => { /* Navigate to next job */ }
            // KeyCode::Char('c') | KeyCode::Char('C') => { /* Cancel selected job */ }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_queue_input_compiles() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();
        app.insert_resource(CurrentScreen(Screen::Queue));
        app.insert_resource(AppState::default());
        app.add_systems(Update, handle_queue_input);
    }
}
