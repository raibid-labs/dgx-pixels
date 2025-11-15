<<<<<<< HEAD
//! # Generation Screen Renderer
//!
//! Renders the main Generation screen where users enter prompts and view results.
//! This is a direct port of `ui/screens/generation.rs` to the Bevy ECS architecture.

=======
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

<<<<<<< HEAD
use crate::bevy_app::{
    components::{Job, JobStatus},
    resources::{AppState, AppTheme, CurrentScreen, GalleryState, InputBuffer, Screen},
};

/// Render the Generation screen.
///
/// This system only runs when CurrentScreen is Screen::Generation.
pub fn render_generation_screen(
    mut ratatui: ResMut<RatatuiContext>,
=======
use crate::bevy_app::components::{Job, JobStatus};
use crate::bevy_app::resources::{AppState, AppTheme, CurrentScreen, GalleryState, InputBuffer, Screen};

/// Render the Generation screen
pub fn render_generation_screen(
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
    current_screen: Res<CurrentScreen>,
    input_buffer: Res<InputBuffer>,
    theme: Res<AppTheme>,
    app_state: Res<AppState>,
    gallery: Res<GalleryState>,
    jobs: Query<&Job>,
<<<<<<< HEAD
=======
    mut ratatui: ResMut<RatatuiContext>,
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
) {
    if current_screen.0 != Screen::Generation {
        return;
    }

<<<<<<< HEAD
    ratatui
        .draw(|frame| {
            render_frame(frame, &input_buffer, &theme, &app_state, &gallery, &jobs);
        })
        .ok();
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
=======
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
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Prompt input
            Constraint::Length(3), // Options row
<<<<<<< HEAD
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
=======
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
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
    };

    let paragraph = Paragraph::new(prompt_text).block(block);
    frame.render_widget(paragraph, area);

<<<<<<< HEAD
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
=======
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
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)

    frame.render_widget(paragraph, area);
}

<<<<<<< HEAD
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
=======
fn render_main_content(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Options/controls
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
            Constraint::Percentage(50), // Preview
        ])
        .split(area);

<<<<<<< HEAD
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
=======
    // Left: Options and controls
    render_controls(frame, main_chunks[0], jobs, theme);

    // Right: Preview area
    render_preview(frame, main_chunks[1], theme);
}

fn render_controls(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
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
<<<<<<< HEAD
    let active_jobs: Vec<&Job> = jobs.iter().filter(|j| j.is_active()).collect();
    if let Some(job) = active_jobs.first() {
=======
    if let Some(job) = jobs.iter().find(|j| matches!(j.status, JobStatus::Generating { .. })) {
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
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
<<<<<<< HEAD
            JobStatus::Generating {
                stage,
                progress,
                eta_s,
            } => {
=======
            JobStatus::Generating { stage, progress, eta_s } => {
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
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
<<<<<<< HEAD
        Span::styled(" [G]enerate ", theme.button()),
        Span::raw("  "),
        Span::styled(" [C]ompare Models ", theme.button()),
=======
        Span::styled(" [Enter] Generate ", theme.highlight()),
        Span::raw("  "),
        Span::styled(" [Esc] Clear ", theme.muted()),
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
    ]));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Generation Options ")
            .borders(Borders::ALL)
<<<<<<< HEAD
            .border_style(theme.border()),
=======
            .border_style(theme.text()),
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
    );

    frame.render_widget(paragraph, area);
}

<<<<<<< HEAD
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
=======
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
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
    let lines = if gallery.images.is_empty() {
        vec![Line::from(Span::styled(
            "No recent generations",
            theme.muted(),
        ))]
    } else {
<<<<<<< HEAD
        let recent: Vec<_> = gallery
=======
        gallery
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
            .images
            .iter()
            .rev()
            .take(3)
            .map(|path| {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
<<<<<<< HEAD
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
=======
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(filename, theme.text()),
                ])
            })
            .collect()
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Recent Generations ")
            .borders(Borders::ALL)
<<<<<<< HEAD
            .border_style(theme.border()),
=======
            .border_style(theme.text()),
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
    );

    frame.render_widget(paragraph, area);
}

<<<<<<< HEAD
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_screen_compiles() {
        // Basic compilation test - full rendering requires bevy_ratatui setup
        let mut app = bevy::app::App::new();
=======
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
>>>>>>> 9d7cbb3 (WS-09: Implement Generation Screen migration to Bevy ECS)
        app.add_systems(Update, render_generation_screen);
    }
}
