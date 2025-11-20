//! # Comparison Screen Renderer
//!
//! Side-by-side model comparison with Sixel preview support.
//! Allows users to compare outputs from different models/LoRAs with the same prompt.

use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::bevy_app::{
    components::PreviewImage,
    resources::{
        comparison_state::{ComparisonMode, ComparisonPane},
        AppTheme, ComparisonState, CurrentScreen, Screen, SettingsState,
    },
    systems::assets::{SixelPreviewCache, SixelRenderOptions, render_image_sixel, supports_sixel},
    systems::render::sixel_utils::render_sixel_to_area,
};

/// Render the Comparison screen
pub fn render_comparison_screen(
    current_screen: Res<CurrentScreen>,
    comparison: Res<ComparisonState>,
    theme: Res<AppTheme>,
    settings: Res<SettingsState>,
    preview_query: Query<&PreviewImage>,
    images: Option<Res<Assets<Image>>>,
    asset_server: Option<Res<AssetServer>>,
    sixel_cache: Option<Res<SixelPreviewCache>>,
    mut ratatui: ResMut<RatatuiContext>,
) {
    if current_screen.0 != Screen::Comparison {
        return;
    }

    ratatui
        .draw(|frame| {
            let area = frame.area();

            match comparison.mode {
                ComparisonMode::Dual => render_dual_comparison(
                    frame,
                    area,
                    &comparison,
                    &theme,
                    &settings,
                    &preview_query,
                    images.as_deref(),
                    asset_server.as_deref(),
                    sixel_cache.as_deref(),
                ),
                ComparisonMode::Multi => render_multi_comparison(frame, area, &comparison, &theme),
            }
        })
        .expect("Failed to render comparison screen");
}

/// Render dual-pane side-by-side comparison
fn render_dual_comparison(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
    settings: &SettingsState,
    preview_query: &Query<&PreviewImage>,
    images: Option<&Assets<Image>>,
    asset_server: Option<&AssetServer>>,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Prompt input
            Constraint::Min(0),    // Dual preview panes
            Constraint::Length(3), // Controls
        ])
        .split(area);

    // Title
    render_title(frame, chunks[0], &theme);

    // Prompt input
    render_prompt_input(frame, chunks[1], comparison, theme);

    // Model browsing overlay (if active)
    if comparison.browsing_models {
        render_model_browser(frame, chunks[2], comparison, theme);
    } else {
        // Dual preview panes
        render_dual_panes(
            frame,
            chunks[2],
            comparison,
            theme,
            settings,
            preview_query,
            images,
            asset_server,
            sixel_cache,
        );
    }

    // Controls
    render_dual_controls(frame, chunks[3], comparison, theme);
}

fn render_title(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let title = Paragraph::new(" Model Comparison - Side by Side")
        .style(theme.header())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.highlight()),
        );
    frame.render_widget(title, area);
}

