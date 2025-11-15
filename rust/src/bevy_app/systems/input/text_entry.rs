//! # Text Entry System
//!
//! Handles text input on the Generation screen (prompt entry).

use bevy::prelude::{info, EventReader, Res, ResMut};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::resources::*;

/// Handle text input for prompt entry.
pub fn handle_text_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut input_buffer: ResMut<InputBuffer>,
    mut app_state: ResMut<AppState>,
) {
    // Only process text input on Generation screen
    if current_screen.0 != Screen::Generation {
        return;
    }

    for event in events.read() {
        match event.code {
            // Character input
            KeyCode::Char(c) => {
                // Skip 'q' and 'Q' as they might be for quitting
                // (though keyboard system already handles this by checking screen)
                input_buffer.insert(c);
                app_state.request_redraw();
            }

            // Editing
            KeyCode::Backspace => {
                input_buffer.backspace();
                app_state.request_redraw();
            }

            KeyCode::Delete => {
                // Delete character after cursor
                let cursor = input_buffer.cursor;
                let text_len = input_buffer.text.len();
                if cursor < text_len {
                    input_buffer.text.remove(cursor);
                    app_state.request_redraw();
                }
            }

            // Cursor movement
            KeyCode::Left => {
                input_buffer.move_left();
                app_state.request_redraw();
            }

            KeyCode::Right => {
                input_buffer.move_right();
                app_state.request_redraw();
            }

            KeyCode::Home => {
                input_buffer.move_to_start();
                app_state.request_redraw();
            }

            KeyCode::End => {
                input_buffer.move_to_end();
                app_state.request_redraw();
            }

            // Submit (handled by WS-08 event system)
            KeyCode::Enter => {
                // TODO: WS-08 will handle job submission
                info!("Enter pressed - job submission pending WS-08");
            }

            _ => {}
        }
    }
}

// Note: Unit tests for input systems require bevy_ratatui message system
// which isn't easily mockable. Integration tests in tests/input_system.rs
// provide coverage for these systems.
