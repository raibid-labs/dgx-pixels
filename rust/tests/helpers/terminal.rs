//! Terminal testing utilities

use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// Create a test terminal with the given size
pub fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("Failed to create test terminal")
}

/// Create a standard size test terminal (120x40)
pub fn create_standard_terminal() -> Terminal<TestBackend> {
    create_test_terminal(120, 40)
}

/// Create a small test terminal (80x24)
pub fn create_small_terminal() -> Terminal<TestBackend> {
    create_test_terminal(80, 24)
}

/// Create a large test terminal (200x60)
pub fn create_large_terminal() -> Terminal<TestBackend> {
    create_test_terminal(200, 60)
}

/// Enable test mode (prevents sixel rendering to stdout)
pub fn enable_test_mode() {
    std::env::set_var("RATATUI_TEST_MODE", "1");
}

/// Disable test mode
pub fn disable_test_mode() {
    std::env::remove_var("RATATUI_TEST_MODE");
}

/// RAII guard for test mode
pub struct TestModeGuard;

impl TestModeGuard {
    pub fn new() -> Self {
        enable_test_mode();
        Self
    }
}

impl Drop for TestModeGuard {
    fn drop(&mut self) {
        disable_test_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_terminal() {
        let terminal = create_test_terminal(80, 24);
        assert_eq!(terminal.size().unwrap().width, 80);
        assert_eq!(terminal.size().unwrap().height, 24);
    }

    #[test]
    fn test_standard_terminal_size() {
        let terminal = create_standard_terminal();
        assert_eq!(terminal.size().unwrap().width, 120);
        assert_eq!(terminal.size().unwrap().height, 40);
    }

    #[test]
    fn test_test_mode_guard() {
        assert!(std::env::var("RATATUI_TEST_MODE").is_err());

        {
            let _guard = TestModeGuard::new();
            assert_eq!(std::env::var("RATATUI_TEST_MODE").unwrap(), "1");
        }

        // Should be removed after guard drops
        assert!(std::env::var("RATATUI_TEST_MODE").is_err());
    }
}
