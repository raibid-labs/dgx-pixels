//! # Settings State Resource
//!
//! Manages application configuration settings across:
//! - Generation defaults (model, steps, cfg_scale, size)
//! - UI preferences (theme, fps_limit, auto_refresh)
//! - Backend configuration (zmq_host, zmq_port, timeout)
//! - Paths (output_dir, cache_dir, models_dir)
//!
//! Settings are persisted to `~/.config/dgx-pixels/config.toml` and can be
//! edited through the Settings screen.

use anyhow::{Context, Result};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main settings resource.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    /// Currently selected setting index (for navigation).
    #[serde(skip)]
    pub selected_index: usize,

    /// Total number of settings (for bounds checking).
    #[serde(skip)]
    pub total_settings: usize,

    /// Whether user is currently editing a value.
    #[serde(skip)]
    pub is_editing: bool,

    /// Buffer for editing text values.
    #[serde(skip)]
    pub edit_buffer: String,

    /// Generation-related settings.
    pub generation: GenerationSettings,

    /// UI-related settings.
    pub ui: UiSettings,

    /// Backend connection settings.
    pub backend: BackendSettings,

    /// File system paths.
    pub paths: PathSettings,
}

/// Settings for image generation defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSettings {
    /// Default model to use for generation.
    pub default_model: String,

    /// Default number of denoising steps.
    pub default_steps: u32,

    /// Default CFG scale (classifier-free guidance).
    pub default_cfg_scale: f32,

    /// Default image size (width, height).
    pub default_size: (u32, u32),

    /// Default sampler name.
    pub default_sampler: String,

    /// Default batch size.
    pub default_batch_size: u32,
}

/// Settings for UI behavior and appearance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Theme name (for future theming support).
    pub theme: String,

    /// Target FPS for rendering.
    pub fps_limit: u32,

    /// Auto-refresh gallery on new generations.
    pub auto_refresh_gallery: bool,

    /// Show image previews in terminal (Sixel).
    pub show_image_previews: bool,

    /// Preview image max width (pixels).
    pub preview_max_width: u32,

    /// Preview image max height (pixels).
    pub preview_max_height: u32,
}

/// Settings for backend connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSettings {
    /// ZeroMQ host address.
    pub zmq_host: String,

    /// ZeroMQ port.
    pub zmq_port: u16,

    /// Request timeout (seconds).
    pub timeout_secs: u32,

    /// Number of retry attempts.
    pub retry_attempts: u32,
}

/// Settings for file system paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSettings {
    /// Output directory for generated images.
    pub output_dir: PathBuf,

    /// Cache directory for temporary files.
    pub cache_dir: PathBuf,

    /// Models directory.
    pub models_dir: PathBuf,

    /// Workflows directory (ComfyUI JSON templates).
    pub workflows_dir: PathBuf,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            total_settings: 18, // Update if adding/removing settings
            is_editing: false,
            edit_buffer: String::new(),
            generation: GenerationSettings::default(),
            ui: UiSettings::default(),
            backend: BackendSettings::default(),
            paths: PathSettings::default(),
        }
    }
}

impl Default for GenerationSettings {
    fn default() -> Self {
        Self {
            default_model: "SDXL Base 1.0".to_string(),
            default_steps: 30,
            default_cfg_scale: 7.5,
            default_size: (1024, 1024),
            default_sampler: "DPM++ 2M Karras".to_string(),
            default_batch_size: 1,
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "Default".to_string(),
            fps_limit: 60,
            auto_refresh_gallery: true,
            show_image_previews: true,
            preview_max_width: 512,
            preview_max_height: 512,
        }
    }
}

impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            zmq_host: "127.0.0.1".to_string(),
            zmq_port: 5555,
            timeout_secs: 30,
            retry_attempts: 3,
        }
    }
}

impl Default for PathSettings {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let dgx_pixels_dir = home.join(".local/share/dgx-pixels");

        Self {
            output_dir: dgx_pixels_dir.join("output"),
            cache_dir: dgx_pixels_dir.join("cache"),
            models_dir: dgx_pixels_dir.join("models"),
            workflows_dir: dgx_pixels_dir.join("workflows"),
        }
    }
}

