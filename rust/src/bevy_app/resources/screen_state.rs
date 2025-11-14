//! # Screen State Resource
//!
//! Tracks the currently active screen in the TUI application.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::CurrentScreen;
//!
//! fn my_system(screen: Res<CurrentScreen>) {
//!     println!("Current screen: {:?}", screen.0);
//! }
//! ```

use bevy::prelude::*;

/// Current active screen resource.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentScreen(pub Screen);

/// Represents the different screens in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Generation,
    Comparison,
    Queue,
    Gallery,
    Models,
    Monitor,
    Settings,
    Help,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        Self(Screen::Generation)
    }
}

impl Screen {
    /// Navigate to next screen (Tab key).
    pub fn next(self) -> Self {
        use Screen::*;
        match self {
            Generation => Comparison,
            Comparison => Queue,
            Queue => Gallery,
            Gallery => Models,
            Models => Monitor,
            Monitor => Settings,
            Settings => Help,
            Help => Generation,
        }
    }

    /// Navigate to previous screen (Shift+Tab).
    pub fn previous(self) -> Self {
        use Screen::*;
        match self {
            Generation => Help,
            Comparison => Generation,
            Queue => Comparison,
            Gallery => Queue,
            Models => Gallery,
            Monitor => Models,
            Settings => Monitor,
            Help => Settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_screen() {
        let screen = CurrentScreen::default();
        assert_eq!(screen.0, Screen::Generation);
    }

    #[test]
    fn test_screen_navigation_forward() {
        let mut screen = Screen::Generation;
        screen = screen.next();
        assert_eq!(screen, Screen::Comparison);
        screen = screen.next();
        assert_eq!(screen, Screen::Queue);
    }

    #[test]
    fn test_screen_navigation_backward() {
        let mut screen = Screen::Generation;
        screen = screen.previous();
        assert_eq!(screen, Screen::Help);
        screen = screen.previous();
        assert_eq!(screen, Screen::Settings);
    }

    #[test]
    fn test_screen_wraps_around() {
        let mut screen = Screen::Help;
        screen = screen.next();
        assert_eq!(screen, Screen::Generation);
    }
}
