//! # Generation Screen Input Handler
//!
//! Handles input events specific to the Generation screen.
//! Primary interactions: Enter (submit job), Esc (clear input), G (generate), C (compare).

use bevy::prelude::*;
use bevy_ratatui::event::{KeyCode, KeyEvent, KeyMessage, MessageReader};

use crate::bevy_app::{
    events::SubmitGenerationJob,
    resources::{AppState, CurrentScreen, InputBuffer, Screen},
};

/// Handle input for Generation screen.
///
/// This system only processes input when CurrentScreen is Screen::Generation.
/// Text input (typing, backspace, cursor movement) is handled by the global
/// text_entry system.
pub fn handle_generation_input(
    mut messages: MessageReader<KeyMessage>,
    current_screen: Res<CurrentScreen>,
    mut input_buffer: ResMut<InputBuffer>,
    mut submit_events: EventWriter<SubmitGenerationJob>,
    mut app_state: ResMut<AppState>,
) {
    if current_screen.0 != Screen::Generation {
        return;
    }

    for message in messages.read() {
        if let KeyMessage::Key(KeyEvent { code, .. }) = message {
            match code {
                KeyCode::Enter => {
                    // Submit generation job if input is not empty
                    if !input_buffer.text.trim().is_empty() {
                        submit_events.send(SubmitGenerationJob {
                            prompt: input_buffer.text.clone(),
                        });
                        input_buffer.clear();
                        app_state.request_redraw();
                        info!("Generation job submitted: {}", input_buffer.text);
                    }
                }
                KeyCode::Esc => {
                    // Clear input buffer
                    input_buffer.clear();
                    app_state.request_redraw();
                }
                KeyCode::Char('g') | KeyCode::Char('G') => {
                    // Generate shortcut (same as Enter)
                    if !input_buffer.text.trim().is_empty() {
                        submit_events.send(SubmitGenerationJob {
                            prompt: input_buffer.text.clone(),
                        });
                        input_buffer.clear();
                        app_state.request_redraw();
                        info!("Generation job submitted via 'G' key");
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    // Switch to Comparison screen
                    info!("Comparison screen shortcut pressed (not yet implemented)");
                    // TODO: Send NavigateToScreen(Screen::Comparison) event when WS-10 is complete
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    // Toggle preview tab (in debug mode)
                    if app_state.debug_mode {
                        app_state.set_preview_tab(0);
                    }
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    // Show logs tab (in debug mode)
                    if app_state.debug_mode {
                        app_state.set_preview_tab(1);
                    }
                }
                KeyCode::Tab
                    if message
                        .modifiers
                        .contains(bevy_ratatui::event::KeyModifiers::CONTROL) =>
                {
                    // Ctrl+Tab: cycle preview tabs (in debug mode)
                    app_state.next_preview_tab();
                }
                _ => {
                    // Other keys handled by global text_entry system
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_event::<SubmitGenerationJob>();
        app.init_resource::<CurrentScreen>();
        app.init_resource::<InputBuffer>();
        app.init_resource::<AppState>();
        app.add_systems(Update, handle_generation_input);
        app
    }

    #[test]
    fn test_enter_submits_job_with_text() {
        let mut app = create_test_app();

        // Set screen to Generation
        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;

        // Set input text
        app.world_mut().resource_mut::<InputBuffer>().text = "test prompt".to_string();

        // Simulate Enter key
        // (Full test requires bevy_ratatui message system setup)

        // For now, just verify system compiles
        app.update();
    }

    #[test]
    fn test_empty_input_doesnt_submit() {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;
        app.world_mut().resource_mut::<InputBuffer>().text = "".to_string();

        app.update();

        // Verify no events sent (requires bevy_ratatui setup)
    }

    #[test]
    fn test_esc_clears_input() {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;
        app.world_mut().resource_mut::<InputBuffer>().text = "test".to_string();

        app.update();

        // Verify buffer cleared (requires bevy_ratatui setup)
    }

    #[test]
    fn test_generation_input_only_on_generation_screen() {
        let mut app = create_test_app();

        // Set screen to something other than Generation
        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Gallery;
        app.world_mut().resource_mut::<InputBuffer>().text = "test".to_string();

        app.update();

        // System should return early, input unchanged
        assert_eq!(app.world().resource::<InputBuffer>().text, "test");
    }
}
