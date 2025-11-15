//! Side-by-Side Model Comparison UI Screen
//!
//! The killer feature of DGX-Pixels - compare multiple models simultaneously
//! to validate training improvements.

use crate::app::App;
use crate::comparison::{ComparisonManager, GenerationParams, ModelConfig};
use crate::ui::screens::{create_block, create_header, create_status_bar};
use crate::ui::{layout::create_layout, theme::Theme};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

/// Comparison screen state
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ComparisonMode {
    /// Configuring comparison parameters
    Setup,
    /// Selecting models to compare
    ModelSelection { slot: usize },
    /// Running comparison
    Running { comparison_id: String },
    /// Viewing results
    Results { comparison_id: String },
}

/// UI state for comparison screen
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ComparisonState {
    /// Current mode
    pub mode: ComparisonMode,

    /// Models being compared (2-3 slots)
    pub selected_models: Vec<Option<ModelConfig>>,

    /// Shared generation parameters
    pub params: GenerationParams,

    /// Use same seed for all models (fair comparison)
    pub use_same_seed: bool,

    /// Available models (from backend)
    pub available_models: Vec<ModelConfig>,

    /// Selected model in picker (for ModelSelection mode)
    pub picker_index: usize,

    /// Comparison manager
    pub comparison_manager: ComparisonManager,

    /// Currently viewing result index
    #[allow(dead_code)]
    pub viewing_result_index: usize,
}

impl Default for ComparisonState {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparisonState {
    pub fn new() -> Self {
        Self {
            mode: ComparisonMode::Setup,
            selected_models: vec![None, None, None], // 3 slots
            params: GenerationParams::default(),
            use_same_seed: true,
            available_models: Self::default_models(),
            picker_index: 0,
            comparison_manager: ComparisonManager::new(),
            viewing_result_index: 0,
        }
    }

    /// Default models for demo/testing
    fn default_models() -> Vec<ModelConfig> {
        vec![
            ModelConfig {
                name: "SDXL Base 1.0".to_string(),
                base: "sd_xl_base_1.0.safetensors".to_string(),
                lora: None,
                lora_strength: 1.0,
            },
            ModelConfig {
                name: "SDXL + Pixel Art LoRA".to_string(),
                base: "sd_xl_base_1.0.safetensors".to_string(),
                lora: Some("pixel_art_v1.safetensors".to_string()),
                lora_strength: 0.8,
            },
            ModelConfig {
                name: "SDXL + Game Sprite LoRA".to_string(),
                base: "sd_xl_base_1.0.safetensors".to_string(),
                lora: Some("game_sprites_v2.safetensors".to_string()),
                lora_strength: 0.9,
            },
        ]
    }

    /// Get number of selected models
    pub fn selected_count(&self) -> usize {
        self.selected_models.iter().filter(|m| m.is_some()).count()
    }

    /// Check if ready to run comparison
    pub fn can_compare(&self) -> bool {
        self.selected_count() >= 2 && !self.params.prompt.is_empty()
    }
}

/// Render the comparison screen
pub fn render(f: &mut Frame, app: &App, state: &ComparisonState) {
    let chunks = create_layout(f.area());

    // Header
    let header = create_header("Side-by-Side Model Comparison");
    f.render_widget(header, chunks[0]);

    // Body - different layouts based on mode
    match &state.mode {
        ComparisonMode::Setup | ComparisonMode::ModelSelection { .. } => {
            render_setup_mode(f, chunks[1], app, state);
        }
        ComparisonMode::Running { comparison_id } => {
            render_running_mode(f, chunks[1], state, comparison_id);
        }
        ComparisonMode::Results { comparison_id } => {
            render_results_mode(f, chunks[1], state, comparison_id);
        }
    }

    // Status bar
    let status = match &state.mode {
        ComparisonMode::Setup => {
            format!(
                "Models: {}/3 | Prompt: {} chars | [Enter] Compare [M] Select Model [ESC] Back",
                state.selected_count(),
                state.params.prompt.len()
            )
        }
        ComparisonMode::ModelSelection { slot } => {
            format!(
                "Selecting model for slot {} | [↑↓] Navigate [Enter] Select [ESC] Cancel",
                slot + 1
            )
        }
        ComparisonMode::Running { .. } => "Generating... | [C] Cancel".to_string(),
        ComparisonMode::Results { .. } => {
            "Results | [1-3] Vote for model [R] Run Again [ESC] Back".to_string()
        }
    };
    let status_bar = create_status_bar(&status);
    f.render_widget(status_bar, chunks[2]);
}

/// Render setup/configuration mode
fn render_setup_mode(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App,
    state: &ComparisonState,
) {
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Prompt input
            Constraint::Length(10), // Model selection grid
            Constraint::Min(5),     // Parameters
            Constraint::Length(3),  // Action buttons
        ])
        .margin(1)
        .split(area);

    // Prompt input
    render_prompt_input(f, body_chunks[0], app);

    // Model selection slots
    render_model_slots(f, body_chunks[1], state);

    // Parameters
    render_parameters(f, body_chunks[2], state);

    // Action buttons
    render_action_buttons(f, body_chunks[3], state);

    // Model picker overlay (if in ModelSelection mode)
    if let ComparisonMode::ModelSelection { slot } = state.mode {
        render_model_picker(f, area, state, slot);
    }
}

