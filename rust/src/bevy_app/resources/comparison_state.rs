//! # Comparison State Resource
//!
//! Manages side-by-side model comparison state and results.

use bevy::prelude::*;
use std::path::PathBuf;

/// Which pane is currently selected in the comparison view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonPane {
    Left,
    Right,
}

/// Comparison mode: single model vs. multi-model comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonMode {
    /// Side-by-side dual comparison (default)
    Dual,
    /// Multi-model comparison (up to 3)
    Multi,
}

/// Comparison state resource for side-by-side model comparison.
#[derive(Resource, Debug, Clone)]
pub struct ComparisonState {
    /// Mode: dual or multi comparison
    pub mode: ComparisonMode,

    /// Models selected for comparison (up to 3)
    pub models: Vec<String>,

    /// Shared prompt for comparison
    pub prompt: String,

    /// Selected model index (for multi mode)
    pub selected_index: usize,

    /// Whether comparison is running
    pub is_running: bool,

    // === Dual Comparison Mode Fields ===
    /// Currently selected pane (left or right)
    pub selected_pane: ComparisonPane,

    /// Left pane model
    pub left_model: Option<String>,

    /// Right pane model
    pub right_model: Option<String>,

    /// Left pane generated image path
    pub left_image: Option<PathBuf>,

    /// Right pane generated image path
    pub right_image: Option<PathBuf>,

    /// Left pane job ID (for tracking)
    pub left_job_id: Option<String>,

    /// Right pane job ID (for tracking)
    pub right_job_id: Option<String>,

    /// Left pane generation metadata
    pub left_metadata: Option<GenerationMetadata>,

    /// Right pane generation metadata
    pub right_metadata: Option<GenerationMetadata>,

    /// Available models (fetched from backend)
    pub available_models: Vec<ModelEntry>,

    /// Currently browsing model list
    pub browsing_models: bool,

    /// Selected index in model list when browsing
    pub model_list_index: usize,
}

/// Generation metadata for a completed comparison
#[derive(Debug, Clone)]
pub struct GenerationMetadata {
    pub size: (u32, u32),
    pub seed: Option<u64>,
    pub inference_time_s: f32,
    pub steps: u32,
}

/// Model entry from backend
#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub name: String,
    pub model_type: String,
    pub path: String,
}

impl Default for ComparisonState {
    fn default() -> Self {
        Self {
            mode: ComparisonMode::Dual,
            models: vec!["base-sdxl".to_string(), "custom-lora".to_string()],
            prompt: String::new(),
            selected_index: 0,
            is_running: false,
            selected_pane: ComparisonPane::Left,
            left_model: Some("SDXL Base 1.0".to_string()),
            right_model: Some("Pixel Art LoRA v1".to_string()),
            left_image: None,
            right_image: None,
            left_job_id: None,
            right_job_id: None,
            left_metadata: None,
            right_metadata: None,
            available_models: Vec::new(),
            browsing_models: false,
            model_list_index: 0,
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

    // === Dual Comparison Methods ===

    /// Switch to the other pane
    pub fn toggle_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            ComparisonPane::Left => ComparisonPane::Right,
            ComparisonPane::Right => ComparisonPane::Left,
        };
    }

    /// Set model for currently selected pane
    pub fn set_selected_pane_model(&mut self, model: String) {
        match self.selected_pane {
            ComparisonPane::Left => self.left_model = Some(model),
            ComparisonPane::Right => self.right_model = Some(model),
        }
    }

    /// Get model for currently selected pane
    pub fn get_selected_pane_model(&self) -> Option<&String> {
        match self.selected_pane {
            ComparisonPane::Left => self.left_model.as_ref(),
            ComparisonPane::Right => self.right_model.as_ref(),
        }
    }

    /// Navigate to next model in available models list
    pub fn next_available_model(&mut self) {
        if !self.available_models.is_empty() {
            self.model_list_index = (self.model_list_index + 1) % self.available_models.len();
        }
    }

    /// Navigate to previous model in available models list
    pub fn previous_available_model(&mut self) {
        if !self.available_models.is_empty() {
            self.model_list_index = if self.model_list_index == 0 {
                self.available_models.len() - 1
            } else {
                self.model_list_index - 1
            };
        }
    }

