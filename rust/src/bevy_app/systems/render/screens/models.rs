use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::bevy_app::resources::{AppTheme, CurrentScreen, ModelStatus, ModelsState, Screen};

/// Render the Models screen with model table and optional metadata panel
pub fn render_models_screen(
    current_screen: Res<CurrentScreen>,
    models_state: Res<ModelsState>,
    theme: Res<AppTheme>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Models {
        return;
    }

    ratatui
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Status bar
                ])
                .split(frame.area());

            // Title
            render_title(frame, chunks[0], &theme);

            // Content (split into table and metadata if metadata panel is visible)
            if models_state.show_metadata {
                let content_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(60), // Models table
                        Constraint::Percentage(40), // Metadata panel
                    ])
                    .split(chunks[1]);

                render_models_table(frame, content_chunks[0], &models_state, &theme);
                render_metadata_panel(frame, content_chunks[1], &models_state, &theme);
            } else {
                render_models_table(frame, chunks[1], &models_state, &theme);
            }

            // Status bar
            render_status_bar(frame, chunks[2], &models_state, &theme);
        })
        .expect("Failed to render models screen");
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(vec![
        Line::from(" Model Manager"),
        Line::from(" Manage AI models, LoRAs, and VAEs"),
    ])
    .style(theme.header())
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.highlight()),
    );
    frame.render_widget(title, area);
}

fn render_models_table(
    frame: &mut Frame,
    area: Rect,
    models_state: &ModelsState,
    theme: &AppTheme,
) {
    // Table headers
    let headers = Row::new(vec![
        Cell::from("Name").style(theme.highlight()),
        Cell::from("Type").style(theme.highlight()),
        Cell::from("Size").style(theme.highlight()),
        Cell::from("Status").style(theme.highlight()),
        Cell::from("Active").style(theme.highlight()),
    ]);

    // Table rows
    let rows: Vec<Row> = models_state
        .models
        .iter()
        .enumerate()
        .map(|(idx, model)| {
            // Status indicator with color
            let (status_text, status_style) = match &model.status {
                ModelStatus::Downloaded => ("✅".to_string(), theme.success()),
                ModelStatus::Available => ("⏳".to_string(), theme.warning()),
                ModelStatus::Downloading(pct) => (format!("📊 {}%", pct), theme.info()),
                ModelStatus::Failed => ("❌".to_string(), theme.error()),
            };

            // Active indicator
            let active_text = if models_state.active_model.as_ref() == Some(&model.name) {
                "✓"
            } else {
                ""
            };

            // Format size
            let size_text = format_size(model.size_mb);

            let row = Row::new(vec![
                Cell::from(model.name.clone()),
                Cell::from(model.model_type.to_string()),
                Cell::from(size_text),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(active_text),
            ]);

            // Highlight selected row
            if idx == models_state.selected_index {
                row.style(theme.highlight())
            } else {
                row.style(theme.text())
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35), // Name
            Constraint::Percentage(15), // Type
            Constraint::Percentage(15), // Size
            Constraint::Percentage(20), // Status
            Constraint::Percentage(15), // Active
        ],
    )
    .header(headers)
    .block(
        Block::default()
            .title(" Models ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(table, area);
}

fn render_metadata_panel(
    frame: &mut Frame,
    area: Rect,
    models_state: &ModelsState,
    theme: &AppTheme,
) {
    let selected_model = models_state.selected_model();

    let lines = if let Some(model) = selected_model {
        vec![
            Line::from(vec![
                Span::styled("Name: ", theme.highlight()),
                Span::raw(&model.name),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Version: ", theme.highlight()),
                Span::raw(&model.metadata.version),
            ]),
            Line::from(vec![
                Span::styled("Type: ", theme.highlight()),
                Span::raw(model.model_type.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Parameters: ", theme.highlight()),
                Span::raw(&model.metadata.parameters),
            ]),
            Line::from(vec![
                Span::styled("Size: ", theme.highlight()),
                Span::raw(format!("{} MB", model.size_mb)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("License: ", theme.highlight()),
                Span::raw(&model.metadata.license),
            ]),
            Line::from(vec![
                Span::styled("Source: ", theme.highlight()),
                Span::raw(&model.metadata.source),
            ]),
            Line::from(""),
            Line::from(Span::styled("Description:", theme.highlight())),
            Line::from(model.metadata.description.clone()),
        ]
    } else {
        vec![Line::from("No model selected")]
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Metadata ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, models_state: &ModelsState, theme: &AppTheme) {
    let (downloaded_mb, total_mb) = models_state.memory_stats();
    let downloaded_gb = downloaded_mb as f64 / 1024.0;
    let total_gb = total_mb as f64 / 1024.0;

    let downloaded_count = models_state
        .models
        .iter()
        .filter(|m| matches!(m.status, ModelStatus::Downloaded))
        .count();

    let status_lines = vec![
        Line::from(vec![
            Span::styled("Models: ", theme.highlight()),
            Span::raw(format!(
                "{}/{} | ",
                downloaded_count,
                models_state.models.len()
            )),
            Span::styled("Storage: ", theme.highlight()),
            Span::raw(format!("{:.1} GB / {:.1} GB", downloaded_gb, total_gb)),
        ]),
        Line::from(vec![
            Span::raw("↑/↓:Navigate | "),
            Span::raw("Enter:Activate | "),
            Span::raw("d:Download | "),
            Span::raw("Del:Remove | "),
            Span::raw("i:Info"),
        ]),
    ];

    let paragraph = Paragraph::new(status_lines)
        .block(
            Block::default()
                .title(" Stats ")
                .borders(Borders::ALL)
                .border_style(theme.text()),
        )
        .style(theme.status_bar());

    frame.render_widget(paragraph, area);
}

/// Format size in MB to human-readable format (MB or GB)
fn format_size(size_mb: usize) -> String {
    if size_mb >= 1024 {
        format!("{:.1}GB", size_mb as f64 / 1024.0)
    } else {
        format!("{}MB", size_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_models_screen_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Models));
        app.insert_resource(ModelsState::default());
        app.insert_resource(AppTheme::default());
        app.add_systems(Update, render_models_screen);
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(144), "144MB");
        assert_eq!(format_size(512), "512MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1024), "1.0GB");
        assert_eq!(format_size(6938), "6.8GB");
    }
}