/// Render running mode (showing progress)
fn render_running_mode(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &ComparisonState,
    comparison_id: &str,
) {
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status
            Constraint::Min(10),   // Progress grid
        ])
        .margin(1)
        .split(area);

    // Status message
    let status_text = vec![Line::from(vec![
        Span::styled("Generating with multiple models... ", Theme::text()),
        Span::styled("This may take 10-15 seconds", Theme::muted()),
    ])];
    let status_para = Paragraph::new(status_text).block(create_block(" Comparison in Progress "));
    f.render_widget(status_para, body_chunks[0]);

    // Progress for each model
    render_progress_grid(f, body_chunks[1], state, comparison_id);
}

/// Render results mode (side-by-side comparison)
fn render_results_mode(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &ComparisonState,
    comparison_id: &str,
) {
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Prompt display
            Constraint::Min(15),   // Side-by-side results
            Constraint::Length(5), // Voting/preference section
        ])
        .margin(1)
        .split(area);

    // Show prompt
    let prompt_text = vec![Line::from(vec![
        Span::styled("Prompt: ", Theme::muted()),
        Span::styled(&state.params.prompt, Theme::text()),
    ])];
    let prompt_para = Paragraph::new(prompt_text).block(create_block(" Comparison Parameters "));
    f.render_widget(prompt_para, body_chunks[0]);

    // Side-by-side results
    render_side_by_side(f, body_chunks[1], state, comparison_id);

    // Voting section
    render_voting_section(f, body_chunks[2], state, comparison_id);
}

/// Render prompt input field
fn render_prompt_input(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = create_block(" Prompt (same for all models) ");
    let inner = block.inner(area);

    let prompt_text = if app.input_buffer.is_empty() {
        Span::styled("Enter comparison prompt...", Theme::muted())
    } else {
        Span::styled(&app.input_buffer, Theme::text())
    };

    let paragraph = Paragraph::new(prompt_text).block(block);
    f.render_widget(paragraph, area);

    // Show cursor
    if app.current_screen == crate::app::Screen::Comparison {
        f.set_cursor_position((inner.x + app.cursor_pos as u16, inner.y));
    }
}

/// Render model selection slots (3 slots for comparison)
fn render_model_slots(f: &mut Frame, area: ratatui::layout::Rect, state: &ComparisonState) {
    let slot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    for (i, slot) in state.selected_models.iter().enumerate() {
        let title = format!(" Model {} ", i + 1);
        let block = create_block(&title);
        let _inner = block.inner(slot_chunks[i]);

        let content = if let Some(model) = slot {
            vec![
                Line::from(Span::styled(&model.name, Theme::highlight())),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Base: ", Theme::muted()),
                    Span::styled(&model.base, Theme::text()),
                ]),
                if let Some(lora) = &model.lora {
                    Line::from(vec![
                        Span::styled("LoRA: ", Theme::muted()),
                        Span::styled(lora, Theme::success()),
                    ])
                } else {
                    Line::from(Span::styled("No LoRA", Theme::muted()))
                },
                Line::from(""),
                Line::from(Span::styled("[M] Change", Theme::button())),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(Span::styled("[ Empty Slot ]", Theme::muted())),
                Line::from(""),
                Line::from(Span::styled("[M] Select Model", Theme::button())),
            ]
        };

        let para = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(para, slot_chunks[i]);
    }
}

