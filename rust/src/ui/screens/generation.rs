use crate::app::{App, JobStatus};
use crate::sixel::{RenderOptions, TerminalCapability};
use crate::ui::screens::{create_block, create_header, create_status_bar};
use crate::ui::{layout::create_layout, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::path::Path;

/// Render the generation screen
pub fn render(f: &mut Frame, app: &App) {
    let chunks = create_layout(f.size());

    // Header
    let header = create_header("Generation");
    f.render_widget(header, chunks[0]);

    // Body
    render_body(f, chunks[1], app);

    // Status bar
    let cache_stats = app.preview_manager.cache_stats();
    let status_text = format!(
        "GPU: Ready | Memory: 104GB free | Cache: {:.1}MB ({} previews)",
        cache_stats.size_mb(),
        cache_stats.entries
    );
    let status = create_status_bar(&status_text);
    f.render_widget(status, chunks[2]);
}

fn render_body(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
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
    render_prompt_input(f, body_chunks[0], app);

    // Options row (model, LoRA, size)
    render_options_row(f, body_chunks[1]);

    // Main content area
    render_main_content(f, body_chunks[2], app);

    // Recent generations
    render_recent_generations(f, body_chunks[3], app);
}

fn render_prompt_input(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = create_block(" Prompt ");
    let inner = block.inner(area);

    let prompt_text = if app.input_buffer.is_empty() {
        Span::styled("Enter your prompt here...", Theme::muted())
    } else {
        Span::styled(&app.input_buffer, Theme::text())
    };

    let paragraph = Paragraph::new(prompt_text).block(block);

    f.render_widget(paragraph, area);

    // Show cursor if input is active
    if !app.input_buffer.is_empty() || app.current_screen == crate::app::Screen::Generation {
        f.set_cursor(inner.x + app.cursor_pos as u16, inner.y);
    }
}

fn render_options_row(f: &mut Frame, area: ratatui::layout::Rect) {
    let options_text = " Model: [SDXL Base ▼]  LoRA: [None ▼]  Size: [1024x1024]  Steps: [30] ";
    let paragraph = Paragraph::new(options_text).style(Theme::text()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border()),
    );

    f.render_widget(paragraph, area);
}

fn render_main_content(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Options/controls
            Constraint::Percentage(50), // Preview
        ])
        .split(area);

    // Left: Options and controls
    render_controls(f, main_chunks[0], app);

    // Right: Preview area (with Sixel if supported)
    render_preview(f, main_chunks[1], app);
}

fn render_controls(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines = vec![
        Line::from(vec![
            Span::raw("Steps:       "),
            Span::styled("30", Theme::text()),
        ]),
        Line::from(vec![
            Span::raw("CFG Scale:   "),
            Span::styled("7.5", Theme::text()),
        ]),
        Line::from(vec![
            Span::raw("Seed:        "),
            Span::styled("Random", Theme::muted()),
        ]),
        Line::from(vec![
            Span::raw("Batch Size:  "),
            Span::styled("1", Theme::text()),
        ]),
        Line::from(""),
    ];

    // Show active job progress if any
    if let Some(job) = app.active_jobs.first() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Active Job: ", Theme::highlight()),
            Span::styled(&job.prompt[..job.prompt.len().min(30)], Theme::text()),
        ]));

        match &job.status {
            JobStatus::Queued => {
                lines.push(Line::from(Span::styled("Status: Queued", Theme::muted())));
            }
            JobStatus::Running {
                stage,
                progress,
                eta_s,
            } => {
                lines.push(Line::from(vec![
                    Span::raw("Stage: "),
                    Span::styled(stage, Theme::text()),
                ]));
                lines.push(Line::from(vec![
                    Span::raw("Progress: "),
                    Span::styled(format!("{:.0}%", progress * 100.0), Theme::highlight()),
                    Span::raw(format!(" (ETA: {:.1}s)", eta_s)),
                ]));
            }
            JobStatus::Complete { duration_s, .. } => {
                lines.push(Line::from(vec![
                    Span::styled("Complete!", Theme::success()),
                    Span::raw(format!(" ({:.1}s)", duration_s)),
                ]));
            }
            JobStatus::Failed { error } => {
                lines.push(Line::from(Span::styled(
                    format!("Error: {}", error),
                    Theme::error(),
                )));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [G]enerate ", Theme::button()),
        Span::raw("  "),
        Span::styled(" [C]ompare Models ", Theme::button()),
    ]));

    let paragraph = Paragraph::new(lines).block(create_block(" Generation Options "));

    f.render_widget(paragraph, area);
}

