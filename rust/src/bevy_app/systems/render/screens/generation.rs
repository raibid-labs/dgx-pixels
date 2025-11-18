//! # Generation Screen Renderer
//!
//! Renders the main Generation screen where users enter prompts and view results.
//! This is a direct port of `ui/screens/generation.rs` to the Bevy ECS architecture.

use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::{
    components::{Job, JobStatus},
    resources::{AppState, AppTheme, CurrentScreen, GalleryState, InputBuffer, Screen},
};

/// Render the Generation screen.
///
/// This system only runs when CurrentScreen is Screen::Generation.
pub fn render_generation_screen(
    mut ratatui: ResMut<RatatuiContext>,
    current_screen: Res<CurrentScreen>,
    input_buffer: Res<InputBuffer>,
    theme: Res<AppTheme>,
    app_state: Res<AppState>,
    gallery: Res<GalleryState>,
    jobs: Query<&Job>,
) {
    if current_screen.0 != Screen::Generation {
        return;
    }

    trace!("Rendering generation screen");

    if let Err(e) = ratatui.draw(|frame| {
        render_frame(frame, &input_buffer, &theme, &app_state, &gallery, &jobs);
    }) {
        error!("Failed to render generation screen: {:?}", e);
    }
}

/// Render the complete frame layout.
fn render_frame(
    frame: &mut Frame,
    input_buffer: &InputBuffer,
    theme: &AppTheme,
    app_state: &AppState,
    gallery: &GalleryState,
    jobs: &Query<&Job>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Prompt input
            Constraint::Length(3), // Options row
            Constraint::Min(8),    // Main content (controls + preview)
            Constraint::Length(6), // Recent generations
        ])
        .margin(1)
        .split(frame.area());

    render_prompt_input(frame, chunks[0], input_buffer, theme);
    render_options_row(frame, chunks[1], theme);
    render_main_content(frame, chunks[2], app_state, theme, jobs);
    render_recent_generations(frame, chunks[3], gallery, theme);
}

/// Render prompt input field with cursor.
fn render_prompt_input(
    frame: &mut Frame,
    area: Rect,
    input_buffer: &InputBuffer,
    theme: &AppTheme,
) {
    let block = Block::default()
        .title(" Prompt ")
        .borders(Borders::ALL)
        .border_style(theme.border());

    let inner = block.inner(area);

    let prompt_text = if input_buffer.text.is_empty() {
        Span::styled("Enter your prompt here...", theme.muted())
    } else {
        Span::styled(&input_buffer.text, theme.text())
    };

    let paragraph = Paragraph::new(prompt_text).block(block);
    frame.render_widget(paragraph, area);

    // Show cursor at current position
    if inner.width > 0 && inner.height > 0 {
        let cursor_x = inner.x + (input_buffer.cursor as u16).min(inner.width - 1);
        let cursor_y = inner.y;
        frame.set_cursor(cursor_x, cursor_y);
    }
}

/// Render options row (model, LoRA, size, steps).
fn render_options_row(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let options_text = " Model: [SDXL Base ▼]  LoRA: [None ▼]  Size: [1024x1024]  Steps: [30] ";
    let paragraph = Paragraph::new(options_text).style(theme.text()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border()),
    );

    frame.render_widget(paragraph, area);
}

/// Render main content area (controls on left, preview on right).
fn render_main_content(
    frame: &mut Frame,
    area: Rect,
    app_state: &AppState,
    theme: &AppTheme,
    jobs: &Query<&Job>,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Controls
            Constraint::Percentage(50), // Preview
        ])
        .split(area);

    render_controls(frame, main_chunks[0], app_state, theme, jobs);
    render_preview(frame, main_chunks[1], app_state, theme);
}

/// Render generation controls and active job status.
fn render_controls(
    frame: &mut Frame,
    area: Rect,
    app_state: &AppState,
    theme: &AppTheme,
    jobs: &Query<&Job>,
) {
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
    let active_jobs: Vec<&Job> = jobs.iter().filter(|j| j.is_active()).collect();
    if let Some(job) = active_jobs.first() {
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
            JobStatus::Generating {
                stage,
                progress,
                eta_s,
            } => {
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
        Span::styled(" [G]enerate ", theme.button()),
        Span::raw("  "),
        Span::styled(" [C]ompare Models ", theme.button()),
    ]));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Generation Options ")
            .borders(Borders::ALL)
            .border_style(theme.border()),
    );

    frame.render_widget(paragraph, area);
}

