//! # Generation Events
//!
//! Events for image generation requests and job management.

use bevy::prelude::*;

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

/// Event handler for generation events.
pub fn handle_generation_events(
    mut submit_events: EventReader<SubmitGenerationJob>,
    mut complete_events: EventReader<GenerationComplete>,
    mut cancel_events: EventReader<CancelJob>,
    zmq_client: Option<Res<crate::bevy_app::systems::zmq::ZmqClientResource>>,
    mut commands: Commands,
    mut job_tracker: ResMut<crate::bevy_app::resources::JobTracker>,
) {
    for event in submit_events.read() {
        info!("Generation job submitted: {}", event.prompt);

        if let Some(ref zmq_client) = zmq_client {
            // Send request to backend via ZMQ
            let client = zmq_client.0.lock();
            let job_id = format!("job-{}", uuid::Uuid::new_v4());
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
                error!("Failed to send generation request: {}", e);
            } else {
                // Create pending job entity
                commands.spawn(crate::bevy_app::components::Job::new(
                    job_id,
                    event.prompt.clone(),
                ));
                job_tracker.submit_job();
                info!("Job submitted to backend");
            }
        } else {
            warn!("No ZMQ client configured, cannot submit job");
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
        // TODO: Send cancel to backend
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
        app.add_systems(Update, handle_generation_events);

        app.world_mut().send_event(SubmitGenerationJob {
            prompt: "test prompt".into(),
        });
        app.update();
        // No assertion - just verify no panic (ZMQ client optional)
    }
}
