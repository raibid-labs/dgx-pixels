pub mod generation;
pub mod comparison;  // NEW: Side-by-side comparison screen
pub mod queue;
pub mod gallery;
pub mod models;
pub mod monitor;
pub mod settings;
pub mod help;

// Common widgets used across screens
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    text::Span,
};
use crate::ui::theme::Theme;

/// Create a standard block with title and borders
pub fn create_block(title: &str) -> Block {
    Block::default()
        .title(Span::styled(title, Theme::title()))
        .borders(Borders::ALL)
        .border_style(Theme::border())
}

/// Create header with app title and shortcuts
pub fn create_header<'a>(subtitle: &str) -> Paragraph<'a> {
    let text = format!(
        " DGX-Pixels v0.1.0 - {} | [Q]uit [?]Help [1-7]Screens ",
        subtitle
    );
    Paragraph::new(text)
        .style(Theme::header())
        .block(Block::default().borders(Borders::ALL))
}

/// Create status bar with system info
pub fn create_status_bar<'a>(status: &'a str) -> Paragraph<'a> {
    Paragraph::new(status)
        .style(Theme::status_bar())
        .block(Block::default().borders(Borders::ALL))
}
