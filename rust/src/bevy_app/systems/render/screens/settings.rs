use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::resources::{AppTheme, CurrentScreen, Screen};

/// Render the Settings screen
pub fn render_settings_screen(
    current_screen: Res<CurrentScreen>,
    theme: Res<AppTheme>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Settings {
        return;
    }

    ratatui
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Content
                    Constraint::Length(1), // Status bar
                ])
                .split(frame.area());

            // Title
            render_title(frame, chunks[0], &theme);

            // Content
            render_content(frame, chunks[1], &theme);

            // Status bar
            render_status_bar(frame, chunks[2], &theme);
        })
        .expect("Failed to render settings screen");
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(" Settings").style(theme.header()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.highlight()),
    );
    frame.render_widget(title, area);
}

fn render_content(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("GENERAL SETTINGS", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Theme:        "),
            Span::styled("Default", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("  Auto-save:    "),
            Span::styled("Enabled", theme.success()),
        ]),
        Line::from(vec![
            Span::raw("  FPS Limit:    "),
            Span::styled("60", theme.text()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("GENERATION DEFAULTS", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Model:        "),
            Span::styled("SDXL Base", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("  Steps:        "),
            Span::styled("30", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("  CFG Scale:    "),
            Span::styled("7.5", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("  Size:         "),
            Span::styled("1024x1024", theme.text()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("PATHS", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Output:       "),
            Span::styled("./output", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("  Models:       "),
            Span::styled("./models", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("  Cache:        "),
            Span::styled("./cache", theme.text()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("BACKEND", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  ZMQ Host:     "),
            Span::styled("127.0.0.1:5555", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("  Timeout:      "),
            Span::styled("30s", theme.text()),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Note: Settings are read-only in this version",
            theme.muted(),
        )]),
        Line::from(vec![Span::styled(
            "Edit config file: ~/.config/dgx-pixels/config.toml",
            theme.muted(),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.text()),
        )
        .style(theme.text());

    frame.render_widget(paragraph, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let status_text = "Config: ~/.config/dgx-pixels/config.toml | Read-only mode";
    let paragraph = Paragraph::new(status_text).style(theme.status_bar());
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_settings_screen_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Settings));
        app.insert_resource(AppTheme::default());
        app.add_systems(Update, render_settings_screen);
    }
}
