//! # Layout Helpers
//!
//! Reusable layout computation functions for Bevy rendering systems.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create a centered rectangle with given percentage width and height.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a two-column layout with given left column width percentage.
pub fn two_columns(left_percent: u16, area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_percent),
            Constraint::Percentage(100 - left_percent),
        ])
        .split(area)
}

/// Create a vertical split with fixed top height.
pub fn top_and_rest(top_height: u16, area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(top_height), Constraint::Min(0)])
        .split(area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        // Centered rect should be roughly in the middle
        assert!(centered.x > 0 && centered.x < 100);
        assert!(centered.y > 0 && centered.y < 100);
        assert!(centered.width > 0 && centered.width <= 50);
        assert!(centered.height > 0 && centered.height <= 50);
    }

    #[test]
    fn test_two_columns() {
        let area = Rect::new(0, 0, 100, 50);
        let columns = two_columns(30, area);

        assert_eq!(columns.len(), 2);
        // First column should be roughly 30% width
        assert!(columns[0].width >= 29 && columns[0].width <= 31);
        // Second column should be roughly 70% width
        assert!(columns[1].width >= 69 && columns[1].width <= 71);
    }

    #[test]
    fn test_top_and_rest() {
        let area = Rect::new(0, 0, 100, 100);
        let split = top_and_rest(10, area);

        assert_eq!(split.len(), 2);
        assert_eq!(split[0].height, 10);
        // Bottom section gets remaining height (100 - 10 = 90)
        assert_eq!(split[1].height, 90);
    }
}
