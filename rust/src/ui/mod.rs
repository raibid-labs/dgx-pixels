pub mod screens;
pub mod theme;
pub mod layout;

use crate::app::App;
use ratatui::{
    backend::Backend,
    Terminal,
    Frame,
};

pub use theme::Theme;
pub use layout::create_layout;

/// Main render function - dispatches to appropriate screen
pub fn render<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> anyhow::Result<()> {
    terminal.draw(|f| {
        ui(f, app);
    })?;
    Ok(())
}

/// UI rendering logic
fn ui(f: &mut Frame, app: &App) {
    use crate::app::Screen;

    match app.current_screen {
        Screen::Generation => screens::generation::render(f, app),
        Screen::Comparison => screens::comparison::render(f, app, &app.comparison_state),
        Screen::Queue => screens::queue::render(f, app),
        Screen::Gallery => screens::gallery::render(f, app),
        Screen::Models => screens::models::render(f, app),
        Screen::Monitor => screens::monitor::render(f, app),
        Screen::Settings => screens::settings::render(f, app),
        Screen::Help => screens::help::render(f, app),
    }
}
