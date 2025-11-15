//! # Models State Resource
//!
//! Manages the state of available AI models (SDXL base models, LoRAs, VAEs)
//! and handles model activation, download tracking, and metadata display.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use dgx_pixels_tui::bevy_app::resources::ModelsState;
//!
//! fn my_system(models: Res<ModelsState>) {
//!     println!("Active model: {:?}", models.active_model);
//!     println!("Available models: {}", models.models.len());
//! }
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Models state resource tracking available AI models.
#[derive(Resource, Debug, Clone)]
pub struct ModelsState {
    /// List of all available models.
    pub models: Vec<ModelInfo>,
    /// Currently selected index in the table.
    pub selected_index: usize,
    /// Currently active model name (used for generation).
    pub active_model: Option<String>,
    /// Whether to show detailed metadata panel.
    pub show_metadata: bool,
    /// Scroll offset for model list.
    pub scroll_offset: usize,
}

impl Default for ModelsState {
    fn default() -> Self {
        Self {
            models: Self::default_models(),
            selected_index: 0,
            active_model: Some("SDXL Base 1.0".to_string()),
            show_metadata: false,
            scroll_offset: 0,
        }
    }
}

impl ModelsState {
    /// Create default models list with placeholder data.
    fn default_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                name: "SDXL Base 1.0".to_string(),
                model_type: ModelType::Base,
                size_mb: 6938,
                status: ModelStatus::Downloaded,
                metadata: ModelMetadata {
                    version: "1.0".to_string(),
                    description:
                        "Stable Diffusion XL base model - high quality 1024x1024 generation"
                            .to_string(),
                    parameters: "2.6B".to_string(),
                    license: "CreativeML Open RAIL++-M".to_string(),
                    source: "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
                },
            },
            ModelInfo {
                name: "SDXL Refiner 1.0".to_string(),
                model_type: ModelType::Base,
                size_mb: 6075,
                status: ModelStatus::Available,
                metadata: ModelMetadata {
                    version: "1.0".to_string(),
                    description: "SDXL refiner model for enhanced detail and quality".to_string(),
                    parameters: "2.3B".to_string(),
                    license: "CreativeML Open RAIL++-M".to_string(),
                    source: "stabilityai/stable-diffusion-xl-refiner-1.0".to_string(),
                },
            },
            ModelInfo {
                name: "Pixel Art LoRA v1".to_string(),
                model_type: ModelType::LoRA,
                size_mb: 144,
                status: ModelStatus::Downloaded,
                metadata: ModelMetadata {
                    version: "1.0.0".to_string(),
                    description:
                        "Custom trained LoRA for pixel art style - optimized for game sprites"
                            .to_string(),
                    parameters: "144M".to_string(),
                    license: "MIT".to_string(),
                    source: "local/pixel-art-lora-v1".to_string(),
                },
            },
            ModelInfo {
                name: "Game Assets LoRA v2".to_string(),
                model_type: ModelType::LoRA,
                size_mb: 156,
                status: ModelStatus::Downloading(42),
                metadata: ModelMetadata {
                    version: "2.1.0".to_string(),
                    description:
                        "LoRA trained on game asset datasets - tiles, sprites, UI elements"
                            .to_string(),
                    parameters: "156M".to_string(),
                    license: "Apache-2.0".to_string(),
                    source: "local/game-assets-lora-v2".to_string(),
                },
            },
            ModelInfo {
                name: "SDXL VAE".to_string(),
                model_type: ModelType::VAE,
                size_mb: 335,
                status: ModelStatus::Downloaded,
                metadata: ModelMetadata {
                    version: "1.0".to_string(),
                    description: "Variational autoencoder for SDXL - improved color accuracy"
                        .to_string(),
                    parameters: "83M".to_string(),
                    license: "CreativeML Open RAIL++-M".to_string(),
                    source: "stabilityai/sdxl-vae".to_string(),
                },
            },
            ModelInfo {
                name: "Retro Pixel LoRA".to_string(),
                model_type: ModelType::LoRA,
                size_mb: 128,
                status: ModelStatus::Available,
                metadata: ModelMetadata {
                    version: "1.2.0".to_string(),
                    description: "8-bit and 16-bit retro game art style".to_string(),
                    parameters: "128M".to_string(),
                    license: "CC-BY-4.0".to_string(),
                    source: "civitai/retro-pixel-lora".to_string(),
                },
            },
        ]
    }

    /// Navigate to next model in the list.
    pub fn next(&mut self) {
        if !self.models.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.models.len();
            self.update_scroll();
        }
    }

    /// Navigate to previous model in the list.
    pub fn previous(&mut self) {
        if !self.models.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.models.len() - 1
            } else {
                self.selected_index - 1
            };
            self.update_scroll();
        }
    }

    /// Toggle model activation (for base models and LoRAs).
    pub fn toggle_active(&mut self) {
        if let Some(model) = self.models.get(self.selected_index) {
            match model.status {
                ModelStatus::Downloaded => {
                    // Toggle activation
                    if self.active_model.as_ref() == Some(&model.name) {
                        self.active_model = None;
                    } else {
                        // For base models, only one can be active
                        // For LoRAs, multiple can be active (TODO: support multiple)
                        self.active_model = Some(model.name.clone());
                    }
                }
                _ => {
                    // Can't activate if not downloaded
                }
            }
        }
    }

    /// Start downloading the selected model.
    pub fn download_selected(&mut self) {
        if let Some(model) = self.models.get_mut(self.selected_index) {
            if model.status == ModelStatus::Available {
                model.status = ModelStatus::Downloading(0);
                // TODO: Send download request via ZeroMQ
            }
        }
    }

    /// Delete the selected model.
    pub fn delete_selected(&mut self) {
        if let Some(model) = self.models.get(self.selected_index) {
            if model.status == ModelStatus::Downloaded {
                // TODO: Implement actual deletion
                // For now, just mark as available
                if let Some(m) = self.models.get_mut(self.selected_index) {
                    m.status = ModelStatus::Available;
                }
            }
        }
    }

    /// Toggle metadata panel visibility.
    pub fn toggle_metadata(&mut self) {
        self.show_metadata = !self.show_metadata;
    }

    /// Update scroll offset to keep selected item visible.
    fn update_scroll(&mut self) {
        // Will be used when implementing scrolling for large model lists
    }

    /// Get the currently selected model.
    pub fn selected_model(&self) -> Option<&ModelInfo> {
        self.models.get(self.selected_index)
    }

    /// Get memory usage statistics.
    pub fn memory_stats(&self) -> (usize, usize) {
        let downloaded_size: usize = self
            .models
            .iter()
            .filter(|m| matches!(m.status, ModelStatus::Downloaded))
            .map(|m| m.size_mb)
            .sum();

        let total_available: usize = self.models.iter().map(|m| m.size_mb).sum();

        (downloaded_size, total_available)
    }
}

