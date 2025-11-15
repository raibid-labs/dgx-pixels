use crate::app::App;
use crate::ui::screens::{create_block, create_header, create_status_bar};
use crate::ui::{layout::create_layout, theme::Theme};
use ratatui::{text::Line, widgets::Paragraph, Frame};

#[allow(unused_variables)]
pub fn render(f: &mut Frame, app: &App) {
    let chunks = create_layout(f.area());

    let header = create_header("System Monitor");
    f.render_widget(header, chunks[0]);

    let lines = vec![
        Line::from(""),
        Line::from("GPU Metrics:"),
        Line::from("  Utilization: --"),
        Line::from("  Temperature: --"),
        Line::from("  Memory: --"),
        Line::from(""),
        Line::from("System Resources:"),
        Line::from("  CPU: --"),
        Line::from("  RAM: --"),
    ];

    let paragraph = Paragraph::new(lines)
        .block(create_block(" GPU & System Monitor "))
        .style(Theme::text());

    f.render_widget(paragraph, chunks[1]);

    let status = create_status_bar("GPU: -- | Mem: --/128GB | Jobs: 0");
    f.render_widget(status, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[tokio::test]
    async fn test_monitor_screen_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();
        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }
}
