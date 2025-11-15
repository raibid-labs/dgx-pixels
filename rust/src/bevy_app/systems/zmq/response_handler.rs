//! # ZMQ Response Handler
//!
//! Processes GenerationComplete events and updates Job entities.

use bevy::prelude::*;

use crate::bevy_app::components::{Job, JobStatus};
use crate::bevy_app::events::GenerationComplete;
use crate::bevy_app::resources::{AppState, GalleryState, JobTracker};

/// Handle job completion responses from backend.
pub fn handle_zmq_responses(
    mut complete_events: EventReader<GenerationComplete>,
    mut job_query: Query<&mut Job>,
    mut gallery: ResMut<GalleryState>,
    mut job_tracker: ResMut<JobTracker>,
    mut app_state: ResMut<AppState>,
) {
    for event in complete_events.read() {
        info!("Processing job completion: {}", event.job_id);

        // Find and update the job entity
        let mut job_found = false;
        for mut job in job_query.iter_mut() {
            if job.id == event.job_id {
                let duration_s = job.elapsed().as_secs_f32();
                job.status = JobStatus::Complete {
                    image_path: event.image_path.clone(),
                    duration_s,
                };
                job_found = true;

                // Add to gallery
                gallery.add_image(event.image_path.clone());

                // Update tracker
                job_tracker.complete_job();

                // Request redraw
                app_state.request_redraw();

                info!("Job {} marked complete, added to gallery", event.job_id);
                break;
            }
        }

        if !job_found {
            warn!("Received completion for unknown job: {}", event.job_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use std::path::PathBuf;

    #[test]
    fn test_response_handler() {
        let mut app = App::new();
        app.add_event::<GenerationComplete>();
        app.insert_resource(GalleryState::default());
        app.insert_resource(JobTracker::default());
        app.insert_resource(AppState::default());
        app.add_systems(Update, handle_zmq_responses);

        // Create a job entity
        app.world_mut().spawn(Job::new("test-123".into(), "test prompt".into()));

        // Send completion event
        app.world_mut().send_event(GenerationComplete {
            job_id: "test-123".into(),
            image_path: PathBuf::from("/tmp/test.png"),
        });

        app.update();

        // Verify job was updated
        let mut job_query = app.world_mut().query::<&Job>();
        assert_eq!(job_query.iter(app.world()).count(), 1);
    }
}
