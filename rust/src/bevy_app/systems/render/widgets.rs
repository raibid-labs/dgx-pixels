//! # Widget Helpers
//!
//! Reusable widget construction functions for Bevy rendering systems.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType},
};

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
}
