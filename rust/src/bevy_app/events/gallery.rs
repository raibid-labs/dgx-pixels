//! # Gallery Events
//!
//! Events for gallery navigation and image management.

use bevy::prelude::*;

/// Event to select next image in gallery.
#[derive(Event, Debug, Clone)]
pub struct SelectNextImage;

/// Event to select previous image in gallery.
#[derive(Event, Debug, Clone)]
pub struct SelectPreviousImage;

/// Event to delete an image from gallery.
#[derive(Event, Debug, Clone)]
pub struct DeleteImage {
    pub image_path: std::path::PathBuf,
}

/// Event handler for gallery events.
pub fn handle_gallery_events(
    mut next_events: EventReader<SelectNextImage>,
    mut prev_events: EventReader<SelectPreviousImage>,
    mut delete_events: EventReader<DeleteImage>,
    mut gallery: ResMut<crate::bevy_app::resources::GalleryState>,
) {
    for _ in next_events.read() {
        gallery.select_next();
        info!("Gallery: selected next image");
    }

    for _ in prev_events.read() {
        gallery.select_previous();
        info!("Gallery: selected previous image");
    }

    for event in delete_events.read() {
        info!("Delete image requested: {:?}", event.image_path);
        // TODO: Implement image deletion
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_gallery_navigation() {
        let mut app = App::new();
        app.add_event::<SelectNextImage>();
        app.add_event::<SelectPreviousImage>();
        app.add_event::<DeleteImage>();
        app.insert_resource(crate::bevy_app::resources::GalleryState::default());
        app.add_systems(Update, handle_gallery_events);

        app.world_mut().send_event(SelectNextImage);
        app.update();
        // No assertion - just verify no panic
    }
}