/// Information about a single model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model_type: ModelType,
    pub size_mb: usize,
    pub status: ModelStatus,
    pub metadata: ModelMetadata,
}

/// Type of AI model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Base SDXL checkpoint model.
    Base,
    /// LoRA (Low-Rank Adaptation) fine-tuning adapter.
    LoRA,
    /// VAE (Variational Autoencoder) for improved encoding/decoding.
    VAE,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Base => write!(f, "Base"),
            ModelType::LoRA => write!(f, "LoRA"),
            ModelType::VAE => write!(f, "VAE"),
        }
    }
}

/// Download and activation status of a model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Model is downloaded and ready to use.
    Downloaded,
    /// Model is available for download but not yet downloaded.
    Available,
    /// Model is currently downloading (progress percentage).
    Downloading(u8),
    /// Model download failed.
    Failed,
}

impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelStatus::Downloaded => write!(f, "Downloaded"),
            ModelStatus::Available => write!(f, "Available"),
            ModelStatus::Downloading(pct) => write!(f, "Downloading {}%", pct),
            ModelStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// Metadata for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub version: String,
    pub description: String,
    pub parameters: String,
    pub license: String,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_models_state() {
        let state = ModelsState::default();
        assert!(!state.models.is_empty());
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.active_model, Some("SDXL Base 1.0".to_string()));
    }

    #[test]
    fn test_navigation_next() {
        let mut state = ModelsState::default();
        let initial = state.selected_index;
        state.next();
        assert_eq!(state.selected_index, initial + 1);
    }

    #[test]
    fn test_navigation_previous() {
        let mut state = ModelsState::default();
        state.previous();
        assert_eq!(state.selected_index, state.models.len() - 1);
    }

    #[test]
    fn test_navigation_wraps() {
        let mut state = ModelsState::default();
        let count = state.models.len();
        for _ in 0..count {
            state.next();
        }
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_toggle_active() {
        let mut state = ModelsState::default();
        state.selected_index = 0; // SDXL Base 1.0

        let initial_active = state.active_model.clone();
        state.toggle_active();
        assert_ne!(state.active_model, initial_active);
    }

    #[test]
    fn test_memory_stats() {
        let state = ModelsState::default();
        let (downloaded, _total) = state.memory_stats();
        assert!(downloaded > 0);
    }

    #[test]
    fn test_download_selected() {
        let mut state = ModelsState::default();
        // Find an available model
        state.selected_index = state
            .models
            .iter()
            .position(|m| m.status == ModelStatus::Available)
            .unwrap_or(1);

        let initial_status = state.models[state.selected_index].status.clone();
        state.download_selected();

        if initial_status == ModelStatus::Available {
            assert!(matches!(
                state.models[state.selected_index].status,
                ModelStatus::Downloading(_)
            ));
        }
    }

    #[test]
    fn test_selected_model() {
        let state = ModelsState::default();
        let selected = state.selected_model();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, state.models[0].name);
    }

    #[test]
    fn test_toggle_metadata() {
        let mut state = ModelsState::default();
        assert!(!state.show_metadata);
        state.toggle_metadata();
        assert!(state.show_metadata);
        state.toggle_metadata();
        assert!(!state.show_metadata);
    }
}
