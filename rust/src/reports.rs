//! Comparison Report Export
//!
//! Export comparison results to CSV/JSON for analysis and training validation

#![allow(dead_code)]

use crate::comparison::ComparisonResult;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

// ============================================================================
// Report Structures
// ============================================================================

/// Full comparison report for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// Report metadata
    pub metadata: ReportMetadata,

    /// Individual comparison results
    pub comparisons: Vec<ComparisonExport>,

    /// Aggregate statistics
    pub statistics: StatisticsExport,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub generated_at: String,
    pub version: String,
    pub total_comparisons: usize,
}

/// Exported comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonExport {
    pub comparison_id: String,
    pub prompt: String,
    pub seed: u64,
    pub models: Vec<ModelExport>,
    pub winner: Option<String>,
    pub notes: Option<String>,
    pub completed_at: String,
}

/// Exported model data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelExport {
    pub name: String,
    pub base_model: String,
    pub lora: Option<String>,
    pub lora_strength: f32,
    pub generation_time_s: Option<f32>,
    pub image_path: Option<String>,
}

/// Exported statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsExport {
    pub total_comparisons: usize,
    pub comparisons_with_preference: usize,
    pub preference_rate: f32,
    pub model_wins: Vec<ModelWinCount>,
    pub avg_generation_time_s: Option<f32>,
}

/// Model win count for statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelWinCount {
    pub model_name: String,
    pub wins: usize,
    pub win_rate: f32,
}

// ============================================================================
// Report Builder
// ============================================================================

/// Builder for creating comparison reports
pub struct ReportBuilder {
    comparisons: Vec<ComparisonResult>,
}

impl ReportBuilder {
    /// Create a new report builder
    pub fn new() -> Self {
        Self {
            comparisons: Vec::new(),
        }
    }

    /// Add a comparison result
    pub fn add_comparison(&mut self, comparison: ComparisonResult) {
        self.comparisons.push(comparison);
    }

    /// Add multiple comparison results
    pub fn add_comparisons(&mut self, comparisons: Vec<ComparisonResult>) {
        self.comparisons.extend(comparisons);
    }