fn render_preview(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    // Create title string outside to avoid lifetime issues
    let title_string = if app.debug_mode {
        let tab_titles = vec!["Preview", "Backend Logs"];
        format!(
            " {} [Ctrl+Tab/P/L] ",
            tab_titles
                .iter()
                .enumerate()
                .map(|(i, &t)| if i == app.preview_tab {
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

    let block = create_block(&title_string);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render content based on selected tab
    if app.debug_mode && app.preview_tab == 1 {
        // Render backend logs
        render_backend_logs(f, inner, app);
        return;
    }

    // Otherwise render preview (tab 0 or non-debug mode)
    // Check if we have a current preview
    if let Some(preview_path) = &app.current_preview {
        match app.terminal_capability {
            TerminalCapability::Sixel => {
                // Try to get cached preview
                if let Some(preview_entry) = app.preview_manager.get_preview(preview_path) {
                    // Render Sixel image
                    render_sixel_preview(f, inner, &preview_entry.sixel_data);
                } else {
                    // Request preview if not cached
                    let options = RenderOptions {
                        width: inner.width.saturating_sub(2),
                        height: inner.height.saturating_sub(2),
                        preserve_aspect: true,
                        high_quality: true,
                    };

                    let _ = app
                        .preview_manager
                        .request_preview(preview_path.clone(), options);

                    // Show loading message
                    render_loading_preview(f, inner);
                }
            }
            TerminalCapability::TextOnly => {
                // Show preview info without Sixel
                render_text_preview_info(f, inner, preview_path);
            }
        }
    } else {
        // No preview available
        render_no_preview(f, inner);
    }
}

fn render_sixel_preview(f: &mut Frame, area: ratatui::layout::Rect, sixel_data: &str) {
    // Note: In a real implementation, we'd use crossterm to write raw Sixel data
    // For now, show a placeholder indicating Sixel would render here
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("[Sixel Preview]", Theme::highlight())),
        Line::from(""),
        Line::from(Span::styled("Preview rendering...", Theme::muted())),
        Line::from(""),
        Line::from(format!("Data size: {} bytes", sixel_data.len())),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Theme::muted())
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_loading_preview(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("Loading preview...", Theme::highlight())),
        Line::from(""),
        Line::from(Span::styled("Decoding image", Theme::muted())),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Theme::muted())
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_text_preview_info(f: &mut Frame, area: ratatui::layout::Rect, path: &Path) {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Preview Available", Theme::highlight())),
        Line::from(""),
        Line::from(format!("File: {}", filename)),
        Line::from(""),
        Line::from(Span::styled(
            "Sixel not supported in this terminal",
            Theme::muted(),
        )),
        Line::from(""),
        Line::from("Use kitty, WezTerm, or iTerm2"),
        Line::from("for inline previews"),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Theme::text())
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_no_preview(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from("    [Preview Area]"),
        Line::from(""),
        Line::from("  Image preview will"),
        Line::from("  appear here after"),
        Line::from("  generation"),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Theme::muted())
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_backend_logs(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let lines: Vec<Line> = if app.backend_logs.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled("No backend logs yet", Theme::muted())),
            Line::from(""),
            Line::from(Span::styled(
                "Logs will appear here as the backend processes requests",
                Theme::muted(),
            )),
        ]
    } else {
        // Show last N lines that fit in the area
        let max_lines = area.height.saturating_sub(2) as usize;
        let start_idx = app.backend_logs.len().saturating_sub(max_lines);

        app.backend_logs[start_idx..]
            .iter()
            .map(|log_line| {
                // Color code log levels
                if log_line.contains("ERROR") || log_line.contains("Error") {
                    Line::from(Span::styled(log_line, Theme::error()))
                } else if log_line.contains("WARN") || log_line.contains("Warning") {
                    Line::from(Span::styled(
                        log_line,
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::Yellow),
                    ))
                } else if log_line.contains("INFO") {
                    Line::from(Span::styled(log_line, Theme::text()))
                } else {
                    Line::from(Span::styled(log_line, Theme::muted()))
                }
            })
            .collect()
    };

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

fn render_recent_generations(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let lines = if app.gallery_images.is_empty() {
        vec![Line::from(Span::styled(
            "No recent generations",
            Theme::muted(),
        ))]
    } else {
        let recent: Vec<_> = app
            .gallery_images
            .iter()
            .rev()
            .take(3)
            .map(|path| {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                Line::from(vec![Span::raw("  "), Span::styled(filename, Theme::text())])
            })
            .collect();

        if recent.is_empty() {
            vec![Line::from(Span::styled(
                "No recent generations",
                Theme::muted(),
            ))]
        } else {
            recent
        }
    };

    let paragraph = Paragraph::new(lines).block(create_block(" Recent Generations "));

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[tokio::test]
    async fn test_generation_screen_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();

        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generation_with_active_job() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        app.add_job("test-job".to_string(), "test prompt".to_string());

        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }
}