/// Render generation parameters
fn render_parameters(f: &mut Frame, area: ratatui::layout::Rect, state: &ComparisonState) {
    let lines = vec![
        Line::from(vec![
            Span::raw("Size: "),
            Span::styled(
                format!("{}x{}", state.params.width, state.params.height),
                Theme::text(),
            ),
            Span::raw("  Steps: "),
            Span::styled(state.params.steps.to_string(), Theme::text()),
            Span::raw("  CFG: "),
            Span::styled(state.params.cfg_scale.to_string(), Theme::text()),
        ]),
        Line::from(vec![
            Span::raw("Seed: "),
            Span::styled(state.params.seed.to_string(), Theme::text()),
            Span::raw("  "),
            if state.use_same_seed {
                Span::styled("[✓] Use same seed (fair comparison)", Theme::success())
            } else {
                Span::styled("[ ] Use same seed", Theme::muted())
            },
        ]),
    ];

    let para = Paragraph::new(lines).block(create_block(" Generation Parameters "));

    f.render_widget(para, area);
}

/// Render action buttons
fn render_action_buttons(f: &mut Frame, area: ratatui::layout::Rect, state: &ComparisonState) {
    let button_text = if state.can_compare() {
        vec![
            Span::styled(" [Enter] Start Comparison ", Theme::button()),
            Span::raw("  "),
            Span::styled(" [ESC] Cancel ", Theme::button()),
        ]
    } else {
        vec![
            Span::styled(
                " [Enter] Start Comparison (need 2+ models) ",
                Theme::muted(),
            ),
            Span::raw("  "),
            Span::styled(" [ESC] Cancel ", Theme::button()),
        ]
    };

    let para = Paragraph::new(Line::from(button_text)).alignment(Alignment::Center);

    f.render_widget(para, area);
}

/// Render model picker overlay
fn render_model_picker(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &ComparisonState,
    _slot: usize,
) {
    // Center the picker (40% width, 60% height)
    let width = (area.width * 40) / 100;
    let height = (area.height * 60) / 100;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    let picker_area = ratatui::layout::Rect {
        x: area.x + x,
        y: area.y + y,
        width,
        height,
    };

    // Background
    let block = Block::default()
        .title(Span::styled(" Select Model ", Theme::title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    f.render_widget(block.clone(), picker_area);

    // Model list
    let inner = block.inner(picker_area);
    let items: Vec<ListItem> = state
        .available_models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let content = if i == state.picker_index {
                Line::from(vec![
                    Span::styled("> ", Theme::highlight()),
                    Span::styled(&model.name, Theme::highlight()),
                ])
            } else {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&model.name, Theme::text()),
                ])
            };
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

/// Render progress grid during generation
fn render_progress_grid(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &ComparisonState,
    _comparison_id: &str,
) {
    let model_count = state.selected_count();
    let constraints = match model_count {
        2 => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        3 => vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ],
        _ => vec![Constraint::Percentage(100)],
    };

    let progress_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, model_opt) in state.selected_models.iter().enumerate() {
        if let Some(model) = model_opt {
            render_model_progress(f, progress_chunks[i], model, 45.0); // TODO: Real progress
        }
    }
}

/// Render progress for a single model
fn render_model_progress(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    model: &ModelConfig,
    progress: f32,
) {
    let title = format!(" {} ", model.name);
    let block = create_block(&title);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Model info
            Constraint::Length(3), // Progress bar
            Constraint::Min(3),    // Preview placeholder
        ])
        .split(inner);

    // Model info
    let info_text = if let Some(lora) = &model.lora {
        format!("Base: {}\nLoRA: {}", model.base, lora)
    } else {
        format!("Base: {}", model.base)
    };
    let info = Paragraph::new(info_text).style(Theme::text());
    f.render_widget(info, chunks[0]);

    // Progress bar
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent((progress as u16).min(100))
        .label(format!("{:.1}%", progress));
    f.render_widget(gauge, chunks[1]);

    // Preview placeholder
    let preview = Paragraph::new("[Preview]\n\nGenerating...")
        .style(Theme::muted())
        .alignment(Alignment::Center);
    f.render_widget(preview, chunks[2]);

    f.render_widget(block, area);
}

