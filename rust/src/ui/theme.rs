use ratatui::style::{Color, Modifier, Style};

/// Color theme for the TUI
pub struct Theme;

impl Theme {
    /// Primary color (Cyan) - Active elements, highlights
    pub fn primary() -> Color {
        Color::Cyan
    }

    /// Secondary color (Yellow) - Warnings, notifications
    pub fn secondary() -> Color {
        Color::Yellow
    }

    /// Success color (Green) - Completed jobs, success states
    pub fn success() -> Color {
        Color::Green
    }

    /// Error color (Red) - Errors, failures
    pub fn error() -> Color {
        Color::Red
    }

    /// Muted color (Gray) - Inactive, disabled elements
    pub fn muted() -> Color {
        Color::DarkGray
    }

    /// Default text style
    pub fn text() -> Style {
        Style::default().fg(Color::White)
    }

    /// Header style
    pub fn header() -> Style {
        Style::default()
            .fg(Self::primary())
            .add_modifier(Modifier::BOLD)
    }

    /// Title style
    pub fn title() -> Style {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    /// Highlighted/selected style
    pub fn highlight() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Self::primary())
            .add_modifier(Modifier::BOLD)
    }

    /// Status bar style
    pub fn status_bar() -> Style {
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
    }

    /// Border style
    pub fn border() -> Style {
        Style::default().fg(Self::primary())
    }

    /// Input field style
    pub fn input() -> Style {
        Style::default()
            .fg(Color::White)
            .bg(Color::Black)
    }

    /// Active input field style
    pub fn input_active() -> Style {
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
    }

    /// Button style
    pub fn button() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Self::primary())
    }

    /// Disabled button style
    pub fn button_disabled() -> Style {
        Style::default()
            .fg(Self::muted())
            .bg(Color::Black)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        assert_eq!(Theme::primary(), Color::Cyan);
        assert_eq!(Theme::success(), Color::Green);
        assert_eq!(Theme::error(), Color::Red);
    }

    #[test]
    fn test_styles_return_style() {
        let style = Theme::header();
        assert!(style.fg.is_some());
    }
}
