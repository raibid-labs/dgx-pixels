//! # Preview Manager Resource
//!
//! Bevy resource wrapper for the Sixel preview manager.
//! Handles async image loading and caching for preview display.

use bevy::prelude::*;
use std::sync::Arc;

use crate::sixel::PreviewManager as SixelPreviewManager;

/// Bevy resource for preview management.
#[derive(Resource)]
pub struct PreviewManagerResource {
    /// Inner preview manager (thread-safe)
    pub manager: Arc<SixelPreviewManager>,
}

impl PreviewManagerResource {
    /// Create a new preview manager resource.
    pub fn new() -> Self {
        Self {
            manager: Arc::new(SixelPreviewManager::new()),
        }
    }
}

impl Default for PreviewManagerResource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_manager_resource_creation() {
        let resource = PreviewManagerResource::new();
        assert!(Arc::strong_count(&resource.manager) >= 1);
    }
}
