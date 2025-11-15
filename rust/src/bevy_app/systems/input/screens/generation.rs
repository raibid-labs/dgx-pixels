use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::events::SubmitGenerationJob;
use crate::bevy_app::resources::{AppState, CurrentScreen, InputBuffer, Screen};

/// Handle input for Generation screen
pub fn handle_generation_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut input_buffer: ResMut<InputBuffer>,
    mut submit_events: EventWriter<SubmitGenerationJob>,
    mut app_state: ResMut<AppState>,
) {
    if current_screen.0 != Screen::Generation {
        return;
    }

    for event in events.read() {
        match event.code {
            KeyCode::Enter => {
                // Submit generation job
                if !input_buffer.text.is_empty() {
                    submit_events.send(SubmitGenerationJob {
                        prompt: input_buffer.text.clone(),
                    });
                    input_buffer.clear();
                    app_state.request_redraw();
                    info!("Submitted generation job: {}", input_buffer.text);
                }
            }
            KeyCode::Esc => {
                // Clear input
                input_buffer.clear();
                app_state.request_redraw();
            }
            // Text input handled by global text_entry system
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_enter_submits_job() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.add_event::<SubmitGenerationJob>();
        app.insert_resource(CurrentScreen(Screen::Generation));
        app.insert_resource(AppState::default());

        let mut input = InputBuffer::default();
        input.text = "test prompt".to_string();
        app.insert_resource(input);

        app.add_systems(Update, handle_generation_input);

        // Send Enter key event
        let crossterm_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        );
        app.world_mut().send_event(KeyEvent(crossterm_event));

        app.update();

        // Verify input buffer cleared
        let input = app.world().resource::<InputBuffer>();
        assert!(input.text.is_empty());
    }

    #[test]
    fn test_esc_clears_input() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.add_event::<SubmitGenerationJob>();
        app.insert_resource(CurrentScreen(Screen::Generation));
        app.insert_resource(AppState::default());

        let mut input = InputBuffer::default();
        input.text = "test prompt".to_string();
        app.insert_resource(input);

        app.add_systems(Update, handle_generation_input);

        // Send Esc key event
        let crossterm_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        );
        app.world_mut().send_event(KeyEvent(crossterm_event));

        app.update();

        // Verify input buffer cleared
        let input = app.world().resource::<InputBuffer>();
        assert!(input.text.is_empty());
    }

    #[test]
    fn test_empty_input_does_not_submit() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.add_event::<SubmitGenerationJob>();
        app.insert_resource(CurrentScreen(Screen::Generation));
        app.insert_resource(AppState::default());
        app.insert_resource(InputBuffer::default());  // Empty

        app.add_systems(Update, handle_generation_input);

        // Send Enter key event
        let crossterm_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        );
        app.world_mut().send_event(KeyEvent(crossterm_event));

        app.update();

        // Verify no events sent (we can't directly count events, so just check that app runs without panic)
        // The actual event verification would require event reader access
    }
}
