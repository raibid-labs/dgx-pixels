/// Preview management with caching and async loading

use anyhow::{Context, Result};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::fmt;
use tokio::sync::mpsc;
use image::GenericImageView;
use tracing::{debug, info, warn};

use super::image_renderer::{ImageRenderer, RenderOptions};
use super::MAX_CACHE_SIZE_MB;

/// Preview entry with metadata
#[derive(Debug, Clone)]
pub struct PreviewEntry {
    /// Path to the image file
    pub path: PathBuf,
    /// Rendered Sixel string
    pub sixel_data: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// When it was last accessed
    pub last_access: Instant,
    /// Image dimensions
    pub dimensions: (u32, u32),
}

/// Preview manager handles image caching and rendering
pub struct PreviewManager {
    /// Image renderer
    renderer: Arc<ImageRenderer>,
    /// Preview cache (path -> PreviewEntry)
    cache: Arc<DashMap<PathBuf, PreviewEntry>>,
    /// Current cache size in bytes
    cache_size: Arc<RwLock<usize>>,
    /// Maximum cache size in bytes
    max_cache_size: usize,
    /// Channel for async preview requests
    request_tx: mpsc::UnboundedSender<PreviewRequest>,
    /// Channel for preview results
    result_rx: Arc<RwLock<mpsc::UnboundedReceiver<PreviewResult>>>,
}

// Manual Debug implementation for PreviewManager
impl fmt::Debug for PreviewManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreviewManager")
            .field("cache_entries", &self.cache.len())
            .field("cache_size", &*self.cache_size.read())
            .field("max_cache_size", &self.max_cache_size)
            .finish()
    }
}

/// Request for preview rendering
#[derive(Debug)]
struct PreviewRequest {
    path: PathBuf,
    options: RenderOptions,
}

/// Result of preview rendering
#[derive(Debug)]
pub struct PreviewResult {
    pub path: PathBuf,
    pub entry: Option<PreviewEntry>,
    pub error: Option<String>,
}

impl PreviewManager {
    /// Create a new preview manager
    pub fn new() -> Self {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (result_tx, result_rx) = mpsc::unbounded_channel();

        let renderer = Arc::new(ImageRenderer::new());
        let cache = Arc::new(DashMap::new());
        let cache_size = Arc::new(RwLock::new(0));

        // Spawn worker task for async preview rendering
        let worker_renderer = Arc::clone(&renderer);
        let worker_cache = Arc::clone(&cache);
        let worker_cache_size = Arc::clone(&cache_size);

        tokio::spawn(async move {
            Self::preview_worker(
                request_rx,
                result_tx,
                worker_renderer,
                worker_cache,
                worker_cache_size,
            )
            .await;
        });

        info!("Preview manager initialized with {}MB cache", MAX_CACHE_SIZE_MB);

        Self {
            renderer,
            cache,
            cache_size,
            max_cache_size: MAX_CACHE_SIZE_MB * 1024 * 1024,
            request_tx,
            result_rx: Arc::new(RwLock::new(result_rx)),
        }
    }

    /// Request a preview (async, returns immediately)
    pub fn request_preview(&self, path: PathBuf, options: RenderOptions) -> Result<()> {
        // Check cache first
        if let Some(entry) = self.cache.get(&path) {
            debug!("Preview cache hit: {:?}", path);
            // Update access time
            let mut entry = entry.clone();
            entry.last_access = Instant::now();
            self.cache.insert(path, entry);
            return Ok(());
        }

        // Send async request
        self.request_tx
            .send(PreviewRequest { path, options })
            .context("Failed to send preview request")
    }

    /// Try to get a preview from cache (non-blocking)
    pub fn get_preview(&self, path: &Path) -> Option<PreviewEntry> {
        self.cache.get(path).map(|entry| {
            let mut entry = entry.clone();
            entry.last_access = Instant::now();
            entry
        })
    }

    /// Check if preview is available in cache
    pub fn has_preview(&self, path: &Path) -> bool {
        self.cache.contains_key(path)
    }

    /// Try to receive completed preview results (non-blocking)
    pub fn try_recv_result(&self) -> Option<PreviewResult> {
        self.result_rx.write().try_recv().ok()
    }

    /// Clear the entire cache
    pub fn clear_cache(&self) {
        info!("Clearing preview cache");
        self.cache.clear();
        *self.cache_size.write() = 0;
    }

