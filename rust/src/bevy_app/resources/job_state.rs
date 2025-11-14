//! # Job Tracker Resource
//!
//! Tracks aggregate statistics about image generation jobs.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::JobTracker;
//!
//! fn show_job_stats(tracker: Res<JobTracker>) {
//!     println!("Active jobs: {}", tracker.active_jobs());
//!     println!("Completed: {}/{}", tracker.total_completed, tracker.total_submitted);
//! }
//! ```

use bevy::prelude::*;

/// Job tracking resource for aggregate statistics.
#[derive(Resource, Debug, Clone, Default)]
pub struct JobTracker {
    /// Total jobs submitted this session
    pub total_submitted: usize,
    /// Total jobs completed successfully
    pub total_completed: usize,
    /// Total jobs that failed
    pub total_failed: usize,
}

impl JobTracker {
    /// Record a new job submission.
    pub fn submit_job(&mut self) {
        self.total_submitted += 1;
    }

    /// Record a job completion.
    pub fn complete_job(&mut self) {
        self.total_completed += 1;
    }

    /// Record a job failure.
    pub fn fail_job(&mut self) {
        self.total_failed += 1;
    }

    /// Get count of currently active jobs.
    pub fn active_jobs(&self) -> usize {
        self.total_submitted
            .saturating_sub(self.total_completed)
            .saturating_sub(self.total_failed)
    }

    /// Get success rate as percentage (0.0 - 100.0).
    pub fn success_rate(&self) -> f32 {
        let finished = self.total_completed + self.total_failed;
        if finished == 0 {
            0.0
        } else {
            (self.total_completed as f32 / finished as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tracker() {
        let tracker = JobTracker::default();
        assert_eq!(tracker.total_submitted, 0);
        assert_eq!(tracker.total_completed, 0);
        assert_eq!(tracker.total_failed, 0);
        assert_eq!(tracker.active_jobs(), 0);
    }

    #[test]
    fn test_submit_jobs() {
        let mut tracker = JobTracker::default();
        tracker.submit_job();
        tracker.submit_job();
        tracker.submit_job();

        assert_eq!(tracker.total_submitted, 3);
        assert_eq!(tracker.active_jobs(), 3);
    }

    #[test]
    fn test_complete_jobs() {
        let mut tracker = JobTracker::default();
        tracker.submit_job();
        tracker.submit_job();
        tracker.complete_job();

        assert_eq!(tracker.total_submitted, 2);
        assert_eq!(tracker.total_completed, 1);
        assert_eq!(tracker.active_jobs(), 1);
    }

    #[test]
    fn test_fail_jobs() {
        let mut tracker = JobTracker::default();
        tracker.submit_job();
        tracker.submit_job();
        tracker.fail_job();

        assert_eq!(tracker.total_submitted, 2);
        assert_eq!(tracker.total_failed, 1);
        assert_eq!(tracker.active_jobs(), 1);
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = JobTracker::default();
        tracker.submit_job();
        tracker.submit_job();
        tracker.submit_job();
        tracker.submit_job();

        tracker.complete_job();
        tracker.complete_job();
        tracker.complete_job();
        tracker.fail_job();

        assert_eq!(tracker.success_rate(), 75.0); // 3 out of 4
    }

    #[test]
    fn test_success_rate_no_jobs() {
        let tracker = JobTracker::default();
        assert_eq!(tracker.success_rate(), 0.0);
    }
}
