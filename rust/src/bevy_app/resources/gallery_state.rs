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
use std::time::SystemTime;

/// Gallery state resource.
#[derive(Resource, Debug, Clone)]
pub struct GalleryState {
    /// All images in gallery
    pub images: Vec<PathBuf>,
    /// Currently selected image index
    pub selected: usize,
    /// Last time the gallery was updated (for change detection)
    pub last_updated: SystemTime,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            selected: 0,
            last_updated: SystemTime::now(),
        }
    }
}

impl GalleryState {
    /// Add image to gallery (if not already present).
    pub fn add_image(&mut self, path: PathBuf) {
        if !self.images.contains(&path) {
            self.images.push(path);
            self.last_updated = SystemTime::now();
        }
    }

    /// Remove image from gallery.
    pub fn remove_image(&mut self, path: &PathBuf) -> bool {
        if let Some(pos) = self.images.iter().position(|p| p == path) {
            self.images.remove(pos);
            self.last_updated = SystemTime::now();

            // Adjust selection if needed
            if self.selected >= self.images.len() && !self.images.is_empty() {
                self.selected = self.images.len() - 1;
            }
            true
        } else {
            false
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

    /// Clear all images from gallery.
    pub fn clear(&mut self) {
        self.images.clear();
        self.selected = 0;
        self.last_updated = SystemTime::now();
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
    fn test_remove_image() {
        let mut gallery = GalleryState::default();
        let path1 = PathBuf::from("image1.png");
        let path2 = PathBuf::from("image2.png");

        gallery.add_image(path1.clone());
        gallery.add_image(path2.clone());
        assert_eq!(gallery.len(), 2);

        let removed = gallery.remove_image(&path1);
        assert!(removed);
        assert_eq!(gallery.len(), 1);
        assert_eq!(gallery.images[0], path2);
    }

    #[test]
    fn test_remove_image_adjusts_selection() {
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("image1.png"));
        gallery.add_image(PathBuf::from("image2.png"));
        gallery.add_image(PathBuf::from("image3.png"));

        gallery.selected = 2; // Select last image
        gallery.remove_image(&PathBuf::from("image3.png"));

        assert_eq!(gallery.selected, 1); // Should adjust to last valid index
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

    #[test]
    fn test_clear() {
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("image1.png"));
        gallery.add_image(PathBuf::from("image2.png"));
        gallery.select_next();

        gallery.clear();
        assert_eq!(gallery.len(), 0);
        assert_eq!(gallery.selected, 0);
        assert!(gallery.is_empty());
    }
}
