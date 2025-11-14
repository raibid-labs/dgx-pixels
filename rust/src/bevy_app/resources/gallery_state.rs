//! # Gallery State Resource
//!
//! Manages the gallery of generated images and selection state.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::GalleryState;
//!
//! fn navigate_gallery(mut gallery: ResMut<GalleryState>) {
//!     gallery.select_next();
//!     if let Some(path) = gallery.current_image() {
//!         println!("Selected: {:?}", path);
//!     }
//! }
//! ```

use bevy::prelude::*;
use std::path::PathBuf;

/// Gallery state resource.
#[derive(Resource, Debug, Clone)]
pub struct GalleryState {
    /// All images in gallery
    pub images: Vec<PathBuf>,
    /// Currently selected image index
    pub selected: usize,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            selected: 0,
        }
    }
}

impl GalleryState {
    /// Add image to gallery (if not already present).
    pub fn add_image(&mut self, path: PathBuf) {
        if !self.images.contains(&path) {
            self.images.push(path);
        }
    }

    /// Select next image (wraps around).
    pub fn select_next(&mut self) {
        if !self.images.is_empty() {
            self.selected = (self.selected + 1) % self.images.len();
        }
    }

    /// Select previous image (wraps around).
    pub fn select_previous(&mut self) {
        if !self.images.is_empty() {
            self.selected = if self.selected == 0 {
                self.images.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Get currently selected image path.
    pub fn current_image(&self) -> Option<&PathBuf> {
        self.images.get(self.selected)
    }

    /// Get total number of images.
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Check if gallery is empty.
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_gallery() {
        let gallery = GalleryState::default();
        assert_eq!(gallery.images.len(), 0);
        assert_eq!(gallery.selected, 0);
        assert!(gallery.is_empty());
    }

    #[test]
    fn test_add_image() {
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("image1.png"));
        gallery.add_image(PathBuf::from("image2.png"));

        assert_eq!(gallery.len(), 2);
        assert!(!gallery.is_empty());
    }

    #[test]
    fn test_add_duplicate_image() {
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("image1.png"));
        gallery.add_image(PathBuf::from("image1.png"));

        assert_eq!(gallery.len(), 1); // Should not add duplicate
    }

    #[test]
    fn test_select_next() {
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("image1.png"));
        gallery.add_image(PathBuf::from("image2.png"));
        gallery.add_image(PathBuf::from("image3.png"));

        assert_eq!(gallery.selected, 0);
        gallery.select_next();
        assert_eq!(gallery.selected, 1);
        gallery.select_next();
        assert_eq!(gallery.selected, 2);
        gallery.select_next(); // Should wrap around
        assert_eq!(gallery.selected, 0);
    }

    #[test]
    fn test_select_previous() {
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("image1.png"));
        gallery.add_image(PathBuf::from("image2.png"));
        gallery.add_image(PathBuf::from("image3.png"));

        assert_eq!(gallery.selected, 0);
        gallery.select_previous(); // Should wrap around to end
        assert_eq!(gallery.selected, 2);
        gallery.select_previous();
        assert_eq!(gallery.selected, 1);
    }

    #[test]
    fn test_current_image() {
        let mut gallery = GalleryState::default();
        assert_eq!(gallery.current_image(), None);

        gallery.add_image(PathBuf::from("image1.png"));
        assert_eq!(gallery.current_image(), Some(&PathBuf::from("image1.png")));

        gallery.add_image(PathBuf::from("image2.png"));
        gallery.select_next();
        assert_eq!(gallery.current_image(), Some(&PathBuf::from("image2.png")));
    }
}
