//! # Comparison Screen Input Handler
//!
//! Handles keyboard input for the side-by-side model comparison screen.

use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::resources::{
    comparison_state::ComparisonMode, ComparisonState, CurrentScreen, Screen,
};

/// Handle input for the Comparison screen
pub fn handle_comparison_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut comparison: ResMut<ComparisonState>,
    // TODO: Add ZmqClient resource for model fetching when integrated
) {
    if current_screen.0 != Screen::Comparison {
        return;
    }

    for event in events.read() {
        // Handle model browser overlay input separately
        if comparison.browsing_models {
            handle_model_browser_input(event.code, &mut comparison);
            continue;
        }

        // Main comparison screen input
        match comparison.mode {
            ComparisonMode::Dual => handle_dual_mode_input(event.code, &mut comparison),
            ComparisonMode::Multi => handle_multi_mode_input(event.code, &mut comparison),
        }
    }
}

/// Handle input when in dual comparison mode
fn handle_dual_mode_input(code: KeyCode, comparison: &mut ComparisonState) {
    match code {
        // Tab: Switch between left and right pane
        KeyCode::Tab => {
            comparison.toggle_pane();
            debug!(
                "Comparison: Switched to {:?} pane",
                comparison.selected_pane
            );
        }

        // 'm' or 'a': Open model browser for selected pane
        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Char('a') | KeyCode::Char('A') => {
            comparison.browsing_models = true;
            // TODO: Fetch available models via ZMQ if list is empty
            info!("Comparison: Opening model browser");
        }

        // 'd': Quick switch to next model for selected pane (if models available)
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if !comparison.available_models.is_empty() {
                comparison.next_available_model();
                comparison.select_current_model();
                info!("Comparison: Quick-switched model");
            }
        }

        // Enter: Start comparison generation
        KeyCode::Enter => {
            if comparison.can_run_comparison() && !comparison.is_running {
                comparison.start_dual_comparison();
                info!("Comparison: Starting dual comparison generation");
                // TODO: Submit generation jobs to backend via ZMQ
            } else if comparison.prompt.is_empty() {
                warn!("Comparison: Cannot run without prompt");
            } else {
                warn!("Comparison: Cannot run without models selected");
            }
        }

        // 'r' or 'c': Reset/clear comparison results
        KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Char('c') | KeyCode::Char('C') => {
            comparison.reset_results();
            info!("Comparison: Reset comparison results");
        }

        // Esc: Stop running comparison
        KeyCode::Esc => {
            if comparison.is_running {
                comparison.stop_comparison();
                info!("Comparison: Stopped comparison");
                // TODO: Cancel jobs via ZMQ
            }
        }

        // Arrow keys: Navigate available models (if browsing)
        KeyCode::Up => {
            if !comparison.available_models.is_empty() {
                comparison.previous_available_model();
            }
        }

        KeyCode::Down => {
            if !comparison.available_models.is_empty() {
                comparison.next_available_model();
            }
        }

        // Ignore other keys
        _ => {}
    }
}

/// Handle input when browsing models
fn handle_model_browser_input(code: KeyCode, comparison: &mut ComparisonState) {
    match code {
        // Arrow keys: Navigate model list
        KeyCode::Up => {
            comparison.previous_available_model();
            debug!("Model browser: Previous model");
        }

        KeyCode::Down => {
            comparison.next_available_model();
            debug!("Model browser: Next model");
        }

        // Enter: Select current model
        KeyCode::Enter => {
            comparison.select_current_model();
            info!("Model browser: Selected model");
        }

        // Esc: Close model browser
        KeyCode::Esc => {
            comparison.browsing_models = false;
            info!("Model browser: Closed");
        }

        // Ignore other keys
        _ => {}
    }
}

/// Handle input when in multi comparison mode (legacy)
fn handle_multi_mode_input(code: KeyCode, comparison: &mut ComparisonState) {
    match code {
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use crate::bevy_app::resources::comparison_state::ComparisonPane;

    #[test]
    fn test_comparison_input_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);
    }

    #[test]
    fn test_dual_mode_toggle_pane() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        assert_eq!(
            app.world().resource::<ComparisonState>().selected_pane,
            ComparisonPane::Left
        );

        // Send Tab key
        let key_event = KeyEvent {
            code: KeyCode::Tab,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        assert_eq!(
            app.world().resource::<ComparisonState>().selected_pane,
            ComparisonPane::Right
        );
    }

    #[test]
    fn test_dual_mode_open_model_browser() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        assert!(!app
            .world()
            .resource::<ComparisonState>()
            .browsing_models);

        // Send 'm' key
        let key_event = KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        assert!(app.world().resource::<ComparisonState>().browsing_models);
    }

    #[test]
    fn test_dual_mode_reset() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));

        let mut comparison = ComparisonState::default();
        comparison.left_image = Some(std::path::PathBuf::from("/test/left.png"));
        comparison.right_image = Some(std::path::PathBuf::from("/test/right.png"));
        comparison.is_running = true;

        app.insert_resource(comparison);
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        // Send 'r' key
        let key_event = KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        let comparison = app.world().resource::<ComparisonState>();
        assert!(comparison.left_image.is_none());
        assert!(comparison.right_image.is_none());
        assert!(!comparison.is_running);
    }

    #[test]
    fn test_model_browser_navigation() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));

        let mut comparison = ComparisonState::default();
        comparison.browsing_models = true;
        comparison.available_models = vec![
            crate::bevy_app::resources::comparison_state::ModelEntry {
                name: "Model A".to_string(),
                model_type: "base".to_string(),
                path: "/models/a".to_string(),
            },
            crate::bevy_app::resources::comparison_state::ModelEntry {
                name: "Model B".to_string(),
                model_type: "lora".to_string(),
                path: "/models/b".to_string(),
            },
        ];
        comparison.model_list_index = 0;

        app.insert_resource(comparison);
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        // Send Down key
        let key_event = KeyEvent {
            code: KeyCode::Down,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        assert_eq!(
            app.world().resource::<ComparisonState>().model_list_index,
            1
        );
    }

    #[test]
    fn test_model_browser_select() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));

        let mut comparison = ComparisonState::default();
        comparison.browsing_models = true;
        comparison.selected_pane = ComparisonPane::Left;
        comparison.available_models = vec![crate::bevy_app::resources::comparison_state::ModelEntry {
            name: "Test Model".to_string(),
            model_type: "base".to_string(),
            path: "/models/test".to_string(),
        }];
        comparison.model_list_index = 0;

        app.insert_resource(comparison);
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        // Send Enter key
        let key_event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        let comparison = app.world().resource::<ComparisonState>();
        assert_eq!(comparison.left_model, Some("Test Model".to_string()));
        assert!(!comparison.browsing_models);
    }

    #[test]
    fn test_only_handles_comparison_screen() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Generation)); // Different screen
        app.insert_resource(ComparisonState::default());
        app.add_event::<KeyEvent>();
        app.add_systems(Update, handle_comparison_input);

        let initial_pane = app.world().resource::<ComparisonState>().selected_pane;

        // Send Tab key
        let key_event = KeyEvent {
            code: KeyCode::Tab,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        app.world_mut().send_event(key_event);
        app.update();

        // Should not have changed since we're on wrong screen
        assert_eq!(
            app.world().resource::<ComparisonState>().selected_pane,
            initial_pane
        );
    }
}
