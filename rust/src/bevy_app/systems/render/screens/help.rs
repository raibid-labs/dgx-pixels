use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::resources::{AppTheme, CurrentScreen, Screen};

/// Render the Help screen
pub fn render_help_screen(
    current_screen: Res<CurrentScreen>,
    theme: Res<AppTheme>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Help {
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
        .expect("Failed to render help screen");
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(" Help").style(theme.header()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.highlight()),
    );
    frame.render_widget(title, area);
}

fn render_content(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("NAVIGATION", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab          ", theme.muted()),
            Span::raw("Next screen"),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab    ", theme.muted()),
            Span::raw("Previous screen"),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+C   ", theme.muted()),
            Span::raw("Quit application"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("GENERATION SCREEN", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Type         ", theme.muted()),
            Span::raw("Enter prompt text"),
        ]),
        Line::from(vec![
            Span::styled("  Enter        ", theme.muted()),
            Span::raw("Submit job for generation"),
        ]),
        Line::from(vec![
            Span::styled("  Esc          ", theme.muted()),
            Span::raw("Clear prompt"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("COMPARISON SCREEN", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓          ", theme.muted()),
            Span::raw("Navigate model list"),
        ]),
        Line::from(vec![
            Span::styled("  a / A        ", theme.muted()),
            Span::raw("Add model to comparison"),
        ]),
        Line::from(vec![
            Span::styled("  d / D        ", theme.muted()),
            Span::raw("Remove selected model"),
        ]),
        Line::from(vec![
            Span::styled("  Enter        ", theme.muted()),
            Span::raw("Generate with selected models"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("QUEUE SCREEN", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓          ", theme.muted()),
            Span::raw("Navigate queue"),
        ]),
        Line::from(vec![
            Span::styled("  c / C        ", theme.muted()),
            Span::raw("Cancel selected job"),
        ]),
        Line::from(vec![
            Span::styled("  r / R        ", theme.muted()),
            Span::raw("Retry failed job"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("GALLERY SCREEN", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ←/→          ", theme.muted()),
            Span::raw("Navigate gallery"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn    ", theme.muted()),
            Span::raw("Jump 10 images"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End     ", theme.muted()),
            Span::raw("First/Last image"),
        ]),
        Line::from(vec![
            Span::styled("  d / D        ", theme.muted()),
            Span::raw("Delete current image"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("MODELS SCREEN", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓          ", theme.muted()),
            Span::raw("Navigate models list"),
        ]),
        Line::from(vec![
            Span::styled("  a / A        ", theme.muted()),
            Span::raw("Activate selected model"),
        ]),
        Line::from(vec![
            Span::styled("  d / D        ", theme.muted()),
            Span::raw("Download model"),
        ]),
        Line::from(vec![
            Span::styled("  r / R        ", theme.muted()),
            Span::raw("Remove selected model"),
        ]),
        Line::from(vec![
            Span::styled("  i / I        ", theme.muted()),
            Span::raw("Toggle model info panel"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("MONITOR SCREEN", theme.highlight())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  r / R        ", theme.muted()),
            Span::raw("Refresh metrics (future)"),
        ]),
        Line::from(vec![
            Span::styled("  p / P        ", theme.muted()),
            Span::raw("Pause auto-refresh (future)"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "For more information, visit: https://github.com/raibid-labs/dgx-pixels",
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
    let status_text = "Use Tab/Shift+Tab to navigate screens | Press 'q' to quit";
    let paragraph = Paragraph::new(status_text).style(theme.status_bar());
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_help_screen_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Help));
        app.insert_resource(AppTheme::default());
        app.add_systems(Update, render_help_screen);
    }
}
