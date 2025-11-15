//! # Navigation System
//!
//! Handles screen navigation via Tab, number keys, and Escape.

use bevy::prelude::{EventReader, info, ResMut};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::resources::*;

/// Handle screen navigation input.
pub fn handle_navigation(
    mut events: EventReader<KeyEvent>,
    mut current_screen: ResMut<CurrentScreen>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        match event.code {
            // Tab navigation
            KeyCode::Tab => {
                current_screen.0 = current_screen.0.next();
                app_state.request_redraw();
                info!("Navigated to screen: {:?}", current_screen.0);
            }

            KeyCode::BackTab => {
                current_screen.0 = current_screen.0.previous();
                app_state.request_redraw();
                info!("Navigated to screen: {:?}", current_screen.0);
            }

            // Number key shortcuts
            KeyCode::Char('1') => {
                current_screen.0 = Screen::Generation;
                app_state.request_redraw();
            }
            KeyCode::Char('2') => {
                current_screen.0 = Screen::Comparison;
                app_state.request_redraw();
            }
            KeyCode::Char('3') => {
                current_screen.0 = Screen::Queue;
                app_state.request_redraw();
            }
            KeyCode::Char('4') => {
                current_screen.0 = Screen::Gallery;
                app_state.request_redraw();
            }
            KeyCode::Char('5') => {
                current_screen.0 = Screen::Models;
                app_state.request_redraw();
            }
            KeyCode::Char('6') => {
                current_screen.0 = Screen::Monitor;
                app_state.request_redraw();
            }
            KeyCode::Char('7') => {
                current_screen.0 = Screen::Settings;
                app_state.request_redraw();
            }
            KeyCode::Char('8') => {
                current_screen.0 = Screen::Help;
                app_state.request_redraw();
            }

            // Escape - navigate back (using history in future)
            KeyCode::Esc => {
                // For now, just go to Generation screen
                // WS-08 will add proper screen history
                current_screen.0 = Screen::Generation;
                app_state.request_redraw();
            }

            _ => {}
        }
    }
}

// Note: Unit tests for input systems require bevy_ratatui message system
// which isn't easily mockable. Integration tests in tests/input_system.rs
// provide coverage for these systems.
