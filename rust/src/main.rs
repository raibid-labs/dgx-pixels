use anyhow::Result;
use clap::Parser;
use tracing::info;

/// DGX-Pixels TUI - AI Pixel Art Generation
#[derive(Parser, Debug)]
#[command(name = "dgx-pixels-tui")]
#[command(about = "Terminal UI for AI pixel art generation", long_about = None)]
struct Args {
    /// Enable debug mode with live backend logs
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
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

    // Run either Bevy-based or classic mode based on feature flag
    #[cfg(feature = "bevy_migration_foundation")]
    {
        info!("Starting Bevy-based DGX-Pixels TUI");
        run_bevy_app()
    }

    #[cfg(not(feature = "bevy_migration_foundation"))]
    {
        info!("Starting classic ratatui DGX-Pixels TUI");
        // Set debug mode in classic app if needed
        if args.debug {
            // Store debug flag for classic app
            // Note: This is a temporary workaround until WS-02 (state migration)
            // In the future, this will be stored in Bevy resources
            std::env::set_var("DGX_PIXELS_DEBUG", "1");
        }
        dgx_pixels_tui::run_classic_app()
    }
}

#[cfg(feature = "bevy_migration_foundation")]
fn run_bevy_app() -> Result<()> {
    use bevy::prelude::*;
    use dgx_pixels_tui::bevy_app::DgxPixelsPlugin;

    App::new().add_plugins(DgxPixelsPlugin).run();

    Ok(())
}
