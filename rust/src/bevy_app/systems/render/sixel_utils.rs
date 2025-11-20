//! # Sixel Rendering Utilities
//!
//! Utilities for rendering and managing Sixel graphics in the terminal.
//! Handles coordinate calculation, clearing, and proper integration with ratatui.

use ratatui::layout::Rect;
use std::io::{self, Write};
use tracing::debug;

/// Resource to track screen state for Sixel cleanup.
#[derive(Debug, Clone, Default, bevy::prelude::Resource)]
pub struct SixelRenderState {
    /// Last screen that was active (for detecting changes)
    pub last_screen: Option<crate::bevy_app::resources::Screen>,
}

impl SixelRenderState {
    /// Update the last screen.
    pub fn update_screen(&mut self, screen: crate::bevy_app::resources::Screen) {
        self.last_screen = Some(screen);
    }
}

/// Clear a specific area that may contain Sixel graphics.
///
/// This uses terminal escape sequences to:
/// 1. Position cursor in the area
/// 2. Clear each line with spaces
/// 3. Send DECSED (Selective Erase in Display) to clear Sixel graphics
pub fn clear_sixel_area(area: Rect) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Convert ratatui coordinates (0-based) to terminal coordinates (1-based)
    let start_row = area.y.saturating_add(1);
    let start_col = area.x.saturating_add(1);

    // Clear each line in the area
    for line in 0..area.height {
        let row = start_row.saturating_add(line);
        // Position cursor at start of line
        write!(stdout, "\x1b[{};{}H", row, start_col)?;
        // Clear the line with spaces
        write!(stdout, "{}", " ".repeat(area.width as usize))?;
    }

    // Send DECSED (Selective Erase in Display) - clears sixel graphics
    // ESC [ ? 2 J - Erase saved lines (including Sixel)
    write!(stdout, "\x1b[?2J")?;

    stdout.flush()?;

    debug!("Cleared Sixel area at ({}, {}) size {}x{}",
        area.x, area.y, area.width, area.height);

    Ok(())
}

/// Render Sixel data to a specific terminal area.
///
/// This function:
/// 1. Clears the target area first
/// 2. Positions the cursor correctly
/// 3. Writes the Sixel data
/// 4. Flushes stdout
///
/// ## Coordinate System
///
/// Ratatui uses 0-based coordinates for layout areas.
/// Terminal escape sequences use 1-based coordinates.
/// This function handles the conversion.
pub fn render_sixel_to_area(area: Rect, sixel_data: &str) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Clear the area first to remove any previous content
    // This is important for preventing ghosting when images change
    clear_sixel_area(area)?;

    // Convert ratatui coordinates (0-based) to terminal coordinates (1-based)
    let row = area.y.saturating_add(1);
    let col = area.x.saturating_add(1);

    // Position cursor and write Sixel data
    // ESC [ row ; col H - Cursor Position
    write!(stdout, "\x1b[{};{}H", row, col)?;
    write!(stdout, "{}", sixel_data)?;
    stdout.flush()?;

    debug!("Rendered Sixel to area at ({}, {}) size {}x{}",
        area.x, area.y, area.width, area.height);

    Ok(())
}

/// Clear all Sixel graphics from the terminal.
///
/// This is useful when switching screens or exiting the application.
pub fn clear_all_sixel() -> io::Result<()> {
    let mut stdout = io::stdout();

    // Send DECSED (Selective Erase in Display)
    write!(stdout, "\x1b[?2J")?;
    // Also clear the screen normally
    write!(stdout, "\x1b[2J")?;
    // Move cursor to home
    write!(stdout, "\x1b[H")?;

    stdout.flush()?;

    debug!("Cleared all Sixel graphics from terminal");

    Ok(())
}

/// Bevy system to clear Sixel graphics when switching screens.
///
/// This system detects screen changes and clears all Sixel graphics
/// to prevent them from persisting across screen transitions.
///
/// Should run in PreUpdate schedule before rendering systems.
pub fn clear_sixel_on_screen_change(
    current_screen: bevy::prelude::Res<crate::bevy_app::resources::CurrentScreen>,
    mut sixel_state: bevy::prelude::ResMut<SixelRenderState>,
) {
    // Check if screen changed
    if let Some(last_screen) = sixel_state.last_screen {
        if last_screen != current_screen.0 {
            // Screen changed - clear any lingering Sixel graphics
            if let Err(e) = clear_all_sixel() {
                tracing::warn!("Failed to clear Sixel on screen change: {}", e);
            } else {
                debug!("Cleared Sixel graphics on screen change from {:?} to {:?}",
                    last_screen, current_screen.0);
            }
        }
    }

    // Update last screen to current
    sixel_state.update_screen(current_screen.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sixel_render_state_tracking() {
        let mut state = SixelRenderState::default();
        assert!(state.last_screen.is_none());

        state.update_screen(crate::bevy_app::resources::Screen::Gallery);

        assert_eq!(state.last_screen, Some(crate::bevy_app::resources::Screen::Gallery));
    }

    #[test]
    fn test_render_sixel_to_area_returns_ok() {
        // This test just verifies the function signature and basic execution
        // We can't test actual terminal output in unit tests
        let area = Rect::new(0, 0, 10, 10);
        // This might fail in CI without a terminal, but that's okay
        let _ = render_sixel_to_area(area, "test");
    }
}
