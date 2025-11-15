//! # Comparison Result Resource
//!
//! Stores generation results for each model in the comparison.

use bevy::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

/// Result data for a single model in comparison.
#[derive(Debug, Clone)]
pub struct ModelResult {
    /// Model name/ID
    pub model_name: String,
    /// Path to generated image
    pub image_path: Option<PathBuf>,
    /// Bevy asset handle for the image
    pub image_handle: Option<Handle<Image>>,
    /// Image dimensions (width, height)
    pub dimensions: Option<(u32, u32)>,
    /// Generation time in seconds
    pub generation_time: Option<Duration>,
    /// Prompt used for generation
    pub prompt: String,
    /// Whether generation is complete
    pub is_complete: bool,
    /// Error message if generation failed
    pub error: Option<String>,
}

impl ModelResult {
    /// Create a new pending result for a model.
    pub fn new(model_name: String, prompt: String) -> Self {
        Self {
            model_name,
            image_path: None,
            image_handle: None,
            dimensions: None,
            generation_time: None,
            prompt,
            is_complete: false,
            error: None,
        }
    }

    /// Mark result as complete with image data.
    pub fn complete(
        &mut self,
        image_path: PathBuf,
        dimensions: (u32, u32),
        generation_time: Duration,
    ) {
        self.image_path = Some(image_path);
        self.dimensions = Some(dimensions);
        self.generation_time = Some(generation_time);
        self.is_complete = true;
    }

    /// Mark result as failed with error message.
    pub fn fail(&mut self, error: String) {
        self.error = Some(error);
        self.is_complete = true;
    }

    /// Check if the image asset is loaded.
    pub fn is_loaded(&self) -> bool {
        self.image_handle.is_some()
    }

    /// Get formatted generation time.
    pub fn generation_time_str(&self) -> String {
        self.generation_time
            .map(|d| format!("{:.1}s", d.as_secs_f32()))
            .unwrap_or_else(|| "--".to_string())
    }

    /// Get formatted dimensions.
    pub fn dimensions_str(&self) -> String {
        self.dimensions
            .map(|(w, h)| format!("{}x{}", w, h))
            .unwrap_or_else(|| "--".to_string())
    }
}

/// Comparison results resource.
#[derive(Resource, Debug, Clone, Default)]
pub struct ComparisonResults {
    /// Results for each model in the comparison
    pub results: Vec<ModelResult>,
}

impl ComparisonResults {
    /// Create new empty results.
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Initialize results for multiple models.
    pub fn init_for_models(models: &[String], prompt: &str) -> Self {
        let results = models
            .iter()
            .map(|model| ModelResult::new(model.clone(), prompt.to_string()))
            .collect();
        Self { results }
    }

    /// Get result for a specific model.
    pub fn get(&self, model_name: &str) -> Option<&ModelResult> {
        self.results.iter().find(|r| r.model_name == model_name)
    }

    /// Get mutable result for a specific model.
    pub fn get_mut(&mut self, model_name: &str) -> Option<&mut ModelResult> {
        self.results
            .iter_mut()
            .find(|r| r.model_name == model_name)
    }

    /// Check if all results are complete.
    pub fn all_complete(&self) -> bool {
        !self.results.is_empty() && self.results.iter().all(|r| r.is_complete)
    }

    /// Check if any results have errors.
    pub fn has_errors(&self) -> bool {
        self.results.iter().any(|r| r.error.is_some())
    }

    /// Clear all results.
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_result_creation() {
        let result = ModelResult::new("test-model".to_string(), "test prompt".to_string());
        assert_eq!(result.model_name, "test-model");
        assert_eq!(result.prompt, "test prompt");
        assert!(!result.is_complete);
        assert!(result.image_path.is_none());
    }

    #[test]
    fn test_model_result_complete() {
        let mut result = ModelResult::new("test-model".to_string(), "test prompt".to_string());
        result.complete(
            PathBuf::from("/test.png"),
            (512, 512),
            Duration::from_secs_f32(3.2),
        );

        assert!(result.is_complete);
        assert_eq!(result.dimensions_str(), "512x512");
        assert_eq!(result.generation_time_str(), "3.2s");
    }

    #[test]
    fn test_model_result_fail() {
        let mut result = ModelResult::new("test-model".to_string(), "test prompt".to_string());
        result.fail("GPU error".to_string());

        assert!(result.is_complete);
        assert_eq!(result.error, Some("GPU error".to_string()));
    }

    #[test]
    fn test_comparison_results_init() {
        let models = vec!["model-a".to_string(), "model-b".to_string()];
        let results = ComparisonResults::init_for_models(&models, "test prompt");

        assert_eq!(results.results.len(), 2);
        assert_eq!(results.results[0].model_name, "model-a");
        assert_eq!(results.results[1].model_name, "model-b");
    }

    #[test]
    fn test_comparison_results_get() {
        let models = vec!["model-a".to_string()];
        let results = ComparisonResults::init_for_models(&models, "test prompt");

        let result = results.get("model-a");
        assert!(result.is_some());
        assert_eq!(result.unwrap().model_name, "model-a");

        let missing = results.get("model-b");
        assert!(missing.is_none());
    }

    #[test]
    fn test_all_complete() {
        let models = vec!["model-a".to_string(), "model-b".to_string()];
        let mut results = ComparisonResults::init_for_models(&models, "test prompt");

        assert!(!results.all_complete());

        // Complete first model
        results
            .get_mut("model-a")
            .unwrap()
            .complete(PathBuf::from("/a.png"), (512, 512), Duration::from_secs(3));
        assert!(!results.all_complete());

        // Complete second model
        results
            .get_mut("model-b")
            .unwrap()
            .complete(PathBuf::from("/b.png"), (512, 512), Duration::from_secs(3));
        assert!(results.all_complete());
    }

    #[test]
    fn test_has_errors() {
        let models = vec!["model-a".to_string()];
        let mut results = ComparisonResults::init_for_models(&models, "test prompt");

        assert!(!results.has_errors());

        results.get_mut("model-a").unwrap().fail("Error".to_string());
        assert!(results.has_errors());
    }
}
