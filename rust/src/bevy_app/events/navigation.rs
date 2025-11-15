//! # Navigation Events
//!
//! Events for screen navigation and UI state changes.

use crate::bevy_app::resources::Screen;
use bevy::prelude::*;

/// Event to navigate to a specific screen.
#[derive(Event, Debug, Clone)]
pub struct NavigateToScreen(pub Screen);

/// Event to go back to previous screen.
#[derive(Event, Debug, Clone)]
pub struct NavigateBack;

/// Event handler for screen navigation.
pub fn handle_navigation_events(
    mut nav_events: EventReader<NavigateToScreen>,
    mut back_events: EventReader<NavigateBack>,
    mut current_screen: ResMut<crate::bevy_app::resources::CurrentScreen>,
    mut app_state: ResMut<crate::bevy_app::resources::AppState>,
) {
    // Handle direct navigation
    for event in nav_events.read() {
        info!("Navigating to screen: {:?}", event.0);
        current_screen.0 = event.0;
        app_state.request_redraw();
    }

    // Handle back navigation
    for _ in back_events.read() {
        info!("Navigating back to Generation screen");
        current_screen.0 = Screen::Generation;
        app_state.request_redraw();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_navigation_event() {
        let mut app = App::new();
        app.add_event::<NavigateToScreen>();
        app.add_event::<NavigateBack>();
        app.insert_resource(crate::bevy_app::resources::CurrentScreen::default());
        app.insert_resource(crate::bevy_app::resources::AppState::default());
        app.add_systems(Update, handle_navigation_events);

        // Send navigation event
        app.world_mut()
            .send_event(NavigateToScreen(Screen::Gallery));
        app.update();

        // Verify screen changed
        let screen = app
            .world()
            .resource::<crate::bevy_app::resources::CurrentScreen>();
        assert_eq!(screen.0, Screen::Gallery);
    }

    #[test]
    fn test_back_navigation() {
        let mut app = App::new();
        app.add_event::<NavigateToScreen>();
        app.add_event::<NavigateBack>();
        app.insert_resource(crate::bevy_app::resources::CurrentScreen(Screen::Gallery));
        app.insert_resource(crate::bevy_app::resources::AppState::default());
        app.add_systems(Update, handle_navigation_events);

        // Send back event
        app.world_mut().send_event(NavigateBack);
        app.update();

        // Verify back to Generation
        let screen = app
            .world()
            .resource::<crate::bevy_app::resources::CurrentScreen>();
        assert_eq!(screen.0, Screen::Generation);
    }
}
