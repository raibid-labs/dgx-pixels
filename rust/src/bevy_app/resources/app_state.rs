//! # Application State Resource
//!
//! Global application state including quit flag, redraw requests, and debug mode.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::AppState;
//!
//! fn check_quit(app_state: Res<AppState>) {
//!     if app_state.should_quit {
//!         println!("Application is quitting");
//!     }
//! }
//! ```

use bevy::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

/// Global application state resource.
#[derive(Resource, Debug, Clone)]
pub struct AppState {
    /// Whether the application should quit
    pub should_quit: bool,

    /// Whether the UI needs redrawing
    pub needs_redraw: bool,

    /// Last render time (for FPS tracking)
    pub last_render: Instant,

    /// Frame counter (for FPS calculation)
    pub frame_count: u64,

    /// Debug mode enabled
    pub debug_mode: bool,

    /// Backend log lines (for debug mode)
    pub backend_logs: Vec<String>,

    /// Current preview tab (0=Preview, 1=Logs)
    pub preview_tab: usize,

    /// Currently displayed preview path
    pub current_preview: Option<PathBuf>,

    /// Screen navigation history
    pub screen_history: Vec<super::Screen>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            should_quit: false,
            needs_redraw: true,
            last_render: Instant::now(),
            frame_count: 0,
            debug_mode: false,
            backend_logs: Vec::new(),
            preview_tab: 0,
            current_preview: None,
            screen_history: Vec::new(),
        }
    }
}

impl AppState {
    /// Mark the application for quitting.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Request a UI redraw.
    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Mark frame as rendered and update FPS tracking.
    pub fn mark_rendered(&mut self) {
        self.frame_count += 1;
        self.last_render = Instant::now();
        self.needs_redraw = false;
    }

    /// Get current FPS.
    pub fn current_fps(&self) -> f64 {
        let elapsed = self.last_render.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frame_count as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Switch to next preview tab (if debug mode enabled).
    pub fn next_preview_tab(&mut self) {
        if self.debug_mode {
            self.preview_tab = (self.preview_tab + 1) % 2;
            self.needs_redraw = true;
        }
    }

    /// Set specific preview tab (if debug mode enabled).
    pub fn set_preview_tab(&mut self, tab: usize) {
        if self.debug_mode && tab < 2 {
            self.preview_tab = tab;
            self.needs_redraw = true;
        }
    }

    /// Add backend log line (truncates to last 500 lines).
    pub fn add_backend_log(&mut self, line: String) {
        self.backend_logs.push(line);
        if self.backend_logs.len() > 500 {
            self.backend_logs.remove(0);
        }
        self.needs_redraw = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_app_state() {
        let state = AppState::default();
        assert!(!state.should_quit);
        assert!(state.needs_redraw);
        assert_eq!(state.frame_count, 0);
        assert!(!state.debug_mode);
        assert_eq!(state.preview_tab, 0);
    }

    #[test]
    fn test_quit() {
        let mut state = AppState::default();
        assert!(!state.should_quit);
        state.quit();
        assert!(state.should_quit);
    }

    #[test]
    fn test_request_redraw() {
        let mut state = AppState::default();
        state.needs_redraw = false;
        state.request_redraw();
        assert!(state.needs_redraw);
    }

    #[test]
    fn test_mark_rendered() {
        let mut state = AppState::default();
        state.mark_rendered();
        assert_eq!(state.frame_count, 1);
        assert!(!state.needs_redraw);
    }

    #[test]
    fn test_preview_tab_switching() {
        let mut state = AppState::default();
        state.debug_mode = true;

        state.next_preview_tab();
        assert_eq!(state.preview_tab, 1);

        state.next_preview_tab();
        assert_eq!(state.preview_tab, 0); // Wraps around
    }

    #[test]
    fn test_preview_tab_requires_debug_mode() {
        let mut state = AppState::default();
        state.debug_mode = false;

        state.next_preview_tab();
        assert_eq!(state.preview_tab, 0); // Should not change
    }

    #[test]
    fn test_backend_log_truncation() {
        let mut state = AppState::default();

        // Add 600 log lines
        for i in 0..600 {
            state.add_backend_log(format!("Log line {}", i));
        }

        // Should be truncated to 500
        assert_eq!(state.backend_logs.len(), 500);
        // Should have kept the most recent ones
        assert_eq!(state.backend_logs[0], "Log line 100");
        assert_eq!(state.backend_logs[499], "Log line 599");
    }
}
