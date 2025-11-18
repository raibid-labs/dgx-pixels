//! # Image Rendering Utilities
//!
//! Converts Bevy Image assets to terminal-renderable formats.
//! Provides ASCII/Unicode fallback for terminals without image support.

use bevy::prelude::*;
use ratatui::text::{Line, Span};
use tracing::{debug, warn};

/// ASCII characters for image rendering (brightness levels).
const ASCII_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// Convert Bevy Image to ASCII art representation.
///
/// This is a fallback for terminals that don't support Sixel or other
/// image protocols. Uses brightness-based ASCII characters.
pub fn render_image_to_ascii(
    image: &Image,
    width: usize,
    height: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    debug!(
        "Rendering image to ASCII: {}x{} -> {}x{}",
        image.width(),
        image.height(),
        width,
        height
    );

    // Get image data
    let img_data = &image.data;
    let img_width = image.width() as usize;
    let img_height = image.height() as usize;

    // Calculate sampling ratios
    let x_ratio = img_width as f32 / width as f32;
    let y_ratio = img_height as f32 / height as f32;

    for y in 0..height {
        let mut chars = Vec::new();

        for x in 0..width {
            // Sample corresponding pixel
            let sample_x = (x as f32 * x_ratio) as usize;
            let sample_y = (y as f32 * y_ratio) as usize;

            let brightness = sample_pixel_brightness(img_data, sample_x, sample_y, img_width);
            let ascii_char = brightness_to_ascii(brightness);

            chars.push(ascii_char);
        }

        let line_str: String = chars.into_iter().collect();
        lines.push(Line::from(Span::raw(line_str)));
    }

    lines
}

/// Sample pixel brightness from image data.
///
/// Assumes RGBA format with 4 bytes per pixel.
fn sample_pixel_brightness(data: &[u8], x: usize, y: usize, width: usize) -> u8 {
    let idx = (y * width + x) * 4; // RGBA = 4 bytes

    if idx + 3 >= data.len() {
        return 0; // Out of bounds
    }

    let r = data[idx] as u32;
    let g = data[idx + 1] as u32;
    let b = data[idx + 2] as u32;
    let a = data[idx + 3] as u32;

    // Weighted brightness calculation with alpha
    let brightness = ((r * 299 + g * 587 + b * 114) / 1000) * a / 255;
    brightness as u8
}

/// Convert brightness (0-255) to ASCII character.
fn brightness_to_ascii(brightness: u8) -> char {
    let index = (brightness as usize * (ASCII_CHARS.len() - 1)) / 255;
    ASCII_CHARS[index]
}

/// Render image with Unicode block characters (better quality).
pub fn render_image_to_unicode(
    image: &Image,
    width: usize,
    height: usize,
) -> Vec<Line<'static>> {
    debug!(
        "Rendering image to Unicode: {}x{} -> {}x{}",
        image.width(),
        image.height(),
        width,
        height
    );

    let mut lines = Vec::new();

    let img_data = &image.data;
    let img_width = image.width() as usize;
    let img_height = image.height() as usize;

    let x_ratio = img_width as f32 / width as f32;
    let y_ratio = img_height as f32 / height as f32;

    for y in 0..height {
        let mut chars = Vec::new();

        for x in 0..width {
            let sample_x = (x as f32 * x_ratio) as usize;
            let sample_y = (y as f32 * y_ratio) as usize;

            let brightness = sample_pixel_brightness(img_data, sample_x, sample_y, img_width);
            let block_char = brightness_to_block(brightness);

            chars.push(block_char);
        }

        let line_str: String = chars.into_iter().collect();
        lines.push(Line::from(Span::raw(line_str)));
    }

    lines
}

/// Convert brightness to Unicode block character.
fn brightness_to_block(brightness: u8) -> char {
    const BLOCKS: &[char] = &[' ', '░', '▒', '▓', '█'];

    let index = (brightness as usize * (BLOCKS.len() - 1)) / 255;
    BLOCKS[index]
}

/// Render image info as text (fallback when image can't be loaded).
pub fn render_image_placeholder(path: &std::path::Path, error: Option<&str>) -> Vec<Line<'static>> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string(); // Convert to owned String

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::raw("[Image Preview]".to_string())),
        Line::from(""),
        Line::from(Span::raw(filename)),
        Line::from(""),
    ];

    if let Some(err) = error {
        warn!("Image loading error for {:?}: {}", path, err);
        lines.push(Line::from(Span::raw(format!("Error: {}", err))));
    } else {
        lines.push(Line::from(Span::raw("Loading...".to_string())));
    }

    lines
}

/// Calculate optimal dimensions for ASCII rendering.
pub fn calculate_ascii_dimensions(
    image_width: u32,
    image_height: u32,
    max_width: u16,
    max_height: u16,
) -> (usize, usize) {
    let img_aspect = image_width as f32 / image_height as f32;
    let term_aspect = max_width as f32 / max_height as f32;

    // Account for character aspect ratio (chars are taller than wide)
    let char_aspect = 0.5; // Typical terminal char is about 2:1 height:width

    let (width, height) = if img_aspect > term_aspect * char_aspect {
        // Image is wider, fit to width
        let w = max_width as usize;
        let h = (w as f32 / img_aspect / char_aspect) as usize;
        (w, h.min(max_height as usize))
    } else {
        // Image is taller, fit to height
        let h = max_height as usize;
        let w = (h as f32 * img_aspect * char_aspect) as usize;
        (w.min(max_width as usize), h)
    };

    debug!(
        "Calculated ASCII dimensions: {}x{} -> {}x{}",
        image_width, image_height, width, height
    );

    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_to_ascii() {
        assert_eq!(brightness_to_ascii(0), ' ');
        assert_eq!(brightness_to_ascii(255), '@');
        assert_eq!(brightness_to_ascii(127), ASCII_CHARS[ASCII_CHARS.len() / 2]);
    }

    #[test]
    fn test_brightness_to_block() {
        assert_eq!(brightness_to_block(0), ' ');
        assert_eq!(brightness_to_block(255), '█');
    }

    #[test]
    fn test_calculate_ascii_dimensions() {
        // Square image
        let (w, h) = calculate_ascii_dimensions(100, 100, 80, 40);
        assert!(w <= 80);
        assert!(h <= 40);

        // Wide image
        let (w, h) = calculate_ascii_dimensions(200, 100, 80, 40);
        assert!(w <= 80);
        assert!(h <= 40);

        // Tall image
        let (w, h) = calculate_ascii_dimensions(100, 200, 80, 40);
        assert!(w <= 80);
        assert!(h <= 40);
    }

    #[test]
    fn test_sample_pixel_brightness() {
        // Create test pixel data (white pixel)
        let data = vec![255u8, 255, 255, 255];
        let brightness = sample_pixel_brightness(&data, 0, 0, 1);
        assert_eq!(brightness, 255);

        // Black pixel
        let data = vec![0u8, 0, 0, 255];
        let brightness = sample_pixel_brightness(&data, 0, 0, 1);
        assert_eq!(brightness, 0);

        // Transparent pixel
        let data = vec![255u8, 255, 255, 0];
        let brightness = sample_pixel_brightness(&data, 0, 0, 1);
        assert_eq!(brightness, 0);
    }
}
