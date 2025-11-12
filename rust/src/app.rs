use crate::sixel::{PreviewManager, TerminalCapability};
use crate::ui::screens::comparison::ComparisonState;
use std::path::PathBuf;
use std::time::Instant;

/// Represents the current screen in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Generation,
    Comparison, // NEW: Side-by-side model comparison
    Queue,
    Gallery,
    Models,
    Monitor,
    Settings,
    Help,
}

/// Job status tracking
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum JobStatus {
    Queued,
    Running {
        stage: String,
        progress: f32,
        eta_s: f32,
    },
    Complete {
        image_path: PathBuf,
        duration_s: f32,
    },
    Failed {
        error: String,
    },
}

/// Active job information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActiveJob {
    pub job_id: String,
    pub prompt: String,
    pub status: JobStatus,
    pub preview_path: Option<PathBuf>,
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Current active screen
    pub current_screen: Screen,

    /// Navigation history (for back button)
    pub screen_history: Vec<Screen>,

    /// Whether the app should quit
    pub should_quit: bool,

    /// Current input buffer (for text entry)
    pub input_buffer: String,

    /// Cursor position in input buffer
    pub cursor_pos: usize,

    /// Last render time (for FPS tracking)
    pub last_render: Instant,

    /// Frame counter (for FPS calculation)
    pub frame_count: u64,

    /// Whether the UI needs redraw
    pub needs_redraw: bool,

    /// Preview manager for Sixel rendering
    pub preview_manager: PreviewManager,

    /// Terminal capability (Sixel support)
    pub terminal_capability: TerminalCapability,

    /// Active jobs being processed
    pub active_jobs: Vec<ActiveJob>,

    /// Currently displayed preview path
    pub current_preview: Option<PathBuf>,

    /// Gallery images
    pub gallery_images: Vec<PathBuf>,

    /// Selected gallery image index
    pub selected_gallery_index: usize,

    /// Comparison screen state (NEW)
    pub comparison_state: ComparisonState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new App instance
    pub fn new() -> Self {
        use crate::sixel::detect_sixel_support;

        let terminal_capability = detect_sixel_support();

        Self {
            current_screen: Screen::Generation,
            screen_history: Vec::new(),
            should_quit: false,
            input_buffer: String::new(),
            cursor_pos: 0,
            last_render: Instant::now(),
            frame_count: 0,
            needs_redraw: true,
            preview_manager: PreviewManager::new(),
            terminal_capability,
            active_jobs: Vec::new(),
            current_preview: None,
            gallery_images: Vec::new(),
            selected_gallery_index: 0,
            comparison_state: ComparisonState::new(),
        }
    }

    /// Navigate to a new screen
    pub fn navigate_to(&mut self, screen: Screen) {
        if self.current_screen != screen {
            self.screen_history.push(self.current_screen);
            self.current_screen = screen;
            self.needs_redraw = true;
        }
    }

    /// Navigate back to previous screen
    pub fn navigate_back(&mut self) {
        if let Some(previous) = self.screen_history.pop() {
            self.current_screen = previous;
            self.needs_redraw = true;
        }
    }

    /// Mark the app for quitting
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Add character to input buffer
    pub fn input_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.needs_redraw = true;
    }

    /// Delete character before cursor
    pub fn input_backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.input_buffer.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.needs_redraw = true;
        }
    }

    /// Clear input buffer
    #[allow(dead_code)]
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_pos = 0;
        self.needs_redraw = true;
    }

    /// Get current FPS
    #[allow(dead_code)]
    pub fn current_fps(&self) -> f64 {
        let elapsed = self.last_render.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frame_count as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Mark frame rendered
    pub fn mark_rendered(&mut self) {
        self.frame_count += 1;
        self.last_render = Instant::now();
        self.needs_redraw = false;
    }

    /// Check if redraw is needed
    #[allow(dead_code)]
    pub fn should_redraw(&self) -> bool {
        self.needs_redraw
    }

    /// Update job status
    #[allow(dead_code)]
    pub fn update_job_status(&mut self, job_id: &str, status: JobStatus) {
        if let Some(job) = self.active_jobs.iter_mut().find(|j| j.job_id == job_id) {
            job.status = status;
            self.needs_redraw = true;
        }
    }

    /// Add new job
    #[allow(dead_code)]
    pub fn add_job(&mut self, job_id: String, prompt: String) {
        self.active_jobs.push(ActiveJob {
            job_id,
            prompt,
            status: JobStatus::Queued,
            preview_path: None,
        });
        self.needs_redraw = true;
    }

    /// Set preview for job
    #[allow(dead_code)]
    pub fn set_job_preview(&mut self, job_id: &str, preview_path: PathBuf) {
        if let Some(job) = self.active_jobs.iter_mut().find(|j| j.job_id == job_id) {
            job.preview_path = Some(preview_path.clone());
            self.current_preview = Some(preview_path);
            self.needs_redraw = true;
        }
    }

    /// Remove completed job
    #[allow(dead_code)]
    pub fn remove_job(&mut self, job_id: &str) {
        self.active_jobs.retain(|j| j.job_id != job_id);
        self.needs_redraw = true;
    }

    /// Add image to gallery
    #[allow(dead_code)]
    pub fn add_to_gallery(&mut self, path: PathBuf) {
        if !self.gallery_images.contains(&path) {
            self.gallery_images.push(path);
            self.needs_redraw = true;
        }
    }

    /// Select next gallery image
    pub fn gallery_next(&mut self) {
        if !self.gallery_images.is_empty() {
            self.selected_gallery_index =
                (self.selected_gallery_index + 1) % self.gallery_images.len();
            self.needs_redraw = true;
        }
    }

    /// Select previous gallery image
    pub fn gallery_prev(&mut self) {
        if !self.gallery_images.is_empty() {
            self.selected_gallery_index = if self.selected_gallery_index == 0 {
                self.gallery_images.len() - 1
            } else {
                self.selected_gallery_index - 1
            };
            self.needs_redraw = true;
        }
    }

    /// Get selected gallery image
    pub fn selected_gallery_image(&self) -> Option<&PathBuf> {
        self.gallery_images.get(self.selected_gallery_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_app_starts_on_generation_screen() {
        let app = App::new();
        assert_eq!(app.current_screen, Screen::Generation);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn test_navigation_forward() {
        let mut app = App::new();
        app.navigate_to(Screen::Queue);
        assert_eq!(app.current_screen, Screen::Queue);
        assert_eq!(app.screen_history.len(), 1);
        assert_eq!(app.screen_history[0], Screen::Generation);
    }

    #[tokio::test]
    async fn test_navigation_back() {
        let mut app = App::new();
        app.navigate_to(Screen::Queue);
        app.navigate_to(Screen::Gallery);
        app.navigate_back();
        assert_eq!(app.current_screen, Screen::Queue);
        app.navigate_back();
        assert_eq!(app.current_screen, Screen::Generation);
    }

    #[tokio::test]
    async fn test_quit() {
        let mut app = App::new();
        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_input_char() {
        let mut app = App::new();
        app.input_char('h');
        app.input_char('i');
        assert_eq!(app.input_buffer, "hi");
        assert_eq!(app.cursor_pos, 2);
    }

    #[tokio::test]
    async fn test_input_backspace() {
        let mut app = App::new();
        app.input_char('h');
        app.input_char('i');
        app.input_backspace();
        assert_eq!(app.input_buffer, "h");
        assert_eq!(app.cursor_pos, 1);
    }

    #[tokio::test]
    async fn test_clear_input() {
        let mut app = App::new();
        app.input_char('t');
        app.input_char('e');
        app.input_char('s');
        app.input_char('t');
        app.clear_input();
        assert_eq!(app.input_buffer, "");
        assert_eq!(app.cursor_pos, 0);
    }

    #[tokio::test]
    async fn test_navigation_same_screen_no_history() {
        let mut app = App::new();
        app.navigate_to(Screen::Generation);
        assert_eq!(app.screen_history.len(), 0);
    }

    #[tokio::test]
    async fn test_add_job() {
        let mut app = App::new();
        app.add_job("job-001".to_string(), "test prompt".to_string());
        assert_eq!(app.active_jobs.len(), 1);
        assert_eq!(app.active_jobs[0].job_id, "job-001");
    }

    #[tokio::test]
    async fn test_update_job_status() {
        let mut app = App::new();
        app.add_job("job-001".to_string(), "test".to_string());

        app.update_job_status(
            "job-001",
            JobStatus::Running {
                stage: "sampling".to_string(),
                progress: 0.5,
                eta_s: 2.0,
            },
        );

        assert!(matches!(
            app.active_jobs[0].status,
            JobStatus::Running { .. }
        ));
    }

    #[tokio::test]
    async fn test_gallery_navigation() {
        let mut app = App::new();
        app.add_to_gallery(PathBuf::from("/test/img1.png"));
        app.add_to_gallery(PathBuf::from("/test/img2.png"));

        assert_eq!(app.selected_gallery_index, 0);
        app.gallery_next();
        assert_eq!(app.selected_gallery_index, 1);
        app.gallery_next();
        assert_eq!(app.selected_gallery_index, 0); // Wraps around

        app.gallery_prev();
        assert_eq!(app.selected_gallery_index, 1); // Wraps backward
    }
}
