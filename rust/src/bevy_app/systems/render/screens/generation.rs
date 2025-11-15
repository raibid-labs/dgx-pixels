use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::components::{Job, JobStatus};
use crate::bevy_app::resources::{AppState, AppTheme, CurrentScreen, GalleryState, InputBuffer, Screen};

/// Render the Generation screen
pub fn render_generation_screen(
    current_screen: Res<CurrentScreen>,
    input_buffer: Res<InputBuffer>,
    theme: Res<AppTheme>,
    app_state: Res<AppState>,
    gallery: Res<GalleryState>,
    jobs: Query<&Job>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Generation {
        return;
    }

    ratatui.draw(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(0),      // Body
                Constraint::Length(1),   // Status bar
            ])
            .split(frame.area());

        // Title
        render_title(frame, chunks[0], &theme);

        // Body
        render_body(frame, chunks[1], &input_buffer, &gallery, &jobs, &theme);

        // Status bar
        render_status_bar(frame, chunks[2], &theme);
    }).expect("Failed to render generation screen");
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(" Generation")
        .style(theme.header())
        .block(Block::default().borders(Borders::ALL).border_style(theme.highlight()));
    frame.render_widget(title, area);
}

fn render_body(
    frame: &mut Frame,
    area: Rect,
    input: &InputBuffer,
    gallery: &GalleryState,
    jobs: &Query<&Job>,
    theme: &AppTheme,
) {
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Prompt input
            Constraint::Length(3), // Options row
            Constraint::Min(8),    // Preview/options area
            Constraint::Length(6), // Recent generations
        ])
        .margin(1)
        .split(area);

    // Prompt input
    render_prompt_input(frame, body_chunks[0], input, theme);

    // Options row
    render_options_row(frame, body_chunks[1], theme);

    // Main content area
    render_main_content(frame, body_chunks[2], jobs, theme);

    // Recent generations
    render_recent_generations(frame, body_chunks[3], gallery, theme);
}

fn render_prompt_input(frame: &mut Frame, area: Rect, input: &InputBuffer, theme: &AppTheme) {
    let block = Block::default()
        .title(" Prompt ")
        .borders(Borders::ALL)
        .border_style(theme.highlight());
    let inner = block.inner(area);

    let prompt_text = if input.text.is_empty() {
        Span::styled("Enter your prompt here...", theme.muted())
    } else {
        Span::styled(&input.text, theme.text())
    };

    let paragraph = Paragraph::new(prompt_text).block(block);
    frame.render_widget(paragraph, area);

    // Show cursor
    if input.cursor < input.text.len() {
        frame.set_cursor(inner.x + input.cursor as u16, inner.y);
    } else {
        frame.set_cursor(inner.x + input.text.len() as u16, inner.y);
    }
}

fn render_options_row(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let options_text = " Model: [SDXL Base ▼]  LoRA: [None ▼]  Size: [1024x1024]  Steps: [30] ";
    let paragraph = Paragraph::new(options_text)
        .style(theme.text())
        .block(Block::default().borders(Borders::ALL).border_style(theme.text()));

    frame.render_widget(paragraph, area);
}

fn render_main_content(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Options/controls
            Constraint::Percentage(50), // Preview
        ])
        .split(area);

    // Left: Options and controls
    render_controls(frame, main_chunks[0], jobs, theme);

    // Right: Preview area
    render_preview(frame, main_chunks[1], theme);
}

fn render_controls(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
    let mut lines = vec![
        Line::from(vec![
            Span::raw("Steps:       "),
            Span::styled("30", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("CFG Scale:   "),
            Span::styled("7.5", theme.text()),
        ]),
        Line::from(vec![
            Span::raw("Seed:        "),
            Span::styled("Random", theme.muted()),
        ]),
        Line::from(vec![
            Span::raw("Batch Size:  "),
            Span::styled("1", theme.text()),
        ]),
        Line::from(""),
    ];

    // Show active job progress if any
    if let Some(job) = jobs.iter().find(|j| matches!(j.status, JobStatus::Generating { .. })) {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Active Job: ", theme.highlight()),
            Span::styled(&job.prompt[..job.prompt.len().min(30)], theme.text()),
        ]));

        match &job.status {
            JobStatus::Pending => {
                lines.push(Line::from(Span::styled("Status: Pending", theme.muted())));
            }
            JobStatus::Queued => {
                lines.push(Line::from(Span::styled("Status: Queued", theme.muted())));
            }
            JobStatus::Generating { stage, progress, eta_s } => {
                lines.push(Line::from(vec![
                    Span::raw("Stage: "),
                    Span::styled(stage, theme.text()),
                ]));
                lines.push(Line::from(vec![
                    Span::raw("Progress: "),
                    Span::styled(format!("{:.0}%", progress * 100.0), theme.highlight()),
                    Span::raw(format!(" (ETA: {:.1}s)", eta_s)),
                ]));
            }
            JobStatus::Complete { duration_s, .. } => {
                lines.push(Line::from(vec![
                    Span::styled("Complete!", theme.success()),
                    Span::raw(format!(" ({:.1}s)", duration_s)),
                ]));
            }
            JobStatus::Failed { error } => {
                lines.push(Line::from(Span::styled(
                    format!("Error: {}", error),
                    theme.error(),
                )));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [Enter] Generate ", theme.highlight()),
        Span::raw("  "),
        Span::styled(" [Esc] Clear ", theme.muted()),
    ]));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Generation Options ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_preview(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from("    [Preview Area]"),
        Line::from(""),
        Line::from("  Image preview will"),
        Line::from("  appear here after"),
        Line::from("  generation"),
        Line::from(""),
        Line::from(Span::styled("(WS-06 Image Assets)", theme.muted())),
    ];

    let paragraph = Paragraph::new(lines)
        .style(theme.muted())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Preview ")
                .borders(Borders::ALL)
                .border_style(theme.text()),
        );

    frame.render_widget(paragraph, area);
}

fn render_recent_generations(frame: &mut Frame, area: Rect, gallery: &GalleryState, theme: &AppTheme) {
    let lines = if gallery.images.is_empty() {
        vec![Line::from(Span::styled(
            "No recent generations",
            theme.muted(),
        ))]
    } else {
        gallery
            .images
            .iter()
            .rev()
            .take(3)
            .map(|path| {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(filename, theme.text()),
                ])
            })
            .collect()
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Recent Generations ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let status_text = "GPU: Ready | Memory: 104GB free | Press Tab for next screen";
    let paragraph = Paragraph::new(status_text).style(theme.status_bar());
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_generation_screen_compiles() {
        // This test just verifies the system compiles and can be instantiated
        let mut app = App::new();
        app.insert_resource(CurrentScreen::default());
        app.insert_resource(InputBuffer::default());
        app.insert_resource(AppTheme::default());
        app.insert_resource(AppState::default());
        app.insert_resource(GalleryState::default());

        // Don't actually run the system - it requires RatatuiContext which needs full setup
        // Just verify it can be added
        app.add_systems(Update, render_generation_screen);
    }
}
