//! Image to Sixel conversion and rendering

use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;
use tracing::{debug, warn};
use viuer::Config;

/// Options for image rendering
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Target width in terminal cells
    pub width: u16,
    /// Target height in terminal cells
    pub height: u16,
    /// Whether to maintain aspect ratio
    pub preserve_aspect: bool,
    /// Whether to use high quality resizing
    pub high_quality: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 40,
            height: 20,
            preserve_aspect: true,
            high_quality: true,
        }
    }
}

/// Image renderer using Sixel protocol
pub struct ImageRenderer {
    /// Sixel configuration
    _config: Config,
}

impl ImageRenderer {
    /// Create a new image renderer
    pub fn new() -> Result<Self> {
        // Validate img2sixel is available
        let check = Command::new("img2sixel")
            .arg("--version")
            .output();

        match check {
            Ok(output) if output.status.success() => {
                debug!("img2sixel found: {:?}",
                    String::from_utf8_lossy(&output.stdout));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "img2sixel not found in PATH. Install with: apt install libsixel-bin"
                ));
            }
        }

        Ok(Self { _config: Config::default() })
    }

    /// Render an image file to the terminal using Sixel
    ///
    /// Returns the rendered Sixel string that can be written to terminal
    pub fn render_image(&self, image_path: &Path, options: &RenderOptions) -> Result<String> {
        debug!("Rendering image: {:?}", image_path);

        // Load image
        let img = image::open(image_path)
            .with_context(|| format!("Failed to load image: {:?}", image_path))?;

        // Resize if needed
        let img = self.resize_image(img, options)?;

        // Convert to Sixel using viuer
        let mut buffer = Vec::new();
        self.render_to_buffer(&img, &mut buffer, options)?;

        String::from_utf8(buffer).context("Failed to convert Sixel data to UTF-8")
    }

    /// Render image directly to stdout
    #[allow(dead_code)]
    pub fn render_to_stdout(&self, image_path: &Path, options: &RenderOptions) -> Result<()> {
        debug!("Rendering to stdout: {:?}", image_path);

        let img = image::open(image_path)?;
        let img = self.resize_image(img, options)?;

        let config = viuer::Config {
            width: Some(options.width as u32),
            height: Some(options.height as u32),
            transparent: false,
            absolute_offset: false,
            ..Default::default()
        };

        viuer::print(&img, &config).context("Failed to render image with viuer")?;

        Ok(())
    }

    /// Render a thumbnail (smaller, faster)
    #[allow(dead_code)]
    pub fn render_thumbnail(&self, image_path: &Path, size: u16) -> Result<String> {
        debug!("Rendering thumbnail: {:?} (size: {})", image_path, size);

        let options = RenderOptions {
            width: size,
            height: size,
            preserve_aspect: true,
            high_quality: false, // Fast mode for thumbnails
        };

        self.render_image(image_path, &options)
    }

    /// Resize image according to options
    fn resize_image(&self, img: DynamicImage, options: &RenderOptions) -> Result<DynamicImage> {
        let (orig_width, orig_height) = img.dimensions();

        let target_width = options.width as u32 * 8; // Approximate pixels per cell
        let target_height = options.height as u32 * 16;

        // Skip resize if already close to target size
        if orig_width <= target_width && orig_height <= target_height {
            return Ok(img);
        }

        let (new_width, new_height) = if options.preserve_aspect {
            let aspect = orig_width as f32 / orig_height as f32;
            let target_aspect = target_width as f32 / target_height as f32;

            if aspect > target_aspect {
                // Width-constrained
                let width = target_width;
                let height = (target_width as f32 / aspect) as u32;
                (width, height)
            } else {
                // Height-constrained
                let width = (target_height as f32 * aspect) as u32;
                let height = target_height;
                (width, height)
            }
        } else {
            (target_width, target_height)
        };

        debug!(
            "Resizing from {}x{} to {}x{}",
            orig_width, orig_height, new_width, new_height
        );

        let filter = if options.high_quality {
            FilterType::Lanczos3
        } else {
            FilterType::Triangle
        };

        Ok(img.resize(new_width, new_height, filter))
    }

    /// Render image to a buffer
    fn render_to_buffer(
        &self,
        img: &DynamicImage,
        buffer: &mut Vec<u8>,
        _options: &RenderOptions,
    ) -> Result<()> {
        // Convert image to RGB8
        let rgb_img = img.to_rgb8();

        // Create a temporary file
        let temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
        let temp_path = temp_file.path();

        // Save the image to the temporary file as PNG
        rgb_img
            .save_with_format(temp_path, image::ImageFormat::Png)
            .context("Failed to save image to temporary file")?;

        // Execute img2sixel to convert the PNG to Sixel
        let output = Command::new("img2sixel")
            .arg(temp_path)
            .output()
            .context("Failed to execute img2sixel. Is it installed and in PATH?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("img2sixel failed: {}", stderr);
            return Err(anyhow::anyhow!("img2sixel failed: {}", stderr));
        }

        // Write the Sixel output to our buffer
        buffer.extend_from_slice(&output.stdout);

        debug!("Successfully encoded image to Sixel using img2sixel");

        Ok(())
    }
}

impl Default for ImageRenderer {
    fn default() -> Self {
        Self::new().expect("img2sixel must be installed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_options_default() {
        let opts = RenderOptions::default();
        assert_eq!(opts.width, 40);
        assert_eq!(opts.height, 20);
        assert!(opts.preserve_aspect);
        assert!(opts.high_quality);
    }

    #[test]
    fn test_image_renderer_creation() {
        let _renderer = ImageRenderer::new();
        // Just verify it can be created (or fails gracefully if img2sixel not installed)
    }

    #[test]
    fn test_resize_aspect_ratio() {
        // Skip test if img2sixel not installed
        if ImageRenderer::new().is_err() {
            return;
        }
        let renderer = ImageRenderer::new().unwrap();

        // Create a test image (100x200)
        let img = DynamicImage::new_rgb8(100, 200);

        let options = RenderOptions {
            width: 10,
            height: 10,
            preserve_aspect: true,
            high_quality: false,
        };

        let resized = renderer.resize_image(img, &options).unwrap();
        let (w, h) = resized.dimensions();

        // Should maintain aspect ratio (1:2)
        assert!(h >= w);
    }
}