fn render_prompt_input(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
) {
    let prompt_text = if comparison.prompt.is_empty() {
        "[Enter shared prompt for comparison...]"
    } else {
        &comparison.prompt
    };

    let prompt_display = Line::from(vec![
        Span::styled("Prompt: ", theme.muted()),
        Span::styled(
            prompt_text,
            if comparison.prompt.is_empty() {
                theme.muted()
            } else {
                theme.text()
            },
        ),
    ]);

    let paragraph = Paragraph::new(prompt_display).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

fn render_dual_panes(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
    settings: &SettingsState,
    preview_query: &Query<&PreviewImage>,
    images: Option<&Assets<Image>>,
    asset_server: Option<&AssetServer>>,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    // Split into left and right panes
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render left pane
    render_pane(
        frame,
        panes[0],
        ComparisonPane::Left,
        &comparison.left_model,
        &comparison.left_image,
        comparison.left_metadata.as_ref(),
        comparison.selected_pane == ComparisonPane::Left,
        comparison.is_running,
        theme,
        settings,
        preview_query,
        images,
        asset_server,
        sixel_cache,
    );

    // Render right pane
    render_pane(
        frame,
        panes[1],
        ComparisonPane::Right,
        &comparison.right_model,
        &comparison.right_image,
        comparison.right_metadata.as_ref(),
        comparison.selected_pane == ComparisonPane::Right,
        comparison.is_running,
        theme,
        settings,
        preview_query,
        images,
        asset_server,
        sixel_cache,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_pane(
    frame: &mut Frame,
    area: Rect,
    pane: ComparisonPane,
    model: &Option<String>,
    image_path: &Option<std::path::PathBuf>,
    metadata: Option<&crate::bevy_app::resources::comparison_state::GenerationMetadata>,
    is_selected: bool,
    is_running: bool,
    theme: &AppTheme,
    settings: &SettingsState,
    preview_query: &Query<&PreviewImage>,
    images: Option<&Assets<Image>>,
    asset_server: Option<&AssetServer>>,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    let pane_name = match pane {
        ComparisonPane::Left => "Left",
        ComparisonPane::Right => "Right",
    };

    let model_name = model.as_deref().unwrap_or("[No model selected]");

    let title = if is_selected {
        format!(" {} > {} ", pane_name, model_name)
    } else {
        format!(" {}: {} ", pane_name, model_name)
    };

    let border_style = if is_selected {
        theme.highlight()
    } else {
        theme.border()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Content layout: preview (top) + metadata (bottom)
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Preview area
            Constraint::Length(5), // Metadata
        ])
        .split(inner);

    // Render preview
    if is_running {
        render_generating_placeholder(frame, content_chunks[0], theme);
    } else if let Some(path) = image_path {
        render_preview_image(
            frame,
            content_chunks[0],
            path,
            theme,
            settings,
            preview_query,
            images,
            asset_server,
            sixel_cache,
        );
    } else {
        render_empty_preview(frame, content_chunks[0], theme);
    }

    // Render metadata
    render_metadata(frame, content_chunks[1], metadata, theme);
}

fn render_generating_placeholder(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("Generating...", theme.highlight())),
        Line::from(""),
        Line::from(Span::styled("Please wait", theme.muted())),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_empty_preview(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("[No image generated]", theme.muted())),
        Line::from(""),
        Line::from(Span::styled("Press Enter to generate", theme.muted())),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_preview_image(
    frame: &mut Frame,
    area: Rect,
    path: &std::path::Path,
    theme: &AppTheme,
    settings: &SettingsState,
    preview_query: &Query<&PreviewImage>,
    images: Option<&Assets<Image>>,
    asset_server: Option<&AssetServer>>,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    // Try to find PreviewImage component for this path
    let preview = preview_query.iter().find(|p| &p.path == path);

    if let (Some(preview), Some(images), Some(asset_server)) = (preview, images, asset_server) {
        if let Some(handle) = &preview.asset_handle {
            match asset_server.load_state(handle) {
                bevy::asset::LoadState::Loaded => {
                    if let Some(image) = images.get(handle) {
                        render_image_with_sixel(
                            frame,
                            area,
                            image,
                            path,
                            theme,
                            settings,
                            sixel_cache,
                        );
                        return;
                    }
                }
                bevy::asset::LoadState::Failed(_) => {
                    render_error_placeholder(frame, area, "Failed to load image", theme);
                    return;
                }
                _ => {
                    render_loading_placeholder(frame, area, theme);
                    return;
                }
            }
        }
    }

    // Fallback: path only
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("[Image preview]", theme.muted())),
        Line::from(""),
        Line::from(Span::styled(
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            theme.muted(),
        )),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_image_with_sixel(
    frame: &mut Frame,
    area: Rect,
    image: &Image,
    path: &std::path::Path,
    theme: &AppTheme,
    settings: &SettingsState,
    sixel_cache: Option<&SixelPreviewCache>,
) {
    let use_sixel = settings.ui.show_image_previews && supports_sixel();

    if use_sixel && sixel_cache.is_some() {
        // Try Sixel rendering
        if let Some(cache) = sixel_cache {
            if let Some(entry) = cache.get(path) {
                // Render cached Sixel
                let _ = render_sixel_to_area(area, &entry.sixel_data);
                return;
            } else {
                // Generate and cache Sixel
                let options = SixelRenderOptions {
                    width: area.width.saturating_sub(4),
                    height: area.height.saturating_sub(4),
                    preserve_aspect: true,
                    high_quality: true,
                };

                if let Ok(sixel_data) = render_image_sixel(image, &options) {
                    let entry = crate::bevy_app::systems::assets::SixelCacheEntry {
                        path: path.to_path_buf(),
                        sixel_data: sixel_data.clone(),
                        size_bytes: sixel_data.len(),
                        last_access: std::time::Instant::now(),
                        dimensions: (image.width(), image.height()),
                    };
                    cache.insert(entry);

                    let _ = render_sixel_to_area(area, &sixel_data);
                    return;
                }
            }
        }
    }

    // Fallback: Unicode block characters or placeholder
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("[Image Preview]", theme.muted())),
        Line::from(""),
        Line::from(Span::styled(
            format!("{}x{}", image.width(), image.height()),
            theme.text(),
        )),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_loading_placeholder(frame: &mut Frame, area: Rect, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Loading image...", theme.muted())),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_error_placeholder(frame: &mut Frame, area: Rect, error: &str, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Error", theme.error())),
        Line::from(Span::styled(error, theme.muted())),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_metadata(
    frame: &mut Frame,
    area: Rect,
    metadata: Option<&crate::bevy_app::resources::comparison_state::GenerationMetadata>,
    theme: &AppTheme,
) {
    let lines = if let Some(meta) = metadata {
        vec![
            Line::from(vec![
                Span::styled("Size: ", theme.muted()),
                Span::styled(format!("{}x{}", meta.size.0, meta.size.1), theme.text()),
            ]),
            Line::from(vec![
                Span::styled("Time: ", theme.muted()),
                Span::styled(format!("{:.2}s", meta.inference_time_s), theme.text()),
            ]),
            Line::from(vec![
                Span::styled("Steps: ", theme.muted()),
                Span::styled(meta.steps.to_string(), theme.text()),
            ]),
            Line::from(vec![
                Span::styled("Seed: ", theme.muted()),
                Span::styled(
                    meta.seed
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "random".to_string()),
                    theme.text(),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(Span::styled("No metadata", theme.muted())),
            Line::from(""),
            Line::from(""),
            Line::from(""),
        ]
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_model_browser(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
) {
    let block = Block::default()
        .title(" Select Model ")
        .borders(Borders::ALL)
        .border_style(theme.highlight());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if comparison.available_models.is_empty() {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled("No models available", theme.muted())),
            Line::from(""),
            Line::from(Span::styled("Press Esc to close", theme.muted())),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
        return;
    }

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "Select model for {} pane:",
            match comparison.selected_pane {
                ComparisonPane::Left => "Left",
                ComparisonPane::Right => "Right",
            }
        ),
        theme.text(),
    )));
    lines.push(Line::from(""));

    // Show models
    for (idx, model) in comparison.available_models.iter().enumerate() {
        let is_selected = idx == comparison.model_list_index;
        let (prefix, style) = if is_selected {
            ("> ", theme.highlight())
        } else {
            ("  ", theme.text())
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(&model.name, style),
            Span::styled(format!(" ({})", model.model_type), theme.muted()),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Up/Down: Navigate | Enter: Select | Esc: Cancel",
        theme.muted(),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_dual_controls(frame: &mut Frame, area: Rect, comparison: &ComparisonState, theme: &AppTheme) {
    let can_run = comparison.can_run_comparison();

    let controls = Line::from(vec![
        Span::raw("  "),
        Span::styled("Tab", theme.highlight()),
        Span::raw(" Switch Pane  "),
        Span::styled("m", theme.highlight()),
        Span::raw(" Change Model  "),
        Span::styled(
            "Enter",
            if can_run {
                theme.highlight()
            } else {
                theme.muted()
            },
        ),
        Span::raw(if can_run {
            " Generate"
        } else {
            " Generate (need prompt & models)"
        }),
        Span::raw("  "),
        Span::styled("r", theme.highlight()),
        Span::raw(" Reset"),
    ]);

    let paragraph = Paragraph::new(controls).block(
        Block::default()
            .title(" Controls ")
            .borders(Borders::ALL)
            .border_style(theme.text()),
    );

    frame.render_widget(paragraph, area);
}

/// Multi-model comparison rendering (legacy mode, up to 3 models)
fn render_multi_comparison(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(6), // Model selection
            Constraint::Length(5), // Prompt input
            Constraint::Min(0),    // Results area
        ])
        .margin(1)
        .split(area);

    // Title
    let title = Paragraph::new(" Model Comparison - Multi")
        .style(theme.header())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.highlight()),
        );
    frame.render_widget(title, chunks[0]);

    // Model selection
    render_multi_model_selection(frame, chunks[1], comparison, theme);

    // Prompt input
    render_prompt_input(frame, chunks[2], comparison, theme);

    // Results area
    render_multi_results(frame, chunks[3], comparison, theme);
}

