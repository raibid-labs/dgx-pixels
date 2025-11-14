use crate::app::{App, Screen};
use crate::events::{is_ctrl_c, key_match, AppEvent};
use crossterm::event::KeyCode;

/// Event handler for the application
pub struct EventHandler;

impl EventHandler {
    /// Handle an application event
    pub fn handle(app: &mut App, event: AppEvent) {
        match event {
            AppEvent::Key(key) => Self::handle_key(app, key),
            AppEvent::Resize(_, _) => {
                app.needs_redraw = true;
            }
            AppEvent::Tick => {
                // Periodic updates if needed
            }
            AppEvent::Mouse => {
                // Mouse events (future)
            }
        }
    }

    /// Handle keyboard input
    fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
        // Global keys (work on all screens)
        if is_ctrl_c(&key) || key_match(&key, KeyCode::Char('q')) {
            app.quit();
            return;
        }

        if key_match(&key, KeyCode::Esc) {
            app.navigate_back();
            return;
        }

        if key_match(&key, KeyCode::Char('?')) || key_match(&key, KeyCode::Char('h')) {
            app.navigate_to(Screen::Help);
            return;
        }

        // Screen-specific keys take priority on input screens
        // (Generation and Comparison screens have text input)
        if matches!(app.current_screen, Screen::Generation | Screen::Comparison) {
            Self::handle_screen_specific(app, key);
            return;
        }

        // Screen navigation keys (only when NOT on input screens)
        match key.code {
            KeyCode::Char('1') => app.navigate_to(Screen::Generation),
            KeyCode::Char('2') => app.navigate_to(Screen::Queue),
            KeyCode::Char('3') => app.navigate_to(Screen::Gallery),
            KeyCode::Char('4') => app.navigate_to(Screen::Models),
            KeyCode::Char('5') => app.navigate_to(Screen::Monitor),
            KeyCode::Char('6') => app.navigate_to(Screen::Settings),
            _ => Self::handle_screen_specific(app, key),
        }
    }

    /// Handle screen-specific keyboard input
    fn handle_screen_specific(app: &mut App, key: crossterm::event::KeyEvent) {
        match app.current_screen {
            Screen::Generation => Self::handle_generation_keys(app, key),
            Screen::Comparison => Self::handle_comparison_keys(app, key),
            Screen::Queue => Self::handle_queue_keys(app, key),
            Screen::Gallery => Self::handle_gallery_keys(app, key),
            Screen::Models => Self::handle_models_keys(app, key),
            Screen::Monitor => Self::handle_monitor_keys(app, key),
            Screen::Settings => Self::handle_settings_keys(app, key),
            Screen::Help => Self::handle_help_keys(app, key),
        }
    }

    fn handle_generation_keys(app: &mut App, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                Self::trigger_generation(app);
            }
            KeyCode::Char(c) => app.input_char(c),
            KeyCode::Backspace => app.input_backspace(),
            _ => {}
        }
    }

    fn trigger_generation(app: &mut App) {
        use crate::messages::Request;
        use tracing::{info, warn};

        if app.input_buffer.trim().is_empty() {
            warn!("Cannot generate: prompt is empty");
            return;
        }

        if app.zmq_client.is_none() {
            warn!("Cannot generate: backend not connected");
            return;
        }

        let job_id = format!("job-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());
        let prompt = app.input_buffer.trim().to_string();

        info!("Triggering generation: {}", prompt);

        let request = Request::Generate {
            id: job_id.clone(),
            prompt: prompt.clone(),
            model: "sd_xl_base_1.0".to_string(),
            lora: None,
            size: (1024, 1024),
            steps: 30,
            cfg_scale: 7.5,
        };

        if let Some(ref client) = app.zmq_client {
            match client.send_request(request) {
                Ok(_) => {
                    info!("Generation request sent: {}", job_id);
                    app.add_job(job_id, prompt);
                    app.input_buffer.clear();
                    app.cursor_pos = 0;
                    app.needs_redraw = true;
                }
                Err(e) => {
                    warn!("Failed to send generation request: {}", e);
                }
            }
        }
    }

    fn handle_comparison_keys(app: &mut App, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(c) => app.input_char(c),
            KeyCode::Backspace => app.input_backspace(),
            _ => {}
        }
    }

    fn handle_queue_keys(_app: &mut App, _key: crossterm::event::KeyEvent) {
        // TODO: Implement queue-specific keys
    }

    fn handle_gallery_keys(app: &mut App, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Left => app.gallery_prev(),
            KeyCode::Right => app.gallery_next(),
            _ => {}
        }
    }

    fn handle_models_keys(_app: &mut App, _key: crossterm::event::KeyEvent) {
        // TODO: Implement model-specific keys
    }

    fn handle_monitor_keys(_app: &mut App, _key: crossterm::event::KeyEvent) {
        // TODO: Implement monitor-specific keys
    }

    fn handle_settings_keys(_app: &mut App, _key: crossterm::event::KeyEvent) {
        // TODO: Implement settings-specific keys
    }

    fn handle_help_keys(_app: &mut App, _key: crossterm::event::KeyEvent) {
        // Help screen is read-only, no specific keys needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    #[tokio::test]
    async fn test_quit_on_q() {
        let mut app = App::new();
        let event = AppEvent::Key(KeyEvent::from(KeyCode::Char('q')));
        EventHandler::handle(&mut app, event);
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_quit_on_ctrl_c() {
        let mut app = App::new();
        let event = AppEvent::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        EventHandler::handle(&mut app, event);
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_screen_navigation() {
        let mut app = App::new();

        let event = AppEvent::Key(KeyEvent::from(KeyCode::Char('2')));
        EventHandler::handle(&mut app, event);
        assert_eq!(app.current_screen, Screen::Queue);

        let event = AppEvent::Key(KeyEvent::from(KeyCode::Char('3')));
        EventHandler::handle(&mut app, event);
        assert_eq!(app.current_screen, Screen::Gallery);
    }

    #[tokio::test]
    async fn test_navigate_back() {
        let mut app = App::new();

        let event = AppEvent::Key(KeyEvent::from(KeyCode::Char('2')));
        EventHandler::handle(&mut app, event);

        let event = AppEvent::Key(KeyEvent::from(KeyCode::Esc));
        EventHandler::handle(&mut app, event);
        assert_eq!(app.current_screen, Screen::Generation);
    }

    #[tokio::test]
    async fn test_help_screen() {
        let mut app = App::new();
        let event = AppEvent::Key(KeyEvent::from(KeyCode::Char('?')));
        EventHandler::handle(&mut app, event);
        assert_eq!(app.current_screen, Screen::Help);
    }
}
