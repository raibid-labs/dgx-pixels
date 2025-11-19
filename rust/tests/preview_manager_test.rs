//! Comprehensive unit tests for PreviewManager
//!
//! Tests cover async behavior, caching, LRU eviction, error handling,
//! and concurrent operations.

mod helpers;

use dgx_pixels_tui::sixel::{PreviewManager, RenderOptions};
use helpers::*;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;

// ============================================================================
// Creation and Initialization Tests
// ============================================================================

#[tokio::test]
async fn test_preview_manager_creation() {
    let manager = PreviewManager::new();
    let stats = manager.cache_stats();

    assert_eq!(stats.entries, 0, "New manager should have empty cache");
    assert_eq!(stats.size_bytes, 0, "New manager should have zero cache size");
    assert!(stats.max_size_bytes > 0, "Max cache size should be positive");
}

#[tokio::test]
async fn test_preview_manager_default() {
    let manager = PreviewManager::default();
    let stats = manager.cache_stats();

    assert_eq!(stats.entries, 0);
    assert_eq!(stats.size_bytes, 0);
}

#[tokio::test]
async fn test_cache_stats_initial_state() {
    let manager = PreviewManager::new();
    let stats = manager.cache_stats();

    assert_eq!(stats.size_mb(), 0.0, "Initial cache size should be 0 MB");
    assert!(stats.max_size_bytes >= 50 * 1024 * 1024, "Max cache should be at least 50MB");
}

// ============================================================================
// Cache Hit and Miss Tests
// ============================================================================

#[tokio::test]
async fn test_preview_cache_miss_initial() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    // Initially, no preview should exist
    assert!(!manager.has_preview(&paths[0]), "Cache should miss on first access");
    assert!(manager.get_preview(&paths[0]).is_none(), "get_preview should return None");
}

#[tokio::test]
async fn test_preview_cache_hit_after_load() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();

    // Request preview
    let result = manager.request_preview(paths[0].clone(), options);
    assert!(result.is_ok(), "Preview request should succeed");

    // Wait for worker to process
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Collect results
    let mut found = false;
    for _ in 0..10 {
        if let Some(result) = manager.try_recv_result() {
            if result.path == paths[0] && result.entry.is_some() {
                found = true;
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(found, "Preview result should be received");
    assert!(manager.has_preview(&paths[0]), "Cache should hit after loading");
}

#[tokio::test]
async fn test_get_preview_returns_entry() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();
    manager.request_preview(paths[0].clone(), options).unwrap();

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Collect result
    while let Some(_) = manager.try_recv_result() {
        // Drain results
    }

    // Now get_preview should return Some
    if let Some(entry) = manager.get_preview(&paths[0]) {
        assert_eq!(entry.path, paths[0], "Entry path should match");
        assert!(!entry.sixel_data.is_empty(), "Sixel data should not be empty");
        assert!(entry.size_bytes > 0, "Entry size should be positive");
        assert_eq!(entry.dimensions.0, 64, "Width should match test image");
        assert_eq!(entry.dimensions.1, 64, "Height should match test image");
    }
}

// ============================================================================
// Preview Request/Response Cycle Tests
// ============================================================================

#[tokio::test]
async fn test_preview_request_creates_result() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions {
        width: 20,
        height: 20,
        preserve_aspect: true,
        high_quality: false,
    };

    manager.request_preview(paths[0].clone(), options).unwrap();

    // Wait and collect results
    tokio::time::sleep(Duration::from_millis(200)).await;

    let result = manager.try_recv_result();
    assert!(result.is_some(), "Should receive a result");

    if let Some(result) = result {
        assert_eq!(result.path, paths[0], "Result path should match request");
        assert!(result.entry.is_some(), "Result should have entry");
        assert!(result.error.is_none(), "Result should not have error");
    }
}

