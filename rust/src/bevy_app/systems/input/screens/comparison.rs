use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::resources::{ComparisonState, CurrentScreen, Screen};

/// Handle input for the Comparison screen
pub fn handle_comparison_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut comparison: ResMut<ComparisonState>,
) {
    if current_screen.0 != Screen::Comparison {
        return;
    }

    for event in events.read() {
        match event.code {
            // Navigation: Arrow keys
            KeyCode::Left => {
                comparison.previous();
                debug!("Comparison: Navigate to previous model");
            }
            KeyCode::Right => {
                comparison.next();
                debug!("Comparison: Navigate to next model");
            }

            // Add model (future: open model selection dialog)
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // For now, add a placeholder model
                let new_model_id = comparison.models.len() + 1;
                comparison.add_model(format!("model-{}", new_model_id));
                info!("Comparison: Add model placeholder");
            }

            // Remove selected model
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => {
                comparison.remove_selected();
                info!("Comparison: Remove selected model");
            }

            // Run comparison
            KeyCode::Enter => {
                if !comparison.models.is_empty() && !comparison.prompt.is_empty() {
                    comparison.start_comparison();
                    info!("Comparison: Start comparison run");
                }
            }

            // Stop comparison
            KeyCode::Esc => {
                if comparison.is_running {
                    comparison.stop_comparison();
                    info!("Comparison: Stop comparison");
                }
            }

            // Clear all models
            KeyCode::Char('c') | KeyCode::Char('C') => {
                comparison.models.clear();
                comparison.selected_index = 0;
                info!("Comparison: Clear all models");
            }

            // Ignore other keys
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_comparison_input_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);
    }

    #[test]
    fn test_add_model() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        let initial_count = {
            let comparison = app.world().resource::<ComparisonState>();
            comparison.models.len()
        };

        // Send 'a' key
        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        let final_count = {
            let comparison = app.world().resource::<ComparisonState>();
            comparison.models.len()
        };

        assert_eq!(final_count, initial_count + 1);
    }

    #[test]
    fn test_navigation() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        // Send right arrow
        let key_event = KeyEvent {
            code: KeyCode::Right,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        let selected = {
            let comparison = app.world().resource::<ComparisonState>();
            comparison.selected_index
        };

        assert_eq!(selected, 1);
    }

    #[test]
    fn test_only_handles_comparison_screen() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Generation)); // Different screen
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        let initial_count = {
            let comparison = app.world().resource::<ComparisonState>();
            comparison.models.len()
        };

        // Send 'a' key
        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        let final_count = {
            let comparison = app.world().resource::<ComparisonState>();
            comparison.models.len()
        };

        // Should not have changed since we're on wrong screen
        assert_eq!(initial_count, final_count);
    }
}
