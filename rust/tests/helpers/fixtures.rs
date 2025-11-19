//! Test fixtures for common test data

use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

/// Create a temporary directory with test images
pub fn create_test_gallery() -> (TempDir, Vec<PathBuf>) {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut paths = Vec::new();

    // Create test image files
    for i in 1..=5 {
        let filename = format!("sprite_{:03}.png", i);
        let path = dir.path().join(&filename);

        // Create a simple test image
        create_test_image(&path, 64, 64);
        paths.push(path);
    }

    (dir, paths)
}

/// Create a test image file at the given path
pub fn create_test_image(path: &std::path::Path, width: u32, height: u32) {
    use image::{ImageBuffer, Rgb};

    // Create a simple gradient image
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let r = ((x as f32 / width as f32) * 255.0) as u8;
        let g = ((y as f32 / height as f32) * 255.0) as u8;
        let b = 128;
        Rgb([r, g, b])
    });

    img.save(path).expect("Failed to save test image");
}

/// Create a corrupt image file (for error testing)
pub fn create_corrupt_image(path: &std::path::Path) {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path).expect("Failed to create file");
    file.write_all(b"NOT A VALID IMAGE FILE")
        .expect("Failed to write corrupt data");
}

/// Create a test job ID
pub fn test_job_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Create multiple test job IDs
pub fn test_job_ids(count: usize) -> Vec<String> {
    (0..count).map(|_| test_job_id()).collect()
}
