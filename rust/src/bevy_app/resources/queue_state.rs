//! # Queue State Resource
//!
//! Manages the job queue selection state for keyboard navigation.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::QueueState;
//!
//! fn navigate_queue(mut queue: ResMut<QueueState>) {
//!     queue.select_next();
//!     println!("Selected job index: {}", queue.selected);
//! }
//! ```

use bevy::prelude::*;

/// Queue state resource for job list navigation.
#[derive(Resource, Debug, Clone)]
pub struct QueueState {
    /// Currently selected job index (in combined list of all jobs)
    pub selected: usize,
    /// Last known total job count (for change detection)
    pub total_jobs: usize,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            selected: 0,
            total_jobs: 0,
        }
    }
}

impl QueueState {
    /// Select next job (wraps around).
    pub fn select_next(&mut self) {
        if self.total_jobs > 0 {
            self.selected = (self.selected + 1) % self.total_jobs;
        }
    }

    /// Select previous job (wraps around).
    pub fn select_previous(&mut self) {
        if self.total_jobs > 0 {
            self.selected = if self.selected == 0 {
                self.total_jobs - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Jump to first job.
    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    /// Jump to last job.
    pub fn select_last(&mut self) {
        if self.total_jobs > 0 {
            self.selected = self.total_jobs - 1;
        }
    }

    /// Update total job count and adjust selection if needed.
    pub fn update_total(&mut self, new_total: usize) {
        self.total_jobs = new_total;

        // Adjust selection if it's out of bounds
        if self.total_jobs == 0 {
            self.selected = 0;
        } else if self.selected >= self.total_jobs {
            self.selected = self.total_jobs - 1;
        }
    }

    /// Check if queue has jobs.
    pub fn has_jobs(&self) -> bool {
        self.total_jobs > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_queue_state() {
        let state = QueueState::default();
        assert_eq!(state.selected, 0);
        assert_eq!(state.total_jobs, 0);
        assert!(!state.has_jobs());
    }

    #[test]
    fn test_select_next() {
        let mut state = QueueState::default();
        state.update_total(3);

        assert_eq!(state.selected, 0);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_next();
        assert_eq!(state.selected, 2);
        state.select_next(); // Should wrap around
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_select_previous() {
        let mut state = QueueState::default();
        state.update_total(3);

        assert_eq!(state.selected, 0);
        state.select_previous(); // Should wrap to end
        assert_eq!(state.selected, 2);
        state.select_previous();
        assert_eq!(state.selected, 1);
        state.select_previous();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_select_first_last() {
        let mut state = QueueState::default();
        state.update_total(5);

        state.select_last();
        assert_eq!(state.selected, 4);

        state.select_first();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_update_total_adjusts_selection() {
        let mut state = QueueState::default();
        state.update_total(10);
        state.selected = 8;

        // Reduce total jobs below selected index
        state.update_total(5);
        assert_eq!(state.selected, 4); // Should clamp to last valid index

        // Set to empty
        state.update_total(0);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_navigation_with_no_jobs() {
        let mut state = QueueState::default();
        state.update_total(0);

        // Navigation should be safe with no jobs
        state.select_next();
        assert_eq!(state.selected, 0);

        state.select_previous();
        assert_eq!(state.selected, 0);

        state.select_last();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_has_jobs() {
        let mut state = QueueState::default();
        assert!(!state.has_jobs());

        state.update_total(1);
        assert!(state.has_jobs());

        state.update_total(0);
        assert!(!state.has_jobs());
    }
}
