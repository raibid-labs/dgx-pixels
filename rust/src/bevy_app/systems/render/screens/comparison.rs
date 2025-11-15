use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::resources::{AppTheme, ComparisonState, CurrentScreen, Screen};

/// Render the Comparison screen
pub fn render_comparison_screen(
    current_screen: Res<CurrentScreen>,
    comparison: Res<ComparisonState>,
    theme: Res<AppTheme>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Comparison {
        return;
    }

    ratatui
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Controls
                ])
                .split(frame.area());

            // Title
            render_title(frame, chunks[0], &theme);

            // Content
            render_content(frame, chunks[1], &comparison, &theme);

            // Controls
            render_controls(frame, chunks[2], &comparison, &theme);
        })
        .expect("Failed to render comparison screen");
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(" Model Comparison")
        .style(theme.header())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.highlight()),
        );
    frame.render_widget(title, area);
}

fn render_content(frame: &mut Frame, area: Rect, comparison: &ComparisonState, theme: &AppTheme) {
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Model selection
            Constraint::Length(5), // Prompt input
            Constraint::Min(0),    // Results area
        ])
        .margin(1)
        .split(area);

    // Model selection
    render_model_selection(frame, content_chunks[0], comparison, theme);

    // Prompt input
    render_prompt_input(frame, content_chunks[1], comparison, theme);

    // Results area
    render_results(frame, content_chunks[2], comparison, theme);
}

fn render_model_selection(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
) {
    let mut lines = vec![Line::from("")];

    for (idx, model) in comparison.models.iter().enumerate() {
        let is_selected = idx == comparison.selected_index;
        let (prefix, style) = if is_selected {
            ("▶ ", theme.highlight())
        } else {
            ("  ", theme.text())
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("Slot {}: ", idx + 1), theme.muted()),
            Span::styled(model, style),
        ]));
    }

    // Add slot indicator if less than 3 models
    if comparison.models.len() < 3 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  + ", theme.muted()),
            Span::styled(
                format!(
                    "Add model (press 'a') - {} slots available",
                    3 - comparison.models.len()
                ),
                theme.muted(),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(format!(" Models Selected ({}/3) ", comparison.models.len()))
            .borders(Borders::ALL)
            .border_style(theme.highlight()),
    );

    frame.render_widget(paragraph, area);
}

fn render_prompt_input(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
) {
    let prompt_text = if comparison.prompt.is_empty() {
        "[Enter prompt for comparison...]"
    } else {
        &comparison.prompt
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Prompt: ", theme.muted()),
            Span::styled(
                prompt_text,
                if comparison.prompt.is_empty() {
                    theme.muted()
                } else {
                    theme.text()
                },
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Shared Prompt ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_results(frame: &mut Frame, area: Rect, comparison: &ComparisonState, theme: &AppTheme) {
    if comparison.is_running {
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "⚙ Generating images with all models...",
                theme.highlight(),
            )),
            Line::from(""),
            Line::from(Span::styled("This may take several minutes", theme.muted())),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Comparison Running ")
                    .borders(Borders::ALL)
                    .border_style(theme.highlight()),
            )
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    } else {
        // Split results area into columns (one per model)
        let num_models = comparison.models.len();
        if num_models == 0 {
            let empty_msg = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled("No models selected", theme.muted())),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'a' to add models for comparison",
                    theme.muted(),
                )),
            ])
            .block(
                Block::default()
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(theme.text()),
            );
            frame.render_widget(empty_msg, area);
            return;
        }

        let constraints: Vec<Constraint> = (0..num_models)
            .map(|_| Constraint::Percentage(100 / num_models as u16))
            .collect();

        let result_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        for (idx, model) in comparison.models.iter().enumerate() {
            render_model_result(frame, result_chunks[idx], model, theme);
        }
    }
}

fn render_model_result(frame: &mut Frame, area: Rect, model: &str, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("[Image preview placeholder]", theme.muted())),
        Line::from(""),
        Line::from(Span::styled("WS-06: Sixel rendering", theme.muted())),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Speed: ", theme.muted()),
            Span::styled("--", theme.text()),
        ]),
        Line::from(vec![
            Span::styled("Quality: ", theme.muted()),
            Span::styled("--", theme.text()),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" {} ", model))
                .borders(Borders::ALL)
                .border_style(theme.text()),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_controls(frame: &mut Frame, area: Rect, comparison: &ComparisonState, theme: &AppTheme) {
    let can_run = !comparison.models.is_empty() && !comparison.prompt.is_empty();

    let controls = vec![Line::from(vec![
        Span::raw("  "),
        Span::styled("←/→", theme.highlight()),
        Span::raw(" Navigate  "),
        Span::styled("a", theme.highlight()),
        Span::raw(" Add  "),
        Span::styled("d", theme.highlight()),
        Span::raw(" Remove  "),
        Span::styled(
            "Enter",
            if can_run {
                theme.highlight()
            } else {
                theme.muted()
            },
        ),
        Span::raw(if can_run {
            " Run"
        } else {
            " Run (need models & prompt)"
        }),
    ])];

    let paragraph = Paragraph::new(controls).block(
        Block::default()
            .title(" Controls ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_comparison_screen_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(AppTheme::default());
        app.insert_resource(ComparisonState::default());
        app.add_systems(Update, render_comparison_screen);
    }
}
