//! # Sixel Image Renderer
//!
//! Converts Bevy Image assets to Sixel protocol format for terminal display.
//! Port of the original sixel/image_renderer.rs to work with Bevy ECS.

use anyhow::{Context, Result};
use bevy::prelude::*;
use image::{imageops::FilterType, DynamicImage, RgbaImage};
use std::process::Command;
use tempfile::NamedTempFile;
use tracing::{debug, warn};

/// Maximum colors for Sixel (256 for best terminal compatibility)
pub const MAX_SIXEL_COLORS: usize = 256;

/// Options for Sixel rendering
#[derive(Debug, Clone)]
pub struct SixelRenderOptions {
    /// Target width in terminal cells
    pub width: u16,
    /// Target height in terminal cells
    pub height: u16,
    /// Whether to maintain aspect ratio
    pub preserve_aspect: bool,
    /// Whether to use high quality resizing
    pub high_quality: bool,
}

impl Default for SixelRenderOptions {
    fn default() -> Self {
        Self {
            width: 40,
            height: 20,
            preserve_aspect: true,
            high_quality: true,
        }
    }
}

/// Check if Sixel rendering is supported
///
/// Checks for img2sixel binary in PATH
pub fn supports_sixel() -> bool {
    Command::new("img2sixel")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Render a Bevy Image to Sixel string
///
/// This is the main entry point for converting Bevy images to Sixel format.
///
/// # Arguments
/// * `image` - Bevy Image asset to convert
/// * `options` - Rendering options (size, quality, etc.)
///
/// # Returns
/// Sixel-encoded string that can be written to terminal
pub fn render_image_sixel(
    image: &Image,
    options: &SixelRenderOptions,
) -> Result<String> {
    debug!(
        "Rendering Bevy image to Sixel: {}x{} -> {}x{} cells",
        image.width(),
        image.height(),
        options.width,
        options.height
    );

    // Convert Bevy Image to DynamicImage
    let dynamic_img = bevy_image_to_dynamic(image)?;

    // Resize if needed
    let resized = resize_image(dynamic_img, options)?;

    // Convert to Sixel using img2sixel
    let sixel_data = encode_to_sixel(&resized)?;

    debug!("Successfully encoded image to Sixel ({} bytes)", sixel_data.len());

    Ok(sixel_data)
}

/// Convert Bevy Image to image crate's DynamicImage
fn bevy_image_to_dynamic(bevy_img: &Image) -> Result<DynamicImage> {
    let width = bevy_img.width();
    let height = bevy_img.height();
    let data = &bevy_img.data;

    // Bevy uses RGBA8 format
    let rgba_img = RgbaImage::from_raw(width, height, data.clone())
        .context("Failed to create RgbaImage from Bevy Image data")?;

    Ok(DynamicImage::ImageRgba8(rgba_img))
}

/// Resize image according to options
fn resize_image(
    img: DynamicImage,
    options: &SixelRenderOptions,
) -> Result<DynamicImage> {
    let (orig_width, orig_height) = (img.width(), img.height());

    // Terminal cells are approximately 8x16 pixels
    let target_width = options.width as u32 * 8;
    let target_height = options.height as u32 * 16;

    // Skip resize if already close to target size
    if orig_width <= target_width && orig_height <= target_height {
        return Ok(img);
    }

    let (new_width, new_height) = if options.preserve_aspect {
        calculate_aspect_preserving_dimensions(
            orig_width,
            orig_height,
            target_width,
            target_height,
        )
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

/// Calculate dimensions that preserve aspect ratio
fn calculate_aspect_preserving_dimensions(
    orig_width: u32,
    orig_height: u32,
    target_width: u32,
    target_height: u32,
) -> (u32, u32) {
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
}

/// Encode DynamicImage to Sixel using img2sixel command
fn encode_to_sixel(img: &DynamicImage) -> Result<String> {
    // Convert to RGB8 (Sixel doesn't need alpha)
    let rgb_img = img.to_rgb8();

    // Create temporary file for PNG
    let temp_file = NamedTempFile::new()
        .context("Failed to create temporary file")?;
    let temp_path = temp_file.path();

    // Save as PNG
    rgb_img
        .save_with_format(temp_path, image::ImageFormat::Png)
        .context("Failed to save image to temporary file")?;

    // Execute img2sixel
    let output = Command::new("img2sixel")
        .arg(temp_path)
        .output()
        .context("Failed to execute img2sixel. Is it installed and in PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("img2sixel failed: {}", stderr);
        return Err(anyhow::anyhow!("img2sixel failed: {}", stderr));
    }

    // Convert stdout to String
    String::from_utf8(output.stdout)
        .context("Failed to convert img2sixel output to UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sixel_render_options_default() {
        let opts = SixelRenderOptions::default();
        assert_eq!(opts.width, 40);
        assert_eq!(opts.height, 20);
        assert!(opts.preserve_aspect);
        assert!(opts.high_quality);
    }

    #[test]
    fn test_calculate_aspect_preserving_dimensions() {
        // Square image
        let (w, h) = calculate_aspect_preserving_dimensions(100, 100, 80, 160);
        assert_eq!(w, 80);
        assert_eq!(h, 80);

        // Wide image (2:1 aspect)
        let (w, h) = calculate_aspect_preserving_dimensions(200, 100, 80, 160);
        assert_eq!(w, 80);
        assert_eq!(h, 40);

        // Tall image (1:2 aspect)
        let (w, h) = calculate_aspect_preserving_dimensions(100, 200, 80, 160);
        assert_eq!(w, 80);
        assert_eq!(h, 160);
    }

    #[test]
    fn test_bevy_image_to_dynamic() {
        // Create a simple 2x2 RGBA image
        let data = vec![
            255, 0, 0, 255,    // Red pixel
            0, 255, 0, 255,    // Green pixel
            0, 0, 255, 255,    // Blue pixel
            255, 255, 255, 255, // White pixel
        ];

        let bevy_img = Image::new(
            bevy::render::render_resource::Extent3d {
                width: 2,
                height: 2,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::render::texture::RenderAssetUsages::default(),
        );

        let dynamic_img = bevy_image_to_dynamic(&bevy_img);
        assert!(dynamic_img.is_ok());

        let img = dynamic_img.unwrap();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
    }

    #[test]
    fn test_supports_sixel() {
        // This test just verifies the function runs without panicking
        // Actual result depends on system configuration
        let _result = supports_sixel();
    }
}
