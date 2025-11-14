//! # Input Buffer Resource
//!
//! Manages text input state including the input buffer and cursor position.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::InputBuffer;
//!
//! fn handle_text_input(mut input: ResMut<InputBuffer>) {
//!     input.insert('a');
//!     input.insert('b');
//!     println!("Input: {}", input.text);
//! }
//! ```

use bevy::prelude::*;

/// Input buffer state resource.
#[derive(Resource, Debug, Clone)]
pub struct InputBuffer {
    /// Current text input
    pub text: String,
    /// Cursor position (character index)
    pub cursor: usize,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }
}

impl InputBuffer {
    /// Insert character at cursor position.
    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Delete character before cursor (backspace).
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start of input.
    pub fn move_to_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end of input.
    pub fn move_to_end(&mut self) {
        self.cursor = self.text.len();
    }

    /// Clear buffer and reset cursor.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_buffer() {
        let buffer = InputBuffer::default();
        assert_eq!(buffer.text, "");
        assert_eq!(buffer.cursor, 0);
    }

    #[test]
    fn test_insert_characters() {
        let mut buffer = InputBuffer::default();
        buffer.insert('a');
        buffer.insert('b');
        buffer.insert('c');
        assert_eq!(buffer.text, "abc");
        assert_eq!(buffer.cursor, 3);
    }

    #[test]
    fn test_backspace() {
        let mut buffer = InputBuffer::default();
        buffer.insert('a');
        buffer.insert('b');
        buffer.backspace();
        assert_eq!(buffer.text, "a");
        assert_eq!(buffer.cursor, 1);
    }

    #[test]
    fn test_backspace_at_start() {
        let mut buffer = InputBuffer::default();
        buffer.backspace(); // Should do nothing
        assert_eq!(buffer.text, "");
        assert_eq!(buffer.cursor, 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut buffer = InputBuffer::default();
        buffer.insert('a');
        buffer.insert('b');
        buffer.insert('c');

        buffer.move_left();
        assert_eq!(buffer.cursor, 2);

        buffer.move_left();
        assert_eq!(buffer.cursor, 1);

        buffer.move_right();
        assert_eq!(buffer.cursor, 2);
    }

    #[test]
    fn test_move_to_boundaries() {
        let mut buffer = InputBuffer::default();
        buffer.insert('h');
        buffer.insert('e');
        buffer.insert('l');
        buffer.insert('l');
        buffer.insert('o');

        buffer.move_to_start();
        assert_eq!(buffer.cursor, 0);

        buffer.move_to_end();
        assert_eq!(buffer.cursor, 5);
    }

    #[test]
    fn test_clear() {
        let mut buffer = InputBuffer::default();
        buffer.insert('t');
        buffer.insert('e');
        buffer.insert('s');
        buffer.insert('t');
        buffer.clear();
        assert_eq!(buffer.text, "");
        assert_eq!(buffer.cursor, 0);
    }
}
