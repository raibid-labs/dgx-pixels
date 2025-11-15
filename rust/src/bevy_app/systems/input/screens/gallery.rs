//! # Gallery Input Handler
//!
//! WS-10: Gallery screen input handling for Bevy-Ratatui migration.
//! Handles keyboard navigation, image selection, and deletion.

use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::events::{DeleteImage, SelectNextImage, SelectPreviousImage};
use crate::bevy_app::resources::{CurrentScreen, GalleryState, Screen};

/// Gallery screen input handler.
///
/// Handles:
/// - Arrow keys: Navigate gallery
/// - Enter: Toggle detail view (future)
/// - d/D: Delete selected image
pub fn handle_gallery_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    gallery: Res<GalleryState>,
    mut select_next: EventWriter<SelectNextImage>,
    mut select_prev: EventWriter<SelectPreviousImage>,
    mut delete: EventWriter<DeleteImage>,
) {
    // Only handle input when on Gallery screen
    if current_screen.0 != Screen::Gallery {
        return;
    }

    for event in events.read() {
        match event.code {
            // Navigation: Arrow keys
            KeyCode::Up | KeyCode::Char('k') => {
                select_prev.send(SelectPreviousImage);
                debug!("Gallery: Navigate to previous image");
            }
            KeyCode::Down | KeyCode::Char('j') => {
                select_next.send(SelectNextImage);
                debug!("Gallery: Navigate to next image");
            }

            // Future: Enter for detail view toggle
            KeyCode::Enter => {
                debug!("Gallery: Detail view toggle (not yet implemented)");
            }

            // Delete current image
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(image_path) = gallery.current_image() {
                    delete.send(DeleteImage {
                        image_path: image_path.clone(),
                    });
                    info!("Gallery: Delete requested for {:?}", image_path);
                }
            }

            // Home/End for quick navigation
            KeyCode::Home => {
                // Jump to first image by sending multiple prev events
                // This is a simple approach; could be optimized with a JumpToFirst event
                for _ in 0..gallery.selected {
                    select_prev.send(SelectPreviousImage);
                }
                debug!("Gallery: Jump to first image");
            }
            KeyCode::End => {
                // Jump to last image by sending multiple next events
                let remaining = gallery.len().saturating_sub(gallery.selected + 1);
                for _ in 0..remaining {
                    select_next.send(SelectNextImage);
                }
                debug!("Gallery: Jump to last image");
            }

            // Page Up/Down for faster navigation
            KeyCode::PageUp => {
                for _ in 0..5 {
                    select_prev.send(SelectPreviousImage);
                }
                debug!("Gallery: Page up (5 images)");
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    select_next.send(SelectNextImage);
                }
                debug!("Gallery: Page down (5 images)");
            }

            // Ignore other keys (let main keyboard handler process them)
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use std::path::PathBuf;

    fn create_key_event(code: KeyCode) -> KeyEvent {
        crossterm::event::KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn test_gallery_navigation_events() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Gallery));
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("/test/img1.png"));
        gallery.add_image(PathBuf::from("/test/img2.png"));
        app.insert_resource(gallery);

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<SelectNextImage>();
        app.add_event::<SelectPreviousImage>();
        app.add_event::<DeleteImage>();

        // Add system
        app.add_systems(Update, handle_gallery_input);

        // Simulate down arrow key press
        app.world_mut().send_event(create_key_event(KeyCode::Down));

        app.update();

        // Verify SelectNextImage event was sent
        let mut next_events = app.world_mut().resource_mut::<Events<SelectNextImage>>();
        let mut reader = next_events.get_cursor();
        assert_eq!(reader.read(&next_events).count(), 1);
    }

    #[test]
    fn test_delete_event() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Gallery));
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("/test/img1.png"));
        app.insert_resource(gallery);

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<SelectNextImage>();
        app.add_event::<SelectPreviousImage>();
        app.add_event::<DeleteImage>();

        // Add system
        app.add_systems(Update, handle_gallery_input);

        // Simulate 'd' key press
        app.world_mut()
            .send_event(create_key_event(KeyCode::Char('d')));

        app.update();

        // Verify DeleteImage event was sent
        let mut delete_events = app.world_mut().resource_mut::<Events<DeleteImage>>();
        let mut reader = delete_events.get_cursor();
        let events: Vec<_> = reader.read(&delete_events).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].image_path, PathBuf::from("/test/img1.png"));
    }

    #[test]
    fn test_no_events_on_other_screens() {
        let mut app = App::new();

        // Setup resources - DIFFERENT screen
        app.insert_resource(CurrentScreen(Screen::Generation));
        app.insert_resource(GalleryState::default());

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<SelectNextImage>();
        app.add_event::<SelectPreviousImage>();
        app.add_event::<DeleteImage>();

        // Add system
        app.add_systems(Update, handle_gallery_input);

        // Simulate down arrow key press
        app.world_mut().send_event(create_key_event(KeyCode::Down));

        app.update();

        // Verify NO events were sent (different screen)
        let mut next_events = app.world_mut().resource_mut::<Events<SelectNextImage>>();
        let mut reader = next_events.get_cursor();
        assert_eq!(reader.read(&next_events).count(), 0);
    }

    #[test]
    fn test_page_navigation() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Gallery));
        let mut gallery = GalleryState::default();
        for i in 0..20 {
            gallery.add_image(PathBuf::from(format!("/test/img{}.png", i)));
        }
        app.insert_resource(gallery);

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<SelectNextImage>();
        app.add_event::<SelectPreviousImage>();
        app.add_event::<DeleteImage>();

        // Add system
        app.add_systems(Update, handle_gallery_input);

        // Simulate PageDown key press
        app.world_mut()
            .send_event(create_key_event(KeyCode::PageDown));

        app.update();

        // Verify 5 SelectNextImage events were sent
        let mut next_events = app.world_mut().resource_mut::<Events<SelectNextImage>>();
        let mut reader = next_events.get_cursor();
        assert_eq!(reader.read(&next_events).count(), 5);
    }
}
