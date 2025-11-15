//! # Theme Resource
//!
//! Application theme as a Bevy resource for consistent styling across all render systems.

use bevy::prelude::*;
use ratatui::style::{Color as RatatuiColor, Modifier, Style};

/// Application theme resource providing consistent colors and styles.
#[derive(Resource, Clone, Debug)]
pub struct AppTheme {
    pub colors: ThemeColors,
}

/// Theme color definitions matching the classic ui/theme.rs design.
#[derive(Clone, Debug)]
pub struct ThemeColors {
    /// Primary color (Cyan) - Active elements, highlights
    pub primary: RatatuiColor,
    /// Secondary color (Yellow) - Warnings, notifications
    pub secondary: RatatuiColor,
    /// Success color (Green) - Completed jobs, success states
    pub success: RatatuiColor,
    /// Error color (Red) - Errors, failures
    pub error: RatatuiColor,
    /// Muted color (Gray) - Inactive, disabled elements
    pub muted: RatatuiColor,
    /// Default text color (White)
    pub text: RatatuiColor,
    /// Background color (Black)
    pub background: RatatuiColor,
    /// Status bar background (DarkGray)
    pub status_bg: RatatuiColor,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            colors: ThemeColors {
                primary: RatatuiColor::Cyan,
                secondary: RatatuiColor::Yellow,
                success: RatatuiColor::Green,
                error: RatatuiColor::Red,
                muted: RatatuiColor::DarkGray,
                text: RatatuiColor::White,
                background: RatatuiColor::Black,
                status_bg: RatatuiColor::DarkGray,
            },
        }
    }
}

impl AppTheme {
    /// Default text style
    pub fn text(&self) -> Style {
        Style::default().fg(self.colors.text)
    }

    /// Header style (bold cyan)
    pub fn header(&self) -> Style {
        Style::default()
            .fg(self.colors.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Title style (bold white)
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.colors.text)
            .add_modifier(Modifier::BOLD)
    }

    /// Highlighted/selected style (black on cyan)
    pub fn highlight(&self) -> Style {
        Style::default()
            .fg(RatatuiColor::Black)
            .bg(self.colors.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Status bar style (white on dark gray)
    pub fn status_bar(&self) -> Style {
        Style::default()
            .fg(self.colors.text)
            .bg(self.colors.status_bg)
    }

    /// Border style (cyan)
    pub fn border(&self) -> Style {
        Style::default().fg(self.colors.primary)
    }

    /// Input field style (white on black)
    pub fn input(&self) -> Style {
        Style::default()
            .fg(self.colors.text)
            .bg(self.colors.background)
    }

    /// Active input field style (white on dark gray)
    pub fn input_active(&self) -> Style {
        Style::default()
            .fg(self.colors.text)
            .bg(self.colors.status_bg)
    }

    /// Button style (black on cyan)
    pub fn button(&self) -> Style {
        Style::default()
            .fg(RatatuiColor::Black)
            .bg(self.colors.primary)
    }

    /// Disabled button style (muted on black)
    pub fn button_disabled(&self) -> Style {
        Style::default()
            .fg(self.colors.muted)
            .bg(self.colors.background)
    }

    /// Success message style (green)
    pub fn success(&self) -> Style {
        Style::default().fg(self.colors.success)
    }

    /// Error message style (bold red)
    pub fn error(&self) -> Style {
        Style::default()
            .fg(self.colors.error)
            .add_modifier(Modifier::BOLD)
    }

    /// Warning message style (yellow)
    pub fn warning(&self) -> Style {
        Style::default().fg(self.colors.secondary)
    }

    /// Muted/dimmed text style (dark gray, italic)
    pub fn muted(&self) -> Style {
        Style::default()
            .fg(self.colors.muted)
            .add_modifier(Modifier::ITALIC)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = AppTheme::default();

        assert_eq!(theme.colors.primary, RatatuiColor::Cyan);
        assert_eq!(theme.colors.success, RatatuiColor::Green);
        assert_eq!(theme.colors.error, RatatuiColor::Red);
        assert_eq!(theme.colors.text, RatatuiColor::White);
    }

    #[test]
    fn test_theme_styles() {
        let theme = AppTheme::default();

        // Test header style
        let header = theme.header();
        assert_eq!(header.fg, Some(RatatuiColor::Cyan));

        // Test highlight style
        let highlight = theme.highlight();
        assert_eq!(highlight.fg, Some(RatatuiColor::Black));
        assert_eq!(highlight.bg, Some(RatatuiColor::Cyan));

        // Test error style
        let error = theme.error();
        assert_eq!(error.fg, Some(RatatuiColor::Red));
    }

    #[test]
    fn test_status_bar_style() {
        let theme = AppTheme::default();
        let status = theme.status_bar();

        assert_eq!(status.fg, Some(RatatuiColor::White));
        assert_eq!(status.bg, Some(RatatuiColor::DarkGray));
    }

    #[test]
    fn test_button_styles() {
        let theme = AppTheme::default();

        let button = theme.button();
        assert_eq!(button.fg, Some(RatatuiColor::Black));
        assert_eq!(button.bg, Some(RatatuiColor::Cyan));

        let disabled = theme.button_disabled();
        assert_eq!(disabled.fg, Some(RatatuiColor::DarkGray));
    }
}
