//! # Keyboard Input System
//!
//! Handles global keyboard shortcuts and special keys.

use bevy::prelude::{EventReader, info, Res, ResMut};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::resources::*;

/// Handle global keyboard input (quit, help, etc.).
pub fn handle_keyboard_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        match event.code {
            // Quit on 'q' (except on Generation screen where it's typing)
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if current_screen.0 != Screen::Generation {
                    info!("Quit requested via 'q' key");
                    app_state.quit();
                }
            }

            // Help screen
            KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => {
                // Help screen navigation handled by navigation system
            }

            // Debug mode preview tab switching
            KeyCode::Char('t') | KeyCode::Char('T') => {
                if app_state.debug_mode {
                    app_state.next_preview_tab();
                }
            }

            // Screen-specific number shortcuts handled by navigation system
            _ => {}
        }
    }
}

// Note: Unit tests for input systems require bevy_ratatui message system
// which isn't easily mockable. Integration tests in tests/input_system.rs
// provide coverage for these systems.
