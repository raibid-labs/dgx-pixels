mod app;
mod comparison;  // NEW: Comparison system
mod events;
mod messages;
mod reports;     // NEW: Report export
mod zmq_client;
mod ui;
mod sixel;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting DGX-Pixels TUI v0.1.0");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

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
    loop {
        // Process preview results from async worker
        while let Some(preview_result) = app.preview_manager.try_recv_result() {
            if preview_result.entry.is_some() {
                info!("Preview ready: {:?}", preview_result.path);
                app.needs_redraw = true;
            }
        }

        // Render UI
        ui::render(terminal, app)?;
        app.mark_rendered();

        // Handle events with timeout
        if event::poll(Duration::from_millis(16))? { // 60Hz target
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
