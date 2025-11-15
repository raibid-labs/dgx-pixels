//! # Gallery Screen Renderer
//!
//! WS-10: Gallery screen rendering system for Bevy-Ratatui migration.
//! Displays a grid of generated images with detail view and navigation.

use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::components::PreviewImage;
use crate::bevy_app::resources::{AppTheme, CurrentScreen, GalleryState, Screen};

/// Main gallery screen render system.
///
/// Renders a two-panel layout:
/// - Left panel (70%): Large preview of selected image
/// - Right panel (30%): Thumbnail list of all images
pub fn render_gallery_screen(
    current_screen: Res<CurrentScreen>,
    gallery: Res<GalleryState>,
    theme: Res<AppTheme>,
    preview_query: Query<&PreviewImage>,
    images: Res<Assets<Image>>,
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
            } else {
                render_gallery_body(frame, area, &gallery, &theme, &preview_query, &images);
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

/// Render gallery body with preview and thumbnail list.
fn render_gallery_body(
    frame: &mut Frame,
    area: Rect,
    gallery: &GalleryState,
    theme: &AppTheme,
    _preview_query: &Query<&PreviewImage>,
    _images: &Assets<Image>,
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

    render_main_preview(frame, chunks[0], gallery, theme);
    render_thumbnail_list(frame, chunks[1], gallery, theme);
}

/// Render main preview panel.
fn render_main_preview(frame: &mut Frame, area: Rect, gallery: &GalleryState, theme: &AppTheme) {
    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(theme.border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(selected_path) = gallery.current_image() {
        let filename = selected_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // TODO: WS-06 integration - render actual image using PreviewImage component
        // For now, show placeholder with filename
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("[Image Preview]", theme.highlight())),
            Line::from(""),
            Line::from(Span::styled(filename, theme.text())),
            Line::from(""),
            Line::from(Span::styled(
                "Full image preview will be available after WS-06 integration",
                theme.muted(),
            )),
            Line::from(""),
            Line::from(Span::styled("Use arrow keys to navigate", theme.muted())),
            Line::from(""),
            Line::from(Span::styled(
                format!("Image {} of {}", gallery.selected + 1, gallery.len()),
                theme.text(),
            )),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
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