    /// Select currently browsed model for the selected pane
    pub fn select_current_model(&mut self) {
        if let Some(model) = self.available_models.get(self.model_list_index) {
            self.set_selected_pane_model(model.name.clone());
            self.browsing_models = false;
        }
    }

    /// Start dual comparison generation
    pub fn start_dual_comparison(&mut self) {
        if !self.prompt.is_empty()
            && self.left_model.is_some()
            && self.right_model.is_some() {
            self.is_running = true;
            // Clear previous results
            self.left_image = None;
            self.right_image = None;
            self.left_metadata = None;
            self.right_metadata = None;
        }
    }

    /// Update left pane result
    pub fn update_left_result(
        &mut self,
        job_id: String,
        image_path: PathBuf,
        metadata: GenerationMetadata
    ) {
        self.left_job_id = Some(job_id);
        self.left_image = Some(image_path);
        self.left_metadata = Some(metadata);
    }

    /// Update right pane result
    pub fn update_right_result(
        &mut self,
        job_id: String,
        image_path: PathBuf,
        metadata: GenerationMetadata
    ) {
        self.right_job_id = Some(job_id);
        self.right_image = Some(image_path);
        self.right_metadata = Some(metadata);
    }

    /// Check if both comparisons are complete
    pub fn is_comparison_complete(&self) -> bool {
        self.left_image.is_some() && self.right_image.is_some()
    }

    /// Reset comparison results
    pub fn reset_results(&mut self) {
        self.left_image = None;
        self.right_image = None;
        self.left_job_id = None;
        self.right_job_id = None;
        self.left_metadata = None;
        self.right_metadata = None;
        self.is_running = false;
    }

    /// Populate available models from backend response
    pub fn set_available_models(&mut self, models: Vec<ModelEntry>) {
        self.available_models = models;
        self.model_list_index = 0;
    }

    /// Check if ready to run comparison
    pub fn can_run_comparison(&self) -> bool {
        match self.mode {
            ComparisonMode::Dual => {
                !self.prompt.is_empty()
                    && self.left_model.is_some()
                    && self.right_model.is_some()
            }
            ComparisonMode::Multi => {
                !self.prompt.is_empty() && !self.models.is_empty()
            }
        }
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
        assert_eq!(state.mode, ComparisonMode::Dual);
        assert_eq!(state.selected_pane, ComparisonPane::Left);
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

    #[test]
    fn test_toggle_pane() {
        let mut state = ComparisonState::default();
        assert_eq!(state.selected_pane, ComparisonPane::Left);

        state.toggle_pane();
        assert_eq!(state.selected_pane, ComparisonPane::Right);

        state.toggle_pane();
        assert_eq!(state.selected_pane, ComparisonPane::Left);
    }

    #[test]
    fn test_set_pane_model() {
        let mut state = ComparisonState::default();

        state.selected_pane = ComparisonPane::Left;
        state.set_selected_pane_model("Test Model".to_string());
        assert_eq!(state.left_model, Some("Test Model".to_string()));

        state.selected_pane = ComparisonPane::Right;
        state.set_selected_pane_model("Another Model".to_string());
        assert_eq!(state.right_model, Some("Another Model".to_string()));
    }

    #[test]
    fn test_can_run_comparison() {
        let mut state = ComparisonState::default();
        state.prompt = String::new();
        assert!(!state.can_run_comparison());

        state.prompt = "test prompt".to_string();
        assert!(state.can_run_comparison()); // Has default models set
    }

    #[test]
    fn test_comparison_complete() {
        let mut state = ComparisonState::default();
        assert!(!state.is_comparison_complete());

        state.left_image = Some(PathBuf::from("/test/left.png"));
        assert!(!state.is_comparison_complete());

        state.right_image = Some(PathBuf::from("/test/right.png"));
        assert!(state.is_comparison_complete());
    }

    #[test]
    fn test_reset_results() {
        let mut state = ComparisonState::default();
        state.left_image = Some(PathBuf::from("/test/left.png"));
        state.right_image = Some(PathBuf::from("/test/right.png"));
        state.is_running = true;

        state.reset_results();

        assert!(state.left_image.is_none());
        assert!(state.right_image.is_none());
        assert!(!state.is_running);
    }
}
