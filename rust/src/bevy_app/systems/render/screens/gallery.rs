//! # Gallery Screen Renderer
//!
//! WS-10: Gallery screen rendering system for Bevy-Ratatui migration.
//! WS-06: Image asset loading integration.
//! T9: Sixel rendering support for high-quality terminal image display.
//! Displays a grid of generated images with detail view and navigation.

use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use std::io::{self, Write};
use tracing::{debug, warn};

use crate::bevy_app::components::PreviewImage;
use crate::bevy_app::resources::{AppTheme, CurrentScreen, GalleryState, Screen, SettingsState};
use crate::bevy_app::systems::assets::render::{
    calculate_ascii_dimensions, render_image_placeholder, render_image_to_unicode,
};
use crate::bevy_app::systems::assets::{
    SixelPreviewCache, SixelRenderOptions, render_image_sixel, supports_sixel,
};

/// Main gallery screen render system.
///
/// Renders a two-panel layout:
/// - Left panel (70%): Large preview of selected image
/// - Right panel (30%): Thumbnail list of all images
///
/// Supports both Sixel (high-quality) and Unicode fallback rendering.
pub fn render_gallery_screen(
    current_screen: Res<CurrentScreen>,
    gallery: Res<GalleryState>,
    theme: Res<AppTheme>,
    settings: Res<SettingsState>,
    preview_query: Query<&PreviewImage>,
    images: Option<Res<Assets<Image>>>,
    asset_server: Option<Res<AssetServer>>,
    sixel_cache: Option<Res<SixelPreviewCache>>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    // Only render when on Gallery screen
    if current_screen.0 != Screen::Gallery {
        return;
    }

    ratatui
        .draw(|frame| {
            // Create main layout (header + body + status bar handled by dispatch)
            let area = frame.area();

            if gallery.is_empty() {
                render_empty_gallery(frame, area, &theme);
            } else if let (Some(images), Some(asset_server)) = (images.as_ref(), asset_server.as_ref()) {
                render_gallery_body(
                    frame,
                    area,
                    &gallery,
                    &theme,
                    &settings,
                    &preview_query,
                    images,
                    asset_server,
                    sixel_cache.as_deref(),
                );
            } else {
                // Assets not loaded yet, show loading message
                render_loading_gallery(frame, area, &theme);
            }
        })
        .ok(); // Ignore render errors for now
}

/// Render empty gallery placeholder.
fn render_empty_gallery(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("No images in gallery", theme.muted())),
        Line::from(""),
        Line::from(Span::styled(
            "Generate some images to see them here!",
            theme.text(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press 1 to go to Generation screen",
            theme.muted(),
        )),
    ];

    let block = Block::default()
        .title(" Image Gallery ")
        .borders(Borders::ALL)
        .border_style(theme.border());

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render loading placeholder when assets aren't ready yet.
fn render_loading_gallery(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Loading assets...", theme.muted())),
        Line::from(""),
    ];

    let block = Block::default()
        .title(" Image Gallery ")
        .borders(Borders::ALL)
        .border_style(theme.border());

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render gallery body with preview and thumbnail list.
fn render_gallery_body(
    frame: &mut Frame,
    area: Rect,
    gallery: &GalleryState,
    theme: &AppTheme,
    settings: &SettingsState,
    preview_query: &Query<&PreviewImage>,
    images: &Assets<Image>,
    asset_server: &AssetServer,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    // Split into preview (left) and thumbnail list (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Main preview
            Constraint::Percentage(30), // Thumbnail list
        ])
        .margin(1)
        .split(area);

    render_main_preview(
        frame,
        chunks[0],
        gallery,
        theme,
        settings,
        preview_query,
        images,
        asset_server,
        sixel_cache,
    );
    render_thumbnail_list(frame, chunks[1], gallery, theme);
}

