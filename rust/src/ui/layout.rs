use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::rc::Rc;

/// Create the main 3-section layout (header, body, footer)
pub fn create_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Body
            Constraint::Length(3), // Footer/Status bar
        ])
        .split(area)
}

/// Create a two-column layout
#[allow(dead_code)]
pub fn two_columns(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
}

/// Create a three-column layout
#[allow(dead_code)]
pub fn three_columns(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area)
}

/// Create a centered popup area
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_layout() {
        let area = Rect::new(0, 0, 100, 50);
        let chunks = create_layout(area);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].height, 3); // Header
        assert_eq!(chunks[2].height, 3); // Footer
    }

    #[test]
    fn test_two_columns() {
        let area = Rect::new(0, 0, 100, 50);
        let chunks = two_columns(area);
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let popup = centered_rect(50, 50, area);
        assert!(popup.width <= area.width);
        assert!(popup.height <= area.height);
    }
}
