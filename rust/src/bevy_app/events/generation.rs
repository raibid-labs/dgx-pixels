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

/// Event handler for generation events (placeholder for WS-05 ZeroMQ integration).
pub fn handle_generation_events(
    mut submit_events: EventReader<SubmitGenerationJob>,
    mut complete_events: EventReader<GenerationComplete>,
    mut cancel_events: EventReader<CancelJob>,
) {
    for event in submit_events.read() {
        info!("Generation job submitted: {}", event.prompt);
        // TODO: WS-05 will send to ZeroMQ backend
    }

    for event in complete_events.read() {
        info!("Generation complete: {} -> {:?}", event.job_id, event.image_path);
        // TODO: WS-06 will load image assets
    }

    for event in cancel_events.read() {
        info!("Cancel job requested: {}", event.job_id);
        // TODO: WS-05 will send cancel to backend
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
        app.add_systems(Update, handle_generation_events);

        app.world_mut().send_event(SubmitGenerationJob {
            prompt: "test prompt".into(),
        });
        app.update();
        // No assertion - just verify no panic
    }
}
