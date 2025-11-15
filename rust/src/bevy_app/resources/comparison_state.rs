//! # Comparison State Resource
//!
//! Manages side-by-side model comparison state and results.

use bevy::prelude::*;

/// Comparison state resource for side-by-side model comparison.
#[derive(Resource, Debug, Clone)]
pub struct ComparisonState {
    /// Models selected for comparison (up to 3)
    pub models: Vec<String>,
    /// Shared prompt for comparison
    pub prompt: String,
    /// Selected model index
    pub selected_index: usize,
    /// Whether comparison is running
    pub is_running: bool,
}

impl Default for ComparisonState {
    fn default() -> Self {
        Self {
            models: vec!["base-sdxl".to_string(), "custom-lora".to_string()],
            prompt: String::new(),
            selected_index: 0,
            is_running: false,
        }
    }
}

impl ComparisonState {
    /// Add a model to comparison (max 3).
    pub fn add_model(&mut self, model: String) {
        if self.models.len() < 3 && !self.models.contains(&model) {
            self.models.push(model);
        }
    }

    /// Remove selected model.
    pub fn remove_selected(&mut self) {
        if !self.models.is_empty() && self.selected_index < self.models.len() {
            self.models.remove(self.selected_index);
            if self.selected_index >= self.models.len() && !self.models.is_empty() {
                self.selected_index = self.models.len() - 1;
            }
        }
    }

    /// Select next model.
    pub fn next(&mut self) {
        if !self.models.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.models.len();
        }
    }

    /// Select previous model.
    pub fn previous(&mut self) {
        if !self.models.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.models.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Start comparison.
    pub fn start_comparison(&mut self) {
        if !self.prompt.is_empty() && !self.models.is_empty() {
            self.is_running = true;
        }
    }

    /// Stop comparison.
    pub fn stop_comparison(&mut self) {
        self.is_running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = ComparisonState::default();
        assert_eq!(state.models.len(), 2);
        assert_eq!(state.selected_index, 0);
        assert!(!state.is_running);
    }

    #[test]
    fn test_add_model() {
        let mut state = ComparisonState::default();
        state.add_model("new-model".to_string());
        assert_eq!(state.models.len(), 3);
    }

    #[test]
    fn test_max_models() {
        let mut state = ComparisonState::default();
        state.add_model("model3".to_string());
        state.add_model("model4".to_string()); // Should not be added
        assert_eq!(state.models.len(), 3); // Max 3
    }

    #[test]
    fn test_navigation() {
        let mut state = ComparisonState::default();
        assert_eq!(state.selected_index, 0);

        state.next();
        assert_eq!(state.selected_index, 1);

        state.next(); // Wrap around
        assert_eq!(state.selected_index, 0);

        state.previous(); // Wrap backwards
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn test_remove_selected() {
        let mut state = ComparisonState::default();
        state.selected_index = 1;
        state.remove_selected();
        assert_eq!(state.models.len(), 1);
        assert_eq!(state.selected_index, 0);
    }
}
