//! # Queue Input Handler
//!
//! Handles keyboard input for the Queue screen, including job navigation and cancellation.

use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;

use crate::bevy_app::components::Job;
use crate::bevy_app::events::CancelJob;
use crate::bevy_app::resources::{AppState, CurrentScreen, QueueState, Screen};

/// Handle input for Queue screen
///
/// Keyboard controls:
/// - Up/Down/k/j: Navigate job list
/// - Home: Jump to first job
/// - End: Jump to last job
/// - c/C: Cancel selected job (if active)
pub fn handle_queue_input(
    mut events: EventReader<KeyEvent>,
    current_screen: Res<CurrentScreen>,
    mut queue_state: ResMut<QueueState>,
    mut cancel_events: EventWriter<CancelJob>,
    mut app_state: ResMut<AppState>,
    jobs: Query<&Job>,
) {
    if current_screen.0 != Screen::Queue {
        return;
    }

    for event in events.read() {
        match event.code {
            // Navigate up (previous job)
            KeyCode::Up | KeyCode::Char('k') => {
                queue_state.select_previous();
                app_state.request_redraw();
                debug!("Queue: Navigate to previous job (index {})", queue_state.selected);
            }

            // Navigate down (next job)
            KeyCode::Down | KeyCode::Char('j') => {
                queue_state.select_next();
                app_state.request_redraw();
                debug!("Queue: Navigate to next job (index {})", queue_state.selected);
            }

            // Jump to first job
            KeyCode::Home => {
                queue_state.select_first();
                app_state.request_redraw();
                debug!("Queue: Jump to first job");
            }

            // Jump to last job
            KeyCode::End => {
                queue_state.select_last();
                app_state.request_redraw();
                debug!("Queue: Jump to last job");
            }

            // Cancel selected job
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Get all jobs sorted by submission time (matching render order)
                let mut all_jobs: Vec<&Job> = jobs.iter().collect();
                all_jobs.sort_by_key(|j| j.submitted_at);

                // Get the selected job
                if let Some(job) = all_jobs.get(queue_state.selected) {
                    // Only allow canceling active jobs
                    if job.is_active() {
                        cancel_events.send(CancelJob {
                            job_id: job.id.clone(),
                        });
                        info!("Queue: Cancel requested for job {}", job.id);
                        app_state.request_redraw();
                    } else {
                        warn!("Queue: Cannot cancel completed/failed job {}", job.id);
                    }
                } else {
                    warn!("Queue: No job selected to cancel");
                }
            }

            // Ignore other keys (let main keyboard handler process them)
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use std::path::PathBuf;
    use crate::bevy_app::components::JobStatus;

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent(crossterm::event::KeyEvent::new(
            code,
            crossterm::event::KeyModifiers::NONE,
        ))
    }

    #[test]
    fn test_queue_input_compiles() {
        let mut app = App::new();
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();
        app.insert_resource(CurrentScreen(Screen::Queue));
        app.insert_resource(QueueState::default());
        app.insert_resource(AppState::default());
        app.add_systems(Update, handle_queue_input);
    }

    #[test]
    fn test_navigation_up_down() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Queue));
        let mut queue_state = QueueState::default();
        queue_state.update_total(5);
        app.insert_resource(queue_state);
        app.insert_resource(AppState::default());

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();

        // Add system
        app.add_systems(Update, handle_queue_input);

        // Test Down arrow
        app.world_mut().send_event(create_key_event(KeyCode::Down));
        app.update();

        let queue_state = app.world().resource::<QueueState>();
        assert_eq!(queue_state.selected, 1);

        // Test Up arrow
        app.world_mut().send_event(create_key_event(KeyCode::Up));
        app.update();

        let queue_state = app.world().resource::<QueueState>();
        assert_eq!(queue_state.selected, 0);
    }

    #[test]
    fn test_vi_style_navigation() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Queue));
        let mut queue_state = QueueState::default();
        queue_state.update_total(5);
        app.insert_resource(queue_state);
        app.insert_resource(AppState::default());

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();

        // Add system
        app.add_systems(Update, handle_queue_input);

        // Test 'j' key (down)
        app.world_mut().send_event(create_key_event(KeyCode::Char('j')));
        app.update();

        let queue_state = app.world().resource::<QueueState>();
        assert_eq!(queue_state.selected, 1);

        // Test 'k' key (up)
        app.world_mut().send_event(create_key_event(KeyCode::Char('k')));
        app.update();

        let queue_state = app.world().resource::<QueueState>();
        assert_eq!(queue_state.selected, 0);
    }

    #[test]
    fn test_home_end_navigation() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Queue));
        let mut queue_state = QueueState::default();
        queue_state.update_total(10);
        queue_state.selected = 5; // Start in middle
        app.insert_resource(queue_state);
        app.insert_resource(AppState::default());

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();

        // Add system
        app.add_systems(Update, handle_queue_input);

        // Test End key
        app.world_mut().send_event(create_key_event(KeyCode::End));
        app.update();

        let queue_state = app.world().resource::<QueueState>();
        assert_eq!(queue_state.selected, 9); // Last job

        // Test Home key
        app.world_mut().send_event(create_key_event(KeyCode::Home));
        app.update();

        let queue_state = app.world().resource::<QueueState>();
        assert_eq!(queue_state.selected, 0); // First job
    }

    #[test]
    fn test_cancel_job_event() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Queue));
        let mut queue_state = QueueState::default();
        queue_state.update_total(1);
        app.insert_resource(queue_state);
        app.insert_resource(AppState::default());

        // Spawn a job entity
        let job = Job::new("job-001".to_string(), "test prompt".to_string());
        app.world_mut().spawn(job);

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();

        // Add system
        app.add_systems(Update, handle_queue_input);

        // Simulate 'c' key press
        app.world_mut().send_event(create_key_event(KeyCode::Char('c')));
        app.update();

        // Verify CancelJob event was sent
        let mut cancel_events = app.world_mut().resource_mut::<Events<CancelJob>>();
        let mut reader = cancel_events.get_cursor();
        let events: Vec<_> = reader.read(&cancel_events).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].job_id, "job-001");
    }

    #[test]
    fn test_no_cancel_for_completed_job() {
        let mut app = App::new();

        // Setup resources
        app.insert_resource(CurrentScreen(Screen::Queue));
        let mut queue_state = QueueState::default();
        queue_state.update_total(1);
        app.insert_resource(queue_state);
        app.insert_resource(AppState::default());

        // Spawn a completed job entity
        let mut job = Job::new("job-001".to_string(), "test prompt".to_string());
        job.status = JobStatus::Complete {
            image_path: PathBuf::from("/test.png"),
            duration_s: 3.5,
        };
        app.world_mut().spawn(job);

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();

        // Add system
        app.add_systems(Update, handle_queue_input);

        // Simulate 'c' key press
        app.world_mut().send_event(create_key_event(KeyCode::Char('c')));
        app.update();

        // Verify NO CancelJob event was sent (job is complete)
        let mut cancel_events = app.world_mut().resource_mut::<Events<CancelJob>>();
        let mut reader = cancel_events.get_cursor();
        assert_eq!(reader.read(&cancel_events).count(), 0);
    }

    #[test]
    fn test_no_input_on_other_screens() {
        let mut app = App::new();

        // Setup resources - DIFFERENT screen
        app.insert_resource(CurrentScreen(Screen::Generation));
        let mut queue_state = QueueState::default();
        queue_state.update_total(5);
        app.insert_resource(queue_state);
        app.insert_resource(AppState::default());

        // Register events
        app.add_event::<KeyEvent>();
        app.add_event::<CancelJob>();

        // Add system
        app.add_systems(Update, handle_queue_input);

        // Simulate down arrow key press
        app.world_mut().send_event(create_key_event(KeyCode::Down));
        app.update();

        // Verify selection didn't change (different screen)
        let queue_state = app.world().resource::<QueueState>();
        assert_eq!(queue_state.selected, 0); // Should still be 0
    }
}
