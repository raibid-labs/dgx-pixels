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

    /// Delete character at cursor position (delete key).
    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
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

    /// Delete word before cursor (Ctrl+W).
    pub fn delete_word(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let text_before = &self.text[..self.cursor];

        // Find the start of the word to delete
        // Skip trailing whitespace first
        let mut pos = text_before.len();
        let chars: Vec<char> = text_before.chars().collect();

        // Skip trailing whitespace
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Delete word characters
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Remove from pos to cursor
        self.text.replace_range(pos..self.cursor, "");
        self.cursor = pos;
    }

    /// Clear all text before cursor (Ctrl+U).
    pub fn delete_to_start(&mut self) {
        if self.cursor > 0 {
            self.text.replace_range(0..self.cursor, "");
            self.cursor = 0;
        }
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
    fn test_delete() {
        let mut buffer = InputBuffer::default();
        buffer.insert('a');
        buffer.insert('b');
        buffer.insert('c');
        buffer.move_to_start();
        buffer.delete();
        assert_eq!(buffer.text, "bc");
        assert_eq!(buffer.cursor, 0);
    }

    #[test]
    fn test_delete_at_end() {
        let mut buffer = InputBuffer::default();
        buffer.insert('a');
        buffer.delete(); // Should do nothing
        assert_eq!(buffer.text, "a");
        assert_eq!(buffer.cursor, 1);
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
    fn test_delete_word() {
        let mut buffer = InputBuffer::default();
        for c in "hello world test".chars() {
            buffer.insert(c);
        }

        buffer.delete_word(); // Delete "test"
        assert_eq!(buffer.text, "hello world ");

        buffer.delete_word(); // Delete "world "
        assert_eq!(buffer.text, "hello ");

        buffer.delete_word(); // Delete "hello "
        assert_eq!(buffer.text, "");
    }

    #[test]
    fn test_delete_word_at_start() {
        let mut buffer = InputBuffer::default();
        buffer.insert('h');
        buffer.insert('i');
        buffer.move_to_start();
        buffer.delete_word(); // Should do nothing
        assert_eq!(buffer.text, "hi");
        assert_eq!(buffer.cursor, 0);
    }

    #[test]
    fn test_delete_to_start() {
        let mut buffer = InputBuffer::default();
        for c in "hello world".chars() {
            buffer.insert(c);
        }

        buffer.move_left();
        buffer.move_left();
        buffer.delete_to_start(); // Delete "hello wor"
        assert_eq!(buffer.text, "ld");
        assert_eq!(buffer.cursor, 0);
    }

    #[test]
    fn test_delete_to_start_at_beginning() {
        let mut buffer = InputBuffer::default();
        buffer.insert('h');
        buffer.insert('i');
        buffer.move_to_start();
        buffer.delete_to_start(); // Should do nothing
        assert_eq!(buffer.text, "hi");
        assert_eq!(buffer.cursor, 0);
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
