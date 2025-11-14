mod app;
mod comparison; // NEW: Comparison system
mod events;
mod messages;
mod reports; // NEW: Report export
mod sixel;
mod ui;
mod zmq_client;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::time::Duration;
use tracing::{info, warn};
use zmq_client::ZmqClient;

/// DGX-Pixels TUI - AI Pixel Art Generation
#[derive(Parser, Debug)]
#[command(name = "dgx-pixels-tui")]
#[command(about = "Terminal UI for AI pixel art generation", long_about = None)]
struct Args {
    /// Enable debug mode with live backend logs
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging to file (not stdout, to avoid interfering with TUI)
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("dgx-pixels-tui.log")?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::sync::Mutex::new(log_file))
        .with_ansi(false) // No ANSI colors in log file
        .init();

    info!("Starting DGX-Pixels TUI v0.1.0 (debug={})", args.debug);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    app.debug_mode = args.debug;

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
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    info!("DGX-Pixels TUI exited");
    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    // Open backend log file if debug mode is enabled
    let mut log_reader = if app.debug_mode {
        match File::open("dgx-pixels-backend.log") {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                // Seek to end to only read new lines
                let _ = reader.seek(SeekFrom::End(0));
                Some(reader)
            }
            Err(e) => {
                warn!("Could not open backend log file: {}", e);
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
        use messages::{ProgressUpdate, Response};
        use std::path::PathBuf;

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
                    duration_s: _,
                } => {
                    info!("Job complete: {}, output: {}", job_id, image_path);
                    // Add to gallery
                    app.add_to_gallery(PathBuf::from(image_path));
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
                    // Add to gallery
                    app.add_to_gallery(PathBuf::from(image_path));
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