    /// Get current cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            size_bytes: *self.cache_size.read(),
            max_size_bytes: self.max_cache_size,
        }
    }

    /// Evict least recently used entries to make space
    fn evict_lru(&self, required_space: usize) {
        let mut current_size = *self.cache_size.read();

        if current_size + required_space <= self.max_cache_size {
            return; // No eviction needed
        }

        debug!(
            "Evicting LRU entries (need {} bytes, have {})",
            required_space, current_size
        );

        // Collect entries sorted by last access time
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().last_access, entry.value().size_bytes))
            .collect();

        entries.sort_by_key(|(_, access, _)| *access);

        // Evict oldest entries until we have enough space
        let target_size = self.max_cache_size - required_space;

        for (path, _, size) in entries {
            if current_size <= target_size {
                break;
            }

            self.cache.remove(&path);
            current_size -= size;
            debug!("Evicted: {:?} ({} bytes)", path, size);
        }

        *self.cache_size.write() = current_size;
    }

    /// Worker task for async preview rendering
    async fn preview_worker(
        mut request_rx: mpsc::UnboundedReceiver<PreviewRequest>,
        result_tx: mpsc::UnboundedSender<PreviewResult>,
        renderer: Arc<ImageRenderer>,
        cache: Arc<DashMap<PathBuf, PreviewEntry>>,
        cache_size: Arc<RwLock<usize>>,
    ) {
        info!("Preview worker started");

        while let Some(request) = request_rx.recv().await {
            debug!("Processing preview request: {:?}", request.path);

            // Spawn blocking task for image rendering
            let path = request.path.clone();
            let options = request.options.clone();
            let renderer = Arc::clone(&renderer);

            let result = tokio::task::spawn_blocking(move || {
                Self::render_preview_blocking(&renderer, &path, &options)
            })
            .await;

            let result = match result {
                Ok(Ok(entry)) => {
                    // Add to cache
                    let size = entry.size_bytes;
                    cache.insert(request.path.clone(), entry.clone());
                    *cache_size.write() += size;

                    PreviewResult {
                        path: request.path,
                        entry: Some(entry),
                        error: None,
                    }
                }
                Ok(Err(e)) => {
                    warn!("Preview rendering failed: {}", e);
                    PreviewResult {
                        path: request.path,
                        entry: None,
                        error: Some(e.to_string()),
                    }
                }
                Err(e) => {
                    warn!("Preview task panicked: {}", e);
                    PreviewResult {
                        path: request.path,
                        entry: None,
                        error: Some(format!("Task panic: {}", e)),
                    }
                }
            };

            if result_tx.send(result).is_err() {
                warn!("Failed to send preview result, receiver dropped");
                break;
            }
        }

        info!("Preview worker stopped");
    }

    /// Render preview in blocking context
    fn render_preview_blocking(
        renderer: &ImageRenderer,
        path: &Path,
        options: &RenderOptions,
    ) -> Result<PreviewEntry> {
        let start = Instant::now();

        // Load and get dimensions
        let img = image::open(path).context("Failed to open image")?;
        let (width, height) = img.dimensions();
        let dimensions = (width, height);

        // Render to Sixel
        let sixel_data = renderer.render_image(path, options)?;
        let size_bytes = sixel_data.len();

        let duration = start.elapsed();
        debug!(
            "Rendered preview in {:?} ({} bytes): {:?}",
            duration, size_bytes, path
        );

        Ok(PreviewEntry {
            path: path.to_path_buf(),
            sixel_data,
            size_bytes,
            last_access: Instant::now(),
            dimensions,
        })
    }
}

impl Default for PreviewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub size_bytes: usize,
    pub max_size_bytes: usize,
}

impl CacheStats {
    /// Get cache usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.max_size_bytes == 0 {
            0.0
        } else {
            (self.size_bytes as f32 / self.max_size_bytes as f32) * 100.0
        }
    }

    /// Get size in MB
    pub fn size_mb(&self) -> f32 {
        self.size_bytes as f32 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preview_manager_creation() {
        let manager = PreviewManager::new();
        let stats = manager.cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.size_bytes, 0);
    }

    #[test]
    fn test_cache_stats_usage() {
        let stats = CacheStats {
            entries: 10,
            size_bytes: 25 * 1024 * 1024, // 25 MB
            max_size_bytes: 50 * 1024 * 1024, // 50 MB
        };

        assert_eq!(stats.usage_percent(), 50.0);
        assert!((stats.size_mb() - 25.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_has_preview_empty() {
        let manager = PreviewManager::new();
        let path = PathBuf::from("/nonexistent/image.png");
        assert!(!manager.has_preview(&path));
    }
}