    /// Build the report
    pub fn build(&self) -> ComparisonReport {
        let comparisons: Vec<ComparisonExport> = self
            .comparisons
            .iter()
            .map(|c| self.export_comparison(c))
            .collect();

        let stats = self.calculate_statistics();

        ComparisonReport {
            metadata: ReportMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                version: "1.0.0".to_string(),
                total_comparisons: comparisons.len(),
            },
            comparisons,
            statistics: stats,
        }
    }

    /// Export a single comparison
    fn export_comparison(&self, comparison: &ComparisonResult) -> ComparisonExport {
        let models: Vec<ModelExport> = comparison
            .results
            .iter()
            .map(|r| ModelExport {
                name: r.model.name.clone(),
                base_model: r.model.base.clone(),
                lora: r.model.lora.clone(),
                lora_strength: r.model.lora_strength,
                generation_time_s: r.duration_s,
                image_path: r
                    .image_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
            })
            .collect();

        let winner = comparison.winner().map(|w| w.name.clone());

        ComparisonExport {
            comparison_id: comparison.comparison_id.clone(),
            prompt: comparison.params.prompt.clone(),
            seed: comparison.params.seed,
            models,
            winner,
            notes: comparison.notes.clone(),
            completed_at: comparison.completed_at.to_rfc3339(),
        }
    }

    /// Calculate aggregate statistics
    fn calculate_statistics(&self) -> StatisticsExport {
        let total = self.comparisons.len();
        let with_preference = self
            .comparisons
            .iter()
            .filter(|c| c.user_preference.is_some())
            .count();

        let preference_rate = if total > 0 {
            (with_preference as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        // Calculate model wins
        let mut wins: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for comparison in &self.comparisons {
            if let Some(winner) = comparison.winner() {
                *wins.entry(winner.name.clone()).or_insert(0) += 1;
            }
        }

        let model_wins: Vec<ModelWinCount> = wins
            .into_iter()
            .map(|(name, count)| {
                let win_rate = if with_preference > 0 {
                    (count as f32 / with_preference as f32) * 100.0
                } else {
                    0.0
                };
                ModelWinCount {
                    model_name: name,
                    wins: count,
                    win_rate,
                }
            })
            .collect();

        // Calculate average generation time
        let mut total_time = 0.0f32;
        let mut time_count = 0;
        for comparison in &self.comparisons {
            for result in &comparison.results {
                if let Some(time) = result.duration_s {
                    total_time += time;
                    time_count += 1;
                }
            }
        }

        let avg_time = if time_count > 0 {
            Some(total_time / time_count as f32)
        } else {
            None
        };

        StatisticsExport {
            total_comparisons: total,
            comparisons_with_preference: with_preference,
            preference_rate,
            model_wins,
            avg_generation_time_s: avg_time,
        }
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Export Functions
// ============================================================================

/// Export report to JSON file
pub fn export_json<P: AsRef<Path>>(report: &ComparisonReport, path: P) -> Result<()> {
    let json =
        serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")?;

    let mut file = File::create(path.as_ref()).context("Failed to create JSON file")?;

    file.write_all(json.as_bytes())
        .context("Failed to write JSON file")?;

    Ok(())
}

/// Export report to CSV file
pub fn export_csv<P: AsRef<Path>>(report: &ComparisonReport, path: P) -> Result<()> {
    let mut file = File::create(path.as_ref()).context("Failed to create CSV file")?;

    // Write header
    writeln!(
        file,
        "comparison_id,prompt,seed,model_name,base_model,lora,lora_strength,generation_time_s,winner,notes,completed_at"
    )?;

    // Write rows
    for comparison in &report.comparisons {
        for model in &comparison.models {
            let is_winner = comparison.winner.as_ref() == Some(&model.name);

            writeln!(
                file,
                "\"{}\",\"{}\",{},\"{}\",\"{}\",\"{}\",{},{},{},\"{}\",\"{}\"",
                comparison.comparison_id,
                escape_csv(&comparison.prompt),
                comparison.seed,
                escape_csv(&model.name),
                escape_csv(&model.base_model),
                model.lora.as_deref().unwrap_or(""),
                model.lora_strength,
                model.generation_time_s.unwrap_or(0.0),
                if is_winner { "1" } else { "0" },
                comparison.notes.as_deref().unwrap_or(""),
                comparison.completed_at,
            )?;
        }
    }

    Ok(())
}

/// Export statistics to separate file
pub fn export_statistics_csv<P: AsRef<Path>>(stats: &StatisticsExport, path: P) -> Result<()> {
    let mut file = File::create(path.as_ref()).context("Failed to create statistics CSV file")?;

    // Write header
    writeln!(file, "model_name,wins,win_rate")?;

    // Write model win statistics
    for model_win in &stats.model_wins {
        writeln!(
            file,
            "\"{}\",{},{:.2}",
            escape_csv(&model_win.model_name),
            model_win.wins,
            model_win.win_rate
        )?;
    }

    Ok(())
}

/// Helper to escape CSV values
fn escape_csv(s: &str) -> String {
    s.replace('"', "\"\"")
}

// ============================================================================
// Training Validation Report
// ============================================================================

/// Generate a training validation report
///
/// This compares pre-trained vs custom LoRA to validate training improvements
pub fn generate_training_validation_report(
    comparisons: &[ComparisonResult],
    base_model_name: &str,
    lora_model_name: &str,
) -> TrainingValidationReport {
    let mut base_wins = 0;
    let mut lora_wins = 0;
    let mut no_preference = 0;

    for comparison in comparisons {
        if let Some(winner) = comparison.winner() {
            if winner.name.contains(base_model_name) {
                base_wins += 1;
            } else if winner.name.contains(lora_model_name) {
                lora_wins += 1;
            }
        } else {
            no_preference += 1;
        }
    }

    let total = comparisons.len();
    let lora_win_rate = if total > 0 {
        (lora_wins as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    let conclusion = if lora_win_rate > 60.0 {
        "Training significantly improved quality".to_string()
    } else if lora_win_rate > 40.0 {
        "Training showed some improvement".to_string()
    } else {
        "Training may need adjustment".to_string()
    };

    TrainingValidationReport {
        base_model: base_model_name.to_string(),
        lora_model: lora_model_name.to_string(),
        total_comparisons: total,
        base_wins,
        lora_wins,
        no_preference,
        lora_win_rate,
        conclusion,
    }
}

/// Training validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingValidationReport {
    pub base_model: String,
    pub lora_model: String,
    pub total_comparisons: usize,
    pub base_wins: usize,
    pub lora_wins: usize,
    pub no_preference: usize,
    pub lora_win_rate: f32,
    pub conclusion: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comparison::{GenerationParams, ModelConfig, ModelResult, ModelResultStatus};

    fn create_test_comparison(winner_idx: Option<usize>) -> ComparisonResult {
        ComparisonResult {
            comparison_id: "test-001".to_string(),
            params: GenerationParams {
                prompt: "16-bit knight sprite".to_string(),
                negative_prompt: None,
                seed: 42,
                width: 1024,
                height: 1024,
                steps: 30,
                cfg_scale: 7.5,
            },
            results: vec![
                ModelResult {
                    model: ModelConfig {
                        name: "Base SDXL".to_string(),
                        base: "sdxl_base.safetensors".to_string(),
                        lora: None,
                        lora_strength: 1.0,
                    },
                    job_id: "job-1".to_string(),
                    image_path: Some("/tmp/img1.png".into()),
                    duration_s: Some(3.5),
                    status: ModelResultStatus::Complete,
                    error: None,
                },
                ModelResult {
                    model: ModelConfig {
                        name: "Pixel Art LoRA".to_string(),
                        base: "sdxl_base.safetensors".to_string(),
                        lora: Some("pixel_art.safetensors".to_string()),
                        lora_strength: 0.8,
                    },
                    job_id: "job-2".to_string(),
                    image_path: Some("/tmp/img2.png".into()),
                    duration_s: Some(3.8),
                    status: ModelResultStatus::Complete,
                    error: None,
                },
            ],
            user_preference: winner_idx,
            notes: Some("LoRA version has better pixel art style".to_string()),
            completed_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_report_builder() {
        let mut builder = ReportBuilder::new();
        builder.add_comparison(create_test_comparison(Some(1)));
        builder.add_comparison(create_test_comparison(Some(1)));
        builder.add_comparison(create_test_comparison(None));

        let report = builder.build();

        assert_eq!(report.comparisons.len(), 3);
        assert_eq!(report.statistics.total_comparisons, 3);
        assert_eq!(report.statistics.comparisons_with_preference, 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut builder = ReportBuilder::new();
        builder.add_comparison(create_test_comparison(Some(1))); // Pixel Art wins
        builder.add_comparison(create_test_comparison(Some(1))); // Pixel Art wins
        builder.add_comparison(create_test_comparison(Some(0))); // Base wins

        let report = builder.build();
        let stats = &report.statistics;

        assert_eq!(stats.comparisons_with_preference, 3);
        assert_eq!(stats.model_wins.len(), 2); // 2 unique models

        // Find Pixel Art LoRA wins
        let pixel_art_wins = stats
            .model_wins
            .iter()
            .find(|m| m.model_name == "Pixel Art LoRA")
            .unwrap();

        assert_eq!(pixel_art_wins.wins, 2);
        assert!((pixel_art_wins.win_rate - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_training_validation_report() {
        let comparisons = vec![
            create_test_comparison(Some(1)), // LoRA wins
            create_test_comparison(Some(1)), // LoRA wins
            create_test_comparison(Some(0)), // Base wins
        ];

        let validation =
            generate_training_validation_report(&comparisons, "Base SDXL", "Pixel Art LoRA");

        assert_eq!(validation.lora_wins, 2);
        assert_eq!(validation.base_wins, 1);
        assert!((validation.lora_win_rate - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_export_json() {
        let mut builder = ReportBuilder::new();
        builder.add_comparison(create_test_comparison(Some(1)));

        let report = builder.build();

        let temp_path = "/tmp/test_report.json";
        export_json(&report, temp_path).expect("Export failed");

        // Verify file exists
        assert!(std::path::Path::new(temp_path).exists());
    }

    #[test]
    fn test_export_csv() {
        let mut builder = ReportBuilder::new();
        builder.add_comparison(create_test_comparison(Some(1)));

        let report = builder.build();

        let temp_path = "/tmp/test_report.csv";
        export_csv(&report, temp_path).expect("Export failed");

        // Verify file exists
        assert!(std::path::Path::new(temp_path).exists());
    }
}
