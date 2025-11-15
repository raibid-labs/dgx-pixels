use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::bevy_app::resources::{CurrentScreen, ModelsState, Screen};

/// Handle input for Models screen
pub fn handle_models_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut models_state: ResMut<ModelsState>,
) {
    if current_screen.0 != Screen::Models {
        return;
    }

    for event in events.read() {
        match event.code {
            KeyCode::Down | KeyCode::Char('j') => {
                models_state.next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                models_state.previous();
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    models_state.next();
                }
            }
            KeyCode::PageUp => {
                for _ in 0..10 {
                    models_state.previous();
                }
            }
            KeyCode::Home => {
                models_state.selected_index = 0;
            }
            KeyCode::End => {
                if !models_state.models.is_empty() {
                    models_state.selected_index = models_state.models.len() - 1;
                }
            }
            KeyCode::Enter => {
                models_state.toggle_active();
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                models_state.download_selected();
            }
            KeyCode::Delete => {
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    models_state.delete_selected();
                }
            }
            KeyCode::Char('i') | KeyCode::Char('I') | KeyCode::Char(' ') => {
                models_state.toggle_metadata();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_models_input_compiles() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.insert_resource(CurrentScreen(Screen::Models));
        app.insert_resource(ModelsState::default());
        app.add_systems(Update, handle_models_input);
    }
}
