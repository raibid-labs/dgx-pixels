//! Integration test for job submission and ZMQ backend integration (T7-T8)
//!
//! Tests the complete pipeline:
//! 1. Enter key → Submit job
//! 2. Job entity created with Bevy ECS
//! 3. JobTracker updated
//! 4. ZMQ communication (graceful degradation without backend)
//! 5. Response handling and job completion
//!
//! Note: Without ZMQ backend, jobs are NOT tracked. The system gracefully
//! degrades by logging warnings but not creating job entities or updating trackers.

#![cfg(feature = "bevy_migration_foundation")]

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
    use dgx_pixels_tui::bevy_app::{
        components::{Job, JobStatus},
        events::{CancelJob, GenerationComplete, SubmitGenerationJob},
        resources::{AppState, CurrentScreen, GalleryState, InputBuffer, JobTracker, Screen},
        systems::{
            input::screens::handle_generation_input,
            zmq::{handle_zmq_responses, poll_zmq},
        },
    };

    /// Create a minimal test app with required resources and systems
    fn create_test_app() -> App {
        let mut app = App::new();

        // Add required events
        app.add_event::<SubmitGenerationJob>();
        app.add_event::<GenerationComplete>();
        app.add_event::<CancelJob>();
        app.add_event::<bevy_ratatui::event::KeyEvent>();

        // Add required resources
        app.init_resource::<CurrentScreen>();
        app.init_resource::<InputBuffer>();
        app.init_resource::<AppState>();
        app.init_resource::<JobTracker>();
        app.init_resource::<GalleryState>();

        // Add input handling system
        app.add_systems(PreUpdate, handle_generation_input);

        // Add event handler (mimicking the actual plugin)
        app.add_systems(
            Update,
            dgx_pixels_tui::bevy_app::events::handle_generation_events,
        );

        // Add ZMQ systems
        app.add_systems(PreUpdate, poll_zmq);
        app.add_systems(Update, handle_zmq_responses);

        app
    }

    /// Helper to create a key event
    fn make_key_event(code: KeyCode) -> bevy_ratatui::event::KeyEvent {
        bevy_ratatui::event::KeyEvent(crossterm::event::KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        })
    }

    #[test]
    fn test_job_submission_without_backend() {
        let mut app = create_test_app();

        // Set screen to Generation
        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;

        // Set input text
        app.world_mut().resource_mut::<InputBuffer>().text = "pixel art knight".to_string();

        // Simulate Enter key event
        app.world_mut().send_event(make_key_event(KeyCode::Enter));

        // Run one update cycle (PreUpdate + Update)
        app.update();

        // Verify input buffer was cleared (input system always clears on submit)
        let input_buffer = app.world().resource::<InputBuffer>();
        assert_eq!(
            input_buffer.text, "",
            "Input buffer should be cleared after Enter press"
        );

        // WITHOUT ZMQ backend: JobTracker is NOT incremented (graceful degradation)
        // The event is received but not processed since there's no backend to send to
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_submitted, 0,
            "JobTracker should NOT increment without ZMQ backend (graceful degradation)"
        );

        // No Job entity should be created without backend
        let mut job_query = app.world_mut().query::<&Job>();
        let job_count = job_query.iter(app.world()).count();
        assert_eq!(
            job_count, 0,
            "No job entities should exist without ZMQ backend"
        );
    }

    #[test]
    fn test_job_completion_flow() {
        let mut app = create_test_app();

        // Manually create a pending job entity (simulating backend acceptance)
        let job_id = "test-job-001".to_string();
        app.world_mut()
            .spawn(Job::new(job_id.clone(), "test prompt".to_string()));

        // Verify job exists with Pending status
        let mut job_query = app.world_mut().query::<&Job>();
        let jobs: Vec<&Job> = job_query.iter(app.world()).collect();
        assert_eq!(jobs.len(), 1);
        assert!(matches!(jobs[0].status, JobStatus::Pending));

        // Simulate job completion event from backend
        app.world_mut().send_event(GenerationComplete {
            job_id: job_id.clone(),
            image_path: std::path::PathBuf::from("/tmp/test-output.png"),
        });

        // Run update cycle
        app.update();

        // Verify job status updated to Complete
        let mut job_query = app.world_mut().query::<&Job>();
        let jobs: Vec<&Job> = job_query.iter(app.world()).collect();
        assert_eq!(jobs.len(), 1);
        assert!(
            matches!(jobs[0].status, JobStatus::Complete { .. }),
            "Job should be marked as complete"
        );

        // Verify JobTracker was updated
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_completed, 1,
            "JobTracker should record one completed job"
        );

        // Verify image was added to gallery
        let gallery = app.world().resource::<GalleryState>();
        assert_eq!(
            gallery.images.len(),
            1,
            "Gallery should contain the generated image"
        );
        assert_eq!(
            gallery.images[0],
            std::path::PathBuf::from("/tmp/test-output.png")
        );
    }

    #[test]
    fn test_multiple_prompts_without_backend() {
        let mut app = create_test_app();

        // Set screen to Generation
        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;

        // Submit first job
        app.world_mut().resource_mut::<InputBuffer>().text = "job 1".to_string();
        app.world_mut().send_event(make_key_event(KeyCode::Enter));
        app.update();

        // Submit second job
        app.world_mut().resource_mut::<InputBuffer>().text = "job 2".to_string();
        app.world_mut().send_event(make_key_event(KeyCode::Enter));
        app.update();

        // Submit third job
        app.world_mut().resource_mut::<InputBuffer>().text = "job 3".to_string();
        app.world_mut().send_event(make_key_event(KeyCode::Enter));
        app.update();

        // Without ZMQ backend, JobTracker should NOT increment
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_submitted, 0,
            "JobTracker should not increment without backend"
        );

        // Input system still works correctly (clears after each submit)
        let input_buffer = app.world().resource::<InputBuffer>();
        assert_eq!(input_buffer.text, "", "Input buffer should be clear");
    }

    #[test]
    fn test_empty_prompt_not_submitted() {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;
        app.world_mut().resource_mut::<InputBuffer>().text = "".to_string();
        app.world_mut().send_event(make_key_event(KeyCode::Enter));
        app.update();

        // Verify no job was submitted
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_submitted, 0,
            "Empty prompts should not be submitted"
        );

        // Input buffer should NOT be cleared (validation prevents submission)
        let input_buffer = app.world().resource::<InputBuffer>();
        assert_eq!(
            input_buffer.text, "",
            "Input buffer unchanged when empty prompt submitted"
        );
    }

    #[test]
    fn test_whitespace_only_prompt_not_submitted() {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;
        let whitespace = "   \t  \n  ".to_string();
        app.world_mut().resource_mut::<InputBuffer>().text = whitespace.clone();
        app.world_mut().send_event(make_key_event(KeyCode::Enter));
        app.update();

        // Verify no job was submitted
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_submitted, 0,
            "Whitespace-only prompts should not be submitted"
        );

        // Input buffer should NOT be cleared (validation prevents submission)
        let input_buffer = app.world().resource::<InputBuffer>();
        assert_eq!(
            input_buffer.text, whitespace,
            "Input buffer unchanged when whitespace-only prompt submitted"
        );
    }

    #[test]
    fn test_g_key_without_backend() {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;
        app.world_mut().resource_mut::<InputBuffer>().text = "test prompt".to_string();
        app.world_mut().send_event(make_key_event(KeyCode::Char('G')));
        app.update();

        // 'G' key behaves same as Enter (submits job)
        // Without backend, job not tracked
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_submitted, 0,
            "'G' key without backend does not track job"
        );

        // Input was cleared (input system always clears on submit)
        let input_buffer = app.world().resource::<InputBuffer>();
        assert_eq!(input_buffer.text, "");
    }

    #[test]
    fn test_esc_clears_input_without_submitting() {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;
        app.world_mut().resource_mut::<InputBuffer>().text = "test prompt".to_string();
        app.world_mut().send_event(make_key_event(KeyCode::Esc));
        app.update();

        // Verify no job was submitted
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_submitted, 0,
            "Esc should not submit jobs"
        );

        // Verify input was cleared
        let input_buffer = app.world().resource::<InputBuffer>();
        assert_eq!(input_buffer.text, "", "Esc should clear input buffer");
    }

    #[test]
    fn test_job_completion_unknown_job() {
        let mut app = create_test_app();

        // Send completion event for non-existent job
        app.world_mut().send_event(GenerationComplete {
            job_id: "unknown-job-999".to_string(),
            image_path: std::path::PathBuf::from("/tmp/test.png"),
        });

        app.update();

        // Verify no crash and no changes to tracker
        let job_tracker = app.world().resource::<JobTracker>();
        assert_eq!(
            job_tracker.total_completed, 0,
            "Completion of unknown job should not update tracker"
        );

        // Gallery should not be updated for unknown jobs
        let gallery = app.world().resource::<GalleryState>();
        assert_eq!(gallery.images.len(), 0, "Gallery should remain empty");
    }

    #[test]
    fn test_poll_zmq_without_client() {
        let mut app = create_test_app();

        // Just verify poll_zmq doesn't panic without ZmqClientResource
        app.update();
        app.update();
        app.update();

        // No assertions needed - test verifies graceful degradation (no panic)
    }

    #[test]
    fn test_input_buffer_cleared_on_valid_submission() {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<CurrentScreen>().0 = Screen::Generation;
        app.world_mut().resource_mut::<InputBuffer>().text = "valid prompt".to_string();
        app.world_mut().send_event(make_key_event(KeyCode::Enter));
        app.update();

        // Even without backend, input system clears buffer on Enter
        let input_buffer = app.world().resource::<InputBuffer>();
        assert_eq!(
            input_buffer.text, "",
            "Input buffer cleared after Enter on non-empty prompt"
        );
    }
}
