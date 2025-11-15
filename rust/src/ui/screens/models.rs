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
    let chunks = create_layout(f.area());

    let header = create_header("Model Manager");
    f.render_widget(header, chunks[0]);

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Base Models", Theme::header())]),
        Line::from("  No models loaded"),
        Line::from(""),
        Line::from(vec![Span::styled("LoRA Adapters", Theme::header())]),
        Line::from("  No LoRAs loaded"),
    ];

    let paragraph = Paragraph::new(lines).block(create_block(" Model Manager "));

    f.render_widget(paragraph, chunks[1]);

    let status = create_status_bar("Loaded: 0 base, 0 LoRA | Memory: 0/128 GB");
    f.render_widget(status, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[tokio::test]
    async fn test_models_screen_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();
        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }
}
