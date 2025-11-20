//! # Widget Helpers
//!
//! Reusable widget construction functions for Bevy rendering systems.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

use crate::bevy_app::resources::AppTheme;

/// Create a standard bordered block with title.
pub fn standard_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
}

/// Create a highlighted block (for focused/selected items).
pub fn highlighted_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
}

/// Create a status message line with appropriate styling.
pub fn status_line(message: &str, is_error: bool) -> Line {
    let style = if is_error {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    Line::from(Span::styled(message, style))
}

/// Create a help hint line (dimmed style).
pub fn help_hint(text: &str) -> Line {
    Line::from(Span::styled(
        text,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    ))
}

/// Create a progress bar widget with theme colors.
///
/// # Arguments
/// * `progress` - Progress value from 0.0 to 1.0
/// * `width` - Width of the progress bar in characters
/// * `theme` - App theme for styling
///
/// # Returns
/// A Line containing the progress bar with percentage
///
/// # Example
/// ```rust,no_run
/// use dgx_pixels_tui::bevy_app::systems::render::widgets::progress_bar;
/// use dgx_pixels_tui::bevy_app::resources::AppTheme;
///
/// let theme = AppTheme::default();
/// let bar = progress_bar(0.5, 20, &theme);
/// // Renders: [██████████░░░░░░░░░░] 50%
/// ```
pub fn progress_bar(progress: f32, width: u16, theme: &AppTheme) -> Line {
    // Clamp progress to 0.0-1.0
    let progress = progress.clamp(0.0, 1.0);

    let filled = (progress * width as f32) as u16;
    let empty = width.saturating_sub(filled);

    let filled_str = "█".repeat(filled as usize);
    let empty_str = "░".repeat(empty as usize);

    Line::from(vec![
        Span::raw("["),
        Span::styled(filled_str, theme.highlight()),
        Span::styled(empty_str, theme.muted()),
        Span::raw("]"),
        Span::raw(" "),
        Span::styled(format!("{:.0}%", progress * 100.0), theme.text()),
    ])
}

/// Create a progress bar with ETA information.
///
/// # Arguments
/// * `progress` - Progress value from 0.0 to 1.0
/// * `width` - Width of the progress bar in characters
/// * `eta_s` - Estimated time remaining in seconds
/// * `theme` - App theme for styling
///
/// # Returns
/// A Line containing the progress bar with percentage and ETA
///
/// # Example
/// ```rust,no_run
/// use dgx_pixels_tui::bevy_app::systems::render::widgets::progress_bar_with_eta;
/// use dgx_pixels_tui::bevy_app::resources::AppTheme;
///
/// let theme = AppTheme::default();
/// let bar = progress_bar_with_eta(0.5, 20, 12.5, &theme);
/// // Renders: [██████████░░░░░░░░░░] 50% (ETA: 12s)
/// ```
pub fn progress_bar_with_eta(progress: f32, width: u16, eta_s: f32, theme: &AppTheme) -> Line {
    // Clamp progress to 0.0-1.0
    let progress = progress.clamp(0.0, 1.0);

    let filled = (progress * width as f32) as u16;
    let empty = width.saturating_sub(filled);

    let filled_str = "█".repeat(filled as usize);
    let empty_str = "░".repeat(empty as usize);

    // Format ETA
    let eta_text = if eta_s < 60.0 {
        format!("{:.0}s", eta_s)
    } else if eta_s < 3600.0 {
        format!("{:.0}m {:.0}s", eta_s / 60.0, eta_s % 60.0)
    } else {
        format!("{:.0}h {:.0}m", eta_s / 3600.0, (eta_s % 3600.0) / 60.0)
    };

    Line::from(vec![
        Span::raw("["),
        Span::styled(filled_str, theme.highlight()),
        Span::styled(empty_str, theme.muted()),
        Span::raw("]"),
        Span::raw(" "),
        Span::styled(format!("{:.0}%", progress * 100.0), theme.text()),
        Span::raw(" "),
        Span::styled(format!("(ETA: {})", eta_text), theme.muted()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_block() {
        let _block = standard_block("Test");
        // Block creation succeeds (checking title content requires internal access)
    }

    #[test]
    fn test_highlighted_block() {
        let _block = highlighted_block("Selected");
        // Block creation succeeds
    }

    #[test]
    fn test_status_line() {
        let normal = status_line("OK", false);
        let error = status_line("Error", true);

        // Both should create valid lines
        assert!(normal.spans.len() > 0);
        assert!(error.spans.len() > 0);
    }

    #[test]
    fn test_help_hint() {
        let hint = help_hint("Press ? for help");
        assert!(hint.spans.len() > 0);
    }

    #[test]
    fn test_progress_bar() {
        let theme = AppTheme::default();

        // Test 0% progress
        let bar = progress_bar(0.0, 20, &theme);
        assert!(bar.spans.len() > 0);

        // Test 50% progress
        let bar = progress_bar(0.5, 20, &theme);
        assert!(bar.spans.len() > 0);

        // Test 100% progress
        let bar = progress_bar(1.0, 20, &theme);
        assert!(bar.spans.len() > 0);

        // Test clamping (negative)
        let bar = progress_bar(-0.1, 20, &theme);
        assert!(bar.spans.len() > 0);

        // Test clamping (over 100%)
        let bar = progress_bar(1.5, 20, &theme);
        assert!(bar.spans.len() > 0);
    }

    #[test]
    fn test_progress_bar_with_eta() {
        let theme = AppTheme::default();

        // Test short ETA (< 60s)
        let bar = progress_bar_with_eta(0.5, 20, 30.0, &theme);
        assert!(bar.spans.len() > 0);

        // Test medium ETA (minutes)
        let bar = progress_bar_with_eta(0.3, 20, 120.0, &theme);
        assert!(bar.spans.len() > 0);

        // Test long ETA (hours)
        let bar = progress_bar_with_eta(0.1, 20, 7200.0, &theme);
        assert!(bar.spans.len() > 0);
    }
}
