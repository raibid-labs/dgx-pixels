use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::components::{Job, JobStatus};
use crate::bevy_app::resources::{AppTheme, CurrentScreen, JobTracker, Screen};

/// Render the Queue screen
pub fn render_queue_screen(
    current_screen: Res<CurrentScreen>,
    jobs: Query<&Job>,
    job_tracker: Res<JobTracker>,
    theme: Res<AppTheme>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Queue {
        return;
    }

    ratatui.draw(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(0),      // Content
                Constraint::Length(1),   // Status bar
            ])
            .split(frame.area());

        // Title
        render_title(frame, chunks[0], &theme);

        // Content
        render_content(frame, chunks[1], &jobs, &job_tracker, &theme);

        // Status bar
        render_status_bar(frame, chunks[2], &job_tracker, &theme);
    }).expect("Failed to render queue screen");
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(" Job Queue")
        .style(theme.header())
        .block(Block::default().borders(Borders::ALL).border_style(theme.highlight()));
    frame.render_widget(title, area);
}

fn render_content(
    frame: &mut Frame,
    area: Rect,
    jobs: &Query<&Job>,
    job_tracker: &JobTracker,
    theme: &AppTheme,
) {
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // Active/queued jobs
            Constraint::Percentage(45), // Completed jobs
            Constraint::Length(3),      // Queue stats
        ])
        .margin(1)
        .split(area);

    // Active and queued jobs
    render_active_jobs(frame, content_chunks[0], jobs, theme);

    // Completed jobs
    render_completed_jobs(frame, content_chunks[1], jobs, theme);

    // Queue statistics
    render_queue_stats(frame, content_chunks[2], job_tracker, theme);
}

fn render_active_jobs(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
    let active_jobs: Vec<&Job> = jobs
        .iter()
        .filter(|j| matches!(j.status, JobStatus::Pending | JobStatus::Queued | JobStatus::Generating { .. }))
        .collect();

    let mut lines = vec![Line::from("")];

    if active_jobs.is_empty() {
        lines.push(Line::from(Span::styled("No active jobs", theme.muted())));
    } else {
        for job in active_jobs.iter().take(10) {
            let prompt_preview = if job.prompt.len() > 40 {
                format!("{}...", &job.prompt[..40])
            } else {
                job.prompt.clone()
            };

            let status_line = match &job.status {
                JobStatus::Pending => {
                    Line::from(vec![
                        Span::styled("⏳ ", theme.muted()),
                        Span::styled(&job.id[..12], theme.muted()),
                        Span::raw(" | "),
                        Span::styled(prompt_preview, theme.text()),
                        Span::raw(" | "),
                        Span::styled("Pending", theme.muted()),
                    ])
                }
                JobStatus::Queued => {
                    Line::from(vec![
                        Span::styled("📋 ", theme.text()),
                        Span::styled(&job.id[..12], theme.text()),
                        Span::raw(" | "),
                        Span::styled(prompt_preview, theme.text()),
                        Span::raw(" | "),
                        Span::styled("Queued", theme.text()),
                    ])
                }
                JobStatus::Generating { stage, progress, .. } => {
                    Line::from(vec![
                        Span::styled("⚙ ", theme.highlight()),
                        Span::styled(&job.id[..12], theme.highlight()),
                        Span::raw(" | "),
                        Span::styled(prompt_preview, theme.text()),
                        Span::raw(" | "),
                        Span::styled(format!("{} {:.0}%", stage, progress * 100.0), theme.highlight()),
                    ])
                }
                _ => Line::from(""),
            };

            lines.push(status_line);
        }
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(format!(" Active Jobs ({}) ", active_jobs.len()))
            .borders(Borders::ALL)
            .border_style(theme.highlight()),
    );

    frame.render_widget(paragraph, area);
}

fn render_completed_jobs(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
    let completed_jobs: Vec<&Job> = jobs
        .iter()
        .filter(|j| matches!(j.status, JobStatus::Complete { .. } | JobStatus::Failed { .. }))
        .collect();

    let mut lines = vec![Line::from("")];

    if completed_jobs.is_empty() {
        lines.push(Line::from(Span::styled("No completed jobs", theme.muted())));
    } else {
        for job in completed_jobs.iter().rev().take(10) {
            let prompt_preview = if job.prompt.len() > 40 {
                format!("{}...", &job.prompt[..40])
            } else {
                job.prompt.clone()
            };

            let status_line = match &job.status {
                JobStatus::Complete { duration_s, .. } => {
                    Line::from(vec![
                        Span::styled("✓ ", theme.success()),
                        Span::styled(&job.id[..12], theme.success()),
                        Span::raw(" | "),
                        Span::styled(prompt_preview, theme.text()),
                        Span::raw(" | "),
                        Span::styled(format!("{:.1}s", duration_s), theme.success()),
                    ])
                }
                JobStatus::Failed { error } => {
                    let error_preview = if error.len() > 30 {
                        format!("{}...", &error[..30])
                    } else {
                        error.clone()
                    };
                    Line::from(vec![
                        Span::styled("✗ ", theme.error()),
                        Span::styled(&job.id[..12], theme.error()),
                        Span::raw(" | "),
                        Span::styled(prompt_preview, theme.text()),
                        Span::raw(" | "),
                        Span::styled(error_preview, theme.error()),
                    ])
                }
                _ => Line::from(""),
            };

            lines.push(status_line);
        }
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(format!(" Completed Jobs ({}) ", completed_jobs.len()))
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_queue_stats(frame: &mut Frame, area: Rect, job_tracker: &JobTracker, theme: &AppTheme) {
    // Calculate stats from tracker
    let total = job_tracker.total_submitted;
    let completed = job_tracker.total_completed;
    let failed = job_tracker.total_failed;

    let success_rate = if total > 0 {
        let rate = (completed as f32 / total as f32) * 100.0;
        format!("{:.1}%", rate)
    } else {
        "--".to_string()
    };

    let lines = vec![Line::from(vec![
        Span::raw("Total Submitted: "),
        Span::styled(format!("{}", total), theme.text()),
        Span::raw("  |  Completed: "),
        Span::styled(format!("{}", completed), theme.success()),
        Span::raw("  |  Failed: "),
        Span::styled(format!("{}", failed), theme.error()),
        Span::raw("  |  Success Rate: "),
        Span::styled(success_rate, theme.success()),
    ])];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Queue Statistics ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, job_tracker: &JobTracker, theme: &AppTheme) {
    let status_text = format!(
        "Total: {} | Completed: {} | Failed: {}",
        job_tracker.total_submitted, job_tracker.total_completed, job_tracker.total_failed
    );
    let paragraph = Paragraph::new(status_text).style(theme.status_bar());
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_queue_screen_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Queue));
        app.insert_resource(AppTheme::default());
        app.insert_resource(JobTracker::default());
        app.add_systems(Update, render_queue_screen);
    }
}
