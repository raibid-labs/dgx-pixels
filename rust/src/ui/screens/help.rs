use crate::app::App;
use crate::ui::screens::{create_block, create_header, create_status_bar};
use crate::ui::{layout::create_layout, theme::Theme};
use ratatui::{
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[allow(unused_variables)]
pub fn render(f: &mut Frame, app: &App) {
    let chunks = create_layout(f.size());

    let header = create_header("Help");
    f.render_widget(header, chunks[0]);

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Keyboard Shortcuts", Theme::header())]),
        Line::from(""),
        Line::from(vec![Span::styled("  Global:", Theme::title())]),
        Line::from("    Q         - Quit application"),
        Line::from("    Esc       - Back to previous screen"),
        Line::from("    ? or H    - Show this help screen"),
        Line::from(""),
        Line::from(vec![Span::styled("  Navigation:", Theme::title())]),
        Line::from("    1         - Generation screen"),
        Line::from("    2         - Queue manager"),
        Line::from("    3         - Gallery"),
        Line::from("    4         - Model manager"),
        Line::from("    5         - System monitor"),
        Line::from("    6         - Settings"),
        Line::from(""),
        Line::from(vec![Span::styled("  Generation Screen:", Theme::title())]),
        Line::from("    G         - Generate image"),
        Line::from("    C         - Compare models"),
        Line::from("    Type      - Enter prompt text"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press Esc to return",
            Theme::secondary(),
        )]),
    ];

    let paragraph = Paragraph::new(lines).block(create_block(" Help & Keyboard Shortcuts "));

    f.render_widget(paragraph, chunks[1]);

    let status = create_status_bar("DGX-Pixels v0.1.0 | Press Esc to return");
    f.render_widget(status, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[tokio::test]
    async fn test_help_screen_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();
        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }
}
