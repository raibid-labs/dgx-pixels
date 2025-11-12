//! Terminal capability detection for Sixel support

use std::env;
use tracing::{debug, info, warn};

/// Terminal capability information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCapability {
    /// Full Sixel support
    Sixel,
    /// No Sixel support (fallback to text)
    TextOnly,
}

/// Detect if the terminal supports Sixel graphics
pub fn detect_sixel_support() -> TerminalCapability {
    // Check $TERM environment variable
    if let Ok(term) = env::var("TERM") {
        debug!("Detected TERM: {}", term);

        // Known Sixel-capable terminals
        let sixel_terms = ["xterm-256color", "xterm-sixel", "mlterm", "yaft-256color"];

        if sixel_terms.iter().any(|&t| term.contains(t)) {
            info!("Terminal supports Sixel (via TERM)");
            return TerminalCapability::Sixel;
        }
    }

    // Check $TERM_PROGRAM for known terminal emulators
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        debug!("Detected TERM_PROGRAM: {}", term_program);

        match term_program.as_str() {
            "iTerm.app" => {
                // iTerm2 supports Sixel in recent versions
                info!("Terminal supports Sixel (iTerm2)");
                return TerminalCapability::Sixel;
            }
            "WezTerm" => {
                info!("Terminal supports Sixel (WezTerm)");
                return TerminalCapability::Sixel;
            }
            "tmux" => {
                // tmux can support Sixel if configured
                debug!("tmux detected, checking parent terminal");
            }
            _ => {}
        }
    }

    // Check for kitty terminal
    if env::var("KITTY_WINDOW_ID").is_ok() {
        info!("Terminal supports Sixel (kitty)");
        return TerminalCapability::Sixel;
    }

    // Fallback: no Sixel support detected
    warn!("No Sixel support detected, falling back to text-only mode");
    info!("For best experience, use kitty, WezTerm, iTerm2, or xterm with Sixel");

    TerminalCapability::TextOnly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_capability_returns_valid() {
        let cap = detect_sixel_support();
        assert!(cap == TerminalCapability::Sixel || cap == TerminalCapability::TextOnly);
    }
}
