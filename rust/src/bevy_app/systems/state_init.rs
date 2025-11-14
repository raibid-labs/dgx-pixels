//! # State Initialization System
//!
//! Initializes all application state resources at startup.

use bevy::prelude::*;

use crate::bevy_app::resources::*;

/// Initialize all application state resources.
///
/// This system runs during the Startup schedule and populates
/// the Bevy world with default resource instances.
pub fn init_app_state(mut commands: Commands) {
    info!("Initializing application state resources");

    // Insert all resource defaults
    commands.insert_resource(AppState::default());
    commands.insert_resource(CurrentScreen::default());
    commands.insert_resource(InputBuffer::default());
    commands.insert_resource(GalleryState::default());
    commands.insert_resource(JobTracker::default());

    info!("Application state resources initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_app_state() {
        let mut app = App::new();
        app.add_systems(Startup, init_app_state);
        app.update();

        // Verify all resources are present
        assert!(app.world().contains_resource::<AppState>());
        assert!(app.world().contains_resource::<CurrentScreen>());
        assert!(app.world().contains_resource::<InputBuffer>());
        assert!(app.world().contains_resource::<GalleryState>());
        assert!(app.world().contains_resource::<JobTracker>());
    }
}
