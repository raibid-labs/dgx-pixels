//! # Help State Resource
//!
//! Manages scrolling and display state for the Help screen.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::HelpState;
//!
//! fn my_system(mut help_state: ResMut<HelpState>) {
//!     help_state.scroll_down(1);
//! }
//! ```

use bevy::prelude::*;

/// Help screen state resource.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct HelpState {
    /// Current scroll offset (line number from top).
    pub scroll_offset: usize,
    /// Total lines of help content.
    pub total_lines: usize,
    /// Visible lines in the viewport.
    pub visible_lines: usize,
}

impl Default for HelpState {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            total_lines: 0,
            visible_lines: 0,
        }
    }
}

impl HelpState {
    /// Scroll down by the specified number of lines.
    pub fn scroll_down(&mut self, lines: usize) {
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
    }

    /// Scroll up by the specified number of lines.
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Jump to the top of the content.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Jump to the bottom of the content.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total_lines.saturating_sub(self.visible_lines);
    }

    /// Scroll down by one page.
    pub fn page_down(&mut self) {
        let page_size = self.visible_lines.saturating_sub(1);
        self.scroll_down(page_size);
    }

    /// Scroll up by one page.
    pub fn page_up(&mut self) {
        let page_size = self.visible_lines.saturating_sub(1);
        self.scroll_up(page_size);
    }

    /// Update viewport dimensions.
    pub fn update_viewport(&mut self, visible_lines: usize, total_lines: usize) {
        self.visible_lines = visible_lines;
        self.total_lines = total_lines;
        // Ensure scroll offset is still valid
        let max_offset = total_lines.saturating_sub(visible_lines);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }

    /// Check if at top of content.
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Check if at bottom of content.
    pub fn is_at_bottom(&self) -> bool {
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);
        self.scroll_offset >= max_offset
    }

    /// Get current scroll percentage (0-100).
    pub fn scroll_percentage(&self) -> u8 {
        if self.total_lines <= self.visible_lines {
            return 100;
        }
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);
        if max_offset == 0 {
            100
        } else {
            ((self.scroll_offset as f32 / max_offset as f32) * 100.0) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_help_state() {
        let state = HelpState::default();
        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.total_lines, 0);
        assert_eq!(state.visible_lines, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut state = HelpState {
            scroll_offset: 0,
            total_lines: 100,
            visible_lines: 20,
        };
        state.scroll_down(5);
        assert_eq!(state.scroll_offset, 5);
    }

    #[test]
    fn test_scroll_down_clamped() {
        let mut state = HelpState {
            scroll_offset: 75,
            total_lines: 100,
            visible_lines: 20,
        };
        state.scroll_down(10);
        assert_eq!(state.scroll_offset, 80); // max is 100-20=80
    }

    #[test]
    fn test_scroll_up() {
        let mut state = HelpState {
            scroll_offset: 10,
            total_lines: 100,
            visible_lines: 20,
        };
        state.scroll_up(5);
        assert_eq!(state.scroll_offset, 5);
    }

    #[test]
    fn test_scroll_up_clamped() {
        let mut state = HelpState {
            scroll_offset: 3,
            total_lines: 100,
            visible_lines: 20,
        };
        state.scroll_up(5);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut state = HelpState {
            scroll_offset: 50,
            total_lines: 100,
            visible_lines: 20,
        };
        state.scroll_to_top();
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut state = HelpState {
            scroll_offset: 0,
            total_lines: 100,
            visible_lines: 20,
        };
        state.scroll_to_bottom();
        assert_eq!(state.scroll_offset, 80);
    }

    #[test]
    fn test_page_down() {
        let mut state = HelpState {
            scroll_offset: 0,
            total_lines: 100,
            visible_lines: 20,
        };
        state.page_down();
        assert_eq!(state.scroll_offset, 19); // visible_lines - 1
    }

    #[test]
    fn test_page_up() {
        let mut state = HelpState {
            scroll_offset: 20,
            total_lines: 100,
            visible_lines: 20,
        };
        state.page_up();
        assert_eq!(state.scroll_offset, 1);
    }

    #[test]
    fn test_update_viewport() {
        let mut state = HelpState {
            scroll_offset: 90,
            total_lines: 100,
            visible_lines: 20,
        };
        state.update_viewport(30, 100);
        assert_eq!(state.visible_lines, 30);
        assert_eq!(state.scroll_offset, 70); // clamped to 100-30
    }

    #[test]
    fn test_is_at_top() {
        let state = HelpState {
            scroll_offset: 0,
            total_lines: 100,
            visible_lines: 20,
        };
        assert!(state.is_at_top());

        let state = HelpState {
            scroll_offset: 5,
            total_lines: 100,
            visible_lines: 20,
        };
        assert!(!state.is_at_top());
    }

    #[test]
    fn test_is_at_bottom() {
        let state = HelpState {
            scroll_offset: 80,
            total_lines: 100,
            visible_lines: 20,
        };
        assert!(state.is_at_bottom());

        let state = HelpState {
            scroll_offset: 70,
            total_lines: 100,
            visible_lines: 20,
        };
        assert!(!state.is_at_bottom());
    }

    #[test]
    fn test_scroll_percentage() {
        let state = HelpState {
            scroll_offset: 0,
            total_lines: 100,
            visible_lines: 20,
        };
        assert_eq!(state.scroll_percentage(), 0);

        let state = HelpState {
            scroll_offset: 40,
            total_lines: 100,
            visible_lines: 20,
        };
        assert_eq!(state.scroll_percentage(), 50); // 40/80 * 100

        let state = HelpState {
            scroll_offset: 80,
            total_lines: 100,
            visible_lines: 20,
        };
        assert_eq!(state.scroll_percentage(), 100);
    }

    #[test]
    fn test_scroll_percentage_fits_on_screen() {
        let state = HelpState {
            scroll_offset: 0,
            total_lines: 10,
            visible_lines: 20,
        };
        assert_eq!(state.scroll_percentage(), 100);
    }
}
