use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::bevy_app::components::{Job, JobStatus};
use crate::bevy_app::resources::{AppTheme, CurrentScreen, JobTracker, QueueState, Screen};

/// Render the Queue screen with scrollable job list and keyboard navigation
pub fn render_queue_screen(
    current_screen: Res<CurrentScreen>,
    jobs: Query<&Job>,
    job_tracker: Res<JobTracker>,
    mut queue_state: ResMut<QueueState>,
    theme: Res<AppTheme>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Queue {
        return;
    }

    // Collect all jobs into a sorted vector
    let mut all_jobs: Vec<&Job> = jobs.iter().collect();
    all_jobs.sort_by_key(|j| j.submitted_at);

    // Update queue state with current job count
    queue_state.update_total(all_jobs.len());

    ratatui
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Job list
                    Constraint::Length(3), // Statistics
                    Constraint::Length(1), // Status bar
                ])
                .split(frame.area());

            // Title
            render_title(frame, chunks[0], all_jobs.len(), &theme);

            // Job list with selection highlighting
            render_job_list(frame, chunks[1], &all_jobs, queue_state.selected, &theme);

            // Queue statistics
            render_queue_stats(frame, chunks[2], &job_tracker, &theme);

            // Status bar
            render_status_bar(frame, chunks[3], &job_tracker, &theme);
        })
        .expect("Failed to render queue screen");
}

fn render_title(frame: &mut Frame, area: Rect, job_count: usize, theme: &AppTheme) {
    let title = Paragraph::new(format!(" Job Queue ({}) ", job_count))
        .style(theme.header())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.highlight()),
        );
    frame.render_widget(title, area);
}

fn render_job_list(
    frame: &mut Frame,
    area: Rect,
    jobs: &[&Job],
    selected: usize,
    theme: &AppTheme,
) {
    if jobs.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No jobs yet. Press 'g' to create a new generation job.",
                theme.muted(),
            )),
        ])
        .block(
            Block::default()
                .title(" Jobs ")
                .borders(Borders::ALL)
                .border_style(theme.text()),
        );
        frame.render_widget(empty_msg, area);
        return;
    }

    // Create list items for each job
    let items: Vec<ListItem> = jobs
        .iter()
        .enumerate()
        .map(|(idx, job)| {
            // Truncate prompt for display
            let prompt_preview = if job.prompt.len() > 50 {
                format!("{}...", &job.prompt[..47])
            } else {
                job.prompt.clone()
            };

            // Format job ID (first 8 chars)
            let job_id = if job.id.len() >= 8 {
                &job.id[..8]
            } else {
                &job.id
            };

            // Determine status icon, text and style
            let (status_icon, status_text, status_style) = match &job.status {
                JobStatus::Pending => ("⏳", "Pending".to_string(), theme.muted()),
                JobStatus::Queued => ("📋", "Queued".to_string(), theme.text()),
                JobStatus::Generating { stage, progress, .. } => {
                    ("🔄", format!("{} {:.0}%", stage, progress * 100.0), theme.highlight())
                }
                JobStatus::Complete { duration_s, .. } => {
                    ("✓", format!("Complete {:.1}s", duration_s), theme.success())
                }
                JobStatus::Failed { error } => {
                    let error_preview = if error.len() > 20 {
                        format!("{}...", &error[..17])
                    } else {
                        error.clone()
                    };
                    ("✗", error_preview, theme.error())
                }
                JobStatus::Cancelled => ("🚫", "Cancelled".to_string(), theme.muted()),
            };

            // Format elapsed time
            let elapsed = job.elapsed();
            let elapsed_str = if elapsed.as_secs() < 60 {
                format!("{}s ago", elapsed.as_secs())
            } else if elapsed.as_secs() < 3600 {
                format!("{}m ago", elapsed.as_secs() / 60)
            } else {
                format!("{}h ago", elapsed.as_secs() / 3600)
            };

            // Create line with proper spacing
            let line = Line::from(vec![
                Span::styled(format!("{} ", status_icon), status_style),
                Span::styled(format!("{:<8}", job_id), theme.muted()),
                Span::raw(" │ "),
                Span::styled(format!("{:<50}", prompt_preview), theme.text()),
                Span::raw(" │ "),
                Span::styled(format!("{:<15}", status_text), status_style),
                Span::raw(" │ "),
                Span::styled(elapsed_str, theme.muted()),
            ]);

            // Highlight selected item
            let item = if idx == selected {
                ListItem::new(line).style(theme.text().add_modifier(Modifier::REVERSED))
            } else {
                ListItem::new(line)
            };

            item
        })
        .collect();

    // Create list widget
    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Jobs (↑/↓ to navigate, {} selected) ",
                selected + 1
            ))
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(list, area);
}

fn render_queue_stats(frame: &mut Frame, area: Rect, job_tracker: &JobTracker, theme: &AppTheme) {
    // Calculate stats from tracker
    let total = job_tracker.total_submitted;
    let completed = job_tracker.total_completed;
    let failed = job_tracker.total_failed;
    let active = job_tracker.active_jobs();

    let success_rate = if total > 0 {
        let rate = (completed as f32 / total as f32) * 100.0;
        format!("{:.1}%", rate)
    } else {
        "--".to_string()
    };

    let lines = vec![Line::from(vec![
        Span::raw("Total: "),
        Span::styled(format!("{}", total), theme.text()),
        Span::raw("  │  Active: "),
        Span::styled(format!("{}", active), theme.highlight()),
        Span::raw("  │  Completed: "),
        Span::styled(format!("{}", completed), theme.success()),
        Span::raw("  │  Failed: "),
        Span::styled(format!("{}", failed), theme.error()),
        Span::raw("  │  Success Rate: "),
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
        "Total: {} | Active: {} | Completed: {} | Failed: {} | [↑/↓] Navigate [Home/End] Jump [c] Cancel",
        job_tracker.total_submitted,
        job_tracker.active_jobs(),
        job_tracker.total_completed,
        job_tracker.total_failed
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
        app.insert_resource(QueueState::default());
        app.add_systems(Update, render_queue_screen);
    }
}
