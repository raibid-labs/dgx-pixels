//! # ZMQ Update Handler
//!
//! Processes progress updates from backend and updates Job entities.

use bevy::prelude::*;

use crate::bevy_app::components::{Job, JobStatus};
use crate::bevy_app::events::{JobProgressUpdate, JobStarted};
use crate::bevy_app::resources::AppState;
use crate::messages::GenerationStage;

/// Handle progress updates from backend and update Job entities.
pub fn handle_zmq_updates(
    mut progress_events: EventReader<JobProgressUpdate>,
    mut started_events: EventReader<JobStarted>,
    mut job_query: Query<&mut Job>,
    mut app_state: ResMut<AppState>,
) {
    // Handle job started events
    for event in started_events.read() {
        debug!("Processing job started event: {}", event.job_id);

        // Find and update the job entity
        for mut job in job_query.iter_mut() {
            if job.id == event.job_id {
                // Update status to Queued (will transition to Generating on first progress)
                if matches!(job.status, JobStatus::Pending) {
                    job.status = JobStatus::Queued;
                    app_state.request_redraw();
                    info!("Job {} moved to Queued state", event.job_id);
                }
                break;
            }
        }
    }

    // Handle progress update events
    for event in progress_events.read() {
        debug!(
            "Processing progress update for job {}: {:?} {:.0}%",
            event.job_id,
            event.stage,
            event.percent * 100.0
        );

        // Find and update the job entity
        let mut job_found = false;
        for mut job in job_query.iter_mut() {
            if job.id == event.job_id {
                // Format stage name for display
                let stage_name = format_stage(&event.stage, event.step, event.total_steps);

                // Update job status to Generating
                job.status = JobStatus::Generating {
                    stage: stage_name,
                    progress: event.percent,
                    eta_s: event.eta_s,
                };

                job_found = true;
                app_state.request_redraw();

                // Log significant milestones
                if event.percent == 0.0
                    || event.percent == 0.5
                    || event.percent == 1.0
                    || event.step % 10 == 0
                {
                    debug!(
                        "Job {} progress: {} - {:.0}% (ETA: {:.1}s)",
                        event.job_id,
                        job.status.stage_name().unwrap_or("Unknown"),
                        event.percent * 100.0,
                        event.eta_s
                    );
                }

                break;
            }
        }

        if !job_found {
            warn!(
                "Received progress update for unknown job: {}",
                event.job_id
            );
        }
    }
}

/// Format stage name for display with step information.
fn format_stage(stage: &GenerationStage, step: u32, total_steps: u32) -> String {
    let stage_name = match stage {
        GenerationStage::Initializing => "Initializing",
        GenerationStage::LoadingModels => "Loading Models",
        GenerationStage::Encoding => "Encoding Prompt",
        GenerationStage::Sampling => "Sampling",
        GenerationStage::Decoding => "Decoding Image",
        GenerationStage::PostProcessing => "Post-Processing",
    };

    // For sampling stage, include step info
    if matches!(stage, GenerationStage::Sampling) {
        format!("{} (Step {}/{})", stage_name, step, total_steps)
    } else {
        stage_name.to_string()
    }
}

/// Helper trait to get stage name from JobStatus.
trait JobStatusExt {
    fn stage_name(&self) -> Option<&str>;
}

impl JobStatusExt for JobStatus {
    fn stage_name(&self) -> Option<&str> {
        match self {
            JobStatus::Generating { stage, .. } => Some(stage.as_str()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_update_handler() {
        let mut app = App::new();
        app.add_event::<JobProgressUpdate>();
        app.add_event::<JobStarted>();
        app.insert_resource(AppState::default());
        app.add_systems(Update, handle_zmq_updates);

        // Create a job entity
        app.world_mut()
            .spawn(Job::new("test-123".into(), "test prompt".into()));

        // Send started event
        app.world_mut().send_event(JobStarted {
            job_id: "test-123".into(),
        });

        app.update();

        // Verify job was updated to Queued
        let mut job_query = app.world_mut().query::<&Job>();
        let job = job_query.iter(app.world()).next().unwrap();
        assert!(matches!(job.status, JobStatus::Queued));
    }

    #[test]
    fn test_progress_update() {
        let mut app = App::new();
        app.add_event::<JobProgressUpdate>();
        app.add_event::<JobStarted>();
        app.insert_resource(AppState::default());
        app.add_systems(Update, handle_zmq_updates);

        // Create a job entity
        app.world_mut()
            .spawn(Job::new("test-123".into(), "test prompt".into()));

        // Send progress update
        app.world_mut().send_event(JobProgressUpdate {
            job_id: "test-123".into(),
            stage: GenerationStage::Sampling,
            step: 15,
            total_steps: 30,
            percent: 0.5,
            eta_s: 12.5,
        });

        app.update();

        // Verify job was updated to Generating
        let mut job_query = app.world_mut().query::<&Job>();
        let job = job_query.iter(app.world()).next().unwrap();
        match &job.status {
            JobStatus::Generating {
                stage,
                progress,
                eta_s,
            } => {
                assert!(stage.contains("Sampling"));
                assert_eq!(*progress, 0.5);
                assert_eq!(*eta_s, 12.5);
            }
            _ => panic!("Expected Generating status"),
        }
    }

    #[test]
    fn test_format_stage() {
        // Test sampling stage with step info
        let result = format_stage(&GenerationStage::Sampling, 15, 30);
        assert_eq!(result, "Sampling (Step 15/30)");

        // Test other stages without step info
        let result = format_stage(&GenerationStage::LoadingModels, 0, 30);
        assert_eq!(result, "Loading Models");

        let result = format_stage(&GenerationStage::Encoding, 5, 30);
        assert_eq!(result, "Encoding Prompt");
    }

    #[test]
    fn test_unknown_job_handling() {
        let mut app = App::new();
        app.add_event::<JobProgressUpdate>();
        app.add_event::<JobStarted>();
        app.insert_resource(AppState::default());
        app.add_systems(Update, handle_zmq_updates);

        // Send progress for non-existent job (should not panic)
        app.world_mut().send_event(JobProgressUpdate {
            job_id: "unknown-job".into(),
            stage: GenerationStage::Sampling,
            step: 1,
            total_steps: 30,
            percent: 0.1,
            eta_s: 10.0,
        });

        app.update();
        // Should complete without panic
    }
}