impl SettingsState {
    /// Get the config file path.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("dgx-pixels")
            .join("config.toml")
    }

    /// Load settings from config file.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            info!("Config file not found, using defaults: {:?}", path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let mut settings: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        // Initialize runtime fields
        settings.selected_index = 0;
        settings.total_settings = 18;
        settings.is_editing = false;
        settings.edit_buffer.clear();

        info!("Settings loaded from {:?}", path);
        Ok(settings)
    }

    /// Save settings to config file.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        // Ensure config directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize settings")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        info!("Settings saved to {:?}", path);
        Ok(())
    }

    /// Reset settings to defaults.
    pub fn reset_to_defaults(&mut self) {
        let defaults = Self::default();
        self.generation = defaults.generation;
        self.ui = defaults.ui;
        self.backend = defaults.backend;
        self.paths = defaults.paths;
    }

    /// Navigate to next setting.
    pub fn next_setting(&mut self) {
        if self.selected_index < self.total_settings - 1 {
            self.selected_index += 1;
        }
    }

    /// Navigate to previous setting.
    pub fn previous_setting(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Start editing the currently selected setting.
    pub fn start_editing(&mut self) {
        self.is_editing = true;

        // Populate edit buffer with current value
        self.edit_buffer = match self.selected_index {
            0 => self.generation.default_model.clone(),
            1 => self.generation.default_steps.to_string(),
            2 => self.generation.default_cfg_scale.to_string(),
            3 => self.generation.default_size.0.to_string(),
            4 => self.generation.default_size.1.to_string(),
            5 => self.generation.default_sampler.clone(),
            6 => self.generation.default_batch_size.to_string(),
            7 => self.ui.theme.clone(),
            8 => self.ui.fps_limit.to_string(),
            9 => self.ui.auto_refresh_gallery.to_string(),
            10 => self.ui.show_image_previews.to_string(),
            11 => self.ui.preview_max_width.to_string(),
            12 => self.ui.preview_max_height.to_string(),
            13 => self.backend.zmq_host.clone(),
            14 => self.backend.zmq_port.to_string(),
            15 => self.backend.timeout_secs.to_string(),
            16 => self.backend.retry_attempts.to_string(),
            17 => self.paths.output_dir.to_string_lossy().to_string(),
            _ => String::new(),
        };
    }

    /// Finish editing and apply the new value.
    pub fn finish_editing(&mut self) -> Result<()> {
        if !self.is_editing {
            return Ok(());
        }

        let value = self.edit_buffer.trim();

        // Apply the edited value to the appropriate setting
        match self.selected_index {
            0 => self.generation.default_model = value.to_string(),
            1 => {
                self.generation.default_steps = value.parse().context("Invalid number for steps")?
            }
            2 => {
                self.generation.default_cfg_scale =
                    value.parse().context("Invalid number for CFG scale")?
            }
            3 => {
                let width: u32 = value.parse().context("Invalid number for width")?;
                self.generation.default_size.0 = width;
            }
            4 => {
                let height: u32 = value.parse().context("Invalid number for height")?;
                self.generation.default_size.1 = height;
            }
            5 => self.generation.default_sampler = value.to_string(),
            6 => {
                self.generation.default_batch_size =
                    value.parse().context("Invalid number for batch size")?
            }
            7 => self.ui.theme = value.to_string(),
            8 => self.ui.fps_limit = value.parse().context("Invalid number for FPS")?,
            9 => {
                self.ui.auto_refresh_gallery =
                    value.parse().context("Invalid boolean for auto refresh")?
            }
            10 => {
                self.ui.show_image_previews =
                    value.parse().context("Invalid boolean for show previews")?
            }
            11 => {
                self.ui.preview_max_width =
                    value.parse().context("Invalid number for preview width")?
            }
            12 => {
                self.ui.preview_max_height =
                    value.parse().context("Invalid number for preview height")?
            }
            13 => self.backend.zmq_host = value.to_string(),
            14 => self.backend.zmq_port = value.parse().context("Invalid number for port")?,
            15 => {
                self.backend.timeout_secs = value.parse().context("Invalid number for timeout")?
            }
            16 => {
                self.backend.retry_attempts = value.parse().context("Invalid number for retries")?
            }
            17 => self.paths.output_dir = PathBuf::from(value),
            _ => {}
        }

        self.is_editing = false;
        self.edit_buffer.clear();
        Ok(())
    }

    /// Cancel editing without applying changes.
    pub fn cancel_editing(&mut self) {
        self.is_editing = false;
        self.edit_buffer.clear();
    }

    /// Toggle a boolean setting.
    pub fn toggle_boolean(&mut self) {
        match self.selected_index {
            9 => self.ui.auto_refresh_gallery = !self.ui.auto_refresh_gallery,
            10 => self.ui.show_image_previews = !self.ui.show_image_previews,
            _ => {}
        }
    }

    /// Increment a numeric setting.
    pub fn increment_value(&mut self) {
        match self.selected_index {
            1 => {
                self.generation.default_steps =
                    self.generation.default_steps.saturating_add(5).min(100)
            }
            2 => {
                self.generation.default_cfg_scale =
                    (self.generation.default_cfg_scale + 0.5).min(20.0)
            }
            3 => {
                self.generation.default_size.0 =
                    self.generation.default_size.0.saturating_add(64).min(2048)
            }
            4 => {
                self.generation.default_size.1 =
                    self.generation.default_size.1.saturating_add(64).min(2048)
            }
            6 => {
                self.generation.default_batch_size =
                    self.generation.default_batch_size.saturating_add(1).min(10)
            }
            8 => self.ui.fps_limit = self.ui.fps_limit.saturating_add(10).min(120),
            11 => {
                self.ui.preview_max_width = self.ui.preview_max_width.saturating_add(64).min(1024)
            }
            12 => {
                self.ui.preview_max_height = self.ui.preview_max_height.saturating_add(64).min(1024)
            }
            14 => self.backend.zmq_port = self.backend.zmq_port.saturating_add(1),
            15 => self.backend.timeout_secs = self.backend.timeout_secs.saturating_add(5).min(300),
            16 => {
                self.backend.retry_attempts = self.backend.retry_attempts.saturating_add(1).min(10)
            }
            _ => {}
        }
    }

    /// Decrement a numeric setting.
    pub fn decrement_value(&mut self) {
        match self.selected_index {
            1 => {
                self.generation.default_steps =
                    self.generation.default_steps.saturating_sub(5).max(10)
            }
            2 => {
                self.generation.default_cfg_scale =
                    (self.generation.default_cfg_scale - 0.5).max(1.0)
            }
            3 => {
                self.generation.default_size.0 =
                    self.generation.default_size.0.saturating_sub(64).max(256)
            }
            4 => {
                self.generation.default_size.1 =
                    self.generation.default_size.1.saturating_sub(64).max(256)
            }
            6 => {
                self.generation.default_batch_size =
                    self.generation.default_batch_size.saturating_sub(1).max(1)
            }
            8 => self.ui.fps_limit = self.ui.fps_limit.saturating_sub(10).max(10),
            11 => self.ui.preview_max_width = self.ui.preview_max_width.saturating_sub(64).max(128),
            12 => {
                self.ui.preview_max_height = self.ui.preview_max_height.saturating_sub(64).max(128)
            }
            14 => self.backend.zmq_port = self.backend.zmq_port.saturating_sub(1).max(1024),
            15 => self.backend.timeout_secs = self.backend.timeout_secs.saturating_sub(5).max(5),
            16 => {
                self.backend.retry_attempts = self.backend.retry_attempts.saturating_sub(1).max(0)
            }
            _ => {}
        }
    }

    /// Get the name of the currently selected setting.
    pub fn selected_setting_name(&self) -> &'static str {
        match self.selected_index {
            0 => "Default Model",
            1 => "Default Steps",
            2 => "Default CFG Scale",
            3 => "Default Width",
            4 => "Default Height",
            5 => "Default Sampler",
            6 => "Default Batch Size",
            7 => "Theme",
            8 => "FPS Limit",
            9 => "Auto Refresh Gallery",
            10 => "Show Image Previews",
            11 => "Preview Max Width",
            12 => "Preview Max Height",
            13 => "ZMQ Host",
            14 => "ZMQ Port",
            15 => "Timeout (seconds)",
            16 => "Retry Attempts",
            17 => "Output Directory",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = SettingsState::default();
        assert_eq!(settings.generation.default_steps, 30);
        assert_eq!(settings.ui.fps_limit, 60);
        assert_eq!(settings.backend.zmq_port, 5555);
    }

    #[test]
    fn test_navigation() {
        let mut settings = SettingsState::default();
        assert_eq!(settings.selected_index, 0);

        settings.next_setting();
        assert_eq!(settings.selected_index, 1);

        settings.previous_setting();
        assert_eq!(settings.selected_index, 0);

        // Should not go below 0
        settings.previous_setting();
        assert_eq!(settings.selected_index, 0);
    }

    #[test]
    fn test_toggle_boolean() {
        let mut settings = SettingsState::default();
        settings.selected_index = 9; // auto_refresh_gallery

        let initial = settings.ui.auto_refresh_gallery;
        settings.toggle_boolean();
        assert_eq!(settings.ui.auto_refresh_gallery, !initial);
    }

    #[test]
    fn test_increment_decrement() {
        let mut settings = SettingsState::default();
        settings.selected_index = 1; // default_steps

        let initial = settings.generation.default_steps;
        settings.increment_value();
        assert_eq!(settings.generation.default_steps, initial + 5);

        settings.decrement_value();
        assert_eq!(settings.generation.default_steps, initial);
    }

    #[test]
    fn test_edit_flow() {
        let mut settings = SettingsState::default();
        settings.selected_index = 1; // default_steps

        // Start editing
        settings.start_editing();
        assert!(settings.is_editing);
        assert_eq!(settings.edit_buffer, "30");

        // Modify buffer
        settings.edit_buffer = "50".to_string();

        // Finish editing
        let result = settings.finish_editing();
        assert!(result.is_ok());
        assert!(!settings.is_editing);
        assert_eq!(settings.generation.default_steps, 50);
    }

    #[test]
    fn test_cancel_editing() {
        let mut settings = SettingsState::default();
        settings.selected_index = 1; // default_steps

        let initial = settings.generation.default_steps;

        settings.start_editing();
        settings.edit_buffer = "999".to_string();
        settings.cancel_editing();

        assert!(!settings.is_editing);
        assert_eq!(settings.generation.default_steps, initial);
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut settings = SettingsState::default();
        settings.generation.default_steps = 99;
        settings.ui.fps_limit = 30;

        settings.reset_to_defaults();

        assert_eq!(settings.generation.default_steps, 30);
        assert_eq!(settings.ui.fps_limit, 60);
    }

    #[test]
    fn test_serialization() {
        let settings = SettingsState::default();
        let toml_str = toml::to_string(&settings).unwrap();
        let deserialized: SettingsState = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            settings.generation.default_steps,
            deserialized.generation.default_steps
        );
        assert_eq!(settings.ui.fps_limit, deserialized.ui.fps_limit);
    }
}
