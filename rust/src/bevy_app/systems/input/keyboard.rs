//! # Keyboard Input System
//!
//! Handles global keyboard shortcuts and special keys.

use bevy::prelude::{info, EventReader, Res, ResMut};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyModifiers};
use std::path::PathBuf;

use crate::bevy_app::resources::*;

/// Handle global keyboard input (quit, help, etc.).
pub fn handle_keyboard_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        // Handle Ctrl+T to load test preview
        if event.modifiers.contains(KeyModifiers::CONTROL) {
            match event.code {
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    let test_path = PathBuf::from("outputs/test_sprite.png");
                    if test_path.exists() {
                        info!("Loading test preview: {:?}", test_path);
                        app_state.current_preview = Some(test_path);
                        app_state.needs_redraw = true;
                    } else {
                        info!("Test preview not found: {:?}", test_path);
                    }
                }
                _ => {}
            }
        } else {
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
}

// Note: Unit tests for input systems require bevy_ratatui message system
// which isn't easily mockable. Integration tests in tests/input_system.rs
// provide coverage for these systems.
