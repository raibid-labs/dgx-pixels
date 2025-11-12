use crate::app::App;
use crate::ui::{layout::create_layout, theme::Theme};
use crate::ui::screens::{create_block, create_header, create_status_bar};
use ratatui::{Frame, widgets::Paragraph, text::Line};

#[allow(unused_variables)]
pub fn render(f: &mut Frame, app: &App) {
    let chunks = create_layout(f.size());

    let header = create_header("Settings");
    f.render_widget(header, chunks[0]);

    let lines = vec![
        Line::from(""),
        Line::from("General Settings:"),
        Line::from("  Theme: Default"),
        Line::from("  Auto-save: Enabled"),
        Line::from(""),
        Line::from("Generation Defaults:"),
        Line::from("  Model: SDXL Base"),
        Line::from("  Steps: 30"),
        Line::from("  CFG Scale: 7.5"),
        Line::from(""),
        Line::from("Paths:"),
        Line::from("  Output: ./output"),
        Line::from("  Models: ./models"),
    ];

    let paragraph = Paragraph::new(lines)
        .block(create_block(" Settings "))
        .style(Theme::text());

    f.render_widget(paragraph, chunks[1]);

    let status = create_status_bar("Config: ~/.config/dgx-pixels/config.toml");
    f.render_widget(status, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[tokio::test]
    async fn test_settings_screen_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();
        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }
}
