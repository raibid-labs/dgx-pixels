//! # ZeroMQ Polling System
//!
//! Polls ZMQ client for responses and updates in PreUpdate schedule.

use bevy::prelude::*;

use super::ZmqClientResource;

/// Poll ZMQ client for responses and updates.
///
/// Runs in PreUpdate schedule to process backend messages before main logic.
pub fn poll_zmq(
    zmq_client: Option<Res<ZmqClientResource>>,
    mut response_events: EventWriter<crate::bevy_app::events::GenerationComplete>,
) {
    let Some(zmq_client) = zmq_client else {
        return; // No ZMQ client configured
    };

    let client = zmq_client.0.lock();

    // Poll for responses
    while let Some(response) = client.try_recv_response() {
        use crate::messages::Response;

        match response {
            Response::JobAccepted {
                job_id,
                estimated_time_s,
            } => {
                info!(
                    "Job accepted by backend: {} (ETA: {:.1}s)",
                    job_id, estimated_time_s
                );
                // Job entity will be created by response_handler
            }
            Response::JobComplete {
                job_id,
                image_path,
                duration_s,
            } => {
                info!(
                    "Job complete: {} -> {} ({:.1}s)",
                    job_id, image_path, duration_s
                );
                // Emit event for response handler
                response_events.send(crate::bevy_app::events::GenerationComplete {
                    job_id,
                    image_path: std::path::PathBuf::from(image_path),
                });
            }
            Response::JobError { job_id, error } => {
                error!("Job failed: {} - {}", job_id, error);
                // TODO: Emit JobFailed event
            }
            _ => {
                debug!("Received other response: {:?}", response);
            }
        }
    }

    // Poll for progress updates
    while let Some(update) = client.try_recv_update() {
        use crate::messages::ProgressUpdate;

        match update {
            ProgressUpdate::JobStarted { job_id, timestamp } => {
                info!("Job {} started at {}", job_id, timestamp);
                // TODO: Update job entity status to Generating
            }
            ProgressUpdate::Progress {
                job_id,
                stage,
                step,
                total_steps,
                percent,
                eta_s,
            } => {
                debug!(
                    "Job {} progress: {:?} {}/{} ({:.1}%) ETA: {:.1}s",
                    job_id,
                    stage,
                    step,
                    total_steps,
                    percent * 100.0,
                    eta_s
                );
                // TODO: Update job progress
            }
            ProgressUpdate::Preview {
                job_id,
                image_path,
                step,
            } => {
                info!(
                    "Preview available for job {} at step {}: {}",
                    job_id, step, image_path
                );
                // TODO: Load preview image
            }
            ProgressUpdate::JobFinished {
                job_id,
                success,
                duration_s,
            } => {
                if success {
                    info!("Job {} finished successfully in {:.1}s", job_id, duration_s);
                } else {
                    warn!(
                        "Job {} finished with failure after {:.1}s",
                        job_id, duration_s
                    );
                }
                // Final completion handled by Response::JobComplete
            }
            ProgressUpdate::JobComplete {
                job_id,
                image_path,
                duration_s,
            } => {
                info!(
                    "Job {} complete: {} ({:.1}s)",
                    job_id, image_path, duration_s
                );
                // Handled by Response::JobComplete
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_poll_without_zmq_client() {
        let mut app = App::new();
        app.add_event::<crate::bevy_app::events::GenerationComplete>();
        app.add_systems(Update, poll_zmq);

        // Should not panic without ZMQ client
        app.update();
    }
}
