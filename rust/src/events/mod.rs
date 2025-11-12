pub mod handler;

pub use handler::EventHandler;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Application events
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AppEvent {
    /// Key press event
    Key(KeyEvent),

    /// Tick event for periodic updates
    Tick,

    /// Resize event
    Resize(u16, u16),

    /// Mouse event (future use)
    Mouse,
}

impl From<crossterm::event::Event> for AppEvent {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Key(key) => AppEvent::Key(key),
            crossterm::event::Event::Resize(w, h) => AppEvent::Resize(w, h),
            crossterm::event::Event::Mouse(_) => AppEvent::Mouse,
            _ => AppEvent::Tick,
        }
    }
}

/// Helper to check if a key event matches a specific key
pub fn key_match(event: &KeyEvent, code: KeyCode) -> bool {
    event.code == code && event.modifiers == KeyModifiers::NONE
}

/// Helper to check if Ctrl+C was pressed
pub fn is_ctrl_c(event: &KeyEvent) -> bool {
    event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_match() {
        let event = KeyEvent::from(KeyCode::Char('q'));
        assert!(key_match(&event, KeyCode::Char('q')));
        assert!(!key_match(&event, KeyCode::Char('w')));
    }

    #[test]
    fn test_ctrl_c() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_ctrl_c(&event));

        let event = KeyEvent::from(KeyCode::Char('c'));
        assert!(!is_ctrl_c(&event));
    }
}
