use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Widget},
};
use std::io::{self, Write};
use tracing::debug;

pub struct SixelImage<'a> {
    sixel_data: &'a str,
}

impl<'a> SixelImage<'a> {
    pub fn new(sixel_data: &'a str) -> Self {
        Self { sixel_data }
    }
}

impl<'a> Widget for SixelImage<'a> {
    fn render(self, area: Rect, _buf: &mut Buffer) {
        debug!("SixelImage widget rendering. Area: {:?}", area);
        debug!("Sixel data length: {}", self.sixel_data.len());

        let mut stdout = io::stdout();

        // Position cursor at the top-left of the render area
        // ANSI escape: ESC[{row};{col}H (rows and columns are 1-indexed)
        let row = area.y + 1; // Convert to 1-indexed
        let col = area.x + 1; // Convert to 1-indexed

        // Clear the area first by writing spaces
        // This ensures old Sixel images don't persist when navigating
        for line in 0..area.height {
            let line_row = row + line as u16;
            let _ = write!(stdout, "\x1b[{};{}H", line_row, col);
            let _ = write!(stdout, "{}", " ".repeat(area.width as usize));
        }

        // Now write cursor positioning + sixel data
        let _ = write!(stdout, "\x1b[{};{}H{}", row, col, self.sixel_data);
        let _ = stdout.flush();

        debug!("Sixel data written to stdout at position ({}, {}) and flushed.", row, col);
    }
}