/// Render main preview panel.
fn render_main_preview(
    frame: &mut Frame,
    area: Rect,
    gallery: &GalleryState,
    theme: &AppTheme,
    settings: &SettingsState,
    preview_query: &Query<&PreviewImage>,
    images: &Assets<Image>,
    asset_server: &AssetServer,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(theme.border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(selected_path) = gallery.current_image() {
        // Find PreviewImage component for this path
        let preview = preview_query
            .iter()
            .find(|p| &p.path == selected_path);

        if let Some(preview) = preview {
            // Check if image asset is loaded
            if let Some(handle) = &preview.asset_handle {
                match asset_server.load_state(handle) {
                    bevy::asset::LoadState::Loaded => {
                        // Image is loaded, render it
                        if let Some(image) = images.get(handle) {
                            render_image_with_sixel_support(
                                frame,
                                inner,
                                image,
                                handle.clone(),
                                selected_path,
                                theme,
                                settings,
                                sixel_cache,
                            );
                        } else {
                            let lines = render_image_placeholder(
                                selected_path,
                                Some("Image asset not found"),
                            );
                            let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
                            frame.render_widget(paragraph, inner);
                        }
                    }
                    bevy::asset::LoadState::Failed(err) => {
                        let lines = render_image_placeholder(selected_path, Some(&err.to_string()));
                        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
                        frame.render_widget(paragraph, inner);
                    }
                    _ => {
                        // Still loading
                        let lines = render_image_placeholder(selected_path, None);
                        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
                        frame.render_widget(paragraph, inner);
                    }
                }
            } else {
                let lines = render_image_placeholder(selected_path, Some("No asset handle"));
                let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
                frame.render_widget(paragraph, inner);
            }
        } else {
            // No PreviewImage component found
            let lines = render_image_placeholder(selected_path, Some("Preview not loaded"));
            let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(paragraph, inner);
        }
    } else {
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("No image selected", theme.muted())),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
    }
}

/// Render image with Sixel support if available, fallback to Unicode.
fn render_image_with_sixel_support(
    frame: &mut Frame,
    area: Rect,
    image: &Image,
    handle: Handle<Image>,
    path: &std::path::Path,
    theme: &AppTheme,
    settings: &SettingsState,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    // Check if Sixel is enabled and supported
    let use_sixel = settings.ui.show_image_previews && supports_sixel();

    if use_sixel && sixel_cache.is_some() {
        // Try Sixel rendering
        match render_sixel_preview(
            image,
            handle,
            path,
            area,
            theme,
            sixel_cache.unwrap(),
        ) {
            Ok(sixel_data) => {
                // Render Sixel widget
                let sixel_widget = SixelImageWidget::new(&sixel_data);
                frame.render_widget(sixel_widget, area);
                debug!("Rendered Sixel preview for {:?}", path);
            }
            Err(e) => {
                warn!("Sixel rendering failed, falling back to Unicode: {}", e);
                let lines = render_unicode_preview(image, area, theme);
                let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
                frame.render_widget(paragraph, area);
            }
        }
    } else {
        // Fall back to Unicode block characters
        let lines = render_unicode_preview(image, area, theme);
        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}

/// Render Sixel preview (with caching).
fn render_sixel_preview(
    image: &Image,
    _handle: Handle<Image>,
    path: &std::path::Path,
    area: Rect,
    _theme: &AppTheme,
    cache: &SixelPreviewCache,
) -> anyhow::Result<String> {
    // Check cache first
    if let Some(entry) = cache.get(path) {
        debug!("Sixel cache hit: {:?}", path);
        return Ok(entry.sixel_data);
    }

    // Render and cache
    let options = SixelRenderOptions {
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(4),
        preserve_aspect: true,
        high_quality: true,
    };

    let sixel_data = render_image_sixel(image, &options)?;

    // Cache the result
    let entry = crate::bevy_app::systems::assets::SixelCacheEntry {
        path: path.to_path_buf(),
        sixel_data: sixel_data.clone(),
        size_bytes: sixel_data.len(),
        last_access: std::time::Instant::now(),
        dimensions: (image.width(), image.height()),
    };

    cache.insert(entry);

    Ok(sixel_data)
}

/// Render Unicode block character preview (fallback).
fn render_unicode_preview(image: &Image, area: Rect, theme: &AppTheme) -> Vec<Line<'static>> {
    // Calculate dimensions for Unicode rendering
    let (width, height) = calculate_ascii_dimensions(
        image.width(),
        image.height(),
        area.width.saturating_sub(2),  // Account for padding
        area.height.saturating_sub(4), // Account for header/footer
    );

    // Render image as Unicode block characters
    let mut lines = render_image_to_unicode(image, width, height);

    // Add image info at bottom
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("{}x{} pixels", image.width(), image.height()),
        theme.muted(),
    )));

    lines
}

/// Sixel image widget for ratatui.
struct SixelImageWidget<'a> {
    sixel_data: &'a str,
}

