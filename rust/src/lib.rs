//! DGX-Pixels TUI - AI Pixel Art Generation
//!
//! This library provides the core functionality for the DGX-Pixels terminal user interface,
//! including both the classic ratatui implementation and the new Bevy ECS-based architecture.

// Public modules
pub mod app;
pub mod comparison;
pub mod events;
pub mod messages;
pub mod reports;
pub mod sixel;
pub mod ui;
pub mod zmq_client;

// Bevy app (feature-gated)
#[cfg(feature = "bevy_migration_foundation")]
pub mod bevy_app;

use anyhow::Result;

/// Run the classic ratatui-based application.
///
/// This is the original imperative event loop implementation that will be
/// gradually replaced by the Bevy ECS architecture during the migration.
#[tokio::main]
pub async fn run_classic_app() -> Result<()> {
    use app::App;
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;
    use tracing::{info, warn};
    use zmq_client::ZmqClient;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Check for debug flag from environment
    if std::env::var("DGX_PIXELS_DEBUG").is_ok() {
        app.debug_mode = true;
        app.preview_tab = 1; // Default to Logs tab in debug mode
    }

    // Initialize ZeroMQ client for backend communication
    match ZmqClient::new_default() {
        Ok(client) => {
            info!("ZeroMQ client connected");
            app.zmq_client = Some(client);
        }
        Err(e) => {
            warn!("Failed to connect to backend: {}", e);
            warn!("Generation features will be disabled");
        }
    }

    // Run the app
    let result = run_classic_event_loop(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = &result {
        eprintln!("Error: {}", err);
    }

    info!("DGX-Pixels TUI exited");
    result
}

/// Classic event loop for the ratatui-based TUI.
async fn run_classic_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut app::App,
) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;
    use std::time::Duration;
    use crossterm::event::{self, Event};
    use tracing::{info, warn};
    use messages::{ProgressUpdate, Response};

    // Open backend log file if debug mode is enabled
    let mut log_reader = if app.debug_mode {
        // Wait a moment for the log file to be created
        std::thread::sleep(std::time::Duration::from_millis(500));

        match File::open("dgx-pixels-backend.log") {
            Ok(f) => {
                info!("Successfully opened backend log file for tailing");
                let reader = BufReader::new(f);
                // Read all existing content first
                let mut initial_reader =
                    BufReader::new(File::open("dgx-pixels-backend.log").unwrap());
                let mut line = String::new();
                while initial_reader.read_line(&mut line).unwrap_or(0) > 0 {
                    app.backend_logs.push(line.trim_end().to_string());
                    line.clear();
                }
                if !app.backend_logs.is_empty() {
                    info!("Read {} initial log lines", app.backend_logs.len());
                    app.needs_redraw = true;
                }
                Some(reader)
            }
            Err(e) => {
                warn!("Could not open backend log file: {}", e);
                app.backend_logs
                    .push(format!("⚠️  Backend log file not found: {}", e));
                app.backend_logs
                    .push("Make sure the backend is running!".to_string());
                app.needs_redraw = true;
                None
            }
        }
    } else {
        None
    };

    loop {
        // Read new backend log lines if in debug mode
        if let Some(ref mut reader) = log_reader {
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                app.backend_logs.push(line.trim_end().to_string());
                // Keep only last 500 lines
                if app.backend_logs.len() > 500 {
                    app.backend_logs.remove(0);
                }
                app.needs_redraw = true;
                line.clear();
            }
        }

        // Process preview results from async worker
        while let Some(preview_result) = app.preview_manager.try_recv_result() {
            if preview_result.entry.is_some() {
                info!("Preview ready: {:?}", preview_result.path);
                app.needs_redraw = true;
            }
        }

        // Poll for ZMQ responses - collect first, then process
        let mut responses = Vec::new();
        let mut updates = Vec::new();

        if let Some(ref client) = app.zmq_client {
            while let Some(response) = client.try_recv_response() {
                responses.push(response);
            }
            while let Some(update) = client.try_recv_update() {
                updates.push(update);
            }
        }

        // Process responses
        for response in responses {
            match response {
                Response::JobAccepted {
                    job_id,
                    estimated_time_s: _,
                } => {
                    info!("Job accepted: {}", job_id);
                }
                Response::JobComplete {
                    job_id,
                    image_path,
                    duration_s,
                } => {
                    info!("Job complete: {}, output: {}", job_id, image_path);
                    let path = PathBuf::from(&image_path);
                    // Add to gallery
                    app.add_to_gallery(path.clone());
                    // Set as current preview
                    app.current_preview = Some(path);
                    // Update job status to complete
                    app.update_job_status(
                        &job_id,
                        app::JobStatus::Complete {
                            image_path: PathBuf::from(image_path),
                            duration_s,
                        },
                    );
                    app.needs_redraw = true;
                }
                Response::JobError { job_id, error } => {
                    warn!("Job {} failed: {}", job_id, error);
                }
                Response::Error { message } => {
                    warn!("Backend error: {}", message);
                }
                _ => {} // Ignore other response types
            }
        }

        // Process progress updates
        for update in updates {
            match update {
                ProgressUpdate::Progress {
                    job_id,
                    stage,
                    percent,
                    eta_s,
                    ..
                } => {
                    let stage_str = format!("{:?}", stage);
                    info!(
                        "Job {} progress: {}% ({})",
                        job_id,
                        percent as u32,
                        stage_str
                    );
                    app.update_job_status(
                        &job_id,
                        app::JobStatus::Running {
                            stage: stage_str,
                            progress: percent / 100.0,
                            eta_s,
                        },
                    );
                    app.needs_redraw = true;
                }
                ProgressUpdate::JobComplete {
                    job_id,
                    image_path,
                    duration_s,
                } => {
                    info!(
                        "Job {} completed in {:.1}s: {}",
                        job_id, duration_s, image_path
                    );
                    let path = PathBuf::from(&image_path);
                    // Add to gallery
                    app.add_to_gallery(path.clone());
                    // Set as current preview
                    app.current_preview = Some(path);
                    // Update job status to complete
                    app.update_job_status(
                        &job_id,
                        app::JobStatus::Complete {
                            image_path: PathBuf::from(image_path),
                            duration_s,
                        },
                    );
                    app.needs_redraw = true;
                }
                _ => {} // Ignore other update types
            }
        }

        // Render UI
        ui::render(terminal, app)?;
        app.mark_rendered();

        // Handle events with timeout
        if event::poll(Duration::from_millis(16))? {
            // 60Hz target
            match event::read()? {
                Event::Key(key) => {
                    events::EventHandler::handle(app, events::AppEvent::Key(key));
                }
                Event::Resize(w, h) => {
                    events::EventHandler::handle(app, events::AppEvent::Resize(w, h));
                }
                _ => {}
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Small yield to prevent CPU spinning
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    Ok(())
}
