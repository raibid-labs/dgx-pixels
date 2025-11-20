//! # Generation Events
//!
//! Events for image generation requests and job management.

use bevy::prelude::*;
use crate::messages::GenerationStage;

/// Event to submit a new generation job.
#[derive(Event, Debug, Clone)]
pub struct SubmitGenerationJob {
    pub prompt: String,
}

/// Event when generation job completes.
#[derive(Event, Debug, Clone)]
pub struct GenerationComplete {
    pub job_id: String,
    pub image_path: std::path::PathBuf,
}

/// Event to cancel a running job.
#[derive(Event, Debug, Clone)]
pub struct CancelJob {
    pub job_id: String,
}

/// Event for job progress updates from backend.
#[derive(Event, Debug, Clone)]
pub struct JobProgressUpdate {
    pub job_id: String,
    pub stage: GenerationStage,
    pub step: u32,
    pub total_steps: u32,
    pub percent: f32,
    pub eta_s: f32,
}

/// Event when job starts processing.
#[derive(Event, Debug, Clone)]
pub struct JobStarted {
    pub job_id: String,
}

/// Event handler for generation events.
pub fn handle_generation_events(
    mut submit_events: EventReader<SubmitGenerationJob>,
    mut complete_events: EventReader<GenerationComplete>,
    mut cancel_events: EventReader<CancelJob>,
    zmq_client: Option<Res<crate::bevy_app::systems::zmq::ZmqClientResource>>,
    mut commands: Commands,
    mut job_tracker: ResMut<crate::bevy_app::resources::JobTracker>,
    mut job_query: Query<&mut crate::bevy_app::components::Job>,
    mut app_state: ResMut<crate::bevy_app::resources::AppState>,
) {
    for event in submit_events.read() {
        info!("Generation job submitted: {}", event.prompt);

        // Always create a job entity for UI feedback
        let job_id = format!("job-{}", uuid::Uuid::new_v4());
        commands.spawn(crate::bevy_app::components::Job::new(
            job_id.clone(),
            event.prompt.clone(),
        ));
        job_tracker.submit_job();

        // Try to send to backend if available
        if let Some(ref zmq_client) = zmq_client {
            let client = zmq_client.0.lock();
            let request = crate::messages::Request::Generate {
                id: job_id.clone(),
                prompt: event.prompt.clone(),
                model: "sdxl".to_string(),
                lora: None,
                size: (1024, 1024),
                steps: 30,
                cfg_scale: 7.5,
            };

            if let Err(e) = client.send_request(request) {
                error!("Failed to send generation request to backend: {}", e);
            } else {
                info!("Job {} sent to backend", job_id);
            }
        } else {
            warn!("No backend connected - job {} created but will not be processed", job_id);
        }
    }

    for event in complete_events.read() {
        info!(
            "Generation complete: {} -> {:?}",
            event.job_id, event.image_path
        );
        // Handled by zmq::response_handler
    }

    for event in cancel_events.read() {
        info!("Cancel job requested: {}", event.job_id);

        // Try to send cancel request to backend
        if let Some(ref zmq_client) = zmq_client {
            let client = zmq_client.0.lock();
            let request = crate::messages::Request::Cancel {
                job_id: event.job_id.clone(),
            };

            if let Err(e) = client.send_request(request) {
                error!("Failed to send cancel request to backend: {}", e);
                // Still mark as cancelled locally
            } else {
                info!("Cancel request for job {} sent to backend", event.job_id);
            }
        } else {
            warn!("No backend connected - cannot cancel job {}", event.job_id);
        }

        // Update job status locally
        for mut job in job_query.iter_mut() {
            if job.id == event.job_id {
                job.status = crate::bevy_app::components::JobStatus::Cancelled;
                info!("Job {} marked as cancelled locally", event.job_id);
                app_state.request_redraw();
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_submit_event() {
        let mut app = App::new();
        app.add_event::<SubmitGenerationJob>();
        app.add_event::<GenerationComplete>();
        app.add_event::<CancelJob>();
        app.insert_resource(crate::bevy_app::resources::JobTracker::default());
        app.insert_resource(crate::bevy_app::resources::AppState::default());
        app.add_systems(Update, handle_generation_events);

        app.world_mut().send_event(SubmitGenerationJob {
            prompt: "test prompt".into(),
        });
        app.update();
        // No assertion - just verify no panic (ZMQ client optional)
    }

    #[test]
    fn test_cancel_event() {
        let mut app = App::new();
        app.add_event::<SubmitGenerationJob>();
        app.add_event::<GenerationComplete>();
        app.add_event::<CancelJob>();
        app.insert_resource(crate::bevy_app::resources::JobTracker::default());
        app.insert_resource(crate::bevy_app::resources::AppState::default());
        app.add_systems(Update, handle_generation_events);

        // Create a job first
        let job_id = "test-job-123".to_string();
        app.world_mut()
            .spawn(crate::bevy_app::components::Job::new(
                job_id.clone(),
                "test prompt".into(),
            ));

        // Send cancel event
        app.world_mut().send_event(CancelJob { job_id });
        app.update();

        // Verify job was marked as cancelled
        let mut job_query = app.world_mut().query::<&crate::bevy_app::components::Job>();
        let jobs: Vec<_> = job_query.iter(app.world()).collect();
        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].is_cancelled());
    }
}