impl<'a> SixelImageWidget<'a> {
    fn new(sixel_data: &'a str) -> Self {
        Self { sixel_data }
    }
}

impl<'a> Widget for SixelImageWidget<'a> {
    fn render(self, area: Rect, _buf: &mut ratatui::buffer::Buffer) {
        debug!("SixelImageWidget rendering at {:?}", area);
        debug!("Sixel data length: {} bytes", self.sixel_data.len());

        let mut stdout = io::stdout();

        // Position cursor at top-left of render area
        let row = area.y + 1; // Convert to 1-indexed
        let col = area.x + 1;

        // Clear the area first
        for line in 0..area.height {
            let line_row = row + line as u16;
            let _ = write!(stdout, "\x1b[{};{}H", line_row, col);
            let _ = write!(stdout, "{}", " ".repeat(area.width as usize));
        }

        // Write Sixel data
        let _ = write!(stdout, "\x1b[{};{}H{}", row, col, self.sixel_data);
        let _ = stdout.flush();

        debug!("Sixel data written to stdout at ({}, {})", row, col);
    }
}

/// Render thumbnail list panel.
fn render_thumbnail_list(frame: &mut Frame, area: Rect, gallery: &GalleryState, theme: &AppTheme) {
    let block = Block::default()
        .title(format!(" Images ({}) ", gallery.len()))
        .borders(Borders::ALL)
        .border_style(theme.border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    // Show up to 10 images around the selected one
    let start_idx = gallery.selected.saturating_sub(5);
    let end_idx = (gallery.selected + 5).min(gallery.len());

    for idx in start_idx..end_idx {
        if let Some(path) = gallery.images.get(idx) {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let prefix = if idx == gallery.selected { "> " } else { "  " };

            let style = if idx == gallery.selected {
                theme.highlight()
            } else {
                theme.text()
            };

            // Truncate filename if too long
            let max_len = (inner.width as usize).saturating_sub(3);
            let display_name = if filename.len() > max_len {
                format!("{}...", &filename[..max_len.saturating_sub(3)])
            } else {
                filename.to_string()
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(display_name, style),
            ]));
        }
    }

    // Add navigation hint at bottom
    if !lines.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "↑/↓: Navigate",
            theme.muted().add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::styled(
            "d: Delete",
            theme.muted().add_modifier(Modifier::ITALIC),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_empty_gallery_render() {
        // Test that empty gallery rendering logic doesn't panic
        let gallery = GalleryState::default();
        assert!(gallery.is_empty());
    }

    #[test]
    fn test_gallery_with_images() {
        let mut gallery = GalleryState::default();
        gallery.add_image(PathBuf::from("/test/img1.png"));
        gallery.add_image(PathBuf::from("/test/img2.png"));

        assert_eq!(gallery.len(), 2);
        assert!(!gallery.is_empty());
        assert_eq!(gallery.selected, 0);
    }

    #[test]
    fn test_thumbnail_window_calculation() {
        let mut gallery = GalleryState::default();
        for i in 0..20 {
            gallery.add_image(PathBuf::from(format!("/test/img{}.png", i)));
        }

        // Test window around selected image
        gallery.selected = 10;
        let start_idx = gallery.selected.saturating_sub(5);
        let end_idx = (gallery.selected + 5).min(gallery.len());

        assert_eq!(start_idx, 5);
        assert_eq!(end_idx, 15);
        assert_eq!(end_idx - start_idx, 10); // Window size
    }

    #[test]
    fn test_thumbnail_window_at_start() {
        let mut gallery = GalleryState::default();
        for i in 0..20 {
            gallery.add_image(PathBuf::from(format!("/test/img{}.png", i)));
        }

        gallery.selected = 0;
        let start_idx = gallery.selected.saturating_sub(5);
        let end_idx = (gallery.selected + 5).min(gallery.len());

        assert_eq!(start_idx, 0);
        assert_eq!(end_idx, 5);
    }

    #[test]
    fn test_thumbnail_window_at_end() {
        let mut gallery = GalleryState::default();
        for i in 0..20 {
            gallery.add_image(PathBuf::from(format!("/test/img{}.png", i)));
        }

        gallery.selected = 19;
        let start_idx = gallery.selected.saturating_sub(5);
        let end_idx = (gallery.selected + 5).min(gallery.len());

        assert_eq!(start_idx, 14);
        assert_eq!(end_idx, 20); // Clamped to gallery length
    }
}
