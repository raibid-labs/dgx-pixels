use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::bevy_app::components::Job;
use crate::bevy_app::resources::{AppTheme, CurrentScreen, JobTracker, Screen};

/// Render the Monitor screen
pub fn render_monitor_screen(
    current_screen: Res<CurrentScreen>,
    jobs: Query<&Job>,
    job_tracker: Res<JobTracker>,
    theme: Res<AppTheme>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Monitor {
        return;
    }

    ratatui
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Content
                    Constraint::Length(1), // Status bar
                ])
                .split(frame.area());

            // Title
            render_title(frame, chunks[0], &theme);

            // Content
            render_content(frame, chunks[1], &jobs, &job_tracker, &theme);

            // Status bar
            render_status_bar(frame, chunks[2], &job_tracker, &theme);
        })
        .expect("Failed to render monitor screen");
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(" System Monitor")
        .style(theme.header())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.highlight()),
        );
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
            Constraint::Percentage(40), // Job statistics
            Constraint::Percentage(30), // System metrics
            Constraint::Percentage(30), // Active jobs
        ])
        .margin(1)
        .split(area);

    // Job statistics
    render_job_statistics(frame, content_chunks[0], job_tracker, theme);

    // System metrics
    render_system_metrics(frame, content_chunks[1], jobs, theme);

    // Active jobs
    render_active_jobs(frame, content_chunks[2], jobs, theme);
}

fn render_job_statistics(
    frame: &mut Frame,
    area: Rect,
    job_tracker: &JobTracker,
    theme: &AppTheme,
) {
    let success_rate = job_tracker.success_rate();
    let active = job_tracker.active_jobs();

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Submitted:  ", theme.muted()),
            Span::styled(format!("{}", job_tracker.total_submitted), theme.text()),
        ]),
        Line::from(vec![
            Span::styled("Completed:        ", theme.muted()),
            Span::styled(format!("{}", job_tracker.total_completed), theme.success()),
        ]),
        Line::from(vec![
            Span::styled("Failed:           ", theme.muted()),
            Span::styled(format!("{}", job_tracker.total_failed), theme.error()),
        ]),
        Line::from(vec![
            Span::styled("Active:           ", theme.muted()),
            Span::styled(format!("{}", active), theme.highlight()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Success Rate:     ", theme.muted()),
            Span::styled(format!("{:.1}%", success_rate), theme.success()),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Job Statistics ")
            .borders(Borders::ALL)
            .border_style(theme.highlight()),
    );

    frame.render_widget(paragraph, area);
}

fn render_system_metrics(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
    // Calculate job counts for queue pressure metric
    let total_jobs = jobs.iter().count();
    let queue_pressure = if total_jobs > 0 {
        (total_jobs as f64 / 100.0).min(1.0)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Queue Pressure ")
                .borders(Borders::ALL)
                .border_style(theme.text()),
        )
        .gauge_style(theme.highlight())
        .ratio(queue_pressure);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Queue pressure gauge
            Constraint::Min(0),    // Other metrics
        ])
        .split(area);

    frame.render_widget(gauge, inner_chunks[0]);

    // System info (placeholder for future GPU metrics)
    let sys_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("GPU Utilization:  ", theme.muted()),
            Span::styled("[WS-XX: Not implemented]", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("Memory Usage:     ", theme.muted()),
            Span::styled("[WS-XX: Not implemented]", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("Temperature:      ", theme.muted()),
            Span::styled("[WS-XX: Not implemented]", theme.muted()),
        ]),
    ];

    let paragraph = Paragraph::new(sys_lines).block(
        Block::default()
            .title(" System Metrics ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, inner_chunks[1]);
}

fn render_active_jobs(frame: &mut Frame, area: Rect, jobs: &Query<&Job>, theme: &AppTheme) {
    let active_jobs: Vec<&Job> = jobs
        .iter()
        .filter(|j| {
            matches!(
                j.status,
                crate::bevy_app::components::JobStatus::Pending
                    | crate::bevy_app::components::JobStatus::Queued
                    | crate::bevy_app::components::JobStatus::Generating { .. }
            )
        })
        .take(5) // Show only first 5
        .collect();

    let mut lines = vec![Line::from("")];

    if active_jobs.is_empty() {
        lines.push(Line::from(Span::styled("No active jobs", theme.muted())));
    } else {
        for job in active_jobs {
            let prompt_preview = if job.prompt.len() > 30 {
                format!("{}...", &job.prompt[..30])
            } else {
                job.prompt.clone()
            };

            let status_text = match &job.status {
                crate::bevy_app::components::JobStatus::Pending => "Pending",
                crate::bevy_app::components::JobStatus::Queued => "Queued",
                crate::bevy_app::components::JobStatus::Generating { stage, .. } => stage.as_str(),
                _ => "Unknown",
            };

            lines.push(Line::from(vec![
                Span::styled("• ", theme.highlight()),
                Span::styled(format!("{:12} ", &job.id[..12]), theme.muted()),
                Span::styled(format!("{:15} ", status_text), theme.text()),
                Span::styled(prompt_preview, theme.text()),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(format!(" Active Jobs ({}) ", active_jobs.len()))
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, job_tracker: &JobTracker, theme: &AppTheme) {
    let status_text = format!(
        "Active: {} | Completed: {} | Failed: {} | Success Rate: {:.1}%",
        job_tracker.active_jobs(),
        job_tracker.total_completed,
        job_tracker.total_failed,
        job_tracker.success_rate()
    );
    let paragraph = Paragraph::new(status_text).style(theme.status_bar());
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_monitor_screen_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Monitor));
        app.insert_resource(AppTheme::default());
        app.insert_resource(JobTracker::default());
        app.add_systems(Update, render_monitor_screen);
    }
}
