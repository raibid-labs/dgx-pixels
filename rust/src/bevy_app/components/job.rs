//! # Job Component
//!
//! Represents an individual image generation job as a Bevy entity.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::components::{Job, JobStatus};
//!
//! fn spawn_job(mut commands: Commands) {
//!     let job = Job::new(
//!         "job-001".to_string(),
//!         "pixel art character".to_string()
//!     );
//!     commands.spawn(job);
//! }
//! ```

use bevy::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

/// Job entity component for tracking image generation jobs.
#[derive(Component, Debug, Clone)]
pub struct Job {
    /// Unique job ID
    pub id: String,
    /// Generation prompt
    pub prompt: String,
    /// Current job status
    pub status: JobStatus,
    /// Submission timestamp
    pub submitted_at: Instant,
}

/// Status of an image generation job.
#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    /// Job submitted, waiting for backend acceptance
    Pending,
    /// Job accepted by backend and queued
    Queued,
    /// Job currently generating
    Generating {
        /// Current generation stage
        stage: String,
        /// Progress (0.0 - 1.0)
        progress: f32,
        /// Estimated time remaining (seconds)
        eta_s: f32,
    },
    /// Job completed successfully
    Complete {
        /// Path to generated image
        image_path: PathBuf,
        /// Time taken to generate (seconds)
        duration_s: f32,
    },
    /// Job failed with error
    Failed {
        /// Error message
        error: String,
    },
    /// Job was cancelled by user
    Cancelled,
}

impl Job {
    /// Create a new job with pending status.
    pub fn new(id: String, prompt: String) -> Self {
        Self {
            id,
            prompt,
            status: JobStatus::Pending,
            submitted_at: Instant::now(),
        }
    }

    /// Check if job is complete.
    pub fn is_complete(&self) -> bool {
        matches!(self.status, JobStatus::Complete { .. })
    }

    /// Check if job failed.
    pub fn is_failed(&self) -> bool {
        matches!(self.status, JobStatus::Failed { .. })
    }

    /// Check if job is cancelled.
    pub fn is_cancelled(&self) -> bool {
        matches!(self.status, JobStatus::Cancelled)
    }

    /// Check if job is active (not complete, failed, or cancelled).
    pub fn is_active(&self) -> bool {
        !self.is_complete() && !self.is_failed() && !self.is_cancelled()
    }

    /// Check if job can be cancelled (Pending, Queued, or Generating).
    pub fn is_cancellable(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Pending | JobStatus::Queued | JobStatus::Generating { .. }
        )
    }

    /// Get elapsed time since submission.
    pub fn elapsed(&self) -> std::time::Duration {
        self.submitted_at.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_job() {
        let job = Job::new("job-001".to_string(), "test prompt".to_string());
        assert_eq!(job.id, "job-001");
        assert_eq!(job.prompt, "test prompt");
        assert!(matches!(job.status, JobStatus::Pending));
    }

    #[test]
    fn test_job_status_checks() {
        let mut job = Job::new("job-001".to_string(), "test".to_string());

        // Pending
        assert!(job.is_active());
        assert!(!job.is_complete());
        assert!(!job.is_failed());
        assert!(!job.is_cancelled());
        assert!(job.is_cancellable());

        // Complete
        job.status = JobStatus::Complete {
            image_path: PathBuf::from("/test.png"),
            duration_s: 3.5,
        };
        assert!(job.is_complete());
        assert!(!job.is_active());
        assert!(!job.is_cancellable());

        // Failed
        job.status = JobStatus::Failed {
            error: "test error".to_string(),
        };
        assert!(job.is_failed());
        assert!(!job.is_active());
        assert!(!job.is_cancellable());

        // Cancelled
        job.status = JobStatus::Cancelled;
        assert!(job.is_cancelled());
        assert!(!job.is_active());
        assert!(!job.is_cancellable());
    }

    #[test]
    fn test_job_elapsed_time() {
        let job = Job::new("job-001".to_string(), "test".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(job.elapsed().as_millis() >= 10);
    }

    #[test]
    fn test_cancellable_statuses() {
        let mut job = Job::new("job-001".to_string(), "test".to_string());

        // Pending is cancellable
        job.status = JobStatus::Pending;
        assert!(job.is_cancellable());

        // Queued is cancellable
        job.status = JobStatus::Queued;
        assert!(job.is_cancellable());

        // Generating is cancellable
        job.status = JobStatus::Generating {
            stage: "Sampling".to_string(),
            progress: 0.5,
            eta_s: 2.0,
        };
        assert!(job.is_cancellable());

        // Complete is not cancellable
        job.status = JobStatus::Complete {
            image_path: PathBuf::from("/test.png"),
            duration_s: 3.5,
        };
        assert!(!job.is_cancellable());

        // Failed is not cancellable
        job.status = JobStatus::Failed {
            error: "test error".to_string(),
        };
        assert!(!job.is_cancellable());

        // Cancelled is not cancellable
        job.status = JobStatus::Cancelled;
        assert!(!job.is_cancellable());
    }
}