/// Render side-by-side results
fn render_side_by_side(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &ComparisonState,
    comparison_id: &str,
) {
    let result = state.comparison_manager.get_completed(comparison_id);

    let model_count = state.selected_count();
    let constraints = match model_count {
        2 => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        3 => vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ],
        _ => vec![Constraint::Percentage(100)],
    };

    let result_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, model_opt) in state.selected_models.iter().enumerate() {
        if let Some(model) = model_opt {
            let model_result = result.and_then(|r| r.results.get(i));

            render_model_result(f, result_chunks[i], model, model_result, i);
        }
    }
}

/// Render result for a single model
fn render_model_result(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    model: &ModelConfig,
    _result: Option<&crate::comparison::ModelResult>,
    index: usize,
) {
    let title = format!(" {} - Model {} ", model.name, index + 1);
    let block = create_block(&title);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Image preview
            Constraint::Length(4), // Metadata
        ])
        .split(inner);

    // Image preview (placeholder for now - will use Sixel later)
    let preview = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("[Image Preview]", Theme::muted())),
        Line::from(""),
        Line::from("Sixel image will"),
        Line::from("appear here"),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(preview, chunks[0]);

    // Metadata
    let metadata = vec![
        Line::from(vec![
            Span::styled("Time: ", Theme::muted()),
            Span::styled("3.5s", Theme::text()),
        ]),
        Line::from(vec![
            Span::styled("LoRA: ", Theme::muted()),
            if model.lora.is_some() {
                Span::styled("Yes", Theme::success())
            } else {
                Span::styled("No", Theme::muted())
            },
        ]),
    ];
    let meta_para = Paragraph::new(metadata);
    f.render_widget(meta_para, chunks[1]);

    f.render_widget(block, area);
}

/// Render voting/preference section
fn render_voting_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &ComparisonState,
    comparison_id: &str,
) {
    let result = state.comparison_manager.get_completed(comparison_id);

    let content = if let Some(result) = result {
        if let Some(winner_idx) = result.user_preference {
            vec![
                Line::from(vec![
                    Span::styled("Your preference: ", Theme::text()),
                    Span::styled(format!("Model {}", winner_idx + 1), Theme::success()),
                ]),
                Line::from(""),
                Line::from(Span::styled("[R] Run comparison again", Theme::button())),
            ]
        } else {
            vec![
                Line::from(Span::styled(
                    "Which model produced better results?",
                    Theme::text(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[1] ", Theme::button()),
                    Span::raw("Model 1  "),
                    Span::styled("[2] ", Theme::button()),
                    Span::raw("Model 2  "),
                    if state.selected_count() >= 3 {
                        Span::styled("[3] ", Theme::button())
                    } else {
                        Span::raw("")
                    },
                    if state.selected_count() >= 3 {
                        Span::raw("Model 3")
                    } else {
                        Span::raw("")
                    },
                ]),
            ]
        }
    } else {
        vec![Line::from(Span::styled(
            "Loading results...",
            Theme::muted(),
        ))]
    };

    let para = Paragraph::new(content)
        .block(create_block(" Vote for Best Result "))
        .alignment(Alignment::Center);

    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_state_creation() {
        let state = ComparisonState::new();
        assert_eq!(state.mode, ComparisonMode::Setup);
        assert_eq!(state.selected_count(), 0);
        assert!(!state.can_compare());
    }

    #[test]
    fn test_can_compare() {
        let mut state = ComparisonState::new();
        assert!(!state.can_compare());

        // Add 2 models
        state.selected_models[0] = Some(ModelConfig::default());
        state.selected_models[1] = Some(ModelConfig::default());
        assert!(!state.can_compare()); // No prompt

        // Add prompt
        state.params.prompt = "test".to_string();
        assert!(state.can_compare());
    }
}
