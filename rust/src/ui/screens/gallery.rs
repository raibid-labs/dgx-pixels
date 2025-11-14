use crate::app::App;
use crate::sixel::{RenderOptions, TerminalCapability};
use crate::ui::screens::{create_block, create_header, create_status_bar};
use crate::ui::{layout::create_layout, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = create_layout(f.size());

    let header = create_header("Gallery");
    f.render_widget(header, chunks[0]);

    render_gallery_body(f, chunks[1], app);

    let (_total_bytes, formatted_size) = calculate_gallery_size(&app.gallery_images);
    let status_text = format!(
        "Total: {} images | Space: {} | Arrow keys to navigate",
        app.gallery_images.len(),
        formatted_size
    );
    let status = create_status_bar(&status_text);
    f.render_widget(status, chunks[2]);
}

/// Calculate total size of gallery images and return (bytes, formatted_string)
fn calculate_gallery_size(images: &[std::path::PathBuf]) -> (u64, String) {
    let total_bytes: u64 = images
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .map(|metadata| metadata.len())
        .sum();

    (total_bytes, format_bytes(total_bytes))
}

/// Format bytes into human-readable format (KB, MB, GB)
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn render_gallery_body(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if app.gallery_images.is_empty() {
        render_empty_gallery(f, area);
        return;
    }

    // Split into preview and thumbnail list
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Main preview
            Constraint::Percentage(30), // Thumbnail list
        ])
        .margin(1)
        .split(area);

    render_main_preview(f, chunks[0], app);
    render_thumbnail_list(f, chunks[1], app);
}

fn render_empty_gallery(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("No images in gallery", Theme::muted())),
        Line::from(""),
        Line::from("Generate some images to see them here!"),
    ];

    let paragraph = Paragraph::new(lines)
        .block(create_block(" Image Gallery "))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_main_preview(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = create_block(" Preview ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(selected_path) = app.selected_gallery_image() {
        match app.terminal_capability {
            TerminalCapability::Sixel => {
                // Check if preview is cached
                if let Some(preview_entry) = app.preview_manager.get_preview(selected_path) {
                    render_sixel_large_preview(f, inner, &preview_entry.sixel_data, selected_path);
                } else {
                    // Request preview
                    let options = RenderOptions {
                        width: inner.width.saturating_sub(4),
                        height: inner.height.saturating_sub(4),
                        preserve_aspect: true,
                        high_quality: true,
                    };

                    let _ = app
                        .preview_manager
                        .request_preview(selected_path.clone(), options);

                    // Show loading
                    render_loading(f, inner);
                }
            }
            TerminalCapability::TextOnly => {
                render_text_only_info(f, inner, selected_path);
            }
        }
    } else {
        render_no_selection(f, inner);
    }
}

fn render_thumbnail_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = create_block(" Images ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();

    // Show up to 10 images around the selected one
    let start_idx = app.selected_gallery_index.saturating_sub(5);
    let end_idx = (app.selected_gallery_index + 5).min(app.gallery_images.len());

    for (idx, path) in app.gallery_images[start_idx..end_idx].iter().enumerate() {
        let actual_idx = start_idx + idx;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let prefix = if actual_idx == app.selected_gallery_index {
            "> "
        } else {
            "  "
        };

        let style = if actual_idx == app.selected_gallery_index {
            Theme::highlight()
        } else {
            Theme::text()
        };

        lines.push(Line::from(vec![
            Span::raw(prefix),
            Span::styled(filename, style),
        ]));

        // Request thumbnail preview for visible items
        if matches!(app.terminal_capability, TerminalCapability::Sixel)
            && !app.preview_manager.has_preview(path)
        {
            let thumbnail_opts = RenderOptions {
                width: 10,
                height: 10,
                preserve_aspect: true,
                high_quality: false,
            };
            let _ = app
                .preview_manager
                .request_preview(path.clone(), thumbnail_opts);
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}

fn render_sixel_large_preview(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    _sixel_data: &str,
    path: &std::path::Path,
) {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "[Sixel Preview Would Appear Here]",
            Theme::highlight(),
        )),
        Line::from(""),
        Line::from(Span::styled(filename, Theme::text())),
        Line::from(""),
        Line::from(Span::styled(
            "Full Sixel rendering available",
            Theme::muted(),
        )),
        Line::from(""),
        Line::from("Use arrow keys to navigate"),
    ];

    let paragraph = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_loading(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("Loading preview...", Theme::highlight())),
        Line::from(""),
        Line::from(Span::styled("Please wait", Theme::muted())),
    ];

    let paragraph = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_text_only_info(f: &mut Frame, area: ratatui::layout::Rect, path: &std::path::Path) {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(filename, Theme::highlight())),
        Line::from(""),
        Line::from("Image preview not available"),
        Line::from(""),
        Line::from(Span::styled(
            "Terminal does not support Sixel",
            Theme::muted(),
        )),
        Line::from(""),
        Line::from("Use kitty, WezTerm, or iTerm2"),
    ];

    let paragraph = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_no_selection(f: &mut Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("No image selected", Theme::muted())),
    ];

    let paragraph = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_gallery_screen_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();
        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gallery_with_images() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        app.add_to_gallery(PathBuf::from("/test/img1.png"));
        app.add_to_gallery(PathBuf::from("/test/img2.png"));

        let result = terminal.draw(|f| render(f, &app));
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(2048), "2.00 KB");
        assert_eq!(format_bytes(1024 * 100), "100.00 KB");
    }

    #[test]
    fn test_format_bytes_megabytes() {
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 5), "5.00 MB");
        assert_eq!(format_bytes(1024 * 1024 + 512 * 1024), "1.50 MB");
        assert_eq!(format_bytes(1024 * 1024 * 100), "100.00 MB");
    }

    #[test]
    fn test_format_bytes_gigabytes() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.00 GB");
        assert_eq!(
            format_bytes(1024 * 1024 * 1024 + 512 * 1024 * 1024),
            "1.50 GB"
        );
    }

    #[test]
    fn test_calculate_gallery_size_empty() {
        let images: Vec<PathBuf> = vec![];
        let (bytes, formatted) = calculate_gallery_size(&images);
        assert_eq!(bytes, 0);
        assert_eq!(formatted, "0 B");
    }

    #[test]
    fn test_calculate_gallery_size_nonexistent_files() {
        // Non-existent files should be gracefully ignored
        let images = vec![
            PathBuf::from("/nonexistent/file1.png"),
            PathBuf::from("/nonexistent/file2.png"),
        ];
        let (bytes, formatted) = calculate_gallery_size(&images);
        assert_eq!(bytes, 0);
        assert_eq!(formatted, "0 B");
    }
}