fn render_multi_model_selection(
    frame: &mut Frame,
    area: Rect,
    comparison: &ComparisonState,
    theme: &AppTheme,
) {
    let mut lines = vec![Line::from("")];

    for (idx, model) in comparison.models.iter().enumerate() {
        let is_selected = idx == comparison.selected_index;
        let (prefix, style) = if is_selected {
            ("> ", theme.highlight())
        } else {
            ("  ", theme.text())
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("Slot {}: ", idx + 1), theme.muted()),
            Span::styled(model, style),
        ]));
    }

    if comparison.models.len() < 3 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  + ", theme.muted()),
            Span::styled(
                format!(
                    "Add model (press 'a') - {} slots available",
                    3 - comparison.models.len()
                ),
                theme.muted(),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(format!(" Models Selected ({}/3) ", comparison.models.len()))
            .borders(Borders::ALL)
            .border_style(theme.highlight()),
    );

    frame.render_widget(paragraph, area);
}

fn render_multi_results(frame: &mut Frame, area: Rect, comparison: &ComparisonState, theme: &AppTheme) {
    if comparison.is_running {
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "Generating images with all models...",
                theme.highlight(),
            )),
            Line::from(""),
            Line::from(Span::styled("This may take several minutes", theme.muted())),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Comparison Running ")
                    .borders(Borders::ALL)
                    .border_style(theme.highlight()),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    } else {
        let num_models = comparison.models.len();
        if num_models == 0 {
            let empty_msg = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled("No models selected", theme.muted())),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'a' to add models for comparison",
                    theme.muted(),
                )),
            ])
            .block(
                Block::default()
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(theme.text()),
            );
            frame.render_widget(empty_msg, area);
            return;
        }

        let constraints: Vec<Constraint> = (0..num_models)
            .map(|_| Constraint::Percentage(100 / num_models as u16))
            .collect();

        let result_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        for (idx, model) in comparison.models.iter().enumerate() {
            render_multi_model_result(frame, result_chunks[idx], model, theme);
        }
    }
}

fn render_multi_model_result(frame: &mut Frame, area: Rect, model: &str, theme: &AppTheme) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("[Image preview placeholder]", theme.muted())),
        Line::from(""),
        Line::from(Span::styled("WS-06: Sixel rendering", theme.muted())),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Speed: ", theme.muted()),
            Span::styled("--", theme.text()),
        ]),
        Line::from(vec![
            Span::styled("Quality: ", theme.muted()),
            Span::styled("--", theme.text()),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" {} ", model))
                .borders(Borders::ALL)
                .border_style(theme.text()),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_comparison_screen_compiles() {
        let mut app = App::new();
        app.insert_resource(CurrentScreen(Screen::Comparison));
        app.insert_resource(AppTheme::default());
        app.insert_resource(ComparisonState::default());
        app.insert_resource(SettingsState::default());
        app.add_systems(Update, render_comparison_screen);
    }
}