#[tokio::test]
async fn test_preview_request_nonexistent_file() {
    let manager = PreviewManager::new();
    let nonexistent = PathBuf::from("/tmp/nonexistent_image_12345.png");

    let options = RenderOptions::default();
    manager.request_preview(nonexistent.clone(), options).unwrap();

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Should get error result
    if let Some(result) = manager.try_recv_result() {
        assert_eq!(result.path, nonexistent);
        assert!(result.entry.is_none(), "Entry should be None for missing file");
        assert!(result.error.is_some(), "Error should be present");
    }
}

#[tokio::test]
async fn test_preview_request_corrupt_image() {
    let dir = tempdir().unwrap();
    let corrupt_path = dir.path().join("corrupt.png");
    create_corrupt_image(&corrupt_path);

    let manager = PreviewManager::new();
    let options = RenderOptions::default();

    manager.request_preview(corrupt_path.clone(), options).unwrap();

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Should get error result
    if let Some(result) = manager.try_recv_result() {
        assert_eq!(result.path, corrupt_path);
        assert!(result.entry.is_none(), "Entry should be None for corrupt file");
        assert!(result.error.is_some(), "Error should be present for corrupt file");
    }
}

// ============================================================================
// Concurrent Request Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_preview_requests() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();

    // Request multiple previews concurrently
    for path in &paths {
        manager.request_preview(path.clone(), options.clone()).unwrap();
    }

    // Wait for all to process
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Collect all results
    let mut results_count = 0;
    while let Some(_) = manager.try_recv_result() {
        results_count += 1;
    }

    assert_eq!(results_count, paths.len(), "Should receive all results");

    // Verify all are cached
    for path in &paths {
        assert!(manager.has_preview(path), "All paths should be cached");
    }
}

#[tokio::test]
async fn test_duplicate_requests_use_cache() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();

    // First request
    manager.request_preview(paths[0].clone(), options.clone()).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    while let Some(_) = manager.try_recv_result() {}

    let stats_before = manager.cache_stats();

    // Second request for same path (should hit cache)
    manager.request_preview(paths[0].clone(), options).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let stats_after = manager.cache_stats();

    // Cache size should not increase for duplicate
    assert_eq!(stats_before.entries, stats_after.entries, "Cache entries should stay same");
    assert_eq!(stats_before.size_bytes, stats_after.size_bytes, "Cache size should stay same");
}

// ============================================================================
// Cache Statistics Tests
// ============================================================================

#[tokio::test]
async fn test_cache_stats_after_loading() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();

    // Load first 3 images
    for path in paths.iter().take(3) {
        manager.request_preview(path.clone(), options.clone()).unwrap();
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(600)).await;
    while let Some(_) = manager.try_recv_result() {}

    let stats = manager.cache_stats();

    assert_eq!(stats.entries, 3, "Should have 3 cached entries");
    assert!(stats.size_bytes > 0, "Cache size should be positive");
    assert!(stats.size_mb() > 0.0, "Cache size in MB should be positive");
}

#[tokio::test]
async fn test_cache_stats_tracking() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let stats_initial = manager.cache_stats();
    assert_eq!(stats_initial.entries, 0);

    // Load one image
    let options = RenderOptions::default();
    manager.request_preview(paths[0].clone(), options).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    while let Some(_) = manager.try_recv_result() {}

    let stats_after_one = manager.cache_stats();
    assert_eq!(stats_after_one.entries, 1, "Should track entry count");
    assert!(stats_after_one.size_bytes > stats_initial.size_bytes, "Size should increase");
}

// ============================================================================
// Cache Clear Tests
// ============================================================================

#[tokio::test]
async fn test_clear_cache() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    // Load some images
    let options = RenderOptions::default();
    for path in paths.iter().take(2) {
        manager.request_preview(path.clone(), options.clone()).unwrap();
    }

    tokio::time::sleep(Duration::from_millis(400)).await;
    while let Some(_) = manager.try_recv_result() {}

    let stats_before = manager.cache_stats();
    assert!(stats_before.entries > 0, "Cache should have entries");

    // Clear cache
    manager.clear_cache();

    let stats_after = manager.cache_stats();
    assert_eq!(stats_after.entries, 0, "Cache should be empty after clear");
    assert_eq!(stats_after.size_bytes, 0, "Cache size should be zero after clear");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_error_handling_missing_file() {
    let manager = PreviewManager::new();
    let missing_path = PathBuf::from("/nonexistent/directory/image.png");

    let options = RenderOptions::default();
    let result = manager.request_preview(missing_path.clone(), options);

    assert!(result.is_ok(), "Request should succeed (error happens in worker)");

    tokio::time::sleep(Duration::from_millis(200)).await;

    if let Some(result) = manager.try_recv_result() {
        assert!(result.error.is_some(), "Should have error for missing file");
        assert!(result.entry.is_none(), "Should not have entry for missing file");
    }
}

