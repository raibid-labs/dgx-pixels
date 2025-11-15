//! # Rendering Dispatch System
//!
//! Routes rendering to screen-specific implementations based on CurrentScreen.

use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::resources::*;

/// Main rendering dispatch system.
///
/// Called in the Update schedule, renders the current screen using RatatuiContext.
pub fn render_dispatch(
    mut ratatui: ResMut<RatatuiContext>,
    current_screen: Res<CurrentScreen>,
    mut app_state: ResMut<AppState>,
) {
    // Always render on first frame or when redraw requested
    // (bevy_ratatui handles frame limiting internally)

    // Render the current screen
    let screen = current_screen.0;

    ratatui
        .draw(|frame| {
            // Create main layout with status bar
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // Main content
                    Constraint::Length(1), // Status bar
                ])
                .split(frame.area());

            // Render screen-specific content (placeholders for now)
            match screen {
                Screen::Generation => render_placeholder(
                    frame,
                    chunks[0],
                    "Generation",
                    "Enter prompt and press Enter to generate. Number keys 1-8 switch screens.",
                ),
                Screen::Comparison => render_placeholder(
                    frame,
                    chunks[0],
                    "Comparison",
                    "Side-by-side model comparison. Will be implemented in WS-10.",
                ),
                Screen::Queue => render_placeholder(
                    frame,
                    chunks[0],
                    "Queue",
                    "View generation job queue. Will be implemented in WS-11.",
                ),
                Screen::Gallery => render_placeholder(
                    frame,
                    chunks[0],
                    "Gallery",
                    "Browse generated images. Will be implemented in WS-12.",
                ),
                Screen::Models => render_placeholder(
                    frame,
                    chunks[0],
                    "Models",
                    "Manage AI models and LoRAs. Will be implemented in WS-13.",
                ),
                Screen::Monitor => render_placeholder(
                    frame,
                    chunks[0],
                    "Monitor",
                    "System monitoring and performance. Will be implemented in WS-14.",
                ),
                Screen::Settings => render_placeholder(
                    frame,
                    chunks[0],
                    "Settings",
                    "Application settings. Will be implemented in WS-15.",
                ),
                Screen::Help => render_placeholder(
                    frame,
                    chunks[0],
                    "Help",
                    "Keyboard shortcuts and help. Will be implemented in WS-16.",
                ),
            }

            // Render status bar
            render_status_bar(frame, chunks[1], screen, &app_state);
        })
        .expect("Failed to render frame");

    // Mark frame as rendered
    app_state.mark_rendered();
}

/// Render a placeholder screen with title and description.
fn render_placeholder(frame: &mut Frame, area: Rect, title: &str, description: &str) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(description, Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation: Tab/BackTab or number keys 1-8",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "Quit: Press 'q' (except on Generation screen)",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render the status bar showing current screen and app state.
fn render_status_bar(frame: &mut Frame, area: Rect, screen: Screen, app_state: &AppState) {
    let status_text = format!(
        " Screen: {:?} | Frame: {} | Debug: {} | Press ? for help ",
        screen,
        app_state.frame_count,
        if app_state.debug_mode { "ON" } else { "OFF" }
    );

    let paragraph =
        Paragraph::new(status_text).style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_render_dispatch_compiles() {
        // Basic compilation test
        // Full rendering tests require bevy_ratatui terminal setup
        let mut app = App::new();
        app.add_systems(Update, render_dispatch);
    }
}
