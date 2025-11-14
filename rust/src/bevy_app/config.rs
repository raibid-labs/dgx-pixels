use std::time::Duration;

/// Configuration for the Bevy app runtime.
#[derive(Debug, Clone)]
pub struct BevyAppConfig {
    /// Update rate (60 FPS = 16.67ms per frame)
    pub update_rate: Duration,
}

impl Default for BevyAppConfig {
    fn default() -> Self {
        Self {
            update_rate: Duration::from_secs_f32(1.0 / 60.0), // 60 FPS
        }
    }
}

impl BevyAppConfig {
    /// Create config with custom update rate.
    pub fn with_update_rate(mut self, fps: u32) -> Self {
        self.update_rate = Duration::from_secs_f32(1.0 / fps as f32);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BevyAppConfig::default();
        assert_eq!(config.update_rate, Duration::from_secs_f32(1.0 / 60.0));
    }

    #[test]
    fn test_custom_fps() {
        let config = BevyAppConfig::default().with_update_rate(30);
        assert_eq!(config.update_rate, Duration::from_secs_f32(1.0 / 30.0));
    }
}