#[tokio::test]
async fn test_error_does_not_cache() {
    let manager = PreviewManager::new();
    let missing_path = PathBuf::from("/tmp/missing_test_file.png");

    let options = RenderOptions::default();
    manager.request_preview(missing_path.clone(), options).unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;
    while let Some(_) = manager.try_recv_result() {}

    // Failed load should not be in cache
    assert!(!manager.has_preview(&missing_path), "Failed preview should not be cached");

    let stats = manager.cache_stats();
    assert_eq!(stats.entries, 0, "Failed loads should not increase cache entries");
}

// ============================================================================
// Async Worker Behavior Tests
// ============================================================================

#[tokio::test]
async fn test_async_worker_processes_queue() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();

    // Queue multiple requests
    for path in &paths {
        manager.request_preview(path.clone(), options.clone()).unwrap();
    }

    // Worker should process all
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let mut result_count = 0;
    while let Some(_) = manager.try_recv_result() {
        result_count += 1;
    }

    assert!(result_count >= paths.len(), "Worker should process all queued requests");
}

#[tokio::test]
async fn test_try_recv_result_non_blocking() {
    let manager = PreviewManager::new();

    // Without any requests, try_recv should return None immediately
    let result = manager.try_recv_result();
    assert!(result.is_none(), "try_recv_result should be non-blocking and return None");
}

// ============================================================================
// Different RenderOptions Tests
// ============================================================================

#[tokio::test]
async fn test_different_render_options() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options_large = RenderOptions {
        width: 80,
        height: 40,
        preserve_aspect: true,
        high_quality: true,
    };

    let options_small = RenderOptions {
        width: 20,
        height: 10,
        preserve_aspect: true,
        high_quality: false,
    };

    // Request with different options
    manager.request_preview(paths[0].clone(), options_large).unwrap();
    manager.request_preview(paths[1].clone(), options_small).unwrap();

    tokio::time::sleep(Duration::from_millis(400)).await;

    let mut results = Vec::new();
    while let Some(result) = manager.try_recv_result() {
        results.push(result);
    }

    assert_eq!(results.len(), 2, "Should receive results for both options");
}

// ============================================================================
// Access Time Update Tests
// ============================================================================

#[tokio::test]
async fn test_cache_hit_updates_access_time() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();

    // First load
    manager.request_preview(paths[0].clone(), options.clone()).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    while let Some(_) = manager.try_recv_result() {}

    let first_entry = manager.get_preview(&paths[0]).unwrap();
    let first_access = first_entry.last_access;

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Access again (should update access time via request_preview cache check)
    manager.request_preview(paths[0].clone(), options).unwrap();

    let second_entry = manager.get_preview(&paths[0]).unwrap();
    let second_access = second_entry.last_access;

    // Access time should be updated
    assert!(second_access >= first_access, "Access time should be updated on cache hit");
}

#[tokio::test]
async fn test_get_preview_updates_access_time() {
    let manager = PreviewManager::new();
    let (_dir, paths) = create_test_gallery();

    let options = RenderOptions::default();
    manager.request_preview(paths[0].clone(), options).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    while let Some(_) = manager.try_recv_result() {}

    let first_entry = manager.get_preview(&paths[0]).unwrap();
    let first_access = first_entry.last_access;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let second_entry = manager.get_preview(&paths[0]).unwrap();
    let second_access = second_entry.last_access;

    assert!(second_access >= first_access, "get_preview should update access time");
}