/// Render preview area with tabs support for debug mode.
fn render_preview(frame: &mut Frame, area: Rect, app_state: &AppState, theme: &AppTheme) {
    // Create title with tab support if debug mode
    let title_string = if app_state.debug_mode {
        let tab_titles = vec!["Preview", "Backend Logs"];
        format!(
            " {} [Ctrl+Tab/P/L] ",
            tab_titles
                .iter()
                .enumerate()
                .map(|(i, &t)| if i == app_state.preview_tab {
                    format!("▸{}", t)
                } else {
                    format!(" {}", t)
                })
                .collect::<Vec<_>>()
                .join(" │ ")
        )
    } else {
        " Preview ".to_string()
    };

    let block = Block::default()
        .title(title_string)
        .borders(Borders::ALL)
        .border_style(theme.border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render content based on selected tab
    if app_state.debug_mode && app_state.preview_tab == 1 {
        render_backend_logs(frame, inner, app_state, theme);
    } else {
        render_preview_content(frame, inner, app_state, theme);
    }
}

/// Render preview content (image info or placeholder).
fn render_preview_content(frame: &mut Frame, area: Rect, app_state: &AppState, theme: &AppTheme) {
    if let Some(preview_path) = &app_state.current_preview {
        let filename = preview_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let size_str = if let Ok(metadata) = std::fs::metadata(preview_path) {
            let size_kb = metadata.len() / 1024;
            format!("{} KB", size_kb)
        } else {
            "Unknown size".to_string()
        };

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled("✓ Generation Complete", theme.success())),
            Line::from(""),
            Line::from(format!("File: {}", filename)),
            Line::from(format!("Size: {}", size_str)),
            Line::from(""),
            Line::from(Span::styled(
                "Preview: Sixel rendering coming soon!",
                theme.muted(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "For now, check outputs/ folder",
                theme.muted(),
            )),
        ];

        let paragraph = Paragraph::new(lines)
            .style(theme.text())
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    } else {
        // No preview available
        let lines = vec![
            Line::from(""),
            Line::from("    [Preview Area]"),
            Line::from(""),
            Line::from("  Image preview will"),
            Line::from("  appear here after"),
            Line::from("  generation"),
        ];

        let paragraph = Paragraph::new(lines)
            .style(theme.muted())
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

/// Render backend logs (debug mode, tab 1).
fn render_backend_logs(frame: &mut Frame, area: Rect, app_state: &AppState, theme: &AppTheme) {
    let lines: Vec<Line> = if app_state.backend_logs.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled("No backend logs yet", theme.muted())),
            Line::from(""),
            Line::from(Span::styled(
                "Logs will appear here as the backend processes requests",
                theme.muted(),
            )),
        ]
    } else {
        // Show last N lines that fit in the area
        let max_lines = area.height.saturating_sub(2) as usize;
        let start_idx = app_state.backend_logs.len().saturating_sub(max_lines);

        app_state.backend_logs[start_idx..]
            .iter()
            .map(|log_line| {
                // Color code log levels
                if log_line.contains("ERROR") || log_line.contains("Error") {
                    Line::from(Span::styled(log_line, theme.error()))
                } else if log_line.contains("WARN") || log_line.contains("Warning") {
                    Line::from(Span::styled(log_line, theme.warning()))
                } else if log_line.contains("INFO") {
                    Line::from(Span::styled(log_line, theme.text()))
                } else {
                    Line::from(Span::styled(log_line, theme.muted()))
                }
            })
            .collect()
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Render recent generations list.
fn render_recent_generations(
    frame: &mut Frame,
    area: Rect,
    gallery: &GalleryState,
    theme: &AppTheme,
) {
    let lines = if gallery.images.is_empty() {
        vec![Line::from(Span::styled(
            "No recent generations",
            theme.muted(),
        ))]
    } else {
        let recent: Vec<_> = gallery
            .images
            .iter()
            .rev()
            .take(3)
            .map(|path| {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                Line::from(vec![Span::raw("  "), Span::styled(filename, theme.text())])
            })
            .collect();

        if recent.is_empty() {
            vec![Line::from(Span::styled(
                "No recent generations",
                theme.muted(),
            ))]
        } else {
            recent
        }
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Recent Generations ")
            .borders(Borders::ALL)
            .border_style(theme.border()),
    );

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_screen_compiles() {
        // Basic compilation test - full rendering requires bevy_ratatui setup
        let mut app = bevy::app::App::new();
        app.add_systems(Update, render_generation_screen);
    }
}
