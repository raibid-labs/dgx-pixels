//! # Text Entry System
//!
//! Handles text input on the Generation screen (prompt entry).
//!
//! Supports:
//! - Character input
//! - Backspace/Delete
//! - Cursor movement (Left/Right/Home/End)
//! - Word deletion (Ctrl+W)
//! - Clear to start (Ctrl+U)

use bevy::prelude::{info, EventReader, Res, ResMut};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyModifiers};

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
        let modifiers = event.modifiers;

        match event.code {
            // Character input
            KeyCode::Char('w') | KeyCode::Char('W') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+W: Delete word before cursor
                input_buffer.delete_word();
                app_state.request_redraw();
            }

            KeyCode::Char('u') | KeyCode::Char('U') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+U: Clear all text before cursor
                input_buffer.delete_to_start();
                app_state.request_redraw();
            }

            KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                // Regular character input (no Ctrl modifier)
                input_buffer.insert(c);
                app_state.request_redraw();
            }

            // Editing
            KeyCode::Backspace => {
                input_buffer.backspace();
                app_state.request_redraw();
            }

            KeyCode::Delete => {
                input_buffer.delete();
                app_state.request_redraw();
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
