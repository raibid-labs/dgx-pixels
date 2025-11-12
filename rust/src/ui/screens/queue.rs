use crate::app::App;
use crate::ui::{layout::create_layout, theme::Theme};
use crate::ui::screens::{create_block, create_header, create_status_bar};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
    text::{Line, Span},
};

/// Render the queue screen
pub fn render(f: &mut Frame, app: &App) {
    let chunks = create_layout(f.size());

    // Header
    let header = create_header("Job Queue");
    f.render_widget(header, chunks[0]);

    // Body
    render_body(f, chunks[1]);

    // Status bar
    let status = create_status_bar("Active: 0 | Queued: 0 | Completed: 0 | Failed: 0");
    f.render_widget(status, chunks[2]);
}

fn render_body(f: &mut Frame, area: ratatui::layout::Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Active jobs
            Constraint::Percentage(40),  // Completed jobs
            Constraint::Percentage(20),  // Queue stats
        ])
        .margin(1)
        .split(area);

    // Active jobs
    render_active_jobs(f, body_chunks[0]);

    // Completed jobs
    render_completed_jobs(f, body_chunks[1]);

    // Queue stats
    render_queue_stats(f, body_chunks[2]);
}

fn render_active_jobs(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("No active jobs", Theme::muted())),
    ];

    let paragraph = Paragraph::new(lines)
        .block(create_block(" Active Jobs (0) "));

    f.render_widget(paragraph, area);
}

fn render_completed_jobs(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("No completed jobs", Theme::muted())),
    ];

    let paragraph = Paragraph::new(lines)
        .block(create_block(" Completed Jobs (0) "));

    f.render_widget(paragraph, area);
}

fn render_queue_stats(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(vec![
            Span::raw("Total Jobs: "),
            Span::styled("0", Theme::text()),
            Span::raw("  |  Avg Time: "),
            Span::styled("--", Theme::muted()),
            Span::raw("  |  Success Rate: "),
            Span::styled("--", Theme::muted()),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(create_block(" Queue Statistics "));

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_queue_screen_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();

        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }
}
